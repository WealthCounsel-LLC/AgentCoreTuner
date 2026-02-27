//! AWS CLI shelling-out functions.
//! All functions are blocking — call them from background threads only.

use crate::types::{
    Agent, AgentRuntimeDetail, CreateAgentRuntimeParams, CreateMemoryParams, InvokeParams,
    MemoryInfo, ModelInfo,
};
use std::process::Command;

// ── CliRunner trait ───────────────────────────────────────────────────────────

/// Output from a CLI command.
#[derive(Clone)]
pub struct CliOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub success: bool,
}

/// Abstracts over running a CLI command so that tests can inject a fake.
pub trait CliRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CliOutput, String>;
}

/// Production implementation — shells out to the real AWS CLI.
pub struct RealCli;

impl CliRunner for RealCli {
    fn run(&self, program: &str, args: &[&str]) -> Result<CliOutput, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run {program}: {e}"))?;
        Ok(CliOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.status.success(),
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn check_output(out: &CliOutput, context: &str) -> Result<String, String> {
    if !out.success {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{context} failed")
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns a list of AWS profile names from `~/.aws/config`.
pub fn get_profiles() -> Vec<String> {
    let cli = RealCli;
    match cli.run("aws", &["configure", "list-profiles", "--output", "text"]) {
        Ok(o) if o.success => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => vec!["default".to_string()],
    }
}

/// Check whether the `aws` CLI binary is available on PATH.
/// Returns `Ok(version_string)` if found, or `Err(reason)` if not.
pub fn check_aws_cli_available() -> Result<String, String> {
    match Command::new("aws").arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            // aws v2 prints to stdout, v1 prints to stderr
            Ok(if stdout.is_empty() { stderr } else { stdout })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("AWS CLI not found on PATH".to_string())
        }
        Err(e) => Err(format!("Failed to check AWS CLI: {e}")),
    }
}

/// Returns the IAM ARN of the currently authenticated caller, or an error on failure.
pub fn get_caller_identity(profile: &str, region: &str) -> Result<String, String> {
    let cli = RealCli;
    match cli.run(
        "aws",
        &[
            "sts",
            "get-caller-identity",
            "--query",
            "Arn",
            "--output",
            "text",
            "--profile",
            profile,
            "--region",
            region,
        ],
    ) {
        Ok(o) if o.success => Ok(String::from_utf8_lossy(&o.stdout).trim().to_string()),
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                "sts get-caller-identity failed".to_string()
            } else {
                stderr
            })
        }
        Err(e) => Err(e),
    }
}

/// Lists AgentCore Runtime agents for the given profile and region.
pub fn list_agents(profile: &str, region: &str) -> Result<Vec<Agent>, String> {
    list_agents_with(&RealCli, profile, region)
}

pub fn list_agents_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<Agent>, String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "list-agent-runtimes",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;

    let stdout = check_output(&out, "bedrock-agentcore-control list-agent-runtimes")?;
    parse_agents(&stdout)
}

pub fn parse_agents(json: &str) -> Result<Vec<Agent>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse response: {e}"))?;

    let arr = parsed
        .get("agentRuntimes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(arr
        .iter()
        .filter_map(|v| {
            let id = v.get("agentRuntimeId")?.as_str()?.to_string();
            let name = v
                .get("agentRuntimeName")
                .and_then(|s| s.as_str())
                .unwrap_or("(unnamed)")
                .to_string();
            let arn = v
                .get("agentRuntimeArn")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let version = v
                .get("agentRuntimeVersion")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let status = v
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let last_updated = v
                .get("lastUpdatedAt")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            Some(Agent { id, name, arn, version, status, last_updated })
        })
        .collect())
}

