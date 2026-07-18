//! Configuration providers.

use super::RuntimeConfig;

/// Supplies runtime configuration from an external source.
pub trait ConfigurationProvider {
    /// Loads configuration for `OneAgent Runtime`.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider cannot construct a valid configuration.
    fn load(&self) -> Result<RuntimeConfig, Box<dyn std::error::Error + Send + Sync>>;
}

/// Provides built-in defaults suitable for local development.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultConfigurationProvider;

impl ConfigurationProvider for DefaultConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RuntimeConfig::default())
    }
}
