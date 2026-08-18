use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, ImpactNodeStatus, ImpactReasonKind,
    NodeId, NodeKind, ResolutionState, SemanticCoverageCapabilityId, SemanticCoverageGapPriority,
    SemanticCoverageStatus, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticImpactAnalyzer, SemanticImpactOptions,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIGURATION_ID: &str = "50000000-0000-0000-0000-000000000000";
const DOCUMENT_ID: &str = "ed647f67-f8fe-476b-8823-8d52b365ab20";
const PROCEDURE_ID: &str = "ed647f67-f8fe-476b-8823-8d52b365ab20:object_module:procedure:Posting";
const CASH_REGISTER_ID: &str = "ac997c18-b62c-4bc3-9079-9a729ad5253c";
const REFUND_REGISTER_ID: &str = "f014a53e-bf0e-4dc4-9a8c-93ef663d9108";
const WRITES_PRODUCER: &str = "oneagent.edt.writes-emission";
const BSL_GRAPH_PRODUCER: &str = "oneagent.edt.bsl-graph";

const ARGUMENT_BEARING: &str = include_str!(
    "../../../crates/bsl/tests/fixtures/writes/argument_bearing_information_register.bsl"
);
const COLLECTION_LEVEL: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/collection_level_write.bsl");
const CHAINED_MANAGER: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/chained_manager_receiver.bsl");
const CHAINED_COMMON: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/chained_common_module_receiver.bsl");
const LOCAL_DOCUMENT: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/local_document_value_flow.bsl");
const LOCAL_PREDEFINED: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/local_predefined_item_value_flow.bsl");
const ALIASED_RECORD_SET: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/aliased_register_record_set.bsl");
const BINARY_FILE: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/binary_file_write.bsl");
const TEXT_FILE: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/text_file_write.bsl");
const ARCHIVE_FILE: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/archive_file_write.bsl");
const UI_FORM: &str = include_str!("../../../crates/bsl/tests/fixtures/writes/ui_form_write.bsl");
const EXTERNAL_INPUT: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/external_input_file_write.bsl");
const COMPUTED_RECEIVER: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/computed_receiver_write.bsl");
const ASYNC_SCOPE: &str =
    include_str!("../../../crates/bsl/tests/fixtures/writes/async_scope_write.bsl");

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn writes_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/writes_project")
}

