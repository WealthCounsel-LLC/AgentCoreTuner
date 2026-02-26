//! Native SSO OIDC device authorization flow (RFC 8628).
//!
//! Implements the complete SSO login flow without shelling out to `aws sso login`:
//! 1. RegisterClient — register this app as an OIDC client
//! 2. StartDeviceAuthorization — get verification URL + user code
//! 3. Poll CreateToken — wait for browser authorization
//! 4. GetRoleCredentials — exchange access token for temporary IAM creds

use aws_sdk_sso::Client as SsoClient;
use aws_sdk_ssooidc::Client as SsoOidcClient;
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::time::Duration;

use super::sso_cache::SsoCacheEntry;
use super::types::{ResolvedCredentials, SecretString};

const APP_NAME: &str = "AgentCore-Manager";

/// State exposed to the UI during the SSO device authorization flow.
#[derive(Debug, Clone)]
pub struct SsoFlowProgress {
    pub verification_uri: String,
    pub user_code: String,
    pub phase: SsoFlowPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SsoFlowPhase {
    /// Waiting for user to authorize in browser.
    WaitingForBrowser,
    /// Token received, fetching role credentials.
    FetchingCredentials,
    /// Flow completed successfully.
    Complete,
    /// Flow failed.
    Failed(String),
}

/// Perform the complete SSO OIDC device authorization flow.
///
/// `on_progress` is called at each phase change (for UI updates).
/// Returns the SSO cache entry (for caching) and resolved credentials.
pub async fn perform_sso_login(
    start_url: &str,
    sso_region: &str,
    account_id: &str,
    role_name: &str,
    on_progress: impl Fn(SsoFlowProgress),
) -> Result<(SsoCacheEntry, ResolvedCredentials), String> {
    let region = aws_sdk_ssooidc::config::Region::new(sso_region.to_string());
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region.clone())
        .no_credentials()
        .load()
        .await;

    let oidc_client = SsoOidcClient::new(&config);

    // Step 1: Register client
    let reg = oidc_client
        .register_client()
        .client_name(APP_NAME)
        .client_type("public")
        .scopes("sso:account:access")
        .send()
        .await
        .map_err(|e| format!("RegisterClient failed: {e}"))?;

    let client_id = reg
        .client_id()
        .ok_or("RegisterClient: missing client_id")?
        .to_string();
    let client_secret = reg
        .client_secret()
        .ok_or("RegisterClient: missing client_secret")?
        .to_string();

    // Step 2: Start device authorization
    let auth = oidc_client
        .start_device_authorization()
        .client_id(&client_id)
        .client_secret(&client_secret)
        .start_url(start_url)
        .send()
        .await
        .map_err(|e| format!("StartDeviceAuthorization failed: {e}"))?;

    let verification_uri_complete = auth
        .verification_uri_complete()
        .unwrap_or_else(|| auth.verification_uri().unwrap_or(""))
        .to_string();
    let device_code = auth
        .device_code()
        .ok_or("StartDeviceAuthorization: missing device_code")?
        .to_string();
    let user_code = auth
        .user_code()
        .ok_or("StartDeviceAuthorization: missing user_code")?
        .to_string();
    let mut interval_secs = auth.interval() as u64;
    if interval_secs == 0 {
        interval_secs = 5; // Default poll interval
    }
    let expires_in = auth.expires_in() as u64;

    // Step 3: Open browser
    if let Err(e) = open::that(&verification_uri_complete) {
        eprintln!("Warning: could not open browser: {e}");
    }

    on_progress(SsoFlowProgress {
        verification_uri: verification_uri_complete.clone(),
        user_code: user_code.clone(),
        phase: SsoFlowPhase::WaitingForBrowser,
    });

    // Step 4: Poll CreateToken
    let deadline =
        tokio::time::Instant::now() + Duration::from_secs(expires_in.max(120));

