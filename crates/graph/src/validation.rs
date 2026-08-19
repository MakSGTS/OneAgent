//! Deterministic validation for already-built semantic graphs.
//!
//! Validation checks graph-state invariants after graph construction. It does
//! not read source files, does not rebuild the graph and does not run semantic
//! resolution. Semantic diagnostics remain responsible for recoverable problems
//! observed during build and resolution, while validation issues describe
//! invalid or degraded graph state.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use oneagent_common::EntityId;
use oneagent_metadata::MetadataKind;

use crate::{
    Confidence, EdgeId, EdgeKind, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionState,
    SemanticDiagnostic, SemanticDiagnosticKind, SemanticGraph, SemanticGraphReport,
    SemanticReferenceCategory, SemanticReferenceRequest, SemanticReferenceRequestId,
    SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome, SemanticReferenceStatistics,
    edge_identity::edge_id as stable_edge_id,
};

/// Stable machine-readable validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticGraphValidationCode {
    /// An edge source node is absent from graph storage.
    MissingSource,
    /// An edge target node is absent from graph storage.
    MissingTarget,
    /// An edge connects node kinds not allowed by the semantic graph schema.
    InvalidEdgeEndpoints,
    /// A containment edge or child owner violates ownership rules.
    InvalidOwner,
    /// A child node has more than one containment owner where one is allowed.
    MultipleOwners,
    /// A self-loop appears on an edge kind where self-loops are forbidden.
    ForbiddenSelfLoop,
    /// A cycle appears in an acyclic semantic relation.
    Cycle,
    /// A node has no attached provenance.
    MissingNodeProvenance,
    /// An edge has no attached provenance.
    MissingEdgeProvenance,
    /// Semantic reference counters are internally inconsistent.
    InconsistentResolutionStatistics,
    /// Diagnostic counters are inconsistent with supplied diagnostics.
    InconsistentDiagnosticStatistics,
    /// A supplied graph report is inconsistent with graph, diagnostics or counters.
    InconsistentReport,
    /// A build-boundary request has not reached a terminal outcome.
    NonTerminalReferenceRequest,
    /// A request source node is absent from the graph snapshot.
    MissingReferenceRequestSource,
    /// A request candidate node is absent from the graph snapshot.
    MissingReferenceRequestCandidate,
    /// A resolved or ambiguous candidate kind is not accepted by the request.
    IncompatibleReferenceRequestCandidate,
    /// A resolved request has no matching direct edge projection.
    MissingReferenceRequestEdgeProjection,
    /// A non-resolved request has a direct edge projection.
    UnexpectedReferenceRequestEdgeProjection,
    /// A failed terminal request has no matching typed diagnostic projection.
    MissingReferenceRequestDiagnosticProjection,
}

impl SemanticGraphValidationCode {
    /// Returns the stable string representation of the validation code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingSource => "graph.structure.missing_source",
            Self::MissingTarget => "graph.structure.missing_target",
            Self::InvalidEdgeEndpoints => "graph.semantic.invalid_edge_endpoints",
            Self::InvalidOwner => "graph.semantic.invalid_owner",
            Self::MultipleOwners => "graph.semantic.multiple_owners",
            Self::ForbiddenSelfLoop => "graph.semantic.forbidden_self_loop",
            Self::Cycle => "graph.semantic.cycle",
            Self::MissingNodeProvenance => "graph.provenance.missing_node",
            Self::MissingEdgeProvenance => "graph.provenance.missing_edge",
            Self::InconsistentResolutionStatistics => {
                "graph.build.inconsistent_resolution_statistics"
            }
            Self::InconsistentDiagnosticStatistics => {
                "graph.build.inconsistent_diagnostic_statistics"
            }
            Self::InconsistentReport => "graph.build.inconsistent_report",
            Self::NonTerminalReferenceRequest => "graph.build.reference_request.non_terminal",
            Self::MissingReferenceRequestSource => "graph.build.reference_request.missing_source",
            Self::MissingReferenceRequestCandidate => {
                "graph.build.reference_request.missing_candidate"
            }
            Self::IncompatibleReferenceRequestCandidate => {
                "graph.build.reference_request.incompatible_candidate"
            }
            Self::MissingReferenceRequestEdgeProjection => {
                "graph.build.reference_request.missing_edge_projection"
            }
            Self::UnexpectedReferenceRequestEdgeProjection => {
                "graph.build.reference_request.unexpected_edge_projection"
            }
            Self::MissingReferenceRequestDiagnosticProjection => {
                "graph.build.reference_request.missing_diagnostic_projection"
            }
        }
    }
}

/// Severity assigned to a validation issue.
///
/// `Error` means that a mandatory graph or build-result invariant is broken.
/// `Warning` means that the graph can still be consumed but quality is reduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticGraphValidationSeverity {
    /// Mandatory invariant violation.
    Error,
    /// Technically valid but degraded graph state.
    Warning,
}

/// Typed category of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticGraphValidationIssueKind {
    /// Graph storage or endpoint integrity.
    Structural,
    /// Source-independent semantic graph schema or ownership invariant.
    Semantic,
    /// Provenance coverage invariant.
    Provenance,
    /// Build-result aggregate consistency invariant.
    BuildConsistency,
}

