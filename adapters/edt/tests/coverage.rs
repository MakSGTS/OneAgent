use oneagent_edt::{
    EdtSemanticCoverageRegistry, EdtSemanticCoverageReport, EdtSemanticGraphBuilder,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, NodeKind, SemanticCoverageCapabilityId, SemanticCoverageRegistry,
    SemanticCoverageStatus,
};
use std::path::{Path, PathBuf};

fn grants_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grants_project")
}

#[test]
fn conditional_grants_preserve_supported_coverage_aggregates() {
    let build = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&grants_fixture())
        .expect("real EDT grants fixture must build");
    let report = EdtSemanticCoverageReport::for_build_result(&build);
    let edt = EdtSemanticCoverageRegistry::audit();
    let graph = SemanticCoverageRegistry::audit();

    assert_eq!(report.edt_pipeline(), &edt);
    assert_eq!(report.graph_domain(), &graph);
    assert_eq!(edt.summary().total(), 101);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&96)
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&5)
    );
    assert_eq!(graph.summary().total(), 85);
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&82)
    );
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&3)
    );

    for capability_id in [
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::AccessRight),
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Grants),
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::References),
    ] {
        let capability = edt
            .capability(capability_id)
            .expect("conditional Grants capability must remain registered");
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
    }

    let observed_access_rights = report
        .observed()
        .nodes()
        .get(&NodeKind::AccessRight)
        .expect("real fixture must observe access rights");
    let observed_grants = report
        .observed()
        .edges()
        .get(&EdgeKind::Grants)
        .expect("real fixture must observe Grants edges");
    assert_eq!(observed_access_rights.total(), 39);
    assert_eq!(observed_access_rights.with_provenance(), 39);
    assert_eq!(observed_grants.total(), 50);
    assert_eq!(observed_grants.with_provenance(), 50);
    assert!(report.validation().is_valid());
    assert_eq!(report.build_report(), &build.report());
}