/// Lists all versions of a specific AgentCore Runtime, returning version strings
/// in descending order (newest first).
pub fn list_agent_runtime_versions(
    profile: &str,
    region: &str,
    agent_runtime_id: &str,
) -> Result<Vec<String>, String> {
    let out = RealCli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "list-agent-runtime-versions",
            "--agent-runtime-id",
            agent_runtime_id,
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;

    let stdout = check_output(&out, "list-agent-runtime-versions")?;
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse response: {e}"))?;

    let arr = parsed
        .get("agentRuntimes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut versions: Vec<String> = arr
        .iter()
        .filter_map(|v| {
            v.get("agentRuntimeVersion")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    // Sort descending numerically.
    versions.sort_by(|a, b| {
        b.parse::<u32>()
            .unwrap_or(0)
            .cmp(&a.parse::<u32>().unwrap_or(0))
    });

    Ok(versions)
}

/// Returns the CloudWatch log group for an AgentCore Runtime agent.
pub fn agent_log_group(agent_id: &str) -> String {
    format!("/aws/bedrock-agentcore/runtimes/{agent_id}-DEFAULT")
}

/// Lists all CloudWatch log groups, following pagination (up to 500 groups).
pub fn list_log_groups(profile: &str, region: &str) -> Result<Vec<String>, String> {
    list_log_groups_with(&RealCli, profile, region)
}

pub fn list_log_groups_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<String>, String> {
    let mut log_groups: Vec<String> = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let mut base_args: Vec<String> = vec![
            "logs".to_string(),
            "describe-log-groups".to_string(),
            "--profile".to_string(),
            profile.to_string(),
            "--region".to_string(),
            region.to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];

        if let Some(ref token) = next_token {
            base_args.push("--next-token".to_string());
            base_args.push(token.clone());
        }

        let str_args: Vec<&str> = base_args.iter().map(|s| s.as_str()).collect();
        let out = cli.run("aws", &str_args)?;
        let stdout = check_output(&out, "aws logs describe-log-groups")?;
        let (groups, token) = parse_log_groups(&stdout)?;
        log_groups.extend(groups);

        next_token = token;
        if next_token.is_none() || log_groups.len() >= 500 {
            break;
        }
    }

    log_groups.sort();
    Ok(log_groups)
}

/// Parses `aws logs describe-log-groups` JSON response.
///
/// Returns `(log_group_names, optional_next_token)`.
pub fn parse_log_groups(json: &str) -> Result<(Vec<String>, Option<String>), String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse log groups: {e}"))?;

    let groups = parsed
        .get("logGroups")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|g| g.get("logGroupName")?.as_str().map(String::from))
        .collect();

    let next_token = parsed
        .get("nextToken")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok((groups, next_token))
}

/// Lists AgentCore memories for the given profile and region.
pub fn list_memories(profile: &str, region: &str) -> Result<Vec<MemoryInfo>, String> {
    list_memories_with(&RealCli, profile, region)
}

pub fn list_memories_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<MemoryInfo>, String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "list-memories",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;

    let stdout = check_output(&out, "bedrock-agentcore-control list-memories")?;
    let mut memories = parse_memories(&stdout)?;

    // The list-memories response only returns IDs; enrich with names via get-memory.
    for mem in &mut memories {
        if mem.name.is_empty() {
            if let Ok(detail) = get_memory_with(cli, profile, region, &mem.id) {
                mem.name = detail.name;
            }
        }
    }

    Ok(memories)
}

pub fn parse_memories(json: &str) -> Result<Vec<MemoryInfo>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse memory list: {e}"))?;

    let arr = parsed
        .get("memories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(arr
        .iter()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let name = v
                .get("name")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            Some(MemoryInfo { id, name })
        })
        .collect())
}

pub fn get_memory(profile: &str, region: &str, memory_id: &str) -> Result<MemoryInfo, String> {
    get_memory_with(&RealCli, profile, region, memory_id)
}

pub fn get_memory_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
    memory_id: &str,
) -> Result<MemoryInfo, String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "get-memory",
            "--memory-id",
            memory_id,
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;

    let stdout = check_output(&out, "bedrock-agentcore-control get-memory")?;
    parse_get_memory_response(&stdout)
}

pub fn parse_get_memory_response(json: &str) -> Result<MemoryInfo, String> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse get-memory response: {e}"))?;
    let mem = v.get("memory").unwrap_or(&v);
    let id = mem
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id in get-memory response")?
        .to_string();
    let name = mem
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(MemoryInfo { id, name })
}

/// Validates an AgentCore memory name: `[a-zA-Z][a-zA-Z0-9_]{0,47}`.
pub fn is_valid_memory_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 48 {
        return false;
    }
    let bytes = name.as_bytes();
    bytes[0].is_ascii_alphabetic()
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

/// Creates a new AgentCore memory resource.
pub fn create_memory(params: &CreateMemoryParams) -> Result<MemoryInfo, String> {
    create_memory_with(&RealCli, params)
}

pub fn create_memory_with(
    cli: &dyn CliRunner,
    params: &CreateMemoryParams,
) -> Result<MemoryInfo, String> {
    if !is_valid_memory_name(&params.name) {
        return Err(
            "Invalid memory name. Must start with a letter and contain only letters, digits, or underscores (max 48 chars).".to_string(),
        );
    }

    let expiry_str = params.event_expiry_duration.to_string();

    let mut args = vec![
        "bedrock-agentcore-control",
        "create-memory",
        "--name",
        &params.name,
        "--event-expiry-duration",
        &expiry_str,
        "--profile",
        &params.profile,
        "--region",
        &params.region,
        "--output",
        "json",
    ];

    if !params.description.is_empty() {
        args.extend_from_slice(&["--description", &params.description]);
    }

    let strategies_json: String;
    if !params.strategies.is_empty() {
        strategies_json = build_strategies_json(&params.strategies);
        args.extend_from_slice(&["--memory-strategies", &strategies_json]);
    }

    let out = cli.run("aws", &args)?;
    let stdout = check_output(&out, "bedrock-agentcore-control create-memory")?;
    parse_create_memory_response(&stdout)
}

