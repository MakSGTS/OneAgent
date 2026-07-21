//! EDT-specific Semantic Coverage Audit.
//!
//! The adapter matrix describes what the EDT loading pipeline can discover,
//! parse and contribute. The composite report keeps this static support data
//! separate from facts observed in one concrete graph build.

use std::collections::BTreeSet;

use oneagent_graph::{
    EdgeKind, NodeKind, SemanticCoverageCapability, SemanticCoverageCapabilityId,
    SemanticCoverageEvidence, SemanticCoverageRegistry, SemanticCoverageReport,
    SemanticCoverageStatus, SemanticGraphReport, SemanticGraphValidationResult,
    SemanticObservedCoverage, SemanticProvenanceCapability, SemanticReferenceCapability,
    semantic_coverage_edge_kinds, semantic_coverage_node_kinds,
};
use oneagent_metadata::MetadataKind;

use crate::{EdtSemanticGraphBuildResult, supported_metadata_directories};

/// Read-only EDT pipeline capability registry.
#[derive(Debug, Clone, Copy)]
pub struct EdtSemanticCoverageRegistry;

impl EdtSemanticCoverageRegistry {
    /// Audits static EDT discovery, parsing and graph contribution capabilities.
    #[must_use]
    pub fn audit() -> SemanticCoverageReport {
        SemanticCoverageReport::from_capabilities(edt_capabilities())
    }
}

/// Combined graph-domain, EDT-specific and observed coverage for one build.
///
/// Static support and observed occurrence remain separate: an absent kind in a
/// particular project does not become unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtSemanticCoverageReport {
    graph_domain: SemanticCoverageReport,
    edt_pipeline: SemanticCoverageReport,
    observed: SemanticObservedCoverage,
    build_report: SemanticGraphReport,
    validation: SemanticGraphValidationResult,
}

impl EdtSemanticCoverageReport {
    /// Builds an owned deterministic audit for an existing EDT graph build.
    #[must_use]
    pub fn for_build_result(result: &EdtSemanticGraphBuildResult) -> Self {
        Self {
            graph_domain: SemanticCoverageRegistry::audit(),
            edt_pipeline: EdtSemanticCoverageRegistry::audit(),
            observed: SemanticObservedCoverage::for_graph(result.graph()),
            build_report: result.report(),
            validation: result.validate(),
        }
    }

    /// Returns source-independent graph-domain coverage.
    #[must_use]
    pub const fn graph_domain(&self) -> &SemanticCoverageReport {
        &self.graph_domain
    }

    /// Returns EDT discovery and graph contribution coverage.
    #[must_use]
    pub const fn edt_pipeline(&self) -> &SemanticCoverageReport {
        &self.edt_pipeline
    }

    /// Returns facts observed in this graph snapshot.
    #[must_use]
    pub const fn observed(&self) -> &SemanticObservedCoverage {
        &self.observed
    }

    /// Returns existing graph, diagnostics, resolution and provenance metrics.
    #[must_use]
    pub const fn build_report(&self) -> &SemanticGraphReport {
        &self.build_report
    }

    /// Returns validation of the audited build result.
    #[must_use]
    pub const fn validation(&self) -> &SemanticGraphValidationResult {
        &self.validation
    }
}

fn edt_capabilities() -> Vec<SemanticCoverageCapability> {
    let mut capabilities = Vec::new();
    capabilities.extend(all_metadata_kinds().into_iter().map(metadata_capability));
    capabilities.extend(
        semantic_coverage_node_kinds()
            .into_iter()
            .map(edt_node_capability),
    );
    capabilities.extend(
        semantic_coverage_edge_kinds()
            .into_iter()
            .map(edt_edge_capability),
    );
    capabilities.extend(
        ownership_child_kinds()
            .into_iter()
            .map(ownership_capability),
    );
    capabilities.extend(
        metadata_reference_target_kinds()
            .into_iter()
            .map(metadata_reference_capability),
    );
    capabilities.push(bsl_call_reference_capability());
    capabilities.extend(
        provenance_capabilities()
            .into_iter()
            .map(edt_provenance_capability),
    );
    capabilities
}

