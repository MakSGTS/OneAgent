//! Deterministic normalization for incremental semantic-index transitions.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EdgeId, EdgeKind, EdgeSnapshot, NodeId, NodeSnapshot, SemanticGraph, SemanticGraphDiff,
    edge_identity::edge_id,
};

/// One validated, deterministic change batch between two canonical snapshots.
#[derive(Debug)]
pub(crate) struct NormalizedSemanticIndexChanges<'previous, 'current> {
    previous: &'previous SemanticGraph,
    current: &'current SemanticGraph,
    operations: Vec<NormalizedSemanticIndexOperation>,
}

impl<'previous, 'current> NormalizedSemanticIndexChanges<'previous, 'current> {
    /// Derives and normalizes the canonical diff for one snapshot transition.
    pub(crate) fn between(
        previous: &'previous SemanticGraph,
        current: &'current SemanticGraph,
    ) -> Result<Self, IncrementalIndexChangeError> {
        let diff = SemanticGraphDiff::between(previous, current);
        Self::from_diff(previous, current, &diff)
    }

    /// Normalizes a supplied diff after proving that it belongs to this pair.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_diff(
        previous: &'previous SemanticGraph,
        current: &'current SemanticGraph,
        supplied: &SemanticGraphDiff,
    ) -> Result<Self, IncrementalIndexChangeError> {
        let canonical = SemanticGraphDiff::between(previous, current);
        if supplied != &canonical {
            return Err(IncrementalIndexChangeError::DiffSnapshotMismatch);
        }

        validate_graph_edge_identities(previous)?;
        validate_graph_edge_identities(current)?;

        let mut operations = Vec::with_capacity(canonical.summary().total_changes());
        let mut node_changes = BTreeSet::new();
        let mut edge_changes = BTreeSet::new();

        for change in canonical.removed_edges() {
            let snapshot = change.old().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidRemovedEdge(change.id().clone())
            })?;
            insert_edge_change(&mut edge_changes, snapshot.id())?;
            require_edge(previous, snapshot, SnapshotSide::Previous)?;
            operations.push(NormalizedSemanticIndexOperation::RemoveEdge(
                snapshot.clone(),
            ));
        }

        for change in canonical.removed_nodes() {
            let snapshot = change.old().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidRemovedNode(change.id().clone())
            })?;
            insert_node_change(&mut node_changes, snapshot.id())?;
            require_node(previous, snapshot.id(), SnapshotSide::Previous)?;
            operations.push(NormalizedSemanticIndexOperation::RemoveNode(
                snapshot.clone(),
            ));
        }

        for change in canonical.modified_nodes() {
            let old = change.old().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidModifiedNode(change.id().clone())
            })?;
            let new = change.new_state().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidModifiedNode(change.id().clone())
            })?;
            if old.id() != new.id() {
                return Err(IncrementalIndexChangeError::NodeIdentityChanged {
                    old: old.id().clone(),
                    new: new.id().clone(),
                });
            }
            insert_node_change(&mut node_changes, old.id())?;
            require_node(previous, old.id(), SnapshotSide::Previous)?;
            require_node(current, new.id(), SnapshotSide::Current)?;

            let operation = if old.name() != new.name() || old.kind() != new.kind() {
                NormalizedSemanticIndexOperation::ReplaceNode {
                    old: old.clone(),
                    new: new.clone(),
                }
            } else {
                NormalizedSemanticIndexOperation::RefreshNode {
                    old: old.clone(),
                    new: new.clone(),
                }
            };
            operations.push(operation);
        }

        for change in canonical.added_nodes() {
            let snapshot = change.new_state().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidAddedNode(change.id().clone())
            })?;
            insert_node_change(&mut node_changes, snapshot.id())?;
            require_node(current, snapshot.id(), SnapshotSide::Current)?;
            operations.push(NormalizedSemanticIndexOperation::AddNode(snapshot.clone()));
        }

        for change in canonical.added_edges() {
            let snapshot = change.new_state().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidAddedEdge(change.id().clone())
            })?;
            insert_edge_change(&mut edge_changes, snapshot.id())?;
            require_edge(current, snapshot, SnapshotSide::Current)?;
            require_current_endpoint(current, snapshot.id(), snapshot.source())?;
            require_current_endpoint(current, snapshot.id(), snapshot.target())?;
            operations.push(NormalizedSemanticIndexOperation::AddEdge(snapshot.clone()));
        }

        for change in canonical.modified_edges() {
            let old = change.old().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidModifiedEdge(change.id().clone())
            })?;
            let new = change.new_state().ok_or_else(|| {
                IncrementalIndexChangeError::InvalidModifiedEdge(change.id().clone())
            })?;
            if old.id() != new.id()
                || old.source() != new.source()
                || old.target() != new.target()
                || old.kind() != new.kind()
            {
                return Err(IncrementalIndexChangeError::EdgeIdentityChanged {
                    old: old.id().clone(),
                    new: new.id().clone(),
                });
            }
            insert_edge_change(&mut edge_changes, old.id())?;
            require_edge(previous, old, SnapshotSide::Previous)?;
            require_edge(current, new, SnapshotSide::Current)?;
            operations.push(NormalizedSemanticIndexOperation::RefreshEdge {
                old: old.clone(),
                new: new.clone(),
            });
        }

        validate_removed_node_dependencies(previous, &canonical)?;
        operations.sort_by(compare_operations);

        Ok(Self {
            previous,
            current,
            operations,
        })
    }

    pub(crate) const fn previous(&self) -> &'previous SemanticGraph {
        self.previous
    }

    pub(crate) const fn current(&self) -> &'current SemanticGraph {
        self.current
    }

    pub(crate) fn operations(&self) -> &[NormalizedSemanticIndexOperation] {
        &self.operations
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

/// One unique logical index operation in a normalized batch.
///
/// A replacement is represented once. Its old projection belongs to phase two
/// and its new projection belongs to phase three when a later task applies the
/// batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NormalizedSemanticIndexOperation {
    RemoveEdge(EdgeSnapshot),
    RemoveNode(NodeSnapshot),
    ReplaceNode {
        old: NodeSnapshot,
        new: NodeSnapshot,
    },
    AddNode(NodeSnapshot),
    AddEdge(EdgeSnapshot),
    RefreshNode {
        old: NodeSnapshot,
        new: NodeSnapshot,
    },
    RefreshEdge {
        old: EdgeSnapshot,
        new: EdgeSnapshot,
    },
}

