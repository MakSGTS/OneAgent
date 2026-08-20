use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtEventSubscriptionError, EdtEventSubscriptionHandlerReason, EdtGraphError,
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, NodeId, NodeKind, NodeModifiedAspect, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticGraph, SemanticImpactAnalyzer, SemanticImpactOptions,
};
use oneagent_metadata::{MetadataKind, MetadataSpecificPayload};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

const CONFIGURATION_ID: &str = "10000000-0000-0000-0000-000000000001";
const PRODUCTS_ID: &str = "20000000-0000-0000-0000-000000000001";
const SERVICES_ID: &str = "20000000-0000-0000-0000-000000000002";
const SALES_ID: &str = "30000000-0000-0000-0000-000000000001";
const EVENTS_ID: &str = "40000000-0000-0000-0000-000000000001";
const PRIMARY_SUBSCRIPTION_ID: &str = "50000000-0000-0000-0000-000000000001";
const SECONDARY_SUBSCRIPTION_ID: &str = "50000000-0000-0000-0000-000000000002";
const LIVE_CONFIGURATION_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const LIVE_PRODUCTS_ID: &str = "92bcb692-56c4-4199-bf7e-e33cdd76a310";
const LIVE_JOB_ID: &str = "dad11c2e-08fc-4a6b-8829-8be6c64c15fc";
const LIVE_AFTER_WRITE_ID: &str = "84774a24-9794-4005-a6c2-b69c42abd13f";
const LIVE_PRESENTATION_FIELDS_ID: &str = "16773cdb-b979-42d5-a2a5-d4b79e8737bd";
const LIVE_UNSUPPORTED_ID: &str = "350a7b29-4ba3-43c4-b77d-47085f53d760";

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn live_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sprint11_event_subscriptions_project")
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("fixture target directory must be created");
    for entry in fs::read_dir(source).expect("fixture directory must be readable") {
        let entry = entry.expect("fixture entry must be readable");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture entry type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("fixture artifact must be copied");
        }
    }
}

fn copied_live_fixture() -> TempDir {
    let target = tempdir().expect("temporary fixture project must be created");
    copy_tree(&live_fixture(), target.path());
    target
}

fn replace_fixture_fragment(path: &Path, old: &str, new: &str) {
    let source = fs::read_to_string(path).expect("fixture artifact must be readable");
    assert!(source.contains(old), "fixture fragment must exist: {old}");
    fs::write(path, source.replacen(old, new, 1)).expect("fixture artifact must be updated");
}

fn named_procedure(graph: &SemanticGraph, value: &str) -> GraphNode {
    graph
        .query()
        .nodes_by_name_and_kind(
            &EntityName::new(value).expect("procedure name must be valid"),
            NodeKind::Procedure,
        )
        .into_iter()
        .next()
        .cloned()
        .expect("fixture procedure must exist")
}

fn project() -> TempDir {
    let project = tempdir().expect("temporary project must be created");
    let configuration = project.path().join("src/Configuration");
    fs::create_dir_all(&configuration).expect("configuration directory must be created");
    fs::write(
        configuration.join("Configuration.mdo"),
        format!(
            concat!(
                "<mdclass:Configuration ",
                "xmlns:mdclass=\"http://g5.1c.ru/v8/dt/metadata/mdclass\" ",
                "uuid=\"{}\"><name>Demo</name></mdclass:Configuration>"
            ),
            CONFIGURATION_ID,
        ),
    )
    .expect("configuration descriptor must be written");
    project
}

fn write_metadata(
    project: &Path,
    directory: &str,
    object_directory: &str,
    element: &str,
    uuid: &str,
    name: &str,
    module: Option<&str>,
) {
    let object = project.join("src").join(directory).join(object_directory);
    fs::create_dir_all(&object).expect("metadata object directory must be created");
    fs::write(
        object.join(format!("{object_directory}.mdo")),
        format!(
            concat!(
                "<mdclass:{} ",
                "xmlns:mdclass=\"http://g5.1c.ru/v8/dt/metadata/mdclass\" ",
                "uuid=\"{}\"><name>{}</name></mdclass:{}>"
            ),
            element, uuid, name, element,
        ),
    )
    .expect("metadata descriptor must be written");
    if let Some(module) = module {
        fs::write(object.join("Module.bsl"), module).expect("module must be written");
    }
}

