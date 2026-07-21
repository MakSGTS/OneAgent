//! Deterministic graph-level impact analysis.
//!
//! Impact analysis consumes an already computed [`SemanticGraphDiff`] together
//! with the previous and current graph snapshots. It does not recompute the
//! diff, mutate graphs, read source files, run semantic resolution or perform
//! graph validation automatically.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Display, Formatter};

use crate::{
    EdgeChange, EdgeId, EdgeKind, NodeChange, NodeId, NodeKind, NodeModifiedAspect, SemanticGraph,
    SemanticGraphDiff, SemanticGraphEdgeFilter, SemanticGraphQuery,
};

/// Graph snapshot used by an impact reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactSnapshot {
    /// The reason comes from the previous graph snapshot.
    Previous,
    /// The reason comes from the current graph snapshot.
    Current,
}

/// Snapshot availability of an affected node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactNodeAvailability {
    /// The node exists only in the previous graph.
    PreviousOnly,
    /// The node exists only in the current graph.
    CurrentOnly,
    /// The node exists in both graph snapshots.
    Both,
}

/// High-level affected node status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactNodeStatus {
    /// The node is directly changed by the graph diff.
    DirectlyChanged,
    /// The node was reached transitively through propagation policy.
    TransitivelyAffected,
    /// The node was removed from the current graph.
    Removed,
}

/// Typed impact seed kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactSeedKind {
    /// Added node seed.
    NodeAdded,
    /// Removed node seed.
    NodeRemoved,
    /// Modified node seed.
    NodeModified,
    /// Added edge seed.
    EdgeAdded,
    /// Removed edge seed.
    EdgeRemoved,
    /// Modified edge seed.
    EdgeModified,
}

/// Typed reason for including a node in the affected set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactReasonKind {
    /// Node was added.
    NodeAdded,
    /// Node was removed.
    NodeRemoved,
    /// Node was modified.
    NodeModified,
    /// Edge touching the node was added.
    EdgeAdded,
    /// Edge touching the node was removed.
    EdgeRemoved,
    /// Edge touching the node was modified.
    EdgeModified,
    /// Node was reached through dependency-to-usage propagation.
    DependencyPropagation,
    /// Node was reached through ownership propagation.
    OwnershipPropagation,
}

/// Direction of impact propagation through a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactPropagationDirection {
    /// Dependency target changed, usage source is affected.
    DependencyToUsage,
    /// Child changed, owner is affected.
    ChildToOwner,
    /// Owner changed, child is affected.
    OwnerToChild,
}

/// Completeness status of an impact analysis result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpactCompleteness {
    /// Result is complete within the requested maximum depth.
    CompleteWithinRequestedDepth,
}

/// Ownership propagation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnershipImpactMode {
    /// Do not propagate impact through containment.
    Disabled,
    /// Propagate from child to owner.
    ChildToOwner,
    /// Propagate from owner to child.
    OwnerToChild,
    /// Propagate both from child to owner and owner to child.
    Bidirectional,
}

/// Provenance-only change handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProvenanceImpactMode {
    /// Ignore provenance-only modified node and edge changes.
    Exclude,
    /// Include provenance-only changes only as direct affected seeds.
    DirectOnly,
    /// Include and propagate provenance-only changes.
    Propagate,
}

/// Typed impact analysis options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticImpactOptions {
    max_depth: usize,
    edge_filter: SemanticGraphEdgeFilter,
    ownership_mode: OwnershipImpactMode,
    provenance_mode: ProvenanceImpactMode,
}