impl NormalizedSemanticIndexOperation {
    fn order_key(&self) -> (u8, u8, &str, u8) {
        match self {
            Self::RemoveEdge(edge) => (0, 0, edge.id().as_str(), 0),
            Self::RemoveNode(node) => (1, 0, node.id().as_str(), 0),
            Self::ReplaceNode { old, .. } => (1, 0, old.id().as_str(), 1),
            Self::AddNode(node) => (2, 0, node.id().as_str(), 0),
            Self::AddEdge(edge) => (3, 0, edge.id().as_str(), 0),
            Self::RefreshNode { old, .. } => (4, 0, old.id().as_str(), 0),
            Self::RefreshEdge { old, .. } => (4, 1, old.id().as_str(), 0),
        }
    }
}

/// Typed validation failure returned before any index state can be changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IncrementalIndexChangeError {
    DiffSnapshotMismatch,
    DuplicateNodeChange(NodeId),
    DuplicateEdgeChange(EdgeId),
    InvalidAddedNode(NodeId),
    InvalidRemovedNode(NodeId),
    InvalidModifiedNode(NodeId),
    InvalidAddedEdge(EdgeId),
    InvalidRemovedEdge(EdgeId),
    InvalidModifiedEdge(EdgeId),
    MissingPreviousNode(NodeId),
    MissingCurrentNode(NodeId),
    MissingPreviousEdge(EdgeId),
    MissingCurrentEdge(EdgeId),
    MissingCurrentEdgeEndpoint { edge: EdgeId, node: NodeId },
    RetainedIncidentEdge { node: NodeId, edge: EdgeId },
    StableEdgeIdentityCollision(EdgeId),
    NodeIdentityChanged { old: NodeId, new: NodeId },
    EdgeIdentityChanged { old: EdgeId, new: EdgeId },
}

#[derive(Debug, Clone, Copy)]
enum SnapshotSide {
    Previous,
    Current,
}

fn compare_operations(
    left: &NormalizedSemanticIndexOperation,
    right: &NormalizedSemanticIndexOperation,
) -> Ordering {
    left.order_key().cmp(&right.order_key())
}

fn insert_node_change(
    changes: &mut BTreeSet<NodeId>,
    id: &NodeId,
) -> Result<(), IncrementalIndexChangeError> {
    if changes.insert(id.clone()) {
        Ok(())
    } else {
        Err(IncrementalIndexChangeError::DuplicateNodeChange(id.clone()))
    }
}

fn insert_edge_change(
    changes: &mut BTreeSet<EdgeId>,
    id: &EdgeId,
) -> Result<(), IncrementalIndexChangeError> {
    if changes.insert(id.clone()) {
        Ok(())
    } else {
        Err(IncrementalIndexChangeError::DuplicateEdgeChange(id.clone()))
    }
}

fn require_node(
    graph: &SemanticGraph,
    id: &NodeId,
    side: SnapshotSide,
) -> Result<(), IncrementalIndexChangeError> {
    if graph.nodes().any(|node| node.id().as_str() == id.as_str()) {
        return Ok(());
    }

    Err(match side {
        SnapshotSide::Previous => IncrementalIndexChangeError::MissingPreviousNode(id.clone()),
        SnapshotSide::Current => IncrementalIndexChangeError::MissingCurrentNode(id.clone()),
    })
}

fn require_edge(
    graph: &SemanticGraph,
    snapshot: &EdgeSnapshot,
    side: SnapshotSide,
) -> Result<(), IncrementalIndexChangeError> {
    if graph.edges().any(|edge| {
        edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()) == *snapshot.id()
    }) {
        return Ok(());
    }

    Err(match side {
        SnapshotSide::Previous => {
            IncrementalIndexChangeError::MissingPreviousEdge(snapshot.id().clone())
        }
        SnapshotSide::Current => {
            IncrementalIndexChangeError::MissingCurrentEdge(snapshot.id().clone())
        }
    })
}

