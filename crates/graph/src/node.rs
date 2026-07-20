//! Nodes stored in the semantic graph.

use oneagent_common::{EntityId, EntityName};

use crate::NodeKind;

/// Semantic graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
}

impl GraphNode {
    /// Creates a semantic graph node.
    #[must_use]
    pub const fn new(id: EntityId, name: EntityName, kind: NodeKind) -> Self {
        Self { id, name, kind }
    }

    /// Returns the node identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the node name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }
}
