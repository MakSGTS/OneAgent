//! Deterministic derived indexes for one complete semantic graph snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::ptr;

use oneagent_common::{EntityId, EntityName};

use crate::{
    EdgeId, EdgeKind, EdgeSnapshot, GraphEdge, GraphNode, NodeId, NodeKind, NodeSnapshot,
    SemanticGraph, SemanticGraphQuery, SemanticResolutionIndex,
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
    pub(crate) fn apply_changes(
        &self,
        changes: &NormalizedSemanticIndexChanges<'_, '_>,
    ) -> Result<Self, SemanticNodeIndexError> {
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

        Ok(next)
    }

    fn is_internally_consistent(&self) -> bool {
        is_exact_node_partition(&self.identities, self.by_name.values())
            && is_exact_node_partition(&self.identities, self.by_kind.values())
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
    InvalidNodeId(NodeId),
    DuplicateNode(EntityId),
    MissingNode(EntityId),
    MissingBucketMember(EntityId),
    InvalidRefresh(EntityId),
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

/// Owned membership for every edge, adjacency, and containment dimension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticEdgeIndexState {
    identities: BTreeSet<EdgeId>,
    kinds: BTreeMap<EdgeKind, BTreeSet<EdgeId>>,
    outgoing: BTreeMap<EntityId, BTreeSet<EdgeId>>,
    outgoing_kinds: BTreeMap<(EntityId, EdgeKind), BTreeSet<EdgeId>>,
    incoming: BTreeMap<EntityId, BTreeSet<EdgeId>>,
    incoming_kinds: BTreeMap<(EntityId, EdgeKind), BTreeSet<EdgeId>>,
    owner_edges: BTreeMap<EntityId, BTreeSet<EdgeId>>,
    owners: BTreeMap<EntityId, BTreeSet<EntityId>>,
    children: BTreeMap<EntityId, BTreeSet<EntityId>>,
    child_kinds: BTreeMap<(EntityId, NodeKind), BTreeSet<EntityId>>,
    child_names: BTreeMap<(EntityId, EntityName), BTreeSet<EntityId>>,
}

impl SemanticEdgeIndexState {
    pub(crate) fn from_graph(graph: &SemanticGraph) -> Self {
        let mut state = Self::empty();
        for edge in graph.edges() {
            state.insert_graph_edge(graph, edge);
        }
        state
    }

    const fn empty() -> Self {
        Self {
            identities: BTreeSet::new(),
            kinds: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            outgoing_kinds: BTreeMap::new(),
            incoming: BTreeMap::new(),
            incoming_kinds: BTreeMap::new(),
            owner_edges: BTreeMap::new(),
            owners: BTreeMap::new(),
            children: BTreeMap::new(),
            child_kinds: BTreeMap::new(),
            child_names: BTreeMap::new(),
        }
    }

    fn insert_graph_edge(&mut self, graph: &SemanticGraph, edge: &GraphEdge) {
        let id = edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind());
        let inserted = self.identities.insert(id.clone());
        debug_assert!(inserted, "canonical graph edge identity must be unique");
        insert_edge_membership(&mut self.kinds, edge.kind(), &id);
        insert_edge_membership(&mut self.outgoing, edge.source().clone(), &id);
        insert_edge_membership(
            &mut self.outgoing_kinds,
            (edge.source().clone(), edge.kind()),
            &id,
        );
        insert_edge_membership(&mut self.incoming, edge.target().clone(), &id);
        insert_edge_membership(
            &mut self.incoming_kinds,
            (edge.target().clone(), edge.kind()),
            &id,
        );
        if edge.kind() == EdgeKind::Contains {
            self.insert_containment(graph, edge.source(), edge.target(), &id);
        }
    }

    fn insert_snapshot(
        &mut self,
        graph: &SemanticGraph,
        edge: &EdgeSnapshot,
    ) -> Result<(), SemanticEdgeIndexError> {
        let source = snapshot_node_entity_id(edge.source())?;
        let target = snapshot_node_entity_id(edge.target())?;
        if !self.identities.insert(edge.id().clone()) {
            return Err(SemanticEdgeIndexError::DuplicateEdge(edge.id().clone()));
        }
        insert_edge_membership(&mut self.kinds, edge.kind(), edge.id());
        insert_edge_membership(&mut self.outgoing, source.clone(), edge.id());
        insert_edge_membership(
            &mut self.outgoing_kinds,
            (source.clone(), edge.kind()),
            edge.id(),
        );
        insert_edge_membership(&mut self.incoming, target.clone(), edge.id());
        insert_edge_membership(
            &mut self.incoming_kinds,
            (target.clone(), edge.kind()),
            edge.id(),
        );
        if edge.kind() == EdgeKind::Contains {
            self.insert_containment(graph, &source, &target, edge.id());
        }
        Ok(())
    }

    fn remove_snapshot(
        &mut self,
        graph: &SemanticGraph,
        edge: &EdgeSnapshot,
    ) -> Result<(), SemanticEdgeIndexError> {
        let source = snapshot_node_entity_id(edge.source())?;
        let target = snapshot_node_entity_id(edge.target())?;
        if !self.identities.remove(edge.id()) {
            return Err(SemanticEdgeIndexError::MissingEdge(edge.id().clone()));
        }
        remove_edge_membership(&mut self.kinds, &edge.kind(), edge.id())?;
        remove_edge_membership(&mut self.outgoing, &source, edge.id())?;
        remove_edge_membership(
            &mut self.outgoing_kinds,
            &(source.clone(), edge.kind()),
            edge.id(),
        )?;
        remove_edge_membership(&mut self.incoming, &target, edge.id())?;
        remove_edge_membership(
            &mut self.incoming_kinds,
            &(target.clone(), edge.kind()),
            edge.id(),
        )?;
        if edge.kind() == EdgeKind::Contains {
            self.remove_containment(graph, &source, &target, edge.id())?;
        }
        Ok(())
    }

    fn insert_containment(
        &mut self,
        graph: &SemanticGraph,
        owner: &EntityId,
        child: &EntityId,
        edge: &EdgeId,
    ) {
        insert_edge_membership(&mut self.owner_edges, child.clone(), edge);
        let (Some(owner_node), Some(child_node)) = (graph.node(owner), graph.node(child)) else {
            return;
        };
        insert_node_membership(&mut self.owners, child.clone(), owner_node.id());
        insert_node_membership(&mut self.children, owner.clone(), child_node.id());
        insert_node_membership(
            &mut self.child_kinds,
            (owner.clone(), child_node.kind()),
            child_node.id(),
        );
        insert_node_membership(
            &mut self.child_names,
            (owner.clone(), child_node.name().clone()),
            child_node.id(),
        );
    }

    fn remove_containment(
        &mut self,
        graph: &SemanticGraph,
        owner: &EntityId,
        child: &EntityId,
        edge: &EdgeId,
    ) -> Result<(), SemanticEdgeIndexError> {
        remove_edge_membership(&mut self.owner_edges, child, edge)?;
        let (Some(owner_node), Some(child_node)) = (graph.node(owner), graph.node(child)) else {
            return Ok(());
        };
        remove_node_membership(&mut self.owners, child, owner_node.id())?;
        remove_node_membership(&mut self.children, owner, child_node.id())?;
        remove_node_membership(
            &mut self.child_kinds,
            &(owner.clone(), child_node.kind()),
            child_node.id(),
        )?;
        remove_node_membership(
            &mut self.child_names,
            &(owner.clone(), child_node.name().clone()),
            child_node.id(),
        )?;
        Ok(())
    }

    fn rekey_containment_child(
        &mut self,
        current: &SemanticGraph,
        old: &NodeSnapshot,
        new: &NodeSnapshot,
    ) -> Result<(), SemanticEdgeIndexError> {
        if old.name() == new.name() && old.kind() == new.kind() {
            return Ok(());
        }
        let child = snapshot_node_entity_id(old.id())?;
        let retained = self
            .incoming_kinds
            .get(&(child.clone(), EdgeKind::Contains))
            .cloned()
            .unwrap_or_default();
        for id in retained {
            let edge = graph_edge_by_id(current, &id)
                .ok_or_else(|| SemanticEdgeIndexError::MissingCurrentEdge(id.clone()))?;
            let owner = edge.source().clone();
            if old.kind() != new.kind() {
                remove_node_membership(
                    &mut self.child_kinds,
                    &(owner.clone(), old.kind()),
                    &child,
                )?;
                insert_node_membership(&mut self.child_kinds, (owner.clone(), new.kind()), &child);
            }
            if old.name() != new.name() {
                remove_node_membership(
                    &mut self.child_names,
                    &(owner.clone(), old.name().clone()),
                    &child,
                )?;
                insert_node_membership(&mut self.child_names, (owner, new.name().clone()), &child);
            }
        }
        Ok(())
    }

    fn validate_refresh(
        &self,
        old: &EdgeSnapshot,
        new: &EdgeSnapshot,
    ) -> Result<(), SemanticEdgeIndexError> {
        if old.id() != new.id()
            || old.source() != new.source()
            || old.target() != new.target()
            || old.kind() != new.kind()
        {
            return Err(SemanticEdgeIndexError::InvalidRefresh(old.id().clone()));
        }
        if !self.identities.contains(old.id()) {
            return Err(SemanticEdgeIndexError::MissingEdge(old.id().clone()));
        }
        Ok(())
    }

    fn validate_removed_node(&self, node: &NodeSnapshot) -> Result<(), SemanticEdgeIndexError> {
        let id = snapshot_node_entity_id(node.id())?;
        if self.outgoing.contains_key(&id) || self.incoming.contains_key(&id) {
            return Err(SemanticEdgeIndexError::RetainedIncidentEdge(id));
        }
        Ok(())
    }

    pub(crate) fn apply_changes(
        &self,
        changes: &NormalizedSemanticIndexChanges<'_, '_>,
    ) -> Result<Self, SemanticEdgeIndexError> {
        let mut next = self.clone();
        for operation in changes.operations() {
            match operation {
                NormalizedSemanticIndexOperation::RemoveEdge(edge) => {
                    next.remove_snapshot(changes.previous(), edge)?;
                }
                NormalizedSemanticIndexOperation::RemoveNode(node) => {
                    next.validate_removed_node(node)?;
                }
                NormalizedSemanticIndexOperation::ReplaceNode { old, new } => {
                    next.rekey_containment_child(changes.current(), old, new)?;
                }
                NormalizedSemanticIndexOperation::AddEdge(edge) => {
                    next.insert_snapshot(changes.current(), edge)?;
                }
                NormalizedSemanticIndexOperation::RefreshEdge { old, new } => {
                    next.validate_refresh(old, new)?;
                }
                NormalizedSemanticIndexOperation::AddNode(_)
                | NormalizedSemanticIndexOperation::RefreshNode { .. } => {}
            }
        }
        Ok(next)
    }

    fn is_internally_consistent(&self, nodes: &BTreeSet<EntityId>) -> bool {
        let contains = self
            .kinds
            .get(&EdgeKind::Contains)
            .cloned()
            .unwrap_or_default();
        is_exact_edge_partition(&self.identities, self.kinds.values())
            && is_exact_edge_partition(&self.identities, self.outgoing.values())
            && is_exact_edge_partition(&self.identities, self.outgoing_kinds.values())
            && is_exact_edge_partition(&self.identities, self.incoming.values())
            && is_exact_edge_partition(&self.identities, self.incoming_kinds.values())
            && is_exact_edge_partition(&contains, self.owner_edges.values())
            && self
                .owners
                .iter()
                .all(|(child, owners)| nodes.contains(child) && owners.is_subset(nodes))
            && self
                .children
                .iter()
                .all(|(owner, children)| nodes.contains(owner) && children.is_subset(nodes))
            && self
                .child_kinds
                .iter()
                .all(|((owner, _), children)| nodes.contains(owner) && children.is_subset(nodes))
            && self
                .child_names
                .iter()
                .all(|((owner, _), children)| nodes.contains(owner) && children.is_subset(nodes))
    }

    #[cfg(test)]
    pub(crate) fn edge_ids(&self) -> &BTreeSet<EdgeId> {
        &self.identities
    }

    #[cfg(test)]
    pub(crate) fn edge_ids_by_kind(&self, kind: EdgeKind) -> Option<&BTreeSet<EdgeId>> {
        self.kinds.get(&kind)
    }

    #[cfg(test)]
    pub(crate) fn outgoing(&self, source: &EntityId) -> Option<&BTreeSet<EdgeId>> {
        self.outgoing.get(source)
    }

    #[cfg(test)]
    pub(crate) fn incoming(&self, target: &EntityId) -> Option<&BTreeSet<EdgeId>> {
        self.incoming.get(target)
    }

    #[cfg(test)]
    pub(crate) fn owners(&self, child: &EntityId) -> Option<&BTreeSet<EntityId>> {
        self.owners.get(child)
    }

    #[cfg(test)]
    pub(crate) fn children_by_name(
        &self,
        owner: &EntityId,
        name: &EntityName,
    ) -> Option<&BTreeSet<EntityId>> {
        self.child_names.get(&(owner.clone(), name.clone()))
    }

    #[cfg(test)]
    pub(crate) fn children_by_kind(
        &self,
        owner: &EntityId,
        kind: NodeKind,
    ) -> Option<&BTreeSet<EntityId>> {
        self.child_kinds.get(&(owner.clone(), kind))
    }
}

