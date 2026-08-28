//! Source-independent diagnostic identity, findings, policy, and reports.
//!
//! Graph remains authoritative for semantic diagnostics, validation issues,
//! provenance, and locations. This module provides bounded normalized values
//! without parsing source or executing graph validation.

mod engine;

pub use engine::DiagnosticEngine;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use oneagent_common::EntityId;
use oneagent_graph::{
    EdgeId, EdgeKind, NodeKind, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticDiagnosticSeverity, SemanticGraphValidationCode, SemanticGraphValidationIssue,
    SemanticGraphValidationIssueKind, SemanticGraphValidationSeverity, SemanticReference,
    SemanticReferenceRequestId,
};

/// Maximum number of semantic diagnostics accepted by one engine evaluation.
pub const MAX_SEMANTIC_DIAGNOSTICS: usize = 65_536;
/// Maximum number of validation issues accepted by one engine evaluation.
pub const MAX_VALIDATION_ISSUES: usize = 65_536;
/// Maximum number of findings in one complete report.
pub const MAX_DIAGNOSTIC_FINDINGS: usize = 65_536;
/// Maximum number of exact identities in one suppression policy.
pub const MAX_DIAGNOSTIC_SUPPRESSIONS: usize = 4_096;
/// Maximum UTF-8 byte length of one diagnostic message.
pub const MAX_DIAGNOSTIC_MESSAGE_BYTES: usize = 4_096;
/// Maximum number of node anchors retained by one finding.
pub const MAX_DIAGNOSTIC_NODE_ANCHORS: usize = 256;
/// Maximum number of provenance records observed by one finding.
pub const MAX_DIAGNOSTIC_PROVENANCE_RECORDS: usize = 256;

/// Closed diagnostic evidence family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticFamily {
    /// Recoverable semantic diagnostic evidence.
    Semantic,
    /// Semantic graph validation evidence.
    Validation,
}

impl DiagnosticFamily {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Validation => "validation",
        }
    }
}

/// Normalized diagnostic severity.
///
/// Declaration order intentionally places errors before warnings in reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    /// A mandatory invariant or semantic input problem.
    Error,
    /// A non-fatal problem that remains visible to consumers.
    Warning,
}

impl DiagnosticSeverity {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// Source-independent reporting category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticCategory {
    /// Query-language or Data Composition source evidence.
    Source,
    /// Semantic reference, ownership, or graph-schema evidence.
    Semantic,
    /// Graph storage or endpoint integrity evidence.
    Structural,
    /// Missing or degraded provenance evidence.
    Provenance,
    /// Build-result aggregate consistency evidence.
    BuildConsistency,
}

impl DiagnosticCategory {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Semantic => "semantic",
            Self::Structural => "structural",
            Self::Provenance => "provenance",
            Self::BuildConsistency => "build_consistency",
        }
    }
}

/// Result of exact-identity suppression evaluation.
///
/// Declaration order intentionally places active findings before suppressed
/// findings in reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticDisposition {
    /// The finding is visible to default consumers.
    Active,
    /// The finding matched an exact identity in the supplied policy.
    Suppressed,
}

impl DiagnosticDisposition {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suppressed => "suppressed",
        }
    }
}

/// Original typed diagnostic code, tagged by evidence family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticCode {
    /// Recoverable semantic diagnostic code.
    Semantic(SemanticDiagnosticCode),
    /// Semantic graph validation code.
    Validation(SemanticGraphValidationCode),
}

impl DiagnosticCode {
    /// Returns the original stable Graph-owned code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Semantic(code) => code.as_str(),
            Self::Validation(code) => code.as_str(),
        }
    }
}

/// Original typed diagnostic kind, tagged by evidence family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    /// Recoverable semantic diagnostic kind.
    Semantic(SemanticDiagnosticKind),
    /// Semantic graph validation issue kind.
    Validation(SemanticGraphValidationIssueKind),
}

impl DiagnosticKind {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Semantic(kind) => semantic_kind_str(kind),
            Self::Validation(kind) => validation_kind_str(kind),
        }
    }
}

/// Stable typed identity of one normalized diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticIdentity {
    /// Identity of recoverable semantic evidence.
    Semantic {
        /// Stable Graph-owned diagnostic code.
        code: SemanticDiagnosticCode,
        /// Stable Graph-owned diagnostic kind.
        kind: SemanticDiagnosticKind,
        /// Optional canonical source-node anchor.
        source_node: Option<EntityId>,
        /// Exact Graph-owned semantic reference.
        reference: SemanticReference,
    },
    /// Identity of semantic graph validation evidence.
    Validation {
        /// Stable Graph-owned validation code.
        code: SemanticGraphValidationCode,
        /// Stable Graph-owned validation issue kind.
        kind: SemanticGraphValidationIssueKind,
        /// Canonical validation node identifiers.
        nodes: Vec<EntityId>,
        /// Optional canonical edge identifier.
        edge_id: Option<EdgeId>,
        /// Optional canonical reference-request identifier.
        reference_request_id: Option<SemanticReferenceRequestId>,
        /// Optional related edge kind.
        edge_kind: Option<EdgeKind>,
        /// Optional related source-node kind.
        source_kind: Option<NodeKind>,
        /// Optional related target-node kind.
        target_kind: Option<NodeKind>,
        /// Stable Graph-owned invariant name.
        invariant: &'static str,
    },
}

impl DiagnosticIdentity {
    /// Creates the exact identity of a semantic diagnostic.
    #[must_use]
    pub fn from_semantic(diagnostic: &SemanticDiagnostic) -> Self {
        Self::Semantic {
            code: diagnostic.code(),
            kind: diagnostic.kind(),
            source_node: diagnostic.source_node().cloned(),
            reference: diagnostic.reference().clone(),
        }
    }

    /// Creates the exact identity of a validation issue.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticErrorKind::TooManyNodeAnchors`] before cloning an
    /// over-bound node collection.
    pub fn from_validation(issue: &SemanticGraphValidationIssue) -> Result<Self, DiagnosticError> {
        validate_count(
            DiagnosticErrorKind::TooManyNodeAnchors,
            issue.nodes().len(),
            MAX_DIAGNOSTIC_NODE_ANCHORS,
        )?;

        Ok(Self::Validation {
            code: issue.code(),
            kind: issue.kind(),
            nodes: issue.nodes().to_vec(),
            edge_id: issue.edge_id().cloned(),
            reference_request_id: issue.reference_request_id().cloned(),
            edge_kind: issue.edge_kind(),
            source_kind: issue.source_kind(),
            target_kind: issue.target_kind(),
            invariant: issue.invariant(),
        })
    }

