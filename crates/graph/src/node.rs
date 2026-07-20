//! Nodes stored in the semantic graph.

use oneagent_common::{EntityId, EntityName};

use crate::{NodeKind, Provenance};

/// Semantic graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    provenance: Vec<Provenance>,
}

impl GraphNode {
    /// Creates a semantic graph node.
    #[must_use]
    pub const fn new(id: EntityId, name: EntityName, kind: NodeKind) -> Self {
        Self {
            id,
            name,
            kind,
            provenance: Vec::new(),
        }
    }

    /// Adds provenance and returns the node.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance.push(provenance);
        self
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

    /// Returns provenance records attached to the node.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Adds provenance to the node.
    pub fn add_provenance(&mut self, provenance: Provenance) {
        self.provenance.push(provenance);
    }
}