impl SemanticImpactOptions {
    /// Creates options with a strict maximum impact depth.
    ///
    /// Depth `0` means only direct seed nodes are returned. Default propagation
    /// includes dependency edge kinds from the Query API, disables ownership
    /// propagation and keeps provenance-only changes direct-only.
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            edge_filter: SemanticGraphEdgeFilter::All,
            ownership_mode: OwnershipImpactMode::Disabled,
            provenance_mode: ProvenanceImpactMode::DirectOnly,
        }
    }

    /// Returns the strict maximum traversal depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the dependency edge kind filter.
    #[must_use]
    pub const fn edge_filter(&self) -> &SemanticGraphEdgeFilter {
        &self.edge_filter
    }

    /// Returns the ownership propagation mode.
    #[must_use]
    pub const fn ownership_mode(&self) -> OwnershipImpactMode {
        self.ownership_mode
    }

    /// Returns the provenance-only change handling mode.
    #[must_use]
    pub const fn provenance_mode(&self) -> ProvenanceImpactMode {
        self.provenance_mode
    }

    /// Sets dependency edge kind filter.
    #[must_use]
    pub fn with_edge_filter(mut self, edge_filter: SemanticGraphEdgeFilter) -> Self {
        self.edge_filter = edge_filter;
        self
    }

    /// Sets ownership propagation mode.
    #[must_use]
    pub const fn with_ownership_mode(mut self, ownership_mode: OwnershipImpactMode) -> Self {
        self.ownership_mode = ownership_mode;
        self
    }

    /// Sets provenance-only change handling mode.
    #[must_use]
    pub const fn with_provenance_mode(mut self, provenance_mode: ProvenanceImpactMode) -> Self {
        self.provenance_mode = provenance_mode;
        self
    }
}

impl Default for SemanticImpactOptions {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Stable identity of a seed change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImpactSeed {
    kind: ImpactSeedKind,
    node_id: Option<NodeId>,
    edge_id: Option<EdgeId>,
}

impl ImpactSeed {
    fn node(kind: ImpactSeedKind, node_id: NodeId) -> Self {
        Self {
            kind,
            node_id: Some(node_id),
            edge_id: None,
        }
    }

    fn edge(kind: ImpactSeedKind, edge_id: EdgeId) -> Self {
        Self {
            kind,
            node_id: None,
            edge_id: Some(edge_id),
        }
    }

    /// Returns the seed kind.
    #[must_use]
    pub const fn kind(&self) -> ImpactSeedKind {
        self.kind
    }

    /// Returns the seed node identifier, when this is a node seed.
    #[must_use]
    pub const fn node_id(&self) -> Option<&NodeId> {
        self.node_id.as_ref()
    }

    /// Returns the seed edge identifier, when this is an edge seed.
    #[must_use]
    pub const fn edge_id(&self) -> Option<&EdgeId> {
        self.edge_id.as_ref()
    }
}

/// Typed reason for an affected node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImpactReason {
    kind: ImpactReasonKind,
    seed: ImpactSeed,
    source_node: Option<NodeId>,
    edge_id: Option<EdgeId>,
    edge_kind: Option<EdgeKind>,
    depth: usize,
    snapshot: ImpactSnapshot,
    propagation: Option<ImpactPropagationDirection>,
}

impl ImpactReason {
    fn new(
        kind: ImpactReasonKind,
        seed: ImpactSeed,
        depth: usize,
        snapshot: ImpactSnapshot,
    ) -> Self {
        Self {
            kind,
            seed,
            source_node: None,
            edge_id: None,
            edge_kind: None,
            depth,
            snapshot,
            propagation: None,
        }
    }

    fn with_source_node(mut self, source_node: NodeId) -> Self {
        self.source_node = Some(source_node);
        self
    }

    fn with_edge(mut self, edge_id: EdgeId, edge_kind: EdgeKind) -> Self {
        self.edge_id = Some(edge_id);
        self.edge_kind = Some(edge_kind);
        self
    }

    const fn with_propagation(mut self, propagation: ImpactPropagationDirection) -> Self {
        self.propagation = Some(propagation);
        self
    }

    /// Returns the reason kind.
    #[must_use]
    pub const fn kind(&self) -> ImpactReasonKind {
        self.kind
    }

    /// Returns the originating seed.
    #[must_use]
    pub const fn seed(&self) -> &ImpactSeed {
        &self.seed
    }

    /// Returns the node from which this reason propagated.
    #[must_use]
    pub const fn source_node(&self) -> Option<&NodeId> {
        self.source_node.as_ref()
    }