fn write_configuration(root: &Path) {
    let directory = root.join("src/Configuration");
    fs::create_dir_all(&directory).expect("configuration directory must be created");
    fs::write(
        directory.join("Configuration.mdo"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{CONFIGURATION_ID}">
  <name>WritesIntegration</name>
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

fn write_document(
    root: &Path,
    source_name: &str,
    uuid: &str,
    declarations: &[&str],
    object_module: Option<&str>,
    manager_module: Option<&str>,
) {
    let object_directory = root.join("src/Documents").join(source_name);
    fs::create_dir_all(&object_directory).expect("Document directory must be created");
    let mut declaration_xml = String::new();
    for declaration in declarations {
        writeln!(
            declaration_xml,
            "  <registerRecords>{declaration}</registerRecords>"
        )
        .expect("writing to a String must succeed");
    }
    fs::write(
        object_directory.join(format!("{source_name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
  <name>{source_name}</name>
{declaration_xml}</mdclass:Document>
"#
        ),
    )
    .expect("Document descriptor must be written");
    if let Some(source) = object_module {
        fs::write(object_directory.join("ObjectModule.bsl"), source)
            .expect("Object Module must be written");
    }
    if let Some(source) = manager_module {
        fs::write(object_directory.join("ManagerModule.bsl"), source)
            .expect("Manager Module must be written");
    }
}

fn write_catalog_object_module(root: &Path, source: &str) {
    let object_directory = root.join("src/Catalogs/WrongOwner");
    fs::create_dir_all(&object_directory).expect("Catalog directory must be created");
    fs::write(
        object_directory.join("WrongOwner.mdo"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="60000000-0000-0000-0000-000000000001">
  <name>WrongOwner</name>
</mdclass:Catalog>
"#,
    )
    .expect("Catalog descriptor must be written");
    fs::write(object_directory.join("ObjectModule.bsl"), source)
        .expect("Catalog Object Module must be written");
}

fn writes_diagnostics(
    result: &oneagent_edt::EdtSemanticGraphBuildResult,
) -> Vec<&SemanticDiagnostic> {
    result
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .provenance()
                .iter()
                .any(|provenance| provenance.producer().as_str() == WRITES_PRODUCER)
        })
        .collect()
}

fn diagnostic_context(diagnostic: &SemanticDiagnostic) -> &str {
    diagnostic.provenance()[0]
        .source()
        .expect("diagnostic provenance source must exist")
        .as_str()
}

fn assert_no_placeholder_nodes(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    assert!(result.graph().nodes_by_kind(NodeKind::Unknown).is_empty());
    assert!(
        result
            .graph()
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
            .is_empty()
    );
}

fn assert_writes_provenance(edge: &GraphEdge) {
    assert_eq!(edge.provenance().len(), 1);
    let provenance = &edge.provenance()[0];
    assert_eq!(provenance.origin(), FactOrigin::Resolved);
    assert_eq!(provenance.confidence(), Confidence::Exact);
    assert_eq!(provenance.resolution(), ResolutionState::Resolved);
    assert_eq!(provenance.producer().as_str(), WRITES_PRODUCER);
    let source = provenance
        .source()
        .expect("Writes provenance source must exist")
        .as_str();
    for field in [
        "#writes",
        "procedure_id#",
        "procedure_name#7:Posting",
        "module_id#",
        "module_artifact#",
        "owner_id#",
        "owner_name#",
        "candidate_line#",
        "candidate_column#",
        "raw_statement#",
        "register_name#",
        "normalized_register#",
        "declaration_descriptor#",
        "raw_declaration#",
        "declaration_kind#21:accumulation_register",
        "declaration_ordinals#",
        "resolved_target#",
        "target_kind#21:accumulation_register",
        "parser_stage#",
        "declaration_reader_stage#",
        "resolver_stage#",
        "contributor_stage#",
    ] {
        assert!(source.contains(field), "missing provenance field {field}");
    }
}

fn assert_writes_query_navigation(
    graph: &oneagent_graph::SemanticGraph,
    target_ids: &[EntityId; 2],
) {
    let query = graph.query();
    let writes = query.edges_by_kind(EdgeKind::Writes);
    let procedure_id = id(PROCEDURE_ID);
    let procedure_node_id = node_id(PROCEDURE_ID);

    assert_eq!(writes.len(), 2);
    assert_eq!(
        graph.node(&procedure_id).map(GraphNode::kind),
        Some(NodeKind::Procedure)
    );
    for (edge, target_id) in writes.iter().zip(target_ids) {
        assert_eq!(edge.source(), &procedure_id);
        assert_eq!(edge.target(), target_id);
        assert_eq!(edge.kind(), EdgeKind::Writes);
        assert_eq!(
            graph.node(edge.target()).map(GraphNode::kind),
            Some(NodeKind::Metadata(MetadataKind::AccumulationRegister))
        );
        assert_writes_provenance(edge);
    }

    let outgoing = query.outgoing_edges_by_kind(&procedure_node_id, EdgeKind::Writes);
    assert_eq!(outgoing, writes);
    let dependencies = query.direct_dependencies(&procedure_node_id);
    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        dependencies
            .iter()
            .map(|relation| relation.node().id().clone())
            .collect::<BTreeSet<_>>(),
        target_ids.iter().cloned().collect()
    );
    assert!(
        dependencies
            .iter()
            .all(|relation| relation.edge().kind() == EdgeKind::Writes)
    );

    for target_id in target_ids {
        let target_node_id = node_id(target_id.as_str());
        let incoming = query.incoming_edges_by_kind(&target_node_id, EdgeKind::Writes);
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].source(), &procedure_id);
        let usages = query.direct_usages(&target_node_id);
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].node().id(), &procedure_id);
        assert_eq!(usages[0].edge().kind(), EdgeKind::Writes);
        assert!(
            query
                .outgoing_edges_by_kind(&target_node_id, EdgeKind::Writes)
                .is_empty()
        );
    }
    assert!(
        query
            .incoming_edges_by_kind(&procedure_node_id, EdgeKind::Writes)
            .is_empty()
    );
    assert_eq!(query.edges_by_kind(EdgeKind::Writes), outgoing);
}

