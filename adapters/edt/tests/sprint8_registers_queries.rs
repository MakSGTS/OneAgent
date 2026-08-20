use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, FactOrigin, GraphNode, ImpactReasonKind, NodeKind, ResolutionState,
    SemanticImpactAnalyzer, SemanticImpactOptions, SemanticReferenceCategory,
    SemanticReferenceRequestOutcome,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

fn sprint8_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint8_registers_queries_project")
}

fn compatibility_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reads_project")
}

fn build(path: &Path) -> EdtSemanticGraphBuildResult {
    FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(path)
        .expect("Sprint 8 representative project must build")
}

fn query_requests(
    result: &EdtSemanticGraphBuildResult,
) -> Vec<&oneagent_graph::SemanticReferenceRequest> {
    result
        .reference_request_query()
        .by_category(SemanticReferenceCategory::QuerySource)
}

fn assert_new_register_production_matrix(result: &EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    let requests = query_requests(result);
    let reads = graph.query().edges_by_kind(EdgeKind::Reads);
    let dependencies = graph.query().edges_by_kind(EdgeKind::DependsOn);

    assert!(result.diagnostics().is_empty());
    assert_eq!(requests.len(), 2);
    assert_eq!(reads.len(), 2);
    assert_eq!(dependencies.len(), 2);
    assert_eq!(result.reference_statistics().total(), 2);
    assert_eq!(result.reference_statistics().resolved(), 2);
    assert_eq!(result.report().resolution().resolved(), 2);
    assert_eq!(
        result.report().edges().by_kind().get(&EdgeKind::Reads),
        Some(&2)
    );
    assert_eq!(
        result.report().edges().by_kind().get(&EdgeKind::DependsOn),
        Some(&2)
    );
    assert!(graph.query().edges_by_kind(EdgeKind::Writes).is_empty());
    assert!(result.validate().is_valid());

    for request in requests {
        assert_eq!(request.outcome(), SemanticReferenceRequestOutcome::Resolved);
        assert_eq!(request.candidates().len(), 1);
        assert_eq!(request.provenance().len(), 2);
        assert!(request.provenance().iter().any(|provenance| {
            provenance.producer().as_str() == "oneagent.edt.query-source-collection"
                && provenance.origin() == FactOrigin::Parsed
                && provenance.resolution() == ResolutionState::Unresolved
        }));
        assert!(request.provenance().iter().any(|provenance| {
            provenance.producer().as_str() == "oneagent.edt.query-source-resolution"
                && provenance.origin() == FactOrigin::Resolved
                && provenance.resolution() == ResolutionState::Resolved
        }));
        assert_eq!(
            graph
                .query()
                .owner_edges(&oneagent_graph::NodeId::new(request.source_node().as_str()))
                .len(),
            1
        );
        assert_eq!(
            graph
                .query()
                .owner(&oneagent_graph::NodeId::new(request.source_node().as_str()))
                .expect("every Query must retain its callable owner")
                .kind(),
            NodeKind::Procedure
        );
        for kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
            let edges = graph.outgoing_by_kind(request.source_node(), kind);
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0].target(), &request.candidates()[0]);
            assert_eq!(edges[0].provenance().len(), 1);
            let provenance = &edges[0].provenance()[0];
            let (producer, origin) = match kind {
                EdgeKind::Reads => ("oneagent.edt.query-reads", FactOrigin::Resolved),
                EdgeKind::DependsOn => ("oneagent.edt.query-dependency", FactOrigin::Derived),
                _ => unreachable!("Sprint 8 projection kind must be exact"),
            };
            assert_eq!(provenance.producer().as_str(), producer);
            assert_eq!(provenance.origin(), origin);
            assert_eq!(provenance.resolution(), ResolutionState::Resolved);
            let source = provenance
                .source()
                .expect("projection provenance source must exist")
                .as_str();
            assert!(source.contains(request.id().as_str()));
            assert!(source.contains(request.candidates()[0].as_str()));
            if kind == EdgeKind::DependsOn {
                assert!(source.contains("proving_fact#5:reads"));
            }
        }
        assert!(
            graph
                .outgoing_by_kind(request.source_node(), EdgeKind::References)
                .is_empty()
        );
    }
}

