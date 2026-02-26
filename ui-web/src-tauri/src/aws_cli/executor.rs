//! Credential-aware CLI executor that injects resolved credentials as env vars.
//!
//! `AuthenticatedCli` implements the existing `CliRunner` trait, making it a
//! drop-in replacement for `RealCli` in any `_with()` function in `aws.rs`.

use crate::aws::{CliOutput, CliRunner};
use crate::credentials::CredentialManager;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

/// A CLI runner that resolves credentials from `CredentialManager` and injects
/// them as environment variables, removing `--profile`/`--region` flags.
///
/// Falls back to passing args unchanged if credential resolution fails
/// (e.g., for `CredentialProcess` profiles or when not yet authenticated).
pub struct AuthenticatedCli {
    cred_manager: Arc<CredentialManager>,
    profile: String,
    region: String,
}

impl AuthenticatedCli {
    pub fn new(cred_manager: Arc<CredentialManager>, profile: &str, region: &str) -> Self {
        Self {
            cred_manager,
            profile: profile.to_string(),
            region: region.to_string(),
        }
    }
}

impl CliRunner for AuthenticatedCli {
    fn run(&self, program: &str, args: &[&str]) -> Result<CliOutput, String> {
        // Try to get cached credentials from the manager
        if let Some(creds) = self.cred_manager.get_cached_credentials(&self.profile) {
            let mut env_vars = HashMap::new();
            env_vars.insert(
                "AWS_ACCESS_KEY_ID".to_string(),
                creds.access_key_id.clone(),
            );
            env_vars.insert(
                "AWS_SECRET_ACCESS_KEY".to_string(),
                creds.secret_access_key.expose().to_string(),
            );
            if let Some(token) = &creds.session_token {
                env_vars.insert("AWS_SESSION_TOKEN".to_string(), token.clone());
            }
            env_vars.insert("AWS_DEFAULT_REGION".to_string(), self.region.clone());

            // Strip --profile and --region from args since we're injecting env vars
            let filtered = filter_profile_region_args(args);
            return run_with_env(program, &filtered, &env_vars);
        }

        // Fallback: run with original args (which include --profile/--region)
        run_with_env(program, args, &HashMap::new())
    }
}

/// Execute a command with optional environment variable overrides.
fn run_with_env(
    program: &str,
    args: &[&str],
    env: &HashMap<String, String>,
) -> Result<CliOutput, String> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run {program}: {e}"))?;
    Ok(CliOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        success: output.status.success(),
    })
}

/// Remove `--profile <value>` and `--region <value>` flag pairs from an argument list.
///
/// When we inject credentials via env vars, the CLI shouldn't also try to
/// resolve a profile (which could conflict or trigger extra config lookups).
fn filter_profile_region_args<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut filtered = Vec::with_capacity(args.len());
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if *arg == "--profile" || *arg == "--region" {
            skip_next = true;
            continue;
        }
        filtered.push(*arg);
    }
    filtered
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_removes_profile_and_region() {
        let args = vec![
            "sts",
            "get-caller-identity",
            "--profile",
            "my-profile",
            "--region",
            "us-east-1",
            "--output",
            "json",
        ];
        let filtered = filter_profile_region_args(&args);
        assert_eq!(filtered, vec!["sts", "get-caller-identity", "--output", "json"]);
    }

    #[test]
    fn filter_no_profile_passes_through() {
        let args = vec!["s3", "ls", "--output", "json"];
        let filtered = filter_profile_region_args(&args);
        assert_eq!(filtered, args);
    }

    #[test]
    fn filter_profile_at_end() {
        let args = vec!["s3", "ls", "--profile", "prod"];
        let filtered = filter_profile_region_args(&args);
        assert_eq!(filtered, vec!["s3", "ls"]);
    }

    #[test]
    fn filter_only_profile_not_region() {
        let args = vec![
            "s3",
            "ls",
            "--profile",
            "staging",
            "--output",
            "json",
        ];
        let filtered = filter_profile_region_args(&args);
        assert_eq!(filtered, vec!["s3", "ls", "--output", "json"]);
    }
}
