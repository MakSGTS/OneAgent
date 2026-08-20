use std::collections::BTreeSet;

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticCoverageCapability, SemanticCoverageCapabilityId,
    SemanticCoverageCategory, SemanticCoverageEvidence, SemanticCoverageGapPriority,
    SemanticCoverageRegistry, SemanticCoverageReport, SemanticCoverageStatus, SemanticGraph,
    SemanticObservedCoverage, SemanticProvenanceCapability, semantic_coverage_edge_kinds,
    semantic_coverage_node_kinds,
};
use oneagent_metadata::MetadataKind;

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: &EntityId) -> Provenance {
    Provenance::new(
        Some(source.clone()),
        ProducerId::new("oneagent.graph.coverage.tests"),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn observed_fixture(reverse: bool) -> SemanticObservedCoverage {
    let owner_id = id("metadata.document.sales");
    let child_id = id("metadata.document.sales.attribute.company");
    let owner = GraphNode::new_with_provenance(
        owner_id.clone(),
        name("Sales"),
        NodeKind::Metadata(MetadataKind::Document),
        vec![provenance(&owner_id)],
    );
    let child = GraphNode::new_with_provenance(
        child_id.clone(),
        name("Company"),
        NodeKind::Attribute,
        vec![provenance(&child_id)],
    );
    let edge = GraphEdge::new_with_provenance(
        owner_id,
        child_id,
        EdgeKind::Contains,
        vec![provenance(&id("source.contains"))],
    );
    let mut graph = SemanticGraph::new();

    if reverse {
        graph.insert_node(child);
        graph.insert_node(owner);
    } else {
        graph.insert_node(owner);
        graph.insert_node(child);
    }
    graph.insert_edge(edge).expect("edge must be valid");

    SemanticObservedCoverage::for_graph(&graph)
}

#[test]
fn registry_is_non_empty_unique_ordered_and_repeatable() {
    let first = SemanticCoverageRegistry::audit();
    let second = SemanticCoverageRegistry::audit();
    let stable_ids = first
        .capabilities()
        .iter()
        .map(SemanticCoverageCapability::stable_id)
        .collect::<Vec<_>>();
    let mut sorted_ids = stable_ids.clone();
    sorted_ids.sort();

    assert!(!first.capabilities().is_empty());
    assert_eq!(first, second);
    assert_eq!(stable_ids, sorted_ids);
    assert!(first.duplicate_ids().is_empty());
    assert!(first.is_consistent());
}

#[test]
fn every_node_and_edge_kind_has_a_registry_entry() {
    let report = SemanticCoverageRegistry::audit();

    for kind in semantic_coverage_node_kinds() {
        let capability = report
            .capability(SemanticCoverageCapabilityId::SemanticNode(kind))
            .expect("every node kind must have coverage");
        assert_eq!(
            capability.category(),
            SemanticCoverageCategory::SemanticNode
        );
        assert_eq!(capability.related_node_kind(), Some(kind));
    }

    for kind in semantic_coverage_edge_kinds() {
        let capability = report
            .capability(SemanticCoverageCapabilityId::SemanticEdge(kind))
            .expect("every edge kind must have coverage");
        assert_eq!(
            capability.category(),
            SemanticCoverageCategory::SemanticEdge
        );
        assert_eq!(capability.related_edge_kind(), Some(kind));
    }
}

#[test]
fn opens_has_stable_supported_graph_and_impact_coverage() {
    let report = SemanticCoverageRegistry::audit();
    let edge = report
        .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Opens))
        .expect("Opens edge coverage must exist");
    let impact = report
        .capability(SemanticCoverageCapabilityId::ImpactPropagation(
            EdgeKind::Opens,
        ))
        .expect("Opens impact coverage must exist");

    assert_eq!(edge.stable_id(), "semantic_edge.opens");
    assert_eq!(edge.status(), SemanticCoverageStatus::Supported);
    assert_eq!(edge.related_edge_kind(), Some(EdgeKind::Opens));
    assert_eq!(impact.stable_id(), "impact_propagation.opens");
    assert_eq!(impact.status(), SemanticCoverageStatus::Supported);
    assert!(
        impact
            .evidence()
            .contains(&SemanticCoverageEvidence::ImpactPropagationExists)
    );
    assert!(edge.missing_evidence().is_empty());
    assert!(impact.missing_evidence().is_empty());
}