fn assert_writes_coverage(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    let coverage = result.coverage_report();
    let writes_capability = coverage
        .edt_pipeline()
        .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes))
        .expect("Writes coverage capability must remain registered");
    assert_eq!(
        writes_capability.status(),
        SemanticCoverageStatus::Supported
    );
    assert_eq!(
        writes_capability.evidence(),
        writes_capability.required_evidence()
    );
    assert!(writes_capability.missing_evidence().is_empty());
    assert!(writes_capability.limitations().is_empty());
    assert_eq!(
        writes_capability.representative_tests(),
        [
            "oneagent_edt::writes::writes_full_builder_emits_canonical_edges_with_query_coverage_and_repeated_build_evidence"
        ]
    );
    assert_eq!(
        coverage
            .edt_pipeline()
            .gaps_by_priority(SemanticCoverageGapPriority::Critical)
            .len(),
        0
    );
    assert_eq!(
        coverage
            .edt_pipeline()
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .len(),
        0
    );
    assert_eq!(
        coverage
            .edt_pipeline()
            .gaps_by_priority(SemanticCoverageGapPriority::Medium)
            .len(),
        5
    );
    for (priority, expected) in [
        (SemanticCoverageGapPriority::Critical, 0),
        (SemanticCoverageGapPriority::High, 0),
        (SemanticCoverageGapPriority::Medium, 5),
    ] {
        let combined = coverage.graph_domain().gaps_by_priority(priority).len()
            + coverage.edt_pipeline().gaps_by_priority(priority).len();
        assert_eq!(combined, expected);
    }
    assert_eq!(coverage.observed().edges()[&EdgeKind::Writes].total(), 2);
}

fn write_resolution_failure_project(root: &Path) {
    write_configuration(root);
    write_document(
        root,
        "ResolutionDocument",
        "60000000-0000-0000-0000-000000000004",
        &[
            "InformationRegister.Unsupported",
            "AccumulationRegister.Missing",
            "AccumulationRegister.Incompatible",
            "AccumulationRegister.Ambiguous",
            "AccumulationRegister.Collision",
            "AccumulationRegister.COLLISION",
        ],
        Some(concat!(
            "Procedure Posting()\n",
            "    RegisterRecords.Undeclared.Write();\n",
            "    RegisterRecords.Unsupported.Write();\n",
            "    RegisterRecords.Missing.Write();\n",
            "    RegisterRecords.Incompatible.Write();\n",
            "    RegisterRecords.Ambiguous.Write();\n",
            "    RegisterRecords.Collision.Write();\n",
            "EndProcedure\n",
        )),
        None,
    );
    for (directory, kind, source_name, uuid, semantic_name) in [
        (
            "InformationRegisters",
            "InformationRegister",
            "Unsupported",
            "61000000-0000-0000-0000-000000000001",
            "Unsupported",
        ),
        (
            "Catalogs",
            "Catalog",
            "Incompatible",
            "61000000-0000-0000-0000-000000000002",
            "Incompatible",
        ),
        (
            "AccumulationRegisters",
            "AccumulationRegister",
            "AmbiguousOne",
            "61000000-0000-0000-0000-000000000003",
            "Ambiguous",
        ),
        (
            "AccumulationRegisters",
            "AccumulationRegister",
            "AmbiguousTwo",
            "61000000-0000-0000-0000-000000000004",
            "Ambiguous",
        ),
    ] {
        write_metadata(root, directory, kind, source_name, uuid, semantic_name);
    }
}

