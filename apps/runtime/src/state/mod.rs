//! Shared application state.

use crate::config::RuntimeConfig;

/// Immutable state shared by runtime adapters and services.
#[derive(Debug, Clone)]
pub struct AppState {
    configuration: RuntimeConfig,
}

impl AppState {
    /// Creates application state.
    #[must_use]
    pub const fn new(configuration: RuntimeConfig) -> Self {
        Self { configuration }
    }

    /// Returns runtime configuration.
    #[must_use]
    pub const fn configuration(&self) -> &RuntimeConfig {
        &self.configuration
    }
}
