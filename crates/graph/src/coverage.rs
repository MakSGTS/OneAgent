//! Deterministic Semantic Model coverage audit.
//!
//! Static coverage describes implemented project capabilities. Observed coverage
//! describes which node and edge kinds occur in one graph snapshot. Absence from
//! a snapshot never changes the static support status of a capability.

use std::collections::{BTreeMap, BTreeSet};

use oneagent_metadata::MetadataKind;

use crate::{EdgeKind, NodeKind, SemanticGraph, SemanticGraphValidationCode};

/// Stable coverage capability category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCoverageCategory {
    /// EDT or another adapter can discover and parse a metadata entity.
    MetadataEntity,
    /// The semantic graph can represent a node kind.
    SemanticNode,
    /// An owner-child relation is represented in the graph.
    OwnershipRelation,
    /// A typed semantic reference is extracted and resolved.
    MetadataReference,
    /// The semantic graph can represent an edge kind.
    SemanticEdge,
    /// A semantic fact carries source provenance.
    ProvenanceSource,
    /// A graph or build invariant is validated.
    ValidationRule,
    /// A semantic relation is available through the Query API.
    QueryCapability,
    /// An edge semantic participates in Impact Analysis.
    ImpactPropagationCapability,
}

impl SemanticCoverageCategory {
    /// Returns the stable machine-readable category identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataEntity => "metadata_entity",
            Self::SemanticNode => "semantic_node",
            Self::OwnershipRelation => "ownership_relation",
            Self::MetadataReference => "metadata_reference",
            Self::SemanticEdge => "semantic_edge",
            Self::ProvenanceSource => "provenance_source",
            Self::ValidationRule => "validation_rule",
            Self::QueryCapability => "query_capability",
            Self::ImpactPropagationCapability => "impact_propagation_capability",
        }
    }
}

/// Static support status, independent from occurrence in one graph snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCoverageStatus {
    /// Every evidence item required by the capability is present.
    Supported,
    /// Some implementation exists, but at least one required stage is absent.
    PartiallySupported,
    /// The capability is relevant but not implemented by the audited pipeline.
    Unsupported,
    /// The stage is objectively inapplicable to the capability.
    NotApplicable,
    /// A model variant exists without production pipeline integration.
    DeclaredOnly,
}

/// Typed evidence used to justify a coverage status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCoverageEvidence {
    /// A source object is discovered.
    Discovered,
    /// Source input is parsed into a typed descriptor.
    Parsed,
    /// A typed semantic representation exists.
    Modeled,
    /// A corresponding `NodeKind` is declared.
    NodeKindDeclared,
    /// A graph builder emits the node.
    NodeEmitted,
    /// Stable semantic identity is assigned.
    StableIdentityAssigned,
    /// Expected typed semantic payload is preserved.
    SemanticPayloadPreserved,
    /// Provenance is attached.
    ProvenanceAttached,
    /// An owner is resolved.
    OwnerResolved,
    /// A containment edge is emitted.
    OwnershipEdgeEmitted,
    /// A semantic reference is extracted.
    ReferenceExtracted,
    /// A semantic reference is resolved.
    ReferenceResolved,
    /// A typed diagnostic is emitted on failure.
    DiagnosticEmitted,
    /// A corresponding `EdgeKind` is declared.
    EdgeKindDeclared,
    /// A semantic edge is emitted.
    SemanticEdgeEmitted,
    /// An endpoint or owner validation rule exists.
    ValidationRuleExists,
    /// The Query API exposes the capability.
    QuerySupportExists,
    /// Impact Analysis has an explicit propagation policy.
    ImpactPropagationExists,
    /// A representative positive test exists.
    PositiveTestExists,
    /// A representative negative test exists.
    NegativeTestExists,
    /// An end-to-end integration test exists.
    IntegrationTestExists,
    /// Resolution statistics include the capability.
    ResolutionStatisticsRecorded,
}

/// Typed metadata reference capability identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticReferenceCapability {
    /// EDT metadata type reference to a target metadata kind.
    MetadataType(MetadataKind),
    /// Local or qualified BSL call reference.
    BslCall,
}

/// Typed provenance path identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticProvenanceCapability {
    /// Top-level metadata object node provenance.
    MetadataObjectNode,
    /// Metadata child node provenance.
    MetadataChildNode,
    /// BSL module node provenance.
    ModuleNode,
    /// BSL declaration node provenance.
    SymbolNode,
    /// Owner-child edge provenance.
    OwnershipEdge,
    /// Resolved reference edge provenance.
    ResolvedReferenceEdge,
    /// Semantic diagnostic provenance.
    Diagnostic,
    /// Pending reference request provenance.
    ReferenceRequest,
}

/// Typed Query API capability identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticQueryCapability {
    /// Stable identifier lookup.
    NodeLookup,
    /// Exact name and kind lookup.
    NameAndKindLookup,
    /// Owner and child navigation.
    OwnershipNavigation,
    /// Typed edge lookup and adjacency.
    EdgeNavigation,
    /// Dependency and usage classification.
    DependencyNavigation,
    /// Bounded deterministic traversal.
    Traversal,
}

