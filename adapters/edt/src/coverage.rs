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
        edt_coverage_node_kinds()
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
            Evidence::SemanticPayloadPreserved,
            Evidence::ProvenanceAttached,
            Evidence::PositiveTestExists,
            Evidence::IntegrationTestExists,
        ]);
    } else if kind != MetadataKind::Unknown {
        evidence.extend([Evidence::Modeled, Evidence::NodeKindDeclared]);
    }
    let status = if supported {
        SemanticCoverageStatus::Supported
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
        SemanticCoverageStatus::Supported if kind == MetadataKind::EventSubscription => capability
            .with_representative_test(
                "oneagent_edt::event_subscriptions::live_derived_fixture_is_consumer_visible_and_deterministic",
            ),
        SemanticCoverageStatus::Supported => capability.with_representative_test(
            "oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind",
        ),
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

    if kind == NodeKind::Unknown {
        return not_applicable_node_capability(
            kind,
            "NodeKind::Unknown is a graph-domain fallback marker; EDT intentionally ignores unsupported source directories and has no production source, identity or provenance contract for flat unknown semantic nodes",
        );
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
    let member_payload = matches!(kind, NodeKind::Attribute | NodeKind::TabularSection);
    let mut required = vec![
        Evidence::Parsed,
        Evidence::Modeled,
        Evidence::NodeKindDeclared,
        Evidence::NodeEmitted,
        Evidence::StableIdentityAssigned,
        Evidence::ProvenanceAttached,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];
    if member_payload {
        required.push(Evidence::SemanticPayloadPreserved);
    }
    let mut evidence = vec![Evidence::Modeled, Evidence::NodeKindDeclared];
    if emitted {
        evidence.extend([
            Evidence::Parsed,
            Evidence::NodeEmitted,
            Evidence::StableIdentityAssigned,
            Evidence::ProvenanceAttached,
        ]);
        if member_payload {
            evidence.push(Evidence::SemanticPayloadPreserved);
        }
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
        _ => capability.with_representative_test(representative_node_test(kind)),
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
        EdgeKind::Contains
            | EdgeKind::Calls
            | EdgeKind::References
            | EdgeKind::Reads
            | EdgeKind::Writes
            | EdgeKind::Grants
            | EdgeKind::Includes
            | EdgeKind::Extends
            | EdgeKind::DependsOn
            | EdgeKind::Opens
            | EdgeKind::Triggers
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

    if !emitted {
        return capability
            .with_limitation("The edge kind is declared but no EDT graph path emits it");
    }

    let capability = capability.with_representative_test(match kind {
            EdgeKind::DependsOn => {
                "oneagent_edt::graph_tests::resolves_metadata_reference_and_depends_on_edges"
            }
            EdgeKind::Extends => {
                "oneagent_edt::graph_tests::emits_metadata_extends_edges_for_adopted_objects"
            }
            EdgeKind::Grants => {
                "oneagent_edt::grants::grants_real_edt_fixture_emits_scoped_access_rights_with_stable_provenance"
            }
            EdgeKind::Includes => {
                "oneagent_edt::includes::includes_real_edt_fixture_emits_direct_metadata_members_with_stable_provenance"
            }
            EdgeKind::Reads => {
                "oneagent_edt::reads::reads_real_edt_fixture_emits_both_target_kinds_with_stable_query_navigation"
            }
            EdgeKind::Writes => {
                "oneagent_edt::writes::writes_full_builder_emits_canonical_edges_with_query_coverage_and_repeated_build_evidence"
            }
            EdgeKind::Opens => {
                "oneagent_edt::sprint7_evidence::sprint7_repository_fixture_proves_modules_references_and_navigation_end_to_end"
            }
            EdgeKind::Triggers => {
                "oneagent_edt::event_subscriptions::live_derived_fixture_is_consumer_visible_and_deterministic"
            }
            _ => "oneagent_edt::graph_tests",
        });
    match kind {
        EdgeKind::Reads | EdgeKind::DependsOn => capability.with_representative_test(
            "oneagent_edt::sprint8_registers_queries::sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible",
        ),
        EdgeKind::Opens => capability
            .with_representative_test(
                "oneagent_edt::form_navigation::changed_common_form_propagates_impact_through_opens",
            )
            .with_representative_test(
                "oneagent_edt::form_navigation::form_navigation_diff_observes_a_missing_target_becoming_resolved",
            )
            .with_representative_test(
                "oneagent_edt::form_navigation::form_navigation_emits_only_unique_resolved_opens_with_stable_evidence",
            ),
        _ => capability,
    }
}

fn ownership_capability(child: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

    let required = [
        Evidence::OwnerResolved,
        Evidence::OwnershipEdgeEmitted,
        Evidence::ProvenanceAttached,
        Evidence::ValidationRuleExists,
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
        Evidence::IntegrationTestExists,
    ];

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::OwnershipRelation(child),
        format!("{} ownership relation", node_title(child)),
        SemanticCoverageStatus::Supported,
        required,
        required,
    )
    .with_node_kind(child)
    .with_edge_kind(EdgeKind::Contains);

    if matches!(
        child,
        NodeKind::Module
            | NodeKind::Procedure
            | NodeKind::Function
            | NodeKind::Form
            | NodeKind::Command
    ) {
        capability.with_representative_test(
            "oneagent_edt::sprint7_evidence::sprint7_repository_fixture_proves_modules_references_and_navigation_end_to_end",
        )
    } else if matches!(
        child,
        NodeKind::Query
            | NodeKind::DataCompositionSchema
            | NodeKind::DataSet
            | NodeKind::DataCompositionField
    ) {
        capability.with_representative_test(
            "oneagent_edt::report_data_composition::live_derived_fixture_is_typed_consumer_visible_and_deterministic",
        )
    } else if matches!(
        child,
        NodeKind::XdtoType
            | NodeKind::HttpServiceUrlTemplate
            | NodeKind::HttpServiceMethod
            | NodeKind::WebServiceOperation
            | NodeKind::WebServiceParameter
    ) {
        capability.with_representative_test(
            "oneagent_edt::xdto_services::live_derived_fixture_is_consumer_visible_and_deterministic",
        )
    } else if child == NodeKind::Attribute {
        capability.with_representative_test(
            "oneagent_edt::ownership::tabular_section_ownership_fixture_builds_with_immediate_owners",
        )
    } else if child == NodeKind::StandardAttribute {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::emits_document_standard_attribute_ownership_edges_through_production_graph_builder",
        )
    } else if child == NodeKind::Measure {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::emits_accounting_register_measure_with_owner",
        )
    } else {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::metadata_object_contains_structure_and_modules",
        )
    }
}