fn assert_resolution_failure_diagnostics(diagnostics: &[&SemanticDiagnostic]) {
    let contexts = diagnostics
        .iter()
        .map(|diagnostic| diagnostic_context(diagnostic))
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 6);
    for outcome in [
        "missing_declaration",
        "unsupported_declaration",
        "missing_target",
        "incompatible_target_kind",
        "ambiguous_target",
        "ambiguous_declaration",
    ] {
        assert!(
            contexts.iter().any(|context| context.contains(outcome)),
            "missing Writes resolution outcome {outcome}"
        );
    }
    let incompatible = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code() == SemanticDiagnosticCode::ReferenceIncompatibleKind)
        .expect("incompatible Writes target diagnostic must exist");
    assert_eq!(
        incompatible.candidates(),
        &[id("61000000-0000-0000-0000-000000000002")]
    );
    assert_eq!(
        incompatible.expected_kinds(),
        &[NodeKind::Metadata(MetadataKind::AccumulationRegister)]
    );
    let ambiguous = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic_context(diagnostic).contains("resolver_outcome#16:ambiguous_target")
        })
        .expect("ambiguous Writes target diagnostic must exist");
    assert_eq!(
        ambiguous.candidates(),
        &[
            id("61000000-0000-0000-0000-000000000003"),
            id("61000000-0000-0000-0000-000000000004"),
        ]
    );
    assert!(contexts.iter().any(|context| {
        context.contains("unsupported_declaration")
            && context.contains("InformationRegister.Unsupported")
    }));
    assert!(contexts.iter().any(|context| {
        context.contains("ambiguous_declaration")
            && context.contains("AccumulationRegister.Collision")
            && context.contains("AccumulationRegister.COLLISION")
    }));
}

#[test]
fn writes_full_builder_emits_canonical_edges_with_query_coverage_and_repeated_build_evidence() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&writes_fixture())
        .expect("Writes fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&writes_fixture())
        .expect("repeated Writes fixture build must succeed");
    let graph = first.graph();
    let query = graph.query();
    let procedure_node_id = node_id(PROCEDURE_ID);
    let target_ids = [id(CASH_REGISTER_ID), id(REFUND_REGISTER_ID)];

    assert_writes_query_navigation(graph, &target_ids);

    let generic_call_diagnostics = first
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .provenance()
                .iter()
                .any(|provenance| provenance.producer().as_str() == BSL_GRAPH_PRODUCER)
        })
        .collect::<Vec<_>>();
    assert_eq!(generic_call_diagnostics.len(), 2);
    assert!(writes_diagnostics(&first).is_empty());
    assert_eq!(first.reference_statistics().total(), 4);
    assert_eq!(first.reference_statistics().resolved(), 2);
    assert_eq!(first.reference_statistics().unresolved(), 2);
    assert_eq!(first.reference_statistics().with_provenance(), 4);

    for kind in [
        EdgeKind::Reads,
        EdgeKind::References,
        EdgeKind::DependsOn,
        EdgeKind::Grants,
    ] {
        assert!(
            query
                .outgoing_edges_by_kind(&procedure_node_id, kind)
                .is_empty()
        );
    }
    assert_no_placeholder_nodes(&first);
    assert!(first.validate().is_valid());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );

    assert_writes_coverage(&first);
}