/// Stable typed identity of one coverage capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCoverageCapabilityId {
    /// End-to-end metadata entity handling.
    MetadataEntity(MetadataKind),
    /// Semantic node model and graph integration.
    SemanticNode(NodeKind),
    /// Owner-child handling for a child node kind.
    OwnershipRelation(NodeKind),
    /// Typed reference extraction and resolution.
    MetadataReference(SemanticReferenceCapability),
    /// Semantic edge model and graph integration.
    SemanticEdge(EdgeKind),
    /// Provenance path.
    ProvenanceSource(SemanticProvenanceCapability),
    /// Validation rule.
    ValidationRule(SemanticGraphValidationCode),
    /// Query API capability.
    QueryCapability(SemanticQueryCapability),
    /// Impact propagation policy for an edge kind.
    ImpactPropagation(EdgeKind),
}

impl SemanticCoverageCapabilityId {
    /// Returns the category implied by this typed identity.
    #[must_use]
    pub const fn category(self) -> SemanticCoverageCategory {
        match self {
            Self::MetadataEntity(_) => SemanticCoverageCategory::MetadataEntity,
            Self::SemanticNode(_) => SemanticCoverageCategory::SemanticNode,
            Self::OwnershipRelation(_) => SemanticCoverageCategory::OwnershipRelation,
            Self::MetadataReference(_) => SemanticCoverageCategory::MetadataReference,
            Self::SemanticEdge(_) => SemanticCoverageCategory::SemanticEdge,
            Self::ProvenanceSource(_) => SemanticCoverageCategory::ProvenanceSource,
            Self::ValidationRule(_) => SemanticCoverageCategory::ValidationRule,
            Self::QueryCapability(_) => SemanticCoverageCategory::QueryCapability,
            Self::ImpactPropagation(_) => SemanticCoverageCategory::ImpactPropagationCapability,
        }
    }

    /// Returns the stable machine-readable capability identifier.
    #[must_use]
    pub fn as_str(self) -> String {
        match self {
            Self::MetadataEntity(kind) => format!("metadata_entity.{}", kind.as_str()),
            Self::SemanticNode(NodeKind::Metadata(kind)) => {
                format!("semantic_node.metadata.{}", kind.as_str())
            }
            Self::SemanticNode(kind) => format!("semantic_node.{}", node_kind_code(kind)),
            Self::OwnershipRelation(NodeKind::Metadata(kind)) => {
                format!("ownership_relation.metadata.{}", kind.as_str())
            }
            Self::OwnershipRelation(kind) => format!("ownership_relation.{}", node_kind_code(kind)),
            Self::MetadataReference(SemanticReferenceCapability::MetadataType(kind)) => {
                format!("metadata_reference.type.{}", kind.as_str())
            }
            Self::MetadataReference(SemanticReferenceCapability::BslCall) => {
                "metadata_reference.bsl_call".to_owned()
            }
            Self::SemanticEdge(kind) => format!("semantic_edge.{}", edge_kind_code(kind)),
            Self::ProvenanceSource(kind) => {
                format!("provenance_source.{}", provenance_kind_code(kind))
            }
            Self::ValidationRule(code) => format!("validation_rule.{}", code.as_str()),
            Self::QueryCapability(kind) => {
                format!("query_capability.{}", query_kind_code(kind))
            }
            Self::ImpactPropagation(kind) => {
                format!("impact_propagation.{}", edge_kind_code(kind))
            }
        }
    }
}

/// One read-only static capability descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCoverageCapability {
    id: SemanticCoverageCapabilityId,
    title: String,
    status: SemanticCoverageStatus,
    evidence: BTreeSet<SemanticCoverageEvidence>,
    required_evidence: BTreeSet<SemanticCoverageEvidence>,
    limitations: Vec<String>,
    related_node_kind: Option<NodeKind>,
    related_edge_kind: Option<EdgeKind>,
    source_metadata_kind: Option<MetadataKind>,
    representative_tests: Vec<String>,
    notes: Vec<String>,
}

impl SemanticCoverageCapability {
    /// Creates a typed capability descriptor.
    #[must_use]
    pub fn new(
        id: SemanticCoverageCapabilityId,
        title: impl Into<String>,
        status: SemanticCoverageStatus,
        evidence: impl IntoIterator<Item = SemanticCoverageEvidence>,
        required_evidence: impl IntoIterator<Item = SemanticCoverageEvidence>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            status,
            evidence: evidence.into_iter().collect(),
            required_evidence: required_evidence.into_iter().collect(),
            limitations: Vec::new(),
            related_node_kind: None,
            related_edge_kind: None,
            source_metadata_kind: None,
            representative_tests: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Associates a node kind with this capability.
    #[must_use]
    pub const fn with_node_kind(mut self, kind: NodeKind) -> Self {
        self.related_node_kind = Some(kind);
        self
    }

    /// Associates an edge kind with this capability.
    #[must_use]
    pub const fn with_edge_kind(mut self, kind: EdgeKind) -> Self {
        self.related_edge_kind = Some(kind);
        self
    }

    /// Associates a source metadata kind with this capability.
    #[must_use]
    pub const fn with_metadata_kind(mut self, kind: MetadataKind) -> Self {
        self.source_metadata_kind = Some(kind);
        self
    }

    /// Adds a known limitation.
    #[must_use]
    pub fn with_limitation(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self.limitations.sort();
        self.limitations.dedup();
        self
    }

    /// Adds a stable representative test reference.
    #[must_use]
    pub fn with_representative_test(mut self, test: impl Into<String>) -> Self {
        self.representative_tests.push(test.into());
        self.representative_tests.sort();
        self.representative_tests.dedup();
        self
    }

    /// Adds a human-readable note that does not affect identity or status.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self.notes.sort();
        self.notes.dedup();
        self
    }