/// Typed atomic edge-state transition failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticEdgeIndexError {
    InvalidNodeId(NodeId),
    DuplicateEdge(EdgeId),
    MissingEdge(EdgeId),
    MissingCurrentEdge(EdgeId),
    MissingBucketMember(EdgeId),
    MissingNodeBucketMember(EntityId),
    RetainedIncidentEdge(EntityId),
    InvalidRefresh(EdgeId),
}

/// Owned semantic-index state prepared and published as one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticIndexState {
    nodes: SemanticNodeIndexState,
    edges: SemanticEdgeIndexState,
}

impl SemanticIndexState {
    pub(crate) fn from_graph(graph: &SemanticGraph) -> Self {
        Self {
            nodes: SemanticNodeIndexState::from_graph(graph),
            edges: SemanticEdgeIndexState::from_graph(graph),
        }
    }

    pub(crate) fn apply_changes(
        &self,
        changes: &NormalizedSemanticIndexChanges<'_, '_>,
    ) -> Result<Self, SemanticIndexStateError> {
        let next = Self {
            nodes: self
                .nodes
                .apply_changes(changes)
                .map_err(SemanticIndexStateError::Node)?,
            edges: self
                .edges
                .apply_changes(changes)
                .map_err(SemanticIndexStateError::Edge)?,
        };
        if !next.nodes.is_internally_consistent()
            || !next.edges.is_internally_consistent(&next.nodes.identities)
        {
            return Err(SemanticIndexStateError::InternalStateInvalid);
        }
        Ok(next)
    }