/// Structured validation issue with deterministic ordering.
///
/// The issue contains stable machine-readable fields for automation and a
/// compact human-readable message for diagnostics. Ordering and deduplication
/// use typed fields rather than debug output or insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphValidationIssue {
    code: SemanticGraphValidationCode,
    severity: SemanticGraphValidationSeverity,
    kind: SemanticGraphValidationIssueKind,
    message: String,
    nodes: Vec<EntityId>,
    edge_id: Option<EdgeId>,
    source_kind: Option<NodeKind>,
    target_kind: Option<NodeKind>,
    edge_kind: Option<EdgeKind>,
    provenance: Vec<Provenance>,
    reference_request_id: Option<SemanticReferenceRequestId>,
    invariant: &'static str,
}

impl SemanticGraphValidationIssue {
    fn new(
        code: SemanticGraphValidationCode,
        severity: SemanticGraphValidationSeverity,
        kind: SemanticGraphValidationIssueKind,
        message: impl Into<String>,
        invariant: &'static str,
    ) -> Self {
        Self {
            code,
            severity,
            kind,
            message: message.into(),
            nodes: Vec::new(),
            edge_id: None,
            source_kind: None,
            target_kind: None,
            edge_kind: None,
            provenance: Vec::new(),
            reference_request_id: None,
            invariant,
        }
    }

    /// Returns the stable validation issue code.
    #[must_use]
    pub const fn code(&self) -> SemanticGraphValidationCode {
        self.code
    }

    /// Returns the validation severity.
    #[must_use]
    pub const fn severity(&self) -> SemanticGraphValidationSeverity {
        self.severity
    }

    /// Returns the typed validation issue kind.
    #[must_use]
    pub const fn kind(&self) -> SemanticGraphValidationIssueKind {
        self.kind
    }

    /// Returns a compact human-readable validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns related node identifiers in deterministic order.
    #[must_use]
    pub fn nodes(&self) -> &[EntityId] {
        &self.nodes
    }

    /// Returns the related deterministic edge identifier, when applicable.
    #[must_use]
    pub const fn edge_id(&self) -> Option<&EdgeId> {
        self.edge_id.as_ref()
    }

    /// Returns the related source node kind, when applicable.
    #[must_use]
    pub const fn source_kind(&self) -> Option<NodeKind> {
        self.source_kind
    }

    /// Returns the related target node kind, when applicable.
    #[must_use]
    pub const fn target_kind(&self) -> Option<NodeKind> {
        self.target_kind
    }

    /// Returns the related edge kind, when applicable.
    #[must_use]
    pub const fn edge_kind(&self) -> Option<EdgeKind> {
        self.edge_kind
    }

    /// Returns provenance records related to the issue.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Returns the related reference request identity, when applicable.
    #[must_use]
    pub const fn reference_request_id(&self) -> Option<&SemanticReferenceRequestId> {
        self.reference_request_id.as_ref()
    }

    /// Returns the invariant context checked by this issue.
    #[must_use]
    pub const fn invariant(&self) -> &'static str {
        self.invariant
    }

    fn with_nodes(mut self, mut nodes: Vec<EntityId>) -> Self {
        nodes.sort();
        nodes.dedup();
        self.nodes = nodes;
        self
    }

    fn with_edge_id(mut self, edge_id: EdgeId) -> Self {
        self.edge_id = Some(edge_id);
        self
    }

    const fn with_source_kind(mut self, source_kind: NodeKind) -> Self {
        self.source_kind = Some(source_kind);
        self
    }

    const fn with_target_kind(mut self, target_kind: NodeKind) -> Self {
        self.target_kind = Some(target_kind);
        self
    }

    const fn with_edge_kind(mut self, edge_kind: EdgeKind) -> Self {
        self.edge_kind = Some(edge_kind);
        self
    }

    fn with_provenance(mut self, provenance: &[Provenance]) -> Self {
        self.provenance = normalized_provenance(provenance);
        self
    }

    fn with_reference_request_id(mut self, id: SemanticReferenceRequestId) -> Self {
        self.reference_request_id = Some(id);
        self
    }

    fn provenance_key(&self) -> Vec<ProvenanceKey> {
        self.provenance
            .iter()
            .map(provenance_key)
            .collect::<Vec<_>>()
    }
}

impl PartialOrd for SemanticGraphValidationIssue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticGraphValidationIssue {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.severity,
            self.code,
            self.kind,
            &self.nodes,
            &self.edge_id,
            &self.reference_request_id,
            self.edge_kind,
            self.source_kind,
            self.target_kind,
            self.invariant,
            self.provenance_key(),
            &self.message,
        )
            .cmp(&(
                other.severity,
                other.code,
                other.kind,
                &other.nodes,
                &other.edge_id,
                &other.reference_request_id,
                other.edge_kind,
                other.source_kind,
                other.target_kind,
                other.invariant,
                other.provenance_key(),
                &other.message,
            ))
    }
}

/// Aggregate counters for a validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphValidationSummary {
    total: usize,
    errors: usize,
    warnings: usize,
    by_code: BTreeMap<SemanticGraphValidationCode, usize>,
    by_kind: BTreeMap<SemanticGraphValidationIssueKind, usize>,
    provenance_issues: usize,
}