    /// Returns the tagged evidence family.
    #[must_use]
    pub const fn family(&self) -> DiagnosticFamily {
        match self {
            Self::Semantic { .. } => DiagnosticFamily::Semantic,
            Self::Validation { .. } => DiagnosticFamily::Validation,
        }
    }

    /// Returns the original tagged stable code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        match self {
            Self::Semantic { code, .. } => DiagnosticCode::Semantic(*code),
            Self::Validation { code, .. } => DiagnosticCode::Validation(*code),
        }
    }

    /// Returns the original tagged stable kind.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        match self {
            Self::Semantic { kind, .. } => DiagnosticKind::Semantic(*kind),
            Self::Validation { kind, .. } => DiagnosticKind::Validation(*kind),
        }
    }
}

/// Original immutable evidence retained by a normalized finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticEvidence {
    /// Recoverable semantic diagnostic evidence.
    Semantic(SemanticDiagnostic),
    /// Semantic graph validation evidence.
    Validation(SemanticGraphValidationIssue),
}

/// Exact in-memory identity suppression policy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DiagnosticPolicy {
    suppressed_identities: BTreeSet<DiagnosticIdentity>,
}

impl DiagnosticPolicy {
    /// Validates and creates an exact-identity suppression policy.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticErrorKind::TooManySuppressions`] when the set is
    /// larger than [`MAX_DIAGNOSTIC_SUPPRESSIONS`].
    pub fn new(
        suppressed_identities: BTreeSet<DiagnosticIdentity>,
    ) -> Result<Self, DiagnosticError> {
        validate_count(
            DiagnosticErrorKind::TooManySuppressions,
            suppressed_identities.len(),
            MAX_DIAGNOSTIC_SUPPRESSIONS,
        )?;

        Ok(Self {
            suppressed_identities,
        })
    }

    /// Returns exact suppressed identities in deterministic order.
    #[must_use]
    pub const fn suppressed_identities(&self) -> &BTreeSet<DiagnosticIdentity> {
        &self.suppressed_identities
    }

    /// Returns whether the complete identity is suppressed.
    #[must_use]
    pub fn suppresses(&self, identity: &DiagnosticIdentity) -> bool {
        self.suppressed_identities.contains(identity)
    }

    /// Returns the exact suppression outcome for an identity.
    #[must_use]
    pub fn disposition(&self, identity: &DiagnosticIdentity) -> DiagnosticDisposition {
        if self.suppresses(identity) {
            DiagnosticDisposition::Suppressed
        } else {
            DiagnosticDisposition::Active
        }
    }
}

/// Read-only typed report filter.
///
/// An empty set accepts every value of that dimension.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DiagnosticFilter {
    families: BTreeSet<DiagnosticFamily>,
    severities: BTreeSet<DiagnosticSeverity>,
    categories: BTreeSet<DiagnosticCategory>,
    dispositions: BTreeSet<DiagnosticDisposition>,
}

impl DiagnosticFilter {
    /// Creates a deterministic typed filter.
    #[must_use]
    pub const fn new(
        families: BTreeSet<DiagnosticFamily>,
        severities: BTreeSet<DiagnosticSeverity>,
        categories: BTreeSet<DiagnosticCategory>,
        dispositions: BTreeSet<DiagnosticDisposition>,
    ) -> Self {
        Self {
            families,
            severities,
            categories,
            dispositions,
        }
    }

    /// Returns accepted families; an empty set means every family.
    #[must_use]
    pub const fn families(&self) -> &BTreeSet<DiagnosticFamily> {
        &self.families
    }

    /// Returns accepted severities; an empty set means every severity.
    #[must_use]
    pub const fn severities(&self) -> &BTreeSet<DiagnosticSeverity> {
        &self.severities
    }

    /// Returns accepted categories; an empty set means every category.
    #[must_use]
    pub const fn categories(&self) -> &BTreeSet<DiagnosticCategory> {
        &self.categories
    }

    /// Returns accepted dispositions; an empty set means every disposition.
    #[must_use]
    pub const fn dispositions(&self) -> &BTreeSet<DiagnosticDisposition> {
        &self.dispositions
    }

    /// Returns whether a finding matches every non-empty dimension.
    #[must_use]
    pub fn matches(&self, finding: &DiagnosticFinding) -> bool {
        accepts(&self.families, finding.family())
            && accepts(&self.severities, finding.severity())
            && accepts(&self.categories, finding.category())
            && accepts(&self.dispositions, finding.disposition())
    }
}

/// One bounded normalized diagnostic finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticFinding {
    identity: DiagnosticIdentity,
    family: DiagnosticFamily,
    severity: DiagnosticSeverity,
    category: DiagnosticCategory,
    code: DiagnosticCode,
    kind: DiagnosticKind,
    message: String,
    disposition: DiagnosticDisposition,
    node_anchors: Vec<EntityId>,
    related_nodes: Vec<EntityId>,
    edge_id: Option<EdgeId>,
    reference_request_id: Option<SemanticReferenceRequestId>,
    provenance_count: usize,
    evidence: DiagnosticEvidence,
}

impl DiagnosticFinding {
    /// Validates and normalizes one semantic diagnostic under `policy`.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed error before cloning an over-bound message,
    /// node-anchor collection, or provenance collection.
    pub fn from_semantic(
        diagnostic: &SemanticDiagnostic,
        policy: &DiagnosticPolicy,
    ) -> Result<Self, DiagnosticError> {
        validate_message(diagnostic.message())?;
        let anchor_count = usize::from(diagnostic.source_node().is_some())
            .checked_add(diagnostic.candidates().len())
            .ok_or_else(|| {
                DiagnosticError::bounded(
                    DiagnosticErrorKind::TooManyNodeAnchors,
                    usize::MAX,
                    MAX_DIAGNOSTIC_NODE_ANCHORS,
                )
            })?;
        validate_count(
            DiagnosticErrorKind::TooManyNodeAnchors,
            anchor_count,
            MAX_DIAGNOSTIC_NODE_ANCHORS,
        )?;
        validate_count(
            DiagnosticErrorKind::TooManyProvenanceRecords,
            diagnostic.provenance().len(),
            MAX_DIAGNOSTIC_PROVENANCE_RECORDS,
        )?;

        let identity = DiagnosticIdentity::from_semantic(diagnostic);
        let node_anchors = diagnostic.source_node().cloned().into_iter().collect();

        Ok(Self {
            family: DiagnosticFamily::Semantic,
            severity: semantic_severity(diagnostic.severity()),
            category: semantic_category(diagnostic.code()),
            code: DiagnosticCode::Semantic(diagnostic.code()),
            kind: DiagnosticKind::Semantic(diagnostic.kind()),
            message: diagnostic.message().to_owned(),
            disposition: policy.disposition(&identity),
            node_anchors,
            related_nodes: diagnostic.candidates().to_vec(),
            edge_id: None,
            reference_request_id: None,
            provenance_count: diagnostic.provenance().len(),
            evidence: DiagnosticEvidence::Semantic(diagnostic.clone()),
            identity,
        })
    }

