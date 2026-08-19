//! Read-only Semantic Query API for semantic graph lookup and traversal.
//!
//! The query layer is source-independent and operates only on an already-built
//! [`SemanticGraph`]. It does not mutate graph storage, read source files,
//! rebuild graph facts, perform semantic resolution or create diagnostics.
//! Unknown identifiers are represented as absent results rather than errors.

use std::collections::{BTreeSet, VecDeque};

use oneagent_common::{EntityId, EntityName};

use crate::{
    EdgeId, EdgeKind, GraphEdge, GraphNode, NodeId, NodeKind, SemanticGraph,
    edge_identity::edge_id as stable_edge_id,
};

/// Direction used by neighbor, dependency and traversal queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticGraphTraversalDirection {
    /// Follow outgoing edges from source to target.
    Downstream,
    /// Follow incoming edges from target to source.
    Upstream,
}

/// Public edge kind filter used by query and traversal methods.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SemanticGraphEdgeFilter {
    /// Allow every edge kind.
    #[default]
    All,
    /// Allow only the listed edge kinds.
    Only(BTreeSet<EdgeKind>),
}

impl SemanticGraphEdgeFilter {
    /// Creates a filter that allows every edge kind.
    #[must_use]
    pub const fn all() -> Self {
        Self::All
    }

    /// Creates a filter that allows exactly one edge kind.
    #[must_use]
    pub fn only(kind: EdgeKind) -> Self {
        Self::Only(BTreeSet::from([kind]))
    }

    /// Creates a filter that allows the supplied edge kinds.
    #[must_use]
    pub fn any(kinds: impl IntoIterator<Item = EdgeKind>) -> Self {
        Self::Only(kinds.into_iter().collect())
    }

    /// Returns whether `kind` is accepted by this filter.
    #[must_use]
    pub fn allows(&self, kind: EdgeKind) -> bool {
        match self {
            Self::All => true,
            Self::Only(kinds) => kinds.contains(&kind),
        }
    }

    fn dependency_only(&self) -> Self {
        match self {
            Self::All => Self::any(DEPENDENCY_EDGE_KINDS),
            Self::Only(kinds) => Self::Only(
                kinds
                    .iter()
                    .copied()
                    .filter(|kind| is_dependency_edge_kind(*kind))
                    .collect(),
            ),
        }
    }
}

/// Options for bounded deterministic graph traversal.
///
/// Traversal uses breadth-first search. `max_depth` is mandatory and strictly
/// limits the number of edges from the start node. Depth `0` returns only the
/// start node when `include_start` is `true`; otherwise it returns an empty
/// result. Unknown start nodes always return an empty result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphTraversalOptions {
    direction: SemanticGraphTraversalDirection,
    max_depth: usize,
    edge_filter: SemanticGraphEdgeFilter,
    include_start: bool,
}

impl SemanticGraphTraversalOptions {
    /// Creates traversal options with all edge kinds and excluded start node.
    #[must_use]
    pub const fn new(direction: SemanticGraphTraversalDirection, max_depth: usize) -> Self {
        Self {
            direction,
            max_depth,
            edge_filter: SemanticGraphEdgeFilter::All,
            include_start: false,
        }
    }

    /// Returns the traversal direction.
    #[must_use]
    pub const fn direction(&self) -> SemanticGraphTraversalDirection {
        self.direction
    }

    /// Returns the strict maximum traversal depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the edge kind filter used by traversal.
    #[must_use]
    pub const fn edge_filter(&self) -> &SemanticGraphEdgeFilter {
        &self.edge_filter
    }

    /// Returns whether the start node is included at depth `0`.
    #[must_use]
    pub const fn include_start(&self) -> bool {
        self.include_start
    }

    /// Sets the edge kind filter and returns the options.
    #[must_use]
    pub fn with_edge_filter(mut self, edge_filter: SemanticGraphEdgeFilter) -> Self {
        self.edge_filter = edge_filter;
        self
    }

