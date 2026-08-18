use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, ImpactNodeStatus, ImpactReasonKind,
    NodeId, NodeKind, ResolutionState, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticGraphQuery, SemanticImpactAnalyzer, SemanticImpactOptions,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIGURATION_ID: &str = "10000000-0000-0000-0000-000000000000";
const CATALOG_ID: &str = "20000000-0000-0000-0000-000000000000";
const INFORMATION_REGISTER_ID: &str = "30000000-0000-0000-0000-000000000000";
const QUERY_HOST_ID: &str = "40000000-0000-0000-0000-000000000000";

fn reads_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reads_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn query_id(owner_kind: &str, owner_name: &str) -> NodeId {
    node_id(&format!(
        "{QUERY_HOST_ID}:object_module:{owner_kind}:{owner_name}:query:Query"
    ))
}

fn write_configuration(root: &Path) {
    let directory = root.join("src/Configuration");
    fs::create_dir_all(&directory).expect("configuration directory must be created");
    fs::write(
        directory.join("Configuration.mdo"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{CONFIGURATION_ID}">
  <name>ReadsTest</name>
</mdclass:Configuration>
"#
        ),
    )
    .expect("configuration descriptor must be written");
}

fn write_metadata(
    root: &Path,
    directory: &str,
    xml_kind: &str,
    source_name: &str,
    uuid: &str,
    semantic_name: &str,
) {
    let object_directory = root.join("src").join(directory).join(source_name);
    fs::create_dir_all(&object_directory).expect("metadata directory must be created");
    fs::write(
        object_directory.join(format!("{source_name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{xml_kind} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
  <name>{semantic_name}</name>
</mdclass:{xml_kind}>
"#
        ),
    )
    .expect("metadata descriptor must be written");
}

fn write_query_host(root: &Path, query_text: &str) {
    write_metadata(
        root,
        "Documents",
        "Document",
        "QueryHost",
        QUERY_HOST_ID,
        "QueryHost",
    );
    fs::write(
        root.join("src/Documents/QueryHost/ObjectModule.bsl"),
        format!(
            "Procedure ReadData()\n    Query = New Query;\n    Query.Text = \"{query_text}\";\nEndProcedure\n"
        ),
    )
    .expect("query module must be written");
}

fn create_query_project(query_text: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_query_host(root.path(), query_text);
    root
}

fn assert_no_query_companion_edges(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    for query in graph.nodes_by_kind(NodeKind::Query) {
        for kind in [EdgeKind::References, EdgeKind::Writes, EdgeKind::DependsOn] {
            assert!(graph.outgoing_by_kind(query.id(), kind).is_empty());
        }
    }
    assert!(graph.nodes_by_kind(NodeKind::Unknown).is_empty());
    assert!(
        graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
            .is_empty()
    );
}

fn assert_resolved_reads_provenance(edge: &GraphEdge) {
    assert_eq!(edge.provenance().len(), 1);
    let provenance = &edge.provenance()[0];
    let source = provenance
        .source()
        .expect("Reads provenance source must exist")
        .as_str();
    for field in [
        ";query#",
        ";owner#",
        ";binding#5:Query",
        ";declaration_line#",
        ";parser_stage#",
        ";resolver_stage#",
        ";contributor_stage#",
        ";raw_source#",
        ";range#",
        ";category#",
        ";namespace#",
        ";local_name#",
        ";resolved_target#",
        ";target_kind#",
    ] {
        assert!(source.contains(field));
    }
    assert!(source.contains("/src/Documents/QueryHost/ObjectModule.bsl#query_reads"));
    assert_eq!(provenance.producer().as_str(), "oneagent.edt.query-reads");
    assert_eq!(provenance.origin(), FactOrigin::Resolved);
    assert_eq!(provenance.confidence(), Confidence::Exact);
    assert_eq!(provenance.resolution(), ResolutionState::Resolved);
}

#[test]
fn reads_real_edt_fixture_emits_both_target_kinds_with_stable_query_navigation() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&reads_fixture())
        .expect("Reads fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&reads_fixture())
        .expect("repeated Reads fixture build must succeed");
    let graph = first.graph();
    let query = graph.query();
    let reads = query.edges_by_kind(EdgeKind::Reads);
    let products_id = id(CATALOG_ID);
    let deletion_queue_id = id(INFORMATION_REGISTER_ID);
    let read_products_id = query_id("procedure", "ReadProducts");
    let read_products_again_id = query_id("procedure", "ReadProductsAgain");
    let read_deletion_queue_id = query_id("function", "ReadDeletionQueue");

    assert!(first.diagnostics().is_empty());
    assert_eq!(first.reference_statistics().total(), 3);
    assert_eq!(first.reference_statistics().resolved(), 3);
    assert_eq!(reads.len(), 3);
    assert_eq!(
        reads
            .iter()
            .filter(|edge| edge.target() == &products_id)
            .count(),
        2
    );
    assert_eq!(
        reads
            .iter()
            .filter(|edge| edge.target() == &deletion_queue_id)
            .count(),
        1
    );
    assert_ne!(read_products_id, read_products_again_id);

    for edge in &reads {
        assert_resolved_reads_provenance(edge);
    }

    for (query_id, target_id) in [
        (&read_products_id, &products_id),
        (&read_products_again_id, &products_id),
        (&read_deletion_queue_id, &deletion_queue_id),
    ] {
        let dependencies = query.direct_dependencies(query_id);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].node().id(), target_id);
        assert_eq!(dependencies[0].edge().kind(), EdgeKind::Reads);
        assert_eq!(
            query.outgoing_edges_by_kind(query_id, EdgeKind::Reads),
            vec![dependencies[0].edge()]
        );
    }

    let product_usages = query.direct_usages(&node_id(CATALOG_ID));
    assert_eq!(product_usages.len(), 2);
    assert!(
        product_usages
            .iter()
            .all(|usage| usage.edge().kind() == EdgeKind::Reads)
    );
    assert!(
        product_usages
            .iter()
            .any(|usage| { usage.node().id().as_str() == read_products_id.as_str() })
    );
    assert!(
        product_usages
            .iter()
            .any(|usage| { usage.node().id().as_str() == read_products_again_id.as_str() })
    );

    let first_edge_ids = reads
        .iter()
        .map(|edge| {
            SemanticGraphQuery::edge_id(
                &node_id(edge.source().as_str()),
                &node_id(edge.target().as_str()),
                edge.kind(),
            )
        })
        .collect::<Vec<_>>();
    let repeated_edge_ids = repeated
        .graph()
        .query()
        .edges_by_kind(EdgeKind::Reads)
        .iter()
        .map(|edge| {
            SemanticGraphQuery::edge_id(
                &node_id(edge.source().as_str()),
                &node_id(edge.target().as_str()),
                edge.kind(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(first_edge_ids, repeated_edge_ids);
    assert_no_query_companion_edges(&first);
    assert!(first.validate().is_valid());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn reads_changed_metadata_target_propagates_impact_to_query_usages() {
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&reads_fixture())
        .expect("Reads fixture must build");
    let previous = result.graph().clone();
    let mut current = previous.clone();
    let target = previous
        .node(&id(CATALOG_ID))
        .expect("Catalog target must exist");
    current.insert_node(GraphNode::new_with_provenance(
        target.id().clone(),
        EntityName::new("ProductsRenamed").expect("name must be valid"),
        target.kind(),
        target.provenance().to_vec(),
    ));
    let diff = previous.diff(&current);

    let impact =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("Reads impact analysis must succeed");

    for query_id in [
        query_id("procedure", "ReadProducts"),
        query_id("procedure", "ReadProductsAgain"),
    ] {
        let affected = impact
            .affected_nodes()
            .iter()
            .find(|node| node.node_id() == &query_id)
            .expect("Query usage must be affected by the changed metadata target");
        assert_eq!(affected.status(), ImpactNodeStatus::TransitivelyAffected);
        assert!(affected.reasons().iter().any(|reason| {
            reason.kind() == ImpactReasonKind::DependencyPropagation
                && reason.edge_kind() == Some(EdgeKind::Reads)
        }));
    }
}

fn assert_rejected_query_case(
    query_text: &str,
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
) {
    let root = create_query_project(query_text);
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("rejected query source must remain recoverable");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated rejected query build must succeed");
    let diagnostic = &first.diagnostics()[0];

    assert_eq!(first.diagnostics().len(), 1);
    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.kind(), kind);
    assert_eq!(
        diagnostic.source_node(),
        Some(&id(&format!(
            "{QUERY_HOST_ID}:object_module:procedure:ReadData:query:Query"
        )))
    );
    assert_eq!(first.reference_statistics().total(), 1);
    assert_eq!(first.reference_statistics().outcome_total(), 1);
    assert_eq!(first.reference_statistics().with_provenance(), 1);
    assert!(
        first
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Reads)
            .is_empty()
    );
    assert_no_query_companion_edges(&first);
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.diff(&repeated).is_empty());
    assert!(first.validate().is_valid());
}