impl SemanticGraphValidationSummary {
    fn from_issues(issues: &[SemanticGraphValidationIssue]) -> Self {
        let mut errors = 0;
        let mut warnings = 0;
        let mut by_code = BTreeMap::new();
        let mut by_kind = BTreeMap::new();
        let mut provenance_issues = 0;

        for issue in issues {
            match issue.severity() {
                SemanticGraphValidationSeverity::Error => errors += 1,
                SemanticGraphValidationSeverity::Warning => warnings += 1,
            }
            *by_code.entry(issue.code()).or_default() += 1;
            *by_kind.entry(issue.kind()).or_default() += 1;
            if issue.kind() == SemanticGraphValidationIssueKind::Provenance {
                provenance_issues += 1;
            }
        }

        Self {
            total: issues.len(),
            errors,
            warnings,
            by_code,
            by_kind,
            provenance_issues,
        }
    }

    /// Returns the total number of validation issues.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns the number of error-level validation issues.
    #[must_use]
    pub const fn errors(&self) -> usize {
        self.errors
    }

    /// Returns the number of warning-level validation issues.
    #[must_use]
    pub const fn warnings(&self) -> usize {
        self.warnings
    }

    /// Returns deterministic issue counts by stable code.
    #[must_use]
    pub const fn by_code(&self) -> &BTreeMap<SemanticGraphValidationCode, usize> {
        &self.by_code
    }

    /// Returns deterministic issue counts by typed issue kind.
    #[must_use]
    pub const fn by_kind(&self) -> &BTreeMap<SemanticGraphValidationIssueKind, usize> {
        &self.by_kind
    }

    /// Returns the number of provenance validation issues.
    #[must_use]
    pub const fn provenance_issues(&self) -> usize {
        self.provenance_issues
    }
}

/// Owned validation result for a graph or graph build snapshot.
///
/// `is_valid` is `true` when the result has no error-level issues. Warning-only
/// results remain valid so existing graphs that use compatibility constructors
/// without provenance can still be consumed while reporting reduced quality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphValidationResult {
    issues: Vec<SemanticGraphValidationIssue>,
    summary: SemanticGraphValidationSummary,
}

impl SemanticGraphValidationResult {
    fn new(mut issues: Vec<SemanticGraphValidationIssue>) -> Self {
        issues.sort();
        issues.dedup();
        let summary = SemanticGraphValidationSummary::from_issues(&issues);

        Self { issues, summary }
    }

    /// Returns validation issues in deterministic order.
    #[must_use]
    pub fn issues(&self) -> &[SemanticGraphValidationIssue] {
        &self.issues
    }

    /// Returns aggregate validation counters.
    #[must_use]
    pub const fn summary(&self) -> &SemanticGraphValidationSummary {
        &self.summary
    }

    /// Returns the number of error-level validation issues.
    #[must_use]
    pub const fn error_count(&self) -> usize {
        self.summary.errors()
    }

    /// Returns the number of warning-level validation issues.
    #[must_use]
    pub const fn warning_count(&self) -> usize {
        self.summary.warnings()
    }

    /// Returns `true` when no mandatory graph or build invariant is broken.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.summary.errors() == 0
    }
}

/// Source-independent semantic graph schema used by validation.
#[derive(Debug, Default, Clone, Copy)]
pub struct SemanticGraphSchema;

impl SemanticGraphSchema {
    /// Returns whether an edge kind is allowed between two node kinds.
    ///
    /// Every edge kind delegates to its explicit accepted endpoint policy.
    /// Unsupported, unknown, and future endpoint families are rejected until
    /// their contracts are added deliberately.
    #[must_use]
    pub const fn allows(
        self,
        source_kind: NodeKind,
        edge_kind: EdgeKind,
        target_kind: NodeKind,
    ) -> bool {
        match edge_kind {
            EdgeKind::Contains => allows_contains(source_kind, target_kind),
            EdgeKind::Calls => is_callable(source_kind) && is_callable(target_kind),
            EdgeKind::References => allows_reference(source_kind, target_kind),
            EdgeKind::DependsOn => allows_depends_on(source_kind, target_kind),
            EdgeKind::Extends => allows_extends(source_kind, target_kind),
            EdgeKind::Grants => allows_grants(source_kind, target_kind),
            EdgeKind::Includes => allows_includes(source_kind, target_kind),
            EdgeKind::Reads => allows_reads(source_kind, target_kind),
            EdgeKind::Writes => allows_writes(source_kind, target_kind),
        }
    }
}

/// Stateless validator for graph and build-result invariants.
#[derive(Debug, Clone, Copy)]
pub struct SemanticGraphValidator;

impl SemanticGraphValidator {
    /// Creates a validator using the default semantic graph schema.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates an already-built graph without mutating it.
    ///
    /// Validation is deterministic, source-independent and does not perform
    /// semantic resolution. Duplicate issues are removed by a stable typed key.
    #[must_use]
    pub fn validate(&self, graph: &SemanticGraph) -> SemanticGraphValidationResult {
        let mut issues = Vec::new();

        Self::validate_edges(graph, &mut issues);
        Self::validate_ownership(graph, &mut issues);
        Self::validate_provenance(graph, &mut issues);

        SemanticGraphValidationResult::new(issues)
    }

