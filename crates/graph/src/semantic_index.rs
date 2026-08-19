//! Deterministic derived indexes for one complete semantic graph snapshot.

use std::collections::{BTreeMap, BTreeSet};

use oneagent_common::{EntityId, EntityName};

use crate::{
    EdgeId, EdgeKind, GraphEdge, GraphNode, NodeId, NodeKind, NodeSnapshot, SemanticGraph,
    edge_identity::edge_id,
    incremental_index::{NormalizedSemanticIndexChanges, NormalizedSemanticIndexOperation},
};

/// Owned membership for the node dimensions of the semantic index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticNodeIndexState {
    identities: BTreeSet<EntityId>,
    by_name: BTreeMap<EntityName, BTreeSet<EntityId>>,
    by_kind: BTreeMap<NodeKind, BTreeSet<EntityId>>,
}

impl SemanticNodeIndexState {
    pub(crate) fn from_graph(graph: &SemanticGraph) -> Self {
        let mut state = Self {
            identities: BTreeSet::new(),
            by_name: BTreeMap::new(),
            by_kind: BTreeMap::new(),
        };

        for node in graph.nodes() {
            let inserted = state.identities.insert(node.id().clone());
            debug_assert!(inserted, "canonical graph node identity must be unique");
            state
                .by_name
                .entry(node.name().clone())
                .or_default()
                .insert(node.id().clone());
            state
                .by_kind
                .entry(node.kind())
                .or_default()
                .insert(node.id().clone());
        }

        state
    }

    /// Applies only node projections and returns a complete private candidate.
    #[allow(dead_code)]
    pub(crate) fn apply_changes(
        &self,
        changes: &NormalizedSemanticIndexChanges<'_, '_>,
    ) -> Result<Self, SemanticNodeIndexError> {
        if self != &Self::from_graph(changes.previous()) {
            return Err(SemanticNodeIndexError::StaleBaseState);
        }

        let mut next = self.clone();
        for operation in changes.operations() {
            match operation {
                NormalizedSemanticIndexOperation::RemoveNode(node) => next.remove(node)?,
                NormalizedSemanticIndexOperation::ReplaceNode { old, new } => {
                    next.remove(old)?;
                    next.insert(new)?;
                }
                NormalizedSemanticIndexOperation::AddNode(node) => next.insert(node)?,
                NormalizedSemanticIndexOperation::RefreshNode { old, new } => {
                    next.validate_refresh(old, new)?;
                }
                NormalizedSemanticIndexOperation::RemoveEdge(_)
                | NormalizedSemanticIndexOperation::AddEdge(_)
                | NormalizedSemanticIndexOperation::RefreshEdge { .. } => {}
            }
        }

        if next != Self::from_graph(changes.current()) {
            return Err(SemanticNodeIndexError::CurrentStateMismatch);
        }

        Ok(next)
    }

    fn insert(&mut self, node: &NodeSnapshot) -> Result<(), SemanticNodeIndexError> {
        let id = snapshot_entity_id(node)?;
        if !self.identities.insert(id.clone()) {
            return Err(SemanticNodeIndexError::DuplicateNode(id));
        }
        self.by_name
            .entry(node.name().clone())
            .or_default()
            .insert(id.clone());
        self.by_kind.entry(node.kind()).or_default().insert(id);
        Ok(())
    }

    fn remove(&mut self, node: &NodeSnapshot) -> Result<(), SemanticNodeIndexError> {
        let id = snapshot_entity_id(node)?;
        if !self.identities.remove(&id) {
            return Err(SemanticNodeIndexError::MissingNode(id));
        }

        remove_bucket_member(&mut self.by_name, node.name(), &id)?;
        remove_bucket_member(&mut self.by_kind, &node.kind(), &id)?;
        Ok(())
    }