fn metadata_capability(kind: MetadataKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let supported = kind == MetadataKind::Configuration
        || supported_metadata_directories()
            .values()
            .any(|candidate| *candidate == kind);
    let representative = representative_metadata_kinds().contains(&kind);
    let required = [
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

    let mut evidence = Vec::new();
    if supported {
        evidence.extend([
            Evidence::Discovered,
            Evidence::Parsed,
            Evidence::Modeled,
            Evidence::NodeKindDeclared,
            Evidence::NodeEmitted,
            Evidence::StableIdentityAssigned,
            Evidence::ProvenanceAttached,
        ]);
    } else if kind != MetadataKind::Unknown {
        evidence.extend([Evidence::Modeled, Evidence::NodeKindDeclared]);
    }
    if representative {
        evidence.extend([
            Evidence::PositiveTestExists,
            Evidence::IntegrationTestExists,
        ]);
    }

    let status = if supported {
        SemanticCoverageStatus::PartiallySupported
    } else {
        SemanticCoverageStatus::Unsupported
    };

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataEntity(kind),
        format!("{} EDT metadata entity", kind.as_str()),
        status,
        evidence,
        required,
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind));

    match status {
        SemanticCoverageStatus::PartiallySupported => {
            let capability = capability.with_limitation(if representative {
                "The EDT pipeline emits this metadata kind, but descriptor fields beyond graph identity, name and kind are not preserved as typed semantic payload"
            } else {
                "The generic EDT path emits this metadata kind without a dedicated representative fixture, and descriptor payload is only partially preserved"
            });
            if representative {
                capability.with_representative_test(if kind == MetadataKind::Command {
                    "oneagent_edt::graph_tests::discovers_top_level_common_command_as_metadata_entity"
                } else {
                    "oneagent_edt::graph_tests::builds_graph_with_configuration_and_metadata_objects"
                })
            } else {
                capability
            }
        }
        SemanticCoverageStatus::Unsupported => capability.with_limitation(
            "MetadataKind is declared but the EDT directory registry does not discover this entity",
        ),
        _ => capability,
    }
}

fn edt_node_capability(kind: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let emitted = edt_emits_node_kind(kind);
    let representative = representative_node_kind(kind);
    let required = [
        Evidence::Parsed,
        Evidence::Modeled,
        Evidence::NodeKindDeclared,
        Evidence::NodeEmitted,
        Evidence::StableIdentityAssigned,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    let mut evidence = vec![Evidence::Modeled, Evidence::NodeKindDeclared];
    if emitted {
        evidence.extend([
            Evidence::Parsed,
            Evidence::NodeEmitted,
            Evidence::StableIdentityAssigned,
            Evidence::ProvenanceAttached,
        ]);
    }
    if representative {
        evidence.extend([
            Evidence::PositiveTestExists,
            Evidence::IntegrationTestExists,
        ]);
    }

    let status = if !emitted {
        SemanticCoverageStatus::Unsupported
    } else if representative {
        SemanticCoverageStatus::Supported
    } else {
        SemanticCoverageStatus::PartiallySupported
    };

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticNode(kind),
        format!("EDT {} node contribution", node_title(kind)),
        status,
        evidence,
        required,
    )
    .with_node_kind(kind);

    match status {
        SemanticCoverageStatus::Unsupported => capability.with_limitation(
            "The graph model declares this node kind, but the EDT structure reader does not extract it",
        ),
        SemanticCoverageStatus::PartiallySupported => capability.with_limitation(
            "The EDT pipeline emits this node kind, but no dedicated representative integration fixture verifies it",
        ),
        _ => capability.with_representative_test("oneagent_edt::graph_tests"),
    }
}

fn edt_edge_capability(kind: EdgeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let emitted = matches!(
        kind,
        EdgeKind::Contains | EdgeKind::Calls | EdgeKind::References
    );
    let required = [
        Evidence::EdgeKindDeclared,
        Evidence::SemanticEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::ValidationRuleExists,
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    let evidence = if emitted {
        required.to_vec()
    } else {
        vec![
            Evidence::EdgeKindDeclared,
            Evidence::ValidationRuleExists,
            Evidence::QuerySupportExists,
        ]
    };

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(kind),
        format!("EDT {} edge contribution", edge_title(kind)),
        if emitted {
            SemanticCoverageStatus::Supported
        } else {
            SemanticCoverageStatus::DeclaredOnly
        },
        evidence,
        required,
    )
    .with_edge_kind(kind);

    if emitted {
        capability.with_representative_test("oneagent_edt::graph_tests")
    } else {
        capability.with_limitation("The edge kind is declared but no EDT graph path emits it")
    }
}

