//! Read and manage SSO token cache entries from `~/.aws/sso/cache/`.
//!
//! The AWS CLI stores SSO access tokens as JSON files in this directory.
//! Filenames are SHA1 hashes of the `startUrl` (legacy) or `sessionName` (modern).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A cached SSO token, matching the AWS CLI JSON format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoCacheEntry {
    #[serde(rename = "startUrl")]
    pub start_url: String,
    pub region: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String, // ISO 8601, e.g. "2025-01-15T12:00:00UTC"
    #[serde(rename = "clientId", default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(rename = "clientSecret", default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(
        rename = "registrationExpiresAt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub registration_expires_at: Option<String>,
    #[serde(rename = "refreshToken", default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Find a valid (non-expired) cached SSO token for the given start URL.
///
/// Scans all `.json` files in the SSO cache directory, looking for one whose
/// `startUrl` matches and whose `expiresAt` is in the future.
pub fn find_cached_token(start_url: &str) -> Option<SsoCacheEntry> {
    find_cached_token_in(&sso_cache_dir(), start_url)
}

/// Find a valid cached token from a specific cache directory (testable).
pub fn find_cached_token_in(cache_dir: &Path, start_url: &str) -> Option<SsoCacheEntry> {
    let entries = read_sso_cache_from(cache_dir);
    entries
        .into_iter()
        .find(|e| e.start_url == start_url && is_token_valid(e))
}

/// Read all SSO cache entries from the given directory.
pub fn read_sso_cache_from(cache_dir: &Path) -> Vec<SsoCacheEntry> {
    let dir = match std::fs::read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Skip files that don't parse as SSO cache (e.g. client registration files)
                if let Ok(cache_entry) = serde_json::from_str::<SsoCacheEntry>(&content) {
                    // Only include entries that have a startUrl (filter out client reg files)
                    if !cache_entry.start_url.is_empty() {
                        entries.push(cache_entry);
                    }
                }
            }
        }
    }
    entries
}

/// Check if an SSO cache entry's access token is still valid (not expired).
pub fn is_token_valid(entry: &SsoCacheEntry) -> bool {
    match parse_expires_at(&entry.expires_at) {
        Some(expires) => expires > SystemTime::now(),
        None => false, // Unparseable expiry → treat as expired
    }
}

/// Check if an SSO cache entry is expiring soon (within 5 minutes).
pub fn is_token_expiring_soon(entry: &SsoCacheEntry) -> bool {
    match parse_expires_at(&entry.expires_at) {
        Some(expires) => {
            let five_min = std::time::Duration::from_secs(300);
            expires > SystemTime::now()
                && expires < SystemTime::now() + five_min
        }
        None => false,
    }
}

/// Parse the `expiresAt` string (ISO 8601) into a SystemTime.
///
/// AWS CLI uses formats like:
/// - `2025-01-15T12:00:00UTC`
/// - `2025-01-15T12:00:00Z`
/// - `2025-01-15T12:00:00+00:00`
fn parse_expires_at(s: &str) -> Option<SystemTime> {
    // Normalize: strip trailing "UTC" → "Z", handle "+00:00"
    let normalized = s
        .trim()
        .trim_end_matches("UTC")
        .trim_end_matches('Z')
        .trim_end_matches("+00:00");

    // Parse "YYYY-MM-DDTHH:MM:SS"
    let parts: Vec<&str> = normalized.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u32> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return None;
    }

    let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);
    let (hour, minute) = (time_parts[0], time_parts[1]);
    let second = if time_parts.len() > 2 {
        time_parts[2]
    } else {
        0
    };

    // Convert to Unix timestamp (simplified — assumes UTC, no leap seconds)
    let days = days_from_civil(year as i64, month, day)?;
    let secs = days as u64 * 86400 + hour as u64 * 3600 + minute as u64 * 60 + second as u64;
    SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(secs))
}

/// Days from Unix epoch to a civil date (algorithm from Howard Hinnant).
fn days_from_civil(y: i64, m: u32, d: u32) -> Option<i64> {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe as i64 - 719468)
}