    #[cfg(test)]
    pub(crate) const fn nodes(&self) -> &SemanticNodeIndexState {
        &self.nodes
    }

    #[cfg(test)]
    pub(crate) const fn edges(&self) -> &SemanticEdgeIndexState {
        &self.edges
    }
}

/// Typed composite-state transition failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticIndexStateError {
    Node(SemanticNodeIndexError),
    Edge(SemanticEdgeIndexError),
    InternalStateInvalid,
}

/// One accepted complete derived state paired with its exact graph snapshot.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct AcceptedSemanticIndex<'graph> {
    graph: &'graph SemanticGraph,
    state: SemanticIndexState,
}

#[allow(dead_code)]
impl<'graph> AcceptedSemanticIndex<'graph> {
    pub(crate) fn rebuild(graph: &'graph SemanticGraph) -> Self {
        Self {
            graph,
            state: SemanticIndexState::from_graph(graph),
        }
    }

    pub(crate) fn transition<'current>(
        &self,
        current: &'current SemanticGraph,
        changes: &NormalizedSemanticIndexChanges<'_, '_>,
    ) -> Result<AcceptedSemanticIndex<'current>, SemanticIndexLifecycleError> {
        if !ptr::eq(self.graph, changes.previous()) {
            return Err(SemanticIndexLifecycleError::StaleBaseSnapshot);
        }
        if !ptr::eq(current, changes.current()) {
            return Err(SemanticIndexLifecycleError::WrongTargetSnapshot);
        }
        let state = self
            .state
            .apply_changes(changes)
            .map_err(SemanticIndexLifecycleError::State)?;
        Ok(AcceptedSemanticIndex {
            graph: current,
            state,
        })
    }

    pub(crate) fn query(&self) -> SemanticGraphQuery<'graph> {
        SemanticGraphQuery::from_index_state(self.graph, &self.state)
    }

    pub(crate) fn resolution_index(&self) -> SemanticResolutionIndex<'graph> {
        SemanticResolutionIndex::from_index_state(self.graph, &self.state)
    }

    #[cfg(test)]
    pub(crate) const fn state(&self) -> &SemanticIndexState {
        &self.state
    }

    #[cfg(test)]
    pub(crate) const fn graph(&self) -> &'graph SemanticGraph {
        self.graph
    }
}

