//! Shared domain primitives for `OneAgent`.

mod hash;
mod source;

pub use hash::{sha256, sha256_hex};

pub use source::{
    MAX_SOURCE_PATH_BYTES, SourceLocation, SourcePath, SourcePathError, SourcePathErrorKind,
    SourcePosition, SourcePositionError, SourceSpan, SourceSpanError,
};

use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Stable identifier used by `OneAgent` domain entities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(String);

impl EntityId {
    /// Creates an identifier from a non-empty string.
    ///
    /// # Errors
    ///
    /// Returns [`EntityIdError`] when the supplied value is empty.
    pub fn new(value: impl Into<String>) -> Result<Self, EntityIdError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(EntityIdError);
        }

        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EntityId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for EntityId {
    type Err = EntityIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Error returned when an entity identifier is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityIdError;

impl Display for EntityIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("entity identifier must not be empty")
    }
}

impl std::error::Error for EntityIdError {}

/// Human-readable name of a domain entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityName(String);

impl EntityName {
    /// Creates a name from a non-empty string.
    ///
    /// # Errors
    ///
    /// Returns [`EntityNameError`] when the supplied value is empty.
    pub fn new(value: impl Into<String>) -> Result<Self, EntityNameError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(EntityNameError);
        }

        Ok(Self(value))
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EntityName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Error returned when an entity name is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityNameError;

impl Display for EntityNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("entity name must not be empty")
    }
}

impl std::error::Error for EntityNameError {}

#[cfg(test)]
mod tests {
    use super::{EntityId, EntityName};

    #[test]
    fn entity_id_rejects_empty_value() {
        assert!(EntityId::new("   ").is_err());
    }

    #[test]
    fn entity_name_preserves_value() {
        let name = EntityName::new("SalesDocument").expect("name must be valid");

        assert_eq!(name.as_str(), "SalesDocument");
    }
}
