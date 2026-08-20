use oneagent_common::EntityId;
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    AccessRight, AccessRightRowRestriction, EdgeKind, ImpactNodeStatus, NodeId, NodeKind,
    SemanticGraph, SemanticImpactAnalyzer, SemanticImpactOptions,
};
use std::path::{Path, PathBuf};

fn grants_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grants_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn conditional_access_right_id() -> EntityId {
    AccessRight::new_with_row_restriction(
        id("bb9ecb4f-1ae1-4cfd-b2d1-badd172736e9"),
        id("Read"),
        Some(
            AccessRightRowRestriction::new("WHERE NOT DeletionMark")
                .expect("fixture condition must be valid"),
        ),
        Vec::new(),
    )
    .expect("fixture conditional access right must be valid")
    .id()
    .clone()
}

fn without_node(graph: &SemanticGraph, removed: &EntityId) -> SemanticGraph {
    let mut filtered = SemanticGraph::new();
    for node in graph.nodes().filter(|node| node.id() != removed) {
        filtered.insert_node(node.clone());
    }
    for edge in graph
        .edges()
        .filter(|edge| edge.source() != removed && edge.target() != removed)
    {
        filtered
            .insert_edge(edge.clone())
            .expect("filtered edge endpoints must exist");
    }
    filtered
}

#[test]
fn conditional_grants_are_visible_to_generic_indexes_diff_impact_and_reports() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&grants_fixture())
        .expect("real EDT grants fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&grants_fixture())
        .expect("repeated real EDT grants fixture must build");
    let graph = first.graph();
    let conditional_id = conditional_access_right_id();
    let conditional_node_id = NodeId::new(conditional_id.as_str());

    let query = graph.query();
    let conditional = query
        .node(&conditional_node_id)
        .expect("complete Query index must find the conditional access right");
    assert_eq!(conditional.kind(), NodeKind::AccessRight);
    assert_eq!(
        conditional
            .access_right_payload()
            .and_then(|payload| payload.row_restriction())
            .expect("conditional access right must preserve typed payload")
            .condition(),
        "WHERE NOT DeletionMark"
    );
    assert_eq!(
        graph
            .resolution_index()
            .resolve_entity_id(&conditional_id)
            .expect("complete Resolution index must find the conditional access right")
            .id(),
        &conditional_id
    );
    assert_eq!(
        query
            .incoming_edges_by_kind(&conditional_node_id, EdgeKind::Grants)
            .len(),
        1
    );
    assert_eq!(
        query
            .outgoing_edges_by_kind(&conditional_node_id, EdgeKind::References)
            .len(),
        1
    );
    assert_eq!(query.nodes_by_kind(NodeKind::AccessRight).len(), 39);

    assert!(first.diagnostics().is_empty());
    assert!(first.validate().is_valid());
    assert_eq!(first.report(), repeated.report());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());

    let current = without_node(graph, &conditional_id);
    let diff = graph.diff(&current);
    assert_eq!(diff.removed_nodes().len(), 1);
    assert_eq!(diff.removed_nodes()[0].id(), &conditional_node_id);
    assert_eq!(diff.removed_edges().len(), 2);
    assert!(diff.removed_edges().iter().any(|change| {
        change.edge_kind() == EdgeKind::Grants && change.target() == &conditional_node_id
    }));
    assert!(diff.removed_edges().iter().any(|change| {
        change.edge_kind() == EdgeKind::References && change.source() == &conditional_node_id
    }));
    let impact =
        SemanticImpactAnalyzer::analyze(graph, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("conditional access-right removal impact must succeed");
    assert!(impact.affected_nodes().iter().any(|affected| {
        affected.node_id() == &conditional_node_id && affected.status() == ImpactNodeStatus::Removed
    }));

    assert_eq!(
        graph.report().nodes().by_kind().get(&NodeKind::AccessRight),
        Some(&39)
    );
    assert_eq!(
        graph.report().edges().by_kind().get(&EdgeKind::Grants),
        Some(&50)
    );
    assert!(current.validate().is_valid());
    assert!(current.query().node_by_entity_id(&conditional_id).is_none());
    assert_eq!(
        current.query().nodes_by_kind(NodeKind::AccessRight).len(),
        38
    );
    assert!(
        current
            .nodes_by_kind(NodeKind::AccessRight)
            .iter()
            .all(|node| node.access_right_payload().is_some())
    );
}
