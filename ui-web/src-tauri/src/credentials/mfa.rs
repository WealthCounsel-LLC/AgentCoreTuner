//! MFA-prompted STS AssumeRole flow.
//!
//! Handles profiles with `role_arn` + `mfa_serial` by prompting for a TOTP code
//! and calling `sts:AssumeRole` with the MFA token.

use aws_sdk_sts::Client as StsClient;
use std::time::Duration;

use super::types::{ResolvedCredentials, SecretString};

/// Assume an IAM role with MFA authentication.
///
/// Requires the source profile's credentials to already be resolved (passed via
/// `aws_config`). The MFA code is provided by the caller (from the UI dialog).
pub async fn assume_role_with_mfa(
    config: &aws_config::SdkConfig,
    role_arn: &str,
    session_name: &str,
    mfa_serial: &str,
    mfa_code: &str,
) -> Result<ResolvedCredentials, String> {
    let sts_client = StsClient::new(config);

    let result = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .serial_number(mfa_serial)
        .token_code(mfa_code)
        .send()
        .await
        .map_err(|e| format!("AssumeRole failed: {e}"))?;

    extract_credentials(result.credentials())
}

/// Assume an IAM role without MFA (for profiles with `role_arn` but no `mfa_serial`).
pub async fn assume_role(
    config: &aws_config::SdkConfig,
    role_arn: &str,
    session_name: &str,
) -> Result<ResolvedCredentials, String> {
    let sts_client = StsClient::new(config);

    let result = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .send()
        .await
        .map_err(|e| format!("AssumeRole failed: {e}"))?;

    extract_credentials(result.credentials())
}

/// Extract ResolvedCredentials from the STS AssumeRole response.
fn extract_credentials(
    creds: Option<&aws_sdk_sts::types::Credentials>,
) -> Result<ResolvedCredentials, String> {
    let creds = creds.ok_or("AssumeRole: no credentials in response")?;

    let access_key = creds.access_key_id().to_string();
    let secret_key = creds.secret_access_key().to_string();
    let session_token = Some(creds.session_token().to_string());

    let expiration = {
        let secs = creds.expiration().secs();
        if secs > 0 {
            std::time::SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_secs(secs as u64))
        } else {
            None
        }
    };

    Ok(ResolvedCredentials {
        access_key_id: access_key,
        secret_access_key: SecretString::new(secret_key),
        session_token,
        expires_at: expiration,
        region: None,
    })
}

/// Validate that a string looks like a valid MFA TOTP code (6 digits).
pub fn is_valid_mfa_code(code: &str) -> bool {
    code.len() == 6 && code.chars().all(|c| c.is_ascii_digit())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_mfa_codes() {
        assert!(is_valid_mfa_code("123456"));
        assert!(is_valid_mfa_code("000000"));
        assert!(is_valid_mfa_code("999999"));
    }

    #[test]
    fn invalid_mfa_codes() {
        assert!(!is_valid_mfa_code("")); // empty
        assert!(!is_valid_mfa_code("12345")); // too short
        assert!(!is_valid_mfa_code("1234567")); // too long
        assert!(!is_valid_mfa_code("12345a")); // non-digit
        assert!(!is_valid_mfa_code("abcdef")); // all non-digit
        assert!(!is_valid_mfa_code(" 12345")); // space
    }
}
