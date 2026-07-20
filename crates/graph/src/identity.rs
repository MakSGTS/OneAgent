//! Stable identifiers for the knowledge graph.

use std::fmt::{self, Display, Formatter};

/// Stable identifier of a node in the Knowledge Graph.
///
/// A node identifier must be deterministic and must not depend on insertion
/// order, collection indexes, or process-specific state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(String);

impl NodeId {
    /// Creates a node identifier from its canonical string representation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the canonical string representation of the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier and returns its canonical string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for NodeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Stable identifier of an edge in the Knowledge Graph.
///
/// Edge identifiers are deterministic and are derived from the semantic
/// identity of the relation rather than from graph insertion order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(String);

impl EdgeId {
    /// Creates an edge identifier from its canonical string representation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the canonical string representation of the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier and returns its canonical string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for EdgeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for EdgeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<String> for EdgeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for EdgeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{EdgeId, NodeId};

    #[test]
    fn node_id_preserves_canonical_value() {
        let identifier = NodeId::new("oneagent://metadata/catalog/Products");

        assert_eq!(identifier.as_str(), "oneagent://metadata/catalog/Products");
        assert_eq!(
            identifier.to_string(),
            "oneagent://metadata/catalog/Products"
        );
    }

    #[test]
    fn node_id_supports_deterministic_ordering() {
        let first = NodeId::new("oneagent://metadata/catalog/Products");
        let second = NodeId::new("oneagent://metadata/document/SalesInvoice");

        assert!(first < second);
    }

    #[test]
    fn edge_id_preserves_canonical_value() {
        let identifier =
            EdgeId::new("oneagent://edge/metadata/catalog/Products/has-attribute/Code");

        assert_eq!(
            identifier.as_str(),
            "oneagent://edge/metadata/catalog/Products/has-attribute/Code"
        );
    }

    #[test]
    fn identifiers_can_be_created_from_strings() {
        let node = NodeId::from(String::from(
            "oneagent://module/Catalog.Products.ObjectModule",
        ));
        let edge = EdgeId::from("oneagent://edge/example");

        assert_eq!(
            node.as_str(),
            "oneagent://module/Catalog.Products.ObjectModule"
        );
        assert_eq!(edge.as_str(), "oneagent://edge/example");
    }
}