    /// Returns the reason edge identifier.
    #[must_use]
    pub const fn edge_id(&self) -> Option<&EdgeId> {
        self.edge_id.as_ref()
    }

    /// Returns the reason edge kind.
    #[must_use]
    pub const fn edge_kind(&self) -> Option<EdgeKind> {
        self.edge_kind
    }

    /// Returns depth from the originating seed.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the snapshot that supplied the reason.
    #[must_use]
    pub const fn snapshot(&self) -> ImpactSnapshot {
        self.snapshot
    }

    /// Returns the propagation direction, when this is a transitive reason.
    #[must_use]
    pub const fn propagation(&self) -> Option<ImpactPropagationDirection> {
        self.propagation
    }
}

/// One unique affected node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedNode {
    node_id: NodeId,
    node_kind: Option<NodeKind>,
    status: ImpactNodeStatus,
    availability: ImpactNodeAvailability,
    depth: usize,
    reasons: Vec<ImpactReason>,
}

impl AffectedNode {
    fn new(
        node_id: NodeId,
        node_kind: Option<NodeKind>,
        status: ImpactNodeStatus,
        availability: ImpactNodeAvailability,
        depth: usize,
        reason: ImpactReason,
    ) -> Self {
        Self {
            node_id,
            node_kind,
            status,
            availability,
            depth,
            reasons: vec![reason],
        }
    }

    /// Returns the affected node identifier.
    #[must_use]
    pub const fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns the node kind from an available snapshot.
    #[must_use]
    pub const fn node_kind(&self) -> Option<NodeKind> {
        self.node_kind
    }

    /// Returns the high-level affected status.
    #[must_use]
    pub const fn status(&self) -> ImpactNodeStatus {
        self.status
    }

    /// Returns snapshot availability of this node.
    #[must_use]
    pub const fn availability(&self) -> ImpactNodeAvailability {
        self.availability
    }

    /// Returns minimal depth from any seed.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the deterministic primary reason.
    ///
    /// # Panics
    ///
    /// Panics only if an `AffectedNode` is constructed without reasons. Public
    /// impact analysis constructors always attach at least one reason.
    #[must_use]
    pub fn primary_reason(&self) -> &ImpactReason {
        self.reasons
            .first()
            .expect("affected node must contain at least one reason")
    }

    /// Returns all unique reasons in deterministic order.
    #[must_use]
    pub fn reasons(&self) -> &[ImpactReason] {
        &self.reasons
    }

    fn add_reason(&mut self, status: ImpactNodeStatus, depth: usize, reason: ImpactReason) {
        self.status = merge_status(self.status, status);
        if depth < self.depth {
            self.depth = depth;
        }
        self.reasons.push(reason);
        self.reasons.sort();
        self.reasons.dedup();
    }
}

/// Compact impact analysis counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticImpactSummary {
    seed_node_changes: usize,
    seed_edge_changes: usize,
    directly_changed_nodes: usize,
    transitively_affected_nodes: usize,
    removed_nodes: usize,
    previous_only_nodes: usize,
    current_nodes: usize,
    max_reached_depth: usize,
    total_affected_nodes: usize,
    requested_max_depth: usize,
}

