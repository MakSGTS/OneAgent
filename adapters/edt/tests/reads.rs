use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, ImpactNodeStatus, ImpactReasonKind,
    NodeId, NodeKind, ResolutionState, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticGraphQuery, SemanticImpactAnalyzer, SemanticImpactOptions, SemanticReferenceCategory,
    SemanticReferenceRequestOutcome,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIGURATION_ID: &str = "10000000-0000-0000-0000-000000000000";
const CATALOG_ID: &str = "20000000-0000-0000-0000-000000000000";
const INFORMATION_REGISTER_ID: &str = "30000000-0000-0000-0000-000000000000";
const QUERY_HOST_ID: &str = "40000000-0000-0000-0000-000000000000";
const UNSUPPORTED_JOIN_EN: &str =
    include_str!("../../../crates/bsl/tests/fixtures/query_language/unsupported_join_en.query");

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

fn multiline_bsl_literal(value: &str) -> String {
    let mut lines = value.split('\n');
    let mut literal = format!(
        "\"{}",
        lines.next().unwrap_or_default().replace('"', "\"\"")
    );
    for line in lines {
        literal.push('\n');
        literal.push_str("    |");
        literal.push_str(&line.replace('"', "\"\""));
    }
    literal.push('"');
    literal
}

fn write_multiline_query_host(root: &Path, query_text: &str, constructor: bool) {
    write_metadata(
        root,
        "Documents",
        "Document",
        "QueryHost",
        QUERY_HOST_ID,
        "QueryHost",
    );
    let literal = multiline_bsl_literal(query_text);
    let declaration = if constructor {
        format!("    Query = New Query({literal});\n")
    } else {
        format!("    Query = New Query;\n    Query.Text = {literal};\n")
    };
    fs::write(
        root.join("src/Documents/QueryHost/ObjectModule.bsl"),
        format!("Procedure ReadData()\n{declaration}EndProcedure\n"),
    )
    .expect("multiline query module must be written");
}

fn create_multiline_query_project(query_text: &str, constructor: bool) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_multiline_query_host(root.path(), query_text, constructor);
    write_supported_negative_targets(root.path());
    root
}

fn create_negative_query_project(query_text: &str) -> tempfile::TempDir {
    let root = create_query_project(query_text);
    write_supported_negative_targets(root.path());
    root
}

fn write_supported_negative_targets(root: &Path) {
    write_metadata(
        root,
        "Catalogs",
        "Catalog",
        "Products",
        "70000000-0000-0000-0000-000000000001",
        "Products",
    );
    write_metadata(
        root,
        "Catalogs",
        "Catalog",
        "NamedProducts",
        "70000000-0000-0000-0000-000000000002",
        "NamedProducts",
    );
    write_metadata(
        root,
        "InformationRegisters",
        "InformationRegister",
        "ProductsInTheSegments",
        "70000000-0000-0000-0000-000000000003",
        "ProductsInTheSegments",
    );
}