fn sso_cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aws/sso/cache")
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_cache_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sso_cache_test_{}",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_cache_file(dir: &Path, filename: &str, content: &str) {
        let path = dir.join(filename);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn parse_valid_cache_entry() {
        let dir = temp_cache_dir();
        write_cache_file(
            &dir,
            "abc123.json",
            r#"{
                "startUrl": "https://my-sso.awsapps.com/start",
                "region": "us-east-1",
                "accessToken": "eyJhbGci...",
                "expiresAt": "2099-12-31T23:59:59UTC"
            }"#,
        );

        let entries = read_sso_cache_from(&dir);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_url, "https://my-sso.awsapps.com/start");
        assert!(is_token_valid(&entries[0]));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn expired_token_detected() {
        let entry = SsoCacheEntry {
            start_url: "https://example.com".into(),
            region: "us-east-1".into(),
            access_token: "token".into(),
            expires_at: "2020-01-01T00:00:00UTC".into(),
            client_id: None,
            client_secret: None,
            registration_expires_at: None,
            refresh_token: None,
        };
        assert!(!is_token_valid(&entry));
    }

    #[test]
    fn future_token_is_valid() {
        let entry = SsoCacheEntry {
            start_url: "https://example.com".into(),
            region: "us-east-1".into(),
            access_token: "token".into(),
            expires_at: "2099-12-31T23:59:59UTC".into(),
            client_id: None,
            client_secret: None,
            registration_expires_at: None,
            refresh_token: None,
        };
        assert!(is_token_valid(&entry));
    }

    #[test]
    fn find_cached_token_matches_start_url() {
        let dir = temp_cache_dir();
        write_cache_file(
            &dir,
            "aaa.json",
            r#"{
                "startUrl": "https://portal-a.awsapps.com/start",
                "region": "us-east-1",
                "accessToken": "token-a",
                "expiresAt": "2099-12-31T23:59:59UTC"
            }"#,
        );
        write_cache_file(
            &dir,
            "bbb.json",
            r#"{
                "startUrl": "https://portal-b.awsapps.com/start",
                "region": "eu-west-1",
                "accessToken": "token-b",
                "expiresAt": "2099-12-31T23:59:59UTC"
            }"#,
        );

        let found =
            find_cached_token_in(&dir, "https://portal-b.awsapps.com/start");
        assert!(found.is_some());
        assert_eq!(found.unwrap().access_token, "token-b");

        let not_found = find_cached_token_in(&dir, "https://no-such-portal.com");
        assert!(not_found.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn skip_expired_when_finding_token() {
        let dir = temp_cache_dir();
        write_cache_file(
            &dir,
            "expired.json",
            r#"{
                "startUrl": "https://portal.awsapps.com/start",
                "region": "us-east-1",
                "accessToken": "old-token",
                "expiresAt": "2020-01-01T00:00:00UTC"
            }"#,
        );

        let found = find_cached_token_in(&dir, "https://portal.awsapps.com/start");
        assert!(found.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_json_files_are_skipped() {
        let dir = temp_cache_dir();
        write_cache_file(&dir, "readme.txt", "not json");
        write_cache_file(&dir, "data.json", "not valid json either");

        let entries = read_sso_cache_from(&dir);
        assert!(entries.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_start_url_entries_are_skipped() {
        let dir = temp_cache_dir();
        // Client registration files have no startUrl
        write_cache_file(
            &dir,
            "client.json",
            r#"{
                "startUrl": "",
                "region": "",
                "accessToken": "",
                "expiresAt": "2099-12-31T23:59:59UTC",
                "clientId": "abc",
                "clientSecret": "xyz"
            }"#,
        );

        let entries = read_sso_cache_from(&dir);
        assert!(entries.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_different_time_formats() {
        // "Z" suffix
        let t = parse_expires_at("2025-06-15T10:30:00Z");
        assert!(t.is_some());

        // "UTC" suffix
        let t = parse_expires_at("2025-06-15T10:30:00UTC");
        assert!(t.is_some());

        // "+00:00" suffix
        let t = parse_expires_at("2025-06-15T10:30:00+00:00");
        assert!(t.is_some());

        // All should parse to the same value
        let z = parse_expires_at("2025-06-15T10:30:00Z").unwrap();
        let utc = parse_expires_at("2025-06-15T10:30:00UTC").unwrap();
        let offset = parse_expires_at("2025-06-15T10:30:00+00:00").unwrap();
        assert_eq!(z, utc);
        assert_eq!(z, offset);
    }

    #[test]
    fn nonexistent_cache_dir_returns_empty() {
        let entries = read_sso_cache_from(Path::new("/nonexistent/sso/cache"));
        assert!(entries.is_empty());
    }

    #[test]
    fn cache_entry_roundtrips_through_json() {
        let entry = SsoCacheEntry {
            start_url: "https://portal.awsapps.com/start".into(),
            region: "us-east-1".into(),
            access_token: "eyJhbGci...".into(),
            expires_at: "2099-12-31T23:59:59UTC".into(),
            client_id: Some("client-123".into()),
            client_secret: Some("secret-456".into()),
            registration_expires_at: Some("2099-06-15T00:00:00UTC".into()),
            refresh_token: Some("refresh-789".into()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: SsoCacheEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.start_url, entry.start_url);
        assert_eq!(parsed.client_id, entry.client_id);
        assert_eq!(parsed.refresh_token, entry.refresh_token);
    }
}
