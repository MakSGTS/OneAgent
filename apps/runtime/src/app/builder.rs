//! Runtime application builder.

use crate::app::{App, Lifecycle, LifecycleState};
use crate::config::{ConfigurationProvider, RuntimeConfig};
use crate::error::RuntimeError;
use crate::state::AppState;

/// Builds a fully initialized [`App`].
#[derive(Debug)]
pub struct AppBuilder {
    configuration: Option<RuntimeConfig>,
    lifecycle: Lifecycle,
}

impl AppBuilder {
    /// Creates an empty application builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            configuration: None,
            lifecycle: Lifecycle::new(),
        }
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

        Ok(App::new(AppState::new(configuration), self.lifecycle))
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
    }
}