    /// Returns the typed identity.
    #[must_use]
    pub const fn id(&self) -> SemanticCoverageCapabilityId {
        self.id
    }

    /// Returns the stable identifier.
    #[must_use]
    pub fn stable_id(&self) -> String {
        self.id.as_str()
    }

    /// Returns the category derived from the identity.
    #[must_use]
    pub const fn category(&self) -> SemanticCoverageCategory {
        self.id.category()
    }

    /// Returns the human-readable title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the static support status.
    #[must_use]
    pub const fn status(&self) -> SemanticCoverageStatus {
        self.status
    }

    /// Returns ordered typed evidence.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<SemanticCoverageEvidence> {
        &self.evidence
    }

    /// Returns ordered required evidence.
    #[must_use]
    pub const fn required_evidence(&self) -> &BTreeSet<SemanticCoverageEvidence> {
        &self.required_evidence
    }

    /// Returns missing required evidence.
    #[must_use]
    pub fn missing_evidence(&self) -> BTreeSet<SemanticCoverageEvidence> {
        self.required_evidence
            .difference(&self.evidence)
            .copied()
            .collect()
    }

    /// Returns known limitations.
    #[must_use]
    pub fn limitations(&self) -> &[String] {
        &self.limitations
    }

    /// Returns the related node kind.
    #[must_use]
    pub const fn related_node_kind(&self) -> Option<NodeKind> {
        self.related_node_kind
    }

    /// Returns the related edge kind.
    #[must_use]
    pub const fn related_edge_kind(&self) -> Option<EdgeKind> {
        self.related_edge_kind
    }

    /// Returns the source metadata kind.
    #[must_use]
    pub const fn source_metadata_kind(&self) -> Option<MetadataKind> {
        self.source_metadata_kind
    }

    /// Returns representative test references.
    #[must_use]
    pub fn representative_tests(&self) -> &[String] {
        &self.representative_tests
    }

    /// Returns human-readable notes.
    #[must_use]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }
}

/// Explicit priority assigned by coverage governance policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCoverageGapPriority {
    /// Silent semantic loss or a systemic identity/provenance failure.
    Critical,
    /// A core entity, ownership relation, reference or edge is absent.
    High,
    /// Support or representative verification is incomplete.
    Medium,
    /// Documentation or an additional non-critical scenario is missing.
    Low,
}

/// One deterministic actionable coverage gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCoverageGap {
    capability_id: SemanticCoverageCapabilityId,
    category: SemanticCoverageCategory,
    status: SemanticCoverageStatus,
    missing_evidence: BTreeSet<SemanticCoverageEvidence>,
    priority: SemanticCoverageGapPriority,
    explanation: String,
    suggested_next_action: String,
    related_node_kind: Option<NodeKind>,
    related_edge_kind: Option<EdgeKind>,
}

impl SemanticCoverageGap {
    fn from_capability(capability: &SemanticCoverageCapability) -> Option<Self> {
        let missing_evidence = capability.missing_evidence();
        if capability.status() == SemanticCoverageStatus::Supported && missing_evidence.is_empty() {
            return None;
        }
        if capability.status() == SemanticCoverageStatus::NotApplicable {
            return None;
        }

        let priority = gap_priority(capability, &missing_evidence);
        let explanation = capability
            .limitations()
            .first()
            .cloned()
            .unwrap_or_else(|| {
                format!(
                    "{} is {} with incomplete required evidence",
                    capability.stable_id(),
                    status_code(capability.status())
                )
            });
        let suggested_next_action = suggested_next_action(capability, &missing_evidence);

        Some(Self {
            capability_id: capability.id(),
            category: capability.category(),
            status: capability.status(),
            missing_evidence,
            priority,
            explanation,
            suggested_next_action,
            related_node_kind: capability.related_node_kind(),
            related_edge_kind: capability.related_edge_kind(),
        })
    }

    /// Returns the stable gap identity.
    #[must_use]
    pub fn stable_id(&self) -> String {
        format!("coverage_gap.{}", self.capability_id.as_str())
    }

    /// Returns the affected capability identity.
    #[must_use]
    pub const fn capability_id(&self) -> SemanticCoverageCapabilityId {
        self.capability_id
    }

    /// Returns the capability category.
    #[must_use]
    pub const fn category(&self) -> SemanticCoverageCategory {
        self.category
    }

    /// Returns the current support status.
    #[must_use]
    pub const fn status(&self) -> SemanticCoverageStatus {
        self.status
    }

    /// Returns ordered missing evidence.
    #[must_use]
    pub const fn missing_evidence(&self) -> &BTreeSet<SemanticCoverageEvidence> {
        &self.missing_evidence
    }

    /// Returns the explicitly assigned priority.
    #[must_use]
    pub const fn priority(&self) -> SemanticCoverageGapPriority {
        self.priority
    }

    /// Returns a concise explanation.
    #[must_use]
    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    /// Returns the recommended next implementation action.
    #[must_use]
    pub fn suggested_next_action(&self) -> &str {
        &self.suggested_next_action
    }

    /// Returns the related node kind.
    #[must_use]
    pub const fn related_node_kind(&self) -> Option<NodeKind> {
        self.related_node_kind
    }

    /// Returns the related edge kind.
    #[must_use]
    pub const fn related_edge_kind(&self) -> Option<EdgeKind> {
        self.related_edge_kind
    }
}

