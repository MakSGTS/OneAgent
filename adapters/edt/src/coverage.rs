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

    let not_applicable_note = match kind {
        MetadataKind::Unknown => Some(
            "MetadataKind::Unknown is a fallback model marker; EDT discovery ignores unsupported directories instead of producing unknown metadata entities",
        ),
        MetadataKind::Form => Some(
            "EDT forms are subordinate metadata objects represented by NodeKind::Form; top-level common forms use MetadataKind::CommonForm",
        ),
        _ => None,
    };
    if let Some(note) = not_applicable_note {
        return not_applicable_metadata_capability(kind, note);
    }

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
                capability.with_representative_test(match kind {
                    MetadataKind::Command => {
                        "oneagent_edt::graph_tests::discovers_top_level_common_command_as_metadata_entity"
                    }
                    MetadataKind::Template => {
                        "oneagent_edt::graph_tests::discovers_top_level_common_template_as_metadata_entity"
                    }
                    _ => {
                        "oneagent_edt::graph_tests::builds_graph_with_configuration_and_metadata_objects"
                    }
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

fn not_applicable_metadata_capability(
    kind: MetadataKind,
    note: &'static str,
) -> SemanticCoverageCapability {
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataEntity(kind),
        format!("{} EDT metadata entity", kind.as_str()),
        SemanticCoverageStatus::NotApplicable,
        [],
        [],
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind))
    .with_note(note)
}