#[test]
fn event_subscription_and_triggers_have_graph_model_coverage_without_impact() {
    let report = SemanticCoverageRegistry::audit();
    let node = report
        .capability(SemanticCoverageCapabilityId::SemanticNode(
            NodeKind::Metadata(MetadataKind::EventSubscription),
        ))
        .expect("Event Subscription node coverage must exist");
    let edge = report
        .capability(SemanticCoverageCapabilityId::SemanticEdge(
            EdgeKind::Triggers,
        ))
        .expect("Triggers edge coverage must exist");
    let impact = report
        .capability(SemanticCoverageCapabilityId::ImpactPropagation(
            EdgeKind::Triggers,
        ))
        .expect("Triggers impact coverage must exist");

    assert_eq!(
        node.stable_id(),
        "semantic_node.metadata.event_subscription"
    );
    assert_eq!(node.status(), SemanticCoverageStatus::Supported);
    assert_eq!(edge.stable_id(), "semantic_edge.triggers");
    assert_eq!(edge.status(), SemanticCoverageStatus::Supported);
    assert_eq!(impact.stable_id(), "impact_propagation.triggers");
    assert_eq!(impact.status(), SemanticCoverageStatus::NotApplicable);
    assert!(impact.evidence().is_empty());
}

#[test]
fn access_right_node_has_graph_domain_coverage_entry() {
    let report = SemanticCoverageRegistry::audit();
    let capability = report
        .capability(SemanticCoverageCapabilityId::SemanticNode(
            NodeKind::AccessRight,
        ))
        .expect("access right node coverage must exist");

    assert_eq!(capability.stable_id(), "semantic_node.access_right");
    assert_eq!(
        capability.category(),
        SemanticCoverageCategory::SemanticNode
    );
    assert_eq!(capability.related_node_kind(), Some(NodeKind::AccessRight));
}

#[test]
fn member_nodes_have_complete_graph_payload_evidence() {
    let report = SemanticCoverageRegistry::audit();

    for kind in [NodeKind::Attribute, NodeKind::TabularSection] {
        let capability = report
            .capability(SemanticCoverageCapabilityId::SemanticNode(kind))
            .expect("member node coverage must exist");

        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::SemanticPayloadPreserved)
        );
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            [
                "oneagent_graph::node::tests::accepts_member_payload_for_attribute_and_tabular_section"
            ]
        );
        assert!(
            report
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
    }
}

#[test]
fn reference_request_provenance_has_complete_graph_domain_evidence() {
    let report = SemanticCoverageRegistry::audit();
    let capability = report
        .capability(SemanticCoverageCapabilityId::ProvenanceSource(
            SemanticProvenanceCapability::ReferenceRequest,
        ))
        .expect("reference request provenance coverage must exist");

    assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
    assert!(capability.missing_evidence().is_empty());
    assert!(capability.limitations().is_empty());
    assert_eq!(
        capability.representative_tests(),
        &["oneagent_graph::reference_request_build"]
    );
    for evidence in [
        SemanticCoverageEvidence::Modeled,
        SemanticCoverageEvidence::ProvenanceAttached,
        SemanticCoverageEvidence::PositiveTestExists,
    ] {
        assert!(capability.evidence().contains(&evidence));
    }
}

#[test]
fn status_contract_matches_required_evidence() {
    let report = SemanticCoverageRegistry::audit();

    for capability in report.capabilities() {
        match capability.status() {
            SemanticCoverageStatus::Supported => {
                assert!(capability.missing_evidence().is_empty());
            }
            SemanticCoverageStatus::PartiallySupported => {
                assert!(!capability.evidence().is_empty());
                assert!(!capability.missing_evidence().is_empty());
            }
            SemanticCoverageStatus::DeclaredOnly => {
                assert!(
                    capability
                        .evidence()
                        .contains(&SemanticCoverageEvidence::NodeKindDeclared)
                        || capability
                            .evidence()
                            .contains(&SemanticCoverageEvidence::EdgeKindDeclared)
                );
            }
            SemanticCoverageStatus::Unsupported => {
                assert!(
                    !capability
                        .evidence()
                        .contains(&SemanticCoverageEvidence::NodeEmitted)
                        && !capability
                            .evidence()
                            .contains(&SemanticCoverageEvidence::SemanticEdgeEmitted)
                );
            }
            SemanticCoverageStatus::NotApplicable => {
                assert!(capability.required_evidence().is_empty());
            }
        }
    }
}

#[test]
fn capability_title_does_not_define_identity() {
    let first = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads),
        "First title",
        SemanticCoverageStatus::DeclaredOnly,
        [SemanticCoverageEvidence::EdgeKindDeclared],
        [SemanticCoverageEvidence::SemanticEdgeEmitted],
    );
    let second = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads),
        "Second title",
        SemanticCoverageStatus::DeclaredOnly,
        [SemanticCoverageEvidence::EdgeKindDeclared],
        [SemanticCoverageEvidence::SemanticEdgeEmitted],
    );

    assert_eq!(first.stable_id(), second.stable_id());
    assert_ne!(first.title(), second.title());
}