fn write_subscription(
    project: &Path,
    object_directory: &str,
    uuid: &str,
    name: &str,
    synonym: Option<&str>,
    sources: &[&str],
    handler: &str,
) {
    let object = project
        .join("src/EventSubscriptions")
        .join(object_directory);
    fs::create_dir_all(&object).expect("Event Subscription directory must be created");
    let synonym = synonym.map_or_else(String::new, |value| {
        format!("<synonym><key>en</key><content>{value}</content></synonym>")
    });
    let mut source_values = String::new();
    for source in sources {
        source_values.push_str("<types>");
        source_values.push_str(source);
        source_values.push_str("</types>");
    }
    fs::write(
        object.join(format!("{object_directory}.mdo")),
        format!(
            concat!(
                "<mdclass:EventSubscription ",
                "xmlns:mdclass=\"http://g5.1c.ru/v8/dt/metadata/mdclass\" ",
                "uuid=\"{}\"><name>{}</name>{}",
                "<source>{}</source><event>BeforeWrite</event>",
                "<handler>{}</handler></mdclass:EventSubscription>"
            ),
            uuid, name, synonym, source_values, handler,
        ),
    )
    .expect("Event Subscription descriptor must be written");
}

fn write_valid_metadata(project: &Path) {
    write_metadata(
        project,
        "Catalogs",
        "Products",
        "Catalog",
        PRODUCTS_ID,
        "Products",
        None,
    );
    write_metadata(
        project,
        "Catalogs",
        "Services",
        "Catalog",
        SERVICES_ID,
        "Services",
        None,
    );
    write_metadata(
        project,
        "Documents",
        "Sales",
        "Document",
        SALES_ID,
        "Sales",
        None,
    );
    write_metadata(
        project,
        "CommonModules",
        "Events",
        "CommonModule",
        EVENTS_ID,
        "Events",
        Some(concat!(
            "Procedure BeforeWrite()\n",
            "EndProcedure\n",
            "Procedure ExportedHandler() Export\n",
            "EndProcedure\n",
        )),
    );
}

fn build(project: &Path) -> oneagent_edt::EdtSemanticGraphBuildResult {
    FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project)
        .expect("production graph build must succeed")
}

fn node(graph: &SemanticGraph, identifier: &str) -> GraphNode {
    graph
        .node(&id(identifier))
        .cloned()
        .expect("node must exist")
}

fn graph_snapshot(graph: &SemanticGraph) -> (Vec<GraphNode>, Vec<String>) {
    let nodes = graph.nodes().cloned().collect();
    let edges = graph
        .edges()
        .map(|edge| {
            format!(
                "{}|{}|{:?}|{:?}",
                edge.source(),
                edge.target(),
                edge.kind(),
                edge.provenance()
            )
        })
        .collect();
    (nodes, edges)
}

fn write_valid_subscriptions(project: &Path) {
    write_subscription(
        project,
        "Primary",
        PRIMARY_SUBSCRIPTION_ID,
        "Primary",
        Some("Primary subscription"),
        &[
            "CatalogObject.Products",
            "CatalogManager.Products",
            "CatalogObject",
            "CatalogObject.Products",
        ],
        "CommonModule.Events.BeforeWrite",
    );
    write_subscription(
        project,
        "Secondary",
        SECONDARY_SUBSCRIPTION_ID,
        "Secondary",
        None,
        &["DocumentObject.Sales"],
        "CommonModule.Events.ExportedHandler",
    );
}

fn assert_primary_payload_and_ownership(graph: &SemanticGraph) {
    let primary = node(graph, PRIMARY_SUBSCRIPTION_ID);
    assert_eq!(
        primary.kind(),
        NodeKind::Metadata(MetadataKind::EventSubscription)
    );
    let metadata_payload = primary
        .metadata_payload()
        .expect("Event Subscription payload must exist");
    assert_eq!(
        metadata_payload.common().synonym(),
        Some("Primary subscription")
    );
    let Some(MetadataSpecificPayload::EventSubscription(payload)) = metadata_payload.specific()
    else {
        panic!("Event Subscription specific payload must exist");
    };
    assert_eq!(payload.event().as_str(), "BeforeWrite");
    assert_eq!(
        graph
            .query()
            .owner(&NodeId::new(PRIMARY_SUBSCRIPTION_ID))
            .expect("configuration owner must exist")
            .id(),
        &id(CONFIGURATION_ID)
    );
}

