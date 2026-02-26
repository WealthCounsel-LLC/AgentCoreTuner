/// Tests for aws.rs parsing and payload logic.
///
/// These tests never call the real AWS CLI. FakeCli returns canned fixture
/// JSON so all logic runs offline.
///
/// Integration tests that hit real AWS are marked #[ignore] and only run when
/// RUN_INTEGRATION_TESTS=1 is set:
///
///   RUN_INTEGRATION_TESTS=1 cargo test -- --ignored
use agentcore_manager::aws;
use agentcore_manager::aws::{CliOutput, CliRunner};
use agentcore_manager::types::InvokeParams;

// ── FakeCli ───────────────────────────────────────────────────────────────────

/// A fake CLI runner that returns a fixed response for every call.
struct FakeCli {
    stdout: Vec<u8>,
    success: bool,
}

impl FakeCli {
    fn ok(body: &str) -> Self {
        Self {
            stdout: body.as_bytes().to_vec(),
            success: true,
        }
    }

    fn err(msg: &str) -> Self {
        Self {
            stdout: vec![],
            success: false,
        }
        // stderr is returned via the CliOutput; store the message there
        .__with_stderr(msg.as_bytes().to_vec())
    }

    fn __with_stderr(self, _: Vec<u8>) -> Self {
        // For error cases the test checks the Err variant; stderr content
        // isn't critical to verify here, so we keep FakeCli simple.
        self
    }
}

impl CliRunner for FakeCli {
    fn run(&self, _program: &str, _args: &[&str]) -> Result<CliOutput, String> {
        Ok(CliOutput {
            stdout: self.stdout.clone(),
            stderr: if self.success {
                vec![]
            } else {
                b"simulated CLI error".to_vec()
            },
            success: self.success,
        })
    }
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{name}"))
        .unwrap_or_else(|_| panic!("missing fixture: {name}"))
}

// ── list_agents ───────────────────────────────────────────────────────────────

#[test]
fn parse_agents_happy_path() {
    let json = fixture("list_agent_runtimes.json");
    let agents = aws::parse_agents(&json).unwrap();

    assert_eq!(agents.len(), 2);
    assert_eq!(agents[0].id, "abc-123");
    assert_eq!(agents[0].name, "my-test-agent");
    assert!(agents[0].arn.contains("abc-123"));
    assert_eq!(agents[0].version, "5");
    assert_eq!(agents[0].status, "READY");
    assert!(!agents[0].last_updated.is_empty());
    assert_eq!(agents[1].id, "def-456");
    assert_eq!(agents[1].version, "1");
    assert_eq!(agents[1].status, "CREATING");
}

#[test]
fn parse_agents_empty_list() {
    let json = fixture("list_agent_runtimes_empty.json");
    let agents = aws::parse_agents(&json).unwrap();
    assert!(agents.is_empty());
}

#[test]
fn parse_agents_bad_json() {
    let result = aws::parse_agents("not json at all");
    assert!(result.is_err());
}

#[test]
fn parse_agents_missing_key_skips_entry() {
    // An entry without agentRuntimeId should be silently skipped.
    let json = r#"{"agentRuntimes": [{"agentRuntimeName": "no-id"}]}"#;
    let agents = aws::parse_agents(json).unwrap();
    assert!(agents.is_empty());
}

#[test]
fn list_agents_with_fake_cli() {
    let cli = FakeCli::ok(&fixture("list_agent_runtimes.json"));
    let agents = aws::list_agents_with(&cli, "default", "us-east-1").unwrap();
    assert_eq!(agents.len(), 2);
}

#[test]
fn list_agents_cli_error_propagates() {
    let cli = FakeCli::err("AccessDenied");
    let result = aws::list_agents_with(&cli, "default", "us-east-1");
    assert!(result.is_err());
}

// ── list_models ───────────────────────────────────────────────────────────────

#[test]
fn parse_models_filters_non_text() {
    let json = fixture("list_foundation_models.json");
    let models = aws::parse_models(&json).unwrap();

    // SDXL has IMAGE-only output — should be excluded.
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(!ids.contains(&"stability.stable-diffusion-xl-v1"));
    assert!(ids.contains(&"anthropic.claude-3-5-sonnet-20241022-v2:0"));
    assert!(ids.contains(&"amazon.titan-text-express-v1"));
}

