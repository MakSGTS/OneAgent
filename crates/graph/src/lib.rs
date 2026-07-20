//! Stable identifiers used by the `OneAgent` Knowledge Graph.

pub mod identity;
pub mod kind;

pub use identity::{EdgeId, NodeId};
pub use kind::{EdgeKind, NodeKind};

use oneagent_common::{EntityId, EntityName};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

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

/// In-memory `OneAgent Semantic Graph`.
#[derive(Debug, Default, Clone)]
pub struct SemanticGraph {
    nodes: BTreeMap<EntityId, GraphNode>,
    edges: BTreeSet<GraphEdge>,
}

impl SemanticGraph {
    /// Creates an empty semantic graph.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeSet::new(),
        }
    }

    /// Inserts or replaces a node.
    pub fn insert_node(&mut self, node: GraphNode) -> Option<GraphNode> {
        self.nodes.insert(node.id().clone(), node)
    }

    /// Inserts an edge after validating both endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::MissingNode`] when either endpoint is absent.
    pub fn insert_edge(&mut self, edge: GraphEdge) -> Result<bool, GraphError> {
        if !self.nodes.contains_key(edge.source()) {
            return Err(GraphError::MissingNode(edge.source().clone()));
        }

        if !self.nodes.contains_key(edge.target()) {
            return Err(GraphError::MissingNode(edge.target().clone()));
        }

        Ok(self.edges.insert(edge))
    }

    /// Returns a node by identifier.
    #[must_use]
    pub fn node(&self, id: &EntityId) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    /// Returns all nodes of a specified kind.
    #[must_use]
    pub fn nodes_by_kind(&self, kind: NodeKind) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|node| node.kind() == kind)
            .collect()
    }

    /// Returns outgoing edges from a node.
    #[must_use]
    pub fn outgoing(&self, source: &EntityId) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source() == source)
            .collect()
    }

    /// Returns outgoing edges of a specified kind.
    #[must_use]
    pub fn outgoing_by_kind(&self, source: &EntityId, kind: EdgeKind) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source() == source && edge.kind() == kind)
            .collect()
    }

    /// Returns incoming edges to a node.
    #[must_use]
    pub fn incoming(&self, target: &EntityId) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.target() == target)
            .collect()
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns `true` when the graph has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Semantic graph operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// An edge references a node absent from the graph.
    MissingNode(EntityId),
}

impl Display for GraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingNode(id) => write!(formatter, "semantic graph node is missing: {id}"),
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph};
    use oneagent_metadata::MetadataKind;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn inserts_valid_edge() {
        let document_id = id("document.sales");
        let form_id = id("form.sales.main");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            document_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));
        graph.insert_node(GraphNode::new(
            form_id.clone(),
            name("MainForm"),
            NodeKind::Form,
        ));

        let inserted = graph
            .insert_edge(GraphEdge::new(
                document_id.clone(),
                form_id,
                EdgeKind::Contains,
            ))
            .expect("edge endpoints must exist");

        assert!(inserted);
        assert_eq!(graph.outgoing(&document_id).len(), 1);
    }

    #[test]
    fn rejects_edge_with_missing_target() {
        let source_id = id("module.sales");
        let target_id = id("procedure.missing");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            source_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));

        let error = graph
            .insert_edge(GraphEdge::new(
                source_id,
                target_id.clone(),
                EdgeKind::Calls,
            ))
            .expect_err("missing target must be rejected");

        assert_eq!(
            error.to_string(),
            format!("semantic graph node is missing: {target_id}")
        );
    }

    #[test]
    fn filters_nodes_and_edges_by_kind() {
        let module_id = id("module.sales");
        let procedure_id = id("procedure.sales.post");
        let query_id = id("query.sales.balance");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            module_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            procedure_id.clone(),
            name("Post"),
            NodeKind::Procedure,
        ));
        graph.insert_node(GraphNode::new(
            query_id.clone(),
            name("BalanceQuery"),
            NodeKind::Query,
        ));

        graph
            .insert_edge(GraphEdge::new(
                module_id.clone(),
                procedure_id,
                EdgeKind::Contains,
            ))
            .expect("contains edge must be valid");
        graph
            .insert_edge(GraphEdge::new(module_id.clone(), query_id, EdgeKind::Reads))
            .expect("reads edge must be valid");

        assert_eq!(graph.nodes_by_kind(NodeKind::Query).len(), 1);
        assert_eq!(graph.outgoing_by_kind(&module_id, EdgeKind::Reads).len(), 1);
    }
}