fn require_current_endpoint(
    current: &SemanticGraph,
    edge: &EdgeId,
    node: &NodeId,
) -> Result<(), IncrementalIndexChangeError> {
    if current
        .nodes()
        .any(|candidate| candidate.id().as_str() == node.as_str())
    {
        Ok(())
    } else {
        Err(IncrementalIndexChangeError::MissingCurrentEdgeEndpoint {
            edge: edge.clone(),
            node: node.clone(),
        })
    }
}

fn validate_removed_node_dependencies(
    previous: &SemanticGraph,
    diff: &SemanticGraphDiff,
) -> Result<(), IncrementalIndexChangeError> {
    let removed_edges = diff
        .removed_edges()
        .iter()
        .map(|change| change.id().clone())
        .collect::<BTreeSet<_>>();

    for change in diff.removed_nodes() {
        for edge in previous.edges().filter(|edge| {
            edge.source().as_str() == change.id().as_str()
                || edge.target().as_str() == change.id().as_str()
        }) {
            let id = edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind());
            if !removed_edges.contains(&id) {
                return Err(IncrementalIndexChangeError::RetainedIncidentEdge {
                    node: change.id().clone(),
                    edge: id,
                });
            }
        }
    }

    Ok(())
}

fn validate_graph_edge_identities(
    graph: &SemanticGraph,
) -> Result<(), IncrementalIndexChangeError> {
    let mut identities = BTreeMap::<EdgeId, (NodeId, NodeId, EdgeKind)>::new();

    for edge in graph.edges() {
        let source = NodeId::new(edge.source().as_str());
        let target = NodeId::new(edge.target().as_str());
        let id = edge_id(source.as_str(), target.as_str(), edge.kind());
        let components = (source, target, edge.kind());

        if let Some(existing) = identities.insert(id.clone(), components.clone())
            && existing != components
        {
            return Err(IncrementalIndexChangeError::StableEdgeIdentityCollision(id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ptr;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::{CommonMetadataPayload, MetadataKind, MetadataPayload};

    use super::{
        IncrementalIndexChangeError, NormalizedSemanticIndexChanges,
        NormalizedSemanticIndexOperation,
    };
    use crate::{
        Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
        ResolutionState, SemanticGraph, SemanticGraphDiff,
        node::GraphNodePayload,
        semantic_index::{
            AcceptedSemanticIndex, SemanticIndexLifecycleError, SemanticIndexState,
            SemanticIndexStateError, SemanticNodeIndexError, SemanticNodeIndexState,
        },
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance(source: &str, origin: FactOrigin) -> Provenance {
        Provenance::new(
            Some(id(source)),
            ProducerId::new("oneagent.graph.incremental_index.tests"),
            origin,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn node(id_value: &str, name_value: &str, kind: NodeKind) -> GraphNode {
        GraphNode::new(id(id_value), name(name_value), kind)
    }

    fn node_with_provenance(
        id_value: &str,
        name_value: &str,
        kind: NodeKind,
        origin: FactOrigin,
    ) -> GraphNode {
        GraphNode::new_with_provenance(
            id(id_value),
            name(name_value),
            kind,
            vec![provenance(id_value, origin)],
        )
    }

    fn metadata_node_with_payload(
        id_value: &str,
        name_value: &str,
        synonym: &str,
        origin: FactOrigin,
    ) -> GraphNode {
        GraphNode::new_with_payload_and_provenance(
            id(id_value),
            name(name_value),
            NodeKind::Metadata(MetadataKind::Catalog),
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(Some(synonym.to_owned())),
                None,
            )),
            vec![provenance(id_value, origin)],
        )
        .expect("metadata payload must match the node kind")
    }

    fn edge(source: &str, target: &str, kind: EdgeKind, origin: FactOrigin) -> GraphEdge {
        GraphEdge::new_with_provenance(
            id(source),
            id(target),
            kind,
            vec![provenance(source, origin)],
        )
    }

    fn insert_nodes(graph: &mut SemanticGraph, nodes: impl IntoIterator<Item = GraphNode>) {
        for node in nodes {
            graph.insert_node(node);
        }
    }

    fn insert_edges(graph: &mut SemanticGraph, edges: impl IntoIterator<Item = GraphEdge>) {
        for edge in edges {
            graph
                .insert_edge(edge)
                .expect("test edge endpoints must exist");
        }
    }

    #[test]
    fn normalizes_empty_and_no_op_transitions() {
        let empty = SemanticGraph::new();
        let empty_batch = NormalizedSemanticIndexChanges::between(&empty, &empty)
            .expect("empty transition must normalize");
        assert!(empty_batch.is_empty());

        let mut graph = SemanticGraph::new();
        graph.insert_node(node("module.sales", "Sales", NodeKind::Module));
        let no_op = NormalizedSemanticIndexChanges::between(&graph, &graph)
            .expect("same-snapshot transition must normalize");

        assert!(no_op.is_empty());
        assert!(ptr::eq(no_op.previous(), no_op.current()));
    }

    #[test]
    fn emits_each_logical_change_once_in_total_phase_order() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("owner", "Owner", NodeKind::Module),
                node("removed", "Removed", NodeKind::Procedure),
                node("renamed", "Before", NodeKind::Procedure),
                node_with_provenance(
                    "refreshed",
                    "Refreshed",
                    NodeKind::Procedure,
                    FactOrigin::Parsed,
                ),
            ],
        );
        insert_edges(
            &mut previous,
            [
                edge("owner", "removed", EdgeKind::Contains, FactOrigin::Declared),
                edge("owner", "renamed", EdgeKind::Calls, FactOrigin::Parsed),
            ],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("owner", "Owner", NodeKind::Module),
                node("renamed", "After", NodeKind::Function),
                node_with_provenance(
                    "refreshed",
                    "Refreshed",
                    NodeKind::Procedure,
                    FactOrigin::Derived,
                ),
                node("added", "Added", NodeKind::Query),
            ],
        );
        insert_edges(
            &mut current,
            [
                edge("owner", "renamed", EdgeKind::Calls, FactOrigin::Resolved),
                edge("owner", "added", EdgeKind::References, FactOrigin::Resolved),
            ],
        );

        let batch = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("mixed transition must normalize");
        let variants = batch
            .operations()
            .iter()
            .map(|operation| match operation {
                NormalizedSemanticIndexOperation::RemoveEdge(_) => "remove_edge",
                NormalizedSemanticIndexOperation::RemoveNode(_) => "remove_node",
                NormalizedSemanticIndexOperation::ReplaceNode { .. } => "replace_node",
                NormalizedSemanticIndexOperation::AddNode(_) => "add_node",
                NormalizedSemanticIndexOperation::AddEdge(_) => "add_edge",
                NormalizedSemanticIndexOperation::RefreshNode { .. } => "refresh_node",
                NormalizedSemanticIndexOperation::RefreshEdge { .. } => "refresh_edge",
            })
            .collect::<Vec<_>>();

        assert_eq!(
            variants,
            [
                "remove_edge",
                "remove_node",
                "replace_node",
                "add_node",
                "add_edge",
                "refresh_node",
                "refresh_edge"
            ]
        );
    }

    #[test]
    fn represents_endpoint_and_kind_changes_as_remove_then_add() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("source", "Source", NodeKind::Module),
                node("old", "Old", NodeKind::Procedure),
                node("new", "New", NodeKind::Function),
            ],
        );
        insert_edges(
            &mut previous,
            [edge("source", "old", EdgeKind::Calls, FactOrigin::Resolved)],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("source", "Source", NodeKind::Module),
                node("old", "Old", NodeKind::Procedure),
                node("new", "New", NodeKind::Function),
            ],
        );
        insert_edges(
            &mut current,
            [edge(
                "source",
                "new",
                EdgeKind::References,
                FactOrigin::Resolved,
            )],
        );

        let batch = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("edge identity replacement must normalize");

        assert!(matches!(
            batch.operations(),
            [
                NormalizedSemanticIndexOperation::RemoveEdge(_),
                NormalizedSemanticIndexOperation::AddEdge(_)
            ]
        ));
    }

    #[test]
    fn duplicate_insertions_and_cancelling_history_collapse_at_snapshot_boundary() {
        let mut previous = SemanticGraph::new();
        previous.insert_node(node("owner", "Owner", NodeKind::Module));
        previous.insert_node(node("child", "Child", NodeKind::Procedure));
        previous.insert_node(node("child", "Child", NodeKind::Procedure));
        let relation = edge("owner", "child", EdgeKind::Contains, FactOrigin::Declared);
        previous
            .insert_edge(relation.clone())
            .expect("first relation must be accepted");
        previous
            .insert_edge(relation)
            .expect("duplicate relation must be collapsed");

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("child", "Child", NodeKind::Procedure),
                node("owner", "Owner", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut current,
            [edge(
                "owner",
                "child",
                EdgeKind::Contains,
                FactOrigin::Declared,
            )],
        );

        let batch = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("equivalent final snapshots must normalize");

        assert!(batch.is_empty());
    }

    #[test]
    fn rejects_a_diff_from_an_unrelated_snapshot_pair() {
        let mut previous = SemanticGraph::new();
        previous.insert_node(node("old", "Old", NodeKind::Module));
        let mut current = SemanticGraph::new();
        current.insert_node(node("new", "New", NodeKind::Module));
        let unrelated = SemanticGraphDiff::between(&previous, &current);

        let error = NormalizedSemanticIndexChanges::from_diff(&previous, &previous, &unrelated)
            .expect_err("wrong snapshot pair must be rejected");

        assert_eq!(error, IncrementalIndexChangeError::DiffSnapshotMismatch);
    }

    #[test]
    fn rejects_retained_incident_edges_when_a_node_is_removed() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("owner", "Owner", NodeKind::Module),
                node("child", "Child", NodeKind::Procedure),
            ],
        );
        insert_edges(
            &mut previous,
            [edge(
                "owner",
                "child",
                EdgeKind::Contains,
                FactOrigin::Declared,
            )],
        );

        let mut current = SemanticGraph::new();
        current.insert_node(node("owner", "Owner", NodeKind::Module));
        current.edges = previous.edges().cloned().collect::<BTreeSet<_>>();

        let error = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect_err("retained incident edge must be rejected");

        assert!(matches!(
            error,
            IncrementalIndexChangeError::RetainedIncidentEdge { .. }
        ));
    }

    #[test]
    fn rejects_added_edges_with_a_missing_current_endpoint() {
        let previous = SemanticGraph::new();
        let mut current = SemanticGraph::new();
        current.insert_node(node("source", "Source", NodeKind::Module));
        current.edges.insert(edge(
            "source",
            "missing",
            EdgeKind::Calls,
            FactOrigin::Resolved,
        ));

        let error = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect_err("dangling added edge must be rejected");

        assert!(matches!(
            error,
            IncrementalIndexChangeError::MissingCurrentEdgeEndpoint { .. }
        ));
    }

    #[test]
    fn repeated_and_reversed_construction_produce_identical_operations() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("z", "Z", NodeKind::Procedure),
                node("a", "A", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut previous,
            [edge("a", "z", EdgeKind::Contains, FactOrigin::Declared)],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("b", "B", NodeKind::Query),
                node("a", "A", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut current,
            [edge("a", "b", EdgeKind::Contains, FactOrigin::Declared)],
        );

        let first = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("first construction must normalize");
        let retry = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("retry must normalize");

        let mut previous_reversed = SemanticGraph::new();
        insert_nodes(
            &mut previous_reversed,
            [
                node("a", "A", NodeKind::Module),
                node("z", "Z", NodeKind::Procedure),
            ],
        );
        insert_edges(
            &mut previous_reversed,
            [edge("a", "z", EdgeKind::Contains, FactOrigin::Declared)],
        );
        let mut current_reversed = SemanticGraph::new();
        insert_nodes(
            &mut current_reversed,
            [
                node("a", "A", NodeKind::Module),
                node("b", "B", NodeKind::Query),
            ],
        );
        insert_edges(
            &mut current_reversed,
            [edge("a", "b", EdgeKind::Contains, FactOrigin::Declared)],
        );
        let reversed =
            NormalizedSemanticIndexChanges::between(&previous_reversed, &current_reversed)
                .expect("reversed insertion order must normalize");

        assert_eq!(first.operations(), retry.operations());
        assert_eq!(first.operations(), reversed.operations());
    }

    #[test]
    fn incrementally_updates_all_node_dimensions_and_matches_a_clean_rebuild() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("stable", "Stable", NodeKind::Module),
                node("shared.old", "Shared", NodeKind::Procedure),
                node("removed", "Removed", NodeKind::Query),
                node("replaced", "Before", NodeKind::Procedure),
                node_with_provenance(
                    "refreshed",
                    "Refreshed",
                    NodeKind::Attribute,
                    FactOrigin::Parsed,
                ),
            ],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("shared.new", "Shared", NodeKind::Procedure),
                node("replaced", "After", NodeKind::Function),
                node("shared.old", "Shared", NodeKind::Procedure),
                node("stable", "Stable", NodeKind::Module),
                node_with_provenance(
                    "refreshed",
                    "Refreshed",
                    NodeKind::Attribute,
                    FactOrigin::Derived,
                ),
            ],
        );

        let previous_state = SemanticNodeIndexState::from_graph(&previous);
        let unaffected_before = previous_state
            .ids_by_name(&name("Stable"))
            .expect("stable bucket must exist")
            .clone();
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("node transition must normalize");
        let incremental = previous_state
            .apply_changes(&changes)
            .expect("node projections must apply atomically");
        let rebuilt = SemanticNodeIndexState::from_graph(&current);

        assert_eq!(incremental, rebuilt);
        assert!(!incremental.ids().contains(&id("removed")));
        assert!(incremental.ids().contains(&id("shared.new")));
        assert!(incremental.ids_by_name(&name("Before")).is_none());
        assert_eq!(
            incremental
                .ids_by_name(&name("Shared"))
                .expect("duplicate-name bucket must exist")
                .iter()
                .map(EntityId::as_str)
                .collect::<Vec<_>>(),
            ["shared.new", "shared.old"]
        );
        assert_eq!(
            incremental
                .ids_by_kind(NodeKind::Function)
                .expect("new kind bucket must exist")
                .iter()
                .map(EntityId::as_str)
                .collect::<Vec<_>>(),
            ["replaced"]
        );
        assert_eq!(
            incremental
                .ids_by_name(&name("Stable"))
                .expect("stable bucket must remain"),
            &unaffected_before
        );
    }

    #[test]
    fn payload_and_provenance_refreshes_leave_node_lookup_keys_unchanged() {
        let mut previous = SemanticGraph::new();
        previous.insert_node(metadata_node_with_payload(
            "catalog.products",
            "Products",
            "Old synonym",
            FactOrigin::Parsed,
        ));
        let mut current = SemanticGraph::new();
        current.insert_node(metadata_node_with_payload(
            "catalog.products",
            "Products",
            "New synonym",
            FactOrigin::Derived,
        ));

        let previous_state = SemanticNodeIndexState::from_graph(&previous);
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("payload and provenance transition must normalize");

        assert!(matches!(
            changes.operations(),
            [NormalizedSemanticIndexOperation::RefreshNode { .. }]
        ));
        let incremental = previous_state
            .apply_changes(&changes)
            .expect("lookup-neutral refresh must apply");

        assert_eq!(incremental, previous_state);
        assert_eq!(incremental, SemanticNodeIndexState::from_graph(&current));
    }

    #[test]
    fn stale_node_state_fails_atomically_and_retry_is_deterministic() {
        let mut previous = SemanticGraph::new();
        previous.insert_node(node("node", "Before", NodeKind::Procedure));
        let mut current = SemanticGraph::new();
        current.insert_node(node("node", "After", NodeKind::Function));
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("replacement must normalize");

        let previous_state = SemanticNodeIndexState::from_graph(&previous);
        let first = previous_state
            .apply_changes(&changes)
            .expect("first application must succeed");
        let retry = previous_state
            .apply_changes(&changes)
            .expect("retry from the accepted previous state must succeed");
        assert_eq!(first, retry);

        let stale_before = first.clone();
        let error = first
            .apply_changes(&changes)
            .expect_err("already-current node state must be stale");
        assert_eq!(
            error,
            SemanticNodeIndexError::MissingBucketMember(id("node"))
        );
        assert_eq!(first, stale_before);
    }

    #[test]
    fn empty_reversed_and_multistep_node_transitions_match_clean_rebuilds() {
        let empty = SemanticGraph::new();
        let empty_state = SemanticNodeIndexState::from_graph(&empty);
        let empty_changes = NormalizedSemanticIndexChanges::between(&empty, &empty)
            .expect("empty transition must normalize");
        assert_eq!(
            empty_state
                .apply_changes(&empty_changes)
                .expect("empty state must apply"),
            empty_state
        );

        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("z", "Z", NodeKind::Procedure),
                node("a", "A", NodeKind::Module),
            ],
        );
        let mut middle = SemanticGraph::new();
        insert_nodes(
            &mut middle,
            [
                node("a", "A", NodeKind::Module),
                node("z", "Renamed", NodeKind::Function),
            ],
        );
        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("b", "B", NodeKind::Query),
                node("z", "Renamed", NodeKind::Function),
            ],
        );

        let first_changes = NormalizedSemanticIndexChanges::between(&previous, &middle)
            .expect("first step must normalize");
        let first = SemanticNodeIndexState::from_graph(&previous)
            .apply_changes(&first_changes)
            .expect("first step must apply");
        assert_eq!(first, SemanticNodeIndexState::from_graph(&middle));

        let second_changes = NormalizedSemanticIndexChanges::between(&middle, &current)
            .expect("second step must normalize");
        let second = first
            .apply_changes(&second_changes)
            .expect("second step must apply");
        assert_eq!(second, SemanticNodeIndexState::from_graph(&current));

        let mut reversed = SemanticGraph::new();
        insert_nodes(
            &mut reversed,
            [
                node("z", "Renamed", NodeKind::Function),
                node("b", "B", NodeKind::Query),
            ],
        );
        assert_eq!(second, SemanticNodeIndexState::from_graph(&reversed));
    }

    #[test]
    fn mixed_edge_changes_update_every_dimension_and_match_clean_rebuild() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("source", "Source", NodeKind::Module),
                node("old", "Old", NodeKind::Procedure),
                node("new", "New", NodeKind::Function),
                node("child", "Child", NodeKind::Attribute),
            ],
        );
        insert_edges(
            &mut previous,
            [
                edge("source", "old", EdgeKind::Calls, FactOrigin::Resolved),
                edge("source", "child", EdgeKind::Contains, FactOrigin::Declared),
                edge("source", "new", EdgeKind::References, FactOrigin::Parsed),
            ],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("child", "Child", NodeKind::Attribute),
                node("new", "New", NodeKind::Function),
                node("old", "Old", NodeKind::Procedure),
                node("source", "Source", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut current,
            [
                edge("source", "new", EdgeKind::Calls, FactOrigin::Resolved),
                edge("source", "child", EdgeKind::Contains, FactOrigin::Derived),
                edge("child", "source", EdgeKind::DependsOn, FactOrigin::Derived),
                edge("source", "new", EdgeKind::References, FactOrigin::Resolved),
            ],
        );

        let previous_state = SemanticIndexState::from_graph(&previous);
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("mixed edge transition must normalize");
        let incremental = previous_state
            .apply_changes(&changes)
            .expect("mixed edge transition must apply atomically");
        let rebuilt = SemanticIndexState::from_graph(&current);

        assert_eq!(incremental, rebuilt);
        assert_eq!(incremental.nodes(), rebuilt.nodes());
        assert_eq!(incremental.edges().edge_ids().len(), 4);
        assert_eq!(
            incremental
                .edges()
                .edge_ids_by_kind(EdgeKind::Calls)
                .expect("Calls bucket must exist")
                .len(),
            1
        );
        assert_eq!(
            incremental
                .edges()
                .outgoing(&id("source"))
                .expect("source adjacency must exist")
                .len(),
            3
        );
        assert_eq!(
            incremental
                .edges()
                .incoming(&id("source"))
                .expect("source incoming adjacency must exist")
                .len(),
            1
        );
    }

    #[test]
    fn node_replacement_rekeys_retained_containment_without_losing_invalid_states() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("owner.a", "Owner A", NodeKind::Module),
                node("owner.b", "Owner B", NodeKind::Module),
                node("child", "Before", NodeKind::Attribute),
            ],
        );
        insert_edges(
            &mut previous,
            [
                edge("owner.a", "child", EdgeKind::Contains, FactOrigin::Declared),
                edge("owner.b", "child", EdgeKind::Contains, FactOrigin::Declared),
                edge("child", "child", EdgeKind::Contains, FactOrigin::Derived),
                edge("child", "owner.a", EdgeKind::Contains, FactOrigin::Derived),
            ],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("owner.b", "Owner B", NodeKind::Module),
                node("child", "After", NodeKind::Query),
                node("owner.a", "Owner A", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut current,
            [
                edge("child", "owner.a", EdgeKind::Contains, FactOrigin::Derived),
                edge("child", "child", EdgeKind::Contains, FactOrigin::Derived),
                edge("owner.b", "child", EdgeKind::Contains, FactOrigin::Declared),
                edge("owner.a", "child", EdgeKind::Contains, FactOrigin::Declared),
            ],
        );

        let previous_state = SemanticIndexState::from_graph(&previous);
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("containment rekey transition must normalize");
        let incremental = previous_state
            .apply_changes(&changes)
            .expect("containment rekey must apply");

        assert_eq!(incremental, SemanticIndexState::from_graph(&current));
        assert!(
            incremental
                .edges()
                .children_by_name(&id("owner.a"), &name("Before"))
                .is_none()
        );
        assert!(
            incremental
                .edges()
                .children_by_name(&id("owner.a"), &name("After"))
                .expect("new child-name bucket must exist")
                .contains(&id("child"))
        );
        assert!(
            incremental
                .edges()
                .children_by_kind(&id("owner.b"), NodeKind::Query)
                .expect("new child-kind bucket must exist")
                .contains(&id("child"))
        );
        assert_eq!(
            incremental
                .edges()
                .owners(&id("child"))
                .expect("all invalid owner candidates must remain")
                .iter()
                .map(EntityId::as_str)
                .collect::<Vec<_>>(),
            ["child", "owner.a", "owner.b"]
        );
    }

    #[test]
    fn node_deletion_removes_all_incident_edge_membership() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("owner", "Owner", NodeKind::Module),
                node("child", "Child", NodeKind::Procedure),
            ],
        );
        insert_edges(
            &mut previous,
            [
                edge("owner", "child", EdgeKind::Contains, FactOrigin::Declared),
                edge("child", "owner", EdgeKind::Calls, FactOrigin::Resolved),
            ],
        );
        let mut current = SemanticGraph::new();
        current.insert_node(node("owner", "Owner", NodeKind::Module));

        let previous_state = SemanticIndexState::from_graph(&previous);
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("incident deletion must normalize");
        let incremental = previous_state
            .apply_changes(&changes)
            .expect("incident deletion must apply");

        assert_eq!(incremental, SemanticIndexState::from_graph(&current));
        assert!(incremental.edges().edge_ids().is_empty());
        assert!(incremental.edges().outgoing(&id("child")).is_none());
        assert!(incremental.edges().incoming(&id("child")).is_none());
        assert!(incremental.edges().owners(&id("child")).is_none());
    }

    #[test]
    fn composite_state_retry_is_deterministic_and_stale_failure_is_atomic() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("a", "A", NodeKind::Module),
                node("b", "B", NodeKind::Procedure),
            ],
        );
        insert_edges(
            &mut previous,
            [edge("a", "b", EdgeKind::Calls, FactOrigin::Resolved)],
        );
        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("b", "B", NodeKind::Procedure),
                node("a", "A", NodeKind::Module),
            ],
        );
        insert_edges(
            &mut current,
            [
                edge("b", "a", EdgeKind::Calls, FactOrigin::Resolved),
                edge("a", "b", EdgeKind::DependsOn, FactOrigin::Derived),
            ],
        );
        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("cycle replacement must normalize");
        let previous_state = SemanticIndexState::from_graph(&previous);

        let first = previous_state
            .apply_changes(&changes)
            .expect("first application must succeed");
        let retry = previous_state
            .apply_changes(&changes)
            .expect("retry must succeed");
        assert_eq!(first, retry);
        assert_eq!(first, SemanticIndexState::from_graph(&current));

        let unchanged = first.clone();
        let error = first
            .apply_changes(&changes)
            .expect_err("already-current composite state must be stale");
        assert!(matches!(
            error,
            SemanticIndexStateError::Edge(
                crate::semantic_index::SemanticEdgeIndexError::MissingEdge(_)
            )
        ));
        assert_eq!(first, unchanged);
    }

    #[test]
    fn accepted_lifecycle_query_and_resolution_match_clean_current_facades() {
        let mut previous = SemanticGraph::new();
        insert_nodes(
            &mut previous,
            [
                node("owner", "Owner", NodeKind::Module),
                node("child.a", "Before", NodeKind::Procedure),
            ],
        );
        insert_edges(
            &mut previous,
            [edge(
                "owner",
                "child.a",
                EdgeKind::Contains,
                FactOrigin::Declared,
            )],
        );

        let mut current = SemanticGraph::new();
        insert_nodes(
            &mut current,
            [
                node("child.b", "After", NodeKind::Function),
                node("owner", "Owner", NodeKind::Module),
                node("child.a", "After", NodeKind::Function),
            ],
        );
        insert_edges(
            &mut current,
            [
                edge("owner", "child.b", EdgeKind::Contains, FactOrigin::Declared),
                edge("owner", "child.a", EdgeKind::Contains, FactOrigin::Declared),
                edge("child.a", "owner", EdgeKind::Calls, FactOrigin::Resolved),
            ],
        );

        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("facade transition must normalize");
        let accepted = AcceptedSemanticIndex::rebuild(&previous)
            .transition(&current, &changes)
            .expect("facade transition must publish");
        let incremental_query = accepted.query();
        let clean_query = current.query();
        let node_ids = |nodes: Vec<&GraphNode>| {
            nodes
                .into_iter()
                .map(|node| node.id().clone())
                .collect::<Vec<_>>()
        };

        assert_eq!(
            node_ids(incremental_query.nodes_by_name(&name("After"))),
            node_ids(clean_query.nodes_by_name(&name("After")))
        );
        assert_eq!(
            node_ids(incremental_query.children(&crate::NodeId::new("owner"))),
            node_ids(clean_query.children(&crate::NodeId::new("owner")))
        );
        assert_eq!(
            incremental_query
                .edges()
                .into_iter()
                .map(|edge| (edge.source().clone(), edge.target().clone(), edge.kind()))
                .collect::<Vec<_>>(),
            clean_query
                .edges()
                .into_iter()
                .map(|edge| (edge.source().clone(), edge.target().clone(), edge.kind()))
                .collect::<Vec<_>>()
        );
        assert!(incremental_query.nodes_by_name(&name("Before")).is_empty());
        assert!(ptr::eq(
            incremental_query
                .node(&crate::NodeId::new("child.a"))
                .expect("current node must resolve"),
            current
                .node(&id("child.a"))
                .expect("canonical node must exist")
        ));

        let incremental_resolution = accepted.resolution_index();
        let clean_resolution = current.resolution_index();
        assert_eq!(
            incremental_resolution
                .resolve_entity_id(&id("child.a"))
                .map(|node| node.id().clone()),
            clean_resolution
                .resolve_entity_id(&id("child.a"))
                .map(|node| node.id().clone())
        );
        assert_eq!(
            incremental_resolution.resolve_child(&id("owner"), &name("After")),
            clean_resolution.resolve_child(&id("owner"), &name("After"))
        );
        assert_eq!(
            incremental_resolution.resolve_child(&id("owner"), &name("Before")),
            clean_resolution.resolve_child(&id("owner"), &name("Before"))
        );
    }

    #[test]
    fn lifecycle_rejects_wrong_base_target_and_replay_without_losing_retry() {
        let mut previous = SemanticGraph::new();
        previous.insert_node(node("node", "Before", NodeKind::Procedure));
        let mut current = SemanticGraph::new();
        current.insert_node(node("node", "After", NodeKind::Function));
        let mut wrong_target = SemanticGraph::new();
        wrong_target.insert_node(node("node", "After", NodeKind::Function));
        let mut unrelated_previous = SemanticGraph::new();
        unrelated_previous.insert_node(node("node", "Before", NodeKind::Procedure));

        let changes = NormalizedSemanticIndexChanges::between(&previous, &current)
            .expect("lifecycle transition must normalize");
        let accepted_previous = AcceptedSemanticIndex::rebuild(&previous);
        let previous_state = accepted_previous.state().clone();

        let wrong_target_error = accepted_previous
            .transition(&wrong_target, &changes)
            .expect_err("wrong target instance must fail");
        assert_eq!(
            wrong_target_error,
            SemanticIndexLifecycleError::WrongTargetSnapshot
        );
        assert_eq!(accepted_previous.state(), &previous_state);

        let unrelated_error = AcceptedSemanticIndex::rebuild(&unrelated_previous)
            .transition(&current, &changes)
            .expect_err("unrelated base instance must fail");
        assert_eq!(
            unrelated_error,
            SemanticIndexLifecycleError::StaleBaseSnapshot
        );

        let first = accepted_previous
            .transition(&current, &changes)
            .expect("retry after failure must succeed");
        let retry = accepted_previous
            .transition(&current, &changes)
            .expect("repeated retry from previous must succeed");
        assert_eq!(first.state(), retry.state());
        assert!(ptr::eq(first.graph(), changes.current()));

        let replay_error = first
            .transition(&current, &changes)
            .expect_err("already-current state must reject replay");
        assert_eq!(replay_error, SemanticIndexLifecycleError::StaleBaseSnapshot);

        let empty_changes = NormalizedSemanticIndexChanges::between(&current, &current)
            .expect("current-to-current transition must normalize");
        let idempotent = first
            .transition(&current, &empty_changes)
            .expect("current-to-current empty transition must succeed");
        assert_eq!(idempotent.state(), first.state());

        let rebuilt = AcceptedSemanticIndex::rebuild(&current);
        assert_eq!(rebuilt.state(), first.state());
    }
}