impl SemanticImpactSummary {
    fn new(
        seed_node_changes: usize,
        seed_edge_changes: usize,
        nodes: &[AffectedNode],
        requested_max_depth: usize,
    ) -> Self {
        let directly_changed_nodes = nodes
            .iter()
            .filter(|node| node.status() == ImpactNodeStatus::DirectlyChanged)
            .count();
        let transitively_affected_nodes = nodes
            .iter()
            .filter(|node| node.status() == ImpactNodeStatus::TransitivelyAffected)
            .count();
        let removed_nodes = nodes
            .iter()
            .filter(|node| node.status() == ImpactNodeStatus::Removed)
            .count();
        let previous_only_nodes = nodes
            .iter()
            .filter(|node| node.availability() == ImpactNodeAvailability::PreviousOnly)
            .count();
        let current_nodes = nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.availability(),
                    ImpactNodeAvailability::CurrentOnly | ImpactNodeAvailability::Both
                )
            })
            .count();
        let max_reached_depth = nodes
            .iter()
            .map(AffectedNode::depth)
            .max()
            .unwrap_or_default();

        Self {
            seed_node_changes,
            seed_edge_changes,
            directly_changed_nodes,
            transitively_affected_nodes,
            removed_nodes,
            previous_only_nodes,
            current_nodes,
            max_reached_depth,
            total_affected_nodes: nodes.len(),
            requested_max_depth,
        }
    }

    /// Returns seed node change count.
    #[must_use]
    pub const fn seed_node_changes(self) -> usize {
        self.seed_node_changes
    }

    /// Returns seed edge change count.
    #[must_use]
    pub const fn seed_edge_changes(self) -> usize {
        self.seed_edge_changes
    }

    /// Returns directly changed node count.
    #[must_use]
    pub const fn directly_changed_nodes(self) -> usize {
        self.directly_changed_nodes
    }

    /// Returns transitively affected node count.
    #[must_use]
    pub const fn transitively_affected_nodes(self) -> usize {
        self.transitively_affected_nodes
    }

    /// Returns removed affected node count.
    #[must_use]
    pub const fn removed_nodes(self) -> usize {
        self.removed_nodes
    }

    /// Returns nodes that only exist in the previous snapshot.
    #[must_use]
    pub const fn previous_only_nodes(self) -> usize {
        self.previous_only_nodes
    }

    /// Returns nodes that exist in the current snapshot.
    #[must_use]
    pub const fn current_nodes(self) -> usize {
        self.current_nodes
    }

    /// Returns maximum reached depth.
    #[must_use]
    pub const fn max_reached_depth(self) -> usize {
        self.max_reached_depth
    }

    /// Returns total unique affected nodes.
    #[must_use]
    pub const fn total_affected_nodes(self) -> usize {
        self.total_affected_nodes
    }

    /// Returns requested maximum depth.
    #[must_use]
    pub const fn requested_max_depth(self) -> usize {
        self.requested_max_depth
    }
}

/// Owned impact analysis result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticImpactResult {
    affected_nodes: Vec<AffectedNode>,
    summary: SemanticImpactSummary,
    completeness: ImpactCompleteness,
}

impl SemanticImpactResult {
    fn new(
        affected_nodes: Vec<AffectedNode>,
        seed_node_changes: usize,
        seed_edge_changes: usize,
        requested_max_depth: usize,
    ) -> Self {
        let summary = SemanticImpactSummary::new(
            seed_node_changes,
            seed_edge_changes,
            &affected_nodes,
            requested_max_depth,
        );

        Self {
            affected_nodes,
            summary,
            completeness: ImpactCompleteness::CompleteWithinRequestedDepth,
        }
    }

    /// Returns unique affected nodes sorted by `NodeId`.
    #[must_use]
    pub fn affected_nodes(&self) -> &[AffectedNode] {
        &self.affected_nodes
    }

    /// Returns compact counters.
    #[must_use]
    pub const fn summary(&self) -> SemanticImpactSummary {
        self.summary
    }

    /// Returns completeness status.
    #[must_use]
    pub const fn completeness(&self) -> ImpactCompleteness {
        self.completeness
    }

    /// Returns `true` when no affected nodes were found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.affected_nodes.is_empty()
    }
}

/// Typed impact analysis error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImpactAnalysisError {
    /// A node seed is absent from the graph snapshot where it must exist.
    MissingSeedNode {
        /// Missing node identifier.
        node: NodeId,
        /// Required snapshot.
        snapshot: ImpactSnapshot,
    },
    /// An edge seed is absent from the graph snapshot where it must exist.
    MissingSeedEdge {
        /// Missing edge identifier.
        edge: EdgeId,
        /// Required snapshot.
        snapshot: ImpactSnapshot,
    },
}