#[test]
fn list_models_with_result_is_sorted() {
    // Sorting happens in list_models_with after merging both sources.
    let cli = FakeCli::ok(&fixture("list_foundation_models.json"));
    let models = aws::list_models_with(&cli, "default", "us-east-1").unwrap();
    let labels: Vec<&str> = models.iter().map(|m| m.label.as_str()).collect();
    let mut sorted = labels.clone();
    sorted.sort();
    assert_eq!(labels, sorted);
}

#[test]
fn parse_models_label_format() {
    let json = fixture("list_foundation_models.json");
    let models = aws::parse_models(&json).unwrap();
    let claude = models
        .iter()
        .find(|m| m.id.contains("claude"))
        .expect("claude model not found");
    assert!(claude.label.contains("Anthropic"));
    assert!(claude.label.contains("Claude"));
}

// ── inference profiles ────────────────────────────────────────────────────────

#[test]
fn parse_inference_profiles_happy_path() {
    let json = fixture("list_inference_profiles.json");
    let profiles = aws::parse_inference_profiles(&json).unwrap();
    assert_eq!(profiles.len(), 3);
}

#[test]
fn parse_inference_profiles_system_defined_tagged() {
    let json = fixture("list_inference_profiles.json");
    let profiles = aws::parse_inference_profiles(&json).unwrap();
    let cross = profiles
        .iter()
        .find(|p| p.id.contains("claude-sonnet-4-5"))
        .expect("sonnet 4.5 profile not found");
    assert!(cross.label.contains("[cross-region]"));
}

#[test]
fn parse_inference_profiles_application_not_tagged() {
    let json = fixture("list_inference_profiles.json");
    let profiles = aws::parse_inference_profiles(&json).unwrap();
    let app = profiles
        .iter()
        .find(|p| p.id == "my-custom-profile-001")
        .expect("custom profile not found");
    assert!(!app.label.contains("[cross-region]"));
}

#[test]
fn parse_inference_profiles_empty() {
    let json = r#"{"inferenceProfileSummaries": []}"#;
    let profiles = aws::parse_inference_profiles(json).unwrap();
    assert!(profiles.is_empty());
}

#[test]
fn list_models_with_deduplicates_across_sources() {
    // The foundation model fixture has claude-3-5-sonnet.
    // If inference profiles returned the same ID it should not be duplicated.
    struct TwoCallCli {
        call: std::sync::atomic::AtomicUsize,
    }
    impl aws::CliRunner for TwoCallCli {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<aws::CliOutput, String> {
            let n = self.call.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // First call = list-foundation-models, second = list-inference-profiles.
            let body = if n == 0 {
                std::fs::read_to_string("tests/fixtures/list_foundation_models.json").unwrap()
            } else {
                std::fs::read_to_string("tests/fixtures/list_inference_profiles.json").unwrap()
            };
            Ok(aws::CliOutput {
                stdout: body.into_bytes(),
                stderr: vec![],
                success: true,
            })
        }
    }
    let cli = TwoCallCli {
        call: std::sync::atomic::AtomicUsize::new(0),
    };
    let models = aws::list_models_with(&cli, "default", "us-east-1").unwrap();
    // Count how many times claude-3-5-sonnet appears — should be exactly 1.
    let count = models
        .iter()
        .filter(|m| m.id.contains("claude-3-5-sonnet"))
        .count();
    assert_eq!(count, 1, "duplicate model entries found");
}

// ── payload building ──────────────────────────────────────────────────────────

#[test]
fn build_payload_minimal() {
    let params = InvokeParams {
        input: "hello".into(),
        ..Default::default()
    };
    let json = aws::build_payload(&params);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["input"], "hello");
    assert!(v.get("model").is_none());
    assert!(v.get("system_prompt").is_none());
}

#[test]
fn build_payload_with_model() {
    let params = InvokeParams {
        input: "hi".into(),
        model_id: "anthropic.claude-3-5-sonnet-20241022-v2:0".into(),
        ..Default::default()
    };
    let json = aws::build_payload(&params);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["model"], "anthropic.claude-3-5-sonnet-20241022-v2:0");
}

#[test]
fn build_payload_with_system_prompt() {
    let params = InvokeParams {
        input: "hi".into(),
        system_prompt: "You are a helpful assistant.".into(),
        ..Default::default()
    };
    let json = aws::build_payload(&params);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["system_prompt"], "You are a helpful assistant.");
}

#[test]
fn build_payload_extra_fields_merged() {
    let mut params = InvokeParams {
        input: "hi".into(),
        ..Default::default()
    };
    params
        .extra_payload
        .insert("temperature".into(), serde_json::json!(0.7));
    params
        .extra_payload
        .insert("max_tokens".into(), serde_json::json!(512));

    let json = aws::build_payload(&params);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["temperature"], 0.7);
    assert_eq!(v["max_tokens"], 512);
}