    /// Validates graph-level and build-level invariants.
    ///
    /// Build-level checks use supplied diagnostics and reference counters. They
    /// do not rerun semantic resolution and do not treat recoverable diagnostics
    /// as validation failures by themselves.
    #[must_use]
    pub fn validate_build_result(
        &self,
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        reference_statistics: SemanticReferenceStatistics,
    ) -> SemanticGraphValidationResult {
        let report = SemanticGraphReport::from_graph_diagnostics_and_references(
            graph,
            diagnostics,
            reference_statistics,
        );

        self.validate_build_result_with_report(graph, diagnostics, reference_statistics, &report)
    }

    /// Validates graph-level and build-level invariants against a supplied report.
    ///
    /// This entry point is intended for build results that cache or transport a
    /// report. When reports are computed on demand, use
    /// [`Self::validate_build_result`].
    #[must_use]
    pub fn validate_build_result_with_report(
        &self,
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        reference_statistics: SemanticReferenceStatistics,
        report: &SemanticGraphReport,
    ) -> SemanticGraphValidationResult {
        let mut issues = self.validate(graph).issues().to_vec();

        Self::validate_reference_statistics(reference_statistics, &mut issues);
        Self::validate_diagnostic_statistics(diagnostics, report, &mut issues);
        Self::validate_report(
            graph,
            diagnostics,
            reference_statistics,
            report,
            &mut issues,
        );

        SemanticGraphValidationResult::new(issues)
    }

    /// Validates a build snapshot whose accepted references are canonical requests.
    #[must_use]
    pub fn validate_build_result_with_reference_requests(
        &self,
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        requests: &SemanticReferenceRequestLedger,
    ) -> SemanticGraphValidationResult {
        self.validate_build_result_with_reference_requests_and_legacy_observations(
            graph,
            diagnostics,
            requests,
            SemanticReferenceStatistics::new(),
        )
    }

    /// Validates a request-aware build snapshot during producer migration.
    ///
    /// `legacy_observations` must exclude accepted references already present
    /// in `requests`.
    #[must_use]
    pub fn validate_build_result_with_reference_requests_and_legacy_observations(
        &self,
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        requests: &SemanticReferenceRequestLedger,
        legacy_observations: SemanticReferenceStatistics,
    ) -> SemanticGraphValidationResult {
        let report =
            SemanticGraphReport::from_graph_diagnostics_reference_requests_and_legacy_observations(
                graph,
                diagnostics,
                requests,
                legacy_observations,
            );
        self.validate_build_result_with_reference_requests_and_report(
            graph,
            diagnostics,
            requests,
            legacy_observations,
            &report,
        )
    }

    /// Validates request projections and a supplied request-aware report.
    #[must_use]
    pub fn validate_build_result_with_reference_requests_and_report(
        &self,
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        requests: &SemanticReferenceRequestLedger,
        legacy_observations: SemanticReferenceStatistics,
        report: &SemanticGraphReport,
    ) -> SemanticGraphValidationResult {
        let statistics = SemanticReferenceStatistics::from_reference_requests(requests)
            .including_legacy_observations(legacy_observations);
        let mut issues = self
            .validate_build_result_with_report(graph, diagnostics, statistics, report)
            .issues()
            .to_vec();
        Self::validate_reference_requests(graph, diagnostics, requests, &mut issues);
        SemanticGraphValidationResult::new(issues)
    }

    fn validate_reference_requests(
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        requests: &SemanticReferenceRequestLedger,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        for request in requests.requests() {
            Self::validate_reference_request_nodes(graph, request, issues);
            Self::validate_reference_request_edge(graph, request, issues);
            Self::validate_reference_request_diagnostic(diagnostics, request, issues);
        }
    }

    fn validate_reference_request_nodes(
        graph: &SemanticGraph,
        request: &SemanticReferenceRequest,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        if request.outcome() == SemanticReferenceRequestOutcome::Collected {
            issues.push(reference_request_issue(
                SemanticGraphValidationCode::NonTerminalReferenceRequest,
                "semantic reference request has not reached a terminal outcome",
                "terminal reference request lifecycle",
                request,
            ));
        }

        if graph.node(request.source_node()).is_none() {
            issues.push(reference_request_issue(
                SemanticGraphValidationCode::MissingReferenceRequestSource,
                "semantic reference request source node is missing",
                "reference request source existence",
                request,
            ));
        }

        for candidate in request.candidates() {
            let Some(node) = graph.node(candidate) else {
                issues.push(reference_request_issue(
                    SemanticGraphValidationCode::MissingReferenceRequestCandidate,
                    "semantic reference request candidate node is missing",
                    "reference request candidate existence",
                    request,
                ));
                continue;
            };
            if matches!(
                request.outcome(),
                SemanticReferenceRequestOutcome::Resolved
                    | SemanticReferenceRequestOutcome::AmbiguousTarget
            ) && !request.expected_kinds().contains(&node.kind())
            {
                issues.push(
                    reference_request_issue(
                        SemanticGraphValidationCode::IncompatibleReferenceRequestCandidate,
                        "semantic reference request candidate kind is incompatible",
                        "reference request candidate kind compatibility",
                        request,
                    )
                    .with_target_kind(node.kind()),
                );
            }
        }
    }