/// Deterministic capability totals by category and support status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCoverageSummary {
    total: usize,
    by_category: BTreeMap<SemanticCoverageCategory, usize>,
    by_status: BTreeMap<SemanticCoverageStatus, usize>,
    by_gap_priority: BTreeMap<SemanticCoverageGapPriority, usize>,
}

impl SemanticCoverageSummary {
    fn from_capabilities_and_gaps(
        capabilities: &[SemanticCoverageCapability],
        gaps: &[SemanticCoverageGap],
    ) -> Self {
        let mut by_category = BTreeMap::new();
        let mut by_status = BTreeMap::new();
        let mut by_gap_priority = BTreeMap::new();

        for capability in capabilities {
            *by_category.entry(capability.category()).or_default() += 1;
            *by_status.entry(capability.status()).or_default() += 1;
        }
        for gap in gaps {
            *by_gap_priority.entry(gap.priority()).or_default() += 1;
        }

        Self {
            total: capabilities.len(),
            by_category,
            by_status,
            by_gap_priority,
        }
    }

    /// Returns the total known capability count.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns deterministic counts by category.
    #[must_use]
    pub const fn by_category(&self) -> &BTreeMap<SemanticCoverageCategory, usize> {
        &self.by_category
    }

    /// Returns deterministic counts by support status.
    #[must_use]
    pub const fn by_status(&self) -> &BTreeMap<SemanticCoverageStatus, usize> {
        &self.by_status
    }

    /// Returns deterministic counts by gap priority.
    #[must_use]
    pub const fn by_gap_priority(&self) -> &BTreeMap<SemanticCoverageGapPriority, usize> {
        &self.by_gap_priority
    }
}

/// Owned deterministic static capability matrix and derived gaps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCoverageReport {
    capabilities: Vec<SemanticCoverageCapability>,
    summary: SemanticCoverageSummary,
    gaps: Vec<SemanticCoverageGap>,
}

impl SemanticCoverageReport {
    /// Builds a report from explicitly declared capability descriptors.
    ///
    /// Capability identity, not title, controls sorting and uniqueness checks.
    #[must_use]
    pub fn from_capabilities(
        capabilities: impl IntoIterator<Item = SemanticCoverageCapability>,
    ) -> Self {
        let mut capabilities = capabilities.into_iter().collect::<Vec<_>>();
        capabilities.sort_by_key(SemanticCoverageCapability::stable_id);

        let mut gaps = capabilities
            .iter()
            .filter_map(SemanticCoverageGap::from_capability)
            .collect::<Vec<_>>();
        gaps.sort_by_key(|gap| (gap.priority(), gap.category(), gap.capability_id().as_str()));

        let summary = SemanticCoverageSummary::from_capabilities_and_gaps(&capabilities, &gaps);

        Self {
            capabilities,
            summary,
            gaps,
        }
    }

    /// Returns capabilities ordered by stable identity.
    #[must_use]
    pub fn capabilities(&self) -> &[SemanticCoverageCapability] {
        &self.capabilities
    }

    /// Returns summary counters.
    #[must_use]
    pub const fn summary(&self) -> &SemanticCoverageSummary {
        &self.summary
    }

    /// Returns gaps ordered by priority, category and capability identity.
    #[must_use]
    pub fn gaps(&self) -> &[SemanticCoverageGap] {
        &self.gaps
    }

    /// Returns a capability by typed identity.
    #[must_use]
    pub fn capability(
        &self,
        id: SemanticCoverageCapabilityId,
    ) -> Option<&SemanticCoverageCapability> {
        self.capabilities
            .iter()
            .find(|capability| capability.id() == id)
    }

    /// Returns duplicate stable capability identities, if any.
    #[must_use]
    pub fn duplicate_ids(&self) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        for capability in &self.capabilities {
            let id = capability.stable_id();
            if !seen.insert(id.clone()) {
                duplicates.insert(id);
            }
        }
        duplicates.into_iter().collect()
    }

    /// Returns capabilities without representative positive test evidence.
    #[must_use]
    pub fn capabilities_without_positive_tests(&self) -> Vec<&SemanticCoverageCapability> {
        self.capabilities
            .iter()
            .filter(|capability| {
                capability.status() != SemanticCoverageStatus::NotApplicable
                    && !capability
                        .evidence()
                        .contains(&SemanticCoverageEvidence::PositiveTestExists)
            })
            .collect()
    }

    /// Returns capabilities without representative negative test evidence.
    #[must_use]
    pub fn capabilities_without_negative_tests(&self) -> Vec<&SemanticCoverageCapability> {
        self.capabilities
            .iter()
            .filter(|capability| {
                capability.status() != SemanticCoverageStatus::NotApplicable
                    && capability
                        .required_evidence()
                        .contains(&SemanticCoverageEvidence::NegativeTestExists)
                    && !capability
                        .evidence()
                        .contains(&SemanticCoverageEvidence::NegativeTestExists)
            })
            .collect()
    }

    /// Returns gaps of one priority in existing deterministic order.
    #[must_use]
    pub fn gaps_by_priority(
        &self,
        priority: SemanticCoverageGapPriority,
    ) -> Vec<&SemanticCoverageGap> {
        self.gaps
            .iter()
            .filter(|gap| gap.priority() == priority)
            .collect()
    }

    /// Returns declared model capabilities not emitted by the audited pipeline.
    #[must_use]
    pub fn declared_but_unused(&self) -> Vec<&SemanticCoverageCapability> {
        self.capabilities
            .iter()
            .filter(|capability| capability.status() == SemanticCoverageStatus::DeclaredOnly)
            .collect()
    }

    /// Returns capabilities that require but lack provenance evidence.
    #[must_use]
    pub fn capabilities_missing_provenance(&self) -> Vec<&SemanticCoverageCapability> {
        self.capabilities
            .iter()
            .filter(|capability| {
                capability
                    .required_evidence()
                    .contains(&SemanticCoverageEvidence::ProvenanceAttached)
                    && !capability
                        .evidence()
                        .contains(&SemanticCoverageEvidence::ProvenanceAttached)
            })
            .collect()
    }

    /// Returns whether registry and derived report invariants hold.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        if !self.duplicate_ids().is_empty()
            || self.summary.total() != self.capabilities.len()
            || self
                .capabilities
                .windows(2)
                .any(|pair| pair[0].stable_id() > pair[1].stable_id())
        {
            return false;
        }

        self.capabilities.iter().all(capability_is_consistent)
            && self.gaps.iter().all(|gap| {
                self.capability(gap.capability_id()).is_some()
                    && (!gap.missing_evidence().is_empty()
                        || gap.status() == SemanticCoverageStatus::DeclaredOnly
                        || gap.status() == SemanticCoverageStatus::Unsupported)
            })
    }
}