impl Display for ImpactAnalysisError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSeedNode { node, snapshot } => {
                write!(
                    formatter,
                    "impact seed node `{node}` is missing in {snapshot:?} snapshot"
                )
            }
            Self::MissingSeedEdge { edge, snapshot } => {
                write!(
                    formatter,
                    "impact seed edge `{edge}` is missing in {snapshot:?} snapshot"
                )
            }
        }
    }
}

impl std::error::Error for ImpactAnalysisError {}

/// Stateless graph-level impact analyzer.
#[derive(Debug, Default, Clone, Copy)]
pub struct SemanticImpactAnalyzer;

impl SemanticImpactAnalyzer {
    /// Analyzes graph-level impact for a directional `previous -> current` diff.
    ///
    /// The supplied diff must correspond to the two graph snapshots. Removed
    /// entities are analyzed against the previous graph; added entities are
    /// analyzed against the current graph; modified entities combine both
    /// snapshots. The result is deterministic and owned.
    ///
    /// # Errors
    ///
    /// Returns [`ImpactAnalysisError`] when the diff references a seed missing
    /// from the snapshot where that seed must exist.
    pub fn analyze(
        previous_graph: &SemanticGraph,
        current_graph: &SemanticGraph,
        diff: &SemanticGraphDiff,
        options: &SemanticImpactOptions,
    ) -> Result<SemanticImpactResult, ImpactAnalysisError> {
        let previous = previous_graph.query();
        let current = current_graph.query();
        let mut state = AnalysisState::new(previous_graph, current_graph, options);
        let seeds = Self::collect_seeds(&previous, &current, diff, options)?;
        let seed_node_changes =
            diff.added_nodes().len() + diff.removed_nodes().len() + diff.modified_nodes().len();
        let seed_edge_changes =
            diff.added_edges().len() + diff.removed_edges().len() + diff.modified_edges().len();

        for seed in &seeds {
            state.add_seed(seed);
        }

        state.propagate(&previous, &current);

        Ok(SemanticImpactResult::new(
            state.into_nodes(),
            seed_node_changes,
            seed_edge_changes,
            options.max_depth(),
        ))
    }

    fn collect_seeds(
        previous: &SemanticGraphQuery<'_>,
        current: &SemanticGraphQuery<'_>,
        diff: &SemanticGraphDiff,
        options: &SemanticImpactOptions,
    ) -> Result<Vec<SeedWork>, ImpactAnalysisError> {
        let mut seeds = Vec::new();

        for change in diff.added_nodes() {
            Self::push_node_seed(
                &mut seeds,
                current,
                change,
                ImpactSnapshot::Current,
                ImpactSeedKind::NodeAdded,
                ImpactReasonKind::NodeAdded,
                true,
            )?;
        }

        for change in diff.removed_nodes() {
            Self::push_node_seed(
                &mut seeds,
                previous,
                change,
                ImpactSnapshot::Previous,
                ImpactSeedKind::NodeRemoved,
                ImpactReasonKind::NodeRemoved,
                true,
            )?;
        }

        for change in diff.modified_nodes() {
            if options.provenance_mode() == ProvenanceImpactMode::Exclude
                && is_provenance_only_node_change(change)
            {
                continue;
            }

            let propagates = !is_provenance_only_node_change(change)
                || options.provenance_mode() == ProvenanceImpactMode::Propagate;

            Self::push_node_seed(
                &mut seeds,
                previous,
                change,
                ImpactSnapshot::Previous,
                ImpactSeedKind::NodeModified,
                ImpactReasonKind::NodeModified,
                propagates,
            )?;
            Self::push_node_seed(
                &mut seeds,
                current,
                change,
                ImpactSnapshot::Current,
                ImpactSeedKind::NodeModified,
                ImpactReasonKind::NodeModified,
                propagates,
            )?;
        }

        for change in diff.added_edges() {
            Self::push_edge_seed(
                &mut seeds,
                current,
                change,
                ImpactSnapshot::Current,
                ImpactSeedKind::EdgeAdded,
                ImpactReasonKind::EdgeAdded,
                true,
            )?;
        }

        for change in diff.removed_edges() {
            Self::push_edge_seed(
                &mut seeds,
                previous,
                change,
                ImpactSnapshot::Previous,
                ImpactSeedKind::EdgeRemoved,
                ImpactReasonKind::EdgeRemoved,
                true,
            )?;
        }

        for change in diff.modified_edges() {
            if options.provenance_mode() == ProvenanceImpactMode::Exclude {
                continue;
            }

            let propagates = options.provenance_mode() == ProvenanceImpactMode::Propagate;

            Self::push_edge_seed(
                &mut seeds,
                previous,
                change,
                ImpactSnapshot::Previous,
                ImpactSeedKind::EdgeModified,
                ImpactReasonKind::EdgeModified,
                propagates,
            )?;
            Self::push_edge_seed(
                &mut seeds,
                current,
                change,
                ImpactSnapshot::Current,
                ImpactSeedKind::EdgeModified,
                ImpactReasonKind::EdgeModified,
                propagates,
            )?;
        }

        seeds.sort();
        seeds.dedup();

        Ok(seeds)
    }