fn assert_valid_relations(graph: &SemanticGraph) {
    let query = graph.query();
    let before_write = query
        .nodes_by_name_and_kind(
            &oneagent_common::EntityName::new("BeforeWrite").expect("name must be valid"),
            NodeKind::Procedure,
        )
        .into_iter()
        .next()
        .expect("non-exported handler must exist")
        .id()
        .clone();
    let exported = query
        .nodes_by_name_and_kind(
            &oneagent_common::EntityName::new("ExportedHandler").expect("name must be valid"),
            NodeKind::Procedure,
        )
        .into_iter()
        .next()
        .expect("exported handler must exist")
        .id()
        .clone();

    let primary = graph.outgoing_by_kind(&id(PRIMARY_SUBSCRIPTION_ID), EdgeKind::References);
    assert_eq!(primary.len(), 3);
    let products = primary
        .iter()
        .find(|edge| edge.target() == &id(PRODUCTS_ID))
        .expect("Products source relation must exist");
    assert_eq!(products.provenance().len(), 4);
    assert!(primary.iter().any(|edge| edge.target() == &id(SERVICES_ID)));
    assert!(primary.iter().any(|edge| edge.target() == &before_write));
    assert_eq!(
        graph
            .outgoing_by_kind(&id(PRIMARY_SUBSCRIPTION_ID), EdgeKind::Triggers)
            .into_iter()
            .map(|edge| edge.target().clone())
            .collect::<Vec<_>>(),
        vec![before_write]
    );

    let secondary = graph.outgoing_by_kind(&id(SECONDARY_SUBSCRIPTION_ID), EdgeKind::References);
    assert_eq!(secondary.len(), 2);
    assert!(secondary.iter().any(|edge| edge.target() == &id(SALES_ID)));
    assert!(secondary.iter().any(|edge| edge.target() == &exported));
    assert_eq!(
        graph
            .outgoing_by_kind(&id(SECONDARY_SUBSCRIPTION_ID), EdgeKind::Triggers)
            .len(),
        1
    );
    assert!(
        primary
            .iter()
            .flat_map(|edge| edge.provenance())
            .all(|provenance| {
                provenance.producer().as_str() == "oneagent.edt.event-subscription-emission"
                    && provenance.source().is_some()
            })
    );
}

fn assert_valid_result(result: &EdtSemanticGraphBuildResult) {
    assert_primary_payload_and_ownership(result.graph());
    assert_valid_relations(result.graph());
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.reference_requests().len(), 0);
    assert_eq!(result.reference_statistics().total(), 6);
    assert_eq!(result.reference_statistics().resolved(), 6);
    assert!(result.validate().is_valid());
}

fn write_failure_project(project: &Path) {
    write_metadata(
        project,
        "Catalogs",
        "Products",
        "Catalog",
        PRODUCTS_ID,
        "Products",
        None,
    );
    for (directory, uuid) in [
        ("DuplicateOne", "30000000-0000-0000-0000-000000000010"),
        ("DuplicateTwo", "30000000-0000-0000-0000-000000000011"),
    ] {
        write_metadata(
            project,
            "Documents",
            directory,
            "Document",
            uuid,
            "Duplicate",
            None,
        );
    }
    write_metadata(
        project,
        "CommonModules",
        "Events",
        "CommonModule",
        EVENTS_ID,
        "Events",
        Some("Function WrongKind()\nEndFunction\n"),
    );
    write_metadata(
        project,
        "CommonModules",
        "Other",
        "CommonModule",
        "40000000-0000-0000-0000-000000000002",
        "Other",
        Some("Procedure Elsewhere()\nEndProcedure\n"),
    );
    write_subscription(
        project,
        "Failures",
        PRIMARY_SUBSCRIPTION_ID,
        "Failures",
        None,
        &[
            "CatalogObject.Products",
            "DocumentObject.Missing",
            "DocumentObject.Duplicate",
            "TaskObject.Products",
            "ConstantValueManager.Flag",
            "Broken.Value.Extra",
        ],
        "CommonModule.Events.Elsewhere",
    );
    write_subscription(
        project,
        "FunctionHandler",
        SECONDARY_SUBSCRIPTION_ID,
        "FunctionHandler",
        None,
        &["CatalogObject.Products"],
        "CommonModule.Events.WrongKind",
    );
}

