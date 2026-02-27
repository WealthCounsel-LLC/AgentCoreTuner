//! Authentication and profile management commands.

use crate::aws;
use crate::credentials::profile_parser;
use crate::credentials::sso_flow;
use crate::credentials::types::ProfileType;
use crate::state::AppState;
use tauri::State;

/// List all AWS profiles from ~/.aws/config.
#[tauri::command]
pub async fn list_profiles() -> Result<Vec<String>, String> {
    Ok(aws::get_profiles())
}

/// Check if AWS CLI is installed and return version.
#[tauri::command]
pub async fn check_aws_cli() -> Result<String, String> {
    aws::check_aws_cli_available()
}

/// Get the caller identity (STS) for a profile/region.
#[tauri::command]
pub async fn get_caller_identity(profile: String, region: String) -> Result<String, String> {
    aws::get_caller_identity(&profile, &region)
}

/// Initiate SSO login for a profile.
///
/// For SSO profiles, uses the native OIDC device authorization flow which
/// opens the browser directly. For non-SSO profiles, returns a descriptive error.
#[tauri::command]
pub async fn sso_login(
    _state: State<'_, AppState>,
    profile: String,
) -> Result<(), String> {
    let profiles = profile_parser::parse_profiles().unwrap_or_default();
    let profile_info = profiles.iter().find(|p| p.name == profile);

    let info = match profile_info {
        Some(info) => info,
        None => {
            return Err(format!(
                "Profile '{}' not found in ~/.aws/config. Available profiles: {}",
                profile,
                profiles
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
    };

    if info.profile_type != ProfileType::Sso {
        return Err(format!(
            "Profile '{}' is type {} and does not support SSO login. \
             {} credentials are managed via ~/.aws/credentials or your configured credential source.",
            profile, info.profile_type, info.profile_type,
        ));
    }

    let start_url = info
        .sso_start_url
        .as_deref()
        .ok_or("SSO profile missing sso_start_url")?;
    let account_id = info
        .sso_account_id
        .as_deref()
        .ok_or("SSO profile missing sso_account_id")?;
    let role_name = info
        .sso_role_name
        .as_deref()
        .ok_or("SSO profile missing sso_role_name")?;
    let sso_region = info
        .sso_region
        .as_deref()
        .or(info.region.as_deref())
        .ok_or("SSO profile missing sso_region and region")?;

    // Call the native SSO flow directly (async — no Tokio runtime nesting).
    sso_flow::perform_sso_login(start_url, sso_region, account_id, role_name, |_progress| {})
        .await?;

    Ok(())
}