/// Typed lifecycle failure produced before a new state is published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticIndexLifecycleError {
    StaleBaseSnapshot,
    WrongTargetSnapshot,
    State(SemanticIndexStateError),
}

fn snapshot_node_entity_id(id: &NodeId) -> Result<EntityId, SemanticEdgeIndexError> {
    EntityId::new(id.as_str()).map_err(|_| SemanticEdgeIndexError::InvalidNodeId(id.clone()))
}

fn graph_edge_by_id<'graph>(
    graph: &'graph SemanticGraph,
    expected: &EdgeId,
) -> Option<&'graph GraphEdge> {
    graph.edges().find(|edge| {
        edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()) == *expected
    })
}

fn insert_edge_membership<Key: Ord + Clone>(
    buckets: &mut BTreeMap<Key, BTreeSet<EdgeId>>,
    key: Key,
    id: &EdgeId,
) {
    buckets.entry(key).or_default().insert(id.clone());
}

fn insert_node_membership<Key: Ord + Clone>(
    buckets: &mut BTreeMap<Key, BTreeSet<EntityId>>,
    key: Key,
    id: &EntityId,
) {
    buckets.entry(key).or_default().insert(id.clone());
}

fn remove_edge_membership<Key: Ord>(
    buckets: &mut BTreeMap<Key, BTreeSet<EdgeId>>,
    key: &Key,
    id: &EdgeId,
) -> Result<(), SemanticEdgeIndexError> {
    let remove_bucket = {
        let Some(ids) = buckets.get_mut(key) else {
            return Err(SemanticEdgeIndexError::MissingBucketMember(id.clone()));
        };
        if !ids.remove(id) {
            return Err(SemanticEdgeIndexError::MissingBucketMember(id.clone()));
        }
        ids.is_empty()
    };
    if remove_bucket {
        buckets.remove(key);
    }
    Ok(())
}

