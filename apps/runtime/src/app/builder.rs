//! Runtime application builder.

use std::sync::Arc;

use crate::app::{App, Lifecycle, LifecycleState};
use crate::config::{ConfigurationProvider, RuntimeConfig};
use crate::error::RuntimeError;
use crate::health::RuntimeHealth;
use crate::service::{RuntimeService, ServiceContainerBuilder};
use crate::state::AppState;

/// Builds a fully initialized [`App`].
#[derive(Debug)]
pub struct AppBuilder {
    configuration: Option<RuntimeConfig>,
    lifecycle: Lifecycle,
    services: ServiceContainerBuilder,
}

impl AppBuilder {
    /// Creates an empty application builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            configuration: None,
            lifecycle: Lifecycle::new(),
            services: ServiceContainerBuilder::new(),
        }
    }

    /// Registers one uniquely named Runtime service in startup order.
    ///
    /// # Errors
    ///
    /// Returns a service identity error for an empty or duplicate name.
    pub fn register_service<S>(
        mut self,
        name: impl Into<String>,
        service: S,
    ) -> Result<Self, RuntimeError>
    where
        S: RuntimeService,
    {
        self.services = self.services.register(name, service)?;
        Ok(self)
    }

    /// Loads configuration from a provider.
    ///
    /// # Errors
    ///
    /// Returns a provider-specific error when configuration cannot be loaded.
    pub fn configure<P>(
        mut self,
        provider: &P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        P: ConfigurationProvider,
    {
        self.lifecycle
            .transition_to(LifecycleState::Building)
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)?;

        self.configuration = Some(provider.load()?);
        Ok(self)
    }

    /// Builds the runtime application.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::MissingConfiguration`] when no configuration provider was used.
    pub fn build(mut self) -> Result<App, RuntimeError> {
        let configuration = self
            .configuration
            .take()
            .ok_or(RuntimeError::MissingConfiguration)?;

        self.lifecycle.transition_to(LifecycleState::Initializing)?;

        let health = RuntimeHealth::new(self.lifecycle.subscribe());
        let state = Arc::new(AppState::with_health(configuration, health));
        let services = self.services.build(Arc::clone(&state));

        Ok(App::new(state, self.lifecycle, services))
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::DefaultConfigurationProvider;

    use super::AppBuilder;

    #[test]
    fn builder_requires_configuration() {
        let error = AppBuilder::new()
            .build()
            .expect_err("configuration must be required");

        assert_eq!(error.to_string(), "runtime configuration is missing");
    }

    #[test]
    fn builder_creates_application() {
        let application = AppBuilder::new()
            .configure(&DefaultConfigurationProvider)
            .expect("default configuration must load")
            .build()
            .expect("application must build");

        assert_eq!(
            application.state().configuration().application_name(),
            "OneAgent Runtime"
        );
        assert_eq!(
            application.state().health().snapshot().lifecycle(),
            crate::LifecycleState::Initializing
        );
        assert!(!application.state().health().snapshot().is_ready());
    }
}
