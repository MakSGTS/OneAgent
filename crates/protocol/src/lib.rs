//! Protocol contracts used by `OneAgent` clients and services.

/// Returns the protocol component name.
#[must_use]
pub const fn component_name() -> &'static str {
    "protocol"
}