    /// Restricts traversal to one edge kind and returns the options.
    #[must_use]
    pub fn with_edge_kind(self, edge_kind: EdgeKind) -> Self {
        self.with_edge_filter(SemanticGraphEdgeFilter::only(edge_kind))
    }

    /// Sets whether to include the start node at depth `0`.
    #[must_use]
    pub const fn with_include_start(mut self, include_start: bool) -> Self {
        self.include_start = include_start;
        self
    }
}

/// Lightweight owned traversal result record.
///
/// The record contains stable identifiers and depth only. It does not borrow or
/// copy graph nodes, keeping traversal results easy to store and compare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphTraversalNode {
    node_id: NodeId,
    depth: usize,
    via_edge: Option<EdgeId>,
}

impl SemanticGraphTraversalNode {
    fn new(node_id: NodeId, depth: usize, via_edge: Option<EdgeId>) -> Self {
        Self {
            node_id,
            depth,
            via_edge,
        }
    }

    /// Returns the reached node identifier.
    #[must_use]
    pub const fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns the breadth-first traversal depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the edge that first discovered this node.
    ///
    /// The start node has no reason edge.
    #[must_use]
    pub const fn via_edge(&self) -> Option<&EdgeId> {
        self.via_edge.as_ref()
    }
}

/// Borrowed direct relation returned by dependency and usage queries.
#[derive(Debug, Clone)]
pub struct SemanticGraphRelation<'graph> {
    node: &'graph GraphNode,
    edge: &'graph GraphEdge,
    edge_id: EdgeId,
    direction: SemanticGraphTraversalDirection,
}

impl<'graph> SemanticGraphRelation<'graph> {
    fn new(
        node: &'graph GraphNode,
        edge: &'graph GraphEdge,
        direction: SemanticGraphTraversalDirection,
    ) -> Self {
        Self {
            node,
            edge,
            edge_id: stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()),
            direction,
        }
    }

    /// Returns the related node.
    #[must_use]
    pub const fn node(&self) -> &'graph GraphNode {
        self.node
    }

    /// Returns the reason edge connecting the query node to the related node.
    #[must_use]
    pub const fn edge(&self) -> &'graph GraphEdge {
        self.edge
    }

    /// Returns the stable identifier of the reason edge.
    #[must_use]
    pub const fn edge_id(&self) -> &EdgeId {
        &self.edge_id
    }

    /// Returns the relation direction.
    #[must_use]
    pub const fn direction(&self) -> SemanticGraphTraversalDirection {
        self.direction
    }
}

/// Borrowed read-only query object for a semantic graph snapshot.
#[derive(Debug, Clone)]
pub struct SemanticGraphQuery<'graph> {
    graph: &'graph SemanticGraph,
}

impl<'graph> SemanticGraphQuery<'graph> {
    /// Creates a read-only query object for `graph`.
    #[must_use]
    pub const fn new(graph: &'graph SemanticGraph) -> Self {
        Self { graph }
    }

    /// Builds the stable edge identifier used by query, diff and validation APIs.
    #[must_use]
    pub fn edge_id(source: &NodeId, target: &NodeId, kind: EdgeKind) -> EdgeId {
        stable_edge_id(source.as_str(), target.as_str(), kind)
    }

    /// Returns `true` when `kind` participates in dependency queries.
    ///
    /// First-version dependency edges are `Calls`, `References`, `Reads`,
    /// `Writes` and `DependsOn`. Ownership `Contains` edges are intentionally
    /// excluded.
    #[must_use]
    pub const fn is_dependency_edge_kind(kind: EdgeKind) -> bool {
        is_dependency_edge_kind(kind)
    }

