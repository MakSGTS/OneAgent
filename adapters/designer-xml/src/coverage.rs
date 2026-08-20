//! Designer XML adapter Semantic Coverage Audit.

use oneagent_graph::{
    EdgeKind, NodeKind, SemanticCoverageCapability, SemanticCoverageCapabilityId,
    SemanticCoverageEvidence, SemanticCoverageReport, SemanticCoverageStatus, SemanticGraph,
    SemanticGraphReport, SemanticGraphValidationResult, SemanticObservedCoverage,
    SemanticProvenanceCapability, SemanticQueryCapability,
};
use oneagent_metadata::MetadataKind;

use crate::metadata_object::ACCEPTED_FAMILIES;

const REPRESENTATIVE_TEST: &str =
    "oneagent_designer_xml::conformance::paired_first_slice_is_non_empty_and_equal";

/// Read-only Designer XML pipeline capability registry.
#[derive(Debug, Clone, Copy)]
pub struct DesignerXmlSemanticCoverageRegistry;

impl DesignerXmlSemanticCoverageRegistry {
    /// Audits the accepted ADR-0036 discovery, parsing, and contribution slice.
    #[must_use]
    pub fn audit() -> SemanticCoverageReport {
        SemanticCoverageReport::from_capabilities(designer_capabilities())
    }
}

/// Static Designer support, observed facts, report, and validation for one graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerXmlSemanticCoverageReport {
    designer_pipeline: SemanticCoverageReport,
    observed: SemanticObservedCoverage,
    graph_report: SemanticGraphReport,
    validation: SemanticGraphValidationResult,
}

impl DesignerXmlSemanticCoverageReport {
    /// Audits a completed Designer XML graph without changing it.
    #[must_use]
    pub fn for_graph(graph: &SemanticGraph) -> Self {
        Self {
            designer_pipeline: DesignerXmlSemanticCoverageRegistry::audit(),
            observed: SemanticObservedCoverage::for_graph(graph),
            graph_report: graph.report(),
            validation: graph.validate(),
        }
    }

    /// Returns static Designer XML pipeline coverage.
    #[must_use]
    pub const fn designer_pipeline(&self) -> &SemanticCoverageReport {
        &self.designer_pipeline
    }

    /// Returns facts observed in this graph snapshot.
    #[must_use]
    pub const fn observed(&self) -> &SemanticObservedCoverage {
        &self.observed
    }

    /// Returns the deterministic graph report.
    #[must_use]
    pub const fn graph_report(&self) -> &SemanticGraphReport {
        &self.graph_report
    }

    /// Returns graph validation for the audited snapshot.
    #[must_use]
    pub const fn validation(&self) -> &SemanticGraphValidationResult {
        &self.validation
    }
}

fn designer_capabilities() -> Vec<SemanticCoverageCapability> {
    let mut capabilities = Vec::new();
    capabilities.push(metadata_capability(MetadataKind::Configuration));
    capabilities.extend(
        ACCEPTED_FAMILIES
            .into_iter()
            .map(|family| metadata_capability(family.kind)),
    );
    capabilities.push(deferred_calculation_register());
    capabilities.push(not_applicable_metadata(
        MetadataKind::Form,
        "Designer common forms are top-level MetadataKind::CommonForm; nested managed forms are outside ADR-0036",
    ));
    capabilities.push(not_applicable_metadata(
        MetadataKind::Unknown,
        "Unknown and deferred Designer artifacts are ignored without placeholder metadata facts",
    ));
    capabilities.extend(
        [NodeKind::Module, NodeKind::Procedure, NodeKind::Function]
            .into_iter()
            .map(node_capability),
    );
    capabilities.extend(
        ACCEPTED_FAMILIES
            .into_iter()
            .map(|family| ownership_capability(NodeKind::Metadata(family.kind))),
    );
    capabilities.extend(
        [NodeKind::Module, NodeKind::Procedure, NodeKind::Function]
            .into_iter()
            .map(ownership_capability),
    );
    capabilities.push(contains_capability());
    capabilities.extend(
        [
            SemanticProvenanceCapability::MetadataObjectNode,
            SemanticProvenanceCapability::ModuleNode,
            SemanticProvenanceCapability::SymbolNode,
            SemanticProvenanceCapability::OwnershipEdge,
        ]
        .into_iter()
        .map(provenance_capability),
    );
    capabilities.extend(
        [
            SemanticQueryCapability::NodeLookup,
            SemanticQueryCapability::NameAndKindLookup,
            SemanticQueryCapability::OwnershipNavigation,
        ]
        .into_iter()
        .map(query_capability),
    );
    capabilities
}