    /// Validates and normalizes one graph validation issue under `policy`.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed error before cloning an over-bound message,
    /// node-anchor collection, or provenance collection.
    pub fn from_validation(
        issue: &SemanticGraphValidationIssue,
        policy: &DiagnosticPolicy,
    ) -> Result<Self, DiagnosticError> {
        validate_message(issue.message())?;
        validate_count(
            DiagnosticErrorKind::TooManyNodeAnchors,
            issue.nodes().len(),
            MAX_DIAGNOSTIC_NODE_ANCHORS,
        )?;
        validate_count(
            DiagnosticErrorKind::TooManyProvenanceRecords,
            issue.provenance().len(),
            MAX_DIAGNOSTIC_PROVENANCE_RECORDS,
        )?;

        let identity = DiagnosticIdentity::from_validation(issue)?;

        Ok(Self {
            family: DiagnosticFamily::Validation,
            severity: validation_severity(issue.severity()),
            category: validation_category(issue.kind()),
            code: DiagnosticCode::Validation(issue.code()),
            kind: DiagnosticKind::Validation(issue.kind()),
            message: issue.message().to_owned(),
            disposition: policy.disposition(&identity),
            node_anchors: issue.nodes().to_vec(),
            related_nodes: Vec::new(),
            edge_id: issue.edge_id().cloned(),
            reference_request_id: issue.reference_request_id().cloned(),
            provenance_count: issue.provenance().len(),
            evidence: DiagnosticEvidence::Validation(issue.clone()),
            identity,
        })
    }

    /// Returns the stable typed identity.
    #[must_use]
    pub const fn identity(&self) -> &DiagnosticIdentity {
        &self.identity
    }

    /// Returns the evidence family.
    #[must_use]
    pub const fn family(&self) -> DiagnosticFamily {
        self.family
    }

    /// Returns normalized severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the source-independent reporting category.
    #[must_use]
    pub const fn category(&self) -> DiagnosticCategory {
        self.category
    }

    /// Returns the original tagged stable code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the original tagged stable kind.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    /// Returns the bounded original message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the exact suppression outcome.
    #[must_use]
    pub const fn disposition(&self) -> DiagnosticDisposition {
        self.disposition
    }

    /// Returns canonical primary node anchors.
    #[must_use]
    pub fn node_anchors(&self) -> &[EntityId] {
        &self.node_anchors
    }

    /// Returns related semantic candidate nodes.
    #[must_use]
    pub fn related_nodes(&self) -> &[EntityId] {
        &self.related_nodes
    }

    /// Returns the optional canonical edge anchor.
    #[must_use]
    pub const fn edge_id(&self) -> Option<&EdgeId> {
        self.edge_id.as_ref()
    }

    /// Returns the optional canonical reference-request anchor.
    #[must_use]
    pub const fn reference_request_id(&self) -> Option<&SemanticReferenceRequestId> {
        self.reference_request_id.as_ref()
    }

    /// Returns the number of observed provenance records.
    #[must_use]
    pub const fn provenance_count(&self) -> usize {
        self.provenance_count
    }

    /// Returns the original typed immutable evidence.
    #[must_use]
    pub const fn evidence(&self) -> &DiagnosticEvidence {
        &self.evidence
    }
}

impl PartialOrd for DiagnosticFinding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DiagnosticFinding {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.disposition,
            self.severity,
            self.category,
            self.family,
            &self.identity,
        )
            .cmp(&(
                other.disposition,
                other.severity,
                other.category,
                other.family,
                &other.identity,
            ))
            .then_with(|| {
                (
                    &self.code,
                    &self.kind,
                    &self.message,
                    &self.node_anchors,
                    &self.related_nodes,
                    &self.edge_id,
                    &self.reference_request_id,
                    self.provenance_count,
                    &self.evidence,
                )
                    .cmp(&(
                        &other.code,
                        &other.kind,
                        &other.message,
                        &other.node_anchors,
                        &other.related_nodes,
                        &other.edge_id,
                        &other.reference_request_id,
                        other.provenance_count,
                        &other.evidence,
                    ))
            })
    }
}

/// Reconciled counters for one complete diagnostic report.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DiagnosticSummary {
    total: usize,
    active: usize,
    suppressed: usize,
    by_family: BTreeMap<DiagnosticFamily, usize>,
    by_severity: BTreeMap<DiagnosticSeverity, usize>,
    by_category: BTreeMap<DiagnosticCategory, usize>,
    active_by_code: BTreeMap<String, usize>,
    suppressed_by_code: BTreeMap<String, usize>,
}

impl DiagnosticSummary {
    fn from_findings(findings: &[DiagnosticFinding]) -> Result<Self, DiagnosticError> {
        let mut summary = Self::default();

        for finding in findings {
            increment(&mut summary.total)?;
            increment_map(&mut summary.by_family, finding.family())?;
            increment_map(&mut summary.by_severity, finding.severity())?;
            increment_map(&mut summary.by_category, finding.category())?;

            match finding.disposition() {
                DiagnosticDisposition::Active => {
                    increment(&mut summary.active)?;
                    increment_map(
                        &mut summary.active_by_code,
                        finding.code().as_str().to_owned(),
                    )?;
                }
                DiagnosticDisposition::Suppressed => {
                    increment(&mut summary.suppressed)?;
                    increment_map(
                        &mut summary.suppressed_by_code,
                        finding.code().as_str().to_owned(),
                    )?;
                }
            }
        }

        debug_assert_eq!(
            summary.active.checked_add(summary.suppressed),
            Some(summary.total)
        );
        debug_assert_eq!(summary.by_family.values().sum::<usize>(), summary.total);
        debug_assert_eq!(summary.by_severity.values().sum::<usize>(), summary.total);
        debug_assert_eq!(summary.by_category.values().sum::<usize>(), summary.total);
        Ok(summary)
    }

    /// Returns the total number of findings.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns the number of active findings.
    #[must_use]
    pub const fn active(&self) -> usize {
        self.active
    }

    /// Returns the number of suppressed findings.
    #[must_use]
    pub const fn suppressed(&self) -> usize {
        self.suppressed
    }

    /// Returns total counts by family.
    #[must_use]
    pub const fn by_family(&self) -> &BTreeMap<DiagnosticFamily, usize> {
        &self.by_family
    }

    /// Returns total counts by normalized severity.
    #[must_use]
    pub const fn by_severity(&self) -> &BTreeMap<DiagnosticSeverity, usize> {
        &self.by_severity
    }

