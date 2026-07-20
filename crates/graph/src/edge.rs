//! Edges stored in the semantic graph.

use oneagent_common::EntityId;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::{EdgeKind, Provenance};

/// Directed semantic edge.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    source: EntityId,
    target: EntityId,
    kind: EdgeKind,
    provenance: Vec<Provenance>,
}

impl GraphEdge {
    /// Creates a semantic edge without provenance.
    #[must_use]
    pub const fn new(source: EntityId, target: EntityId, kind: EdgeKind) -> Self {
        Self::new_with_provenance(source, target, kind, Vec::new())
    }

    /// Creates a semantic edge with provenance records.
    #[must_use]
    pub const fn new_with_provenance(
        source: EntityId,
        target: EntityId,
        kind: EdgeKind,
        provenance: Vec<Provenance>,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            provenance,
        }
    }

    /// Adds provenance and returns the edge.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance.push(provenance);
        self
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

    /// Returns provenance records attached to the edge.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Adds provenance to the edge.
    pub fn add_provenance(&mut self, provenance: Provenance) {
        self.provenance.push(provenance);
    }
}

impl PartialEq for GraphEdge {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.target == other.target && self.kind == other.kind
    }
}

impl Eq for GraphEdge {}

impl PartialOrd for GraphEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GraphEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.source, &self.target, self.kind).cmp(&(&other.source, &other.target, other.kind))
    }
}

impl Hash for GraphEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.target.hash(state);
        self.kind.hash(state);
    }
}