fn ownership_capability(child: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let unsupported = matches!(child, NodeKind::StandardAttribute | NodeKind::Measure);
    let partial = child == NodeKind::Attribute;
    let required = [
        Evidence::OwnerResolved,
        Evidence::OwnershipEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::ValidationRuleExists,
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    let mut evidence = if unsupported {
        vec![Evidence::ValidationRuleExists, Evidence::QuerySupportExists]
    } else {
        required.to_vec()
    };
    if partial {
        evidence.retain(|item| *item != Evidence::IntegrationTestExists);
    }

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::OwnershipRelation(child),
        format!("{} ownership relation", node_title(child)),
        if unsupported {
            SemanticCoverageStatus::Unsupported
        } else if partial {
            SemanticCoverageStatus::PartiallySupported
        } else {
            SemanticCoverageStatus::Supported
        },
        evidence,
        required,
    )
    .with_node_kind(child)
    .with_edge_kind(EdgeKind::Contains);

    if unsupported {
        capability.with_limitation(
            "Validation knows the owner rule, but the EDT pipeline emits neither the child nor its ownership edge",
        )
    } else if partial {
        capability.with_limitation(
            "Attributes nested in tabular sections are currently attached to the top-level metadata object instead of the tabular section",
        )
    } else {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::metadata_object_contains_structure_and_modules",
        )
    }
}

fn metadata_reference_capability(kind: MetadataKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let representative = matches!(kind, MetadataKind::Catalog | MetadataKind::Document);
    let required = [
        Evidence::Parsed,
        Evidence::ReferenceExtracted,
        Evidence::ReferenceResolved,
        Evidence::DiagnosticEmitted,
        Evidence::SemanticEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::ResolutionStatisticsRecorded,
        Evidence::PositiveTestExists,
        Evidence::NegativeTestExists,
        Evidence::IntegrationTestExists,
    ];
    let mut evidence = vec![
        Evidence::Parsed,
        Evidence::ReferenceExtracted,
        Evidence::ReferenceResolved,
        Evidence::DiagnosticEmitted,
        Evidence::SemanticEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::ResolutionStatisticsRecorded,
        Evidence::NegativeTestExists,
    ];
    if representative {
        evidence.extend([
            Evidence::PositiveTestExists,
            Evidence::IntegrationTestExists,
        ]);
    }

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataReference(SemanticReferenceCapability::MetadataType(
            kind,
        )),
        format!("{} metadata type reference", kind.as_str()),
        if representative {
            SemanticCoverageStatus::Supported
        } else {
            SemanticCoverageStatus::PartiallySupported
        },
        evidence,
        required,
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind))
    .with_edge_kind(EdgeKind::References);

    if representative {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::resolves_metadata_reference_edges",
        )
    } else {
        capability.with_limitation(
            "Reference prefix mapping is implemented, but this target kind lacks a representative successful integration fixture",
        )
    }
}

fn bsl_call_reference_capability() -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataReference(SemanticReferenceCapability::BslCall),
        "BSL call reference",
        SemanticCoverageStatus::Supported,
        [
            Evidence::Parsed,
            Evidence::ReferenceExtracted,
            Evidence::ReferenceResolved,
            Evidence::DiagnosticEmitted,
            Evidence::SemanticEdgeEmitted,
            Evidence::ProvenanceAttached,
            Evidence::ResolutionStatisticsRecorded,
            Evidence::PositiveTestExists,
            Evidence::NegativeTestExists,
            Evidence::IntegrationTestExists,
        ],
        [
            Evidence::Parsed,
            Evidence::ReferenceExtracted,
            Evidence::ReferenceResolved,
            Evidence::DiagnosticEmitted,
            Evidence::SemanticEdgeEmitted,
            Evidence::ProvenanceAttached,
            Evidence::ResolutionStatisticsRecorded,
            Evidence::PositiveTestExists,
            Evidence::NegativeTestExists,
            Evidence::IntegrationTestExists,
        ],
    )
    .with_edge_kind(EdgeKind::Calls)
    .with_representative_test(
        "oneagent_edt::graph_tests::resolves_cross_module_call_through_production_graph_builder",
    )
    .with_representative_test(
        "oneagent_edt::graph_tests::preserves_unresolved_bsl_calls_as_deterministic_build_diagnostics",
    )
}