#[test]
fn reads_parser_and_missing_failures_are_typed_counted_and_deterministic() {
    for (query_text, code, kind) in [
        (
            "BROKEN",
            SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
            SemanticDiagnosticKind::QueryLanguageMalformedSyntax,
        ),
        (
            "SELECT Ref FROM Document.Sales",
            SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace,
            SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace,
        ),
        (
            "SELECT Ref FROM &Source",
            SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource,
            SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource,
        ),
        (
            "SELECT Ref FROM Catalog.Missing",
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticKind::UnresolvedTarget,
        ),
    ] {
        assert_rejected_query_case(query_text, code, kind);
    }
}

#[test]
fn reads_ambiguous_target_is_sorted_counted_and_emits_no_edge() {
    let ambiguous_root = create_query_project("SELECT Ref FROM Catalog.Products");
    write_metadata(
        ambiguous_root.path(),
        "Catalogs",
        "Catalog",
        "ProductsOne",
        "50000000-0000-0000-0000-000000000001",
        "Products",
    );
    write_metadata(
        ambiguous_root.path(),
        "Catalogs",
        "Catalog",
        "ProductsTwo",
        "50000000-0000-0000-0000-000000000002",
        "Products",
    );
    let ambiguous = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(ambiguous_root.path())
        .expect("ambiguous query source must remain recoverable");
    assert_eq!(ambiguous.diagnostics().len(), 1);
    assert_eq!(
        ambiguous.diagnostics()[0].code(),
        SemanticDiagnosticCode::ReferenceAmbiguous
    );
    assert_eq!(
        ambiguous.diagnostics()[0].candidates(),
        &[
            id("50000000-0000-0000-0000-000000000001"),
            id("50000000-0000-0000-0000-000000000002"),
        ]
    );
    assert_eq!(ambiguous.reference_statistics().ambiguous(), 1);
    assert!(
        ambiguous
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Reads)
            .is_empty()
    );
    assert_no_query_companion_edges(&ambiguous);
}