/// Source-independent graph-domain coverage registry.
#[derive(Debug, Clone, Copy)]
pub struct SemanticCoverageRegistry;

impl SemanticCoverageRegistry {
    /// Audits graph-domain model, validation, query, impact and provenance capabilities.
    #[must_use]
    pub fn audit() -> SemanticCoverageReport {
        SemanticCoverageReport::from_capabilities(graph_capabilities())
    }
}

/// Occurrence and provenance counters for one node or edge kind.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticObservedKindCoverage {
    total: usize,
    with_provenance: usize,
    without_provenance: usize,
}

impl SemanticObservedKindCoverage {
    fn record(&mut self, has_provenance: bool) {
        self.total += 1;
        if has_provenance {
            self.with_provenance += 1;
        } else {
            self.without_provenance += 1;
        }
    }

    /// Returns the observed fact count.
    #[must_use]
    pub const fn total(self) -> usize {
        self.total
    }

    /// Returns the count with provenance.
    #[must_use]
    pub const fn with_provenance(self) -> usize {
        self.with_provenance
    }

    /// Returns the count without provenance.
    #[must_use]
    pub const fn without_provenance(self) -> usize {
        self.without_provenance
    }
}

/// Deterministic observed coverage for one built semantic graph.
///
/// This report records occurrence only. A kind absent from this snapshot is not
/// classified as unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticObservedCoverage {
    nodes: BTreeMap<NodeKind, SemanticObservedKindCoverage>,
    edges: BTreeMap<EdgeKind, SemanticObservedKindCoverage>,
}

impl SemanticObservedCoverage {
    /// Builds observed coverage without reading source files or mutating `graph`.
    #[must_use]
    pub fn for_graph(graph: &SemanticGraph) -> Self {
        let mut nodes = BTreeMap::<NodeKind, SemanticObservedKindCoverage>::new();
        let mut edges = BTreeMap::<EdgeKind, SemanticObservedKindCoverage>::new();

        for node in graph.nodes() {
            nodes
                .entry(node.kind())
                .or_default()
                .record(!node.provenance().is_empty());
        }
        for edge in graph.edges() {
            edges
                .entry(edge.kind())
                .or_default()
                .record(!edge.provenance().is_empty());
        }

        Self { nodes, edges }
    }

    /// Returns observed node kinds and provenance counters.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeMap<NodeKind, SemanticObservedKindCoverage> {
        &self.nodes
    }

    /// Returns observed edge kinds and provenance counters.
    #[must_use]
    pub const fn edges(&self) -> &BTreeMap<EdgeKind, SemanticObservedKindCoverage> {
        &self.edges
    }

    /// Returns the total observed node count.
    #[must_use]
    pub fn total_nodes(&self) -> usize {
        self.nodes.values().map(|coverage| coverage.total()).sum()
    }

    /// Returns the total observed edge count.
    #[must_use]
    pub fn total_edges(&self) -> usize {
        self.edges.values().map(|coverage| coverage.total()).sum()
    }

    /// Returns observed node kinds absent from a supplied static registry.
    #[must_use]
    pub fn unregistered_node_kinds(&self, registry: &SemanticCoverageReport) -> Vec<NodeKind> {
        self.nodes
            .keys()
            .copied()
            .filter(|kind| {
                registry
                    .capability(SemanticCoverageCapabilityId::SemanticNode(*kind))
                    .is_none()
            })
            .collect()
    }

    /// Returns observed edge kinds absent from a supplied static registry.
    #[must_use]
    pub fn unregistered_edge_kinds(&self, registry: &SemanticCoverageReport) -> Vec<EdgeKind> {
        self.edges
            .keys()
            .copied()
            .filter(|kind| {
                registry
                    .capability(SemanticCoverageCapabilityId::SemanticEdge(*kind))
                    .is_none()
            })
            .collect()
    }
}

fn graph_capabilities() -> Vec<SemanticCoverageCapability> {
    let mut capabilities = Vec::new();
    capabilities.extend(all_node_kinds().into_iter().map(graph_node_capability));
    capabilities.extend(all_edge_kinds().into_iter().map(graph_edge_capability));
    capabilities.extend(
        all_validation_codes()
            .into_iter()
            .map(validation_capability),
    );
    capabilities.extend(all_query_capabilities().into_iter().map(query_capability));
    capabilities.extend(all_edge_kinds().into_iter().map(impact_capability));
    capabilities.extend(
        all_provenance_capabilities()
            .into_iter()
            .map(provenance_capability),
    );
    capabilities
}

