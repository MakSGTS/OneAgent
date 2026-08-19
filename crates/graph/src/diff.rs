//! Deterministic semantic graph snapshot diffs.

use std::collections::BTreeMap;

use oneagent_common::EntityName;

use crate::{
    Confidence, EdgeId, EdgeKind, FactOrigin, GraphEdge, GraphNode, GraphNodePayload, NodeId,
    NodeKind, ProducerId, Provenance, ResolutionState, SemanticGraph,
    edge_identity::edge_id as stable_edge_id,
};

/// Deterministic diff between two semantic graph snapshots.
///
/// The diff is directional: `old -> new`. Added entities exist only in the new
/// graph, removed entities exist only in the old graph, and modified entities
/// share the same stable identity but differ in typed content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphDiff {
    added_nodes: Vec<NodeChange>,
    removed_nodes: Vec<NodeChange>,
    modified_nodes: Vec<NodeChange>,
    added_edges: Vec<EdgeChange>,
    removed_edges: Vec<EdgeChange>,
    modified_edges: Vec<EdgeChange>,
    summary: GraphDiffSummary,
}

impl SemanticGraphDiff {
    /// Compares two graph snapshots in the `old -> new` direction.
    ///
    /// The operation does not mutate either graph, does not run semantic
    /// resolution and does not depend on source adapter state.
    #[must_use]
    pub fn between(old: &SemanticGraph, new: &SemanticGraph) -> Self {
        let old_nodes = node_snapshots(old);
        let new_nodes = node_snapshots(new);
        let old_edges = edge_snapshots(old);
        let new_edges = edge_snapshots(new);

        let (added_nodes, removed_nodes, modified_nodes) = diff_nodes(&old_nodes, &new_nodes);
        let (added_edges, removed_edges, modified_edges) = diff_edges(&old_edges, &new_edges);
        let summary = GraphDiffSummary::new(
            added_nodes.len(),
            removed_nodes.len(),
            modified_nodes.len(),
            added_edges.len(),
            removed_edges.len(),
            modified_edges.len(),
        );

        Self {
            added_nodes,
            removed_nodes,
            modified_nodes,
            added_edges,
            removed_edges,
            modified_edges,
            summary,
        }
    }

    /// Returns nodes that exist only in the new graph.
    #[must_use]
    pub fn added_nodes(&self) -> &[NodeChange] {
        &self.added_nodes
    }

    /// Returns nodes that exist only in the old graph.
    #[must_use]
    pub fn removed_nodes(&self) -> &[NodeChange] {
        &self.removed_nodes
    }

    /// Returns nodes whose identity exists in both graphs but whose content changed.
    #[must_use]
    pub fn modified_nodes(&self) -> &[NodeChange] {
        &self.modified_nodes
    }

    /// Returns edges that exist only in the new graph.
    #[must_use]
    pub fn added_edges(&self) -> &[EdgeChange] {
        &self.added_edges
    }

    /// Returns edges that exist only in the old graph.
    #[must_use]
    pub fn removed_edges(&self) -> &[EdgeChange] {
        &self.removed_edges
    }

    /// Returns edges whose identity exists in both graphs but whose content changed.
    ///
    /// Current edge identity is `(source NodeId, target NodeId, EdgeKind)`.
    /// Therefore source, target or kind changes are represented as one removed
    /// edge and one added edge. Only provenance can currently produce a
    /// modified edge for the same edge identity.
    #[must_use]
    pub fn modified_edges(&self) -> &[EdgeChange] {
        &self.modified_edges
    }

    /// Returns compact change counters.
    #[must_use]
    pub const fn summary(&self) -> GraphDiffSummary {
        self.summary
    }

    /// Returns `true` when the diff has no node or edge changes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.summary.total_changes() == 0
    }
}

/// Compact directional diff counters.
///
/// `total_changes` is defined as the sum of added, removed and modified nodes
/// plus added, removed and modified edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphDiffSummary {
    nodes_added: usize,
    nodes_removed: usize,
    nodes_modified: usize,
    edges_added: usize,
    edges_removed: usize,
    edges_modified: usize,
}

impl GraphDiffSummary {
    const fn new(
        nodes_added: usize,
        nodes_removed: usize,
        nodes_modified: usize,
        edges_added: usize,
        edges_removed: usize,
        edges_modified: usize,
    ) -> Self {
        Self {
            nodes_added,
            nodes_removed,
            nodes_modified,
            edges_added,
            edges_removed,
            edges_modified,
        }
    }