    /// Returns total counts by reporting category.
    #[must_use]
    pub const fn by_category(&self) -> &BTreeMap<DiagnosticCategory, usize> {
        &self.by_category
    }

    /// Returns active counts by stable code string.
    #[must_use]
    pub const fn active_by_code(&self) -> &BTreeMap<String, usize> {
        &self.active_by_code
    }

    /// Returns suppressed counts by stable code string.
    #[must_use]
    pub const fn suppressed_by_code(&self) -> &BTreeMap<String, usize> {
        &self.suppressed_by_code
    }
}

/// Complete ordered diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    findings: Vec<DiagnosticFinding>,
    summary: DiagnosticSummary,
}

impl DiagnosticReport {
    /// Creates an empty complete report.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            findings: Vec::new(),
            summary: DiagnosticSummary::default(),
        }
    }

    /// Validates and creates a complete report from normalized findings.
    ///
    /// Exact duplicate findings collapse to one value. Equal identities with
    /// different observable content fail closed.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticErrorKind::TooManyFindings`] for an over-bound
    /// normalized result or [`DiagnosticErrorKind::ConflictingEvidence`] for
    /// an identity collision.
    pub fn new(findings: Vec<DiagnosticFinding>) -> Result<Self, DiagnosticError> {
        let mut normalized = BTreeMap::new();
        for finding in findings {
            match normalized.entry(finding.identity().clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(finding);
                }
                std::collections::btree_map::Entry::Occupied(entry) if entry.get() == &finding => {}
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(DiagnosticError::conflicting_evidence());
                }
            }
        }
        validate_count(
            DiagnosticErrorKind::TooManyFindings,
            normalized.len(),
            MAX_DIAGNOSTIC_FINDINGS,
        )?;
        let mut findings = normalized.into_values().collect::<Vec<_>>();
        findings.sort();
        let summary = DiagnosticSummary::from_findings(&findings)?;
        Ok(Self { findings, summary })
    }

    /// Returns complete findings in canonical order.
    #[must_use]
    pub fn findings(&self) -> &[DiagnosticFinding] {
        &self.findings
    }

    /// Returns counters for the complete unfiltered report.
    #[must_use]
    pub const fn summary(&self) -> &DiagnosticSummary {
        &self.summary
    }

    /// Returns findings matching every non-empty filter dimension.
    ///
    /// Filtering preserves complete report order and never changes the report
    /// summary.
    pub fn filtered<'report, 'filter>(
        &'report self,
        filter: &'filter DiagnosticFilter,
    ) -> impl Iterator<Item = &'report DiagnosticFinding> + use<'report, 'filter> {
        self.findings
            .iter()
            .filter(move |finding| filter.matches(finding))
    }
}

impl Default for DiagnosticReport {
    fn default() -> Self {
        Self::empty()
    }
}

/// Closed diagnostic domain failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticErrorKind {
    /// Semantic input count exceeds its accepted bound.
    TooManySemanticDiagnostics,
    /// Validation input count exceeds its accepted bound.
    TooManyValidationIssues,
    /// Normalized report count exceeds its accepted bound.
    TooManyFindings,
    /// Exact suppression identity count exceeds its accepted bound.
    TooManySuppressions,
    /// A diagnostic message exceeds its accepted byte bound.
    MessageTooLarge,
    /// A finding has too many node anchors.
    TooManyNodeAnchors,
    /// A finding observes too many provenance records.
    TooManyProvenanceRecords,
    /// Equal identities carry different observable evidence.
    ConflictingEvidence,
}

/// Bounded redacted diagnostic domain error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticError {
    kind: DiagnosticErrorKind,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl DiagnosticError {
    const fn bounded(kind: DiagnosticErrorKind, actual: usize, maximum: usize) -> Self {
        Self {
            kind,
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    const fn conflicting_evidence() -> Self {
        Self {
            kind: DiagnosticErrorKind::ConflictingEvidence,
            actual: None,
            maximum: None,
        }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> DiagnosticErrorKind {
        self.kind
    }

    /// Returns the rejected count for a bounded-count failure.
    #[must_use]
    pub const fn actual(self) -> Option<usize> {
        self.actual
    }

    /// Returns the accepted maximum for a bounded-count failure.
    #[must_use]
    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

impl Display for DiagnosticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.actual, self.maximum) {
            (Some(actual), Some(maximum)) => write!(
                formatter,
                "diagnostic domain rejected a bounded count: kind={:?}, actual={actual}, maximum={maximum}",
                self.kind
            ),
            _ => write!(
                formatter,
                "diagnostic domain rejected evidence: kind={:?}",
                self.kind
            ),
        }
    }
}

impl std::error::Error for DiagnosticError {}

const fn semantic_severity(severity: SemanticDiagnosticSeverity) -> DiagnosticSeverity {
    match severity {
        SemanticDiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
        SemanticDiagnosticSeverity::Error => DiagnosticSeverity::Error,
    }
}

const fn validation_severity(severity: SemanticGraphValidationSeverity) -> DiagnosticSeverity {
    match severity {
        SemanticGraphValidationSeverity::Error => DiagnosticSeverity::Error,
        SemanticGraphValidationSeverity::Warning => DiagnosticSeverity::Warning,
    }
}

const fn semantic_category(code: SemanticDiagnosticCode) -> DiagnosticCategory {
    match code {
        SemanticDiagnosticCode::QueryLanguageMalformedSyntax
        | SemanticDiagnosticCode::QueryLanguageUnsupportedStructure
        | SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace
        | SemanticDiagnosticCode::QueryLanguageVirtualTableSource
        | SemanticDiagnosticCode::QueryLanguageTemporaryTableSource
        | SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource
        | SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred
        | SemanticDiagnosticCode::DataCompositionFieldFolderDeferred
        | SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType
        | SemanticDiagnosticCode::DataCompositionUnsupportedFieldType => DiagnosticCategory::Source,
        SemanticDiagnosticCode::ReferenceMalformedFormat
        | SemanticDiagnosticCode::ReferenceUnsupportedPrefix
        | SemanticDiagnosticCode::ReferenceUnresolved
        | SemanticDiagnosticCode::ReferenceAmbiguous
        | SemanticDiagnosticCode::ReferenceIncompatibleKind
        | SemanticDiagnosticCode::ReferenceInvalidOwner
        | SemanticDiagnosticCode::DuplicateSemanticEdgeRequest => DiagnosticCategory::Semantic,
    }
}

const fn validation_category(kind: SemanticGraphValidationIssueKind) -> DiagnosticCategory {
    match kind {
        SemanticGraphValidationIssueKind::Structural => DiagnosticCategory::Structural,
        SemanticGraphValidationIssueKind::Semantic => DiagnosticCategory::Semantic,
        SemanticGraphValidationIssueKind::Provenance => DiagnosticCategory::Provenance,
        SemanticGraphValidationIssueKind::BuildConsistency => DiagnosticCategory::BuildConsistency,
    }
}

const fn semantic_kind_str(kind: SemanticDiagnosticKind) -> &'static str {
    match kind {
        SemanticDiagnosticKind::QueryLanguageMalformedSyntax => "query_language_malformed_syntax",
        SemanticDiagnosticKind::QueryLanguageUnsupportedStructure => {
            "query_language_unsupported_structure"
        }
        SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace => {
            "query_language_unsupported_persistent_namespace"
        }
        SemanticDiagnosticKind::QueryLanguageVirtualTableSource => {
            "query_language_virtual_table_source"
        }
        SemanticDiagnosticKind::QueryLanguageTemporaryTableSource => {
            "query_language_temporary_table_source"
        }
        SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource => {
            "query_language_external_or_parameter_data_source"
        }
        SemanticDiagnosticKind::DataCompositionNestedDataSetDeferred => {
            "data_composition_nested_data_set_deferred"
        }
        SemanticDiagnosticKind::DataCompositionFieldFolderDeferred => {
            "data_composition_field_folder_deferred"
        }
        SemanticDiagnosticKind::DataCompositionUnsupportedDataSetType => {
            "data_composition_unsupported_data_set_type"
        }
        SemanticDiagnosticKind::DataCompositionUnsupportedFieldType => {
            "data_composition_unsupported_field_type"
        }
        SemanticDiagnosticKind::MalformedReferenceFormat => "malformed_reference_format",
        SemanticDiagnosticKind::UnsupportedReferencePrefix => "unsupported_reference_prefix",
        SemanticDiagnosticKind::UnresolvedTarget => "unresolved_target",
        SemanticDiagnosticKind::AmbiguousTarget => "ambiguous_target",
        SemanticDiagnosticKind::IncompatibleTargetKind => "incompatible_target_kind",
        SemanticDiagnosticKind::InvalidOwnerReference => "invalid_owner_reference",
        SemanticDiagnosticKind::DuplicateSemanticEdgeRequest => "duplicate_semantic_edge_request",
    }
}