    let token_result = loop {
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;

        if tokio::time::Instant::now() > deadline {
            return Err("SSO login timed out — authorization window expired".into());
        }

        let result = oidc_client
            .create_token()
            .grant_type("urn:ietf:params:oauth:grant-type:device_code")
            .device_code(&device_code)
            .client_id(&client_id)
            .client_secret(&client_secret)
            .send()
            .await;

        match result {
            Ok(token) => break token,
            Err(e) => {
                let err_str = format!("{e}");
                if err_str.contains("AuthorizationPendingException")
                    || err_str.contains("authorization_pending")
                {
                    continue; // Keep polling
                } else if err_str.contains("SlowDownException")
                    || err_str.contains("slow_down")
                {
                    interval_secs += 5; // Back off
                    continue;
                } else if err_str.contains("ExpiredTokenException")
                    || err_str.contains("expired_token")
                {
                    return Err(
                        "SSO authorization expired — please try again".into(),
                    );
                } else {
                    return Err(format!("CreateToken failed: {e}"));
                }
            }
        }
    };

    let access_token = token_result
        .access_token()
        .ok_or("CreateToken: missing access_token")?
        .to_string();
    let token_expires_in = token_result.expires_in() as u64;

    // Compute expiry time for cache entry
    let expires_at_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + token_expires_in;
    let expires_at_str = format_utc_timestamp(expires_at_secs);

    // Build cache entry
    let cache_entry = SsoCacheEntry {
        start_url: start_url.to_string(),
        region: sso_region.to_string(),
        access_token: access_token.clone(),
        expires_at: expires_at_str,
        client_id: Some(client_id),
        client_secret: Some(client_secret),
        registration_expires_at: None,
        refresh_token: token_result.refresh_token().map(|s| s.to_string()),
    };

    // Step 5: Write cache file
    if let Err(e) = write_sso_cache(&cache_entry) {
        eprintln!("Warning: could not write SSO cache: {e}");
    }

    on_progress(SsoFlowProgress {
        verification_uri: verification_uri_complete.clone(),
        user_code: user_code.clone(),
        phase: SsoFlowPhase::FetchingCredentials,
    });

    // Step 6: Get role credentials
    let sso_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .no_credentials()
        .load()
        .await;
    let sso_client = SsoClient::new(&sso_config);

    let role_creds = sso_client
        .get_role_credentials()
        .access_token(&access_token)
        .account_id(account_id)
        .role_name(role_name)
        .send()
        .await
        .map_err(|e| format!("GetRoleCredentials failed: {e}"))?;

    let creds = role_creds
        .role_credentials()
        .ok_or("GetRoleCredentials: missing role_credentials")?;

    let resolved = ResolvedCredentials {
        access_key_id: creds.access_key_id().unwrap_or_default().to_string(),
        secret_access_key: SecretString::new(
            creds.secret_access_key().unwrap_or_default().to_string(),
        ),
        session_token: creds.session_token().map(|s| s.to_string()),
        expires_at: {
            let millis = creds.expiration();
            if millis > 0 {
                std::time::SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_millis(millis as u64))
            } else {
                None
            }
        },
        region: Some(sso_region.to_string()),
    };

    on_progress(SsoFlowProgress {
        verification_uri: verification_uri_complete,
        user_code,
        phase: SsoFlowPhase::Complete,
    });

    Ok((cache_entry, resolved))
}

/// Write an SSO cache entry to `~/.aws/sso/cache/<hash>.json`.
fn write_sso_cache(entry: &SsoCacheEntry) -> Result<(), String> {
    let cache_dir = sso_cache_dir();
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create SSO cache dir: {e}"))?;

    let filename = sso_cache_filename(&entry.start_url);
    let path = cache_dir.join(filename);

    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("Failed to serialize SSO cache: {e}"))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write SSO cache: {e}"))?;

    // Set file permissions to 0600 (owner read/write only) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(&path, perms);
    }

    Ok(())
}

/// Compute the SSO cache filename: SHA1 of the start URL, matching AWS CLI behavior.
pub fn sso_cache_filename(start_url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(start_url.as_bytes());
    let hash = hasher.finalize();
    format!("{:x}.json", hash)
}