    /// Returns the number of added nodes.
    #[must_use]
    pub const fn nodes_added(self) -> usize {
        self.nodes_added
    }

    /// Returns the number of removed nodes.
    #[must_use]
    pub const fn nodes_removed(self) -> usize {
        self.nodes_removed
    }

    /// Returns the number of modified nodes.
    #[must_use]
    pub const fn nodes_modified(self) -> usize {
        self.nodes_modified
    }

    /// Returns the number of added edges.
    #[must_use]
    pub const fn edges_added(self) -> usize {
        self.edges_added
    }

    /// Returns the number of removed edges.
    #[must_use]
    pub const fn edges_removed(self) -> usize {
        self.edges_removed
    }

    /// Returns the number of modified edges.
    #[must_use]
    pub const fn edges_modified(self) -> usize {
        self.edges_modified
    }

    /// Returns the total number of node and edge changes.
    #[must_use]
    pub const fn total_changes(self) -> usize {
        self.nodes_added
            + self.nodes_removed
            + self.nodes_modified
            + self.edges_added
            + self.edges_removed
            + self.edges_modified
    }
}

/// Category of a node or edge change in an `old -> new` diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GraphChangeKind {
    /// The entity exists only in the new graph.
    Added,
    /// The entity exists only in the old graph.
    Removed,
    /// The entity exists in both graphs but typed content changed.
    Modified,
}

/// Modified node aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeModifiedAspect {
    /// Node semantic content changed.
    ///
    /// Current semantic node content is canonical name, [`NodeKind`] and typed payload.
    SemanticContent,
    /// Node provenance changed after order-insensitive normalization.
    Provenance,
}

/// Modified edge aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeModifiedAspect {
    /// Edge provenance changed after order-insensitive normalization.
    Provenance,
}

/// Owned snapshot of graph node state used by diff records.
///
/// The snapshot identity is [`NodeId`]. Mutable content currently consists of
/// canonical name, [`NodeKind`], typed payload and normalized provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    id: NodeId,
    name: EntityName,
    kind: NodeKind,
    payload: GraphNodePayload,
    provenance: Vec<Provenance>,
}

impl NodeSnapshot {
    fn from_node(node: &GraphNode) -> Self {
        Self {
            id: node_id(node.id().as_str()),
            name: node.name().clone(),
            kind: node.kind(),
            payload: node.payload().clone(),
            provenance: normalized_provenance(node.provenance()),
        }
    }

    /// Returns the stable node identity.
    #[must_use]
    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    /// Returns the canonical semantic name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns typed semantic content stored by the node.
    #[must_use]
    pub const fn payload(&self) -> &GraphNodePayload {
        &self.payload
    }

    /// Returns normalized provenance records.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    fn has_same_semantic_content(&self, other: &Self) -> bool {
        self.name == other.name && self.kind == other.kind && self.payload == other.payload
    }
}

/// Owned snapshot of graph edge state used by diff records.
///
/// Edge identity is derived from source [`NodeId`], target [`NodeId`] and
/// [`EdgeKind`]. With the current graph model, those fields define the whole
/// semantic relation identity; changing any of them produces a removed edge and
/// an added edge rather than a modified edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeSnapshot {
    id: EdgeId,
    source: NodeId,
    target: NodeId,
    kind: EdgeKind,
    provenance: Vec<Provenance>,
}

impl EdgeSnapshot {
    fn from_edge(edge: &GraphEdge) -> Self {
        let source = node_id(edge.source().as_str());
        let target = node_id(edge.target().as_str());
        Self {
            id: stable_edge_id(source.as_str(), target.as_str(), edge.kind()),
            source,
            target,
            kind: edge.kind(),
            provenance: normalized_provenance(edge.provenance()),
        }
    }

    /// Returns the stable edge identity.
    #[must_use]
    pub const fn id(&self) -> &EdgeId {
        &self.id
    }

    /// Returns the source node identity.
    #[must_use]
    pub const fn source(&self) -> &NodeId {
        &self.source
    }

    /// Returns the target node identity.
    #[must_use]
    pub const fn target(&self) -> &NodeId {
        &self.target
    }