fn graph_node_capability(kind: NodeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    let mut required = vec![
        Evidence::Modeled,
        Evidence::NodeKindDeclared,
        Evidence::StableIdentityAssigned,
        Evidence::ProvenanceAttached,
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
    ];
    if matches!(kind, NodeKind::Attribute | NodeKind::TabularSection) {
        required.push(Evidence::SemanticPayloadPreserved);
    }
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticNode(kind),
        format!("{} semantic node", node_kind_code(kind)),
        SemanticCoverageStatus::Supported,
        required.clone(),
        required,
    )
    .with_node_kind(kind)
    .with_representative_test(graph_node_representative_test(kind))
}

const fn graph_node_representative_test(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Attribute | NodeKind::TabularSection => {
            "oneagent_graph::node::tests::accepts_member_payload_for_attribute_and_tabular_section"
        }
        _ => "oneagent_graph::coverage",
    }
}

fn graph_edge_capability(kind: EdgeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    let required = [
        Evidence::Modeled,
        Evidence::EdgeKindDeclared,
        Evidence::ProvenanceAttached,
        Evidence::ValidationRuleExists,
        Evidence::QuerySupportExists,
        Evidence::PositiveTestExists,
    ];
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::SemanticEdge(kind),
        format!("{} semantic edge", edge_kind_code(kind)),
        SemanticCoverageStatus::Supported,
        required,
        required,
    )
    .with_edge_kind(kind)
    .with_representative_test("oneagent_graph::coverage")
}

fn validation_capability(code: SemanticGraphValidationCode) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::ValidationRule(code),
        format!("{} validation rule", code.as_str()),
        SemanticCoverageStatus::Supported,
        [
            Evidence::Modeled,
            Evidence::ValidationRuleExists,
            Evidence::PositiveTestExists,
            Evidence::NegativeTestExists,
        ],
        [
            Evidence::Modeled,
            Evidence::ValidationRuleExists,
            Evidence::PositiveTestExists,
            Evidence::NegativeTestExists,
        ],
    )
    .with_representative_test("oneagent_graph::validation")
}

fn query_capability(kind: SemanticQueryCapability) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::QueryCapability(kind),
        format!("{} query capability", query_kind_code(kind)),
        SemanticCoverageStatus::Supported,
        [
            Evidence::Modeled,
            Evidence::QuerySupportExists,
            Evidence::PositiveTestExists,
            Evidence::NegativeTestExists,
        ],
        [
            Evidence::Modeled,
            Evidence::QuerySupportExists,
            Evidence::PositiveTestExists,
        ],
    )
    .with_representative_test("oneagent_graph::query")
}

fn impact_capability(kind: EdgeKind) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    let propagates = matches!(
        kind,
        EdgeKind::Contains
            | EdgeKind::Calls
            | EdgeKind::References
            | EdgeKind::Reads
            | EdgeKind::Writes
            | EdgeKind::DependsOn
            | EdgeKind::Opens
    );
    if propagates {
        SemanticCoverageCapability::new(
            SemanticCoverageCapabilityId::ImpactPropagation(kind),
            format!("{} impact propagation", edge_kind_code(kind)),
            SemanticCoverageStatus::Supported,
            [
                Evidence::ImpactPropagationExists,
                Evidence::PositiveTestExists,
            ],
            [
                Evidence::ImpactPropagationExists,
                Evidence::PositiveTestExists,
            ],
        )
        .with_edge_kind(kind)
        .with_representative_test("oneagent_graph::impact")
    } else {
        SemanticCoverageCapability::new(
            SemanticCoverageCapabilityId::ImpactPropagation(kind),
            format!("{} impact propagation", edge_kind_code(kind)),
            SemanticCoverageStatus::NotApplicable,
            [],
            [],
        )
        .with_edge_kind(kind)
        .with_note("The first impact policy intentionally excludes this edge semantic")
    }
}

fn provenance_capability(kind: SemanticProvenanceCapability) -> SemanticCoverageCapability {
    use SemanticCoverageEvidence as Evidence;
    let representative_test = if kind == SemanticProvenanceCapability::ReferenceRequest {
        "oneagent_graph::reference_request_build"
    } else {
        "oneagent_graph::provenance"
    };
    SemanticCoverageCapability::new(
        SemanticCoverageCapabilityId::ProvenanceSource(kind),
        format!("{} provenance", provenance_kind_code(kind)),
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
    )
    .with_representative_test(representative_test)
}