fn assert_failure_result(result: &EdtSemanticGraphBuildResult) {
    let codes = result
        .diagnostics()
        .iter()
        .map(oneagent_graph::SemanticDiagnostic::code)
        .collect::<Vec<_>>();
    for code in [
        SemanticDiagnosticCode::ReferenceMalformedFormat,
        SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticCode::ReferenceAmbiguous,
        SemanticDiagnosticCode::ReferenceIncompatibleKind,
        SemanticDiagnosticCode::ReferenceInvalidOwner,
    ] {
        assert!(codes.contains(&code), "missing diagnostic code {code:?}");
    }
    assert_eq!(result.diagnostics().len(), 7);
    assert!(result.diagnostics().iter().all(|diagnostic| {
        diagnostic.source_node().is_some() && !diagnostic.provenance().is_empty()
    }));
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == SemanticDiagnosticKind::InvalidOwnerReference
            && !diagnostic.candidates().is_empty()
    }));
    for subscription in [PRIMARY_SUBSCRIPTION_ID, SECONDARY_SUBSCRIPTION_ID] {
        let references = result
            .graph()
            .outgoing_by_kind(&id(subscription), EdgeKind::References);
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].target(), &id(PRODUCTS_ID));
        assert!(
            result
                .graph()
                .outgoing_by_kind(&id(subscription), EdgeKind::Triggers)
                .is_empty()
        );
    }
    assert!(result.graph().nodes_by_kind(NodeKind::Unknown).is_empty());
    assert_eq!(result.reference_requests().len(), 0);
    let statistics = result.reference_statistics();
    assert_eq!(statistics.total(), 9);
    assert_eq!(statistics.resolved(), 2);
    assert_eq!(statistics.unresolved(), 1);
    assert_eq!(statistics.ambiguous(), 1);
    assert_eq!(statistics.incompatible_target_kind(), 2);
    assert_eq!(statistics.unsupported_prefix(), 1);
    assert_eq!(statistics.malformed_format(), 1);
    assert_eq!(statistics.invalid_owner_reference(), 1);
    assert!(result.validate().is_valid());
}

#[test]
fn emits_typed_exact_family_duplicate_and_owned_handler_semantics() {
    let project = project();
    write_valid_metadata(project.path());
    write_valid_subscriptions(project.path());
    let result = build(project.path());
    assert_valid_result(&result);
}

#[test]
fn recoverable_failures_are_typed_counted_and_emit_no_placeholder_relations() {
    let project = project();
    write_failure_project(project.path());
    let result = build(project.path());
    assert_failure_result(&result);
}

#[test]
fn reordered_sources_and_repeated_builds_are_equal() {
    let project = project();
    write_valid_metadata(project.path());
    write_subscription(
        project.path(),
        "Primary",
        PRIMARY_SUBSCRIPTION_ID,
        "Primary",
        None,
        &[
            "CatalogObject.Products",
            "CatalogObject",
            "CatalogObject.Products",
        ],
        "CommonModule.Events.BeforeWrite",
    );
    let first = build(project.path());
    let repeated = build(project.path());
    write_subscription(
        project.path(),
        "Primary",
        PRIMARY_SUBSCRIPTION_ID,
        "Primary",
        None,
        &[
            "CatalogObject.Products",
            "CatalogObject.Products",
            "CatalogObject",
        ],
        "CommonModule.Events.BeforeWrite",
    );
    let reordered = build(project.path());

    assert_eq!(
        graph_snapshot(first.graph()),
        graph_snapshot(repeated.graph())
    );
    assert_eq!(
        graph_snapshot(first.graph()),
        graph_snapshot(reordered.graph())
    );
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(first.diagnostics(), reordered.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert_eq!(
        first.reference_statistics(),
        reordered.reference_statistics()
    );
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert_eq!(first.reference_requests(), reordered.reference_requests());
    assert_eq!(first.report(), repeated.report());
    assert_eq!(first.report(), reordered.report());
    assert_eq!(first.validate(), repeated.validate());
    assert_eq!(first.validate(), reordered.validate());
}

#[test]
fn malformed_handler_is_fatal_before_a_successful_build_result() {
    let project = project();
    write_subscription(
        project.path(),
        "Broken",
        PRIMARY_SUBSCRIPTION_ID,
        "Broken",
        None,
        &["CatalogObject.Products"],
        "CommonModule.Events",
    );

    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project.path())
        .expect_err("malformed handler must abort the build");
    assert!(matches!(
        error,
        EdtGraphError::EventSubscription(EdtEventSubscriptionError::InvalidHandler {
            reason: EdtEventSubscriptionHandlerReason::MissingComponents,
            ..
        })
    ));
}

