//! Authentication and profile management commands.

use crate::aws;
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
    Ok(aws::get_caller_identity(&profile, &region))
}

/// Initiate SSO login for a profile.
#[tauri::command]
pub async fn sso_login(
    _state: State<'_, AppState>,
    profile: String,
) -> Result<(), String> {
    aws::sso_login(&profile)
}