const fn validation_kind_str(kind: SemanticGraphValidationIssueKind) -> &'static str {
    match kind {
        SemanticGraphValidationIssueKind::Structural => "structural",
        SemanticGraphValidationIssueKind::Semantic => "semantic",
        SemanticGraphValidationIssueKind::Provenance => "provenance",
        SemanticGraphValidationIssueKind::BuildConsistency => "build_consistency",
    }
}

fn validate_message(message: &str) -> Result<(), DiagnosticError> {
    validate_count(
        DiagnosticErrorKind::MessageTooLarge,
        message.len(),
        MAX_DIAGNOSTIC_MESSAGE_BYTES,
    )
}

fn validate_count(
    kind: DiagnosticErrorKind,
    actual: usize,
    maximum: usize,
) -> Result<(), DiagnosticError> {
    if actual > maximum {
        Err(DiagnosticError::bounded(kind, actual, maximum))
    } else {
        Ok(())
    }
}

fn accepts<T: Ord + Copy>(accepted: &BTreeSet<T>, value: T) -> bool {
    accepted.is_empty() || accepted.contains(&value)
}

fn increment(value: &mut usize) -> Result<(), DiagnosticError> {
    *value = value.checked_add(1).ok_or_else(|| {
        DiagnosticError::bounded(
            DiagnosticErrorKind::TooManyFindings,
            usize::MAX,
            MAX_DIAGNOSTIC_FINDINGS,
        )
    })?;
    Ok(())
}