fn build_strategies_json(strategies: &[crate::types::MemoryStrategy]) -> String {
    let items: Vec<serde_json::Value> = strategies
        .iter()
        .map(|s| {
            // The AWS CLI expects a tagged union: the key is the strategy kind,
            // e.g. "semanticMemoryStrategy", with "name" and "namespaces" inside.
            let (key, default_name, default_ns) = match s.strategy_type.as_str() {
                "SEMANTIC_MEMORY" => (
                    "semanticMemoryStrategy",
                    "SemanticExtractor",
                    "/{actorId}/semantic/",
                ),
                "SUMMARY_MEMORY" => (
                    "summaryMemoryStrategy",
                    "SessionSummarizer",
                    "/{actorId}/{sessionId}/summaries/",
                ),
                "USER_PREFERENCE" => (
                    "userPreferenceMemoryStrategy",
                    "PreferenceLearner",
                    "/{actorId}/preferences/",
                ),
                "EPISODIC_MEMORY" => (
                    "episodicMemoryStrategy",
                    "EpisodicRecorder",
                    "/{actorId}/{sessionId}/episodes/",
                ),
                other => (other, "Strategy", "/default/"),
            };

            let name = if s.name.is_empty() {
                default_name.to_string()
            } else {
                s.name.clone()
            };
            let ns = if s.namespace.is_empty() {
                default_ns.to_string()
            } else {
                s.namespace.clone()
            };

            let mut inner = serde_json::Map::new();
            inner.insert("name".into(), serde_json::Value::String(name));
            inner.insert(
                "namespaces".into(),
                serde_json::Value::Array(vec![serde_json::Value::String(ns)]),
            );

            let mut outer = serde_json::Map::new();
            outer.insert(key.into(), serde_json::Value::Object(inner));
            serde_json::Value::Object(outer)
        })
        .collect();
    serde_json::Value::Array(items).to_string()
}

pub fn parse_create_memory_response(json: &str) -> Result<MemoryInfo, String> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse create-memory response: {e}"))?;
    // Response is wrapped in a "memory" object: {"memory": {"id": "...", "name": "..."}}
    let mem = v.get("memory").unwrap_or(&v);
    let id = mem
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id in create-memory response")?
        .to_string();
    let name = mem
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(MemoryInfo { id, name })
}

/// Deletes an AgentCore memory resource.
pub fn delete_memory(profile: &str, region: &str, memory_id: &str) -> Result<(), String> {
    delete_memory_with(&RealCli, profile, region, memory_id)
}

pub fn delete_memory_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
    memory_id: &str,
) -> Result<(), String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "delete-memory",
            "--memory-id",
            memory_id,
            "--profile",
            profile,
            "--region",
            region,
        ],
    )?;
    check_output(&out, "bedrock-agentcore-control delete-memory")?;
    Ok(())
}

// ── Agent Runtime Management ─────────────────────────────────────────────────

/// Retrieves full configuration details for an AgentCore Runtime.
pub fn get_agent_runtime(
    profile: &str,
    region: &str,
    agent_runtime_id: &str,
) -> Result<AgentRuntimeDetail, String> {
    get_agent_runtime_with(&RealCli, profile, region, agent_runtime_id)
}

pub fn get_agent_runtime_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
    agent_runtime_id: &str,
) -> Result<AgentRuntimeDetail, String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "get-agent-runtime",
            "--agent-runtime-id",
            agent_runtime_id,
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;
    let stdout = check_output(&out, "bedrock-agentcore-control get-agent-runtime")?;
    parse_agent_runtime_detail(&stdout)
}