fn metadata_capability(kind: MetadataKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::Discovered,
        Evidence::Parsed,
        Evidence::Modeled,
        Evidence::NodeKindDeclared,
        Evidence::NodeEmitted,
        Evidence::StableIdentityAssigned,
        Evidence::SemanticPayloadPreserved,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataEntity(kind),
        format!("{} Designer XML metadata entity", kind.as_str()),
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind))
    .with_representative_test(REPRESENTATIVE_TEST)
}

fn deferred_calculation_register() -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataEntity(MetadataKind::CalculationRegister),
        "calculation_register Designer XML metadata entity",
        SemanticCoverageStatus::Unsupported,
        [Evidence::Modeled, Evidence::NodeKindDeclared],
        [
            Evidence::Discovered,
            Evidence::Parsed,
            Evidence::NodeEmitted,
            Evidence::StableIdentityAssigned,
            Evidence::ProvenanceAttached,
            Evidence::PositiveTestExists,
            Evidence::IntegrationTestExists,
        ],
    )
    .with_metadata_kind(MetadataKind::CalculationRegister)
    .with_node_kind(NodeKind::Metadata(MetadataKind::CalculationRegister))
    .with_limitation(
        "The paired corpus has no direct CalculationRegisters artifact proving the Designer root/path contract",
    )
}

fn not_applicable_metadata(kind: MetadataKind, note: &'static str) -> SemanticCoverageCapability {
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataEntity(kind),
        format!("{} Designer XML metadata entity", kind.as_str()),
        SemanticCoverageStatus::NotApplicable,
        [],
        [],
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind))
    .with_note(note)
}

fn node_capability(kind: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::Parsed,
        Evidence::Modeled,
        Evidence::NodeKindDeclared,
        Evidence::NodeEmitted,
        Evidence::StableIdentityAssigned,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticNode(kind),
        format!("{kind:?} Designer XML semantic node"),
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_node_kind(kind)
    .with_representative_test(REPRESENTATIVE_TEST)
}

fn ownership_capability(child: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::OwnerResolved,
        Evidence::OwnershipEdgeEmitted,
        Evidence::StableIdentityAssigned,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::OwnershipRelation(child),
        format!("Designer XML ownership of {child:?}"),
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_node_kind(child)
    .with_edge_kind(EdgeKind::Contains)
    .with_representative_test(REPRESENTATIVE_TEST)
}

fn contains_capability() -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::EdgeKindDeclared,
        Evidence::SemanticEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Contains),
        "Designer XML Contains contribution",
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_edge_kind(EdgeKind::Contains)
    .with_representative_test(REPRESENTATIVE_TEST)
}

fn provenance_capability(kind: SemanticProvenanceCapability) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::ProvenanceSource(kind),
        format!("{kind:?} Designer XML provenance"),
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_representative_test(REPRESENTATIVE_TEST)
}

fn query_capability(kind: SemanticQueryCapability) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let evidence = [
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::QueryCapability(kind),
        format!("{kind:?} over Designer XML facts"),
        SemanticCoverageStatus::Supported,
        evidence,
        evidence,
    )
    .with_representative_test(REPRESENTATIVE_TEST)
}

#[cfg(test)]
mod tests {
    use oneagent_graph::{SemanticCoverageCapabilityId, SemanticCoverageStatus};
    use oneagent_metadata::MetadataKind;

    use super::DesignerXmlSemanticCoverageRegistry;

    #[test]
    fn registry_is_deterministic_consistent_and_truthful() {
        let first = DesignerXmlSemanticCoverageRegistry::audit();
        let second = DesignerXmlSemanticCoverageRegistry::audit();

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(first.summary().total(), 58);
        assert_eq!(
            first.summary().by_status()[&SemanticCoverageStatus::Supported],
            55
        );
        assert_eq!(
            first.summary().by_status()[&SemanticCoverageStatus::Unsupported],
            1
        );
        assert_eq!(
            first.summary().by_status()[&SemanticCoverageStatus::NotApplicable],
            2
        );
        assert_eq!(first.gaps().len(), 1);
        assert_eq!(
            first.gaps()[0].capability_id(),
            SemanticCoverageCapabilityId::MetadataEntity(MetadataKind::CalculationRegister)
        );
    }
}
