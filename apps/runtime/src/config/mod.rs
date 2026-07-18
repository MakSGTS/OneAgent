//! Runtime configuration.

mod provider;

pub use provider::{ConfigurationProvider, DefaultConfigurationProvider};

/// Immutable runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    application_name: String,
    environment: String,
}

impl RuntimeConfig {
    /// Creates a new runtime configuration.
    #[must_use]
    pub fn new(application_name: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            application_name: application_name.into(),
            environment: environment.into(),
        }
    }

    /// Returns the application name.
    #[must_use]
    pub fn application_name(&self) -> &str {
        &self.application_name
    }

    /// Returns the current environment name.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new("OneAgent Runtime", "development")
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeConfig;

    #[test]
    fn default_configuration_is_valid() {
        let configuration = RuntimeConfig::default();

        assert_eq!(configuration.application_name(), "OneAgent Runtime");
        assert_eq!(configuration.environment(), "development");
    }
}