    /// Returns the edge kind.
    #[must_use]
    pub const fn kind(&self) -> EdgeKind {
        self.kind
    }

    /// Returns normalized provenance records.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }
}

/// Directional node change record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeChange {
    id: NodeId,
    kind: GraphChangeKind,
    node_kind: NodeKind,
    old: Option<NodeSnapshot>,
    new: Option<NodeSnapshot>,
    modified_aspects: Vec<NodeModifiedAspect>,
}

impl NodeChange {
    fn added(node: NodeSnapshot) -> Self {
        Self {
            id: node.id.clone(),
            kind: GraphChangeKind::Added,
            node_kind: node.kind(),
            old: None,
            new: Some(node),
            modified_aspects: Vec::new(),
        }
    }

    fn removed(node: NodeSnapshot) -> Self {
        Self {
            id: node.id.clone(),
            kind: GraphChangeKind::Removed,
            node_kind: node.kind(),
            old: Some(node),
            new: None,
            modified_aspects: Vec::new(),
        }
    }

    fn modified(
        old: NodeSnapshot,
        new: NodeSnapshot,
        modified_aspects: Vec<NodeModifiedAspect>,
    ) -> Self {
        Self {
            id: old.id.clone(),
            kind: GraphChangeKind::Modified,
            node_kind: new.kind(),
            old: Some(old),
            new: Some(new),
            modified_aspects,
        }
    }

    /// Returns the node identity.
    #[must_use]
    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    /// Returns the change category.
    #[must_use]
    pub const fn kind(&self) -> GraphChangeKind {
        self.kind
    }

    /// Returns old node state for removed and modified nodes.
    #[must_use]
    pub const fn old(&self) -> Option<&NodeSnapshot> {
        self.old.as_ref()
    }

    /// Returns new node state for added and modified nodes.
    #[must_use]
    pub const fn new_state(&self) -> Option<&NodeSnapshot> {
        self.new.as_ref()
    }

    /// Returns typed modified aspects.
    #[must_use]
    pub fn modified_aspects(&self) -> &[NodeModifiedAspect] {
        &self.modified_aspects
    }

    /// Returns the relevant node kind for added/removed nodes or the new kind for modified nodes.
    #[must_use]
    pub const fn node_kind(&self) -> NodeKind {
        self.node_kind
    }
}

/// Directional edge change record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeChange {
    id: EdgeId,
    kind: GraphChangeKind,
    source: NodeId,
    target: NodeId,
    edge_kind: EdgeKind,
    old: Option<EdgeSnapshot>,
    new: Option<EdgeSnapshot>,
    modified_aspects: Vec<EdgeModifiedAspect>,
}

impl EdgeChange {
    fn added(edge: EdgeSnapshot) -> Self {
        Self {
            id: edge.id.clone(),
            kind: GraphChangeKind::Added,
            source: edge.source.clone(),
            target: edge.target.clone(),
            edge_kind: edge.kind,
            old: None,
            new: Some(edge),
            modified_aspects: Vec::new(),
        }
    }

    fn removed(edge: EdgeSnapshot) -> Self {
        Self {
            id: edge.id.clone(),
            kind: GraphChangeKind::Removed,
            source: edge.source.clone(),
            target: edge.target.clone(),
            edge_kind: edge.kind,
            old: Some(edge),
            new: None,
            modified_aspects: Vec::new(),
        }
    }

    fn modified(
        old: EdgeSnapshot,
        new: EdgeSnapshot,
        modified_aspects: Vec<EdgeModifiedAspect>,
    ) -> Self {
        Self {
            id: old.id.clone(),
            kind: GraphChangeKind::Modified,
            source: old.source.clone(),
            target: old.target.clone(),
            edge_kind: old.kind,
            old: Some(old),
            new: Some(new),
            modified_aspects,
        }
    }

    /// Returns the edge identity.
    #[must_use]
    pub const fn id(&self) -> &EdgeId {
        &self.id
    }

    /// Returns the change category.
    #[must_use]
    pub const fn kind(&self) -> GraphChangeKind {
        self.kind
    }

    /// Returns old edge state for removed and modified edges.
    #[must_use]
    pub const fn old(&self) -> Option<&EdgeSnapshot> {
        self.old.as_ref()
    }

