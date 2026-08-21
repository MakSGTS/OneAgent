//! Runtime configuration.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod provider;

pub use provider::{ConfigurationProvider, DefaultConfigurationProvider};

/// Immutable runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    application_name: String,
    environment: String,
    http_bind_address: SocketAddr,
}

impl RuntimeConfig {
    /// Creates a new runtime configuration.
    #[must_use]
    pub fn new(application_name: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            application_name: application_name.into(),
            environment: environment.into(),
            http_bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000),
        }
    }

    /// Overrides the HTTP listener bind address.
    #[must_use]
    pub const fn with_http_bind_address(mut self, address: SocketAddr) -> Self {
        self.http_bind_address = address;
        self
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

    /// Returns the HTTP listener bind address.
    #[must_use]
    pub const fn http_bind_address(&self) -> SocketAddr {
        self.http_bind_address
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new("OneAgent Runtime", "development")
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::RuntimeConfig;

    #[test]
    fn default_configuration_is_valid() {
        let configuration = RuntimeConfig::default();

        assert_eq!(configuration.application_name(), "OneAgent Runtime");
        assert_eq!(configuration.environment(), "development");
        assert_eq!(
            configuration.http_bind_address(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000)
        );
    }

    #[test]
    fn http_bind_address_override_preserves_other_configuration() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let configuration = RuntimeConfig::new("Runtime", "test").with_http_bind_address(address);

        assert_eq!(configuration.application_name(), "Runtime");
        assert_eq!(configuration.environment(), "test");
        assert_eq!(configuration.http_bind_address(), address);
    }
}