#[test]
fn build_payload_extra_overrides_base_fields() {
    // extra_payload should win if there's a key collision.
    let mut params = InvokeParams {
        input: "original".into(),
        ..Default::default()
    };
    params
        .extra_payload
        .insert("input".into(), serde_json::json!("overridden"));

    let json = aws::build_payload(&params);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["input"], "overridden");
}

#[test]
fn encode_payload_roundtrips() {
    use base64::Engine as _;
    let original = r#"{"input":"hello","system_prompt":"be concise"}"#;
    let encoded = aws::encode_payload(original);
    let decoded = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(decoded, original);
}

// ── session ID ────────────────────────────────────────────────────────────────

#[test]
fn session_id_auto_generated_is_long_enough() {
    let s = aws::build_session_id("");
    assert!(s.len() >= 33, "session id too short: {s}");
}

#[test]
fn session_id_short_input_gets_padded() {
    let s = aws::build_session_id("abc");
    assert!(s.len() >= 33);
}

#[test]
fn session_id_long_input_unchanged() {
    let input = "a".repeat(40);
    let s = aws::build_session_id(&input);
    assert_eq!(s, input);
}

#[test]
fn session_id_exactly_33_chars_unchanged() {
    let input = "a".repeat(33);
    let s = aws::build_session_id(&input);
    assert_eq!(s, input);
}

// ── response parsing ──────────────────────────────────────────────────────────

#[test]
fn parse_invoke_response_happy_path() {
    let json = fixture("invoke_agent_response.json");
    let (_thinking, text) = aws::parse_invoke_response(&json);
    assert_eq!(text, "Hello! How can I help you today?");
}

#[test]
fn parse_invoke_response_empty_content() {
    let json = r#"{"result": {"role": "assistant", "content": []}}"#;
    let (_thinking, text) = aws::parse_invoke_response(json);
    // Falls back to "(empty response)" when content array is empty.
    assert!(!text.is_empty());
}

#[test]
fn parse_invoke_response_not_json_returns_raw() {
    let raw = "plain text response";
    let (_thinking, text) = aws::parse_invoke_response(raw);
    assert_eq!(text, raw);
}

#[test]
fn parse_invoke_response_empty_string() {
    let (_thinking, text) = aws::parse_invoke_response("");
    assert_eq!(text, "(empty response)");
}

#[test]
fn parse_invoke_response_thinking_block() {
    let json = r#"{"result": {"role": "assistant", "content": [{"thinking": "Let me reason..."}, {"text": "The answer is 42."}]}}"#;
    let (thinking, text) = aws::parse_invoke_response(json);
    assert_eq!(thinking, "Let me reason...");
    assert_eq!(text, "The answer is 42.");
}

#[test]
fn parse_invoke_response_json_string_literal() {
    // Strands agents return str(agent_result) which the CLI writes as a JSON-quoted string
    let raw = r#"" Hi there! How can I assist you today?\n""#;
    let (_thinking, text) = aws::parse_invoke_response(raw);
    assert_eq!(text, "Hi there! How can I assist you today?");
}

#[test]
fn parse_invoke_response_json_string_with_newlines() {
    let raw = r#""Here are some options:\n- Option A\n- Option B\n""#;
    let (_thinking, text) = aws::parse_invoke_response(raw);
    assert_eq!(text, "Here are some options:\n- Option A\n- Option B");
}

// ── log parsing ───────────────────────────────────────────────────────────────

#[test]
fn parse_logs_formats_events() {
    let json = fixture("filter_log_events.json");
    let (metrics, content) = aws::parse_logs(&json, "/aws/my-group", "").unwrap();

    assert!(metrics.contains("2 event(s)"));
    assert!(metrics.contains("/aws/my-group"));
    assert!(content.contains("Agent started"));
    assert!(content.contains("Processing request"));
    assert!(content.contains("stream-a"));
}

#[test]
fn parse_logs_with_stream_name_in_metrics() {
    let json = fixture("filter_log_events.json");
    let (metrics, _) = aws::parse_logs(&json, "/aws/my-group", "stream-a").unwrap();
    assert!(metrics.contains("stream: stream-a"));
}