pub fn parse_agent_runtime_detail(json: &str) -> Result<AgentRuntimeDetail, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse agent runtime: {e}"))?;

    let id = v
        .get("agentRuntimeId")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let name = v
        .get("agentRuntimeName")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let arn = v
        .get("agentRuntimeArn")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let version = v
        .get("agentRuntimeVersion")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let status = v
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let description = v
        .get("description")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let role_arn = v
        .get("roleArn")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let network_mode = v
        .pointer("/networkConfiguration/networkMode")
        .and_then(|s| s.as_str())
        .unwrap_or("PUBLIC")
        .to_string();
    let idle_session_timeout = v
        .pointer("/lifecycleConfiguration/idleRuntimeSessionTimeout")
        .and_then(|n| n.as_u64())
        .unwrap_or(900) as u32;
    let max_lifetime = v
        .pointer("/lifecycleConfiguration/maxLifetime")
        .and_then(|n| n.as_u64())
        .unwrap_or(28800) as u32;
    let protocol = v
        .pointer("/protocolConfiguration/serverProtocol")
        .and_then(|s| s.as_str())
        .unwrap_or("HTTP")
        .to_string();
    let code_s3_bucket = v
        .pointer("/agentRuntimeArtifact/codeConfiguration/code/s3/bucket")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let code_s3_prefix = v
        .pointer("/agentRuntimeArtifact/codeConfiguration/code/s3/prefix")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let code_runtime = v
        .pointer("/agentRuntimeArtifact/codeConfiguration/runtime")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let code_entry_point = v
        .pointer("/agentRuntimeArtifact/codeConfiguration/entryPoint")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    let environment_variables = v
        .get("environmentVariables")
        .and_then(|obj| obj.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, val)| Some((k.clone(), val.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let created_at = v
        .get("createdAt")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let last_updated_at = v
        .get("lastUpdatedAt")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    Ok(AgentRuntimeDetail {
        id,
        name,
        arn,
        version,
        status,
        description,
        role_arn,
        network_mode,
        idle_session_timeout,
        max_lifetime,
        protocol,
        code_s3_bucket,
        code_s3_prefix,
        code_runtime,
        code_entry_point,
        environment_variables,
        created_at,
        last_updated_at,
    })
}

/// Creates a new AgentCore Runtime.
pub fn create_agent_runtime(params: &CreateAgentRuntimeParams) -> Result<AgentRuntimeDetail, String> {
    create_agent_runtime_with(&RealCli, params)
}

pub fn create_agent_runtime_with(
    cli: &dyn CliRunner,
    params: &CreateAgentRuntimeParams,
) -> Result<AgentRuntimeDetail, String> {
    let artifact = serde_json::json!({
        "codeConfiguration": {
            "code": {
                "s3": {
                    "bucket": params.code_s3_bucket,
                    "prefix": params.code_s3_prefix,
                }
            },
            "runtime": params.code_runtime,
            "entryPoint": [params.code_entry_point],
        }
    });
    let network = serde_json::json!({
        "networkMode": params.network_mode,
    });
    let lifecycle = serde_json::json!({
        "idleRuntimeSessionTimeout": params.idle_session_timeout,
        "maxLifetime": params.max_lifetime,
    });

    let artifact_str = artifact.to_string();
    let network_str = network.to_string();
    let lifecycle_str = lifecycle.to_string();

    let mut args = vec![
        "bedrock-agentcore-control",
        "create-agent-runtime",
        "--agent-runtime-name",
        &params.name,
        "--agent-runtime-artifact",
        &artifact_str,
        "--role-arn",
        &params.role_arn,
        "--network-configuration",
        &network_str,
        "--lifecycle-configuration",
        &lifecycle_str,
        "--profile",
        &params.profile,
        "--region",
        &params.region,
        "--output",
        "json",
    ];

    let desc_val;
    if !params.description.is_empty() {
        desc_val = params.description.clone();
        args.insert(4, "--description");
        args.insert(5, &desc_val);
    }

    let protocol_str;
    if !params.protocol.is_empty() && params.protocol != "HTTP" {
        protocol_str = serde_json::json!({"serverProtocol": params.protocol}).to_string();
        args.push("--protocol-configuration");
        args.push(&protocol_str);
    }

    let env_str;
    if !params.environment_variables.is_empty() {
        env_str = serde_json::to_string(&params.environment_variables)
            .map_err(|e| format!("Failed to serialize env vars: {e}"))?;
        args.push("--environment-variables");
        args.push(&env_str);
    }

    let out = cli.run("aws", &args)?;
    let stdout = check_output(&out, "bedrock-agentcore-control create-agent-runtime")?;
    parse_agent_runtime_detail(&stdout)
}

/// Updates an existing AgentCore Runtime (creates a new version).
pub fn update_agent_runtime(params: &CreateAgentRuntimeParams) -> Result<AgentRuntimeDetail, String> {
    update_agent_runtime_with(&RealCli, params)
}

pub fn update_agent_runtime_with(
    cli: &dyn CliRunner,
    params: &CreateAgentRuntimeParams,
) -> Result<AgentRuntimeDetail, String> {
    let artifact = serde_json::json!({
        "codeConfiguration": {
            "code": {
                "s3": {
                    "bucket": params.code_s3_bucket,
                    "prefix": params.code_s3_prefix,
                }
            },
            "runtime": params.code_runtime,
            "entryPoint": [params.code_entry_point],
        }
    });
    let network = serde_json::json!({
        "networkMode": params.network_mode,
    });
    let lifecycle = serde_json::json!({
        "idleRuntimeSessionTimeout": params.idle_session_timeout,
        "maxLifetime": params.max_lifetime,
    });

    let artifact_str = artifact.to_string();
    let network_str = network.to_string();
    let lifecycle_str = lifecycle.to_string();

    let mut args = vec![
        "bedrock-agentcore-control",
        "update-agent-runtime",
        "--agent-runtime-id",
        &params.agent_runtime_id,
        "--agent-runtime-artifact",
        &artifact_str,
        "--role-arn",
        &params.role_arn,
        "--network-configuration",
        &network_str,
        "--lifecycle-configuration",
        &lifecycle_str,
        "--profile",
        &params.profile,
        "--region",
        &params.region,
        "--output",
        "json",
    ];

    let desc_val;
    if !params.description.is_empty() {
        desc_val = params.description.clone();
        args.insert(4, "--description");
        args.insert(5, &desc_val);
    }

    let protocol_str;
    if !params.protocol.is_empty() {
        protocol_str = serde_json::json!({"serverProtocol": params.protocol}).to_string();
        args.push("--protocol-configuration");
        args.push(&protocol_str);
    }

    let env_str;
    if !params.environment_variables.is_empty() {
        env_str = serde_json::to_string(&params.environment_variables)
            .map_err(|e| format!("Failed to serialize env vars: {e}"))?;
        args.push("--environment-variables");
        args.push(&env_str);
    }

    let out = cli.run("aws", &args)?;
    let stdout = check_output(&out, "bedrock-agentcore-control update-agent-runtime")?;
    parse_agent_runtime_detail(&stdout)
}

/// Deletes an AgentCore Runtime.
pub fn delete_agent_runtime(
    profile: &str,
    region: &str,
    agent_runtime_id: &str,
) -> Result<(), String> {
    delete_agent_runtime_with(&RealCli, profile, region, agent_runtime_id)
}

pub fn delete_agent_runtime_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
    agent_runtime_id: &str,
) -> Result<(), String> {
    let out = cli.run(
        "aws",
        &[
            "bedrock-agentcore-control",
            "delete-agent-runtime",
            "--agent-runtime-id",
            agent_runtime_id,
            "--profile",
            profile,
            "--region",
            region,
        ],
    )?;
    check_output(&out, "bedrock-agentcore-control delete-agent-runtime")?;
    Ok(())
}

