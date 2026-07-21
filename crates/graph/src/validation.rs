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
    SemanticDiagnostic, SemanticGraph, SemanticGraphReport, SemanticReferenceStatistics,
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
    /// The first schema version validates currently constrained relations and
    /// intentionally leaves broad dependency-like relations unrestricted until
    /// the semantic model defines stronger constraints.
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
            EdgeKind::Reads
            | EdgeKind::Writes
            | EdgeKind::Grants
            | EdgeKind::Includes
            | EdgeKind::Extends
            | EdgeKind::DependsOn => true,
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

    fn validate_edges(graph: &SemanticGraph, issues: &mut Vec<SemanticGraphValidationIssue>) {
        for edge in graph.edges() {
            let edge_id = edge_id(edge.source(), edge.target(), edge.kind());
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
                .with_edge_id(edge_id(edge.source(), edge.target(), edge.kind()))
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
        _ => false,
    }
}

const fn allows_reference(source: NodeKind, target: NodeKind) -> bool {
    matches!(source, NodeKind::Unknown)
        || matches!(target, NodeKind::Unknown | NodeKind::Metadata(_))
        || (is_reference_participant(source) && is_reference_participant(target))
}

const fn is_reference_participant(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Metadata(_)
            | NodeKind::Module
            | NodeKind::Procedure
            | NodeKind::Function
            | NodeKind::Query
            | NodeKind::Form
            | NodeKind::Command
            | NodeKind::Attribute
            | NodeKind::StandardAttribute
            | NodeKind::TabularSection
            | NodeKind::Dimension
            | NodeKind::Resource
            | NodeKind::Measure
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

const fn forbids_self_loop(kind: EdgeKind) -> bool {
    matches!(kind, EdgeKind::Contains | EdgeKind::Calls)
}

fn edge_id(source: &EntityId, target: &EntityId, kind: EdgeKind) -> EdgeId {
    EdgeId::new(format!(
        "edge:source#{}:{};target#{}:{};kind:{}",
        source.as_str().len(),
        source.as_str(),
        target.as_str().len(),
        target.as_str(),
        edge_kind_code(kind)
    ))
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
    }
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
