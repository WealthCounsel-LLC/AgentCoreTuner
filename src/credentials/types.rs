//! Core types for the AWS credential management module.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// ── Profile classification ──────────────────────────────────────────────────

/// How a profile obtains its credentials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileType {
    /// Static IAM access key in `~/.aws/credentials`.
    Iam,
    /// AWS SSO (Identity Center) with OIDC device flow.
    Sso,
    /// STS AssumeRole, possibly chained from a source profile.
    AssumeRole,
    /// External process via `credential_process` setting.
    CredentialProcess,
    /// Could not determine type from config keys.
    Unknown,
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iam => write!(f, "IAM"),
            Self::Sso => write!(f, "SSO"),
            Self::AssumeRole => write!(f, "AssumeRole"),
            Self::CredentialProcess => write!(f, "credential_process"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Parsed information about a single AWS profile.
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub profile_type: ProfileType,
    // SSO fields
    pub sso_start_url: Option<String>,
    pub sso_account_id: Option<String>,
    pub sso_role_name: Option<String>,
    pub sso_session: Option<String>,
    pub sso_region: Option<String>,
    // AssumeRole fields
    pub source_profile: Option<String>,
    pub role_arn: Option<String>,
    pub mfa_serial: Option<String>,
    // Common
    pub region: Option<String>,
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            profile_type: ProfileType::Unknown,
            sso_start_url: None,
            sso_account_id: None,
            sso_role_name: None,
            sso_session: None,
            sso_region: None,
            source_profile: None,
            role_arn: None,
            mfa_serial: None,
            region: None,
        }
    }
}

// ── Credential status ───────────────────────────────────────────────────────

/// Current authentication state for a profile.
#[derive(Debug, Clone, PartialEq)]
pub enum CredentialStatus {
    /// Credentials are valid and not expiring soon.
    Valid,
    /// Credentials will expire within 5 minutes.
    Expiring,
    /// Credentials have expired.
    Expired,
    /// No credentials have been obtained for this profile.
    NotAuthenticated,
    /// Currently refreshing credentials.
    Refreshing,
    /// An error occurred checking status.
    Error(String),
}

impl CredentialStatus {
    /// Short string for UI display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Expiring => "expiring",
            Self::Expired => "expired",
            Self::NotAuthenticated => "not-authenticated",
            Self::Refreshing => "refreshing",
            Self::Error(_) => "error",
        }
    }
}

// ── Resolved credentials ────────────────────────────────────────────────────

/// Ready-to-use AWS credentials resolved from a profile.
#[derive(Clone)]
pub struct ResolvedCredentials {
    pub access_key_id: String,
    pub secret_access_key: SecretString,
    pub session_token: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub region: Option<String>,
}

impl std::fmt::Debug for ResolvedCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedCredentials")
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &self.secret_access_key)
            .field("session_token", &self.session_token.as_ref().map(|_| "***"))
            .field("expires_at", &self.expires_at)
            .field("region", &self.region)
            .finish()
    }
}

// ── SecretString ────────────────────────────────────────────────────────────

/// A string wrapper that redacts its contents in Debug output.
/// Call `.expose()` to access the inner value.
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Access the underlying secret value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretString(***)")
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_string_debug_is_redacted() {
        let s = SecretString::new("super-secret-key".to_string());
        let debug = format!("{s:?}");
        assert!(!debug.contains("super-secret-key"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn secret_string_expose_returns_value() {
        let s = SecretString::new("my-key".to_string());
        assert_eq!(s.expose(), "my-key");
    }

    #[test]
    fn credential_status_as_str() {
        assert_eq!(CredentialStatus::Valid.as_str(), "valid");
        assert_eq!(CredentialStatus::Expired.as_str(), "expired");
        assert_eq!(CredentialStatus::NotAuthenticated.as_str(), "not-authenticated");
        assert_eq!(CredentialStatus::Error("oops".into()).as_str(), "error");
    }

    #[test]
    fn profile_type_display() {
        assert_eq!(ProfileType::Sso.to_string(), "SSO");
        assert_eq!(ProfileType::AssumeRole.to_string(), "AssumeRole");
    }

    #[test]
    fn resolved_credentials_debug_redacts_secrets() {
        let creds = ResolvedCredentials {
            access_key_id: "AKIA123".to_string(),
            secret_access_key: SecretString::new("wJalrXUtnFEMI/K7MDENG".to_string()),
            session_token: Some("token123".to_string()),
            expires_at: None,
            region: Some("us-east-1".to_string()),
        };
        let debug = format!("{creds:?}");
        assert!(!debug.contains("wJalrXUtnFEMI"));
        assert!(!debug.contains("token123"));
        assert!(debug.contains("AKIA123")); // access key ID is not secret
    }
}