    fn push_node_seed(
        seeds: &mut Vec<SeedWork>,
        query: &SemanticGraphQuery<'_>,
        change: &NodeChange,
        snapshot: ImpactSnapshot,
        seed_kind: ImpactSeedKind,
        reason_kind: ImpactReasonKind,
        propagates: bool,
    ) -> Result<(), ImpactAnalysisError> {
        let node = query
            .node(change.id())
            .ok_or_else(|| ImpactAnalysisError::MissingSeedNode {
                node: change.id().clone(),
                snapshot,
            })?;
        let seed = ImpactSeed::node(seed_kind, change.id().clone());
        let reason = ImpactReason::new(reason_kind, seed.clone(), 0, snapshot);
        let status = match seed_kind {
            ImpactSeedKind::NodeRemoved => ImpactNodeStatus::Removed,
            _ => ImpactNodeStatus::DirectlyChanged,
        };

        seeds.push(SeedWork {
            node_id: change.id().clone(),
            node_kind: Some(node.kind()),
            snapshot,
            status,
            reason,
            propagates,
        });

        Ok(())
    }

    fn push_edge_seed(
        seeds: &mut Vec<SeedWork>,
        query: &SemanticGraphQuery<'_>,
        change: &EdgeChange,
        snapshot: ImpactSnapshot,
        seed_kind: ImpactSeedKind,
        reason_kind: ImpactReasonKind,
        propagates: bool,
    ) -> Result<(), ImpactAnalysisError> {
        let edge = query
            .edge(change.id())
            .ok_or_else(|| ImpactAnalysisError::MissingSeedEdge {
                edge: change.id().clone(),
                snapshot,
            })?;
        let seed = ImpactSeed::edge(seed_kind, change.id().clone());

        for node_id in [change.source(), change.target()] {
            let Some(node) = query.node(node_id) else {
                return Err(ImpactAnalysisError::MissingSeedNode {
                    node: node_id.clone(),
                    snapshot,
                });
            };
            let reason = ImpactReason::new(reason_kind, seed.clone(), 0, snapshot)
                .with_source_node(node_id.clone())
                .with_edge(change.id().clone(), change.edge_kind());

            seeds.push(SeedWork {
                node_id: node_id.clone(),
                node_kind: Some(node.kind()),
                snapshot,
                status: ImpactNodeStatus::DirectlyChanged,
                reason,
                propagates,
            });
        }

        debug_assert_eq!(edge.kind(), change.edge_kind());

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SeedWork {
    node_id: NodeId,
    node_kind: Option<NodeKind>,
    snapshot: ImpactSnapshot,
    status: ImpactNodeStatus,
    reason: ImpactReason,
    propagates: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct QueueItem {
    node_id: NodeId,
    snapshot: ImpactSnapshot,
    depth: usize,
    seed: ImpactSeed,
}

struct AnalysisState<'graph, 'options> {
    previous_graph: &'graph SemanticGraph,
    current_graph: &'graph SemanticGraph,
    options: &'options SemanticImpactOptions,
    affected: BTreeMap<NodeId, AffectedNode>,
    queue: VecDeque<QueueItem>,
    visited: BTreeMap<(ImpactSnapshot, NodeId), usize>,
}

impl<'graph, 'options> AnalysisState<'graph, 'options> {
    fn new(
        previous_graph: &'graph SemanticGraph,
        current_graph: &'graph SemanticGraph,
        options: &'options SemanticImpactOptions,
    ) -> Self {
        Self {
            previous_graph,
            current_graph,
            options,
            affected: BTreeMap::new(),
            queue: VecDeque::new(),
            visited: BTreeMap::new(),
        }
    }

    fn add_seed(&mut self, seed: &SeedWork) {
        self.add_affected(
            seed.node_id.clone(),
            seed.node_kind,
            seed.status,
            0,
            seed.reason.clone(),
        );

        if seed.propagates {
            let key = (seed.snapshot, seed.node_id.clone());
            self.visited.entry(key).or_insert(0);
            self.queue.push_back(QueueItem {
                node_id: seed.node_id.clone(),
                snapshot: seed.snapshot,
                depth: 0,
                seed: seed.reason.seed().clone(),
            });
        }
    }

    fn propagate(&mut self, previous: &SemanticGraphQuery<'_>, current: &SemanticGraphQuery<'_>) {
        while let Some(item) = self.queue.pop_front() {
            if item.depth == self.options.max_depth() {
                continue;
            }

            let query = match item.snapshot {
                ImpactSnapshot::Previous => previous,
                ImpactSnapshot::Current => current,
            };
            let mut steps = self.dependency_steps(query, &item);
            steps.extend(self.ownership_steps(query, &item));
            steps.sort();

            for step in steps {
                let next_depth = item.depth + 1;
                let reason = step.reason(item.seed.clone(), item.node_id.clone(), next_depth);
                self.add_affected(
                    step.node_id.clone(),
                    step.node_kind,
                    ImpactNodeStatus::TransitivelyAffected,
                    next_depth,
                    reason,
                );

                let key = (item.snapshot, step.node_id.clone());
                match self.visited.get(&key).copied() {
                    Some(existing_depth) if existing_depth <= next_depth => {}
                    _ => {
                        self.visited.insert(key, next_depth);
                        self.queue.push_back(QueueItem {
                            node_id: step.node_id,
                            snapshot: item.snapshot,
                            depth: next_depth,
                            seed: item.seed.clone(),
                        });
                    }
                }
            }
        }
    }

    fn dependency_steps(
        &self,
        query: &SemanticGraphQuery<'_>,
        item: &QueueItem,
    ) -> Vec<PropagationStep> {
        query
            .direct_usages_with_filter(&item.node_id, self.options.edge_filter())
            .into_iter()
            .map(|relation| PropagationStep {
                node_id: NodeId::new(relation.node().id().as_str().to_owned()),
                node_kind: Some(relation.node().kind()),
                edge_id: relation.edge_id().clone(),
                edge_kind: relation.edge().kind(),
                snapshot: item.snapshot,
                reason_kind: ImpactReasonKind::DependencyPropagation,
                propagation: ImpactPropagationDirection::DependencyToUsage,
            })
            .collect()
    }

    fn ownership_steps(
        &self,
        query: &SemanticGraphQuery<'_>,
        item: &QueueItem,
    ) -> Vec<PropagationStep> {
        let mut steps = Vec::new();

        if matches!(
            self.options.ownership_mode(),
            OwnershipImpactMode::ChildToOwner | OwnershipImpactMode::Bidirectional
        ) {
            for edge in query.owner_edges(&item.node_id) {
                if let Some(owner) = query.node_by_entity_id(edge.source()) {
                    steps.push(PropagationStep {
                        node_id: NodeId::new(owner.id().as_str().to_owned()),
                        node_kind: Some(owner.kind()),
                        edge_id: SemanticGraphQuery::edge_id(
                            &NodeId::new(edge.source().as_str().to_owned()),
                            &NodeId::new(edge.target().as_str().to_owned()),
                            edge.kind(),
                        ),
                        edge_kind: edge.kind(),
                        snapshot: item.snapshot,
                        reason_kind: ImpactReasonKind::OwnershipPropagation,
                        propagation: ImpactPropagationDirection::ChildToOwner,
                    });
                }
            }
        }

        if matches!(
            self.options.ownership_mode(),
            OwnershipImpactMode::OwnerToChild | OwnershipImpactMode::Bidirectional
        ) {
            for edge in query.outgoing_edges_by_kind(&item.node_id, EdgeKind::Contains) {
                if let Some(child) = query.node_by_entity_id(edge.target()) {
                    steps.push(PropagationStep {
                        node_id: NodeId::new(child.id().as_str().to_owned()),
                        node_kind: Some(child.kind()),
                        edge_id: SemanticGraphQuery::edge_id(
                            &NodeId::new(edge.source().as_str().to_owned()),
                            &NodeId::new(edge.target().as_str().to_owned()),
                            edge.kind(),
                        ),
                        edge_kind: edge.kind(),
                        snapshot: item.snapshot,
                        reason_kind: ImpactReasonKind::OwnershipPropagation,
                        propagation: ImpactPropagationDirection::OwnerToChild,
                    });
                }
            }
        }

        steps
    }

    fn add_affected(
        &mut self,
        node_id: NodeId,
        node_kind: Option<NodeKind>,
        status: ImpactNodeStatus,
        depth: usize,
        reason: ImpactReason,
    ) {
        let availability = self.availability(&node_id);
        let node_kind = node_kind.or_else(|| self.node_kind(&node_id));

        match self.affected.get_mut(&node_id) {
            Some(node) => node.add_reason(status, depth, reason),
            None => {
                self.affected.insert(
                    node_id.clone(),
                    AffectedNode::new(node_id, node_kind, status, availability, depth, reason),
                );
            }
        }
    }

    fn availability(&self, node_id: &NodeId) -> ImpactNodeAvailability {
        let previous = self.previous_graph.query().node(node_id).is_some();
        let current = self.current_graph.query().node(node_id).is_some();

        match (previous, current) {
            (true, true) => ImpactNodeAvailability::Both,
            (false, true) => ImpactNodeAvailability::CurrentOnly,
            (true | false, false) => ImpactNodeAvailability::PreviousOnly,
        }
    }

    fn node_kind(&self, node_id: &NodeId) -> Option<NodeKind> {
        self.current_graph
            .query()
            .node(node_id)
            .or_else(|| self.previous_graph.query().node(node_id))
            .map(crate::GraphNode::kind)
    }

    fn into_nodes(self) -> Vec<AffectedNode> {
        self.affected.into_values().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PropagationStep {
    node_id: NodeId,
    node_kind: Option<NodeKind>,
    edge_id: EdgeId,
    edge_kind: EdgeKind,
    snapshot: ImpactSnapshot,
    reason_kind: ImpactReasonKind,
    propagation: ImpactPropagationDirection,
}

impl PropagationStep {
    fn reason(&self, seed: ImpactSeed, source_node: NodeId, depth: usize) -> ImpactReason {
        ImpactReason::new(self.reason_kind, seed, depth, self.snapshot)
            .with_source_node(source_node)
            .with_edge(self.edge_id.clone(), self.edge_kind)
            .with_propagation(self.propagation)
    }
}

const fn merge_status(current: ImpactNodeStatus, incoming: ImpactNodeStatus) -> ImpactNodeStatus {
    match (current, incoming) {
        (ImpactNodeStatus::Removed, _) | (_, ImpactNodeStatus::Removed) => {
            ImpactNodeStatus::Removed
        }
        (ImpactNodeStatus::DirectlyChanged, _) | (_, ImpactNodeStatus::DirectlyChanged) => {
            ImpactNodeStatus::DirectlyChanged
        }
        _ => ImpactNodeStatus::TransitivelyAffected,
    }
}

fn is_provenance_only_node_change(change: &NodeChange) -> bool {
    change.modified_aspects() == [NodeModifiedAspect::Provenance]
}
