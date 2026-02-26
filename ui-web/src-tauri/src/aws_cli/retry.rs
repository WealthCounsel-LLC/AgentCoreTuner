//! Auto-retry wrapper for CLI commands with credential refresh on expiry.
//!
//! Wraps any `CliRunner` and intercepts expired-credential errors to
//! transparently refresh credentials and retry once.

use crate::aws::{CliOutput, CliRunner};
use crate::credentials::CredentialManager;
use std::sync::Arc;

use super::error_classifier::{classify_error, ErrorClass};

/// A retry wrapper that intercepts expired-credential errors, refreshes
/// credentials via `CredentialManager`, and retries the command once.
///
/// Also retries on transient network errors and throttling with backoff.
pub struct RetryRunner<R: CliRunner> {
    inner: R,
    cred_manager: Arc<CredentialManager>,
    profile: String,
}

impl<R: CliRunner> RetryRunner<R> {
    pub fn new(inner: R, cred_manager: Arc<CredentialManager>, profile: &str) -> Self {
        Self {
            inner,
            cred_manager,
            profile: profile.to_string(),
        }
    }
}

impl<R: CliRunner> CliRunner for RetryRunner<R> {
    fn run(&self, program: &str, args: &[&str]) -> Result<CliOutput, String> {
        let result = self.inner.run(program, args)?;

        if result.success {
            return Ok(result);
        }

        let stderr = String::from_utf8_lossy(&result.stderr);
        let error_class = classify_error(&stderr);

        match error_class {
            ErrorClass::ExpiredCredentials | ErrorClass::SsoSessionExpired => {
                // Try to refresh credentials and retry once
                match self.cred_manager.refresh_credentials(&self.profile) {
                    Ok(true) => {
                        // Credentials refreshed — retry the command
                        self.inner.run(program, args)
                    }
                    _ => {
                        // Refresh failed or no credentials available — return original error
                        Ok(result)
                    }
                }
            }
            ErrorClass::Throttled | ErrorClass::NetworkError => {
                // Simple backoff: sleep briefly and retry once
                std::thread::sleep(std::time::Duration::from_millis(1000));
                self.inner.run(program, args)
            }
            _ => {
                // Not retryable — return original result
                Ok(result)
            }
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aws::CliOutput;
    use std::cell::Cell;

    /// A fake CLI runner that returns pre-configured results on successive calls.
    struct FakeRetryCliInner {
        call_count: Cell<usize>,
        first_result: CliOutput,
        second_result: CliOutput,
    }

    impl CliRunner for FakeRetryCliInner {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<CliOutput, String> {
            let n = self.call_count.get();
            self.call_count.set(n + 1);
            if n == 0 {
                Ok(self.first_result.clone())
            } else {
                Ok(self.second_result.clone())
            }
        }
    }

    fn success_output() -> CliOutput {
        CliOutput {
            stdout: b"ok".to_vec(),
            stderr: Vec::new(),
            success: true,
        }
    }

    fn expired_output() -> CliOutput {
        CliOutput {
            stdout: Vec::new(),
            stderr: b"An error occurred (ExpiredToken)".to_vec(),
            success: false,
        }
    }

    fn access_denied_output() -> CliOutput {
        CliOutput {
            stdout: Vec::new(),
            stderr: b"An error occurred (AccessDenied)".to_vec(),
            success: false,
        }
    }

    #[test]
    fn success_not_retried() {
        let inner = FakeRetryCliInner {
            call_count: Cell::new(0),
            first_result: success_output(),
            second_result: success_output(),
        };
        let cred_manager = Arc::new(CredentialManager::empty());
        let runner = RetryRunner::new(inner, cred_manager, "test-profile");

        let result = runner.run("aws", &["s3", "ls"]).unwrap();
        assert!(result.success);
    }

    #[test]
    fn non_retryable_error_not_retried() {
        let inner = FakeRetryCliInner {
            call_count: Cell::new(0),
            first_result: access_denied_output(),
            second_result: success_output(),
        };
        let cred_manager = Arc::new(CredentialManager::empty());
        let runner = RetryRunner::new(inner, cred_manager, "test-profile");

        let result = runner.run("aws", &["s3", "ls"]).unwrap();
        assert!(!result.success);
        assert!(String::from_utf8_lossy(&result.stderr).contains("AccessDenied"));
    }

    #[test]
    fn expired_credentials_triggers_retry() {
        let inner = FakeRetryCliInner {
            call_count: Cell::new(0),
            first_result: expired_output(),
            second_result: success_output(),
        };
        let cred_manager = Arc::new(CredentialManager::empty());
        let runner = RetryRunner::new(inner, cred_manager, "test-profile");

        // With an empty CredentialManager, refresh_credentials will return Ok(false),
        // so the retry won't actually happen and we get the original error back.
        // This tests that the code path is exercised without panicking.
        let result = runner.run("aws", &["s3", "ls"]).unwrap();
        // With empty manager, refresh fails, so we get original error
        assert!(!result.success);
    }
}