    /// Looks up a node by stable [`NodeId`] in `O(log n)` graph storage time.
    #[must_use]
    pub fn node(&self, id: &NodeId) -> Option<&'graph GraphNode> {
        entity_id_from_node_id(id).and_then(|id| self.graph.node(&id))
    }

    /// Looks up a node by the concrete entity identifier used by graph storage.
    #[must_use]
    pub fn node_by_entity_id(&self, id: &EntityId) -> Option<&'graph GraphNode> {
        self.graph.node(id)
    }

    /// Returns whether a node exists.
    #[must_use]
    pub fn contains_node(&self, id: &NodeId) -> bool {
        self.node(id).is_some()
    }

    /// Returns all nodes in deterministic `NodeId` order.
    #[must_use]
    pub fn nodes(&self) -> Vec<&'graph GraphNode> {
        self.graph.nodes().collect()
    }

    /// Returns nodes of `kind` in deterministic `NodeId` order.
    #[must_use]
    pub fn nodes_by_kind(&self, kind: NodeKind) -> Vec<&'graph GraphNode> {
        self.graph.nodes_by_kind(kind)
    }

    /// Returns nodes with exact canonical name in deterministic `NodeId` order.
    ///
    /// The first query API version defines canonical name as the existing typed
    /// [`GraphNode::name`] field. It performs exact matching only.
    #[must_use]
    pub fn nodes_by_name(&self, name: &EntityName) -> Vec<&'graph GraphNode> {
        self.graph
            .nodes()
            .filter(|node| node.name() == name)
            .collect()
    }

    /// Returns nodes with exact canonical name and kind in deterministic `NodeId` order.
    #[must_use]
    pub fn nodes_by_name_and_kind(
        &self,
        name: &EntityName,
        kind: NodeKind,
    ) -> Vec<&'graph GraphNode> {
        self.graph
            .nodes()
            .filter(|node| node.name() == name && node.kind() == kind)
            .collect()
    }

    /// Returns all containment owners of `child` in deterministic `NodeId` order.
    ///
    /// The canonical owner relation is `EdgeKind::Contains` from owner to child.
    /// Returning all owners keeps invalid multiple-owner states observable to
    /// callers and to the Validation API.
    #[must_use]
    pub fn owners(&self, child: &NodeId) -> Vec<&'graph GraphNode> {
        self.owner_edges(child)
            .into_iter()
            .filter_map(|edge| self.graph.node(edge.source()))
            .collect()
    }

    /// Returns the single owner of `child`, when exactly one owner exists.
    ///
    /// Returns `None` for unknown nodes, root nodes and invalid multiple-owner
    /// states. Use [`Self::owners`] to inspect all owner candidates.
    #[must_use]
    pub fn owner(&self, child: &NodeId) -> Option<&'graph GraphNode> {
        let owners = self.owners(child);
        if owners.len() == 1 {
            Some(owners[0])
        } else {
            None
        }
    }

    /// Returns containment owner edges of `child` in deterministic edge order.
    #[must_use]
    pub fn owner_edges(&self, child: &NodeId) -> Vec<&'graph GraphEdge> {
        self.incoming_edges_by_kind(child, EdgeKind::Contains)
    }

    /// Returns immediate child nodes owned by `owner` in deterministic `NodeId` order.
    #[must_use]
    pub fn children(&self, owner: &NodeId) -> Vec<&'graph GraphNode> {
        let mut children = self
            .outgoing_edges_by_kind(owner, EdgeKind::Contains)
            .into_iter()
            .filter_map(|edge| self.graph.node(edge.target()))
            .collect::<Vec<_>>();

        children.sort_by_key(|node| node.id().clone());
        children.dedup_by_key(|node| node.id().clone());
        children
    }

    /// Returns immediate child nodes of `kind` in deterministic `NodeId` order.
    #[must_use]
    pub fn children_by_kind(&self, owner: &NodeId, kind: NodeKind) -> Vec<&'graph GraphNode> {
        self.children(owner)
            .into_iter()
            .filter(|node| node.kind() == kind)
            .collect()
    }

    /// Looks up an edge by stable [`EdgeId`].
    ///
    /// The current storage does not maintain a separate edge-id index, so this
    /// lookup scans deterministic edge storage and compares stable identifiers.
    #[must_use]
    pub fn edge(&self, id: &EdgeId) -> Option<&'graph GraphEdge> {
        self.graph.edges().find(|edge| {
            stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()) == *id
        })
    }

    /// Returns whether an edge exists.
    #[must_use]
    pub fn contains_edge(&self, id: &EdgeId) -> bool {
        self.edge(id).is_some()
    }

    /// Returns all edges in deterministic `EdgeId` order.
    #[must_use]
    pub fn edges(&self) -> Vec<&'graph GraphEdge> {
        sorted_edges(self.graph.edges())
    }

    /// Returns edges of `kind` in deterministic `EdgeId` order.
    #[must_use]
    pub fn edges_by_kind(&self, kind: EdgeKind) -> Vec<&'graph GraphEdge> {
        sorted_edges(self.graph.edges().filter(|edge| edge.kind() == kind))
    }

    /// Returns outgoing edges of `node` in deterministic `EdgeId` order.
    ///
    /// Unknown nodes return an empty result.
    #[must_use]
    pub fn outgoing_edges(&self, node: &NodeId) -> Vec<&'graph GraphEdge> {
        let Some(id) = entity_id_from_node_id(node) else {
            return Vec::new();
        };

        sorted_edges(self.graph.outgoing(&id))
    }

    /// Returns incoming edges of `node` in deterministic `EdgeId` order.
    ///
    /// Unknown nodes return an empty result.
    #[must_use]
    pub fn incoming_edges(&self, node: &NodeId) -> Vec<&'graph GraphEdge> {
        let Some(id) = entity_id_from_node_id(node) else {
            return Vec::new();
        };

        sorted_edges(self.graph.incoming(&id))
    }

    /// Returns outgoing edges of `kind` in deterministic `EdgeId` order.
    #[must_use]
    pub fn outgoing_edges_by_kind(&self, node: &NodeId, kind: EdgeKind) -> Vec<&'graph GraphEdge> {
        self.outgoing_edges(node)
            .into_iter()
            .filter(|edge| edge.kind() == kind)
            .collect()
    }

    /// Returns incoming edges of `kind` in deterministic `EdgeId` order.
    #[must_use]
    pub fn incoming_edges_by_kind(&self, node: &NodeId, kind: EdgeKind) -> Vec<&'graph GraphEdge> {
        self.incoming_edges(node)
            .into_iter()
            .filter(|edge| edge.kind() == kind)
            .collect()
    }

    /// Returns direct downstream neighbor nodes in deterministic `NodeId` order.
    #[must_use]
    pub fn downstream_neighbors(&self, node: &NodeId) -> Vec<&'graph GraphNode> {
        self.neighbors(
            node,
            SemanticGraphTraversalDirection::Downstream,
            &SemanticGraphEdgeFilter::All,
        )
    }

    /// Returns direct upstream neighbor nodes in deterministic `NodeId` order.
    #[must_use]
    pub fn upstream_neighbors(&self, node: &NodeId) -> Vec<&'graph GraphNode> {
        self.neighbors(
            node,
            SemanticGraphTraversalDirection::Upstream,
            &SemanticGraphEdgeFilter::All,
        )
    }

    /// Returns direct downstream neighbors filtered by edge kind.
    #[must_use]
    pub fn downstream_neighbors_by_kind(
        &self,
        node: &NodeId,
        kind: EdgeKind,
    ) -> Vec<&'graph GraphNode> {
        self.neighbors(
            node,
            SemanticGraphTraversalDirection::Downstream,
            &SemanticGraphEdgeFilter::only(kind),
        )
    }

    /// Returns direct upstream neighbors filtered by edge kind.
    #[must_use]
    pub fn upstream_neighbors_by_kind(
        &self,
        node: &NodeId,
        kind: EdgeKind,
    ) -> Vec<&'graph GraphNode> {
        self.neighbors(
            node,
            SemanticGraphTraversalDirection::Upstream,
            &SemanticGraphEdgeFilter::only(kind),
        )
    }

    /// Returns direct neighbor nodes for `direction` and `edge_filter`.
    ///
    /// Neighbors are deduplicated by `NodeId` and sorted by `NodeId`.
    #[must_use]
    pub fn neighbors(
        &self,
        node: &NodeId,
        direction: SemanticGraphTraversalDirection,
        edge_filter: &SemanticGraphEdgeFilter,
    ) -> Vec<&'graph GraphNode> {
        let edges = match direction {
            SemanticGraphTraversalDirection::Downstream => self.outgoing_edges(node),
            SemanticGraphTraversalDirection::Upstream => self.incoming_edges(node),
        };
        let mut nodes = edges
            .into_iter()
            .filter(|edge| edge_filter.allows(edge.kind()))
            .filter_map(|edge| match direction {
                SemanticGraphTraversalDirection::Downstream => self.graph.node(edge.target()),
                SemanticGraphTraversalDirection::Upstream => self.graph.node(edge.source()),
            })
            .collect::<Vec<_>>();

        nodes.sort_by_key(|node| node.id().clone());
        nodes.dedup_by_key(|node| node.id().clone());
        nodes
    }

    /// Returns direct dependencies of `node`.
    ///
    /// Dependencies follow outgoing `Calls`, `References`, `Reads`, `Writes`
    /// and `DependsOn` edges. Ownership `Contains` edges are excluded.
    #[must_use]
    pub fn direct_dependencies(&self, node: &NodeId) -> Vec<SemanticGraphRelation<'graph>> {
        self.direct_dependencies_with_filter(node, &SemanticGraphEdgeFilter::All)
    }

    /// Returns direct dependencies filtered by edge kind.
    #[must_use]
    pub fn direct_dependencies_by_kind(
        &self,
        node: &NodeId,
        kind: EdgeKind,
    ) -> Vec<SemanticGraphRelation<'graph>> {
        self.direct_dependencies_with_filter(node, &SemanticGraphEdgeFilter::only(kind))
    }

    /// Returns direct dependencies filtered by edge kinds.
    #[must_use]
    pub fn direct_dependencies_with_filter(
        &self,
        node: &NodeId,
        edge_filter: &SemanticGraphEdgeFilter,
    ) -> Vec<SemanticGraphRelation<'graph>> {
        self.relations(
            node,
            SemanticGraphTraversalDirection::Downstream,
            &edge_filter.dependency_only(),
        )
    }

    /// Returns direct usages of `node`.
    ///
    /// Usages invert direct dependency direction and follow incoming dependency
    /// edges.
    #[must_use]
    pub fn direct_usages(&self, node: &NodeId) -> Vec<SemanticGraphRelation<'graph>> {
        self.direct_usages_with_filter(node, &SemanticGraphEdgeFilter::All)
    }

    /// Returns direct usages filtered by edge kind.
    #[must_use]
    pub fn direct_usages_by_kind(
        &self,
        node: &NodeId,
        kind: EdgeKind,
    ) -> Vec<SemanticGraphRelation<'graph>> {
        self.direct_usages_with_filter(node, &SemanticGraphEdgeFilter::only(kind))
    }

    /// Returns direct usages filtered by edge kinds.
    #[must_use]
    pub fn direct_usages_with_filter(
        &self,
        node: &NodeId,
        edge_filter: &SemanticGraphEdgeFilter,
    ) -> Vec<SemanticGraphRelation<'graph>> {
        self.relations(
            node,
            SemanticGraphTraversalDirection::Upstream,
            &edge_filter.dependency_only(),
        )
    }

    /// Performs bounded deterministic breadth-first traversal.
    ///
    /// Each node appears at most once. Cycles and self-loops cannot make
    /// traversal unbounded because the start node is marked visited before
    /// expansion and `max_depth` is always enforced.
    #[must_use]
    pub fn traverse(
        &self,
        start: &NodeId,
        options: &SemanticGraphTraversalOptions,
    ) -> Vec<SemanticGraphTraversalNode> {
        let Some(start_id) = entity_id_from_node_id(start) else {
            return Vec::new();
        };
        if self.graph.node(&start_id).is_none() {
            return Vec::new();
        }

        let mut visited = BTreeSet::from([start_id.clone()]);
        let mut queue = VecDeque::from([(start_id, 0_usize)]);
        let mut results = Vec::new();

        if options.include_start() {
            results.push(SemanticGraphTraversalNode::new(start.clone(), 0, None));
        }

        while let Some((current, depth)) = queue.pop_front() {
            if depth == options.max_depth() {
                continue;
            }

            let current_node_id = node_id_from_entity(&current);
            for (neighbor_id, edge_id) in self.traversal_neighbors(&current_node_id, options) {
                if visited.insert(neighbor_id.clone()) {
                    let next_depth = depth + 1;
                    queue.push_back((neighbor_id.clone(), next_depth));
                    results.push(SemanticGraphTraversalNode::new(
                        node_id_from_entity(&neighbor_id),
                        next_depth,
                        Some(edge_id),
                    ));
                }
            }
        }

        results
    }

    fn relations(
        &self,
        node: &NodeId,
        direction: SemanticGraphTraversalDirection,
        edge_filter: &SemanticGraphEdgeFilter,
    ) -> Vec<SemanticGraphRelation<'graph>> {
        let edges = match direction {
            SemanticGraphTraversalDirection::Downstream => self.outgoing_edges(node),
            SemanticGraphTraversalDirection::Upstream => self.incoming_edges(node),
        };
        let mut relations = edges
            .into_iter()
            .filter(|edge| edge_filter.allows(edge.kind()))
            .filter_map(|edge| {
                let related = match direction {
                    SemanticGraphTraversalDirection::Downstream => self.graph.node(edge.target()),
                    SemanticGraphTraversalDirection::Upstream => self.graph.node(edge.source()),
                }?;

                Some(SemanticGraphRelation::new(related, edge, direction))
            })
            .collect::<Vec<_>>();

        relations.sort_by(|left, right| {
            (left.node().id(), left.edge_id()).cmp(&(right.node().id(), right.edge_id()))
        });
        relations
    }

    fn traversal_neighbors(
        &self,
        node: &NodeId,
        options: &SemanticGraphTraversalOptions,
    ) -> Vec<(EntityId, EdgeId)> {
        let edges = match options.direction() {
            SemanticGraphTraversalDirection::Downstream => self.outgoing_edges(node),
            SemanticGraphTraversalDirection::Upstream => self.incoming_edges(node),
        };
        let mut neighbors = edges
            .into_iter()
            .filter(|edge| options.edge_filter().allows(edge.kind()))
            .filter_map(|edge| {
                let neighbor = match options.direction() {
                    SemanticGraphTraversalDirection::Downstream => edge.target(),
                    SemanticGraphTraversalDirection::Upstream => edge.source(),
                };

                self.graph.node(neighbor)?;

                Some((
                    neighbor.clone(),
                    stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind()),
                ))
            })
            .collect::<Vec<_>>();

        neighbors.sort_by(|left, right| {
            (node_id_from_entity(&left.0), left.1.clone())
                .cmp(&(node_id_from_entity(&right.0), right.1.clone()))
        });
        neighbors
    }
}

const DEPENDENCY_EDGE_KINDS: [EdgeKind; 5] = [
    EdgeKind::Calls,
    EdgeKind::References,
    EdgeKind::Reads,
    EdgeKind::Writes,
    EdgeKind::DependsOn,
];

const fn is_dependency_edge_kind(kind: EdgeKind) -> bool {
    matches!(
        kind,
        EdgeKind::Calls
            | EdgeKind::References
            | EdgeKind::Reads
            | EdgeKind::Writes
            | EdgeKind::DependsOn
    )
}

fn sorted_edges<'graph>(
    edges: impl IntoIterator<Item = &'graph GraphEdge>,
) -> Vec<&'graph GraphEdge> {
    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort_by_key(|edge| {
        stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind())
    });
    edges.dedup_by_key(|edge| {
        stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind())
    });
    edges
}

fn entity_id_from_node_id(id: &NodeId) -> Option<EntityId> {
    EntityId::new(id.as_str()).ok()
}

fn node_id_from_entity(id: &EntityId) -> NodeId {
    NodeId::new(id.as_str().to_owned())
}