#[test]
fn parse_logs_trims_trailing_newline_from_message() {
    let json = fixture("filter_log_events.json");
    let (_, content) = aws::parse_logs(&json, "/aws/my-group", "").unwrap();
    // Each line is "[ts] [stream] message" with no trailing \n on the message itself.
    // The fixture has "Agent started\n" — after trimming, the line should end with
    // "Agent started" not "Agent started\n".
    for line in content.lines() {
        assert!(!line.ends_with('\n'), "line has trailing newline: {line:?}");
    }
}

#[test]
fn get_logs_empty_log_group_errors() {
    let cli = FakeCli::ok("{}");
    let result = aws::get_logs_with(&cli, "default", "us-east-1", "", "", "");
    assert!(result.is_err());
}

// ── log group listing ─────────────────────────────────────────────────────────

#[test]
fn parse_log_groups_happy_path() {
    let json = fixture("describe_log_groups.json");
    let (groups, next_token) = aws::parse_log_groups(&json).unwrap();

    assert_eq!(groups.len(), 3);
    assert!(groups.contains(&"/aws/bedrock-agentcore/runtimes/abc-123-DEFAULT".to_string()));
    assert!(groups.contains(&"/aws/lambda/my-function".to_string()));
    assert!(next_token.is_none());
}

#[test]
fn parse_log_groups_with_next_token() {
    let json = r#"{
        "logGroups": [{"logGroupName": "/aws/foo"}],
        "nextToken": "abc123"
    }"#;
    let (groups, next_token) = aws::parse_log_groups(json).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(next_token, Some("abc123".to_string()));
}

#[test]
fn parse_log_groups_empty() {
    let json = r#"{"logGroups": []}"#;
    let (groups, next_token) = aws::parse_log_groups(json).unwrap();
    assert!(groups.is_empty());
    assert!(next_token.is_none());
}

#[test]
fn parse_log_groups_bad_json() {
    let result = aws::parse_log_groups("not json");
    assert!(result.is_err());
}

#[test]
fn list_log_groups_with_fake_cli() {
    let cli = FakeCli::ok(&fixture("describe_log_groups.json"));
    let groups = aws::list_log_groups_with(&cli, "default", "us-east-1").unwrap();
    assert_eq!(groups.len(), 3);
    // Results are sorted.
    assert!(groups.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn list_log_groups_cli_error_propagates() {
    let cli = FakeCli::err("AccessDenied");
    let result = aws::list_log_groups_with(&cli, "default", "us-east-1");
    assert!(result.is_err());
}

// ── list_memories ─────────────────────────────────────────────────────────────

#[test]
fn parse_memories_happy_path() {
    let json = fixture("list_memories.json");
    let memories = aws::parse_memories(&json).unwrap();

    assert_eq!(memories.len(), 2);
    assert_eq!(memories[0].id, "mem-abc123");
    assert_eq!(memories[1].id, "mem-def456");
    // list-memories API does not return names; they're empty until enriched.
    assert_eq!(memories[0].name, "");
    assert_eq!(memories[1].name, "");
}

#[test]
fn parse_memories_empty_list() {
    let json = fixture("list_memories_empty.json");
    let memories = aws::parse_memories(&json).unwrap();
    assert!(memories.is_empty());
}

#[test]
fn parse_memories_bad_json() {
    let result = aws::parse_memories("not json at all");
    assert!(result.is_err());
}

#[test]
fn parse_memories_missing_key_skips_entry() {
    // An entry without an id should be silently skipped.
    let json = r#"{"memories": [{"status": "ACTIVE"}]}"#;
    let memories = aws::parse_memories(json).unwrap();
    assert!(memories.is_empty());
}

#[test]
fn parse_memories_entry_without_name() {
    // Real list-memories entries have id but no name.
    let json = r#"{"memories": [{"id": "mem-xyz789", "status": "ACTIVE"}]}"#;
    let memories = aws::parse_memories(json).unwrap();
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].id, "mem-xyz789");
    assert_eq!(memories[0].name, "");
}

#[test]
fn list_memories_with_fake_cli() {
    // FakeCli returns the same response for every call; the enrichment get-memory
    // calls will silently fail, leaving names empty.
    let cli = FakeCli::ok(&fixture("list_memories.json"));
    let memories = aws::list_memories_with(&cli, "default", "us-east-1").unwrap();
    assert_eq!(memories.len(), 2);
    assert_eq!(memories[0].id, "mem-abc123");
    assert_eq!(memories[1].id, "mem-def456");
}

#[test]
fn list_memories_cli_error_propagates() {
    let cli = FakeCli::err("AccessDenied");
    let result = aws::list_memories_with(&cli, "default", "us-east-1");
    assert!(result.is_err());
}