fn assert_four_category_compatibility(
    compatibility: &EdtSemanticGraphBuildResult,
    registers: &EdtSemanticGraphBuildResult,
) {
    let mut expected_kinds = query_requests(compatibility)
        .into_iter()
        .chain(query_requests(registers))
        .map(|request| request.expected_kinds()[0])
        .collect::<Vec<_>>();
    expected_kinds.sort();

    assert_eq!(
        expected_kinds,
        [
            NodeKind::Metadata(MetadataKind::Catalog),
            NodeKind::Metadata(MetadataKind::Catalog),
            NodeKind::Metadata(MetadataKind::InformationRegister),
            NodeKind::Metadata(MetadataKind::AccumulationRegister),
            NodeKind::Metadata(MetadataKind::AccountingRegister),
        ]
    );
    for result in [compatibility, registers] {
        assert!(result.diagnostics().is_empty());
        assert_eq!(
            result.graph().query().edges_by_kind(EdgeKind::Reads).len(),
            query_requests(result).len()
        );
        assert_eq!(
            result
                .graph()
                .query()
                .edges_by_kind(EdgeKind::DependsOn)
                .len(),
            query_requests(result).len()
        );
    }
}

fn assert_register_impact(result: &EdtSemanticGraphBuildResult) {
    let previous = result.graph().clone();
    let mut current = previous.clone();
    let target_id = EntityId::new("3f1de785-2fe5-4a59-8998-b4f9b74f2c55")
        .expect("target identifier must be valid");
    let target = previous
        .node(&target_id)
        .expect("InventoryCost target must exist");
    current.insert_node(GraphNode::new_with_provenance(
        target.id().clone(),
        EntityName::new("InventoryCostChanged").expect("target name must be valid"),
        target.kind(),
        target.provenance().to_vec(),
    ));
    let diff = previous.diff(&current);
    let impact =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("Sprint 8 impact analysis must succeed");
    let query_id = query_requests(result)
        .into_iter()
        .find(|request| request.candidates() == [target_id.clone()])
        .expect("InventoryCost request must exist")
        .source_node();
    let affected = impact
        .affected_nodes()
        .iter()
        .find(|node| node.node_id().as_str() == query_id.as_str())
        .expect("InventoryCost Query must be affected exactly once");

    for kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
        assert!(affected.reasons().iter().any(|reason| {
            reason.kind() == ImpactReasonKind::DependencyPropagation
                && reason.edge_kind() == Some(kind)
        }));
    }
}

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(current).expect("fixture directory must be readable") {
            let path = entry.expect("fixture entry must be readable").path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push(
                    path.strip_prefix(root)
                        .expect("fixture file must be below root")
                        .to_path_buf(),
                );
            }
        }
    }

    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort();
    files
}

fn copy_fixture(root: &Path, destination: &Path, reversed: bool) {
    let mut files = fixture_files(root);
    if reversed {
        files.reverse();
    }
    for relative in files {
        let target = destination.join(&relative);
        fs::create_dir_all(target.parent().expect("fixture file must have a parent"))
            .expect("fixture parent must be created");
        fs::copy(root.join(&relative), target).expect("fixture file must be copied");
    }
}

fn assert_repeated_and_reordered_builds_are_equal() {
    let root = tempfile::tempdir().expect("temporary fixture root must be created");
    copy_fixture(&sprint8_fixture(), root.path(), false);
    let first = build(root.path());
    let repeated = build(root.path());
    fs::remove_dir_all(root.path().join("src")).expect("temporary source tree must be removed");
    copy_fixture(&sprint8_fixture(), root.path(), true);
    let reordered = build(root.path());

    for candidate in [&repeated, &reordered] {
        assert_eq!(first.reference_requests(), candidate.reference_requests());
        assert_eq!(first.diagnostics(), candidate.diagnostics());
        assert_eq!(
            first.reference_statistics(),
            candidate.reference_statistics()
        );
        assert_eq!(first.report(), candidate.report());
        assert!(first.graph().diff(candidate.graph()).is_empty());
        assert!(first.diff(candidate).is_empty());
    }
}

#[test]
fn sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible() {
    let registers = build(&sprint8_fixture());
    let compatibility = build(&compatibility_fixture());

    assert_new_register_production_matrix(&registers);
    assert_four_category_compatibility(&compatibility, &registers);
    assert_register_impact(&registers);
    assert_repeated_and_reordered_builds_are_equal();
}