// ── Deploy Agent Bundle ──────────────────────────────────────────────────────

/// Builds the agent code bundle and uploads it to S3.
///
/// Runs `agent-cdk/deploy.py` which:
/// 1. Vendors deps with `uv pip install --target` for ARM64 / Python 3.13
/// 2. Zips main.py + deps
/// 3. Uploads to the specified S3 bucket/prefix
///
/// When `runtime_id` is non-empty, deploy.py also calls UpdateAgentRuntime.
/// When empty, it only builds and uploads (for pre-create).
pub fn deploy_agent_bundle(
    profile: &str,
    region: &str,
    bucket: &str,
    runtime_id: &str,
    role_arn: &str,
) -> Result<String, String> {
    // Find the deploy script relative to the executable / project root.
    let deploy_script = find_deploy_script()?;

    let mut args = vec![
        deploy_script.to_string_lossy().to_string(),
        "--profile".to_string(),
        profile.to_string(),
        "--region".to_string(),
        region.to_string(),
        "--bucket".to_string(),
        bucket.to_string(),
    ];

    if !runtime_id.is_empty() {
        args.push("--runtime-id".to_string());
        args.push(runtime_id.to_string());
    } else {
        args.push("--upload-only".to_string());
    }
    if !role_arn.is_empty() {
        args.push("--role-arn".to_string());
        args.push(role_arn.to_string());
    }

    // Use the venv Python if available, otherwise fall back to system python3.
    let venv_python = deploy_script
        .parent()
        .map(|d| d.join(".venv/bin/python"))
        .filter(|p| p.exists());
    let python = venv_python
        .as_deref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "python3".to_string());

    let output = Command::new(&python)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run deploy.py: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let msg = if stderr.is_empty() { &stdout } else { &stderr };
        return Err(format!("Deploy failed: {}", msg.trim()));
    }

    Ok(stdout)
}

/// Locates the deploy.py script.
fn find_deploy_script() -> Result<std::path::PathBuf, String> {
    // Try relative to CWD first (development), then relative to the executable.
    let candidates = [
        std::path::PathBuf::from("agent-cdk/deploy.py"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("agent-cdk/deploy.py")))
            .unwrap_or_default(),
    ];
    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    Err("Could not find agent-cdk/deploy.py. Run from the project root.".to_string())
}

// ── S3 Buckets ──────────────────────────────────────────────────────────────