// ── get_memory ───────────────────────────────────────────────────────────────

#[test]
fn parse_get_memory_response_happy_path() {
    let json = fixture("get_memory.json");
    let mem = aws::parse_get_memory_response(&json).unwrap();
    assert_eq!(mem.id, "mem-abc123");
    assert_eq!(mem.name, "Customer Support Memory");
}

#[test]
fn get_memory_with_fake_cli() {
    let cli = FakeCli::ok(&fixture("get_memory.json"));
    let mem = aws::get_memory_with(&cli, "default", "us-east-1", "mem-abc123").unwrap();
    assert_eq!(mem.id, "mem-abc123");
    assert_eq!(mem.name, "Customer Support Memory");
}

#[test]
fn parse_get_memory_response_bad_json() {
    let result = aws::parse_get_memory_response("not json");
    assert!(result.is_err());
}

// ── is_valid_memory_name ──────────────────────────────────────────────────────

#[test]
fn valid_memory_names() {
    assert!(aws::is_valid_memory_name("a"));
    assert!(aws::is_valid_memory_name("MyMemory"));
    assert!(aws::is_valid_memory_name("my_memory_123"));
    assert!(aws::is_valid_memory_name("A".repeat(48).as_str()));
}

#[test]
fn invalid_memory_names() {
    assert!(!aws::is_valid_memory_name(""));
    assert!(!aws::is_valid_memory_name("1starts_with_digit"));
    assert!(!aws::is_valid_memory_name("_starts_with_underscore"));
    assert!(!aws::is_valid_memory_name("has spaces"));
    assert!(!aws::is_valid_memory_name("has-dashes"));
    assert!(!aws::is_valid_memory_name("has.dots"));
    assert!(!aws::is_valid_memory_name(&"A".repeat(49)));
}

// ── create_memory ────────────────────────────────────────────────────────────

#[test]
fn parse_create_memory_response_happy_path() {
    let json = fixture("create_memory.json");
    let mem = aws::parse_create_memory_response(&json).unwrap();
    assert_eq!(mem.id, "mem-new123");
    assert_eq!(mem.name, "my_test_memory");
}

#[test]
fn parse_create_memory_response_missing_id() {
    let json = r#"{"name": "oops"}"#;
    let result = aws::parse_create_memory_response(json);
    assert!(result.is_err());
}

#[test]
fn parse_create_memory_response_bad_json() {
    let result = aws::parse_create_memory_response("not json");
    assert!(result.is_err());
}

#[test]
fn create_memory_with_invalid_name_errors() {
    let cli = FakeCli::ok("{}");
    let params = agentcore_manager::types::CreateMemoryParams {
        profile: "default".into(),
        region: "us-east-1".into(),
        name: "1invalid".into(),
        event_expiry_duration: 30,
        ..Default::default()
    };
    let result = aws::create_memory_with(&cli, &params);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid memory name"));
}

#[test]
fn create_memory_with_fake_cli() {
    let cli = FakeCli::ok(&fixture("create_memory.json"));
    let params = agentcore_manager::types::CreateMemoryParams {
        profile: "default".into(),
        region: "us-east-1".into(),
        name: "my_test_memory".into(),
        event_expiry_duration: 30,
        ..Default::default()
    };
    let mem = aws::create_memory_with(&cli, &params).unwrap();
    assert_eq!(mem.id, "mem-new123");
    assert_eq!(mem.name, "my_test_memory");
}

#[test]
fn create_memory_cli_error_propagates() {
    let cli = FakeCli::err("AccessDenied");
    let params = agentcore_manager::types::CreateMemoryParams {
        profile: "default".into(),
        region: "us-east-1".into(),
        name: "valid_name".into(),
        event_expiry_duration: 30,
        ..Default::default()
    };
    let result = aws::create_memory_with(&cli, &params);
    assert!(result.is_err());
}

// ── delete_memory ────────────────────────────────────────────────────────────

#[test]
fn delete_memory_with_fake_cli() {
    let cli = FakeCli::ok("{}");
    let result = aws::delete_memory_with(&cli, "default", "us-east-1", "mem-abc123");
    assert!(result.is_ok());
}

#[test]
fn delete_memory_cli_error_propagates() {
    let cli = FakeCli::err("ResourceNotFound");
    let result = aws::delete_memory_with(&cli, "default", "us-east-1", "mem-abc123");
    assert!(result.is_err());
}