fn metadata_reference_capability(kind: MetadataKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;

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

    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::MetadataReference(SemanticReferenceCapability::MetadataType(
            kind,
        )),
        format!("{} metadata type reference", kind.as_str()),
        SemanticCoverageStatus::Supported,
        required,
        required,
    )
    .with_metadata_kind(kind)
    .with_node_kind(NodeKind::Metadata(kind))
    .with_edge_kind(EdgeKind::References)
    .with_representative_test(
        "oneagent_edt::graph_tests::resolves_all_mapped_metadata_reference_target_kinds_through_production_builder",
    )
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

    let capability = SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::ProvenanceSource(kind),
        format!("EDT {} provenance", provenance_title(kind)),
        SemanticCoverageStatus::Supported,
        [
            Evidence::Modeled,
            Evidence::ProvenanceAttached,
            Evidence::PositiveTestExists,
        ],
        [
            Evidence::Modeled,
            Evidence::ProvenanceAttached,
            Evidence::PositiveTestExists,
        ],
    );

    if kind == SemanticProvenanceCapability::ReferenceRequest {
        capability
            .with_representative_test(
                "oneagent_edt::graph_tests::resolves_metadata_reference_and_depends_on_edges",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::resolves_all_mapped_metadata_reference_target_kinds_through_production_builder",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::missing_metadata_reference_target_is_reported",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::ambiguous_metadata_reference_target_is_reported",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::incompatible_metadata_reference_target_kind_is_reported",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::production_builder_preserves_explicit_partial_workspace_request",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::duplicate_metadata_type_reference_creates_one_depends_on_edge",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::duplicate_identical_reference_diagnostic_is_deduplicated",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::request_identity_survives_missing_to_resolved_production_diff",
            )
            .with_representative_test(
                "oneagent_edt::graph_tests::production_builder_propagates_explicit_partial_scope_to_query_requests",
            )
            .with_representative_test(
                "oneagent_edt::reads::reads_parser_and_missing_failures_are_typed_counted_and_deterministic",
            )
            .with_representative_test(
                "oneagent_edt::reads::reads_ambiguous_target_is_sorted_counted_and_emits_no_edge",
            )
            .with_representative_test(
                "oneagent_edt::reads::reads_incompatible_target_is_typed_counted_and_emits_no_edge",
            )
            .with_representative_test(
                "oneagent_edt::sprint8_registers_queries::sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible",
            )
    } else {
        capability.with_representative_test(
            "oneagent_edt::graph_tests::attaches_provenance_to_edt_graph_facts",
        )
    }
}

