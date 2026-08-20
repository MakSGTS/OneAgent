use oneagent_edt::{
    EdtSemanticCoverageRegistry, EdtSemanticCoverageReport, EdtSemanticGraphBuilder,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, NodeKind, SemanticCoverageCapabilityId, SemanticCoverageRegistry,
    SemanticCoverageStatus,
};
use oneagent_metadata::MetadataKind;
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

fn xdto_services_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint13_xdto_services_project")
}

#[test]
fn xdto_service_fixture_closes_nodes_and_ownership_coverage() {
    let build = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&xdto_services_fixture())
        .expect("live-derived XDTO/service fixture must build");
    let report = EdtSemanticCoverageReport::for_build_result(&build);
    let edt = report.edt_pipeline();
    let graph = report.graph_domain();

    for kind in [
        NodeKind::XdtoType,
        NodeKind::HttpServiceUrlTemplate,
        NodeKind::HttpServiceMethod,
        NodeKind::WebServiceOperation,
        NodeKind::WebServiceParameter,
    ] {
        for capability_id in [
            SemanticCoverageCapabilityId::SemanticNode(kind),
            SemanticCoverageCapabilityId::OwnershipRelation(kind),
        ] {
            let capability = edt
                .capability(capability_id)
                .expect("XDTO/service capability must be registered");
            assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
            assert_eq!(capability.evidence(), capability.required_evidence());
            assert!(capability.limitations().is_empty());
            assert_eq!(
                capability.representative_tests(),
                [
                    "oneagent_edt::xdto_services::live_derived_fixture_is_consumer_visible_and_deterministic"
                ]
            );
        }
        let observed = report
            .observed()
            .nodes()
            .get(&kind)
            .expect("fixture must observe each new kind");
        assert_eq!(observed.total(), observed.with_provenance());
    }

    assert_eq!(edt.summary().total(), 120);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&115)
    );
    assert_eq!(graph.summary().total(), 96);
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&92)
    );
    assert!(edt.gaps().is_empty());
    assert!(report.validation().is_valid());
}

#[test]
fn report_data_composition_fixture_closes_nodes_and_ownership_coverage() {
    let build = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&report_data_composition_fixture())
        .expect("live-derived Report Data Composition fixture must build");
    let report = EdtSemanticCoverageReport::for_build_result(&build);
    let edt = report.edt_pipeline();
    let graph = report.graph_domain();

    for capability_id in [
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::DataCompositionSchema),
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::DataSet),
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::DataCompositionField),
        SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::DataCompositionSchema),
        SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::DataSet),
        SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::DataCompositionField),
    ] {
        let capability = edt
            .capability(capability_id)
            .expect("Data Composition capability must be registered");
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            [
                "oneagent_edt::report_data_composition::live_derived_fixture_is_typed_consumer_visible_and_deterministic"
            ]
        );
    }

    assert_eq!(edt.summary().total(), 120);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&115)
    );
    assert_eq!(graph.summary().total(), 96);
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&92)
    );
    assert!(edt.gaps().is_empty());
    assert_eq!(
        report
            .observed()
            .nodes()
            .get(&NodeKind::DataCompositionSchema)
            .expect("fixture must observe Schemas")
            .total(),
        7
    );
    assert_eq!(
        report
            .observed()
            .nodes()
            .get(&NodeKind::DataSet)
            .expect("fixture must observe Data Sets")
            .total(),
        6
    );
    assert_eq!(
        report
            .observed()
            .nodes()
            .get(&NodeKind::DataCompositionField)
            .expect("fixture must observe Fields")
            .total(),
        6
    );
    assert!(report.validation().is_valid());
}

#[test]
fn event_subscription_fixture_closes_metadata_node_and_triggers_coverage() {
    let build = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&event_subscriptions_fixture())
        .expect("live-derived Event Subscription fixture must build");
    let report = EdtSemanticCoverageReport::for_build_result(&build);
    let edt = report.edt_pipeline();

    for capability_id in [
        SemanticCoverageCapabilityId::MetadataEntity(MetadataKind::EventSubscription),
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::Metadata(
            MetadataKind::EventSubscription,
        )),
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Triggers),
    ] {
        let capability = edt
            .capability(capability_id)
            .expect("Event Subscription capability must be registered");
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            [
                "oneagent_edt::event_subscriptions::live_derived_fixture_is_consumer_visible_and_deterministic"
            ]
        );
    }

    assert_eq!(edt.summary().total(), 120);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&115)
    );
    assert!(edt.gaps().is_empty());
    assert_eq!(
        report
            .observed()
            .nodes()
            .get(&NodeKind::Metadata(MetadataKind::EventSubscription))
            .expect("fixture must observe Event Subscription metadata")
            .total(),
        3
    );
    assert_eq!(
        report
            .observed()
            .edges()
            .get(&EdgeKind::Triggers)
            .expect("fixture must observe Triggers")
            .total(),
        3
    );
    assert!(report.validation().is_valid());
}

#[test]
fn nested_subsystems_expand_supported_evidence_without_registry_changes() {
    let build = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&subsystem_fixture())
        .expect("provenance-backed Subsystem fixture must build");
    let report = EdtSemanticCoverageReport::for_build_result(&build);
    let edt = EdtSemanticCoverageRegistry::audit();
    let graph = SemanticCoverageRegistry::audit();

    assert_eq!(report.edt_pipeline(), &edt);
    assert_eq!(report.graph_domain(), &graph);
    assert_eq!(edt.summary().total(), 120);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&115)
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&5)
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Unsupported),
        None
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::DeclaredOnly),
        None
    );
    assert_eq!(graph.summary().total(), 96);
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&92)
    );
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&4)
    );

    for capability_id in [
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::Subsystem),
        SemanticCoverageCapabilityId::SemanticNode(NodeKind::Metadata(MetadataKind::Subsystem)),
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Contains),
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Includes),
    ] {
        let capability = edt
            .capability(capability_id)
            .expect("nested Subsystem capability must remain registered");
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
    }

    let observed_subsystems = report
        .observed()
        .nodes()
        .get(&NodeKind::Subsystem)
        .expect("fixture must observe flat Subsystems");
    let observed_metadata_subsystems = report
        .observed()
        .nodes()
        .get(&NodeKind::Metadata(MetadataKind::Subsystem))
        .expect("fixture must observe metadata Subsystems");
    let observed_includes = report
        .observed()
        .edges()
        .get(&EdgeKind::Includes)
        .expect("fixture must observe direct Includes edges");
    assert_eq!(observed_subsystems.total(), 9);
    assert_eq!(observed_subsystems.with_provenance(), 9);
    assert_eq!(observed_metadata_subsystems.total(), 9);
    assert_eq!(observed_metadata_subsystems.with_provenance(), 9);
    assert_eq!(observed_includes.total(), 10);
    assert_eq!(observed_includes.with_provenance(), 10);
    assert!(report.validation().is_valid());
    assert_eq!(report.build_report(), &build.report());
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
    assert_eq!(edt.summary().total(), 120);
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&115)
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&5)
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::Unsupported),
        None
    );
    assert_eq!(
        edt.summary()
            .by_status()
            .get(&SemanticCoverageStatus::DeclaredOnly),
        None
    );
    assert_eq!(graph.summary().total(), 96);
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::Supported),
        Some(&92)
    );
    assert_eq!(
        graph
            .summary()
            .by_status()
            .get(&SemanticCoverageStatus::NotApplicable),
        Some(&4)
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
