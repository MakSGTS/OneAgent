//! Shared application state.

use crate::app::LifecycleState;
use crate::config::RuntimeConfig;
use crate::health::RuntimeHealth;

/// Immutable state shared by runtime adapters and services.
#[derive(Debug, Clone)]
pub struct AppState {
    configuration: RuntimeConfig,
    health: RuntimeHealth,
}

impl AppState {
    /// Creates application state.
    #[must_use]
    pub fn new(configuration: RuntimeConfig) -> Self {
        Self {
            configuration,
            health: RuntimeHealth::detached(LifecycleState::Created),
        }
    }

    pub(crate) const fn with_health(configuration: RuntimeConfig, health: RuntimeHealth) -> Self {
        Self {
            configuration,
            health,
        }
    }

    /// Returns runtime configuration.
    #[must_use]
    pub const fn configuration(&self) -> &RuntimeConfig {
        &self.configuration
    }

    /// Returns read-only health derived from canonical Runtime lifecycle state.
    #[must_use]
    pub const fn health(&self) -> &RuntimeHealth {
        &self.health
    }
}