#[test]
fn capability_declaration_order_does_not_change_report() {
    let first = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads),
        "Reads edge",
        SemanticCoverageStatus::Supported,
        [SemanticCoverageEvidence::EdgeKindDeclared],
        [SemanticCoverageEvidence::EdgeKindDeclared],
    )
    .with_edge_kind(EdgeKind::Reads);
    let second = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes),
        "Writes edge",
        SemanticCoverageStatus::Supported,
        [SemanticCoverageEvidence::EdgeKindDeclared],
        [SemanticCoverageEvidence::EdgeKindDeclared],
    )
    .with_edge_kind(EdgeKind::Writes);

    assert_eq!(
        SemanticCoverageReport::from_capabilities([first.clone(), second.clone()]),
        SemanticCoverageReport::from_capabilities([second, first])
    );
}

#[test]
fn report_summary_and_gap_list_match_capabilities() {
    let report = SemanticCoverageRegistry::audit();
    let status_total = report.summary().by_status().values().sum::<usize>();
    let category_total = report.summary().by_category().values().sum::<usize>();
    let gap_total = report.summary().by_gap_priority().values().sum::<usize>();

    assert_eq!(report.summary().total(), report.capabilities().len());
    assert_eq!(status_total, report.capabilities().len());
    assert_eq!(category_total, report.capabilities().len());
    assert_eq!(gap_total, report.gaps().len());
    assert_eq!(
        report.declared_but_unused().len(),
        report
            .capabilities()
            .iter()
            .filter(|capability| capability.status() == SemanticCoverageStatus::DeclaredOnly)
            .count()
    );
}

#[test]
fn gaps_are_stable_unique_and_deterministically_sorted() {
    let report = SemanticCoverageRegistry::audit();
    let keys = report
        .gaps()
        .iter()
        .map(|gap| (gap.priority(), gap.category(), gap.stable_id()))
        .collect::<Vec<_>>();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    let unique_ids = report
        .gaps()
        .iter()
        .map(oneagent_graph::SemanticCoverageGap::stable_id)
        .collect::<BTreeSet<_>>();

    assert_eq!(keys, sorted_keys);
    assert_eq!(unique_ids.len(), report.gaps().len());
    assert!(
        report
            .gaps()
            .iter()
            .all(|gap| gap.priority() >= SemanticCoverageGapPriority::Critical)
    );
}

#[test]
fn emitted_fact_without_required_provenance_is_critical() {
    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes),
        "Writes edge",
        SemanticCoverageStatus::PartiallySupported,
        [
            SemanticCoverageEvidence::EdgeKindDeclared,
            SemanticCoverageEvidence::SemanticEdgeEmitted,
        ],
        [
            SemanticCoverageEvidence::EdgeKindDeclared,
            SemanticCoverageEvidence::SemanticEdgeEmitted,
            SemanticCoverageEvidence::ProvenanceAttached,
        ],
    );
    let report = SemanticCoverageReport::from_capabilities([capability]);

    assert_eq!(report.gaps().len(), 1);
    assert_eq!(
        report.gaps()[0].priority(),
        SemanticCoverageGapPriority::Critical
    );
}

#[test]
fn empty_graph_has_empty_observed_coverage() {
    let observed = SemanticObservedCoverage::for_graph(&SemanticGraph::new());

    assert!(observed.nodes().is_empty());
    assert!(observed.edges().is_empty());
    assert_eq!(observed.total_nodes(), 0);
    assert_eq!(observed.total_edges(), 0);
}

#[test]
fn observed_coverage_counts_kinds_and_provenance() {
    let observed = observed_fixture(false);
    let document = observed.nodes()[&NodeKind::Metadata(MetadataKind::Document)];
    let attribute = observed.nodes()[&NodeKind::Attribute];
    let contains = observed.edges()[&EdgeKind::Contains];

    assert_eq!(observed.total_nodes(), 2);
    assert_eq!(observed.total_edges(), 1);
    assert_eq!(document.total(), 1);
    assert_eq!(document.with_provenance(), 1);
    assert_eq!(attribute.without_provenance(), 0);
    assert_eq!(contains.with_provenance(), 1);
    let registry = SemanticCoverageRegistry::audit();
    assert!(observed.unregistered_node_kinds(&registry).is_empty());
    assert!(observed.unregistered_edge_kinds(&registry).is_empty());
}

#[test]
fn observed_coverage_does_not_depend_on_insertion_order() {
    assert_eq!(observed_fixture(false), observed_fixture(true));
}

#[test]
fn unknown_occurrence_is_observed_without_changing_model_support() {
    let unknown_id = id("unknown.node");
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new(
        unknown_id,
        name("Unknown"),
        NodeKind::Unknown,
    ));

    let observed = SemanticObservedCoverage::for_graph(&graph);
    let static_report = SemanticCoverageRegistry::audit();

    assert_eq!(observed.nodes()[&NodeKind::Unknown].total(), 1);
    assert_eq!(
        static_report
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Unknown,
            ))
            .expect("unknown kind must be registered")
            .status(),
        SemanticCoverageStatus::Supported
    );
}