    fn validate_refresh(
        &self,
        old: &NodeSnapshot,
        new: &NodeSnapshot,
    ) -> Result<(), SemanticNodeIndexError> {
        let old_id = snapshot_entity_id(old)?;
        let new_id = snapshot_entity_id(new)?;
        if old_id != new_id || old.name() != new.name() || old.kind() != new.kind() {
            return Err(SemanticNodeIndexError::InvalidRefresh(old_id));
        }
        if !self.identities.contains(&old_id)
            || !self
                .by_name
                .get(old.name())
                .is_some_and(|ids| ids.contains(&old_id))
            || !self
                .by_kind
                .get(&old.kind())
                .is_some_and(|ids| ids.contains(&old_id))
        {
            return Err(SemanticNodeIndexError::MissingNode(old_id));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn ids(&self) -> &BTreeSet<EntityId> {
        &self.identities
    }

    #[cfg(test)]
    pub(crate) fn ids_by_name(&self, name: &EntityName) -> Option<&BTreeSet<EntityId>> {
        self.by_name.get(name)
    }

    #[cfg(test)]
    pub(crate) fn ids_by_kind(&self, kind: NodeKind) -> Option<&BTreeSet<EntityId>> {
        self.by_kind.get(&kind)
    }
}

/// Typed atomic node-state transition failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticNodeIndexError {
    StaleBaseState,
    InvalidNodeId(NodeId),
    DuplicateNode(EntityId),
    MissingNode(EntityId),
    MissingBucketMember(EntityId),
    InvalidRefresh(EntityId),
    CurrentStateMismatch,
}

fn snapshot_entity_id(node: &NodeSnapshot) -> Result<EntityId, SemanticNodeIndexError> {
    EntityId::new(node.id().as_str())
        .map_err(|_| SemanticNodeIndexError::InvalidNodeId(node.id().clone()))
}

fn remove_bucket_member<Key: Ord>(
    buckets: &mut BTreeMap<Key, BTreeSet<EntityId>>,
    key: &Key,
    id: &EntityId,
) -> Result<(), SemanticNodeIndexError> {
    let remove_bucket = {
        let Some(ids) = buckets.get_mut(key) else {
            return Err(SemanticNodeIndexError::MissingBucketMember(id.clone()));
        };
        if !ids.remove(id) {
            return Err(SemanticNodeIndexError::MissingBucketMember(id.clone()));
        }
        ids.is_empty()
    };
    if remove_bucket {
        buckets.remove(key);
    }
    Ok(())
}

/// Crate-internal read-only lookup state derived from one borrowed graph snapshot.
#[derive(Debug)]
pub(crate) struct SemanticIndex<'graph> {
    nodes_by_id: BTreeMap<EntityId, &'graph GraphNode>,
    nodes_by_name: BTreeMap<EntityName, Vec<&'graph GraphNode>>,
    nodes_by_kind: BTreeMap<NodeKind, Vec<&'graph GraphNode>>,
    edges_by_id: BTreeMap<EdgeId, &'graph GraphEdge>,
    edges_by_kind: BTreeMap<EdgeKind, Vec<&'graph GraphEdge>>,
    outgoing_edges: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    outgoing_edges_by_kind: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    incoming_edges: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    incoming_edges_by_kind: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    owner_edges_by_child: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    owners_by_child: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    children_by_owner: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    children_by_owner_kind: BTreeMap<(EntityId, NodeKind), Vec<&'graph GraphNode>>,
    children_by_owner_name: BTreeMap<(EntityId, EntityName), Vec<&'graph GraphNode>>,
}

impl<'graph> SemanticIndex<'graph> {
    /// Builds lookup state without changing or copying canonical graph facts.
    pub(crate) fn new(graph: &'graph SemanticGraph) -> Self {
        let node_state = SemanticNodeIndexState::from_graph(graph);
        let NodeViews {
            identities: nodes_by_id,
            names: nodes_by_name,
            kinds: nodes_by_kind,
        } = build_node_views(graph, &node_state);

        let mut ordered_edges = graph
            .edges()
            .map(|edge| {
                (
                    edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()),
                    edge,
                )
            })
            .collect::<Vec<_>>();
        ordered_edges.sort_by(|left, right| left.0.cmp(&right.0));

        let mut edges_by_id = BTreeMap::new();
        let mut edges_by_kind = BTreeMap::<EdgeKind, Vec<&GraphEdge>>::new();
        for (id, edge) in ordered_edges {
            edges_by_kind.entry(edge.kind()).or_default().push(edge);
            edges_by_id.insert(id, edge);
        }

        let relations = build_relation_indexes(&nodes_by_id, &edges_by_id);