fn increment_map<K: Ord>(map: &mut BTreeMap<K, usize>, key: K) -> Result<(), DiagnosticError> {
    increment(map.entry(key).or_default())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        Confidence, FactOrigin, GraphNode, NodeKind, ProducerId, Provenance, ResolutionState,
        SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphValidationCode,
        SemanticGraphValidationIssueKind, SemanticReference,
    };

    use super::{
        DiagnosticCategory, DiagnosticCode, DiagnosticDisposition, DiagnosticError,
        DiagnosticErrorKind, DiagnosticFamily, DiagnosticFilter, DiagnosticFinding,
        DiagnosticIdentity, DiagnosticKind, DiagnosticPolicy, DiagnosticReport, DiagnosticSeverity,
        MAX_DIAGNOSTIC_FINDINGS, MAX_DIAGNOSTIC_MESSAGE_BYTES, MAX_DIAGNOSTIC_NODE_ANCHORS,
        MAX_DIAGNOSTIC_PROVENANCE_RECORDS, MAX_DIAGNOSTIC_SUPPRESSIONS, MAX_SEMANTIC_DIAGNOSTICS,
        MAX_VALIDATION_ISSUES, semantic_category, validate_count, validation_category,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance() -> Provenance {
        Provenance::new(
            Some(id("metadata.source")),
            ProducerId::new("oneagent.analysis.diagnostics.tests"),
            FactOrigin::Resolved,
            Confidence::Exact,
            ResolutionState::Resolved,
        )
    }

    fn semantic(
        code: SemanticDiagnosticCode,
        severity: SemanticDiagnosticSeverity,
        kind: SemanticDiagnosticKind,
        message: impl Into<String>,
        source: &str,
    ) -> SemanticDiagnostic {
        SemanticDiagnostic::new(
            code,
            severity,
            kind,
            message,
            SemanticReference::NodeId("metadata.target".to_owned()),
        )
        .with_source_node(id(source))
    }

    fn validation_issue() -> oneagent_graph::SemanticGraphValidationIssue {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id("metadata.source"),
            name("Source"),
            NodeKind::Module,
        ));
        graph
            .validate()
            .issues()
            .first()
            .expect("missing provenance must produce one issue")
            .clone()
    }

    #[test]
    fn closed_normalized_vocabularies_have_stable_strings_and_order() {
        assert_eq!(DiagnosticFamily::Semantic.as_str(), "semantic");
        assert_eq!(DiagnosticFamily::Validation.as_str(), "validation");
        assert_eq!(DiagnosticSeverity::Error.as_str(), "error");
        assert_eq!(DiagnosticSeverity::Warning.as_str(), "warning");
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
        assert_eq!(DiagnosticCategory::Source.as_str(), "source");
        assert_eq!(DiagnosticCategory::Semantic.as_str(), "semantic");
        assert_eq!(DiagnosticCategory::Structural.as_str(), "structural");
        assert_eq!(DiagnosticCategory::Provenance.as_str(), "provenance");
        assert_eq!(
            DiagnosticCategory::BuildConsistency.as_str(),
            "build_consistency"
        );
        assert_eq!(DiagnosticDisposition::Active.as_str(), "active");
        assert_eq!(DiagnosticDisposition::Suppressed.as_str(), "suppressed");
        assert!(DiagnosticDisposition::Active < DiagnosticDisposition::Suppressed);
    }

    #[test]
    fn every_source_kind_maps_to_its_accepted_category() {
        let semantic_codes = [
            SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace,
            SemanticDiagnosticCode::QueryLanguageVirtualTableSource,
            SemanticDiagnosticCode::QueryLanguageTemporaryTableSource,
            SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource,
            SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
            SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType,
            SemanticDiagnosticCode::DataCompositionUnsupportedFieldType,
        ];
        for code in semantic_codes {
            assert_eq!(semantic_category(code), DiagnosticCategory::Source);
        }

        let semantic_reference_codes = [
            SemanticDiagnosticCode::ReferenceMalformedFormat,
            SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticCode::DuplicateSemanticEdgeRequest,
        ];
        for code in semantic_reference_codes {
            assert_eq!(semantic_category(code), DiagnosticCategory::Semantic);
        }

        assert_eq!(
            validation_category(SemanticGraphValidationIssueKind::Structural),
            DiagnosticCategory::Structural
        );
        assert_eq!(
            validation_category(SemanticGraphValidationIssueKind::Semantic),
            DiagnosticCategory::Semantic
        );
        assert_eq!(
            validation_category(SemanticGraphValidationIssueKind::Provenance),
            DiagnosticCategory::Provenance
        );
        assert_eq!(
            validation_category(SemanticGraphValidationIssueKind::BuildConsistency),
            DiagnosticCategory::BuildConsistency
        );
    }

    #[test]
    fn semantic_source_codes_and_kinds_preserve_stable_projections() {
        let values = [
            (
                SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
                SemanticDiagnosticKind::QueryLanguageMalformedSyntax,
                "query_language.malformed_syntax",
                "query_language_malformed_syntax",
            ),
            (
                SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
                SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
                "query_language.unsupported_structure",
                "query_language_unsupported_structure",
            ),
            (
                SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace,
                SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace,
                "query_language.unsupported_persistent_namespace",
                "query_language_unsupported_persistent_namespace",
            ),
            (
                SemanticDiagnosticCode::QueryLanguageVirtualTableSource,
                SemanticDiagnosticKind::QueryLanguageVirtualTableSource,
                "query_language.virtual_table_source",
                "query_language_virtual_table_source",
            ),
            (
                SemanticDiagnosticCode::QueryLanguageTemporaryTableSource,
                SemanticDiagnosticKind::QueryLanguageTemporaryTableSource,
                "query_language.temporary_table_source",
                "query_language_temporary_table_source",
            ),
            (
                SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource,
                SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource,
                "query_language.external_or_parameter_data_source",
                "query_language_external_or_parameter_data_source",
            ),
            (
                SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
                SemanticDiagnosticKind::DataCompositionNestedDataSetDeferred,
                "data_composition.nested_data_set_deferred",
                "data_composition_nested_data_set_deferred",
            ),
            (
                SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
                SemanticDiagnosticKind::DataCompositionFieldFolderDeferred,
                "data_composition.field_folder_deferred",
                "data_composition_field_folder_deferred",
            ),
            (
                SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType,
                SemanticDiagnosticKind::DataCompositionUnsupportedDataSetType,
                "data_composition.unsupported_data_set_type",
                "data_composition_unsupported_data_set_type",
            ),
            (
                SemanticDiagnosticCode::DataCompositionUnsupportedFieldType,
                SemanticDiagnosticKind::DataCompositionUnsupportedFieldType,
                "data_composition.unsupported_field_type",
                "data_composition_unsupported_field_type",
            ),
        ];

        for (code, kind, expected_code, expected_kind) in values {
            assert_eq!(DiagnosticCode::Semantic(code).as_str(), expected_code);
            assert_eq!(DiagnosticKind::Semantic(kind).as_str(), expected_kind);
        }
    }

    #[test]
    fn semantic_reference_codes_and_kinds_preserve_stable_projections() {
        let values = [
            (
                SemanticDiagnosticCode::ReferenceMalformedFormat,
                SemanticDiagnosticKind::MalformedReferenceFormat,
                "semantic.reference.malformed_format",
                "malformed_reference_format",
            ),
            (
                SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
                SemanticDiagnosticKind::UnsupportedReferencePrefix,
                "semantic.reference.unsupported_prefix",
                "unsupported_reference_prefix",
            ),
            (
                SemanticDiagnosticCode::ReferenceUnresolved,
                SemanticDiagnosticKind::UnresolvedTarget,
                "semantic.reference.unresolved",
                "unresolved_target",
            ),
            (
                SemanticDiagnosticCode::ReferenceAmbiguous,
                SemanticDiagnosticKind::AmbiguousTarget,
                "semantic.reference.ambiguous",
                "ambiguous_target",
            ),
            (
                SemanticDiagnosticCode::ReferenceIncompatibleKind,
                SemanticDiagnosticKind::IncompatibleTargetKind,
                "semantic.reference.incompatible_kind",
                "incompatible_target_kind",
            ),
            (
                SemanticDiagnosticCode::ReferenceInvalidOwner,
                SemanticDiagnosticKind::InvalidOwnerReference,
                "semantic.reference.invalid_owner",
                "invalid_owner_reference",
            ),
            (
                SemanticDiagnosticCode::DuplicateSemanticEdgeRequest,
                SemanticDiagnosticKind::DuplicateSemanticEdgeRequest,
                "semantic.edge.duplicate_request",
                "duplicate_semantic_edge_request",
            ),
        ];

        for (code, kind, expected_code, expected_kind) in values {
            assert_eq!(DiagnosticCode::Semantic(code).as_str(), expected_code);
            assert_eq!(DiagnosticKind::Semantic(kind).as_str(), expected_kind);
        }
    }

    #[test]
    fn all_validation_codes_and_kinds_preserve_stable_projections() {
        let codes = [
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
            SemanticGraphValidationCode::NonTerminalReferenceRequest,
            SemanticGraphValidationCode::MissingReferenceRequestSource,
            SemanticGraphValidationCode::MissingReferenceRequestCandidate,
            SemanticGraphValidationCode::IncompatibleReferenceRequestCandidate,
            SemanticGraphValidationCode::MissingReferenceRequestEdgeProjection,
            SemanticGraphValidationCode::UnexpectedReferenceRequestEdgeProjection,
            SemanticGraphValidationCode::MissingReferenceRequestDiagnosticProjection,
        ];
        for code in codes {
            assert_eq!(DiagnosticCode::Validation(code).as_str(), code.as_str());
        }

        let kinds = [
            (SemanticGraphValidationIssueKind::Structural, "structural"),
            (SemanticGraphValidationIssueKind::Semantic, "semantic"),
            (SemanticGraphValidationIssueKind::Provenance, "provenance"),
            (
                SemanticGraphValidationIssueKind::BuildConsistency,
                "build_consistency",
            ),
        ];
        for (kind, expected) in kinds {
            assert_eq!(DiagnosticKind::Validation(kind).as_str(), expected);
        }
    }

    #[test]
    fn identities_preserve_family_tags_and_exact_fields() {
        let diagnostic = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "unresolved",
            "metadata.source",
        );
        let semantic_identity = DiagnosticIdentity::from_semantic(&diagnostic);
        let validation_identity = DiagnosticIdentity::from_validation(&validation_issue())
            .expect("validation identity must fit bounds");

        assert_eq!(semantic_identity.family(), DiagnosticFamily::Semantic);
        assert_eq!(validation_identity.family(), DiagnosticFamily::Validation);
        assert_ne!(semantic_identity, validation_identity);
        assert_eq!(
            semantic_identity,
            DiagnosticIdentity::from_semantic(&diagnostic)
        );
    }

    #[test]
    fn default_and_exact_suppression_preserve_evidence() {
        let diagnostic = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "unresolved",
            "metadata.source",
        );
        let identity = DiagnosticIdentity::from_semantic(&diagnostic);
        let active = DiagnosticFinding::from_semantic(&diagnostic, &DiagnosticPolicy::default())
            .expect("default finding must be valid");
        let policy = DiagnosticPolicy::new(BTreeSet::from([identity.clone()]))
            .expect("single suppression must be valid");
        let suppressed = DiagnosticFinding::from_semantic(&diagnostic, &policy)
            .expect("suppressed finding must be valid");

        assert_eq!(active.disposition(), DiagnosticDisposition::Active);
        assert_eq!(suppressed.disposition(), DiagnosticDisposition::Suppressed);
        assert_eq!(active.identity(), suppressed.identity());
        assert_eq!(active.evidence(), suppressed.evidence());
        assert!(policy.suppresses(&identity));
    }

    #[test]
    fn suppression_bound_accepts_exact_and_rejects_one_over() {
        let identities = (0..=MAX_DIAGNOSTIC_SUPPRESSIONS)
            .map(|index| {
                DiagnosticIdentity::from_semantic(&semantic(
                    SemanticDiagnosticCode::ReferenceUnresolved,
                    SemanticDiagnosticSeverity::Error,
                    SemanticDiagnosticKind::UnresolvedTarget,
                    "unresolved",
                    &format!("metadata.source.{index}"),
                ))
            })
            .collect::<Vec<_>>();

        assert!(
            DiagnosticPolicy::new(
                identities[..MAX_DIAGNOSTIC_SUPPRESSIONS]
                    .iter()
                    .cloned()
                    .collect()
            )
            .is_ok()
        );
        let error = DiagnosticPolicy::new(identities.into_iter().collect())
            .expect_err("one-over suppression policy must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::TooManySuppressions);
        assert_eq!(error.actual(), Some(MAX_DIAGNOSTIC_SUPPRESSIONS + 1));
    }

    #[test]
    fn message_bound_uses_utf8_bytes_and_does_not_echo_rejected_content() {
        let exact = "x".repeat(MAX_DIAGNOSTIC_MESSAGE_BYTES);
        let over = "s".repeat(MAX_DIAGNOSTIC_MESSAGE_BYTES + 1);
        let exact_diagnostic = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            exact,
            "metadata.source",
        );
        let over_diagnostic = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            over,
            "metadata.source",
        );

        assert!(
            DiagnosticFinding::from_semantic(&exact_diagnostic, &DiagnosticPolicy::default())
                .is_ok()
        );
        let error =
            DiagnosticFinding::from_semantic(&over_diagnostic, &DiagnosticPolicy::default())
                .expect_err("one-over message must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::MessageTooLarge);
        assert_eq!(error.actual(), Some(MAX_DIAGNOSTIC_MESSAGE_BYTES + 1));
        assert!(!error.to_string().contains(&"s".repeat(32)));
        assert!(!format!("{error:?}").contains(&"s".repeat(32)));
    }

    #[test]
    fn node_anchor_bound_accepts_exact_and_rejects_one_over() {
        let candidates = (0..MAX_DIAGNOSTIC_NODE_ANCHORS)
            .map(|index| id(&format!("metadata.candidate.{index}")))
            .collect::<Vec<_>>();
        let exact = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            "ambiguous",
            SemanticReference::NodeId("metadata.target".to_owned()),
        )
        .with_candidates(candidates.clone());
        let over = exact.clone().with_source_node(id("metadata.source"));

        assert!(DiagnosticFinding::from_semantic(&exact, &DiagnosticPolicy::default()).is_ok());
        let error = DiagnosticFinding::from_semantic(&over, &DiagnosticPolicy::default())
            .expect_err("one-over anchor collection must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::TooManyNodeAnchors);
        assert_eq!(error.actual(), Some(MAX_DIAGNOSTIC_NODE_ANCHORS + 1));
    }

    #[test]
    fn provenance_bound_accepts_exact_and_rejects_one_over() {
        let base = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "unresolved",
            "metadata.source",
        );
        let exact = base
            .clone()
            .with_provenance(vec![provenance(); MAX_DIAGNOSTIC_PROVENANCE_RECORDS]);
        let over = base.with_provenance(vec![provenance(); MAX_DIAGNOSTIC_PROVENANCE_RECORDS + 1]);

        assert!(DiagnosticFinding::from_semantic(&exact, &DiagnosticPolicy::default()).is_ok());
        let error = DiagnosticFinding::from_semantic(&over, &DiagnosticPolicy::default())
            .expect_err("one-over provenance collection must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::TooManyProvenanceRecords);
        assert_eq!(error.actual(), Some(MAX_DIAGNOSTIC_PROVENANCE_RECORDS + 1));
    }

    #[test]
    fn accepted_count_bounds_are_exact_and_closed() {
        let bounds = [
            (
                DiagnosticErrorKind::TooManySemanticDiagnostics,
                MAX_SEMANTIC_DIAGNOSTICS,
            ),
            (
                DiagnosticErrorKind::TooManyValidationIssues,
                MAX_VALIDATION_ISSUES,
            ),
            (
                DiagnosticErrorKind::TooManyFindings,
                MAX_DIAGNOSTIC_FINDINGS,
            ),
            (
                DiagnosticErrorKind::TooManySuppressions,
                MAX_DIAGNOSTIC_SUPPRESSIONS,
            ),
            (
                DiagnosticErrorKind::TooManyNodeAnchors,
                MAX_DIAGNOSTIC_NODE_ANCHORS,
            ),
            (
                DiagnosticErrorKind::TooManyProvenanceRecords,
                MAX_DIAGNOSTIC_PROVENANCE_RECORDS,
            ),
        ];

        for (kind, maximum) in bounds {
            assert!(validate_count(kind, maximum, maximum).is_ok());
            let error =
                validate_count(kind, maximum + 1, maximum).expect_err("one-over count must fail");
            assert_eq!(error.kind(), kind);
            assert_eq!(error.actual(), Some(maximum + 1));
            assert_eq!(error.maximum(), Some(maximum));
        }
    }

    #[test]
    fn semantic_and_validation_findings_preserve_owned_anchors() {
        let diagnostic = semantic(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            "ambiguous",
            "metadata.source",
        )
        .with_candidates(vec![id("metadata.candidate")]);
        let semantic_finding =
            DiagnosticFinding::from_semantic(&diagnostic, &DiagnosticPolicy::default())
                .expect("semantic finding must be valid");
        let issue = validation_issue();
        let validation_finding =
            DiagnosticFinding::from_validation(&issue, &DiagnosticPolicy::default())
                .expect("validation finding must be valid");

        assert_eq!(semantic_finding.node_anchors(), &[id("metadata.source")]);
        assert_eq!(
            semantic_finding.related_nodes(),
            &[id("metadata.candidate")]
        );
        assert_eq!(validation_finding.node_anchors(), issue.nodes());
        assert!(validation_finding.related_nodes().is_empty());
        assert_eq!(semantic_finding.provenance_count(), 0);

        let unavailable = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "unresolved",
            SemanticReference::NodeId("metadata.target".to_owned()),
        );
        let unavailable =
            DiagnosticFinding::from_semantic(&unavailable, &DiagnosticPolicy::default())
                .expect("missing source anchor remains a valid explicit absence");
        assert!(unavailable.node_anchors().is_empty());
    }

    #[test]
    fn report_order_summary_filter_and_repetition_are_deterministic() {
        let warning = semantic(
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticSeverity::Warning,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
            "unsupported",
            "metadata.warning",
        );
        let error = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "unresolved",
            "metadata.error",
        );
        let suppressed_identity = DiagnosticIdentity::from_semantic(&warning);
        let policy = DiagnosticPolicy::new(BTreeSet::from([suppressed_identity]))
            .expect("policy must be valid");
        let warning =
            DiagnosticFinding::from_semantic(&warning, &policy).expect("warning must be valid");
        let error = DiagnosticFinding::from_semantic(&error, &policy).expect("error must be valid");

        let first = DiagnosticReport::new(vec![warning.clone(), error.clone()])
            .expect("report must be valid");
        let second =
            DiagnosticReport::new(vec![error, warning]).expect("reordered report must be valid");
        assert_eq!(first, second);
        assert_eq!(
            first.findings()[0].disposition(),
            DiagnosticDisposition::Active
        );
        assert_eq!(first.findings()[0].severity(), DiagnosticSeverity::Error);
        assert_eq!(first.summary().total(), 2);
        assert_eq!(first.summary().active(), 1);
        assert_eq!(first.summary().suppressed(), 1);
        assert_eq!(first.summary().by_family().values().sum::<usize>(), 2);
        assert_eq!(first.summary().by_severity().values().sum::<usize>(), 2);
        assert_eq!(first.summary().by_category().values().sum::<usize>(), 2);
        assert_eq!(first.summary().active_by_code().values().sum::<usize>(), 1);
        assert_eq!(
            first.summary().suppressed_by_code().values().sum::<usize>(),
            1
        );

        let filter = DiagnosticFilter::new(
            BTreeSet::from([DiagnosticFamily::Semantic]),
            BTreeSet::from([DiagnosticSeverity::Error]),
            BTreeSet::new(),
            BTreeSet::from([DiagnosticDisposition::Active]),
        );
        let filtered = first.filtered(&filter).collect::<Vec<_>>();
        assert_eq!(filtered.len(), 1);
        assert_eq!(first.summary().total(), 2);
    }

    #[test]
    fn report_collapses_exact_duplicates_and_rejects_identity_conflicts() {
        let original = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "original",
            "metadata.source",
        );
        let conflicting = semantic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Warning,
            SemanticDiagnosticKind::UnresolvedTarget,
            "conflicting-secret-marker",
            "metadata.source",
        );
        let original = DiagnosticFinding::from_semantic(&original, &DiagnosticPolicy::default())
            .expect("original finding must be valid");
        let conflicting =
            DiagnosticFinding::from_semantic(&conflicting, &DiagnosticPolicy::default())
                .expect("conflicting finding must be individually valid");

        let duplicate = DiagnosticReport::new(vec![original.clone(), original.clone()])
            .expect("exact duplicates must collapse");
        assert_eq!(duplicate.findings(), std::slice::from_ref(&original));

        let error = DiagnosticReport::new(vec![conflicting, original])
            .expect_err("different content for one identity must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::ConflictingEvidence);
        assert!(!error.to_string().contains("conflicting-secret-marker"));
        assert!(!format!("{error:?}").contains("conflicting-secret-marker"));
    }

    #[test]
    fn empty_report_and_filter_are_complete() {
        let report = DiagnosticReport::empty();
        assert!(report.findings().is_empty());
        assert_eq!(report.summary().total(), 0);
        assert_eq!(report.filtered(&DiagnosticFilter::default()).count(), 0);
    }

    #[test]
    fn every_error_kind_is_closed_and_redacted() {
        let kinds = [
            DiagnosticErrorKind::TooManySemanticDiagnostics,
            DiagnosticErrorKind::TooManyValidationIssues,
            DiagnosticErrorKind::TooManyFindings,
            DiagnosticErrorKind::TooManySuppressions,
            DiagnosticErrorKind::MessageTooLarge,
            DiagnosticErrorKind::TooManyNodeAnchors,
            DiagnosticErrorKind::TooManyProvenanceRecords,
            DiagnosticErrorKind::ConflictingEvidence,
        ];

        for kind in kinds {
            let error = if kind == DiagnosticErrorKind::ConflictingEvidence {
                DiagnosticError::conflicting_evidence()
            } else {
                DiagnosticError::bounded(kind, 2, 1)
            };
            assert_eq!(error.kind(), kind);
            assert!(!error.to_string().contains("metadata"));
            assert!(!format!("{error:?}").contains("metadata"));
        }
    }
}