/// Lists S3 bucket names, optionally filtered by a prefix.
pub fn list_s3_buckets(profile: &str, region: &str) -> Result<Vec<String>, String> {
    list_s3_buckets_with(&RealCli, profile, region)
}

pub fn list_s3_buckets_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<String>, String> {
    let out = cli.run(
        "aws",
        &[
            "s3api",
            "list-buckets",
            "--query",
            "Buckets[].Name",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;
    let stdout = check_output(&out, "s3api list-buckets")?;
    parse_s3_buckets(&stdout)
}

pub fn parse_s3_buckets(json: &str) -> Result<Vec<String>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse S3 buckets: {e}"))?;

    let arr = parsed
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut buckets: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    buckets.sort();
    Ok(buckets)
}

// ── IAM Roles ───────────────────────────────────────────────────────────────

/// Lists IAM role ARNs whose names contain "AgentCore" or "Bedrock" (case-insensitive).
pub fn list_agentcore_roles(profile: &str, region: &str) -> Result<Vec<String>, String> {
    list_agentcore_roles_with(&RealCli, profile, region)
}

pub fn list_agentcore_roles_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<String>, String> {
    let out = cli.run(
        "aws",
        &[
            "iam",
            "list-roles",
            "--query",
            "Roles[].Arn",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;
    let stdout = check_output(&out, "iam list-roles")?;
    parse_agentcore_roles(&stdout)
}

pub fn parse_agentcore_roles(json: &str) -> Result<Vec<String>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse IAM roles: {e}"))?;

    let arr = parsed
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut roles: Vec<String> = arr
        .iter()
        .filter_map(|v| {
            let arn = v.as_str()?;
            let lower = arn.to_lowercase();
            if lower.contains("agentcore") || lower.contains("bedrock") {
                // Skip service-linked roles (they can't be used directly).
                if lower.contains("aws-service-role/") {
                    return None;
                }
                Some(arn.to_string())
            } else {
                None
            }
        })
        .collect();
    roles.sort();
    Ok(roles)
}

/// Lists Bedrock models: foundation models + inference profiles merged and deduped.
///
/// `list-foundation-models` returns base model IDs. Marketplace/cross-region
/// models you've enabled (e.g. `us.anthropic.claude-sonnet-4-5-...`) only appear
/// in `list-inference-profiles`. Both sources are fetched and merged.
pub fn list_models(profile: &str, region: &str) -> Result<Vec<ModelInfo>, String> {
    list_models_with(&RealCli, profile, region)
}

pub fn list_models_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
) -> Result<Vec<ModelInfo>, String> {
    let base_out = cli.run(
        "aws",
        &[
            "bedrock",
            "list-foundation-models",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;
    let base_json = check_output(&base_out, "bedrock list-foundation-models")?;
    let mut models = parse_models(&base_json)?;

    // Also fetch inference profiles (cross-region + marketplace models).
    // Failures here are non-fatal — we still return the base models.
    let profile_out = cli.run(
        "aws",
        &[
            "bedrock",
            "list-inference-profiles",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    );
    if let Ok(out) = profile_out {
        if let Ok(json) = check_output(&out, "bedrock list-inference-profiles") {
            if let Ok(profiles) = parse_inference_profiles(&json) {
                let existing_ids: std::collections::HashSet<String> =
                    models.iter().map(|m| m.id.clone()).collect();
                for p in profiles {
                    if !existing_ids.contains(&p.id) {
                        models.push(p);
                    }
                }
            }
        }
    }

    models.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(models)
}

pub fn parse_models(json: &str) -> Result<Vec<ModelInfo>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse model list: {e}"))?;

    let models = parsed
        .get("modelSummaries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter(|m| {
            let text_in = m
                .get("inputModalities")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().any(|x| x.as_str() == Some("TEXT")))
                .unwrap_or(false);
            let text_out = m
                .get("outputModalities")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().any(|x| x.as_str() == Some("TEXT")))
                .unwrap_or(false);
            let on_demand = m
                .get("inferenceTypesSupported")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().any(|x| x.as_str() == Some("ON_DEMAND")))
                .unwrap_or(false);
            text_in && text_out && on_demand
        })
        .filter_map(|m| {
            let id = m.get("modelId")?.as_str()?.to_string();
            let name = m.get("modelName").and_then(|v| v.as_str()).unwrap_or(&id);
            let provider = m.get("providerName").and_then(|v| v.as_str()).unwrap_or("");
            let label = if provider.is_empty() {
                name.to_string()
            } else {
                format!("{provider} — {name}")
            };
            Some(ModelInfo { id, label })
        })
        .collect();

    Ok(models)
}

