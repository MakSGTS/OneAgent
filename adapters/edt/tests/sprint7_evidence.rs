use oneagent_common::EntityId;
use oneagent_edt::{
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{EdgeKind, FactOrigin, NodeId, NodeKind, ResolutionState};
use oneagent_metadata::MetadataKind;
use std::path::{Path, PathBuf};

const PRODUCTS_ID: &str = "72000000-0000-0000-0000-000000000000";
const PRICE_IMPORT_ID: &str = "72100000-0000-0000-0000-000000000000";
const PRODUCT_SHARED_ID: &str = "72200000-0000-0000-0000-000000000000";
const OPEN_PRICE_IMPORT_ID: &str = "72300000-0000-0000-0000-000000000000";
const COUNTERPARTIES_ID: &str = "73000000-0000-0000-0000-000000000000";
const COUNTERPARTY_SHARED_ID: &str = "73100000-0000-0000-0000-000000000000";
const GLOBAL_OPEN_ID: &str = "74000000-0000-0000-0000-000000000000";
const WORKSPACE_ID: &str = "75000000-0000-0000-0000-000000000000";
const MY_TASKS_ID: &str = "76000000-0000-0000-0000-000000000000";
const TASK_FORM_ID: &str = "76100000-0000-0000-0000-000000000000";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint7_forms_commands_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("test identifier must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn assert_owner(result: &EdtSemanticGraphBuildResult, child: &str, owner: &str) {
    assert_eq!(
        result
            .graph()
            .query()
            .owner(&node_id(child))
            .expect("production child owner must exist")
            .id()
            .as_str(),
        owner
    );
}

fn assert_reference_pair(result: &EdtSemanticGraphBuildResult, source: &str, target: &str) {
    for kind in [EdgeKind::References, EdgeKind::DependsOn] {
        let edges = result.graph().outgoing_by_kind(&id(source), kind);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target().as_str(), target);
        assert!(!edges[0].provenance().is_empty());
    }
}

fn assert_modules_symbols_queries_and_references(result: &EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    let query = graph.query();
    for (module, owner) in [
        (
            "72100000-0000-0000-0000-000000000000:form_module",
            PRICE_IMPORT_ID,
        ),
        (
            "72300000-0000-0000-0000-000000000000:command_module",
            OPEN_PRICE_IMPORT_ID,
        ),
        (
            "74000000-0000-0000-0000-000000000000:command_module",
            GLOBAL_OPEN_ID,
        ),
        (
            "75000000-0000-0000-0000-000000000000:common_module",
            WORKSPACE_ID,
        ),
    ] {
        assert_eq!(
            query
                .node(&node_id(module))
                .expect("production Module must exist")
                .kind(),
            NodeKind::Module
        );
        assert_owner(result, module, owner);
    }
    for (name, kind) in [
        ("LoadPrices", NodeKind::Procedure),
        ("FormCaption", NodeKind::Function),
        ("ShowWorkspace", NodeKind::Procedure),
        ("WorkspaceCaption", NodeKind::Function),
    ] {
        assert!(
            graph
                .nodes()
                .any(|node| node.name().as_str() == name && node.kind() == kind)
        );
    }
    let queries = query.nodes_by_kind(NodeKind::Query);
    assert_eq!(queries.len(), 1);
    assert_eq!(
        query
            .owner(&node_id(queries[0].id().as_str()))
            .expect("production Query owner must exist")
            .kind(),
        NodeKind::Procedure
    );
    assert_eq!(query.edges_by_kind(EdgeKind::Reads).len(), 1);
    assert_reference_pair(result, OPEN_PRICE_IMPORT_ID, COUNTERPARTIES_ID);
    assert_reference_pair(result, GLOBAL_OPEN_ID, MY_TASKS_ID);
}

fn assert_navigation_and_diagnostics(result: &EdtSemanticGraphBuildResult) {
    let query = result.graph().query();
    let subordinate_execute =
        node_id("72300000-0000-0000-0000-000000000000:command_module:procedure:Execute");
    let subordinate_opens = query.outgoing_edges_by_kind(&subordinate_execute, EdgeKind::Opens);
    assert_eq!(subordinate_opens.len(), 3);
    for target in [PRICE_IMPORT_ID, WORKSPACE_ID, COUNTERPARTY_SHARED_ID] {
        assert!(
            subordinate_opens
                .iter()
                .any(|edge| edge.target().as_str() == target)
        );
    }
    assert!(
        subordinate_opens
            .iter()
            .all(|edge| edge.target().as_str() != PRODUCT_SHARED_ID)
    );
    let duplicate = subordinate_opens
        .iter()
        .find(|edge| edge.target().as_str() == PRICE_IMPORT_ID)
        .expect("duplicate navigation target must resolve");
    assert_eq!(duplicate.provenance().len(), 2);

    let common_execute =
        node_id("74000000-0000-0000-0000-000000000000:command_module:procedure:Execute");
    let common_opens = query.outgoing_edges_by_kind(&common_execute, EdgeKind::Opens);
    assert_eq!(common_opens.len(), 2);
    for target in [WORKSPACE_ID, TASK_FORM_ID] {
        assert!(
            common_opens
                .iter()
                .any(|edge| edge.target().as_str() == target)
        );
    }
    for edge in subordinate_opens.iter().copied().chain(common_opens) {
        assert_eq!(edge.provenance()[0].origin(), FactOrigin::Resolved);
        assert_eq!(edge.provenance()[0].resolution(), ResolutionState::Resolved);
        assert_eq!(
            edge.provenance()[0].producer().as_str(),
            "oneagent.edt.form-navigation"
        );
    }
    let navigation_diagnostics = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .provenance()
                .iter()
                .any(|evidence| evidence.producer().as_str() == "oneagent.edt.form-navigation")
        })
        .collect::<Vec<_>>();
    assert_eq!(navigation_diagnostics.len(), 4);
    assert!(navigation_diagnostics.iter().all(|diagnostic| {
        diagnostic.source_node().is_some() && !diagnostic.provenance().is_empty()
    }));
}

#[test]
fn sprint7_repository_fixture_proves_modules_references_and_navigation_end_to_end() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture())
        .expect("Sprint 7 repository fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture())
        .expect("repeated Sprint 7 repository fixture build must succeed");
    let graph = first.graph();
    let query = graph.query();

    assert_modules_symbols_queries_and_references(&first);
    assert_navigation_and_diagnostics(&first);

    assert_eq!(
        query
            .node(&node_id(PRODUCTS_ID))
            .expect("Products Catalog must exist")
            .kind(),
        NodeKind::Metadata(MetadataKind::Catalog)
    );
    assert!(first.validate().is_valid());
    assert_eq!(first.report().resolution(), first.reference_statistics());
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}