fn edt_provenance_capability(kind: SemanticProvenanceCapability) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let partial = kind == SemanticProvenanceCapability::ReferenceRequest;
    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::ProvenanceSource(kind),
        format!("EDT {} provenance", provenance_title(kind)),
        if partial {
            SemanticCoverageStatus::PartiallySupported
        } else {
            SemanticCoverageStatus::Supported
        },
        if partial {
            vec![Evidence::Modeled]
        } else {
            vec![
                Evidence::Modeled,
                Evidence::ProvenanceAttached,
                Evidence::PositiveTestExists,
            ]
        },
        [
            Evidence::Modeled,
            Evidence::ProvenanceAttached,
            Evidence::PositiveTestExists,
        ],
    );

    if partial {
        capability.with_limitation(
            "Pending metadata references preserve source context, but do not carry a public Provenance value until edge or diagnostic creation",
        )
    } else {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::attaches_provenance_to_edt_graph_facts",
        )
    }
}

fn all_metadata_kinds() -> Vec<MetadataKind> {
    semantic_coverage_node_kinds()
        .into_iter()
        .filter_map(|kind| match kind {
            NodeKind::Metadata(kind) => Some(kind),
            _ => None,
        })
        .collect()
}

fn representative_metadata_kinds() -> BTreeSet<MetadataKind> {
    BTreeSet::from([
        MetadataKind::Configuration,
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::CommonModule,
        MetadataKind::AccumulationRegister,
        MetadataKind::Command,
    ])
}

const fn ownership_child_kinds() -> [NodeKind; 11] {
    [
        NodeKind::Module,
        NodeKind::Procedure,
        NodeKind::Function,
        NodeKind::Attribute,
        NodeKind::StandardAttribute,
        NodeKind::TabularSection,
        NodeKind::Form,
        NodeKind::Command,
        NodeKind::Dimension,
        NodeKind::Resource,
        NodeKind::Measure,
    ]
}

const fn metadata_reference_target_kinds() -> [MetadataKind; 9] {
    [
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::Enumeration,
        MetadataKind::InformationRegister,
        MetadataKind::AccumulationRegister,
        MetadataKind::AccountingRegister,
        MetadataKind::CalculationRegister,
        MetadataKind::BusinessProcess,
        MetadataKind::Task,
    ]
}

const fn provenance_capabilities() -> [SemanticProvenanceCapability; 8] {
    [
        SemanticProvenanceCapability::MetadataObjectNode,
        SemanticProvenanceCapability::MetadataChildNode,
        SemanticProvenanceCapability::ModuleNode,
        SemanticProvenanceCapability::SymbolNode,
        SemanticProvenanceCapability::OwnershipEdge,
        SemanticProvenanceCapability::ResolvedReferenceEdge,
        SemanticProvenanceCapability::Diagnostic,
        SemanticProvenanceCapability::ReferenceRequest,
    ]
}

const fn node_title(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Metadata(_) => "metadata object",
        NodeKind::Module => "module",
        NodeKind::Procedure => "procedure",
        NodeKind::Function => "function",
        NodeKind::Query => "query",
        NodeKind::Form => "form",
        NodeKind::Command => "command",
        NodeKind::Attribute => "attribute",
        NodeKind::StandardAttribute => "standard attribute",
        NodeKind::TabularSection => "tabular section",
        NodeKind::Dimension => "dimension",
        NodeKind::Resource => "resource",
        NodeKind::Measure => "measure",
        NodeKind::Role => "role",
        NodeKind::Subsystem => "subsystem",
        NodeKind::Unknown => "unknown",
    }
}

const fn provenance_title(kind: SemanticProvenanceCapability) -> &'static str {
    match kind {
        SemanticProvenanceCapability::MetadataObjectNode => "metadata object node",
        SemanticProvenanceCapability::MetadataChildNode => "metadata child node",
        SemanticProvenanceCapability::ModuleNode => "module node",
        SemanticProvenanceCapability::SymbolNode => "symbol node",
        SemanticProvenanceCapability::OwnershipEdge => "ownership edge",
        SemanticProvenanceCapability::ResolvedReferenceEdge => "resolved reference edge",
        SemanticProvenanceCapability::Diagnostic => "diagnostic",
        SemanticProvenanceCapability::ReferenceRequest => "reference request",
    }
}

