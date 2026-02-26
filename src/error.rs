//! Unified error types for the AWS authentication module.

/// Errors that can occur during credential resolution and management.
#[derive(Debug)]
pub enum AuthError {
    /// Failed to parse AWS config/credentials files.
    ConfigParse(String),
    /// Profile not found in AWS config.
    ProfileNotFound(String),
    /// SSO token is missing or expired — login required.
    SsoLoginRequired(String),
    /// SSO OIDC flow failed.
    SsoFlowFailed(String),
    /// STS AssumeRole call failed.
    AssumeRoleFailed(String),
    /// MFA code was required but not provided.
    MfaRequired { mfa_serial: String },
    /// OS keychain operation failed.
    KeychainError(String),
    /// Credential resolution failed for an unspecified reason.
    ResolutionFailed(String),
    /// Credentials have expired.
    CredentialsExpired,
    /// AWS API call failed.
    AwsApi(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigParse(msg) => write!(f, "Config parse error: {msg}"),
            Self::ProfileNotFound(name) => write!(f, "Profile not found: {name}"),
            Self::SsoLoginRequired(profile) => {
                write!(f, "SSO login required for profile: {profile}")
            }
            Self::SsoFlowFailed(msg) => write!(f, "SSO login failed: {msg}"),
            Self::AssumeRoleFailed(msg) => write!(f, "AssumeRole failed: {msg}"),
            Self::MfaRequired { mfa_serial } => {
                write!(f, "MFA code required (device: {mfa_serial})")
            }
            Self::KeychainError(msg) => write!(f, "Keychain error: {msg}"),
            Self::ResolutionFailed(msg) => write!(f, "Credential resolution failed: {msg}"),
            Self::CredentialsExpired => write!(f, "Credentials have expired"),
            Self::AwsApi(msg) => write!(f, "AWS API error: {msg}"),
        }
    }
}

impl std::error::Error for AuthError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let err = AuthError::ProfileNotFound("staging".into());
        assert_eq!(err.to_string(), "Profile not found: staging");

        let err = AuthError::MfaRequired {
            mfa_serial: "arn:aws:iam::123:mfa/user".into(),
        };
        assert!(err.to_string().contains("MFA code required"));
    }
}