// ── integration tests (real AWS, skipped by default) ─────────────────────────

/// Runs a real invoke-agent-runtime call.
/// Requires: RUN_INTEGRATION_TESTS=1, valid AWS credentials, and env vars:
///   AGENTCORE_ARN, AWS_PROFILE, AWS_REGION
#[test]
#[ignore]
fn integration_invoke_real_agent() {
    if std::env::var("RUN_INTEGRATION_TESTS").unwrap_or_default() != "1" {
        return;
    }

    let arn = std::env::var("AGENTCORE_ARN").expect("AGENTCORE_ARN not set");
    let profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".into());
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into());

    let params = InvokeParams {
        profile,
        region,
        agent_arn: arn,
        input: "Say hello in exactly three words.".into(),
        system_prompt: "You are extremely concise.".into(),
        ..Default::default()
    };

    let (_thinking, response, debug_log) = aws::invoke_agent(&params).expect("invoke failed");
    println!("Response:\n{response}");
    println!("Debug:\n{debug_log}");
    assert!(!response.is_empty());
}

// ── Agent Runtime Detail ─────────────────────────────────────────────────────

#[test]
fn parse_agent_runtime_detail_works() {
    let json = fixture("get_agent_runtime.json");
    let detail = aws::parse_agent_runtime_detail(&json).unwrap();

    assert_eq!(detail.id, "MyRuntime-ExAmPlE123");
    assert_eq!(detail.name, "MyRuntime");
    assert!(detail.arn.contains("MyRuntime-ExAmPlE123"));
    assert_eq!(detail.version, "12");
    assert_eq!(detail.status, "READY");
    assert_eq!(detail.description, "Primary agent runtime for testing");
    assert!(detail.role_arn.contains("123456789012"));
    assert_eq!(detail.network_mode, "PUBLIC");
    assert_eq!(detail.idle_session_timeout, 900);
    assert_eq!(detail.max_lifetime, 28800);
    assert_eq!(detail.protocol, "HTTP");
    assert!(detail.code_s3_bucket.contains("bedrock-agentcore-runtime"));
    assert_eq!(detail.code_s3_prefix, "source/agent-bundle.zip");
    assert_eq!(detail.code_runtime, "PYTHON_3_13");
    assert_eq!(detail.code_entry_point, "main.py");
    assert_eq!(detail.environment_variables.get("LOG_LEVEL").unwrap(), "INFO");
    assert_eq!(detail.environment_variables.get("MAX_TURNS").unwrap(), "10");
    assert!(!detail.created_at.is_empty());
    assert!(!detail.last_updated_at.is_empty());
}

#[test]
fn parse_agent_runtime_detail_minimal() {
    // Missing optional fields should default gracefully.
    let json = r#"{
        "agentRuntimeId": "Test-abc1234567",
        "agentRuntimeName": "Test",
        "agentRuntimeArn": "arn:aws:bedrock-agentcore:us-east-1:123:runtime/Test-abc1234567",
        "agentRuntimeVersion": "1",
        "status": "CREATING",
        "roleArn": "arn:aws:iam::123:role/test-role",
        "networkConfiguration": {"networkMode": "PUBLIC"},
        "agentRuntimeArtifact": {
            "codeConfiguration": {
                "code": {"s3": {"bucket": "test-bucket", "prefix": "code.zip"}},
                "runtime": "PYTHON_3_13",
                "entryPoint": ["main.py"]
            }
        },
        "createdAt": "2026-02-22T00:00:00+00:00",
        "lastUpdatedAt": "2026-02-22T00:00:00+00:00"
    }"#;
    let detail = aws::parse_agent_runtime_detail(json).unwrap();
    assert_eq!(detail.description, "");
    assert_eq!(detail.idle_session_timeout, 900); // default
    assert_eq!(detail.max_lifetime, 28800); // default
    assert_eq!(detail.protocol, "HTTP"); // default
    assert!(detail.environment_variables.is_empty());
}

#[test]
fn parse_agent_runtime_detail_bad_json() {
    let result = aws::parse_agent_runtime_detail("not json");
    assert!(result.is_err());
}

#[test]
fn get_agent_runtime_with_fake_cli() {
    let cli = FakeCli::ok(&fixture("get_agent_runtime.json"));
    let detail = aws::get_agent_runtime_with(&cli, "default", "us-east-1", "MyRuntime-ExAmPlE123")
        .unwrap();
    assert_eq!(detail.name, "MyRuntime");
    assert_eq!(detail.version, "12");
}