fn edt_emits_node_kind(kind: NodeKind) -> bool {
    match kind {
        NodeKind::Metadata(kind) => {
            kind == MetadataKind::Configuration
                || supported_metadata_directories()
                    .values()
                    .any(|candidate| *candidate == kind)
        }
        NodeKind::Module
        | NodeKind::Procedure
        | NodeKind::Function
        | NodeKind::Form
        | NodeKind::Command
        | NodeKind::Attribute
        | NodeKind::TabularSection
        | NodeKind::Dimension
        | NodeKind::Resource => true,
        NodeKind::Query
        | NodeKind::StandardAttribute
        | NodeKind::Measure
        | NodeKind::Role
        | NodeKind::Subsystem
        | NodeKind::Unknown => false,
    }
}

fn representative_node_kind(kind: NodeKind) -> bool {
    match kind {
        NodeKind::Metadata(kind) => representative_metadata_kinds().contains(&kind),
        _ => edt_emits_node_kind(kind),
    }
}

const fn edge_title(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Calls => "calls",
        EdgeKind::References => "references",
        EdgeKind::Reads => "reads",
        EdgeKind::Writes => "writes",
        EdgeKind::Grants => "grants",
        EdgeKind::Includes => "includes",
        EdgeKind::Extends => "extends",
        EdgeKind::DependsOn => "depends on",
    }
}

#[cfg(test)]
mod tests {
    use oneagent_graph::{
        SemanticCoverageCapabilityId, SemanticCoverageEvidence, SemanticCoverageGapPriority,
        SemanticCoverageStatus, SemanticReferenceCapability,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeSet;

    use super::EdtSemanticCoverageRegistry;

    #[test]
    fn registry_is_deterministic_unique_and_consistent() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert!(!first.capabilities().is_empty());
        assert_eq!(first.summary().total(), first.capabilities().len());
    }

    #[test]
    fn registry_distinguishes_supported_partial_and_unsupported_entities() {
        let report = EdtSemanticCoverageRegistry::audit();

        assert_eq!(
            report
                .capability(SemanticCoverageCapabilityId::MetadataEntity(
                    MetadataKind::Document,
                ))
                .expect("document coverage must exist")
                .status(),
            SemanticCoverageStatus::PartiallySupported
        );
        assert_eq!(
            report
                .capability(SemanticCoverageCapabilityId::MetadataEntity(
                    MetadataKind::InformationRegister,
                ))
                .expect("register coverage must exist")
                .status(),
            SemanticCoverageStatus::PartiallySupported
        );
        assert_eq!(
            report
                .capability(SemanticCoverageCapabilityId::MetadataEntity(
                    MetadataKind::Template,
                ))
                .expect("template coverage must exist")
                .status(),
            SemanticCoverageStatus::Unsupported
        );
    }

    #[test]
    fn bsl_call_observability_closes_the_typed_gap() {
        let report = EdtSemanticCoverageRegistry::audit();
        let capability = report
            .capability(SemanticCoverageCapabilityId::MetadataReference(
                SemanticReferenceCapability::BslCall,
            ))
            .expect("BSL call coverage must exist");

        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::DiagnosticEmitted)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::ResolutionStatisticsRecorded)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::NegativeTestExists)
        );
        assert!(capability.missing_evidence().is_empty());
        assert!(
            report
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
    }

    #[test]
    fn common_command_discovery_closes_the_high_gap() {
        let report = EdtSemanticCoverageRegistry::audit();
        let capability = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Command,
            ))
            .expect("command coverage must exist");

        assert_eq!(
            capability.status(),
            SemanticCoverageStatus::PartiallySupported
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::Discovered)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::Parsed)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::NodeEmitted)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::StableIdentityAssigned)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::ProvenanceAttached)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::PositiveTestExists)
        );
        assert!(
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::IntegrationTestExists)
        );
        assert_eq!(
            capability.missing_evidence(),
            BTreeSet::from([SemanticCoverageEvidence::SemanticPayloadPreserved])
        );

        let command_gap = report
            .gaps()
            .iter()
            .find(|gap| gap.capability_id() == capability.id())
            .expect("payload completion must remain visible");
        assert_eq!(command_gap.priority(), SemanticCoverageGapPriority::Medium);

        let next_high_gap = report
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .into_iter()
            .next()
            .expect("a High gap must remain");
        assert_eq!(
            next_high_gap.capability_id(),
            SemanticCoverageCapabilityId::MetadataEntity(MetadataKind::Form)
        );
    }
}