fn all_metadata_kinds() -> Vec<MetadataKind> {
    edt_coverage_node_kinds()
        .into_iter()
        .filter_map(|kind| match kind {
            NodeKind::Metadata(kind) => Some(kind),
            _ => None,
        })
        .collect()
}

fn edt_coverage_node_kinds() -> Vec<NodeKind> {
    semantic_coverage_node_kinds()
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
        MetadataKind::Subsystem,
        MetadataKind::Enumeration,
        MetadataKind::Report,
        MetadataKind::DataProcessor,
        MetadataKind::InformationRegister,
        MetadataKind::AccountingRegister,
        MetadataKind::CalculationRegister,
        MetadataKind::BusinessProcess,
        MetadataKind::Task,
        MetadataKind::Role,
        MetadataKind::CommonForm,
        MetadataKind::HttpService,
        MetadataKind::WebService,
        MetadataKind::XdtoPackage,
        MetadataKind::EventSubscription,
    ])
}

const fn ownership_child_kinds() -> [NodeKind; 19] {
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
        NodeKind::DataCompositionSchema,
        NodeKind::DataSet,
        NodeKind::DataCompositionField,
        NodeKind::XdtoType,
        NodeKind::HttpServiceUrlTemplate,
        NodeKind::HttpServiceMethod,
        NodeKind::WebServiceOperation,
        NodeKind::WebServiceParameter,
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
        NodeKind::DataCompositionSchema => "data composition schema",
        NodeKind::DataSet => "data set",
        NodeKind::DataCompositionField => "data composition field",
        NodeKind::XdtoType => "XDTO type",
        NodeKind::HttpServiceUrlTemplate => "HTTP service URL template",
        NodeKind::HttpServiceMethod => "HTTP service method",
        NodeKind::WebServiceOperation => "Web service operation",
        NodeKind::WebServiceParameter => "Web service parameter",
        NodeKind::Form => "form",
        NodeKind::Command => "command",
        NodeKind::Attribute => "attribute",
        NodeKind::StandardAttribute => "standard attribute",
        NodeKind::TabularSection => "tabular section",
        NodeKind::Dimension => "dimension",
        NodeKind::Resource => "resource",
        NodeKind::Measure => "measure",
        NodeKind::Role => "role",
        NodeKind::AccessRight => "access right",
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
        | NodeKind::Role
        | NodeKind::StandardAttribute
        | NodeKind::Subsystem
        | NodeKind::AccessRight
        | NodeKind::DataCompositionSchema
        | NodeKind::DataSet
        | NodeKind::DataCompositionField
        | NodeKind::XdtoType
        | NodeKind::HttpServiceUrlTemplate
        | NodeKind::HttpServiceMethod
        | NodeKind::WebServiceOperation
        | NodeKind::WebServiceParameter => true,
        NodeKind::Unknown => false,
    }
}

fn representative_node_kind(kind: NodeKind) -> bool {
    match kind {
        NodeKind::Metadata(kind) => representative_metadata_kinds().contains(&kind),
        _ => edt_emits_node_kind(kind),
    }
}

const fn representative_node_test(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Metadata(MetadataKind::EventSubscription) => {
            "oneagent_edt::event_subscriptions::live_derived_fixture_is_consumer_visible_and_deterministic"
        }
        NodeKind::Metadata(_) => {
            "oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind"
        }
        NodeKind::Module
        | NodeKind::Procedure
        | NodeKind::Function
        | NodeKind::Query
        | NodeKind::Form
        | NodeKind::Command => {
            "oneagent_edt::sprint7_evidence::sprint7_repository_fixture_proves_modules_references_and_navigation_end_to_end"
        }
        NodeKind::Attribute | NodeKind::TabularSection => {
            "oneagent_edt::grants::grants_real_edt_fixture_emits_scoped_access_rights_with_stable_provenance"
        }
        NodeKind::DataCompositionSchema | NodeKind::DataSet | NodeKind::DataCompositionField => {
            "oneagent_edt::report_data_composition::live_derived_fixture_is_typed_consumer_visible_and_deterministic"
        }
        NodeKind::XdtoType
        | NodeKind::HttpServiceUrlTemplate
        | NodeKind::HttpServiceMethod
        | NodeKind::WebServiceOperation
        | NodeKind::WebServiceParameter => {
            "oneagent_edt::xdto_services::live_derived_fixture_is_consumer_visible_and_deterministic"
        }
        _ => "oneagent_edt::graph_tests",
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
        EdgeKind::Opens => "opens",
        EdgeKind::Triggers => "triggers",
    }
}