    fn validate_reference_request_edge(
        graph: &SemanticGraph,
        request: &SemanticReferenceRequest,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        let edge_kind = reference_request_edge_kind(request.category());
        let has_projection = request.candidates().iter().any(|candidate| {
            graph.edges().any(|edge| {
                edge.source() == request.source_node()
                    && edge.target() == candidate
                    && edge.kind() == edge_kind
            })
        });

        if request.outcome() == SemanticReferenceRequestOutcome::Resolved && !has_projection {
            issues.push(
                reference_request_issue(
                    SemanticGraphValidationCode::MissingReferenceRequestEdgeProjection,
                    "resolved semantic reference request has no direct edge projection",
                    "resolved reference request edge projection",
                    request,
                )
                .with_edge_kind(edge_kind),
            );
        } else if request.outcome() != SemanticReferenceRequestOutcome::Resolved && has_projection {
            issues.push(
                reference_request_issue(
                    SemanticGraphValidationCode::UnexpectedReferenceRequestEdgeProjection,
                    "non-resolved semantic reference request has a direct edge projection",
                    "non-resolved reference request edge projection",
                    request,
                )
                .with_edge_kind(edge_kind),
            );
        }
    }

    fn validate_reference_request_diagnostic(
        diagnostics: &[SemanticDiagnostic],
        request: &SemanticReferenceRequest,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        let Some(expected_kind) = reference_request_diagnostic_kind(request.outcome()) else {
            return;
        };
        let has_projection = diagnostics.iter().any(|diagnostic| {
            diagnostic.kind() == expected_kind
                && diagnostic.source_node() == Some(request.source_node())
                && diagnostic.reference() == request.reference()
                && diagnostic.expected_kinds() == request.expected_kinds()
                && diagnostic.candidates() == request.candidates()
                && !diagnostic.provenance().is_empty()
        });
        if !has_projection {
            issues.push(reference_request_issue(
                SemanticGraphValidationCode::MissingReferenceRequestDiagnosticProjection,
                "failed semantic reference request has no matching typed diagnostic projection",
                "failed reference request diagnostic projection",
                request,
            ));
        }
    }

    fn validate_edges(graph: &SemanticGraph, issues: &mut Vec<SemanticGraphValidationIssue>) {
        for edge in graph.edges() {
            let edge_id =
                stable_edge_id(edge.source().as_str(), edge.target().as_str(), edge.kind());
            let source = graph.node(edge.source());
            let target = graph.node(edge.target());

            if source.is_none() {
                issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::MissingSource,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Structural,
                        "semantic graph edge source node is missing",
                        "edge endpoints must exist",
                    )
                    .with_nodes(vec![edge.source().clone(), edge.target().clone()])
                    .with_edge_id(edge_id.clone())
                    .with_edge_kind(edge.kind())
                    .with_provenance(edge.provenance()),
                );
            }