#[test]
fn writes_changed_register_propagates_impact_to_the_writing_procedure() {
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&writes_fixture())
        .expect("Writes fixture must build");
    let previous = result.graph().clone();
    let mut current = previous.clone();
    let target = previous
        .node(&id(CASH_REGISTER_ID))
        .expect("Accumulation Register target must exist");
    current.insert_node(GraphNode::new_with_provenance(
        target.id().clone(),
        EntityName::new("CashAccountBalanceRenamed").expect("name must be valid"),
        target.kind(),
        target.provenance().to_vec(),
    ));
    let diff = previous.diff(&current);
    let impact =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("Writes impact analysis must succeed");
    let affected = impact
        .affected_nodes()
        .iter()
        .find(|node| node.node_id() == &node_id(PROCEDURE_ID))
        .expect("writing Procedure must be affected by the changed register");

    assert_eq!(affected.status(), ImpactNodeStatus::TransitivelyAffected);
    assert!(affected.reasons().iter().any(|reason| {
        reason.kind() == ImpactReasonKind::DependencyPropagation
            && reason.edge_kind() == Some(EdgeKind::Writes)
    }));
}

#[test]
fn writes_full_builder_rejects_repository_boundaries_without_placeholder_edges() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    let object_source = [
        "RegisterRecords.TopLevel.Write();",
        "Procedure Malformed()\n    RegisterRecords.Broken.Write(\nEndProcedure",
        "Function FunctionWriter()\n    RegisterRecords.FunctionTarget.Write();\nEndFunction",
        "Procedure UnsupportedReceiver()\n    Other.Stock.Write();\nEndProcedure",
        ARGUMENT_BEARING,
        COLLECTION_LEVEL,
        CHAINED_MANAGER,
        CHAINED_COMMON,
        LOCAL_DOCUMENT,
        LOCAL_PREDEFINED,
        ALIASED_RECORD_SET,
        BINARY_FILE,
        TEXT_FILE,
        ARCHIVE_FILE,
        UI_FORM,
        EXTERNAL_INPUT,
        COMPUTED_RECEIVER,
        ASYNC_SCOPE,
    ]
    .join("\n");
    write_document(
        root.path(),
        "BoundaryDocument",
        "60000000-0000-0000-0000-000000000002",
        &[],
        Some(&object_source),
        Some(
            "Procedure ManagerWriter()\n    RegisterRecords.ManagerTarget.Write();\nEndProcedure\n",
        ),
    );
    write_catalog_object_module(
        root.path(),
        "Procedure CatalogWriter()\n    RegisterRecords.CatalogTarget.Write();\nEndProcedure\n",
    );

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("unsupported Writes boundaries must remain recoverable");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated unsupported Writes build must succeed");
    let writes_diagnostics = writes_diagnostics(&first);
    let contexts = writes_diagnostics
        .iter()
        .map(|diagnostic| diagnostic_context(diagnostic))
        .collect::<Vec<_>>();

    assert!(
        first
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Writes)
            .is_empty()
    );
    assert!(!writes_diagnostics.is_empty());
    for reason in [
        "malformed_or_incomplete_statement",
        "computed_receiver",
        "extra_receiver_components",
        "collection_level_write",
        "requires_value_flow",
        "unsupported_receiver",
        "non_empty_arguments",
        "missing_containing_symbol",
        "unsupported_containing_symbol.function",
        "unsupported_module_kind.manager_module",
        "unsupported_owner_kind.catalog",
    ] {
        assert!(
            contexts.iter().any(|context| context.contains(reason)),
            "missing rejected Writes reason {reason}"
        );
    }
    assert!(writes_diagnostics.iter().any(|diagnostic| {
        diagnostic.code() == SemanticDiagnosticCode::ReferenceMalformedFormat
            && diagnostic.kind() == SemanticDiagnosticKind::MalformedReferenceFormat
    }));
    assert!(writes_diagnostics.iter().all(|diagnostic| {
        diagnostic.provenance()[0].producer().as_str() == WRITES_PRODUCER
            && diagnostic.provenance()[0].confidence() == Confidence::Exact
            && diagnostic.provenance()[0].resolution() == ResolutionState::Unresolved
    }));
    assert_no_placeholder_nodes(&first);
    assert!(first.validate().is_valid());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn writes_comments_strings_and_property_assignments_create_no_observation() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_document(
        root.path(),
        "SilentDocument",
        "60000000-0000-0000-0000-000000000003",
        &[],
        Some(concat!(
            "Procedure Silent()\n",
            "    RegisterRecords.Stock.Write = True;\n",
            "    Text = \"RegisterRecords.Stock.Write();\";\n",
            "    // RegisterRecords.Stock.Write();\n",
            "EndProcedure\n",
        )),
        None,
    );

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("silent Writes syntax must build");

    assert!(
        result
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Writes)
            .is_empty()
    );
    assert!(writes_diagnostics(&result).is_empty());
    assert_eq!(result.reference_statistics().total(), 1);
    assert_eq!(result.reference_statistics().unresolved(), 1);
    assert_eq!(result.diagnostics().len(), 1);
    assert!(
        result.diagnostics()[0]
            .provenance()
            .iter()
            .all(|provenance| provenance.producer().as_str() == BSL_GRAPH_PRODUCER)
    );
    assert_no_placeholder_nodes(&result);
    assert!(result.validate().is_valid());
}