        Self {
            nodes_by_id,
            nodes_by_name,
            nodes_by_kind,
            edges_by_id,
            edges_by_kind,
            outgoing_edges: relations.outgoing_edges,
            outgoing_edges_by_kind: relations.outgoing_edges_by_kind,
            incoming_edges: relations.incoming_edges,
            incoming_edges_by_kind: relations.incoming_edges_by_kind,
            owner_edges_by_child: relations.owner_edges_by_child,
            owners_by_child: relations.owners_by_child,
            children_by_owner: relations.children_by_owner,
            children_by_owner_kind: relations.children_by_owner_kind,
            children_by_owner_name: relations.children_by_owner_name,
        }
    }

    pub(crate) fn node(&self, id: &EntityId) -> Option<&'graph GraphNode> {
        self.nodes_by_id.get(id).copied()
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &'graph GraphNode> + '_ {
        self.nodes_by_id.values().copied()
    }

    pub(crate) fn nodes_by_name(&self, name: &EntityName) -> &[&'graph GraphNode] {
        self.nodes_by_name.get(name).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn nodes_by_kind(&self, kind: NodeKind) -> &[&'graph GraphNode] {
        self.nodes_by_kind.get(&kind).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn edge(&self, id: &EdgeId) -> Option<&'graph GraphEdge> {
        self.edges_by_id.get(id).copied()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = &'graph GraphEdge> + '_ {
        self.edges_by_id.values().copied()
    }

    pub(crate) fn edges_by_kind(&self, kind: EdgeKind) -> &[&'graph GraphEdge] {
        self.edges_by_kind.get(&kind).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn outgoing_edges(&self, node: &EntityId) -> &[&'graph GraphEdge] {
        self.outgoing_edges.get(node).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn outgoing_edges_by_kind(
        &self,
        node: &EntityId,
        kind: EdgeKind,
    ) -> &[&'graph GraphEdge] {
        self.outgoing_edges_by_kind
            .get(&(node.clone(), kind))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn incoming_edges(&self, node: &EntityId) -> &[&'graph GraphEdge] {
        self.incoming_edges.get(node).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn incoming_edges_by_kind(
        &self,
        node: &EntityId,
        kind: EdgeKind,
    ) -> &[&'graph GraphEdge] {
        self.incoming_edges_by_kind
            .get(&(node.clone(), kind))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn owner_edges(&self, child: &EntityId) -> &[&'graph GraphEdge] {
        self.owner_edges_by_child
            .get(child)
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn owners(&self, child: &EntityId) -> &[&'graph GraphNode] {
        self.owners_by_child.get(child).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn children(&self, owner: &EntityId) -> &[&'graph GraphNode] {
        self.children_by_owner.get(owner).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn children_by_kind(
        &self,
        owner: &EntityId,
        kind: NodeKind,
    ) -> &[&'graph GraphNode] {
        self.children_by_owner_kind
            .get(&(owner.clone(), kind))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn children_by_name(
        &self,
        owner: &EntityId,
        name: &EntityName,
    ) -> &[&'graph GraphNode] {
        self.children_by_owner_name
            .get(&(owner.clone(), name.clone()))
            .map_or(&[], Vec::as_slice)
    }
}

struct NodeViews<'graph> {
    identities: BTreeMap<EntityId, &'graph GraphNode>,
    names: BTreeMap<EntityName, Vec<&'graph GraphNode>>,
    kinds: BTreeMap<NodeKind, Vec<&'graph GraphNode>>,
}

fn build_node_views<'graph>(
    graph: &'graph SemanticGraph,
    state: &SemanticNodeIndexState,
) -> NodeViews<'graph> {
    let identities = state
        .identities
        .iter()
        .map(|id| {
            let node = graph
                .node(id)
                .expect("semantic node index state must resolve in its canonical graph");
            (id.clone(), node)
        })
        .collect();
    let names = state
        .by_name
        .iter()
        .map(|(name, ids)| {
            let nodes = ids
                .iter()
                .map(|id| {
                    graph
                        .node(id)
                        .expect("semantic node name state must resolve in its canonical graph")
                })
                .collect();
            (name.clone(), nodes)
        })
        .collect();
    let kinds = state
        .by_kind
        .iter()
        .map(|(kind, ids)| {
            let nodes = ids
                .iter()
                .map(|id| {
                    graph
                        .node(id)
                        .expect("semantic node kind state must resolve in its canonical graph")
                })
                .collect();
            (*kind, nodes)
        })
        .collect();

    NodeViews {
        identities,
        names,
        kinds,
    }
}

struct RelationIndexes<'graph> {
    outgoing_edges: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    outgoing_edges_by_kind: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    incoming_edges: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    incoming_edges_by_kind: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    owner_edges_by_child: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    owners_by_child: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    children_by_owner: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    children_by_owner_kind: BTreeMap<(EntityId, NodeKind), Vec<&'graph GraphNode>>,
    children_by_owner_name: BTreeMap<(EntityId, EntityName), Vec<&'graph GraphNode>>,
}

fn build_relation_indexes<'graph>(
    nodes_by_id: &BTreeMap<EntityId, &'graph GraphNode>,
    edges_by_id: &BTreeMap<EdgeId, &'graph GraphEdge>,
) -> RelationIndexes<'graph> {
    let mut indexes = RelationIndexes {
        outgoing_edges: BTreeMap::new(),
        outgoing_edges_by_kind: BTreeMap::new(),
        incoming_edges: BTreeMap::new(),
        incoming_edges_by_kind: BTreeMap::new(),
        owner_edges_by_child: BTreeMap::new(),
        owners_by_child: BTreeMap::new(),
        children_by_owner: BTreeMap::new(),
        children_by_owner_kind: BTreeMap::new(),
        children_by_owner_name: BTreeMap::new(),
    };

    for edge in edges_by_id.values().copied() {
        indexes
            .outgoing_edges
            .entry(edge.source().clone())
            .or_default()
            .push(edge);
        indexes
            .outgoing_edges_by_kind
            .entry((edge.source().clone(), edge.kind()))
            .or_default()
            .push(edge);
        indexes
            .incoming_edges
            .entry(edge.target().clone())
            .or_default()
            .push(edge);
        indexes
            .incoming_edges_by_kind
            .entry((edge.target().clone(), edge.kind()))
            .or_default()
            .push(edge);

        if edge.kind() != EdgeKind::Contains {
            continue;
        }

        indexes
            .owner_edges_by_child
            .entry(edge.target().clone())
            .or_default()
            .push(edge);

        let (Some(owner), Some(child)) = (
            nodes_by_id.get(edge.source()).copied(),
            nodes_by_id.get(edge.target()).copied(),
        ) else {
            continue;
        };

        indexes
            .owners_by_child
            .entry(edge.target().clone())
            .or_default()
            .push(owner);
        indexes
            .children_by_owner
            .entry(edge.source().clone())
            .or_default()
            .push(child);
        indexes
            .children_by_owner_kind
            .entry((edge.source().clone(), child.kind()))
            .or_default()
            .push(child);
        indexes
            .children_by_owner_name
            .entry((edge.source().clone(), child.name().clone()))
            .or_default()
            .push(child);
    }

    for nodes in indexes
        .owners_by_child
        .values_mut()
        .chain(indexes.children_by_owner.values_mut())
        .chain(indexes.children_by_owner_kind.values_mut())
        .chain(indexes.children_by_owner_name.values_mut())
    {
        sort_and_deduplicate_nodes(nodes);
    }

    indexes
}

fn sort_and_deduplicate_nodes(nodes: &mut Vec<&GraphNode>) {
    nodes.sort_by_key(|node| node.id());
    nodes.dedup_by_key(|node| node.id());
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;

    use super::SemanticIndex;
    use crate::{EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph, SemanticGraphQuery};

    const NODE_KINDS: [NodeKind; 17] = [
        NodeKind::Metadata(MetadataKind::Catalog),
        NodeKind::Module,
        NodeKind::Procedure,
        NodeKind::Function,
        NodeKind::Query,
        NodeKind::Form,
        NodeKind::Command,
        NodeKind::Attribute,
        NodeKind::StandardAttribute,
        NodeKind::TabularSection,
        NodeKind::Dimension,
        NodeKind::Resource,
        NodeKind::Measure,
        NodeKind::Role,
        NodeKind::AccessRight,
        NodeKind::Subsystem,
        NodeKind::Unknown,
    ];
    const EDGE_KINDS: [EdgeKind; 9] = [
        EdgeKind::Contains,
        EdgeKind::Calls,
        EdgeKind::References,
        EdgeKind::Reads,
        EdgeKind::Writes,
        EdgeKind::Grants,
        EdgeKind::Includes,
        EdgeKind::Extends,
        EdgeKind::DependsOn,
    ];

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn fixture(reverse: bool) -> SemanticGraph {
        let mut nodes = NODE_KINDS
            .iter()
            .enumerate()
            .map(|(index, kind)| {
                let node_name = if index < 2 { "Shared" } else { "Unique" };
                GraphNode::new(id(&format!("node.{index:02}")), name(node_name), *kind)
            })
            .collect::<Vec<_>>();
        let mut edges = EDGE_KINDS
            .iter()
            .map(|kind| GraphEdge::new(id("node.00"), id("node.01"), *kind))
            .collect::<Vec<_>>();

        if reverse {
            nodes.reverse();
            edges.reverse();
        }

        let mut graph = SemanticGraph::new();
        for node in nodes {
            graph.insert_node(node);
        }
        for edge in edges {
            graph.insert_edge(edge).expect("edge endpoints must exist");
        }
        graph
    }

    #[test]
    fn empty_snapshot_has_no_placeholder_entries() {
        let graph = SemanticGraph::new();
        let index = SemanticIndex::new(&graph);

        assert!(index.nodes().next().is_none());
        assert!(index.edges().next().is_none());
        assert!(index.node(&id("missing")).is_none());
        assert!(index.nodes_by_name(&name("Missing")).is_empty());
        assert!(index.nodes_by_kind(NodeKind::Module).is_empty());
        assert!(index.edge(&crate::EdgeId::new("edge:missing")).is_none());
        assert!(index.edges_by_kind(EdgeKind::Calls).is_empty());
    }

    #[test]
    fn indexes_every_represented_kind_and_borrows_canonical_objects() {
        let graph = fixture(false);
        let index = SemanticIndex::new(&graph);

        for kind in NODE_KINDS {
            assert_eq!(index.nodes_by_kind(kind).len(), 1);
        }
        for kind in EDGE_KINDS {
            let edges = index.edges_by_kind(kind);
            assert_eq!(edges.len(), 1);
            let expected_id = SemanticGraphQuery::edge_id(
                &crate::NodeId::new(edges[0].source().as_str()),
                &crate::NodeId::new(edges[0].target().as_str()),
                kind,
            );
            assert!(ptr::eq(
                index.edge(&expected_id).expect("edge must be indexed"),
                edges[0]
            ));
        }

        let canonical = graph.node(&id("node.00")).expect("node must exist");
        assert!(ptr::eq(
            index.node(&id("node.00")).expect("node must be indexed"),
            canonical
        ));
        assert_eq!(index.nodes_by_name(&name("Shared")).len(), 2);
        assert_eq!(
            index
                .nodes_by_name(&name("Shared"))
                .iter()
                .map(|node| node.id().as_str())
                .collect::<Vec<_>>(),
            vec!["node.00", "node.01"]
        );
    }

    #[test]
    fn construction_and_order_do_not_depend_on_insertion_order() {
        let normal = fixture(false);
        let reversed = fixture(true);
        let normal_index = SemanticIndex::new(&normal);
        let repeated_index = SemanticIndex::new(&normal);
        let reversed_index = SemanticIndex::new(&reversed);

        let node_ids = |index: &SemanticIndex<'_>| {
            index
                .nodes()
                .map(|node| node.id().as_str().to_owned())
                .collect::<Vec<_>>()
        };
        let edge_ids = |index: &SemanticIndex<'_>| {
            index
                .edges()
                .map(|edge| {
                    SemanticGraphQuery::edge_id(
                        &crate::NodeId::new(edge.source().as_str()),
                        &crate::NodeId::new(edge.target().as_str()),
                        edge.kind(),
                    )
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(node_ids(&normal_index), node_ids(&repeated_index));
        assert_eq!(node_ids(&normal_index), node_ids(&reversed_index));
        assert_eq!(edge_ids(&normal_index), edge_ids(&repeated_index));
        assert_eq!(edge_ids(&normal_index), edge_ids(&reversed_index));
        assert!(
            edge_ids(&normal_index)
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }

    #[test]
    fn adjacency_indexes_match_independent_canonical_scans() {
        let graph = fixture(false);
        let reversed = fixture(true);
        let index = SemanticIndex::new(&graph);
        let reversed_index = SemanticIndex::new(&reversed);
        let source = id("node.00");
        let target = id("node.01");
        let missing = id("node.missing");

        let edge_ids = |edges: &[&GraphEdge]| {
            edges
                .iter()
                .map(|edge| {
                    SemanticGraphQuery::edge_id(
                        &crate::NodeId::new(edge.source().as_str()),
                        &crate::NodeId::new(edge.target().as_str()),
                        edge.kind(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let mut canonical = graph
            .edges()
            .filter(|edge| edge.source() == &source)
            .collect::<Vec<_>>();
        canonical.sort_by_key(|edge| {
            SemanticGraphQuery::edge_id(
                &crate::NodeId::new(edge.source().as_str()),
                &crate::NodeId::new(edge.target().as_str()),
                edge.kind(),
            )
        });

        assert_eq!(
            edge_ids(index.outgoing_edges(&source)),
            edge_ids(&canonical)
        );
        assert_eq!(
            edge_ids(index.outgoing_edges(&source)),
            edge_ids(reversed_index.outgoing_edges(&source))
        );
        assert_eq!(index.outgoing_edges(&source).len(), EDGE_KINDS.len());
        assert_eq!(index.incoming_edges(&target).len(), EDGE_KINDS.len());
        assert_eq!(
            index.outgoing_edges_by_kind(&source, EdgeKind::Calls),
            index.incoming_edges_by_kind(&target, EdgeKind::Calls)
        );
        assert_eq!(
            index.outgoing_edges_by_kind(&source, EdgeKind::Calls).len(),
            1
        );
        assert!(index.outgoing_edges(&missing).is_empty());
        assert!(index.incoming_edges(&missing).is_empty());
        assert!(
            edge_ids(index.outgoing_edges(&source))
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }

    #[test]
    fn containment_indexes_preserve_duplicate_and_invalid_states() {
        let owner_a = GraphNode::new(id("owner.a"), name("Owner A"), NodeKind::Module);
        let owner_b = GraphNode::new(id("owner.b"), name("Owner B"), NodeKind::Module);
        let child_a = GraphNode::new(id("child.a"), name("Shared"), NodeKind::Attribute);
        let child_b = GraphNode::new(id("child.b"), name("Shared"), NodeKind::Attribute);
        let mut graph = SemanticGraph::new();
        for node in [owner_a, owner_b, child_a, child_b] {
            graph.insert_node(node);
        }
        for edge in [
            GraphEdge::new(id("owner.a"), id("child.a"), EdgeKind::Contains),
            GraphEdge::new(id("owner.b"), id("child.a"), EdgeKind::Contains),
            GraphEdge::new(id("owner.a"), id("child.b"), EdgeKind::Contains),
            GraphEdge::new(id("owner.a"), id("owner.a"), EdgeKind::Contains),
        ] {
            graph.insert_edge(edge).expect("edge endpoints must exist");
        }

        let index = SemanticIndex::new(&graph);
        let node_ids = |nodes: &[&GraphNode]| {
            nodes
                .iter()
                .map(|node| node.id().as_str().to_owned())
                .collect::<Vec<_>>()
        };

        assert_eq!(
            node_ids(index.owners(&id("child.a"))),
            vec!["owner.a".to_owned(), "owner.b".to_owned()]
        );
        assert_eq!(index.owner_edges(&id("child.a")).len(), 2);
        assert_eq!(
            node_ids(index.children(&id("owner.a"))),
            vec![
                "child.a".to_owned(),
                "child.b".to_owned(),
                "owner.a".to_owned()
            ]
        );
        assert_eq!(
            node_ids(index.children_by_kind(&id("owner.a"), NodeKind::Attribute)),
            vec!["child.a".to_owned(), "child.b".to_owned()]
        );
        assert_eq!(
            node_ids(index.children_by_name(&id("owner.a"), &name("Shared"))),
            vec!["child.a".to_owned(), "child.b".to_owned()]
        );
        assert_eq!(
            node_ids(index.owners(&id("owner.a"))),
            vec!["owner.a".to_owned()]
        );
        assert!(index.children(&id("missing")).is_empty());
        assert!(
            index
                .children_by_name(&id("owner.b"), &name("Missing"))
                .is_empty()
        );
    }
}