fn edt_node_capability(kind: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    if kind == NodeKind::Metadata(MetadataKind::Unknown) {
        return not_applicable_node_capability(
            kind,
            "MetadataKind::Unknown is a fallback model marker; EDT has no discovery, parsing or graph emission contract for unknown metadata object nodes",
        )
        .with_metadata_kind(MetadataKind::Unknown);
    }

    if kind == NodeKind::Metadata(MetadataKind::Form) {
        return not_applicable_node_capability(kind, "The EDT adapter emits subordinate forms as NodeKind::Form and common forms as NodeKind::Metadata(MetadataKind::CommonForm)")
        .with_metadata_kind(MetadataKind::Form)
        .with_note(
            "The EDT adapter emits subordinate forms as NodeKind::Form and common forms as NodeKind::Metadata(MetadataKind::CommonForm)",
        );
    }

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

fn not_applicable_node_capability(
    kind: NodeKind,
    note: &'static str,
) -> SemanticCoverageCapability {
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticNode(kind),
        format!("EDT {} node contribution", node_title(kind)),
        SemanticCoverageStatus::NotApplicable,
        [],
        [],
    )
    .with_node_kind(kind)
    .with_note(note)
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
        MetadataKind::Template,
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
        | NodeKind::Query
        | NodeKind::Form
        | NodeKind::Command
        | NodeKind::Attribute
        | NodeKind::TabularSection
        | NodeKind::Dimension
        | NodeKind::Resource
        | NodeKind::Measure
        | NodeKind::Role => true,
        NodeKind::StandardAttribute | NodeKind::Subsystem | NodeKind::Unknown => false,
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
        EdgeKind, NodeKind, SemanticCoverageCapabilityId, SemanticCoverageEvidence,
        SemanticCoverageGapPriority, SemanticCoverageStatus, SemanticReferenceCapability,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeSet;
    use std::fs;

    use super::EdtSemanticCoverageRegistry;
    use crate::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};

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
            SemanticCoverageStatus::PartiallySupported
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
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
    }

    #[test]
    fn generic_top_level_form_capabilities_are_not_applicable_to_edt() {
        let report = EdtSemanticCoverageRegistry::audit();
        let entity = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Form,
            ))
            .expect("form entity coverage must exist");
        let node = report
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Metadata(MetadataKind::Form),
            ))
            .expect("form metadata node coverage must exist");

        for capability in [entity, node] {
            assert_eq!(capability.status(), SemanticCoverageStatus::NotApplicable);
            assert!(capability.evidence().is_empty());
            assert!(capability.required_evidence().is_empty());
            assert!(capability.missing_evidence().is_empty());
            assert!(!capability.notes().is_empty());
            assert!(
                report
                    .gaps()
                    .iter()
                    .all(|gap| gap.capability_id() != capability.id())
            );
        }

        let command = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Command,
            ))
            .expect("command coverage must exist");
        assert_eq!(command.status(), SemanticCoverageStatus::PartiallySupported);
        assert_eq!(
            command.missing_evidence(),
            BTreeSet::from([SemanticCoverageEvidence::SemanticPayloadPreserved])
        );

        let next_high_gap = report
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .into_iter()
            .next()
            .expect("a High gap must remain");
        assert_eq!(
            next_high_gap.capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
    }

    #[test]
    fn common_template_discovery_closes_the_selected_high_gap() {
        let report = EdtSemanticCoverageRegistry::audit();
        let capability = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Template,
            ))
            .expect("template coverage must exist");
        let node = report
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Metadata(MetadataKind::Template),
            ))
            .expect("template node coverage must exist");

        assert_eq!(
            capability.status(),
            SemanticCoverageStatus::PartiallySupported
        );
        for evidence in [
            SemanticCoverageEvidence::Discovered,
            SemanticCoverageEvidence::Parsed,
            SemanticCoverageEvidence::NodeEmitted,
            SemanticCoverageEvidence::StableIdentityAssigned,
            SemanticCoverageEvidence::ProvenanceAttached,
            SemanticCoverageEvidence::PositiveTestExists,
            SemanticCoverageEvidence::IntegrationTestExists,
        ] {
            assert!(capability.evidence().contains(&evidence));
        }
        assert_eq!(
            capability.missing_evidence(),
            BTreeSet::from([SemanticCoverageEvidence::SemanticPayloadPreserved])
        );
        assert_eq!(node.status(), SemanticCoverageStatus::Supported);
        assert!(node.missing_evidence().is_empty());

        let template_gap = report
            .gaps()
            .iter()
            .find(|gap| gap.capability_id() == capability.id())
            .expect("payload completion must remain visible");
        assert_eq!(template_gap.priority(), SemanticCoverageGapPriority::Medium);

        let form = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Form,
            ))
            .expect("form coverage must exist");
        assert_eq!(form.status(), SemanticCoverageStatus::NotApplicable);

        let next_high_gap = report
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .into_iter()
            .next()
            .expect("a High gap must remain");
        assert_eq!(
            next_high_gap.capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
    }

    #[test]
    fn unknown_metadata_entity_is_not_applicable_to_edt() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Unknown,
            ))
            .expect("unknown metadata capability must remain registered");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::NotApplicable);
        assert!(capability.evidence().is_empty());
        assert!(capability.required_evidence().is_empty());
        assert!(capability.missing_evidence().is_empty());
        assert!(!capability.notes().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );

        let document = first
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Document,
            ))
            .expect("ordinary metadata capability must remain registered");
        assert_eq!(
            document.status(),
            SemanticCoverageStatus::PartiallySupported
        );
        assert!(
            document
                .required_evidence()
                .contains(&SemanticCoverageEvidence::Discovered)
        );

        let high_gaps = first.gaps_by_priority(SemanticCoverageGapPriority::High);
        let actual_high_ids = high_gaps
            .iter()
            .map(|gap| gap.capability_id())
            .collect::<BTreeSet<_>>();
        let expected_high_ids = BTreeSet::from([
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::Subsystem),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::Unknown),
            SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::Measure),
            SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::StandardAttribute),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::DependsOn),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Extends),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Grants),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Includes),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes),
        ]);

        assert_eq!(actual_high_ids, expected_high_ids);
        assert_eq!(high_gaps.len(), 11);
        assert_eq!(
            high_gaps[0].capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            44
        );
    }

    #[test]
    fn unknown_metadata_node_is_not_applicable_to_edt() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Metadata(MetadataKind::Unknown),
            ))
            .expect("unknown metadata node coverage must remain registered");
        let metadata_entity = first
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Unknown,
            ))
            .expect("unknown metadata entity coverage must remain registered");
        let flat_unknown = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Unknown,
            ))
            .expect("flat unknown node coverage must remain registered");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::NotApplicable);
        assert!(capability.evidence().is_empty());
        assert!(capability.required_evidence().is_empty());
        assert!(capability.missing_evidence().is_empty());
        assert_eq!(
            capability.source_metadata_kind(),
            Some(MetadataKind::Unknown)
        );
        assert_eq!(
            capability.related_node_kind(),
            Some(NodeKind::Metadata(MetadataKind::Unknown))
        );
        assert!(!capability.notes().is_empty());

        assert_eq!(
            metadata_entity.status(),
            SemanticCoverageStatus::NotApplicable
        );
        assert_eq!(flat_unknown.status(), SemanticCoverageStatus::Unsupported);
        assert!(
            flat_unknown
                .missing_evidence()
                .contains(&SemanticCoverageEvidence::NodeEmitted)
        );

        let high_gaps = first.gaps_by_priority(SemanticCoverageGapPriority::High);
        let actual_high_ids = high_gaps
            .iter()
            .map(|gap| gap.capability_id())
            .collect::<BTreeSet<_>>();
        let expected_high_ids = BTreeSet::from([
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::Subsystem),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::Unknown),
            SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::Measure),
            SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::StandardAttribute),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::DependsOn),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Extends),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Grants),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Includes),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads),
            SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes),
        ]);

        assert_eq!(actual_high_ids, expected_high_ids);
        assert!(
            high_gaps
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert_eq!(high_gaps.len(), 11);
        assert_eq!(
            high_gaps[0].capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            44
        );
    }

    #[test]
    fn unsupported_edt_directory_does_not_create_unknown_metadata_node() {
        let root = tempfile::tempdir().expect("temporary EDT project must be created");
        let configuration_directory = root.path().join("src/Configuration");
        fs::create_dir_all(&configuration_directory)
            .expect("configuration directory must be created");
        fs::write(
            configuration_directory.join("Configuration.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="11111111-2222-3333-4444-555555555555">
    <name>Demo</name>
</mdclass:Configuration>
"#,
        )
        .expect("configuration descriptor must be created");

        let unknown_directory = root.path().join("src/UnknownMetadata/FutureObject");
        fs::create_dir_all(&unknown_directory).expect("unknown directory must be created");
        fs::write(
            unknown_directory.join("FutureObject.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:FutureObject
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>FutureObject</name>
</mdclass:FutureObject>
"#,
        )
        .expect("unknown descriptor must be created");

        let build = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("unsupported EDT directory must be ignored");
        let graph = build.graph();

        assert!(build.diagnostics().is_empty());
        assert!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
                .is_empty()
        );
        assert!(graph.nodes_by_kind(NodeKind::Unknown).is_empty());
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration))
                .len(),
            1
        );
    }

    #[test]
    fn role_node_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(NodeKind::Role))
            .expect("role node coverage must exist");
        let standard_attribute = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::StandardAttribute,
            ))
            .expect("standard attribute coverage must remain registered");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert_eq!(
            standard_attribute.status(),
            SemanticCoverageStatus::Unsupported
        );
        assert!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .iter()
                .any(|gap| gap.capability_id() == standard_attribute.id())
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .len(),
            11
        );
        assert_eq!(
            first.gaps_by_priority(SemanticCoverageGapPriority::High)[0].capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            44
        );
    }

    #[test]
    fn query_node_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(NodeKind::Query))
            .expect("query node coverage must exist");
        let reads = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads))
            .expect("reads edge coverage must remain registered");
        let writes = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes))
            .expect("writes edge coverage must remain registered");
        let depends_on = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(
                EdgeKind::DependsOn,
            ))
            .expect("depends_on edge coverage must remain registered");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );

        for edge_capability in [reads, writes, depends_on] {
            assert_eq!(
                edge_capability.status(),
                SemanticCoverageStatus::DeclaredOnly
            );
            assert!(
                first
                    .gaps_by_priority(SemanticCoverageGapPriority::High)
                    .iter()
                    .any(|gap| gap.capability_id() == edge_capability.id())
            );
        }
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .len(),
            11
        );
        assert_eq!(
            first.gaps_by_priority(SemanticCoverageGapPriority::High)[0].capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            44
        );
    }

    #[test]
    fn measure_node_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Measure,
            ))
            .expect("measure node coverage must exist");
        let ownership = first
            .capability(SemanticCoverageCapabilityId::OwnershipRelation(
                NodeKind::Measure,
            ))
            .expect("measure ownership coverage must exist");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );

        assert_eq!(ownership.status(), SemanticCoverageStatus::Unsupported);
        assert!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .iter()
                .any(|gap| gap.capability_id() == ownership.id())
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .len(),
            11
        );
        assert_eq!(
            first.gaps_by_priority(SemanticCoverageGapPriority::High)[0].capability_id(),
            SemanticCoverageCapabilityId::SemanticNode(NodeKind::StandardAttribute)
        );
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            44
        );
    }
}