/// Parses `bedrock list-inference-profiles` response into ModelInfo entries.
///
/// Includes both SYSTEM_DEFINED (cross-region) and APPLICATION (user-created) profiles.
pub fn parse_inference_profiles(json: &str) -> Result<Vec<ModelInfo>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse inference profiles: {e}"))?;

    let profiles = parsed
        .get("inferenceProfileSummaries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|p| {
            let id = p.get("inferenceProfileId")?.as_str()?.to_string();
            let name = p
                .get("inferenceProfileName")
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
                .to_string();
            let profile_type = p
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("SYSTEM_DEFINED");
            // Tag cross-region profiles so users can tell them apart.
            let label = if profile_type == "SYSTEM_DEFINED" {
                format!("{name} [cross-region]")
            } else {
                name
            };
            Some(ModelInfo { id, label })
        })
        .collect();

    Ok(profiles)
}

/// Invokes an AgentCore Runtime agent.
///
/// Returns `Ok((thinking, response_text, debug_log))`.
pub fn invoke_agent(params: &InvokeParams) -> Result<(String, String, String), String> {
    invoke_agent_with(&RealCli, params)
}

pub fn invoke_agent_with(
    cli: &dyn CliRunner,
    params: &InvokeParams,
) -> Result<(String, String, String), String> {
    if params.agent_arn.is_empty() {
        return Err("No agent selected — fetch agents first.".to_string());
    }

    let session = build_session_id(&params.session_id);
    let payload_json = build_payload(params);
    let payload = encode_payload(&payload_json);

    let mut args = vec![
        "bedrock-agentcore",
        "invoke-agent-runtime",
        "--agent-runtime-arn",
        params.agent_arn.as_str(),
        "--runtime-session-id",
        session.as_str(),
        "--payload",
        payload.as_str(),
        "--profile",
        params.profile.as_str(),
        "--region",
        params.region.as_str(),
    ];

    if !params.qualifier.is_empty() && params.qualifier != "LATEST" {
        args.extend_from_slice(&["--qualifier", params.qualifier.as_str()]);
    }
    if !params.runtime_user_id.is_empty() {
        args.extend_from_slice(&["--runtime-user-id", params.runtime_user_id.as_str()]);
    }
    let outfile = std::env::temp_dir().join("agentcore-response.bin");
    let outfile_str = outfile.to_string_lossy().into_owned();
    args.push(outfile_str.as_str());

    let out = cli.run("aws", &args)?;

    let debug_log = format!(
        "CMD: aws bedrock-agentcore invoke-agent-runtime \\\n  \
         --agent-runtime-arn {arn} \\\n  \
         --runtime-session-id {session} \\\n  \
         --payload '<base64({payload_json})>'{qualifier}{user_id}{memory} \\\n  \
         --profile {profile} --region {region}\n\n\
         STDOUT:\n{stdout}\n\nSTDERR:\n{stderr}",
        arn = params.agent_arn,
        qualifier = if params.qualifier.is_empty() {
            String::new()
        } else {
            format!(" \\\n  --qualifier {}", params.qualifier)
        },
        user_id = if params.runtime_user_id.is_empty() {
            String::new()
        } else {
            format!(" \\\n  --runtime-user-id {}", params.runtime_user_id)
        },
        memory = if params.memory_id.is_empty() {
            String::new()
        } else {
            format!(" \\\n  # memory_id in payload: {}", params.memory_id)
        },
        profile = params.profile,
        region = params.region,
        stdout = String::from_utf8_lossy(&out.stdout).trim(),
        stderr = String::from_utf8_lossy(&out.stderr).trim(),
    );

    if !out.success {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!(
            "invoke-agent-runtime failed: {}",
            if stderr.is_empty() {
                "non-zero exit".to_string()
            } else {
                stderr
            }
        ));
    }

    let raw = std::fs::read_to_string(&outfile)
        .map_err(|e| format!("Failed to read response file: {e}"))?;

    let (thinking, response) = parse_invoke_response(&raw);
    Ok((thinking, response, debug_log))
}

// ── Pure functions (easily unit-tested) ──────────────────────────────────────

