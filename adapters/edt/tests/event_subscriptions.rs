use oneagent_common::EntityId;
use oneagent_edt::{
    EdtEventSubscriptionError, EdtEventSubscriptionHandlerReason, EdtGraphError,
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, GraphNode, NodeId, NodeKind, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticGraph,
};
use oneagent_metadata::{MetadataKind, MetadataSpecificPayload};
use std::fs;
use std::path::Path;
use tempfile::{TempDir, tempdir};

const CONFIGURATION_ID: &str = "10000000-0000-0000-0000-000000000001";
const PRODUCTS_ID: &str = "20000000-0000-0000-0000-000000000001";
const SERVICES_ID: &str = "20000000-0000-0000-0000-000000000002";
const SALES_ID: &str = "30000000-0000-0000-0000-000000000001";
const EVENTS_ID: &str = "40000000-0000-0000-0000-000000000001";
const PRIMARY_SUBSCRIPTION_ID: &str = "50000000-0000-0000-0000-000000000001";
const SECONDARY_SUBSCRIPTION_ID: &str = "50000000-0000-0000-0000-000000000002";

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
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
