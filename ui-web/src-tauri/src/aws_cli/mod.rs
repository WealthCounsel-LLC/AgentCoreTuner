//! Credential-aware AWS CLI execution layer.
//!
//! Provides `AuthenticatedCli` (a `CliRunner` that injects resolved credentials
//! as env vars) and error classification for auto-retry decisions.

pub mod error_classifier;
pub mod executor;
pub mod retry;

pub use error_classifier::{classify_error, ErrorClass};
pub use executor::AuthenticatedCli;
pub use retry::RetryRunner;
