use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    AccessRight, AccessRightRowRestriction, Confidence, EdgeKind, FactOrigin, ImpactNodeStatus,
    NodeId, NodeKind, ProducerId, Provenance, ResolutionState, SemanticGraph,
    SemanticImpactAnalyzer, SemanticImpactOptions, data_composition_field_id, data_set_id,
    data_set_query_id,
};
use std::path::{Path, PathBuf};

fn grants_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grants_project")
}

fn subsystem_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint10_subsystems_project")
}

fn event_subscriptions_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sprint11_event_subscriptions_project")
}

fn report_data_composition_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sprint12_report_data_composition_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

#[test]
fn report_data_composition_is_visible_through_complete_generic_indexes() {
    let fixture = report_data_composition_fixture();
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture)
        .expect("live-derived Report Data Composition fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture)
        .expect("repeated Report Data Composition fixture build must succeed");
    let graph = first.graph();
    let query = graph.query();
    let schema_id = id("b4233d51-daa9-47ff-8b51-f65c31fc8037");
    let data_set_id = data_set_id(&schema_id, &name("DataSet")).expect("Data Set ID must be valid");
    let field_id =
        data_composition_field_id(&data_set_id, &name("Profile")).expect("Field ID must be valid");
    let query_id = data_set_query_id(&data_set_id).expect("Query ID must be valid");

    for (entity_id, kind) in [
        (&schema_id, NodeKind::DataCompositionSchema),
        (&data_set_id, NodeKind::DataSet),
        (&field_id, NodeKind::DataCompositionField),
        (&query_id, NodeKind::Query),
    ] {
        assert_eq!(
            graph
                .resolution_index()
                .resolve_entity_id_of_kind(entity_id, kind)
                .expect("complete Resolution index must find the typed fixture node")
                .id(),
            entity_id
        );
        assert_eq!(
            query
                .node(&NodeId::new(entity_id.as_str()))
                .expect("complete Query index must find the typed fixture node")
                .kind(),
            kind
        );
    }
    assert_eq!(
        query
            .owner(&NodeId::new(schema_id.as_str()))
            .expect("Query index must find the Report owner")
            .id(),
        &id("ce87e3e8-2d05-415b-bd27-c366ca871097")
    );
    assert_eq!(
        query
            .owner(&NodeId::new(data_set_id.as_str()))
            .expect("Query index must find the Schema owner")
            .id(),
        &schema_id
    );
    assert_eq!(
        query
            .children(&NodeId::new(data_set_id.as_str()))
            .into_iter()
            .map(|node| node.id().clone())
            .collect::<Vec<_>>(),
        [field_id, query_id]
    );
    assert!(
        query
            .direct_dependencies(&NodeId::new(data_set_id.as_str()))
            .is_empty()
    );
    assert_eq!(first.report(), repeated.report());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
    assert!(first.validate().is_valid());
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
fn subsystem_hierarchy_complete_indexes_and_provenance_diff_match_repeated_builds() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&subsystem_fixture())
        .expect("provenance-backed Subsystem fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&subsystem_fixture())
        .expect("repeated Subsystem fixture build must succeed");
    let graph = first.graph();
    let dns_core = NodeId::new("09aab5d3-1bb5-481b-bab0-6794171c94af:subsystem");
    let expected_members = [
        id("38675671-3207-4902-87cd-9e6d276ab265"),
        id("93569485-444c-422c-b938-7574b9778420"),
    ];

    assert_eq!(
        graph
            .query()
            .transitive_subsystem_members(&dns_core)
            .into_iter()
            .map(|node| node.id().clone())
            .collect::<Vec<_>>(),
        expected_members
    );
    assert_eq!(
        repeated
            .graph()
            .query()
            .transitive_subsystem_members(&dns_core)
            .into_iter()
            .map(|node| node.id().clone())
            .collect::<Vec<_>>(),
        expected_members
    );
    for member in &expected_members {
        assert_eq!(
            graph
                .resolution_index()
                .resolve_entity_id(member)
                .expect("complete Resolution index must find each transitive member")
                .id(),
            member
        );
    }
    assert_eq!(first.report(), repeated.report());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());

    let changed_provenance = Provenance::new(
        Some(id("sprint10-provenance-transition")),
        ProducerId::new("oneagent.edt.tests.sprint10-provenance-transition"),
        FactOrigin::Derived,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    );
    let mut changed = SemanticGraph::new();
    for source_node in graph.nodes() {
        let mut node = source_node.clone();
        if node.id().as_str() == "e8c846bb-4d2c-4ae3-966f-28d107e54b20:subsystem" {
            node.add_provenance(changed_provenance.clone());
        }
        changed.insert_node(node);
    }
    for source_edge in graph.edges() {
        let mut edge = source_edge.clone();
        if edge.source().as_str() == "62513da0-595d-4b8e-bcba-29d34846ca48:subsystem"
            && edge.target().as_str() == "e8c846bb-4d2c-4ae3-966f-28d107e54b20:subsystem"
            && edge.kind() == EdgeKind::Includes
        {
            edge.add_provenance(changed_provenance.clone());
        }
        changed
            .insert_edge(edge)
            .expect("fixture edge endpoints must remain complete");
    }
    let provenance_diff = graph.diff(&changed);
    assert_eq!(provenance_diff.modified_nodes().len(), 1);
    assert_eq!(provenance_diff.modified_edges().len(), 1);
    assert_eq!(provenance_diff.summary().total_changes(), 2);
    assert!(changed.validate().is_valid());
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

#[test]
fn event_subscriptions_are_visible_through_complete_generic_indexes() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&event_subscriptions_fixture())
        .expect("live-derived Event Subscription fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&event_subscriptions_fixture())
        .expect("repeated Event Subscription fixture build must succeed");
    let graph = first.graph();
    let query = graph.query();
    let subscription = id("84774a24-9794-4005-a6c2-b69c42abd13f");
    let subscription_node = NodeId::new(subscription.as_str());

    assert_eq!(
        graph
            .resolution_index()
            .resolve_entity_id_of_kind(
                &subscription,
                NodeKind::Metadata(oneagent_metadata::MetadataKind::EventSubscription),
            )
            .expect("complete Resolution index must resolve the subscription")
            .id(),
        &subscription
    );
    assert_eq!(
        query
            .nodes_by_kind(NodeKind::Metadata(
                oneagent_metadata::MetadataKind::EventSubscription,
            ))
            .len(),
        3
    );
    assert_eq!(query.owner_edges(&subscription_node).len(), 1);
    assert_eq!(
        query
            .outgoing_edges_by_kind(&subscription_node, EdgeKind::References)
            .len(),
        2
    );
    assert_eq!(
        query
            .outgoing_edges_by_kind(&subscription_node, EdgeKind::Triggers)
            .len(),
        1
    );
    assert_eq!(
        query
            .direct_dependencies_by_kind(&subscription_node, EdgeKind::Triggers)
            .len(),
        0
    );
    assert_eq!(first.report(), repeated.report());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
    assert!(first.validate().is_valid());
}