/// Ensures session ID meets the 33-char minimum.
pub fn build_session_id(session_id: &str) -> String {
    if session_id.is_empty() {
        format!(
            "session-{:0>25}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    } else if session_id.len() < 33 {
        format!("{session_id:0>33}")
    } else {
        session_id.to_string()
    }
}

/// Builds the payload JSON string from InvokeParams.
///
/// Field precedence (later entries win on collision):
///   base fields (`input`, `model`, `system_prompt`) → `extra_payload`
pub fn build_payload(params: &InvokeParams) -> String {
    let mut map = serde_json::Map::new();

    map.insert("input".into(), serde_json::json!(params.input));

    if !params.model_id.is_empty() {
        map.insert("model".into(), serde_json::json!(params.model_id));
    }
    if !params.system_prompt.is_empty() {
        map.insert("system_prompt".into(), serde_json::json!(params.system_prompt));
    }
    if !params.memory_id.is_empty() {
        map.insert("memory_id".into(), serde_json::json!(params.memory_id));
    }

    // Merge extra fields last — callers can override anything above.
    for (k, v) in &params.extra_payload {
        map.insert(k.clone(), v.clone());
    }

    serde_json::Value::Object(map).to_string()
}

/// Base64-encodes a payload JSON string for the CLI `--payload` flag.
pub fn encode_payload(json: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

/// Parses the agent runtime response body.
///
/// The AWS CLI writes the raw response body to the output file. Strands agents
/// return one of:
///   - A JSON string literal:  `"Hello!\n- item"`
///   - A structured object:    `{"result": {"role": "assistant", "content": [...]}}`
///
/// Returns `(thinking, response_text)`. `thinking` is empty if no thinking blocks.
pub fn parse_invoke_response(raw: &str) -> (String, String) {
    fn normalize(s: &str) -> String {
        s.lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    let raw = raw.trim();

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        // Case 1: plain JSON string — Strands returns str(agent_result)
        if let Some(s) = v.as_str() {
            let text = normalize(s);
            return (
                String::new(),
                if text.is_empty() { "(empty response)".to_string() } else { text },
            );
        }

        // Case 2: structured object with content blocks
        let blocks = v
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array());

        if let Some(blocks) = blocks {
            let thinking = blocks
                .iter()
                .filter_map(|b| b.get("thinking").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n\n");

            let text = blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("");

            return (
                normalize(&thinking),
                {
                    let t = normalize(&text);
                    if t.is_empty() { "(empty response)".to_string() } else { t }
                },
            );
        }
    }

    // Fallback: return raw text as-is (already a plain string, not JSON-quoted)
    let fallback = normalize(raw);
    (
        String::new(),
        if fallback.is_empty() { "(empty response)".to_string() } else { fallback },
    )
}

/// Runs `aws sso login` for the given profile.
pub fn sso_login_cli(profile: &str) -> Result<(), String> {
    let out = RealCli.run("aws", &["sso", "login", "--profile", profile])?;
    if out.success {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(format!("aws sso login failed: {stderr}"))
    }
}

/// Fetches CloudWatch log events for an AgentCore agent.
pub fn get_logs(
    profile: &str,
    region: &str,
    log_group: &str,
    log_stream: &str,
    filter: &str,
) -> Result<(String, String), String> {
    get_logs_with(&RealCli, profile, region, log_group, log_stream, filter)
}

pub fn get_logs_with(
    cli: &dyn CliRunner,
    profile: &str,
    region: &str,
    log_group: &str,
    log_stream: &str,
    filter: &str,
) -> Result<(String, String), String> {
    if log_group.is_empty() {
        return Err(
            "Log group is required. Fetch an agent first to auto-populate, or enter it manually."
                .to_string(),
        );
    }

    // Default to the last 24 hours so we don't scan all of history.
    let start_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .saturating_sub(24 * 60 * 60 * 1000);
    let start_ms_str = start_ms.to_string();

    let mut args = vec![
        "logs",
        "filter-log-events",
        "--log-group-name",
        log_group,
        "--start-time",
        start_ms_str.as_str(),
        "--limit",
        "200",
        "--profile",
        profile,
        "--region",
        region,
        "--output",
        "json",
    ];

    if !log_stream.is_empty() {
        // Treat as prefix — much more useful than exact stream name match.
        args.extend_from_slice(&["--log-stream-name-prefix", log_stream]);
    }
    if !filter.is_empty() {
        args.extend_from_slice(&["--filter-pattern", filter]);
    }

    let out = cli.run("aws", &args)?;
    let stdout = check_output(&out, "aws logs filter-log-events")?;
    parse_logs(&stdout, log_group, log_stream)
}

pub fn parse_logs(
    json: &str,
    log_group: &str,
    log_stream: &str,
) -> Result<(String, String), String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse log response: {e}"))?;

    let events = parsed
        .get("events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let count = events.len();
    let mut lines = Vec::with_capacity(count);

    for event in &events {
        let ts = event
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .map(|ms| {
                let secs = ms / 1000;
                let ms_part = ms % 1000;
                format!("{secs}.{ms_part:03}")
            })
            .unwrap_or_else(|| "?".to_string());

        let msg = event
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim_end_matches('\n');

        let stream = event
            .get("logStreamName")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        lines.push(format!("[{ts}] [{stream}] {msg}"));
    }

    let metrics = format!(
        "{count} event(s)  |  log group: {log_group}{}",
        if !log_stream.is_empty() {
            format!("  |  stream: {log_stream}")
        } else {
            String::new()
        }
    );

    Ok((metrics, lines.join("\n")))
}