            if target.is_none() {
                issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::MissingTarget,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Structural,
                        "semantic graph edge target node is missing",
                        "edge endpoints must exist",
                    )
                    .with_nodes(vec![edge.source().clone(), edge.target().clone()])
                    .with_edge_id(edge_id.clone())
                    .with_edge_kind(edge.kind())
                    .with_provenance(edge.provenance()),
                );
            }

            let (Some(source), Some(target)) = (source, target) else {
                continue;
            };

            if edge.source() == edge.target() && forbids_self_loop(edge.kind()) {
                issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::ForbiddenSelfLoop,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Semantic,
                        "semantic graph edge kind forbids self-loops",
                        "edge self-loop policy",
                    )
                    .with_nodes(vec![edge.source().clone()])
                    .with_edge_id(edge_id.clone())
                    .with_source_kind(source.kind())
                    .with_target_kind(target.kind())
                    .with_edge_kind(edge.kind())
                    .with_provenance(edge.provenance()),
                );
            }

            if !SemanticGraphSchema.allows(source.kind(), edge.kind(), target.kind()) {
                issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::InvalidEdgeEndpoints,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Semantic,
                        "semantic graph edge endpoints are not allowed by schema",
                        "edge endpoint schema",
                    )
                    .with_nodes(vec![edge.source().clone(), edge.target().clone()])
                    .with_edge_id(edge_id)
                    .with_source_kind(source.kind())
                    .with_target_kind(target.kind())
                    .with_edge_kind(edge.kind())
                    .with_provenance(edge.provenance()),
                );
            }
        }
    }

    fn validate_ownership(graph: &SemanticGraph, issues: &mut Vec<SemanticGraphValidationIssue>) {
        let ownership = ownership_index(graph);

        for node in graph.nodes() {
            if !requires_owner(node.kind()) {
                continue;
            }

            match ownership.get(node.id()) {
                None => issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::InvalidOwner,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Semantic,
                        "semantic graph child node has no containment owner",
                        "mandatory owner edge",
                    )
                    .with_nodes(vec![node.id().clone()])
                    .with_target_kind(node.kind())
                    .with_edge_kind(EdgeKind::Contains)
                    .with_provenance(node.provenance()),
                ),
                Some(owners) if owners.len() > 1 => issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::MultipleOwners,
                        SemanticGraphValidationSeverity::Error,
                        SemanticGraphValidationIssueKind::Semantic,
                        "semantic graph child node has multiple containment owners",
                        "single canonical owner",
                    )
                    .with_nodes(
                        owners
                            .iter()
                            .cloned()
                            .chain(std::iter::once(node.id().clone()))
                            .collect(),
                    )
                    .with_target_kind(node.kind())
                    .with_edge_kind(EdgeKind::Contains)
                    .with_provenance(node.provenance()),
                ),
                Some(owners) => {
                    let owner_id = owners.iter().next().expect("owner must exist");
                    let Some(owner) = graph.node(owner_id) else {
                        continue;
                    };

                    if !allows_owner(owner.kind(), node.kind()) {
                        issues.push(
                            SemanticGraphValidationIssue::new(
                                SemanticGraphValidationCode::InvalidOwner,
                                SemanticGraphValidationSeverity::Error,
                                SemanticGraphValidationIssueKind::Semantic,
                                "semantic graph child owner kind is invalid",
                                "owner-child kind schema",
                            )
                            .with_nodes(vec![owner_id.clone(), node.id().clone()])
                            .with_source_kind(owner.kind())
                            .with_target_kind(node.kind())
                            .with_edge_kind(EdgeKind::Contains)
                            .with_provenance(node.provenance()),
                        );
                    }
                }
            }
        }

        Self::validate_ownership_cycles(graph, &ownership, issues);
    }

    fn validate_ownership_cycles(
        graph: &SemanticGraph,
        ownership: &BTreeMap<EntityId, BTreeSet<EntityId>>,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        for node in graph.nodes() {
            let mut path = Vec::new();
            let mut seen = BTreeSet::new();
            let mut current = node.id().clone();

            while let Some(owners) = ownership.get(&current) {
                let Some(owner) = owners.iter().next() else {
                    break;
                };

                path.push(current.clone());

                if !seen.insert(owner.clone()) {
                    path.push(owner.clone());
                    issues.push(
                        SemanticGraphValidationIssue::new(
                            SemanticGraphValidationCode::Cycle,
                            SemanticGraphValidationSeverity::Error,
                            SemanticGraphValidationIssueKind::Semantic,
                            "semantic graph ownership hierarchy contains a cycle",
                            "acyclic ownership hierarchy",
                        )
                        .with_nodes(path)
                        .with_edge_kind(EdgeKind::Contains),
                    );
                    break;
                }

                current = owner.clone();
            }
        }
    }

    fn validate_provenance(graph: &SemanticGraph, issues: &mut Vec<SemanticGraphValidationIssue>) {
        for node in graph.nodes() {
            if node.provenance().is_empty() {
                issues.push(
                    SemanticGraphValidationIssue::new(
                        SemanticGraphValidationCode::MissingNodeProvenance,
                        SemanticGraphValidationSeverity::Warning,
                        SemanticGraphValidationIssueKind::Provenance,
                        "semantic graph node has no provenance",
                        "graph facts should retain provenance",
                    )
                    .with_nodes(vec![node.id().clone()])
                    .with_target_kind(node.kind()),
                );
            }
        }

        for edge in graph.edges() {
            if edge.provenance().is_empty() {
                let source_kind = graph.node(edge.source()).map(crate::GraphNode::kind);
                let target_kind = graph.node(edge.target()).map(crate::GraphNode::kind);
                let mut issue = SemanticGraphValidationIssue::new(
                    SemanticGraphValidationCode::MissingEdgeProvenance,
                    SemanticGraphValidationSeverity::Warning,
                    SemanticGraphValidationIssueKind::Provenance,
                    "semantic graph edge has no provenance",
                    "graph facts should retain provenance",
                )
                .with_nodes(vec![edge.source().clone(), edge.target().clone()])
                .with_edge_id(stable_edge_id(
                    edge.source().as_str(),
                    edge.target().as_str(),
                    edge.kind(),
                ))
                .with_edge_kind(edge.kind());

                if let Some(kind) = source_kind {
                    issue = issue.with_source_kind(kind);
                }
                if let Some(kind) = target_kind {
                    issue = issue.with_target_kind(kind);
                }

                issues.push(issue);
            }
        }
    }

    fn validate_reference_statistics(
        statistics: SemanticReferenceStatistics,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        if statistics.total() != statistics.outcome_total() {
            issues.push(SemanticGraphValidationIssue::new(
                SemanticGraphValidationCode::InconsistentResolutionStatistics,
                SemanticGraphValidationSeverity::Error,
                SemanticGraphValidationIssueKind::BuildConsistency,
                "semantic reference total does not match outcome counters",
                "reference statistics totals",
            ));
        }

        if statistics.total() != statistics.with_provenance() + statistics.without_provenance() {
            issues.push(SemanticGraphValidationIssue::new(
                SemanticGraphValidationCode::InconsistentResolutionStatistics,
                SemanticGraphValidationSeverity::Error,
                SemanticGraphValidationIssueKind::BuildConsistency,
                "semantic reference total does not match provenance counters",
                "reference provenance statistics totals",
            ));
        }
    }

    fn validate_diagnostic_statistics(
        diagnostics: &[SemanticDiagnostic],
        report: &SemanticGraphReport,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        if report.diagnostics().total() != diagnostics.len() {
            issues.push(SemanticGraphValidationIssue::new(
                SemanticGraphValidationCode::InconsistentDiagnosticStatistics,
                SemanticGraphValidationSeverity::Error,
                SemanticGraphValidationIssueKind::BuildConsistency,
                "semantic diagnostic report total does not match supplied diagnostics",
                "diagnostic statistics totals",
            ));
        }
    }

    fn validate_report(
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        reference_statistics: SemanticReferenceStatistics,
        report: &SemanticGraphReport,
        issues: &mut Vec<SemanticGraphValidationIssue>,
    ) {
        let expected = SemanticGraphReport::from_graph_diagnostics_and_references(
            graph,
            diagnostics,
            reference_statistics,
        );

        if report != &expected {
            issues.push(SemanticGraphValidationIssue::new(
                SemanticGraphValidationCode::InconsistentReport,
                SemanticGraphValidationSeverity::Error,
                SemanticGraphValidationIssueKind::BuildConsistency,
                "semantic graph report is inconsistent with graph, diagnostics or counters",
                "graph report consistency",
            ));
        }
    }
}