    /// Returns new edge state for added and modified edges.
    #[must_use]
    pub const fn new_state(&self) -> Option<&EdgeSnapshot> {
        self.new.as_ref()
    }

    /// Returns typed modified aspects.
    #[must_use]
    pub fn modified_aspects(&self) -> &[EdgeModifiedAspect] {
        &self.modified_aspects
    }

    /// Returns the relevant source node identity.
    #[must_use]
    pub const fn source(&self) -> &NodeId {
        &self.source
    }

    /// Returns the relevant target node identity.
    #[must_use]
    pub const fn target(&self) -> &NodeId {
        &self.target
    }

    /// Returns the relevant edge kind.
    #[must_use]
    pub const fn edge_kind(&self) -> EdgeKind {
        self.edge_kind
    }
}

fn diff_nodes(
    old_nodes: &BTreeMap<NodeId, NodeSnapshot>,
    new_nodes: &BTreeMap<NodeId, NodeSnapshot>,
) -> (Vec<NodeChange>, Vec<NodeChange>, Vec<NodeChange>) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for (id, old) in old_nodes {
        match new_nodes.get(id) {
            Some(new) => {
                let mut aspects = Vec::new();
                if !old.has_same_semantic_content(new) {
                    aspects.push(NodeModifiedAspect::SemanticContent);
                }
                if old.provenance != new.provenance {
                    aspects.push(NodeModifiedAspect::Provenance);
                }
                if !aspects.is_empty() {
                    modified.push(NodeChange::modified(old.clone(), new.clone(), aspects));
                }
            }
            None => removed.push(NodeChange::removed(old.clone())),
        }
    }

    for (id, new) in new_nodes {
        if !old_nodes.contains_key(id) {
            added.push(NodeChange::added(new.clone()));
        }
    }

    (added, removed, modified)
}

fn diff_edges(
    old_edges: &BTreeMap<EdgeId, EdgeSnapshot>,
    new_edges: &BTreeMap<EdgeId, EdgeSnapshot>,
) -> (Vec<EdgeChange>, Vec<EdgeChange>, Vec<EdgeChange>) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for (id, old) in old_edges {
        match new_edges.get(id) {
            Some(new) => {
                if old.provenance != new.provenance {
                    modified.push(EdgeChange::modified(
                        old.clone(),
                        new.clone(),
                        vec![EdgeModifiedAspect::Provenance],
                    ));
                }
            }
            None => removed.push(EdgeChange::removed(old.clone())),
        }
    }

    for (id, new) in new_edges {
        if !old_edges.contains_key(id) {
            added.push(EdgeChange::added(new.clone()));
        }
    }

    (added, removed, modified)
}

fn node_snapshots(graph: &SemanticGraph) -> BTreeMap<NodeId, NodeSnapshot> {
    graph
        .nodes()
        .map(NodeSnapshot::from_node)
        .map(|snapshot| (snapshot.id.clone(), snapshot))
        .collect()
}

fn edge_snapshots(graph: &SemanticGraph) -> BTreeMap<EdgeId, EdgeSnapshot> {
    graph
        .edges()
        .map(EdgeSnapshot::from_edge)
        .map(|snapshot| (snapshot.id.clone(), snapshot))
        .collect()
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value.to_owned())
}

fn normalized_provenance(provenance: &[Provenance]) -> Vec<Provenance> {
    let mut provenance = provenance.to_vec();
    provenance.sort_by_key(provenance_key);
    provenance.dedup_by_key(|record| provenance_key(record));
    provenance
}