#[test]
fn writes_resolution_failures_are_typed_counted_sorted_and_deterministic() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_resolution_failure_project(root.path());

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("Writes resolution failures must remain recoverable");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated Writes resolution build must succeed");
    let diagnostics = writes_diagnostics(&first);
    assert_resolution_failure_diagnostics(&diagnostics);

    assert!(
        first
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Writes)
            .is_empty()
    );
    assert_eq!(first.reference_statistics().total(), 12);
    assert_eq!(first.reference_statistics().resolved(), 0);
    assert_eq!(first.reference_statistics().unsupported_prefix(), 1);
    assert_eq!(first.reference_statistics().unresolved(), 8);
    assert_eq!(first.reference_statistics().ambiguous(), 2);
    assert_eq!(first.reference_statistics().incompatible_target_kind(), 1);
    assert_eq!(first.reference_statistics().with_provenance(), 12);
    assert_no_placeholder_nodes(&first);
    assert!(first.validate().is_valid());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn writes_duplicate_occurrences_keep_one_edge_all_evidence_and_per_observation_statistics() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_document(
        root.path(),
        "DuplicateDocument",
        DOCUMENT_ID,
        &["AccumulationRegister.Stock", "AccumulationRegister.Stock"],
        Some(concat!(
            "Procedure Posting()\n",
            "    RegisterRecords.Stock.Write();\n",
            "    RegisterRecords.Stock.Write();\n",
            "EndProcedure\n",
        )),
        None,
    );
    write_metadata(
        root.path(),
        "AccumulationRegisters",
        "AccumulationRegister",
        "Stock",
        "62000000-0000-0000-0000-000000000001",
        "Stock",
    );

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("duplicate Writes project must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated duplicate Writes build must succeed");
    let writes = first.graph().query().edges_by_kind(EdgeKind::Writes);

    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].source(), &id(PROCEDURE_ID));
    assert_eq!(
        writes[0].target(),
        &id("62000000-0000-0000-0000-000000000001")
    );
    assert_eq!(writes[0].provenance().len(), 2);
    assert!(
        writes[0]
            .provenance()
            .windows(2)
            .all(|pair| pair[0].source() < pair[1].source())
    );
    assert!(writes[0].provenance().iter().all(|provenance| {
        provenance
            .source()
            .expect("Writes provenance source must exist")
            .as_str()
            .contains("declaration_ordinals#3:1,2")
    }));
    assert!(writes_diagnostics(&first).is_empty());
    assert_eq!(first.reference_statistics().total(), 4);
    assert_eq!(first.reference_statistics().resolved(), 2);
    assert_eq!(first.reference_statistics().unresolved(), 2);
    assert_eq!(first.diagnostics().len(), 2);
    assert!(first.validate().is_valid());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert_eq!(
        first.reference_statistics(),
        repeated.reference_statistics()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}