fn assert_no_unaccepted_query_edges(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    for query in graph.nodes_by_kind(NodeKind::Query) {
        for kind in [EdgeKind::References, EdgeKind::Writes] {
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

fn assert_no_resolved_query_data_edges(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    for kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
        assert!(result.graph().query().edges_by_kind(kind).is_empty());
    }
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
        ";raw_source#",
        ";range#",
        ";category#",
        ";namespace#",
        ";local_name#",
        ";request#",
        ";outcome#8:resolved",
        ";projection#5:reads",
        ";collection_evidence#",
        ";resolved_target#",
        ";target_kind#",
    ] {
        assert!(source.contains(field));
    }
    assert!(source.contains("/src/Documents/QueryHost/ObjectModule.bsl#query_source_request"));
    assert_eq!(provenance.producer().as_str(), "oneagent.edt.query-reads");
    assert_eq!(provenance.origin(), FactOrigin::Resolved);
    assert_eq!(provenance.confidence(), Confidence::Exact);
    assert_eq!(provenance.resolution(), ResolutionState::Resolved);
}

fn assert_derived_dependency_provenance(edge: &GraphEdge) {
    assert_eq!(edge.provenance().len(), 1);
    let provenance = &edge.provenance()[0];
    let source = provenance
        .source()
        .expect("DependsOn provenance source must exist")
        .as_str();

    for field in [
        ";request#",
        ";query#",
        ";outcome#8:resolved",
        ";projection#10:depends_on",
        ";resolved_target#",
        ";target_kind#",
        ";proving_fact#5:reads",
        ";normalization#21:query_data_dependency",
    ] {
        assert!(source.contains(field), "missing provenance field `{field}`");
    }
    assert_eq!(
        provenance.producer().as_str(),
        "oneagent.edt.query-dependency"
    );
    assert_eq!(provenance.origin(), FactOrigin::Derived);
    assert_eq!(provenance.confidence(), Confidence::Exact);
    assert_eq!(provenance.resolution(), ResolutionState::Resolved);
}

fn assert_query_data_navigation(
    graph: &oneagent_graph::SemanticGraph,
    products_id: &EntityId,
    deletion_queue_id: &EntityId,
    read_products_id: &NodeId,
    read_products_again_id: &NodeId,
    read_deletion_queue_id: &NodeId,
) {
    let query = graph.query();
    for (query_id, target_id) in [
        (read_products_id, products_id),
        (read_products_again_id, products_id),
        (read_deletion_queue_id, deletion_queue_id),
    ] {
        let dependencies = query.direct_dependencies(query_id);
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.iter().all(|dependency| {
            dependency.node().id() == target_id
                && matches!(
                    dependency.edge().kind(),
                    EdgeKind::Reads | EdgeKind::DependsOn
                )
        }));
        for kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
            assert_eq!(query.outgoing_edges_by_kind(query_id, kind).len(), 1);
        }
    }

    let product_usages = query.direct_usages(&node_id(CATALOG_ID));
    assert_eq!(product_usages.len(), 4);
    assert!(
        product_usages
            .iter()
            .all(|usage| { matches!(usage.edge().kind(), EdgeKind::Reads | EdgeKind::DependsOn) })
    );
    for query_id in [read_products_id, read_products_again_id] {
        assert!(
            product_usages
                .iter()
                .any(|usage| usage.node().id().as_str() == query_id.as_str())
        );
    }
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
    let normalized_dependencies = query.edges_by_kind(EdgeKind::DependsOn);
    let products_id = id(CATALOG_ID);
    let deletion_queue_id = id(INFORMATION_REGISTER_ID);
    let read_products_id = query_id("procedure", "ReadProducts");
    let read_products_again_id = query_id("procedure", "ReadProductsAgain");
    let read_deletion_queue_id = query_id("function", "ReadDeletionQueue");

    assert!(first.diagnostics().is_empty());
    assert_eq!(first.reference_statistics().total(), 3);
    assert_eq!(first.reference_statistics().resolved(), 3);
    let query_requests = first
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource);
    assert_eq!(query_requests.len(), 3);
    assert!(query_requests.iter().all(|request| {
        request.outcome() == SemanticReferenceRequestOutcome::Resolved
            && request.candidates().len() == 1
            && request.provenance().len() == 2
    }));
    assert_eq!(reads.len(), 3);
    assert_eq!(normalized_dependencies.len(), 3);
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
    for edge in &normalized_dependencies {
        assert_derived_dependency_provenance(edge);
    }

    assert_query_data_navigation(
        graph,
        &products_id,
        &deletion_queue_id,
        &read_products_id,
        &read_products_again_id,
        &read_deletion_queue_id,
    );

    let report = first.report();
    assert_eq!(report.edges().by_kind().get(&EdgeKind::Reads), Some(&3));
    assert_eq!(report.edges().by_kind().get(&EdgeKind::DependsOn), Some(&3));
    assert_eq!(report.resolution().resolved(), 3);
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
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert_eq!(first.report(), repeated.report());
    assert_no_unaccepted_query_edges(&first);
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
        for edge_kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
            assert!(affected.reasons().iter().any(|reason| {
                reason.kind() == ImpactReasonKind::DependencyPropagation
                    && reason.edge_kind() == Some(edge_kind)
            }));
        }
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
    let query_requests = first
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource);
    if code == SemanticDiagnosticCode::ReferenceUnresolved {
        assert_eq!(query_requests.len(), 1);
        assert_eq!(
            query_requests[0].outcome(),
            SemanticReferenceRequestOutcome::MissingTarget
        );
        assert_eq!(diagnostic.reference(), query_requests[0].reference());
    } else {
        assert!(query_requests.is_empty());
    }
    assert_no_resolved_query_data_edges(&first);
    assert_no_unaccepted_query_edges(&first);
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
    assert!(first.validate().is_valid());
}