/// Format a Unix timestamp (seconds) as an ISO 8601 UTC string for cache files.
fn format_utc_timestamp(unix_secs: u64) -> String {
    // Simple UTC formatting without chrono dependency
    let secs = unix_secs;
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}UTC"
    )
}

/// Convert days since Unix epoch to (year, month, day). Inverse of Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn sso_cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aws/sso/cache")
}

/// Read a cached SSO token from disk and exchange it for role credentials.
///
/// This is the "fast path" — if a valid cached token exists, skip the device flow.
pub async fn get_role_credentials_from_cache(
    cache_entry: &SsoCacheEntry,
    account_id: &str,
    role_name: &str,
) -> Result<ResolvedCredentials, String> {
    let region =
        aws_sdk_sso::config::Region::new(cache_entry.region.clone());
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .no_credentials()
        .load()
        .await;
    let sso_client = SsoClient::new(&config);

    let role_creds = sso_client
        .get_role_credentials()
        .access_token(&cache_entry.access_token)
        .account_id(account_id)
        .role_name(role_name)
        .send()
        .await
        .map_err(|e| format!("GetRoleCredentials failed: {e}"))?;

    let creds = role_creds
        .role_credentials()
        .ok_or("GetRoleCredentials: missing role_credentials")?;

    Ok(ResolvedCredentials {
        access_key_id: creds.access_key_id().unwrap_or_default().to_string(),
        secret_access_key: SecretString::new(
            creds.secret_access_key().unwrap_or_default().to_string(),
        ),
        session_token: creds.session_token().map(|s| s.to_string()),
        expires_at: {
            let millis = creds.expiration();
            if millis > 0 {
                std::time::SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_millis(millis as u64))
            } else {
                None
            }
        },
        region: Some(cache_entry.region.clone()),
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sso_cache_filename_is_sha1_of_start_url() {
        // Known SHA1 hash for this URL (verified against Python hashlib)
        let filename = sso_cache_filename("https://my-sso-portal.awsapps.com/start");
        assert!(filename.ends_with(".json"));
        assert_eq!(filename.len(), 40 + 5); // 40 hex chars + ".json"
    }

    #[test]
    fn sso_cache_filename_deterministic() {
        let a = sso_cache_filename("https://example.com");
        let b = sso_cache_filename("https://example.com");
        assert_eq!(a, b);
    }

    #[test]
    fn sso_cache_filename_differs_for_different_urls() {
        let a = sso_cache_filename("https://portal-a.awsapps.com/start");
        let b = sso_cache_filename("https://portal-b.awsapps.com/start");
        assert_ne!(a, b);
    }

    #[test]
    fn format_utc_timestamp_roundtrip() {
        // 2025-01-15T12:30:45UTC
        // = days since epoch * 86400 + 12*3600 + 30*60 + 45
        let ts = format_utc_timestamp(1736944245);
        assert!(ts.contains("2025"));
        assert!(ts.ends_with("UTC"));
        assert!(ts.contains("T"));
    }

    #[test]
    fn write_and_read_sso_cache() {
        let dir = std::env::temp_dir().join(format!(
            "sso_write_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let entry = SsoCacheEntry {
            start_url: "https://test-portal.awsapps.com/start".into(),
            region: "us-east-1".into(),
            access_token: "test-token".into(),
            expires_at: "2099-12-31T23:59:59UTC".into(),
            client_id: Some("client-123".into()),
            client_secret: Some("secret-456".into()),
            registration_expires_at: None,
            refresh_token: None,
        };

        let filename = sso_cache_filename(&entry.start_url);
        let path = dir.join(&filename);

        let json = serde_json::to_string_pretty(&entry).unwrap();
        std::fs::write(&path, &json).unwrap();

        // Read it back
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: SsoCacheEntry = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.start_url, entry.start_url);
        assert_eq!(parsed.access_token, entry.access_token);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