#[test]
fn project_without_event_subscriptions_preserves_existing_metadata_behavior() {
    let project = project();
    write_metadata(
        project.path(),
        "Catalogs",
        "Products",
        "Catalog",
        PRODUCTS_ID,
        "Products",
        None,
    );

    let result = build(project.path());
    assert_eq!(
        result
            .graph()
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Catalog))
            .len(),
        1
    );
    assert!(
        result
            .graph()
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::EventSubscription))
            .is_empty()
    );
    assert_eq!(result.reference_statistics().total(), 0);
    assert!(result.reference_requests().is_empty());
    assert!(result.diagnostics().is_empty());
    assert!(result.validate().is_valid());
}

#[test]
#[allow(clippy::too_many_lines)]
fn live_derived_fixture_is_consumer_visible_and_deterministic() {
    let first = build(&live_fixture());
    let repeated = build(&live_fixture());
    let graph = first.graph();
    let query = graph.query();
    let after_write = node(graph, LIVE_AFTER_WRITE_ID);
    let metadata_payload = after_write
        .metadata_payload()
        .expect("live Event Subscription payload must exist");
    let Some(MetadataSpecificPayload::EventSubscription(payload)) = metadata_payload.specific()
    else {
        panic!("live Event Subscription specific payload must exist");
    };

    assert_eq!(
        payload.event().as_str(),
        "AfterWriteDataHistoryVersionsProcessing"
    );
    assert_eq!(
        metadata_payload.common().synonym(),
        Some("After write data history versions processing")
    );
    assert_eq!(
        query
            .owner(&NodeId::new(LIVE_AFTER_WRITE_ID))
            .expect("Configuration ownership must be queryable")
            .id(),
        &id(LIVE_CONFIGURATION_ID)
    );
    assert_eq!(
        query
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::EventSubscription))
            .len(),
        3
    );

    let exported = named_procedure(graph, "AfterWriteDataHistoryVersionsProcessing");
    let exact_handler = named_procedure(graph, "BusinessProcessPresentationFieldsGetProcessing");
    let non_exported = named_procedure(graph, "DeleteOldDataHistoryVersions");
    let after_references =
        query.outgoing_edges_by_kind(&NodeId::new(LIVE_AFTER_WRITE_ID), EdgeKind::References);
    assert_eq!(after_references.len(), 2);
    let products = after_references
        .iter()
        .find(|edge| edge.target() == &id(LIVE_PRODUCTS_ID))
        .expect("equivalent manager/object selectors must resolve Products");
    assert_eq!(products.provenance().len(), 2);
    assert!(
        after_references
            .iter()
            .any(|edge| edge.target() == exported.id())
    );

    let exact_references = query.outgoing_edges_by_kind(
        &NodeId::new(LIVE_PRESENTATION_FIELDS_ID),
        EdgeKind::References,
    );
    assert!(
        exact_references
            .iter()
            .any(|edge| edge.target() == &id(LIVE_JOB_ID))
    );
    assert!(
        exact_references
            .iter()
            .any(|edge| edge.target() == exact_handler.id())
    );
    assert_eq!(
        query
            .outgoing_edges_by_kind(&NodeId::new(LIVE_UNSUPPORTED_ID), EdgeKind::References,)
            .into_iter()
            .map(GraphEdge::target)
            .collect::<Vec<_>>(),
        vec![non_exported.id()]
    );

    for (subscription, handler) in [
        (LIVE_AFTER_WRITE_ID, exported.id()),
        (LIVE_PRESENTATION_FIELDS_ID, exact_handler.id()),
        (LIVE_UNSUPPORTED_ID, non_exported.id()),
    ] {
        let subscription = NodeId::new(subscription);
        assert_eq!(
            query
                .outgoing_edges_by_kind(&subscription, EdgeKind::Triggers)
                .into_iter()
                .map(GraphEdge::target)
                .collect::<Vec<_>>(),
            vec![handler]
        );
        assert!(
            query
                .direct_dependencies_by_kind(&subscription, EdgeKind::Triggers)
                .is_empty()
        );
    }
    assert_eq!(
        query
            .direct_dependencies(&NodeId::new(LIVE_AFTER_WRITE_ID))
            .len(),
        2
    );

    assert_eq!(first.diagnostics().len(), 1);
    assert_eq!(
        first.diagnostics()[0].code(),
        SemanticDiagnosticCode::ReferenceUnsupportedPrefix
    );
    assert_eq!(first.reference_requests().len(), 0);
    assert_eq!(first.reference_statistics().total(), 7);
    assert_eq!(first.reference_statistics().resolved(), 6);
    assert_eq!(first.reference_statistics().unsupported_prefix(), 1);
    assert_eq!(
        first
            .report()
            .nodes()
            .by_kind()
            .get(&NodeKind::Metadata(MetadataKind::EventSubscription)),
        Some(&3)
    );
    assert_eq!(
        first.report().edges().by_kind().get(&EdgeKind::References),
        Some(&5)
    );
    assert_eq!(
        first.report().edges().by_kind().get(&EdgeKind::Triggers),
        Some(&3)
    );
    assert!(first.validate().is_valid());
    assert_eq!(graph_snapshot(graph), graph_snapshot(repeated.graph()));
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
    assert_eq!(first.report(), repeated.report());

    let trigger_edge = query
        .outgoing_edges_by_kind(&NodeId::new(LIVE_AFTER_WRITE_ID), EdgeKind::Triggers)[0]
        .clone();
    let mut trigger_only_previous = SemanticGraph::new();
    trigger_only_previous.insert_node(after_write.clone());
    trigger_only_previous.insert_node(exported.clone());
    trigger_only_previous
        .insert_edge(trigger_edge.clone())
        .expect("fixture Triggers edge must be valid");
    let mut trigger_only_current = SemanticGraph::new();
    trigger_only_current.insert_node(after_write);
    trigger_only_current.insert_node(GraphNode::new(
        exported.id().clone(),
        EntityName::new("AfterWriteDataHistoryVersionsProcessingChanged")
            .expect("changed handler name must be valid"),
        NodeKind::Procedure,
    ));
    trigger_only_current
        .insert_edge(trigger_edge)
        .expect("fixture-derived Triggers edge must be valid");
    let trigger_only_diff = trigger_only_previous.diff(&trigger_only_current);
    let impact = SemanticImpactAnalyzer::analyze(
        &trigger_only_previous,
        &trigger_only_current,
        &trigger_only_diff,
        &SemanticImpactOptions::new(1),
    )
    .expect("Triggers-only impact analysis must succeed");
    assert_eq!(impact.affected_nodes().len(), 1);
    assert_eq!(
        impact.affected_nodes()[0].node_id().as_str(),
        exported.id().as_str()
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn production_diffs_cover_subscription_payload_source_handler_and_relation_transitions() {
    let event_project = copied_live_fixture();
    let event_baseline = build(event_project.path());
    let after_path = event_project.path().join(
        "src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo",
    );
    replace_fixture_fragment(
        &after_path,
        "<event>AfterWriteDataHistoryVersionsProcessing</event>",
        "<event>BeforeWrite</event>",
    );
    let event_changed = build(event_project.path());
    let event_diff = event_baseline.graph().diff(event_changed.graph());
    assert_eq!(event_diff.modified_nodes().len(), 1);
    assert_eq!(
        event_diff.modified_nodes()[0].modified_aspects(),
        &[
            NodeModifiedAspect::SemanticContent,
            NodeModifiedAspect::Provenance,
        ]
    );
    assert!(event_diff.added_edges().is_empty());
    assert!(event_diff.removed_edges().is_empty());

    let overlap_project = copied_live_fixture();
    let overlap_baseline = build(overlap_project.path());
    let overlap_path = overlap_project.path().join(
        "src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo",
    );
    replace_fixture_fragment(&overlap_path, "    <types>CatalogObject</types>\n", "");
    let overlap_removed = build(overlap_project.path());
    let overlap_diff = overlap_baseline.graph().diff(overlap_removed.graph());
    assert_eq!(overlap_diff.modified_edges().len(), 1);
    assert_eq!(
        overlap_diff.modified_edges()[0].edge_kind(),
        EdgeKind::References
    );

    let added_project = copied_live_fixture();
    let added_baseline = build(added_project.path());
    let exact_path = added_project.path().join(
        "src/EventSubscriptions/GetBusinessProcessPresentationFields/GetBusinessProcessPresentationFields.mdo",
    );
    replace_fixture_fragment(
        &exact_path,
        "    <types>BusinessProcessManager.Job</types>",
        "    <types>BusinessProcessManager.Job</types>\n    <types>CatalogObject.Products</types>",
    );
    let source_added = build(added_project.path());
    let added_diff = added_baseline.graph().diff(source_added.graph());
    assert_eq!(added_diff.added_edges().len(), 1);
    assert_eq!(
        added_diff.added_edges()[0].edge_kind(),
        EdgeKind::References
    );
    let removed_diff = source_added.graph().diff(added_baseline.graph());
    assert_eq!(removed_diff.removed_edges().len(), 1);
    assert_eq!(
        removed_diff.removed_edges()[0].edge_kind(),
        EdgeKind::References
    );

    let retarget_project = copied_live_fixture();
    let retarget_baseline = build(retarget_project.path());
    let retarget_path = retarget_project.path().join(
        "src/EventSubscriptions/GetBusinessProcessPresentationFields/GetBusinessProcessPresentationFields.mdo",
    );
    replace_fixture_fragment(
        &retarget_path,
        "<types>BusinessProcessManager.Job</types>",
        "<types>CatalogObject.Products</types>",
    );
    let source_retargeted = build(retarget_project.path());
    let retarget_diff = retarget_baseline.graph().diff(source_retargeted.graph());
    assert_eq!(retarget_diff.added_edges().len(), 1);
    assert_eq!(retarget_diff.removed_edges().len(), 1);
    assert!(
        retarget_diff
            .added_edges()
            .iter()
            .chain(retarget_diff.removed_edges())
            .all(|change| change.edge_kind() == EdgeKind::References)
    );

    let handler_project = copied_live_fixture();
    let handler_baseline = build(handler_project.path());
    let handler_path = handler_project.path().join(
        "src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo",
    );
    replace_fixture_fragment(
        &handler_path,
        "CommonModule.DataHistoryManagement.AfterWriteDataHistoryVersionsProcessing",
        "CommonModule.DataHistoryManagement.DeleteOldDataHistoryVersions",
    );
    let handler_retargeted = build(handler_project.path());
    let handler_diff = handler_baseline.graph().diff(handler_retargeted.graph());
    assert_eq!(handler_diff.added_edges().len(), 2);
    assert_eq!(handler_diff.removed_edges().len(), 2);
    for kind in [EdgeKind::References, EdgeKind::Triggers] {
        assert_eq!(
            handler_diff
                .added_edges()
                .iter()
                .filter(|change| change.edge_kind() == kind)
                .count(),
            1
        );
        assert_eq!(
            handler_diff
                .removed_edges()
                .iter()
                .filter(|change| change.edge_kind() == kind)
                .count(),
            1
        );
    }

    let removed_project = copied_live_fixture();
    let removed_baseline = build(removed_project.path());
    fs::remove_dir_all(
        removed_project
            .path()
            .join("src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing"),
    )
    .expect("fixture subscription directory must be removable");
    let subscription_removed = build(removed_project.path());
    let subscription_remove_diff = removed_baseline.graph().diff(subscription_removed.graph());
    assert_eq!(subscription_remove_diff.removed_nodes().len(), 1);
    assert_eq!(
        subscription_remove_diff.removed_nodes()[0].id().as_str(),
        LIVE_AFTER_WRITE_ID
    );
    assert_eq!(subscription_remove_diff.removed_edges().len(), 4);
    let subscription_add_diff = subscription_removed.graph().diff(removed_baseline.graph());
    assert_eq!(subscription_add_diff.added_nodes().len(), 1);
    assert_eq!(subscription_add_diff.added_edges().len(), 4);

    for result in [
        &event_changed,
        &overlap_removed,
        &source_added,
        &source_retargeted,
        &handler_retargeted,
        &subscription_removed,
    ] {
        assert!(result.validate().is_valid());
        assert!(result.reference_requests().is_empty());
    }
}
