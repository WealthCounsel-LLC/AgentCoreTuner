//! AWS credential management module.
//!
//! Provides profile enumeration, credential resolution, SSO login,
//! and credential status tracking.

pub mod keychain;
pub mod mfa;
pub mod profile_parser;
pub mod sso_cache;
pub mod sso_flow;
pub mod types;

use std::collections::HashMap;
use std::sync::Mutex;
use types::{CredentialStatus, ProfileInfo, ProfileType, ResolvedCredentials};

/// Central facade for all credential operations.
///
/// Owns a Tokio runtime for executing async AWS SDK calls.
/// Safe to call blocking methods from `std::thread::spawn` contexts.
pub struct CredentialManager {
    profiles: HashMap<String, ProfileInfo>,
    rt: tokio::runtime::Runtime,
    /// Cached resolved credentials per profile (in-memory only).
    cached_creds: Mutex<HashMap<String, ResolvedCredentials>>,
}

impl CredentialManager {
    /// Create an empty CredentialManager (fallback when config parsing fails).
    pub fn empty() -> Self {
        Self {
            profiles: HashMap::new(),
            rt: tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime"),
            cached_creds: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new CredentialManager by parsing AWS config files.
    pub fn new() -> Result<Self, String> {
        let parsed = profile_parser::parse_profiles()?;
        let profiles = parsed
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
        Ok(Self {
            profiles,
            rt,
            cached_creds: Mutex::new(HashMap::new()),
        })
    }

    /// Get info for a specific profile by name.
    pub fn get_profile_info(&self, name: &str) -> Option<&ProfileInfo> {
        self.profiles.get(name)
    }

    /// List all known profiles, sorted alphabetically with `default` first.
    pub fn list_profiles(&self) -> Vec<&ProfileInfo> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| match (a.name.as_str(), b.name.as_str()) {
            ("default", _) => std::cmp::Ordering::Less,
            (_, "default") => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        profiles
    }

    /// Return profile names only (for populating UI dropdowns).
    pub fn profile_names(&self) -> Vec<String> {
        self.list_profiles()
            .iter()
            .map(|p| p.name.clone())
            .collect()
    }

    /// Get the current credential status for a profile.
    pub fn get_status(&self, profile: &str) -> CredentialStatus {
        // Check in-memory cache first
        if let Ok(cache) = self.cached_creds.lock() {
            if let Some(creds) = cache.get(profile) {
                if let Some(expires) = creds.expires_at {
                    if expires > std::time::SystemTime::now() {
                        let five_min = std::time::Duration::from_secs(300);
                        return if expires
                            < std::time::SystemTime::now() + five_min
                        {
                            CredentialStatus::Expiring
                        } else {
                            CredentialStatus::Valid
                        };
                    } else {
                        return CredentialStatus::Expired;
                    }
                }
                return CredentialStatus::Valid;
            }
        }

        let info = match self.profiles.get(profile) {
            Some(info) => info,
            None => return CredentialStatus::NotAuthenticated,
        };

        match &info.profile_type {
            ProfileType::Sso => self.sso_status(info),
            ProfileType::Iam => CredentialStatus::Valid,
            ProfileType::AssumeRole => CredentialStatus::NotAuthenticated,
            ProfileType::CredentialProcess => CredentialStatus::Valid,
            ProfileType::Unknown => CredentialStatus::NotAuthenticated,
        }
    }

    /// Perform SSO login for a profile. Blocking — safe to call from a spawned thread.
    ///
    /// `on_progress` is called on the calling thread for each SSO flow phase.
    pub fn start_sso_login(
        &self,
        profile: &str,
        on_progress: impl Fn(sso_flow::SsoFlowProgress) + Send + 'static,
    ) -> Result<ResolvedCredentials, String> {
        let info = self.profiles.get(profile).ok_or_else(|| {
            if self.profiles.is_empty() {
                "No AWS profiles found. Create ~/.aws/config with an SSO profile first.".to_string()
            } else {
                format!(
                    "Profile '{profile}' not found. Available: {}",
                    self.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            }
        })?;

        match &info.profile_type {
            ProfileType::Sso => {}
            other => {
                return Err(format!(
                    "Profile '{profile}' is type {other}, not SSO. \
                     SSO login requires an SSO profile with sso_start_url in ~/.aws/config."
                ))
            }
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

        let (_, creds) = self.rt.block_on(sso_flow::perform_sso_login(
            start_url,
            sso_region,
            account_id,
            role_name,
            on_progress,
        ))?;

        // Cache the resolved credentials
        if let Ok(mut cache) = self.cached_creds.lock() {
            cache.insert(profile.to_string(), creds.clone());
        }

        Ok(creds)
    }

    /// Try to resolve credentials from cache (SSO token cache → GetRoleCredentials).
    /// Blocking — safe to call from a spawned thread.
    pub fn resolve_sso_from_cache(
        &self,
        profile: &str,
    ) -> Result<ResolvedCredentials, String> {
        let info = self
            .profiles
            .get(profile)
            .ok_or_else(|| format!("Profile not found: {profile}"))?;

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

        let cache_entry = sso_cache::find_cached_token(start_url)
            .ok_or_else(|| format!("No valid SSO token cached for {start_url}"))?;

        let creds = self.rt.block_on(sso_flow::get_role_credentials_from_cache(
            &cache_entry,
            account_id,
            role_name,
        ))?;

        // Cache the resolved credentials
        if let Ok(mut cache) = self.cached_creds.lock() {
            cache.insert(profile.to_string(), creds.clone());
        }

        Ok(creds)
    }

    /// Get cached resolved credentials for a profile (if available).
    pub fn get_cached_credentials(
        &self,
        profile: &str,
    ) -> Option<ResolvedCredentials> {
        self.cached_creds
            .lock()
            .ok()
            .and_then(|cache| cache.get(profile).cloned())
    }

    /// Assume an IAM role for a profile that has `role_arn`.
    /// If the profile has `mfa_serial`, the `mfa_code` must be provided.
    /// Blocking — safe to call from a spawned thread.
    pub fn assume_role_for_profile(
        &self,
        profile: &str,
        mfa_code: Option<&str>,
    ) -> Result<ResolvedCredentials, String> {
        let info = self
            .profiles
            .get(profile)
            .ok_or_else(|| format!("Profile not found: {profile}"))?;

        let role_arn = info
            .role_arn
            .as_deref()
            .ok_or("Profile missing role_arn")?;
        let session_name = format!("agentcore-{}", profile.replace(' ', "-"));

        let region = info
            .region
            .as_deref()
            .unwrap_or("us-east-1");

        // Build config for the source profile (if any) to provide base credentials
        let config = self.rt.block_on(async {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(region.to_string()));

            // If source profile has cached creds, use them
            if let Some(source) = &info.source_profile {
                if let Some(source_creds) = self.get_cached_credentials(source) {
                    loader = loader.credentials_provider(
                        aws_sdk_sts::config::Credentials::new(
                            &source_creds.access_key_id,
                            source_creds.secret_access_key.expose(),
                            source_creds.session_token.clone(),
                            source_creds.expires_at.map(|t| t.into()),
                            "agentcore-manager",
                        ),
                    );
                }
            }

            loader.load().await
        });

        let creds = match (&info.mfa_serial, mfa_code) {
            (Some(mfa_serial), Some(code)) => {
                self.rt.block_on(mfa::assume_role_with_mfa(
                    &config,
                    role_arn,
                    &session_name,
                    mfa_serial,
                    code,
                ))?
            }
            (Some(mfa_serial), None) => {
                return Err(format!(
                    "MFA code required (device: {mfa_serial})"
                ));
            }
            (None, _) => {
                self.rt.block_on(mfa::assume_role(
                    &config,
                    role_arn,
                    &session_name,
                ))?
            }
        };

        // Cache the resolved credentials
        if let Ok(mut cache) = self.cached_creds.lock() {
            cache.insert(profile.to_string(), creds.clone());
        }

        Ok(creds)
    }

    /// Resolve credentials for a profile, using the appropriate method for
    /// its type. Checks in-memory cache first, then keychain for IAM profiles.
    ///
    /// Returns `None` if the profile needs an interactive flow (SSO login, MFA prompt).
    pub fn resolve_credentials(&self, profile: &str) -> Result<Option<ResolvedCredentials>, String> {
        // Check in-memory cache first
        if let Some(creds) = self.get_cached_credentials(profile) {
            if let Some(expires) = creds.expires_at {
                if expires > std::time::SystemTime::now() {
                    return Ok(Some(creds));
                }
                // Expired — fall through to re-resolve
            } else {
                return Ok(Some(creds));
            }
        }

        let info = self
            .profiles
            .get(profile)
            .ok_or_else(|| format!("Profile not found: {profile}"))?;

        match &info.profile_type {
            ProfileType::Iam => {
                // Try keychain first, then fall through (credentials file is handled by AWS SDK)
                match keychain::load_credentials(profile) {
                    Ok(Some(creds)) => {
                        if let Ok(mut cache) = self.cached_creds.lock() {
                            cache.insert(profile.to_string(), creds.clone());
                        }
                        Ok(Some(creds))
                    }
                    _ => Ok(None), // No keychain entry; AuthenticatedCli will fall back to --profile
                }
            }
            ProfileType::Sso => {
                // Try cached SSO token → GetRoleCredentials
                match self.resolve_sso_from_cache(profile) {
                    Ok(creds) => Ok(Some(creds)),
                    Err(_) => Ok(None), // Needs interactive SSO login
                }
            }
            ProfileType::AssumeRole => {
                // Try without MFA (will fail if MFA is required)
                if info.mfa_serial.is_some() {
                    return Ok(None); // Needs interactive MFA prompt
                }
                match self.assume_role_for_profile(profile, None) {
                    Ok(creds) => Ok(Some(creds)),
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None), // CredentialProcess/Unknown — let CLI handle it
        }
    }

    /// Refresh credentials for a profile, clearing any cached values first.
    /// Returns `true` if fresh credentials were obtained.
    pub fn refresh_credentials(&self, profile: &str) -> Result<bool, String> {
        // Clear in-memory cache for this profile
        if let Ok(mut cache) = self.cached_creds.lock() {
            cache.remove(profile);
        }

        match self.resolve_credentials(profile)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Check SSO credential status by reading the SSO token cache.
    fn sso_status(&self, info: &ProfileInfo) -> CredentialStatus {
        let start_url = match &info.sso_start_url {
            Some(url) => url.as_str(),
            None => return CredentialStatus::NotAuthenticated,
        };

        match sso_cache::find_cached_token(start_url) {
            Some(entry) => {
                if sso_cache::is_token_expiring_soon(&entry) {
                    CredentialStatus::Expiring
                } else {
                    CredentialStatus::Valid
                }
            }
            None => CredentialStatus::NotAuthenticated,
        }
    }
}

