//! Parse `~/.aws/config` and `~/.aws/credentials` to enumerate and classify profiles.

use ini::Ini;
use std::path::{Path, PathBuf};

use super::types::{ProfileInfo, ProfileType};

/// Parse all AWS profiles from the default config and credentials file paths.
///
/// Respects `AWS_CONFIG_FILE` and `AWS_SHARED_CREDENTIALS_FILE` env overrides.
pub fn parse_profiles() -> Result<Vec<ProfileInfo>, String> {
    parse_profiles_from(&aws_config_path(), &aws_credentials_path())
}

/// Parse profiles from explicit file paths (used by tests and the public entry point).
///
/// Profiles in the config file use `[profile X]` headers (except `[default]`).
/// Profiles in the credentials file use plain `[X]` headers.
pub fn parse_profiles_from(
    config_path: &Path,
    credentials_path: &Path,
) -> Result<Vec<ProfileInfo>, String> {
    let mut profiles: Vec<ProfileInfo> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Parse config file (e.g. ~/.aws/config)
    if config_path.exists() {
        let config = Ini::load_from_file(config_path)
            .map_err(|e| format!("Failed to parse {}: {e}", config_path.display()))?;

        for (section, props) in config.iter() {
            let name = match section {
                Some(s) => {
                    // Config file uses "[profile foo]" for named profiles, "[default]" for default
                    s.strip_prefix("profile ").unwrap_or(s).to_string()
                }
                None => continue, // Skip the global/anonymous section
            };

            let profile_type = classify_profile(&props);

            profiles.push(ProfileInfo {
                name: name.clone(),
                profile_type,
                sso_start_url: prop_str(&props, "sso_start_url"),
                sso_account_id: prop_str(&props, "sso_account_id"),
                sso_role_name: prop_str(&props, "sso_role_name"),
                sso_session: prop_str(&props, "sso_session"),
                sso_region: prop_str(&props, "sso_region"),
                source_profile: prop_str(&props, "source_profile"),
                role_arn: prop_str(&props, "role_arn"),
                mfa_serial: prop_str(&props, "mfa_serial"),
                region: prop_str(&props, "region"),
            });
            seen_names.insert(name);
        }
    }

    // Parse credentials file for profiles not already found in config
    if credentials_path.exists() {
        let creds = Ini::load_from_file(credentials_path)
            .map_err(|e| format!("Failed to parse {}: {e}", credentials_path.display()))?;

        for (section, props) in creds.iter() {
            if let Some(name) = section {
                if !seen_names.contains(name) {
                    let has_key = props.get("aws_access_key_id").is_some();
                    profiles.push(ProfileInfo {
                        name: name.to_string(),
                        profile_type: if has_key {
                            ProfileType::Iam
                        } else {
                            ProfileType::Unknown
                        },
                        region: prop_str(&props, "region"),
                        ..ProfileInfo::default()
                    });
                    seen_names.insert(name.to_string());
                }
            }
        }
    }

    // Sort: "default" first, then alphabetical
    profiles.sort_by(|a, b| match (a.name.as_str(), b.name.as_str()) {
        ("default", _) => std::cmp::Ordering::Less,
        (_, "default") => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(profiles)
}

/// Classify a profile section by which keys are present.
///
/// Detection order (first match wins):
/// 1. `sso_start_url` or `sso_session` → Sso
/// 2. `credential_process` → CredentialProcess
/// 3. `role_arn` → AssumeRole
/// 4. `aws_access_key_id` → Iam
/// 5. Otherwise → Unknown
fn classify_profile(props: &ini::Properties) -> ProfileType {
    if props.get("sso_start_url").is_some() || props.get("sso_session").is_some() {
        ProfileType::Sso
    } else if props.get("credential_process").is_some() {
        ProfileType::CredentialProcess
    } else if props.get("role_arn").is_some() {
        ProfileType::AssumeRole
    } else if props.get("aws_access_key_id").is_some() {
        ProfileType::Iam
    } else {
        ProfileType::Unknown
    }
}

fn prop_str(props: &ini::Properties, key: &str) -> Option<String> {
    props.get(key).map(|s| s.to_string())
}

fn aws_config_path() -> PathBuf {
    std::env::var("AWS_CONFIG_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".aws/config")
        })
}

fn aws_credentials_path() -> PathBuf {
    std::env::var("AWS_SHARED_CREDENTIALS_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".aws/credentials")
        })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper: write config to temp files and parse directly (no env vars needed).
    fn parse_with_config(config_content: &str, creds_content: Option<&str>) -> Vec<ProfileInfo> {
        let dir = std::env::temp_dir().join(format!(
            "aws_profile_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let config_path = dir.join("config");
        let mut f = std::fs::File::create(&config_path).unwrap();
        f.write_all(config_content.as_bytes()).unwrap();

        let creds_path = dir.join("credentials");
        if let Some(content) = creds_content {
            let mut f = std::fs::File::create(&creds_path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }

        let result = parse_profiles_from(&config_path, &creds_path).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn parse_sso_profile() {
        let profiles = parse_with_config(
            "\
[profile my-sso]
sso_start_url = https://my-sso-portal.awsapps.com/start
sso_account_id = 123456789012
sso_role_name = ReadOnly
sso_region = us-east-1
region = us-west-2
",
            None,
        );

        assert_eq!(profiles.len(), 1);
        let p = &profiles[0];
        assert_eq!(p.name, "my-sso");
        assert_eq!(p.profile_type, ProfileType::Sso);
        assert_eq!(
            p.sso_start_url.as_deref(),
            Some("https://my-sso-portal.awsapps.com/start")
        );
        assert_eq!(p.sso_account_id.as_deref(), Some("123456789012"));
        assert_eq!(p.sso_role_name.as_deref(), Some("ReadOnly"));
        assert_eq!(p.region.as_deref(), Some("us-west-2"));
    }

    #[test]
    fn parse_sso_session_profile() {
        let profiles = parse_with_config(
            "\
[sso-session my-sso]
sso_start_url = https://my-sso-portal.awsapps.com/start
sso_region = us-east-1

[profile dev]
sso_session = my-sso
sso_account_id = 123456789012
sso_role_name = Developer
region = us-west-2
",
            None,
        );

        let dev = profiles.iter().find(|p| p.name == "dev").unwrap();
        assert_eq!(dev.profile_type, ProfileType::Sso);
        assert_eq!(dev.sso_session.as_deref(), Some("my-sso"));
    }

    #[test]
    fn parse_iam_profile() {
        let profiles = parse_with_config(
            "\
[profile static-keys]
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
region = eu-west-1
",
            None,
        );

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].profile_type, ProfileType::Iam);
    }

    #[test]
    fn parse_assume_role_profile() {
        let profiles = parse_with_config(
            "\
[profile base]
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = secret

[profile cross-account]
role_arn = arn:aws:iam::987654321098:role/CrossAccountRole
source_profile = base
mfa_serial = arn:aws:iam::123456789012:mfa/user
region = us-east-1
",
            None,
        );

        let cross = profiles.iter().find(|p| p.name == "cross-account").unwrap();
        assert_eq!(cross.profile_type, ProfileType::AssumeRole);
        assert_eq!(
            cross.role_arn.as_deref(),
            Some("arn:aws:iam::987654321098:role/CrossAccountRole")
        );
        assert_eq!(cross.source_profile.as_deref(), Some("base"));
        assert_eq!(
            cross.mfa_serial.as_deref(),
            Some("arn:aws:iam::123456789012:mfa/user")
        );
    }

    #[test]
    fn parse_credential_process_profile() {
        let profiles = parse_with_config(
            "\
[profile external]
credential_process = /usr/local/bin/my-credential-helper
region = ap-southeast-1
",
            None,
        );

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].profile_type, ProfileType::CredentialProcess);
    }

    #[test]
    fn parse_default_profile() {
        let profiles = parse_with_config(
            "\
[default]
region = us-east-1

[profile other]
region = eu-west-1
",
            None,
        );

        assert_eq!(profiles.len(), 2);
        // Default should sort first
        assert_eq!(profiles[0].name, "default");
        assert_eq!(profiles[1].name, "other");
    }

    #[test]
    fn credentials_only_profile_detected_as_iam() {
        let profiles = parse_with_config(
            "\
[default]
region = us-east-1
",
            Some(
                "\
[creds-only]
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = secret
",
            ),
        );

        let creds = profiles.iter().find(|p| p.name == "creds-only").unwrap();
        assert_eq!(creds.profile_type, ProfileType::Iam);
    }

    #[test]
    fn credentials_file_does_not_duplicate_config_profile() {
        let profiles = parse_with_config(
            "\
[profile shared]
region = us-east-1
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = secret
",
            Some(
                "\
[shared]
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = different-secret
",
            ),
        );

        let count = profiles.iter().filter(|p| p.name == "shared").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn mixed_profile_types() {
        let profiles = parse_with_config(
            "\
[default]
region = us-east-1
aws_access_key_id = AKIAIOSFODNN7EXAMPLE
aws_secret_access_key = secret

[profile sso-dev]
sso_start_url = https://example.awsapps.com/start
sso_account_id = 111111111111
sso_role_name = Developer
region = us-west-2

[profile role-admin]
role_arn = arn:aws:iam::222222222222:role/Admin
source_profile = default
region = us-east-1

[profile ext-helper]
credential_process = /usr/local/bin/helper
",
            None,
        );

        assert_eq!(profiles.len(), 4);
        assert_eq!(profiles[0].name, "default");
        assert_eq!(profiles[0].profile_type, ProfileType::Iam);

        let sso = profiles.iter().find(|p| p.name == "sso-dev").unwrap();
        assert_eq!(sso.profile_type, ProfileType::Sso);

        let role = profiles.iter().find(|p| p.name == "role-admin").unwrap();
        assert_eq!(role.profile_type, ProfileType::AssumeRole);

        let ext = profiles.iter().find(|p| p.name == "ext-helper").unwrap();
        assert_eq!(ext.profile_type, ProfileType::CredentialProcess);
    }

    #[test]
    fn empty_profile_classified_as_unknown() {
        let profiles = parse_with_config(
            "\
[profile empty]
region = us-east-1
",
            None,
        );

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].profile_type, ProfileType::Unknown);
    }

    #[test]
    fn nonexistent_files_return_empty() {
        let config = Path::new("/nonexistent/path/config");
        let creds = Path::new("/nonexistent/path/credentials");
        let profiles = parse_profiles_from(config, creds).unwrap();
        assert!(profiles.is_empty());
    }
}