fn remove_node_membership<Key: Ord>(
    buckets: &mut BTreeMap<Key, BTreeSet<EntityId>>,
    key: &Key,
    id: &EntityId,
) -> Result<(), SemanticEdgeIndexError> {
    let remove_bucket = {
        let Some(ids) = buckets.get_mut(key) else {
            return Err(SemanticEdgeIndexError::MissingNodeBucketMember(id.clone()));
        };
        if !ids.remove(id) {
            return Err(SemanticEdgeIndexError::MissingNodeBucketMember(id.clone()));
        }
        ids.is_empty()
    };
    if remove_bucket {
        buckets.remove(key);
    }
    Ok(())
}

fn is_exact_node_partition<'a>(
    identities: &BTreeSet<EntityId>,
    buckets: impl Iterator<Item = &'a BTreeSet<EntityId>>,
) -> bool {
    let mut counts = BTreeMap::<EntityId, usize>::new();
    for bucket in buckets {
        for id in bucket {
            *counts.entry(id.clone()).or_default() += 1;
        }
    }
    identities.iter().all(|id| counts.get(id) == Some(&1))
        && counts.keys().all(|id| identities.contains(id))
}

fn is_exact_edge_partition<'a>(
    identities: &BTreeSet<EdgeId>,
    buckets: impl Iterator<Item = &'a BTreeSet<EdgeId>>,
) -> bool {
    let mut counts = BTreeMap::<EdgeId, usize>::new();
    for bucket in buckets {
        for id in bucket {
            *counts.entry(id.clone()).or_default() += 1;
        }
    }
    identities.iter().all(|id| counts.get(id) == Some(&1))
        && counts.keys().all(|id| identities.contains(id))
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
        let state = SemanticIndexState::from_graph(graph);
        Self::from_state(graph, &state)
    }

    /// Resolves one already accepted owned state through its canonical graph.
    pub(crate) fn from_state(graph: &'graph SemanticGraph, state: &SemanticIndexState) -> Self {
        let NodeViews {
            identities: nodes_by_id,
            names: nodes_by_name,
            kinds: nodes_by_kind,
        } = build_node_views(graph, &state.nodes);
        let EdgeViews {
            identities: edges_by_id,
            kinds: edges_by_kind,
            outgoing: outgoing_edges,
            outgoing_kinds: outgoing_edges_by_kind,
            incoming: incoming_edges,
            incoming_kinds: incoming_edges_by_kind,
            owner_edges: owner_edges_by_child,
            owners: owners_by_child,
            children: children_by_owner,
            child_kinds: children_by_owner_kind,
            child_names: children_by_owner_name,
        } = build_edge_views(graph, &state.edges);

        Self {
            nodes_by_id,
            nodes_by_name,
            nodes_by_kind,
            edges_by_id,
            edges_by_kind,
            outgoing_edges,
            outgoing_edges_by_kind,
            incoming_edges,
            incoming_edges_by_kind,
            owner_edges_by_child,
            owners_by_child,
            children_by_owner,
            children_by_owner_kind,
            children_by_owner_name,
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

struct EdgeViews<'graph> {
    identities: BTreeMap<EdgeId, &'graph GraphEdge>,
    kinds: BTreeMap<EdgeKind, Vec<&'graph GraphEdge>>,
    outgoing: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    outgoing_kinds: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    incoming: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    incoming_kinds: BTreeMap<(EntityId, EdgeKind), Vec<&'graph GraphEdge>>,
    owner_edges: BTreeMap<EntityId, Vec<&'graph GraphEdge>>,
    owners: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    children: BTreeMap<EntityId, Vec<&'graph GraphNode>>,
    child_kinds: BTreeMap<(EntityId, NodeKind), Vec<&'graph GraphNode>>,
    child_names: BTreeMap<(EntityId, EntityName), Vec<&'graph GraphNode>>,
}

fn build_edge_views<'graph>(
    graph: &'graph SemanticGraph,
    state: &SemanticEdgeIndexState,
) -> EdgeViews<'graph> {
    let identities = state
        .identities
        .iter()
        .map(|id| {
            let edge = graph_edge_by_id(graph, id)
                .expect("semantic edge index state must resolve in its canonical graph");
            (id.clone(), edge)
        })
        .collect();

    EdgeViews {
        identities,
        kinds: build_edge_bucket_views(graph, &state.kinds),
        outgoing: build_edge_bucket_views(graph, &state.outgoing),
        outgoing_kinds: build_edge_bucket_views(graph, &state.outgoing_kinds),
        incoming: build_edge_bucket_views(graph, &state.incoming),
        incoming_kinds: build_edge_bucket_views(graph, &state.incoming_kinds),
        owner_edges: build_edge_bucket_views(graph, &state.owner_edges),
        owners: build_node_bucket_views(graph, &state.owners),
        children: build_node_bucket_views(graph, &state.children),
        child_kinds: build_node_bucket_views(graph, &state.child_kinds),
        child_names: build_node_bucket_views(graph, &state.child_names),
    }
}

fn build_edge_bucket_views<'graph, Key: Ord + Clone>(
    graph: &'graph SemanticGraph,
    buckets: &BTreeMap<Key, BTreeSet<EdgeId>>,
) -> BTreeMap<Key, Vec<&'graph GraphEdge>> {
    buckets
        .iter()
        .map(|(key, ids)| {
            let edges = ids
                .iter()
                .map(|id| {
                    graph_edge_by_id(graph, id)
                        .expect("semantic edge bucket must resolve in its canonical graph")
                })
                .collect();
            (key.clone(), edges)
        })
        .collect()
}

fn build_node_bucket_views<'graph, Key: Ord + Clone>(
    graph: &'graph SemanticGraph,
    buckets: &BTreeMap<Key, BTreeSet<EntityId>>,
) -> BTreeMap<Key, Vec<&'graph GraphNode>> {
    buckets
        .iter()
        .map(|(key, ids)| {
            let nodes = ids
                .iter()
                .map(|id| {
                    graph
                        .node(id)
                        .expect("semantic node bucket must resolve in its canonical graph")
                })
                .collect();
            (key.clone(), nodes)
        })
        .collect()
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
