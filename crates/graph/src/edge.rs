//! Edges stored in the semantic graph.

use oneagent_common::EntityId;

use crate::EdgeKind;

/// Directed semantic edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphEdge {
    source: EntityId,
    target: EntityId,
    kind: EdgeKind,
}

impl GraphEdge {
    /// Creates a semantic edge.
    #[must_use]
    pub const fn new(source: EntityId, target: EntityId, kind: EdgeKind) -> Self {
        Self {
            source,
            target,
            kind,
        }
    }

    /// Returns the source node identifier.
    #[must_use]
    pub const fn source(&self) -> &EntityId {
        &self.source
    }

    /// Returns the target node identifier.
    #[must_use]
    pub const fn target(&self) -> &EntityId {
        &self.target
    }

    /// Returns the edge kind.
    #[must_use]
    pub const fn kind(&self) -> EdgeKind {
        self.kind
    }
}