fn capability_is_consistent(capability: &SemanticCoverageCapability) -> bool {
    let missing = capability.missing_evidence();
    let identity_is_consistent = match capability.id() {
        SemanticCoverageCapabilityId::MetadataEntity(kind) => {
            capability.source_metadata_kind() == Some(kind)
                && capability.related_node_kind() == Some(NodeKind::Metadata(kind))
        }
        SemanticCoverageCapabilityId::SemanticNode(kind) => {
            capability.related_node_kind() == Some(kind) && capability.related_edge_kind().is_none()
        }
        SemanticCoverageCapabilityId::OwnershipRelation(kind) => {
            capability.related_node_kind() == Some(kind)
                && capability.related_edge_kind() == Some(EdgeKind::Contains)
        }
        SemanticCoverageCapabilityId::MetadataReference(_) => {
            capability.related_edge_kind().is_some()
        }
        SemanticCoverageCapabilityId::SemanticEdge(kind)
        | SemanticCoverageCapabilityId::ImpactPropagation(kind) => {
            capability.related_edge_kind() == Some(kind)
        }
        SemanticCoverageCapabilityId::ProvenanceSource(_)
        | SemanticCoverageCapabilityId::ValidationRule(_)
        | SemanticCoverageCapabilityId::QueryCapability(_) => true,
    };
    let status_is_consistent = match capability.status() {
        SemanticCoverageStatus::Supported => missing.is_empty(),
        SemanticCoverageStatus::PartiallySupported => {
            !capability.evidence().is_empty() && !missing.is_empty()
        }
        SemanticCoverageStatus::Unsupported => {
            !capability
                .evidence()
                .contains(&SemanticCoverageEvidence::NodeEmitted)
                && !capability
                    .evidence()
                    .contains(&SemanticCoverageEvidence::SemanticEdgeEmitted)
        }
        SemanticCoverageStatus::NotApplicable => capability.required_evidence().is_empty(),
        SemanticCoverageStatus::DeclaredOnly => {
            capability
                .evidence()
                .contains(&SemanticCoverageEvidence::NodeKindDeclared)
                || capability
                    .evidence()
                    .contains(&SemanticCoverageEvidence::EdgeKindDeclared)
        }
    };

    identity_is_consistent && status_is_consistent
}

fn gap_priority(
    capability: &SemanticCoverageCapability,
    missing: &BTreeSet<SemanticCoverageEvidence>,
) -> SemanticCoverageGapPriority {
    use SemanticCoverageCategory as Category;
    use SemanticCoverageEvidence as Evidence;
    use SemanticCoverageGapPriority as Priority;

    if capability.id()
        == SemanticCoverageCapabilityId::MetadataReference(SemanticReferenceCapability::BslCall)
        && missing.contains(&Evidence::DiagnosticEmitted)
    {
        return Priority::Critical;
    }

    if missing.contains(&Evidence::ProvenanceAttached)
        && (capability.evidence().contains(&Evidence::NodeEmitted)
            || capability
                .evidence()
                .contains(&Evidence::SemanticEdgeEmitted))
    {
        return Priority::Critical;
    }

    match (capability.category(), capability.status()) {
        (
            Category::MetadataEntity
            | Category::SemanticNode
            | Category::OwnershipRelation
            | Category::MetadataReference
            | Category::SemanticEdge,
            SemanticCoverageStatus::Unsupported | SemanticCoverageStatus::DeclaredOnly,
        ) => Priority::High,
        (_, SemanticCoverageStatus::PartiallySupported) => Priority::Medium,
        _ if missing.contains(&Evidence::PositiveTestExists)
            || missing.contains(&Evidence::NegativeTestExists) =>
        {
            Priority::Medium
        }
        _ => Priority::Low,
    }
}

fn suggested_next_action(
    capability: &SemanticCoverageCapability,
    missing: &BTreeSet<SemanticCoverageEvidence>,
) -> String {
    use SemanticCoverageEvidence as Evidence;
    if missing.contains(&Evidence::NodeEmitted) {
        return "Emit the typed node in its owning adapter and add representative integration coverage"
            .to_owned();
    }
    if missing.contains(&Evidence::OwnershipEdgeEmitted) {
        return "Emit and validate the owner-child edge with provenance".to_owned();
    }
    if missing.contains(&Evidence::ReferenceResolved)
        || missing.contains(&Evidence::SemanticEdgeEmitted)
    {
        return "Complete typed resolution and emit the resulting semantic edge".to_owned();
    }
    if missing.contains(&Evidence::ProvenanceAttached) {
        return "Attach provenance at the fact creation site and add regression coverage"
            .to_owned();
    }
    if missing.contains(&Evidence::PositiveTestExists)
        || missing.contains(&Evidence::NegativeTestExists)
    {
        return "Add the missing representative positive or negative test".to_owned();
    }
    format!("Complete missing evidence for {}", capability.stable_id())
}

const fn status_code(status: SemanticCoverageStatus) -> &'static str {
    match status {
        SemanticCoverageStatus::Supported => "supported",
        SemanticCoverageStatus::PartiallySupported => "partially_supported",
        SemanticCoverageStatus::Unsupported => "unsupported",
        SemanticCoverageStatus::NotApplicable => "not_applicable",
        SemanticCoverageStatus::DeclaredOnly => "declared_only",
    }
}

fn all_node_kinds() -> Vec<NodeKind> {
    let mut kinds = all_metadata_kinds()
        .into_iter()
        .map(NodeKind::Metadata)
        .collect::<Vec<_>>();
    kinds.extend([
        NodeKind::Module,
        NodeKind::Procedure,
        NodeKind::Function,
        NodeKind::Query,
        NodeKind::Form,
        NodeKind::Command,
        NodeKind::Attribute,
        NodeKind::StandardAttribute,
        NodeKind::TabularSection,
        NodeKind::Dimension,
        NodeKind::Resource,
        NodeKind::Measure,
        NodeKind::Role,
        NodeKind::AccessRight,
        NodeKind::Subsystem,
        NodeKind::Unknown,
    ]);
    kinds
}

/// Returns every currently declared node kind for registry consistency checks.
#[must_use]
pub fn semantic_coverage_node_kinds() -> Vec<NodeKind> {
    all_node_kinds()
}