fn assert_rejected_builder_query_case(
    query_text: &str,
    multiline: bool,
    constructor: bool,
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
) {
    let root = if multiline {
        create_multiline_query_project(query_text, constructor)
    } else {
        create_negative_query_project(query_text)
    };
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("rejected query source must remain recoverable");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated rejected query build must succeed");
    let expected_query_id = id(&format!(
        "{QUERY_HOST_ID}:object_module:procedure:ReadData:query:Query"
    ));
    let query_diagnostics = first
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.source_node() == Some(&expected_query_id))
        .collect::<Vec<_>>();

    assert_eq!(query_diagnostics.len(), 1);
    let diagnostic = query_diagnostics[0];
    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.kind(), kind);
    assert_eq!(diagnostic.source_node(), Some(&expected_query_id));
    assert!(first.reference_statistics().total() >= 1);
    assert_eq!(
        first.reference_statistics().outcome_total(),
        first.reference_statistics().total()
    );
    assert_eq!(
        first.reference_statistics().with_provenance(),
        first.reference_statistics().total()
    );
    assert_eq!(first.reference_statistics().unsupported_prefix(), 1);
    assert!(
        first
            .reference_request_query()
            .by_category(SemanticReferenceCategory::QuerySource)
            .is_empty()
    );
    assert_no_resolved_query_data_edges(&first);
    assert_no_unaccepted_query_edges(&first);
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
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
fn reads_unsupported_categories_are_typed_all_or_nothing_and_deterministic() {
    for (query_text, code, kind) in [
        (
            "SELECT Ref FROM Catalog.Products LEFT JOIN Catalog.NamedProducts",
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
        ),
        (
            "SELECT Ref FROM Catalog.Products UNION ALL SELECT Ref FROM Catalog.NamedProducts",
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
        ),
        (
            "SELECT Ref FROM Catalog.Products WHERE Ref IN (SELECT Ref FROM Catalog.NamedProducts)",
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
        ),
        (
            "SELECT Ref FROM Catalog.Products; SELECT Ref FROM Catalog.NamedProducts",
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
        ),
        (
            "SELECT Ref INTO TempProducts FROM Catalog.Products",
            SemanticDiagnosticCode::QueryLanguageTemporaryTableSource,
            SemanticDiagnosticKind::QueryLanguageTemporaryTableSource,
        ),
        (
            "SELECT Ref FROM AccumulationRegister.Products.Balance()",
            SemanticDiagnosticCode::QueryLanguageVirtualTableSource,
            SemanticDiagnosticKind::QueryLanguageVirtualTableSource,
        ),
    ] {
        assert_rejected_builder_query_case(query_text, false, false, code, kind);
    }
}

#[test]
fn reads_multiline_constructor_rejects_join_without_partial_reads() {
    assert_rejected_builder_query_case(
        UNSUPPORTED_JOIN_EN,
        true,
        true,
        SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
        SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
    );
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
    let requests = ambiguous
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].outcome(),
        SemanticReferenceRequestOutcome::AmbiguousTarget
    );
    assert_eq!(
        ambiguous.diagnostics()[0].reference(),
        requests[0].reference()
    );
    assert_no_resolved_query_data_edges(&ambiguous);
    assert_no_unaccepted_query_edges(&ambiguous);
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
    let requests = incompatible
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].outcome(),
        SemanticReferenceRequestOutcome::IncompatibleTargetKind
    );
    assert_eq!(
        incompatible.diagnostics()[0].reference(),
        requests[0].reference()
    );
    assert_no_resolved_query_data_edges(&incompatible);
    assert_no_unaccepted_query_edges(&incompatible);
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
    let before_dependencies = before.graph().query().edges_by_kind(EdgeKind::DependsOn);
    let after_dependencies = after.graph().query().edges_by_kind(EdgeKind::DependsOn);
    let before_request = before
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource)[0];
    let after_request = after
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource)[0];

    assert_eq!(before_query.id(), after_query.id());
    assert_eq!(before_owner.id(), after_owner.id());
    assert_eq!(before_reads.len(), 1);
    assert_eq!(after_reads.len(), 1);
    assert_eq!(before_reads[0].source(), after_reads[0].source());
    assert_eq!(before_reads[0].target(), after_reads[0].target());
    assert_eq!(before_reads[0].kind(), after_reads[0].kind());
    assert_eq!(before_dependencies.len(), 1);
    assert_eq!(after_dependencies.len(), 1);
    assert_eq!(
        before_dependencies[0].source(),
        after_dependencies[0].source()
    );
    assert_eq!(
        before_dependencies[0].target(),
        after_dependencies[0].target()
    );
    assert_eq!(before_request.id(), after_request.id());
    assert_ne!(before_request.provenance(), after_request.provenance());
    let build_diff = before.diff(&after);
    assert!(build_diff.reference_requests().added().is_empty());
    assert!(build_diff.reference_requests().removed().is_empty());
    assert_eq!(build_diff.reference_requests().modified().len(), 1);
    assert!(before.validate().is_valid());
    assert!(after.validate().is_valid());
}