impl Default for SemanticGraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

fn reference_request_issue(
    code: SemanticGraphValidationCode,
    message: &'static str,
    invariant: &'static str,
    request: &SemanticReferenceRequest,
) -> SemanticGraphValidationIssue {
    SemanticGraphValidationIssue::new(
        code,
        SemanticGraphValidationSeverity::Error,
        SemanticGraphValidationIssueKind::BuildConsistency,
        message,
        invariant,
    )
    .with_nodes(
        std::iter::once(request.source_node().clone())
            .chain(request.candidates().iter().cloned())
            .collect(),
    )
    .with_provenance(request.provenance())
    .with_reference_request_id(request.id().clone())
}

const fn reference_request_edge_kind(category: SemanticReferenceCategory) -> EdgeKind {
    match category {
        SemanticReferenceCategory::MetadataType | SemanticReferenceCategory::ProtectedResource => {
            EdgeKind::References
        }
        SemanticReferenceCategory::Callable => EdgeKind::Calls,
        SemanticReferenceCategory::QuerySource => EdgeKind::Reads,
        SemanticReferenceCategory::WriteTarget => EdgeKind::Writes,
        SemanticReferenceCategory::SubsystemMember => EdgeKind::Includes,
        SemanticReferenceCategory::ExtensionTarget => EdgeKind::Extends,
    }
}

const fn reference_request_diagnostic_kind(
    outcome: SemanticReferenceRequestOutcome,
) -> Option<SemanticDiagnosticKind> {
    match outcome {
        SemanticReferenceRequestOutcome::MissingTarget => {
            Some(SemanticDiagnosticKind::UnresolvedTarget)
        }
        SemanticReferenceRequestOutcome::AmbiguousTarget => {
            Some(SemanticDiagnosticKind::AmbiguousTarget)
        }
        SemanticReferenceRequestOutcome::IncompatibleTargetKind => {
            Some(SemanticDiagnosticKind::IncompatibleTargetKind)
        }
        SemanticReferenceRequestOutcome::InvalidOwnerReference => {
            Some(SemanticDiagnosticKind::InvalidOwnerReference)
        }
        SemanticReferenceRequestOutcome::Collected
        | SemanticReferenceRequestOutcome::Resolved
        | SemanticReferenceRequestOutcome::PartialWorkspace => None,
    }
}

fn ownership_index(graph: &SemanticGraph) -> BTreeMap<EntityId, BTreeSet<EntityId>> {
    let mut ownership = BTreeMap::<EntityId, BTreeSet<EntityId>>::new();

    for edge in graph
        .edges()
        .filter(|edge| edge.kind() == EdgeKind::Contains)
    {
        ownership
            .entry(edge.target().clone())
            .or_default()
            .insert(edge.source().clone());
    }

    ownership
}

const fn allows_contains(source: NodeKind, target: NodeKind) -> bool {
    match target {
        NodeKind::Metadata(MetadataKind::Configuration) => false,
        NodeKind::Metadata(_)
        | NodeKind::Module
        | NodeKind::StandardAttribute
        | NodeKind::TabularSection
        | NodeKind::Dimension
        | NodeKind::Resource
        | NodeKind::Measure
        | NodeKind::Form
        | NodeKind::Command => matches!(source, NodeKind::Metadata(_)),
        NodeKind::Attribute => matches!(source, NodeKind::Metadata(_) | NodeKind::TabularSection),
        NodeKind::Procedure | NodeKind::Function => matches!(source, NodeKind::Module),
        NodeKind::Query => matches!(source, NodeKind::Procedure | NodeKind::Function),
        _ => false,
    }
}

const fn allows_reference(source: NodeKind, target: NodeKind) -> bool {
    (matches!(
        source,
        NodeKind::Attribute | NodeKind::Dimension | NodeKind::Resource
    ) && matches!(
        target,
        NodeKind::Metadata(
            MetadataKind::Catalog
                | MetadataKind::Document
                | MetadataKind::Enumeration
                | MetadataKind::InformationRegister
                | MetadataKind::AccumulationRegister
                | MetadataKind::AccountingRegister
                | MetadataKind::CalculationRegister
                | MetadataKind::BusinessProcess
                | MetadataKind::Task
        )
    )) || (matches!(source, NodeKind::AccessRight)
        && matches!(
            target,
            NodeKind::Metadata(
                MetadataKind::Configuration
                    | MetadataKind::Catalog
                    | MetadataKind::Document
                    | MetadataKind::InformationRegister
                    | MetadataKind::AccumulationRegister
            )
        ))
}