/// Returns every currently declared edge kind for registry consistency checks.
#[must_use]
pub const fn semantic_coverage_edge_kinds() -> [EdgeKind; 11] {
    all_edge_kinds()
}

const fn all_edge_kinds() -> [EdgeKind; 11] {
    [
        EdgeKind::Contains,
        EdgeKind::Calls,
        EdgeKind::References,
        EdgeKind::Reads,
        EdgeKind::Writes,
        EdgeKind::Grants,
        EdgeKind::Includes,
        EdgeKind::Extends,
        EdgeKind::DependsOn,
        EdgeKind::Opens,
        EdgeKind::Triggers,
    ]
}

const fn all_validation_codes() -> [SemanticGraphValidationCode; 12] {
    [
        SemanticGraphValidationCode::MissingSource,
        SemanticGraphValidationCode::MissingTarget,
        SemanticGraphValidationCode::InvalidEdgeEndpoints,
        SemanticGraphValidationCode::InvalidOwner,
        SemanticGraphValidationCode::MultipleOwners,
        SemanticGraphValidationCode::ForbiddenSelfLoop,
        SemanticGraphValidationCode::Cycle,
        SemanticGraphValidationCode::MissingNodeProvenance,
        SemanticGraphValidationCode::MissingEdgeProvenance,
        SemanticGraphValidationCode::InconsistentResolutionStatistics,
        SemanticGraphValidationCode::InconsistentDiagnosticStatistics,
        SemanticGraphValidationCode::InconsistentReport,
    ]
}

const fn all_query_capabilities() -> [SemanticQueryCapability; 6] {
    [
        SemanticQueryCapability::NodeLookup,
        SemanticQueryCapability::NameAndKindLookup,
        SemanticQueryCapability::OwnershipNavigation,
        SemanticQueryCapability::EdgeNavigation,
        SemanticQueryCapability::DependencyNavigation,
        SemanticQueryCapability::Traversal,
    ]
}

const fn all_provenance_capabilities() -> [SemanticProvenanceCapability; 8] {
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

const fn all_metadata_kinds() -> [MetadataKind; 24] {
    [
        MetadataKind::Configuration,
        MetadataKind::Subsystem,
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::Enumeration,
        MetadataKind::CommonModule,
        MetadataKind::Report,
        MetadataKind::DataProcessor,
        MetadataKind::InformationRegister,
        MetadataKind::AccumulationRegister,
        MetadataKind::AccountingRegister,
        MetadataKind::CalculationRegister,
        MetadataKind::BusinessProcess,
        MetadataKind::Task,
        MetadataKind::Role,
        MetadataKind::CommonForm,
        MetadataKind::Form,
        MetadataKind::Command,
        MetadataKind::Template,
        MetadataKind::HttpService,
        MetadataKind::WebService,
        MetadataKind::XdtoPackage,
        MetadataKind::EventSubscription,
        MetadataKind::Unknown,
    ]
}

const fn node_kind_code(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Metadata(kind) => kind.as_str(),
        NodeKind::Module => "module",
        NodeKind::Procedure => "procedure",
        NodeKind::Function => "function",
        NodeKind::Query => "query",
        NodeKind::Form => "form",
        NodeKind::Command => "command",
        NodeKind::Attribute => "attribute",
        NodeKind::StandardAttribute => "standard_attribute",
        NodeKind::TabularSection => "tabular_section",
        NodeKind::Dimension => "dimension",
        NodeKind::Resource => "resource",
        NodeKind::Measure => "measure",
        NodeKind::Role => "role",
        NodeKind::AccessRight => "access_right",
        NodeKind::Subsystem => "subsystem",
        NodeKind::Unknown => "unknown",
    }
}

const fn edge_kind_code(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Calls => "calls",
        EdgeKind::References => "references",
        EdgeKind::Reads => "reads",
        EdgeKind::Writes => "writes",
        EdgeKind::Grants => "grants",
        EdgeKind::Includes => "includes",
        EdgeKind::Extends => "extends",
        EdgeKind::DependsOn => "depends_on",
        EdgeKind::Opens => "opens",
        EdgeKind::Triggers => "triggers",
    }
}

const fn provenance_kind_code(kind: SemanticProvenanceCapability) -> &'static str {
    match kind {
        SemanticProvenanceCapability::MetadataObjectNode => "metadata_object_node",
        SemanticProvenanceCapability::MetadataChildNode => "metadata_child_node",
        SemanticProvenanceCapability::ModuleNode => "module_node",
        SemanticProvenanceCapability::SymbolNode => "symbol_node",
        SemanticProvenanceCapability::OwnershipEdge => "ownership_edge",
        SemanticProvenanceCapability::ResolvedReferenceEdge => "resolved_reference_edge",
        SemanticProvenanceCapability::Diagnostic => "diagnostic",
        SemanticProvenanceCapability::ReferenceRequest => "reference_request",
    }
}

const fn query_kind_code(kind: SemanticQueryCapability) -> &'static str {
    match kind {
        SemanticQueryCapability::NodeLookup => "node_lookup",
        SemanticQueryCapability::NameAndKindLookup => "name_and_kind_lookup",
        SemanticQueryCapability::OwnershipNavigation => "ownership_navigation",
        SemanticQueryCapability::EdgeNavigation => "edge_navigation",
        SemanticQueryCapability::DependencyNavigation => "dependency_navigation",
        SemanticQueryCapability::Traversal => "traversal",
    }
}