fn provenance_key(provenance: &Provenance) -> ProvenanceKey {
    ProvenanceKey {
        source: provenance.source().map(|source| source.as_str().to_owned()),
        producer: provenance.producer().clone(),
        origin: provenance.origin(),
        confidence: provenance.confidence(),
        resolution: provenance.resolution(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProvenanceKey {
    source: Option<String>,
    producer: ProducerId,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use crate::{
        Confidence, EdgeKind, EdgeModifiedAspect, FactOrigin, GraphChangeKind, GraphEdge,
        GraphNode, NodeKind, NodeModifiedAspect, ProducerId, Provenance, ResolutionState,
        SemanticGraph, SemanticGraphDiff,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance(source: &str) -> Provenance {
        Provenance::new(
            Some(id(source)),
            ProducerId::new("oneagent.graph.diff.tests"),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn node(id_value: &str, name_value: &str, kind: NodeKind) -> GraphNode {
        GraphNode::new(id(id_value), name(name_value), kind)
    }

    fn node_with_provenance(id_value: &str, name_value: &str, source: &str) -> GraphNode {
        GraphNode::new_with_provenance(
            id(id_value),
            name(name_value),
            NodeKind::Module,
            vec![provenance(source)],
        )
    }

    fn graph_with_node(node: GraphNode) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        graph.insert_node(node);
        graph
    }

    fn graph_with_edge(edge: GraphEdge) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        graph.insert_node(node("source", "Source", NodeKind::Module));
        graph.insert_node(node("target", "Target", NodeKind::Function));
        graph.insert_node(node("other", "Other", NodeKind::Procedure));
        graph.insert_edge(edge).expect("edge must be valid");
        graph
    }

    #[test]
    fn empty_graphs_create_empty_diff() {
        let old = SemanticGraph::new();
        let new = SemanticGraph::new();
        let diff = SemanticGraphDiff::between(&old, &new);

        assert!(diff.is_empty());
        assert_eq!(diff.summary().total_changes(), 0);
    }

    #[test]
    fn identical_graphs_create_empty_diff() {
        let graph = graph_with_node(node("module.sales", "Sales", NodeKind::Module));

        let diff = graph.diff(&graph);

        assert!(diff.is_empty());
    }

    #[test]
    fn detects_added_and_removed_nodes_directionally() {
        let old = graph_with_node(node("node.old", "Old", NodeKind::Module));
        let new = graph_with_node(node("node.new", "New", NodeKind::Module));
        let forward = old.diff(&new);
        let reverse = new.diff(&old);

        assert_eq!(forward.added_nodes().len(), 1);
        assert_eq!(forward.added_nodes()[0].id().as_str(), "node.new");
        assert_eq!(forward.removed_nodes().len(), 1);
        assert_eq!(forward.removed_nodes()[0].id().as_str(), "node.old");
        assert_eq!(reverse.added_nodes()[0].id().as_str(), "node.old");
        assert_eq!(reverse.removed_nodes()[0].id().as_str(), "node.new");
    }

    #[test]
    fn detects_node_semantic_content_change() {
        let old = graph_with_node(node("node.same", "OldName", NodeKind::Module));
        let new = graph_with_node(node("node.same", "NewName", NodeKind::Procedure));

        let diff = old.diff(&new);
        let change = &diff.modified_nodes()[0];

        assert_eq!(change.kind(), GraphChangeKind::Modified);
        assert_eq!(
            change.modified_aspects(),
            &[NodeModifiedAspect::SemanticContent]
        );
        assert_eq!(
            change.old().expect("old node must exist").name().as_str(),
            "OldName"
        );
        assert_eq!(
            change
                .new_state()
                .expect("new node must exist")
                .name()
                .as_str(),
            "NewName"
        );
    }

    #[test]
    fn detects_node_provenance_and_combined_change() {
        let old = graph_with_node(node_with_provenance("node.same", "Name", "source.old"));
        let provenance_only =
            graph_with_node(node_with_provenance("node.same", "Name", "source.new"));
        let combined = graph_with_node(GraphNode::new_with_provenance(
            id("node.same"),
            name("Renamed"),
            NodeKind::Function,
            vec![provenance("source.new")],
        ));

        let provenance_diff = old.diff(&provenance_only);
        let combined_diff = old.diff(&combined);

        assert_eq!(
            provenance_diff.modified_nodes()[0].modified_aspects(),
            &[NodeModifiedAspect::Provenance]
        );
        assert_eq!(
            combined_diff.modified_nodes()[0].modified_aspects(),
            &[
                NodeModifiedAspect::SemanticContent,
                NodeModifiedAspect::Provenance
            ]
        );
    }

    #[test]
    fn detects_added_removed_and_modified_edges() {
        let old_edge = GraphEdge::new_with_provenance(
            id("source"),
            id("target"),
            EdgeKind::Calls,
            vec![provenance("edge.old")],
        );
        let new_edge = GraphEdge::new_with_provenance(
            id("source"),
            id("target"),
            EdgeKind::Calls,
            vec![provenance("edge.new")],
        );
        let old = graph_with_edge(old_edge);
        let new = graph_with_edge(new_edge);

        let diff = old.diff(&new);
        let change = &diff.modified_edges()[0];

        assert_eq!(change.kind(), GraphChangeKind::Modified);
        assert_eq!(change.modified_aspects(), &[EdgeModifiedAspect::Provenance]);
        assert_eq!(change.source().as_str(), "source");
        assert_eq!(change.target().as_str(), "target");
        assert_eq!(change.edge_kind(), EdgeKind::Calls);
        assert_eq!(diff.summary().edges_modified(), 1);
    }

    #[test]
    fn edge_identity_change_is_removed_and_added() {
        let old = graph_with_edge(GraphEdge::new(id("source"), id("target"), EdgeKind::Calls));
        let new = graph_with_edge(GraphEdge::new(id("source"), id("other"), EdgeKind::Calls));

        let diff = old.diff(&new);

        assert_eq!(diff.removed_edges().len(), 1);
        assert_eq!(diff.added_edges().len(), 1);
        assert!(diff.modified_edges().is_empty());
        assert_eq!(diff.summary().edges_removed(), 1);
        assert_eq!(diff.summary().edges_added(), 1);
    }

    #[test]
    fn changes_are_sorted_deterministically() {
        let mut old = SemanticGraph::new();
        let mut new = SemanticGraph::new();

        old.insert_node(node("node.c", "C", NodeKind::Module));
        old.insert_node(node("node.a", "A", NodeKind::Module));
        new.insert_node(node("node.d", "D", NodeKind::Module));
        new.insert_node(node("node.b", "B", NodeKind::Module));

        let diff = old.diff(&new);

        assert_eq!(diff.removed_nodes()[0].id().as_str(), "node.a");
        assert_eq!(diff.removed_nodes()[1].id().as_str(), "node.c");
        assert_eq!(diff.added_nodes()[0].id().as_str(), "node.b");
        assert_eq!(diff.added_nodes()[1].id().as_str(), "node.d");
    }

    #[test]
    fn insertion_order_does_not_affect_diff() {
        let mut old_left = SemanticGraph::new();
        let mut old_right = SemanticGraph::new();
        let mut new_left = SemanticGraph::new();
        let mut new_right = SemanticGraph::new();

        old_left.insert_node(node("node.a", "A", NodeKind::Module));
        old_left.insert_node(node("node.b", "B", NodeKind::Procedure));
        old_right.insert_node(node("node.b", "B", NodeKind::Procedure));
        old_right.insert_node(node("node.a", "A", NodeKind::Module));
        new_left.insert_node(node("node.a", "A2", NodeKind::Module));
        new_left.insert_node(node("node.c", "C", NodeKind::Function));
        new_right.insert_node(node("node.c", "C", NodeKind::Function));
        new_right.insert_node(node("node.a", "A2", NodeKind::Module));

        assert_eq!(old_left.diff(&new_left), old_right.diff(&new_right));
    }

    #[test]
    fn provenance_order_does_not_affect_diff() {
        let left = graph_with_node(GraphNode::new_with_provenance(
            id("node.same"),
            name("Name"),
            NodeKind::Module,
            vec![provenance("source.a"), provenance("source.b")],
        ));
        let right = graph_with_node(GraphNode::new_with_provenance(
            id("node.same"),
            name("Name"),
            NodeKind::Module,
            vec![provenance("source.b"), provenance("source.a")],
        ));

        assert!(left.diff(&right).is_empty());
    }

    #[test]
    fn summary_counts_all_changes() {
        let old = graph_with_node(node("node.old", "Old", NodeKind::Module));
        let new = graph_with_node(node("node.new", "New", NodeKind::Function));

        let summary = old.diff(&new).summary();

        assert_eq!(summary.nodes_added(), 1);
        assert_eq!(summary.nodes_removed(), 1);
        assert_eq!(summary.total_changes(), 2);
    }

    #[test]
    fn diff_does_not_mutate_source_graphs_and_is_repeatable() {
        let old = graph_with_node(node("node.old", "Old", NodeKind::Module));
        let new = graph_with_node(node("node.new", "New", NodeKind::Function));
        let old_report = old.report();
        let new_report = new.report();

        let first = old.diff(&new);
        let second = old.diff(&new);

        assert_eq!(first, second);
        assert_eq!(old.report(), old_report);
        assert_eq!(new.report(), new_report);
    }
}
