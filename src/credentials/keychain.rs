//! OS keychain integration for storing and retrieving IAM credentials.
//!
//! Uses the `keyring` crate which maps to:
//! - macOS: Keychain Services
//! - Windows: Credential Manager
//! - Linux: Secret Service (DBUS) or kernel keyring

use super::types::{ResolvedCredentials, SecretString};

const SERVICE_NAME: &str = "agentcore-manager";

/// Store IAM credentials for a profile in the OS keychain.
pub fn store_credentials(profile: &str, creds: &ResolvedCredentials) -> Result<(), String> {
    let key_entry = keyring::Entry::new(SERVICE_NAME, &format!("{profile}/access-key-id"))
        .map_err(|e| format!("Keychain error: {e}"))?;
    key_entry
        .set_password(&creds.access_key_id)
        .map_err(|e| format!("Failed to store access key: {e}"))?;

    let secret_entry =
        keyring::Entry::new(SERVICE_NAME, &format!("{profile}/secret-access-key"))
            .map_err(|e| format!("Keychain error: {e}"))?;
    secret_entry
        .set_password(creds.secret_access_key.expose())
        .map_err(|e| format!("Failed to store secret key: {e}"))?;

    // Session token (optional — for temporary credentials)
    if let Some(token) = &creds.session_token {
        let token_entry =
            keyring::Entry::new(SERVICE_NAME, &format!("{profile}/session-token"))
                .map_err(|e| format!("Keychain error: {e}"))?;
        token_entry
            .set_password(token)
            .map_err(|e| format!("Failed to store session token: {e}"))?;
    }

    Ok(())
}

/// Load IAM credentials for a profile from the OS keychain.
///
/// Returns `None` if credentials are not stored in the keychain.
pub fn load_credentials(profile: &str) -> Result<Option<ResolvedCredentials>, String> {
    let key_entry = keyring::Entry::new(SERVICE_NAME, &format!("{profile}/access-key-id"))
        .map_err(|e| format!("Keychain error: {e}"))?;
    let secret_entry =
        keyring::Entry::new(SERVICE_NAME, &format!("{profile}/secret-access-key"))
            .map_err(|e| format!("Keychain error: {e}"))?;

    let access_key = match key_entry.get_password() {
        Ok(k) => k,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(format!("Keychain read error: {e}")),
    };

    let secret_key = match secret_entry.get_password() {
        Ok(k) => k,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(format!("Keychain read error: {e}")),
    };

    // Session token is optional
    let session_token = {
        let token_entry =
            keyring::Entry::new(SERVICE_NAME, &format!("{profile}/session-token"))
                .map_err(|e| format!("Keychain error: {e}"))?;
        match token_entry.get_password() {
            Ok(t) => Some(t),
            Err(keyring::Error::NoEntry) => None,
            Err(e) => return Err(format!("Keychain read error: {e}")),
        }
    };

    Ok(Some(ResolvedCredentials {
        access_key_id: access_key,
        secret_access_key: SecretString::new(secret_key),
        session_token,
        expires_at: None, // Static creds don't expire
        region: None,
    }))
}

/// Delete stored credentials for a profile from the OS keychain.
pub fn delete_credentials(profile: &str) -> Result<(), String> {
    for suffix in &["access-key-id", "secret-access-key", "session-token"] {
        let entry = keyring::Entry::new(SERVICE_NAME, &format!("{profile}/{suffix}"))
            .map_err(|e| format!("Keychain error: {e}"))?;
        match entry.delete_credential() {
            Ok(()) => {}
            Err(keyring::Error::NoEntry) => {} // Already deleted, fine
            Err(e) => return Err(format!("Keychain delete error: {e}")),
        }
    }
    Ok(())
}

// Note: Keychain tests require an actual OS keychain, so they're not run in CI.
// Manual testing: cargo test --lib -- credentials::keychain --ignored

#[cfg(test)]
mod tests {
    use super::*;

    // These tests interact with the real OS keychain.
    // Only run them manually with: cargo test -- --ignored
    #[test]
    #[ignore]
    fn keychain_roundtrip() {
        let profile = "agentcore-test-roundtrip";
        let creds = ResolvedCredentials {
            access_key_id: "AKIATEST123".to_string(),
            secret_access_key: SecretString::new("test-secret-key".to_string()),
            session_token: Some("test-session-token".to_string()),
            expires_at: None,
            region: None,
        };

        // Store
        store_credentials(profile, &creds).expect("store failed");

        // Load
        let loaded = load_credentials(profile)
            .expect("load failed")
            .expect("should find stored creds");
        assert_eq!(loaded.access_key_id, "AKIATEST123");
        assert_eq!(loaded.secret_access_key.expose(), "test-secret-key");
        assert_eq!(loaded.session_token.as_deref(), Some("test-session-token"));

        // Delete
        delete_credentials(profile).expect("delete failed");

        // Verify deleted
        let gone = load_credentials(profile).expect("load after delete failed");
        assert!(gone.is_none());
    }
}