#[cfg(test)]
mod tests {
    use oneagent_graph::{
        EdgeKind, NodeKind, ResolutionState, SemanticCoverageCapabilityId,
        SemanticCoverageEvidence, SemanticCoverageGapPriority, SemanticCoverageRegistry,
        SemanticCoverageReport, SemanticCoverageStatus, SemanticDiagnosticSeverity,
        SemanticProvenanceCapability, SemanticReference, SemanticReferenceCapability,
    };
    use oneagent_metadata::MetadataKind;
    use std::{collections::BTreeSet, fs};

    use super::{EdtSemanticCoverageRegistry, all_metadata_kinds, metadata_reference_target_kinds};
    use crate::{
        EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
        query_source_resolution::WorkspaceResolutionScope,
    };

    fn sprint7_fixture() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sprint7_forms_commands_project")
    }

    fn assert_no_unplanned_high_gap(report: &SemanticCoverageReport) {
        let actual = report
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .into_iter()
            .map(|gap| gap.capability_id().as_str())
            .collect::<BTreeSet<_>>();

        assert!(actual.is_empty());
    }

    fn assert_metadata_node_has_complete_production_evidence(kind: MetadataKind) {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Metadata(kind),
            ))
            .expect("metadata node coverage must exist");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.related_node_kind(),
            Some(NodeKind::Metadata(kind))
        );
        assert_eq!(
            capability.representative_tests(),
            ["oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind"]
        );
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
    }

    #[test]
    fn subsystem_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::Subsystem);
    }

    #[test]
    fn enumeration_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::Enumeration);
    }

    #[test]
    fn report_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::Report);
    }

    #[test]
    fn data_processor_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::DataProcessor);
    }

    #[test]
    fn information_register_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::InformationRegister);
    }

    #[test]
    fn accounting_register_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::AccountingRegister);
    }

    #[test]
    fn calculation_register_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::CalculationRegister);
    }

    #[test]
    fn business_process_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::BusinessProcess);
    }

    #[test]
    fn task_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::Task);
    }

    #[test]
    fn role_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::Role);
    }

    #[test]
    fn common_form_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::CommonForm);
    }

    #[test]
    fn http_service_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::HttpService);
    }

    #[test]
    fn web_service_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::WebService);
    }

    #[test]
    fn xdto_package_metadata_node_has_complete_production_evidence() {
        assert_metadata_node_has_complete_production_evidence(MetadataKind::XdtoPackage);
    }

    #[test]
    fn member_nodes_have_complete_edt_payload_evidence() {
        let report = EdtSemanticCoverageRegistry::audit();
        let representative = "oneagent_edt::grants::grants_real_edt_fixture_emits_scoped_access_rights_with_stable_provenance";

        for kind in [NodeKind::Attribute, NodeKind::TabularSection] {
            let capability = report
                .capability(SemanticCoverageCapabilityId::SemanticNode(kind))
                .expect("EDT member node coverage must exist");

            assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
            assert_eq!(capability.evidence(), capability.required_evidence());
            assert!(
                capability
                    .evidence()
                    .contains(&SemanticCoverageEvidence::SemanticPayloadPreserved)
            );
            assert!(capability.missing_evidence().is_empty());
            assert!(capability.limitations().is_empty());
            assert_eq!(capability.representative_tests(), [representative]);
            assert!(
                report
                    .gaps()
                    .iter()
                    .all(|gap| gap.capability_id() != capability.id())
            );
        }
    }

    #[test]
    fn registry_is_deterministic_unique_and_consistent() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert!(!first.capabilities().is_empty());
        assert_eq!(first.summary().total(), first.capabilities().len());
        assert_eq!(first.summary().total(), 120);
        assert_eq!(
            first
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::Supported),
            Some(&115)
        );
        assert_eq!(
            first
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::NotApplicable),
            Some(&5)
        );
        assert_eq!(
            first
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::Unsupported),
            None
        );
        assert_eq!(
            first
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::DeclaredOnly),
            None
        );
        assert_no_unplanned_high_gap(&first);

        let graph_domain = SemanticCoverageRegistry::audit();
        assert_eq!(graph_domain.summary().total(), 96);
        assert_eq!(
            graph_domain
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::Supported),
            Some(&92)
        );
        assert_eq!(
            graph_domain
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::NotApplicable),
            Some(&4)
        );
        assert!(graph_domain.summary().by_gap_priority().is_empty());
    }

    #[test]
    fn opens_production_has_complete_edt_evidence() {
        let report = EdtSemanticCoverageRegistry::audit();
        let capability = report
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Opens))
            .expect("EDT Opens edge coverage must exist");

        assert_eq!(capability.stable_id(), "semantic_edge.opens");
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            [
                "oneagent_edt::form_navigation::changed_common_form_propagates_impact_through_opens",
                "oneagent_edt::form_navigation::form_navigation_diff_observes_a_missing_target_becoming_resolved",
                "oneagent_edt::form_navigation::form_navigation_emits_only_unique_resolved_opens_with_stable_evidence",
                "oneagent_edt::sprint7_evidence::sprint7_repository_fixture_proves_modules_references_and_navigation_end_to_end",
            ]
        );
        assert_no_unplanned_high_gap(&report);
    }

    #[test]
    fn opens_partial_workspace_outcome_uses_the_repository_fixture() {
        let result = FileSystemEdtSemanticGraphBuilder::build_graph_with_metadata_reference_scope(
            &sprint7_fixture(),
            WorkspaceResolutionScope::Partial,
        )
        .expect("partial Sprint 7 repository fixture must build");
        let diagnostic = result
            .diagnostics()
            .iter()
            .find(|diagnostic| {
                diagnostic.reference()
                    == &SemanticReference::Raw("Catalog.Absent.Form.Missing".to_owned())
                    && diagnostic.provenance().iter().any(|evidence| {
                        evidence.producer().as_str() == "oneagent.edt.form-navigation"
                    })
            })
            .expect("missing Form target must retain a partial-workspace diagnostic");

        assert_eq!(diagnostic.severity(), SemanticDiagnosticSeverity::Warning);
        assert_eq!(diagnostic.provenance().len(), 1);
        assert_eq!(
            diagnostic.provenance()[0].resolution(),
            ResolutionState::Partial
        );
        assert_eq!(
            result.graph().query().edges_by_kind(EdgeKind::Opens).len(),
            5
        );
        assert!(
            result
                .graph()
                .nodes()
                .all(|node| node.name().as_str() != "Absent")
        );
        assert!(result.validate().is_valid());
    }

    #[test]
    fn reference_request_provenance_has_complete_edt_production_evidence() {
        let report = EdtSemanticCoverageRegistry::audit();
        let capability = report
            .capability(SemanticCoverageCapabilityId::ProvenanceSource(
                SemanticProvenanceCapability::ReferenceRequest,
            ))
            .expect("EDT reference request provenance coverage must exist");

        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            [
                "oneagent_edt::graph_tests::ambiguous_metadata_reference_target_is_reported",
                "oneagent_edt::graph_tests::duplicate_identical_reference_diagnostic_is_deduplicated",
                "oneagent_edt::graph_tests::duplicate_metadata_type_reference_creates_one_depends_on_edge",
                "oneagent_edt::graph_tests::incompatible_metadata_reference_target_kind_is_reported",
                "oneagent_edt::graph_tests::missing_metadata_reference_target_is_reported",
                "oneagent_edt::graph_tests::production_builder_preserves_explicit_partial_workspace_request",
                "oneagent_edt::graph_tests::production_builder_propagates_explicit_partial_scope_to_query_requests",
                "oneagent_edt::graph_tests::request_identity_survives_missing_to_resolved_production_diff",
                "oneagent_edt::graph_tests::resolves_all_mapped_metadata_reference_target_kinds_through_production_builder",
                "oneagent_edt::graph_tests::resolves_metadata_reference_and_depends_on_edges",
                "oneagent_edt::reads::reads_ambiguous_target_is_sorted_counted_and_emits_no_edge",
                "oneagent_edt::reads::reads_incompatible_target_is_typed_counted_and_emits_no_edge",
                "oneagent_edt::reads::reads_parser_and_missing_failures_are_typed_counted_and_deterministic",
                "oneagent_edt::sprint8_registers_queries::sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible",
            ]
        );
        assert!(
            report
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
    }

    #[test]
    fn sprint8_query_evidence_preserves_supported_coverage_aggregates() {
        let report = EdtSemanticCoverageRegistry::audit();
        let representative = "oneagent_edt::sprint8_registers_queries::sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible";
        let requests = report
            .capability(SemanticCoverageCapabilityId::ProvenanceSource(
                SemanticProvenanceCapability::ReferenceRequest,
            ))
            .expect("ReferenceRequest coverage must exist");

        for kind in [EdgeKind::Reads, EdgeKind::DependsOn] {
            let capability = report
                .capability(SemanticCoverageCapabilityId::SemanticEdge(kind))
                .expect("Sprint 8 edge coverage must exist");
            assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
            assert_eq!(capability.evidence(), capability.required_evidence());
            assert!(
                capability
                    .representative_tests()
                    .iter()
                    .any(|test| test == representative)
            );
        }
        assert_eq!(requests.status(), SemanticCoverageStatus::Supported);
        assert!(
            requests
                .representative_tests()
                .iter()
                .any(|test| test == representative)
        );
        assert_eq!(report.summary().total(), 120);
        assert_eq!(
            report
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::Supported),
            Some(&115)
        );
        assert_eq!(
            report
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::NotApplicable),
            Some(&5)
        );
        assert_eq!(
            report
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::Unsupported),
            None
        );
        assert_eq!(
            report
                .summary()
                .by_status()
                .get(&SemanticCoverageStatus::DeclaredOnly),
            None
        );
        assert_no_unplanned_high_gap(&report);
    }

    #[test]
    fn all_mapped_metadata_reference_capabilities_are_complete_and_deterministic() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let expected_kinds = metadata_reference_target_kinds();
        let mut expected_registry_order = expected_kinds.to_vec();
        expected_registry_order.sort_by_key(|kind| kind.as_str());
        let actual_kinds = first
            .capabilities()
            .iter()
            .filter_map(|capability| match capability.id() {
                SemanticCoverageCapabilityId::MetadataReference(
                    SemanticReferenceCapability::MetadataType(kind),
                ) => Some(kind),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(first, second);
        assert_eq!(first.capabilities(), second.capabilities());
        assert_eq!(first.gaps(), second.gaps());
        assert_eq!(actual_kinds, expected_registry_order);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());

        for kind in expected_kinds {
            let capability = first
                .capability(SemanticCoverageCapabilityId::MetadataReference(
                    SemanticReferenceCapability::MetadataType(kind),
                ))
                .expect("mapped metadata reference capability must exist");

            assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
            assert_eq!(capability.evidence(), capability.required_evidence());
            assert!(capability.missing_evidence().is_empty());
            assert!(capability.limitations().is_empty());
            assert_eq!(
                capability.representative_tests(),
                [
                    "oneagent_edt::graph_tests::resolves_all_mapped_metadata_reference_target_kinds_through_production_builder"
                ]
            );
            assert!(
                first
                    .gaps()
                    .iter()
                    .all(|gap| gap.capability_id() != capability.id())
            );
        }

        assert!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Critical)
                .is_empty()
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );

        let graph_domain = SemanticCoverageRegistry::audit();
        assert_eq!(
            graph_domain
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len()
                + first
                    .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                    .len(),
            0
        );
    }

    #[test]
    fn writes_edge_production_closes_the_final_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let writes = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Writes))
            .expect("writes edge coverage must exist");

        assert_eq!(first, second);
        assert_eq!(first.capabilities(), second.capabilities());
        assert_eq!(first.gaps(), second.gaps());
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(writes.status(), SemanticCoverageStatus::Supported);
        assert_eq!(writes.evidence(), writes.required_evidence());
        assert!(writes.missing_evidence().is_empty());
        assert!(writes.limitations().is_empty());
        assert_eq!(
            writes.representative_tests(),
            [
                "oneagent_edt::writes::writes_full_builder_emits_canonical_edges_with_query_coverage_and_repeated_build_evidence"
            ]
        );
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != writes.id())
        );
        assert!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Critical)
                .is_empty()
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );

        let graph_domain = SemanticCoverageRegistry::audit();
        assert_eq!(
            graph_domain
                .gaps_by_priority(SemanticCoverageGapPriority::Critical)
                .len()
                + first
                    .gaps_by_priority(SemanticCoverageGapPriority::Critical)
                    .len(),
            0
        );
        assert_eq!(
            graph_domain
                .gaps_by_priority(SemanticCoverageGapPriority::High)
                .len()
                + first
                    .gaps_by_priority(SemanticCoverageGapPriority::High)
                    .len(),
            0
        );
        assert_eq!(
            graph_domain
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len()
                + first
                    .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                    .len(),
            0
        );
    }

    #[test]
    fn metadata_payload_transition_completes_every_supported_entity() {
        let report = EdtSemanticCoverageRegistry::audit();
        let representative =
            "oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind";

        for kind in all_metadata_kinds().into_iter().filter(|kind| {
            !matches!(
                kind,
                MetadataKind::Form | MetadataKind::EventSubscription | MetadataKind::Unknown
            )
        }) {
            let capability = report
                .capability(SemanticCoverageCapabilityId::MetadataEntity(kind))
                .expect("supported metadata coverage must exist");

            assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
            assert_eq!(capability.evidence(), capability.required_evidence());
            assert!(capability.missing_evidence().is_empty());
            assert!(capability.limitations().is_empty());
            assert_eq!(capability.representative_tests(), [representative]);
            assert!(
                report
                    .gaps()
                    .iter()
                    .all(|gap| gap.capability_id() != capability.id())
            );
        }
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

        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            ["oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind"]
        );
        assert!(
            report
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );

        assert_no_unplanned_high_gap(&report);
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
        assert_eq!(command.status(), SemanticCoverageStatus::Supported);
        assert_eq!(command.evidence(), command.required_evidence());
        assert!(command.missing_evidence().is_empty());

        assert_no_unplanned_high_gap(&report);
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

        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.limitations().is_empty());
        assert_eq!(
            capability.representative_tests(),
            ["oneagent_edt::payload::payload_matrix_covers_every_supported_edt_metadata_kind"]
        );
        assert_eq!(node.status(), SemanticCoverageStatus::Supported);
        assert!(node.missing_evidence().is_empty());
        assert!(
            report
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );

        let form = report
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Form,
            ))
            .expect("form coverage must exist");
        assert_eq!(form.status(), SemanticCoverageStatus::NotApplicable);

        assert_no_unplanned_high_gap(&report);
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
        assert_eq!(document.status(), SemanticCoverageStatus::Supported);
        assert_eq!(document.evidence(), document.required_evidence());
        assert!(document.missing_evidence().is_empty());

        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
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
        assert_eq!(flat_unknown.status(), SemanticCoverageStatus::NotApplicable);
        assert!(flat_unknown.evidence().is_empty());
        assert!(flat_unknown.required_evidence().is_empty());
        assert!(flat_unknown.missing_evidence().is_empty());
        assert!(flat_unknown.limitations().is_empty());
        assert!(!flat_unknown.notes().is_empty());

        let high_gaps = first.gaps_by_priority(SemanticCoverageGapPriority::High);
        assert!(high_gaps.is_empty());
        assert!(
            high_gaps
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert!(
            high_gaps
                .iter()
                .all(|gap| gap.capability_id() != flat_unknown.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );
    }

    #[test]
    fn flat_unknown_node_is_not_applicable_to_edt() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Unknown,
            ))
            .expect("flat unknown node coverage must remain registered");
        let measure_ownership = first
            .capability(SemanticCoverageCapabilityId::OwnershipRelation(
                NodeKind::Measure,
            ))
            .expect("measure ownership coverage must remain registered");
        let reads = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Reads))
            .expect("reads edge coverage must remain registered");
        let subsystem = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Subsystem,
            ))
            .expect("subsystem node coverage must remain supported");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::NotApplicable);
        assert!(capability.evidence().is_empty());
        assert!(capability.required_evidence().is_empty());
        assert!(capability.missing_evidence().is_empty());
        assert_eq!(capability.related_node_kind(), Some(NodeKind::Unknown));
        assert!(capability.limitations().is_empty());
        assert!(!capability.notes().is_empty());
        assert!(capability.representative_tests().is_empty());

        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert_eq!(
            measure_ownership.status(),
            SemanticCoverageStatus::Supported
        );
        assert_eq!(
            measure_ownership.evidence(),
            measure_ownership.required_evidence()
        );
        assert!(measure_ownership.missing_evidence().is_empty());
        assert_eq!(reads.status(), SemanticCoverageStatus::Supported);
        assert_eq!(subsystem.status(), SemanticCoverageStatus::Supported);

        assert_eq!(
            first
                .summary()
                .by_gap_priority()
                .get(&SemanticCoverageGapPriority::High),
            None
        );
        assert_eq!(
            first
                .summary()
                .by_gap_priority()
                .get(&SemanticCoverageGapPriority::Medium),
            None
        );
        assert_no_unplanned_high_gap(&first);
    }

    #[test]
    fn access_right_node_and_grants_edge_share_complete_production_evidence() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::AccessRight,
            ))
            .expect("access right node coverage must exist");
        let grants = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(EdgeKind::Grants))
            .expect("grants edge coverage must exist");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert_eq!(capability.related_node_kind(), Some(NodeKind::AccessRight));
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert_eq!(grants.status(), SemanticCoverageStatus::Supported);
        assert_eq!(grants.evidence(), grants.required_evidence());
        assert!(grants.missing_evidence().is_empty());
        assert!(grants.representative_tests().iter().any(|test| {
            test == "oneagent_edt::grants::grants_real_edt_fixture_emits_scoped_access_rights_with_stable_provenance"
        }));
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != grants.id())
        );
        assert_eq!(
            first
                .summary()
                .by_gap_priority()
                .get(&SemanticCoverageGapPriority::High),
            None
        );
        assert_eq!(
            first
                .summary()
                .by_gap_priority()
                .get(&SemanticCoverageGapPriority::Medium),
            None
        );
        assert_no_unplanned_high_gap(&first);
    }

    #[test]
    fn includes_edge_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let includes = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(
                EdgeKind::Includes,
            ))
            .expect("Includes edge coverage must exist");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert_eq!(includes.status(), SemanticCoverageStatus::Supported);
        assert_eq!(includes.evidence(), includes.required_evidence());
        assert!(includes.missing_evidence().is_empty());
        assert!(includes.representative_tests().iter().any(|test| {
            test == "oneagent_edt::includes::includes_real_edt_fixture_emits_direct_metadata_members_with_stable_provenance"
        }));
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != includes.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
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
        let unknown = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Unknown,
            ))
            .expect("unknown coverage must remain registered");

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
        assert_eq!(unknown.status(), SemanticCoverageStatus::NotApplicable);
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != unknown.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );
    }

    #[test]
    fn standard_attribute_node_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::StandardAttribute,
            ))
            .expect("standard attribute node coverage must exist");
        let ownership = first
            .capability(SemanticCoverageCapabilityId::OwnershipRelation(
                NodeKind::StandardAttribute,
            ))
            .expect("standard attribute ownership coverage must remain registered");

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
        assert_eq!(ownership.status(), SemanticCoverageStatus::Supported);
        assert_eq!(ownership.evidence(), ownership.required_evidence());
        assert!(ownership.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != ownership.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );
    }

    #[test]
    fn subsystem_node_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Subsystem,
            ))
            .expect("subsystem node coverage must exist");
        let unknown = first
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Unknown,
            ))
            .expect("unknown node coverage must remain registered");

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
        assert_eq!(unknown.status(), SemanticCoverageStatus::NotApplicable);
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != unknown.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
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

        assert_eq!(reads.status(), SemanticCoverageStatus::Supported);
        assert_eq!(reads.evidence(), reads.required_evidence());
        assert!(reads.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != reads.id())
        );
        assert_eq!(writes.status(), SemanticCoverageStatus::Supported);
        assert_eq!(writes.evidence(), writes.required_evidence());
        assert!(writes.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != writes.id())
        );
        assert_eq!(depends_on.status(), SemanticCoverageStatus::Supported);
        assert_eq!(depends_on.evidence(), depends_on.required_evidence());
        assert!(depends_on.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != depends_on.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );
    }

    #[test]
    fn extends_edge_production_closes_the_selected_high_gap() {
        let first = EdtSemanticCoverageRegistry::audit();
        let second = EdtSemanticCoverageRegistry::audit();
        let capability = first
            .capability(SemanticCoverageCapabilityId::SemanticEdge(
                EdgeKind::Extends,
            ))
            .expect("extends edge coverage must exist");

        assert_eq!(first, second);
        assert!(first.is_consistent());
        assert!(first.duplicate_ids().is_empty());
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
        assert!(capability.representative_tests().iter().any(|test| {
            test == "oneagent_edt::graph_tests::emits_metadata_extends_edges_for_adopted_objects"
        }));
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != capability.id())
        );
        assert_no_unplanned_high_gap(&first);
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

        assert_eq!(ownership.status(), SemanticCoverageStatus::Supported);
        assert_eq!(ownership.evidence(), ownership.required_evidence());
        assert!(ownership.missing_evidence().is_empty());
        assert!(
            first
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != ownership.id())
        );
        assert_no_unplanned_high_gap(&first);
        assert_eq!(
            first
                .gaps_by_priority(SemanticCoverageGapPriority::Medium)
                .len(),
            0
        );
    }
}
