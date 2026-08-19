//! Deterministic derived indexes for one complete semantic graph snapshot.

use std::collections::BTreeMap;

use oneagent_common::{EntityId, EntityName};

use crate::{
    EdgeId, EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph, edge_identity::edge_id,
};

/// Crate-internal read-only lookup state derived from one borrowed graph snapshot.
#[derive(Debug)]
pub(crate) struct SemanticIndex<'graph> {
    nodes_by_id: BTreeMap<EntityId, &'graph GraphNode>,
    nodes_by_name: BTreeMap<EntityName, Vec<&'graph GraphNode>>,
    nodes_by_kind: BTreeMap<NodeKind, Vec<&'graph GraphNode>>,
    edges_by_id: BTreeMap<EdgeId, &'graph GraphEdge>,
    edges_by_kind: BTreeMap<EdgeKind, Vec<&'graph GraphEdge>>,
}

impl<'graph> SemanticIndex<'graph> {
    /// Builds lookup state without changing or copying canonical graph facts.
    pub(crate) fn new(graph: &'graph SemanticGraph) -> Self {
        let mut nodes_by_id = BTreeMap::new();
        let mut nodes_by_name = BTreeMap::<EntityName, Vec<&GraphNode>>::new();
        let mut nodes_by_kind = BTreeMap::<NodeKind, Vec<&GraphNode>>::new();

        for node in graph.nodes() {
            nodes_by_id.insert(node.id().clone(), node);
            nodes_by_name
                .entry(node.name().clone())
                .or_default()
                .push(node);
            nodes_by_kind.entry(node.kind()).or_default().push(node);
        }

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

        Self {
            nodes_by_id,
            nodes_by_name,
            nodes_by_kind,
            edges_by_id,
            edges_by_kind,
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
}