#[test]
fn reads_incompatible_target_is_typed_counted_and_emits_no_edge() {
    let incompatible_root = create_query_project("SELECT Ref FROM Catalog.Products");
    write_metadata(
        incompatible_root.path(),
        "Documents",
        "Document",
        "Products",
        "60000000-0000-0000-0000-000000000000",
        "Products",
    );
    let incompatible = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(incompatible_root.path())
        .expect("incompatible query source must remain recoverable");
    assert_eq!(incompatible.diagnostics().len(), 1);
    assert_eq!(
        incompatible.diagnostics()[0].code(),
        SemanticDiagnosticCode::ReferenceIncompatibleKind
    );
    assert_eq!(
        incompatible.diagnostics()[0].kind(),
        SemanticDiagnosticKind::IncompatibleTargetKind
    );
    assert_eq!(
        incompatible.diagnostics()[0].candidates(),
        &[id("60000000-0000-0000-0000-000000000000")]
    );
    assert_eq!(
        incompatible.diagnostics()[0].expected_kinds(),
        &[NodeKind::Metadata(MetadataKind::Catalog)]
    );
    assert_eq!(
        incompatible
            .reference_statistics()
            .incompatible_target_kind(),
        1
    );
    assert!(
        incompatible
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Reads)
            .is_empty()
    );
    assert_no_query_companion_edges(&incompatible);
}

#[test]
fn reads_query_identity_and_edge_identity_survive_query_text_changes() {
    let root = create_query_project("SELECT Ref FROM Catalog.Products");
    write_metadata(
        root.path(),
        "Catalogs",
        "Catalog",
        "Products",
        CATALOG_ID,
        "Products",
    );
    let before = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("initial query graph must build");
    write_query_host(root.path(), "SELECT Ref FROM Catalog.Products AS Product");
    let after = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("changed query graph must build");
    let query_id = query_id("procedure", "ReadData");
    let before_query = before
        .graph()
        .query()
        .node(&query_id)
        .expect("initial Query node must exist");
    let after_query = after
        .graph()
        .query()
        .node(&query_id)
        .expect("changed Query node must exist");
    let before_owner = before
        .graph()
        .query()
        .owner(&query_id)
        .expect("initial Query owner must exist");
    let after_owner = after
        .graph()
        .query()
        .owner(&query_id)
        .expect("changed Query owner must exist");
    let before_reads = before.graph().query().edges_by_kind(EdgeKind::Reads);
    let after_reads = after.graph().query().edges_by_kind(EdgeKind::Reads);

    assert_eq!(before_query.id(), after_query.id());
    assert_eq!(before_owner.id(), after_owner.id());
    assert_eq!(before_reads.len(), 1);
    assert_eq!(after_reads.len(), 1);
    assert_eq!(before_reads[0].source(), after_reads[0].source());
    assert_eq!(before_reads[0].target(), after_reads[0].target());
    assert_eq!(before_reads[0].kind(), after_reads[0].kind());
    assert!(before.validate().is_valid());
    assert!(after.validate().is_valid());
}
