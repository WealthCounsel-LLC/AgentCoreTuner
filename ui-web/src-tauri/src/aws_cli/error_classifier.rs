//! Classify AWS CLI stderr messages into actionable error types.
//!
//! Used by the retry logic to determine whether to auto-refresh credentials
//! and retry, or to surface the error to the user.

/// Classification of an AWS CLI error for retry/recovery decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorClass {
    /// Credentials or session token has expired — refresh and retry.
    ExpiredCredentials,
    /// SSO token specifically has expired — trigger SSO re-login.
    SsoSessionExpired,
    /// Credentials are invalid (wrong key, revoked) — no point retrying.
    InvalidCredentials,
    /// Permission denied — credentials work but lack required permissions.
    AccessDenied,
    /// No credentials found at all — redirect to profile setup.
    NoCredentials,
    /// Network/connectivity issue — retry may help.
    NetworkError,
    /// Throttling — retry with backoff.
    Throttled,
    /// Unknown error — don't retry automatically.
    Unknown,
}

impl ErrorClass {
    /// Whether this error class is worth retrying with refreshed credentials.
    pub fn should_retry_with_refresh(&self) -> bool {
        matches!(self, Self::ExpiredCredentials | Self::SsoSessionExpired)
    }

    /// Whether this error class may resolve with a simple retry (e.g., network).
    pub fn should_retry_with_backoff(&self) -> bool {
        matches!(self, Self::Throttled | Self::NetworkError)
    }
}

/// Classify an AWS CLI stderr string into an error category.
pub fn classify_error(stderr: &str) -> ErrorClass {
    let lower = stderr.to_lowercase();

    // Check most specific patterns first
    if lower.contains("expiredtoken")
        || lower.contains("expired token")
        || lower.contains("token has expired")
        || lower.contains("security token expired")
        || lower.contains("security token included in the request is expired")
    {
        return ErrorClass::ExpiredCredentials;
    }

    if lower.contains("sso") && (lower.contains("expired") || lower.contains("refresh failed")) {
        return ErrorClass::SsoSessionExpired;
    }

    if lower.contains("invalidclienttokenid")
        || lower.contains("invalid client token")
        || lower.contains("signaturedoesnotmatch")
        || lower.contains("the security token included in the request is invalid")
    {
        return ErrorClass::InvalidCredentials;
    }

    if lower.contains("accessdenied")
        || lower.contains("access denied")
        || lower.contains("not authorized")
        || lower.contains("unauthorizedaccess")
    {
        return ErrorClass::AccessDenied;
    }

    if lower.contains("unable to locate credentials")
        || lower.contains("no credentials")
        || lower.contains("credentials not found")
    {
        return ErrorClass::NoCredentials;
    }

    if lower.contains("could not connect")
        || lower.contains("connection refused")
        || lower.contains("name or service not known")
        || lower.contains("network is unreachable")
        || lower.contains("timed out")
        || lower.contains("connection reset")
    {
        return ErrorClass::NetworkError;
    }

    if lower.contains("throttling")
        || lower.contains("rate exceeded")
        || lower.contains("too many requests")
        || lower.contains("requestlimitexceeded")
    {
        return ErrorClass::Throttled;
    }

    ErrorClass::Unknown
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_expired_token() {
        assert_eq!(
            classify_error("An error occurred (ExpiredToken)"),
            ErrorClass::ExpiredCredentials
        );
        assert_eq!(
            classify_error("The security token included in the request is expired"),
            ErrorClass::ExpiredCredentials
        );
    }

    #[test]
    fn classify_sso_expired() {
        assert_eq!(
            classify_error("Error: SSO Token refresh failed"),
            ErrorClass::SsoSessionExpired
        );
        assert_eq!(
            classify_error("The SSO session associated with this profile has expired"),
            ErrorClass::SsoSessionExpired
        );
    }

    #[test]
    fn classify_invalid_credentials() {
        assert_eq!(
            classify_error("An error occurred (InvalidClientTokenId)"),
            ErrorClass::InvalidCredentials
        );
        assert_eq!(
            classify_error("The security token included in the request is invalid"),
            ErrorClass::InvalidCredentials
        );
    }

    #[test]
    fn classify_access_denied() {
        assert_eq!(
            classify_error("An error occurred (AccessDenied)"),
            ErrorClass::AccessDenied
        );
        assert_eq!(
            classify_error("User is not authorized to perform this action"),
            ErrorClass::AccessDenied
        );
    }

    #[test]
    fn classify_no_credentials() {
        assert_eq!(
            classify_error("Unable to locate credentials"),
            ErrorClass::NoCredentials
        );
    }

    #[test]
    fn classify_network_error() {
        assert_eq!(
            classify_error("Could not connect to the endpoint URL"),
            ErrorClass::NetworkError
        );
        assert_eq!(
            classify_error("Connection refused"),
            ErrorClass::NetworkError
        );
    }

    #[test]
    fn classify_throttling() {
        assert_eq!(
            classify_error("An error occurred (Throttling)"),
            ErrorClass::Throttled
        );
        assert_eq!(
            classify_error("Rate exceeded"),
            ErrorClass::Throttled
        );
    }

    #[test]
    fn classify_unknown() {
        assert_eq!(
            classify_error("some random error message"),
            ErrorClass::Unknown
        );
    }

    #[test]
    fn retry_decisions() {
        assert!(ErrorClass::ExpiredCredentials.should_retry_with_refresh());
        assert!(ErrorClass::SsoSessionExpired.should_retry_with_refresh());
        assert!(!ErrorClass::AccessDenied.should_retry_with_refresh());

        assert!(ErrorClass::Throttled.should_retry_with_backoff());
        assert!(ErrorClass::NetworkError.should_retry_with_backoff());
        assert!(!ErrorClass::ExpiredCredentials.should_retry_with_backoff());
    }
}