const fn allows_depends_on(source: NodeKind, target: NodeKind) -> bool {
    matches!(
        source,
        NodeKind::Attribute | NodeKind::Dimension | NodeKind::Resource
    ) && matches!(target, NodeKind::Metadata(_))
}

const fn allows_reads(source: NodeKind, target: NodeKind) -> bool {
    matches!(source, NodeKind::Query)
        && matches!(
            target,
            NodeKind::Metadata(MetadataKind::Catalog | MetadataKind::InformationRegister)
        )
}

const fn allows_writes(source: NodeKind, target: NodeKind) -> bool {
    matches!(source, NodeKind::Procedure)
        && matches!(
            target,
            NodeKind::Metadata(MetadataKind::AccumulationRegister)
        )
}

const fn allows_owner(owner: NodeKind, child: NodeKind) -> bool {
    allows_contains(owner, child)
}

const fn requires_owner(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Module
            | NodeKind::Procedure
            | NodeKind::Function
            | NodeKind::Attribute
            | NodeKind::StandardAttribute
            | NodeKind::TabularSection
            | NodeKind::Dimension
            | NodeKind::Resource
            | NodeKind::Measure
            | NodeKind::Form
            | NodeKind::Command
    )
}

const fn is_callable(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::Procedure | NodeKind::Function)
}

const fn allows_extends(source_kind: NodeKind, target_kind: NodeKind) -> bool {
    match (source_kind, target_kind) {
        (NodeKind::Metadata(source_metadata), NodeKind::Metadata(target_metadata)) => {
            source_metadata as u8 == target_metadata as u8
        }
        _ => false,
    }
}

const fn allows_grants(source_kind: NodeKind, target_kind: NodeKind) -> bool {
    matches!(
        (source_kind, target_kind),
        (NodeKind::Role, NodeKind::AccessRight)
    )
}

const fn allows_includes(source_kind: NodeKind, target_kind: NodeKind) -> bool {
    matches!(source_kind, NodeKind::Subsystem)
        && matches!(
            target_kind,
            NodeKind::Metadata(
                MetadataKind::Catalog
                    | MetadataKind::Document
                    | MetadataKind::Enumeration
                    | MetadataKind::CommonModule
                    | MetadataKind::Report
                    | MetadataKind::DataProcessor
                    | MetadataKind::InformationRegister
                    | MetadataKind::AccumulationRegister
                    | MetadataKind::AccountingRegister
                    | MetadataKind::CalculationRegister
                    | MetadataKind::BusinessProcess
                    | MetadataKind::Task
                    | MetadataKind::Role
                    | MetadataKind::Command
                    | MetadataKind::CommonForm
                    | MetadataKind::Template
                    | MetadataKind::HttpService
                    | MetadataKind::WebService
                    | MetadataKind::XdtoPackage
            )
        )
}

const fn forbids_self_loop(kind: EdgeKind) -> bool {
    matches!(
        kind,
        EdgeKind::Contains | EdgeKind::Calls | EdgeKind::Includes | EdgeKind::Extends
    )
}

fn normalized_provenance(provenance: &[Provenance]) -> Vec<Provenance> {
    let mut provenance = provenance.to_vec();
    provenance.sort_by_key(provenance_key);
    provenance.dedup_by_key(|record| provenance_key(record));
    provenance
}

fn provenance_key(provenance: &Provenance) -> ProvenanceKey {
    ProvenanceKey {
        source: provenance.source().map(|source| source.as_str().to_owned()),
        producer: provenance.producer().clone(),
        origin: provenance.origin(),
        confidence: provenance.confidence(),
        resolution: provenance.resolution(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProvenanceKey {
    source: Option<String>,
    producer: ProducerId,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;

    use super::SemanticGraphValidator;
    use crate::{
        EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph, SemanticGraphValidationCode,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn public_storage_prevents_duplicate_node_ids() {
        let mut graph = SemanticGraph::new();
        let node_id = id("node.same");

        assert!(
            graph
                .insert_node(GraphNode::new(
                    node_id.clone(),
                    name("First"),
                    NodeKind::Module,
                ))
                .is_none()
        );
        assert!(
            graph
                .insert_node(GraphNode::new(node_id, name("Second"), NodeKind::Module))
                .is_some()
        );

        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn public_storage_prevents_duplicate_edge_ids() {
        let source = id("procedure.source");
        let target = id("procedure.target");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            source.clone(),
            name("Source"),
            NodeKind::Procedure,
        ));
        graph.insert_node(GraphNode::new(
            target.clone(),
            name("Target"),
            NodeKind::Procedure,
        ));

        assert!(
            graph
                .insert_edge(GraphEdge::new(
                    source.clone(),
                    target.clone(),
                    EdgeKind::Calls
                ))
                .expect("edge must be insertable")
        );
        assert!(
            !graph
                .insert_edge(GraphEdge::new(source, target, EdgeKind::Calls))
                .expect("duplicate edge must be a no-op")
        );

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn warning_only_result_is_valid() {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id("metadata.document.sales"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));

        let result = SemanticGraphValidator::new().validate(&graph);

        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(
            result.issues()[0].code(),
            SemanticGraphValidationCode::MissingNodeProvenance
        );
    }
}
