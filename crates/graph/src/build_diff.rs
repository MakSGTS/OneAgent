//! Deterministic semantic graph build result diffs.

use std::collections::{BTreeMap, BTreeSet};

use oneagent_common::EntityId;

use crate::{
    Confidence, EdgeKind, FactOrigin, GraphDiffSummary, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphDiff, SemanticGraphReport,
    SemanticReference, SemanticReferenceStatistics,
};

/// Deterministic diff between two complete semantic graph build snapshots.
///
/// The diff is directional: `previous -> current`. It reuses
/// [`SemanticGraphDiff`] for node and edge comparison, then compares semantic
/// diagnostics, reference resolution counters, graph quality report aggregates
/// and provenance coverage aggregates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphBuildDiff {
    graph: SemanticGraphDiff,
    diagnostics: DiagnosticDiff,
    resolution: ResolutionStatisticsDiff,
    report: GraphReportDiff,
    provenance: ProvenanceCoverageDiff,
    summary: BuildDiffSummary,
}

impl SemanticGraphBuildDiff {
    /// Compares two graph build snapshots in the `previous -> current` direction.
    ///
    /// This method does not rebuild graphs, does not read source files and does
    /// not run semantic resolution. Reports are computed once per input snapshot.
    #[must_use]
    pub fn between(
        previous_graph: &SemanticGraph,
        previous_diagnostics: &[SemanticDiagnostic],
        previous_resolution: SemanticReferenceStatistics,
        current_graph: &SemanticGraph,
        current_diagnostics: &[SemanticDiagnostic],
        current_resolution: SemanticReferenceStatistics,
    ) -> Self {
        let graph = SemanticGraphDiff::between(previous_graph, current_graph);
        let diagnostics = DiagnosticDiff::between(previous_diagnostics, current_diagnostics);
        let resolution = ResolutionStatisticsDiff::between(previous_resolution, current_resolution);
        let previous_report = SemanticGraphReport::from_graph_diagnostics_and_references(
            previous_graph,
            previous_diagnostics,
            previous_resolution,
        );
        let current_report = SemanticGraphReport::from_graph_diagnostics_and_references(
            current_graph,
            current_diagnostics,
            current_resolution,
        );
        let report = GraphReportDiff::between(&previous_report, &current_report);
        let provenance = ProvenanceCoverageDiff::between(
            previous_report.provenance(),
            current_report.provenance(),
        );
        let summary = BuildDiffSummary::new(
            graph.summary(),
            diagnostics.summary(),
            resolution.changed_metrics().len(),
            report.changed_metric_count(),
            provenance.changed_metrics().len(),
        );

        Self {
            graph,
            diagnostics,
            resolution,
            report,
            provenance,
            summary,
        }
    }

    /// Returns the reused graph diff for nodes and edges.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraphDiff {
        &self.graph
    }

    /// Returns semantic diagnostic changes.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticDiff {
        &self.diagnostics
    }

    /// Returns reference resolution counter changes.
    #[must_use]
    pub const fn resolution(&self) -> &ResolutionStatisticsDiff {
        &self.resolution
    }

    /// Returns graph quality report aggregate changes.
    #[must_use]
    pub const fn report(&self) -> &GraphReportDiff {
        &self.report
    }

    /// Returns provenance coverage aggregate changes.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceCoverageDiff {
        &self.provenance
    }

    /// Returns compact build diff counters.
    #[must_use]
    pub const fn summary(&self) -> BuildDiffSummary {
        self.summary
    }

    /// Returns `true` when no graph entities, diagnostics or metrics changed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.summary.total_changes() == 0
    }
}

/// Compact counters for a full build result diff.
///
/// `total_changes` is the sum of changed graph entities, diagnostic records,
/// changed resolution metrics, changed report metrics and changed provenance
/// coverage metrics. Numeric delta magnitudes are not counted as separate
/// events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildDiffSummary {
    nodes: usize,
    edges: usize,
    diagnostics: usize,
    resolution_metrics: usize,
    report_metrics: usize,
    provenance_coverage: usize,
}

impl BuildDiffSummary {
    const fn new(
        graph: GraphDiffSummary,
        diagnostics: DiagnosticDiffSummary,
        resolution_metric_changes: usize,
        report_metric_changes: usize,
        provenance_coverage_changes: usize,
    ) -> Self {
        Self {
            nodes: graph.nodes_added() + graph.nodes_removed() + graph.nodes_modified(),
            edges: graph.edges_added() + graph.edges_removed() + graph.edges_modified(),
            diagnostics: diagnostics.total_changes(),
            resolution_metrics: resolution_metric_changes,
            report_metrics: report_metric_changes,
            provenance_coverage: provenance_coverage_changes,
        }
    }

    /// Returns changed graph node records.
    #[must_use]
    pub const fn node_changes(self) -> usize {
        self.nodes
    }

    /// Returns changed graph edge records.
    #[must_use]
    pub const fn edge_changes(self) -> usize {
        self.edges
    }

    /// Returns added, removed and modified diagnostics.
    #[must_use]
    pub const fn diagnostic_changes(self) -> usize {
        self.diagnostics
    }

    /// Returns changed resolution metric entries.
    #[must_use]
    pub const fn resolution_metric_changes(self) -> usize {
        self.resolution_metrics
    }

    /// Returns changed graph quality report metric entries.
    #[must_use]
    pub const fn report_metric_changes(self) -> usize {
        self.report_metrics
    }

    /// Returns changed provenance coverage metric entries.
    #[must_use]
    pub const fn provenance_coverage_changes(self) -> usize {
        self.provenance_coverage
    }

    /// Returns the total number of changed records and changed metric entries.
    #[must_use]
    pub const fn total_changes(self) -> usize {
        self.nodes
            + self.edges
            + self.diagnostics
            + self.resolution_metrics
            + self.report_metrics
            + self.provenance_coverage
    }
}

/// Deterministic semantic diagnostic diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticDiff {
    added: Vec<DiagnosticChange>,
    removed: Vec<DiagnosticChange>,
    modified: Vec<DiagnosticChange>,
    summary: DiagnosticDiffSummary,
}

impl DiagnosticDiff {
    /// Compares diagnostics in any input order.
    #[must_use]
    pub fn between(previous: &[SemanticDiagnostic], current: &[SemanticDiagnostic]) -> Self {
        let previous = diagnostic_index(previous);
        let current = diagnostic_index(current);
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        for (identity, previous_diagnostic) in &previous {
            match current.get(identity) {
                Some(current_diagnostic) => {
                    let aspects =
                        diagnostic_modified_aspects(previous_diagnostic, current_diagnostic);
                    if !aspects.is_empty() {
                        modified.push(DiagnosticChange::modified(
                            identity.clone(),
                            previous_diagnostic.clone(),
                            current_diagnostic.clone(),
                            aspects,
                        ));
                    }
                }
                None => removed.push(DiagnosticChange::removed(
                    identity.clone(),
                    previous_diagnostic.clone(),
                )),
            }
        }

        for (identity, current_diagnostic) in &current {
            if !previous.contains_key(identity) {
                added.push(DiagnosticChange::added(
                    identity.clone(),
                    current_diagnostic.clone(),
                ));
            }
        }

        let summary = DiagnosticDiffSummary::new(added.len(), removed.len(), modified.len());

        Self {
            added,
            removed,
            modified,
            summary,
        }
    }

    /// Returns diagnostics present only in the current build result.
    #[must_use]
    pub fn added(&self) -> &[DiagnosticChange] {
        &self.added
    }

    /// Returns diagnostics present only in the previous build result.
    #[must_use]
    pub fn removed(&self) -> &[DiagnosticChange] {
        &self.removed
    }

    /// Returns diagnostics with the same identity and changed observable content.
    #[must_use]
    pub fn modified(&self) -> &[DiagnosticChange] {
        &self.modified
    }

    /// Returns compact diagnostic diff counters.
    #[must_use]
    pub const fn summary(&self) -> DiagnosticDiffSummary {
        self.summary
    }
}

/// Stable semantic diagnostic identity used by build result diff.
///
/// The identity includes typed diagnostic code, problem kind, optional source
/// node and semantic reference context. Severity, message, expected kinds,
/// actual kind, candidates and provenance are observable content and may
/// produce a modified diagnostic for the same identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticIdentity {
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
    source_node: Option<EntityId>,
    reference: SemanticReference,
}

impl DiagnosticIdentity {
    fn from_diagnostic(diagnostic: &SemanticDiagnostic) -> Self {
        Self {
            code: diagnostic.code(),
            kind: diagnostic.kind(),
            source_node: diagnostic.source_node().cloned(),
            reference: diagnostic.reference().clone(),
        }
    }

    /// Returns the diagnostic code.
    #[must_use]
    pub const fn code(&self) -> SemanticDiagnosticCode {
        self.code
    }

    /// Returns the diagnostic problem kind.
    #[must_use]
    pub const fn kind(&self) -> SemanticDiagnosticKind {
        self.kind
    }

    /// Returns the optional source node identity.
    #[must_use]
    pub const fn source_node(&self) -> Option<&EntityId> {
        self.source_node.as_ref()
    }

    /// Returns the semantic reference context.
    #[must_use]
    pub const fn reference(&self) -> &SemanticReference {
        &self.reference
    }
}

/// Category of a semantic diagnostic change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticChangeKind {
    /// The diagnostic exists only in the current build result.
    Added,
    /// The diagnostic exists only in the previous build result.
    Removed,
    /// The diagnostic exists in both results but observable content changed.
    Modified,
}

/// Modified diagnostic aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticModifiedAspect {
    /// Diagnostic severity changed.
    Severity,
    /// Human-readable message changed.
    Message,
    /// Expected node kind list changed.
    ExpectedKinds,
    /// Actual node kind changed.
    ActualKind,
    /// Candidate target list changed.
    Candidates,
    /// Diagnostic provenance changed after order-insensitive normalization.
    Provenance,
}

/// Directional semantic diagnostic change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticChange {
    identity: DiagnosticIdentity,
    kind: DiagnosticChangeKind,
    previous: Option<SemanticDiagnostic>,
    current: Option<SemanticDiagnostic>,
    modified_aspects: Vec<DiagnosticModifiedAspect>,
}

impl DiagnosticChange {
    fn added(identity: DiagnosticIdentity, diagnostic: SemanticDiagnostic) -> Self {
        Self {
            identity,
            kind: DiagnosticChangeKind::Added,
            previous: None,
            current: Some(diagnostic),
            modified_aspects: Vec::new(),
        }
    }

    fn removed(identity: DiagnosticIdentity, diagnostic: SemanticDiagnostic) -> Self {
        Self {
            identity,
            kind: DiagnosticChangeKind::Removed,
            previous: Some(diagnostic),
            current: None,
            modified_aspects: Vec::new(),
        }
    }

    fn modified(
        identity: DiagnosticIdentity,
        previous: SemanticDiagnostic,
        current: SemanticDiagnostic,
        modified_aspects: Vec<DiagnosticModifiedAspect>,
    ) -> Self {
        Self {
            identity,
            kind: DiagnosticChangeKind::Modified,
            previous: Some(previous),
            current: Some(current),
            modified_aspects,
        }
    }

    /// Returns the diagnostic identity.
    #[must_use]
    pub const fn identity(&self) -> &DiagnosticIdentity {
        &self.identity
    }

    /// Returns the change category.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticChangeKind {
        self.kind
    }

    /// Returns the previous diagnostic for removed and modified changes.
    #[must_use]
    pub const fn previous(&self) -> Option<&SemanticDiagnostic> {
        self.previous.as_ref()
    }

    /// Returns the current diagnostic for added and modified changes.
    #[must_use]
    pub const fn current(&self) -> Option<&SemanticDiagnostic> {
        self.current.as_ref()
    }

    /// Returns typed modified aspects.
    #[must_use]
    pub fn modified_aspects(&self) -> &[DiagnosticModifiedAspect] {
        &self.modified_aspects
    }
}

/// Compact diagnostic diff counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticDiffSummary {
    added: usize,
    removed: usize,
    modified: usize,
}

impl DiagnosticDiffSummary {
    const fn new(added: usize, removed: usize, modified: usize) -> Self {
        Self {
            added,
            removed,
            modified,
        }
    }

    /// Returns added diagnostics.
    #[must_use]
    pub const fn added(self) -> usize {
        self.added
    }

    /// Returns removed diagnostics.
    #[must_use]
    pub const fn removed(self) -> usize {
        self.removed
    }

    /// Returns modified diagnostics.
    #[must_use]
    pub const fn modified(self) -> usize {
        self.modified
    }

    /// Returns added + removed + modified diagnostics.
    #[must_use]
    pub const fn total_changes(self) -> usize {
        self.added + self.removed + self.modified
    }
}

/// Direction of a counter change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CountChangeDirection {
    /// The counter increased.
    Increased,
    /// The counter decreased.
    Decreased,
}

/// Safe unsigned counter delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountDelta {
    direction: CountChangeDirection,
    magnitude: usize,
}

impl CountDelta {
    fn between(previous: usize, current: usize) -> Option<Self> {
        match current.cmp(&previous) {
            std::cmp::Ordering::Greater => Some(Self {
                direction: CountChangeDirection::Increased,
                magnitude: current - previous,
            }),
            std::cmp::Ordering::Less => Some(Self {
                direction: CountChangeDirection::Decreased,
                magnitude: previous - current,
            }),
            std::cmp::Ordering::Equal => None,
        }
    }

    /// Returns whether the counter increased or decreased.
    #[must_use]
    pub const fn direction(self) -> CountChangeDirection {
        self.direction
    }

    /// Returns the absolute counter change magnitude.
    #[must_use]
    pub const fn magnitude(self) -> usize {
        self.magnitude
    }
}

/// Previous/current values for a changed counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountChange<K> {
    key: K,
    previous: usize,
    current: usize,
    delta: CountDelta,
}

impl<K> CountChange<K> {
    fn new(key: K, previous: usize, current: usize) -> Option<Self> {
        CountDelta::between(previous, current).map(|delta| Self {
            key,
            previous,
            current,
            delta,
        })
    }

    /// Returns the typed metric or distribution key.
    #[must_use]
    pub const fn key(&self) -> &K {
        &self.key
    }

    /// Returns the previous counter value.
    #[must_use]
    pub const fn previous(&self) -> usize {
        self.previous
    }

    /// Returns the current counter value.
    #[must_use]
    pub const fn current(&self) -> usize {
        self.current
    }

    /// Returns the safe unsigned delta.
    #[must_use]
    pub const fn delta(&self) -> CountDelta {
        self.delta
    }
}

/// Resolution statistics metric key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionStatisticsMetric {
    /// Total processed references.
    TotalReferences,
    /// Raw references with malformed source format.
    MalformedFormatReferences,
    /// Raw references using unsupported source prefixes.
    UnsupportedPrefixReferences,
    /// Successfully resolved references.
    ResolvedReferences,
    /// Unresolved references.
    UnresolvedReferences,
    /// Ambiguous references.
    AmbiguousReferences,
    /// References with incompatible target kind.
    IncompatibleTargetKindReferences,
    /// Invalid owner references.
    InvalidOwnerReferences,
    /// Duplicate edge reference requests.
    DuplicateEdgeRequests,
    /// References with provenance.
    ReferencesWithProvenance,
    /// References without provenance.
    ReferencesWithoutProvenance,
}

/// Deterministic reference resolution statistics diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionStatisticsDiff {
    changed_metrics: Vec<CountChange<ResolutionStatisticsMetric>>,
}

impl ResolutionStatisticsDiff {
    /// Compares reference resolution statistics.
    #[must_use]
    pub fn between(
        previous: SemanticReferenceStatistics,
        current: SemanticReferenceStatistics,
    ) -> Self {
        let metrics = [
            (
                ResolutionStatisticsMetric::TotalReferences,
                previous.total(),
                current.total(),
            ),
            (
                ResolutionStatisticsMetric::MalformedFormatReferences,
                previous.malformed_format(),
                current.malformed_format(),
            ),
            (
                ResolutionStatisticsMetric::UnsupportedPrefixReferences,
                previous.unsupported_prefix(),
                current.unsupported_prefix(),
            ),
            (
                ResolutionStatisticsMetric::ResolvedReferences,
                previous.resolved(),
                current.resolved(),
            ),
            (
                ResolutionStatisticsMetric::UnresolvedReferences,
                previous.unresolved(),
                current.unresolved(),
            ),
            (
                ResolutionStatisticsMetric::AmbiguousReferences,
                previous.ambiguous(),
                current.ambiguous(),
            ),
            (
                ResolutionStatisticsMetric::IncompatibleTargetKindReferences,
                previous.incompatible_target_kind(),
                current.incompatible_target_kind(),
            ),
            (
                ResolutionStatisticsMetric::InvalidOwnerReferences,
                previous.invalid_owner_reference(),
                current.invalid_owner_reference(),
            ),
            (
                ResolutionStatisticsMetric::DuplicateEdgeRequests,
                previous.duplicate_edge_request(),
                current.duplicate_edge_request(),
            ),
            (
                ResolutionStatisticsMetric::ReferencesWithProvenance,
                previous.with_provenance(),
                current.with_provenance(),
            ),
            (
                ResolutionStatisticsMetric::ReferencesWithoutProvenance,
                previous.without_provenance(),
                current.without_provenance(),
            ),
        ];

        Self {
            changed_metrics: metrics
                .into_iter()
                .filter_map(|(key, previous, current)| CountChange::new(key, previous, current))
                .collect(),
        }
    }

    /// Returns changed resolution metrics.
    #[must_use]
    pub fn changed_metrics(&self) -> &[CountChange<ResolutionStatisticsMetric>] {
        &self.changed_metrics
    }
}

/// Graph quality scalar metric key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GraphReportMetric {
    /// Total graph nodes.
    TotalNodes,
    /// Total graph edges.
    TotalEdges,
    /// Total diagnostics in successful build result.
    TotalDiagnostics,
    /// Recoverable diagnostics in successful build result.
    RecoverableDiagnostics,
    /// Nodes with provenance.
    NodesWithProvenance,
    /// Nodes without provenance.
    NodesWithoutProvenance,
    /// Edges with provenance.
    EdgesWithProvenance,
    /// Edges without provenance.
    EdgesWithoutProvenance,
    /// Edges with missing source node.
    EdgesMissingSources,
    /// Edges with missing target node.
    EdgesMissingTargets,
    /// Diagnostics with provenance.
    DiagnosticsWithProvenance,
    /// Diagnostics without provenance.
    DiagnosticsWithoutProvenance,
}

/// Deterministic graph quality report diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphReportDiff {
    scalar_metrics: Vec<CountChange<GraphReportMetric>>,
    nodes_by_kind: Vec<CountChange<NodeKind>>,
    edges_by_kind: Vec<CountChange<EdgeKind>>,
    diagnostics_by_code: Vec<CountChange<SemanticDiagnosticCode>>,
    diagnostics_by_severity: Vec<CountChange<SemanticDiagnosticSeverity>>,
    diagnostics_by_kind: Vec<CountChange<SemanticDiagnosticKind>>,
}

impl GraphReportDiff {
    /// Compares two graph quality reports.
    #[must_use]
    pub fn between(previous: &SemanticGraphReport, current: &SemanticGraphReport) -> Self {
        let scalar_metrics = [
            (
                GraphReportMetric::TotalNodes,
                previous.graph().total_nodes(),
                current.graph().total_nodes(),
            ),
            (
                GraphReportMetric::TotalEdges,
                previous.graph().total_edges(),
                current.graph().total_edges(),
            ),
            (
                GraphReportMetric::TotalDiagnostics,
                previous.graph().total_diagnostics(),
                current.graph().total_diagnostics(),
            ),
            (
                GraphReportMetric::RecoverableDiagnostics,
                previous.graph().recoverable_diagnostics(),
                current.graph().recoverable_diagnostics(),
            ),
            (
                GraphReportMetric::NodesWithProvenance,
                previous.nodes().with_provenance(),
                current.nodes().with_provenance(),
            ),
            (
                GraphReportMetric::NodesWithoutProvenance,
                previous.nodes().without_provenance(),
                current.nodes().without_provenance(),
            ),
            (
                GraphReportMetric::EdgesWithProvenance,
                previous.edges().with_provenance(),
                current.edges().with_provenance(),
            ),
            (
                GraphReportMetric::EdgesWithoutProvenance,
                previous.edges().without_provenance(),
                current.edges().without_provenance(),
            ),
            (
                GraphReportMetric::EdgesMissingSources,
                previous.edges().missing_sources(),
                current.edges().missing_sources(),
            ),
            (
                GraphReportMetric::EdgesMissingTargets,
                previous.edges().missing_targets(),
                current.edges().missing_targets(),
            ),
            (
                GraphReportMetric::DiagnosticsWithProvenance,
                previous.diagnostics().with_provenance(),
                current.diagnostics().with_provenance(),
            ),
            (
                GraphReportMetric::DiagnosticsWithoutProvenance,
                previous.diagnostics().without_provenance(),
                current.diagnostics().without_provenance(),
            ),
        ]
        .into_iter()
        .filter_map(|(key, previous, current)| CountChange::new(key, previous, current))
        .collect();

        Self {
            scalar_metrics,
            nodes_by_kind: diff_distribution(previous.nodes().by_kind(), current.nodes().by_kind()),
            edges_by_kind: diff_distribution(previous.edges().by_kind(), current.edges().by_kind()),
            diagnostics_by_code: diff_distribution(
                previous.diagnostics().by_code(),
                current.diagnostics().by_code(),
            ),
            diagnostics_by_severity: diff_distribution(
                previous.diagnostics().by_severity(),
                current.diagnostics().by_severity(),
            ),
            diagnostics_by_kind: diff_distribution(
                previous.diagnostics().by_kind(),
                current.diagnostics().by_kind(),
            ),
        }
    }

    /// Returns changed scalar report metrics.
    #[must_use]
    pub fn scalar_metrics(&self) -> &[CountChange<GraphReportMetric>] {
        &self.scalar_metrics
    }

    /// Returns changed node distribution entries.
    #[must_use]
    pub fn nodes_by_kind(&self) -> &[CountChange<NodeKind>] {
        &self.nodes_by_kind
    }

    /// Returns changed edge distribution entries.
    #[must_use]
    pub fn edges_by_kind(&self) -> &[CountChange<EdgeKind>] {
        &self.edges_by_kind
    }

    /// Returns changed diagnostic code distribution entries.
    #[must_use]
    pub fn diagnostics_by_code(&self) -> &[CountChange<SemanticDiagnosticCode>] {
        &self.diagnostics_by_code
    }

    /// Returns changed diagnostic severity distribution entries.
    #[must_use]
    pub fn diagnostics_by_severity(&self) -> &[CountChange<SemanticDiagnosticSeverity>] {
        &self.diagnostics_by_severity
    }

    /// Returns changed diagnostic kind distribution entries.
    #[must_use]
    pub fn diagnostics_by_kind(&self) -> &[CountChange<SemanticDiagnosticKind>] {
        &self.diagnostics_by_kind
    }

    /// Returns the number of changed report metric entries.
    #[must_use]
    pub fn changed_metric_count(&self) -> usize {
        self.scalar_metrics.len()
            + self.nodes_by_kind.len()
            + self.edges_by_kind.len()
            + self.diagnostics_by_code.len()
            + self.diagnostics_by_severity.len()
            + self.diagnostics_by_kind.len()
    }
}

/// Provenance coverage metric key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProvenanceCoverageMetric {
    /// Nodes with provenance.
    NodesWithProvenance,
    /// Nodes without provenance.
    NodesWithoutProvenance,
    /// Edges with provenance.
    EdgesWithProvenance,
    /// Edges without provenance.
    EdgesWithoutProvenance,
    /// Diagnostics with provenance.
    DiagnosticsWithProvenance,
    /// Diagnostics without provenance.
    DiagnosticsWithoutProvenance,
    /// References with provenance.
    ReferencesWithProvenance,
    /// References without provenance.
    ReferencesWithoutProvenance,
}

/// Deterministic provenance coverage diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceCoverageDiff {
    changed_metrics: Vec<CountChange<ProvenanceCoverageMetric>>,
}

impl ProvenanceCoverageDiff {
    /// Compares provenance coverage summaries.
    #[must_use]
    pub fn between(
        previous: &crate::ProvenanceCoverageSummary,
        current: &crate::ProvenanceCoverageSummary,
    ) -> Self {
        let changed_metrics = [
            (
                ProvenanceCoverageMetric::NodesWithProvenance,
                previous.nodes_with_provenance(),
                current.nodes_with_provenance(),
            ),
            (
                ProvenanceCoverageMetric::NodesWithoutProvenance,
                previous.nodes_without_provenance(),
                current.nodes_without_provenance(),
            ),
            (
                ProvenanceCoverageMetric::EdgesWithProvenance,
                previous.edges_with_provenance(),
                current.edges_with_provenance(),
            ),
            (
                ProvenanceCoverageMetric::EdgesWithoutProvenance,
                previous.edges_without_provenance(),
                current.edges_without_provenance(),
            ),
            (
                ProvenanceCoverageMetric::DiagnosticsWithProvenance,
                previous.diagnostics_with_provenance(),
                current.diagnostics_with_provenance(),
            ),
            (
                ProvenanceCoverageMetric::DiagnosticsWithoutProvenance,
                previous.diagnostics_without_provenance(),
                current.diagnostics_without_provenance(),
            ),
            (
                ProvenanceCoverageMetric::ReferencesWithProvenance,
                previous.references_with_provenance(),
                current.references_with_provenance(),
            ),
            (
                ProvenanceCoverageMetric::ReferencesWithoutProvenance,
                previous.references_without_provenance(),
                current.references_without_provenance(),
            ),
        ]
        .into_iter()
        .filter_map(|(key, previous, current)| CountChange::new(key, previous, current))
        .collect();

        Self { changed_metrics }
    }

    /// Returns changed provenance coverage metrics.
    #[must_use]
    pub fn changed_metrics(&self) -> &[CountChange<ProvenanceCoverageMetric>] {
        &self.changed_metrics
    }
}

fn diagnostic_index(
    diagnostics: &[SemanticDiagnostic],
) -> BTreeMap<DiagnosticIdentity, SemanticDiagnostic> {
    let mut diagnostics = diagnostics.to_vec();
    diagnostics.sort();
    diagnostics
        .into_iter()
        .map(|diagnostic| (DiagnosticIdentity::from_diagnostic(&diagnostic), diagnostic))
        .collect()
}

fn diagnostic_modified_aspects(
    previous: &SemanticDiagnostic,
    current: &SemanticDiagnostic,
) -> Vec<DiagnosticModifiedAspect> {
    let mut aspects = Vec::new();
    if previous.severity() != current.severity() {
        aspects.push(DiagnosticModifiedAspect::Severity);
    }
    if previous.message() != current.message() {
        aspects.push(DiagnosticModifiedAspect::Message);
    }
    if previous.expected_kinds() != current.expected_kinds() {
        aspects.push(DiagnosticModifiedAspect::ExpectedKinds);
    }
    if previous.actual_kind() != current.actual_kind() {
        aspects.push(DiagnosticModifiedAspect::ActualKind);
    }
    if previous.candidates() != current.candidates() {
        aspects.push(DiagnosticModifiedAspect::Candidates);
    }
    if normalized_provenance(previous.provenance()) != normalized_provenance(current.provenance()) {
        aspects.push(DiagnosticModifiedAspect::Provenance);
    }

    aspects
}

fn diff_distribution<K>(
    previous: &BTreeMap<K, usize>,
    current: &BTreeMap<K, usize>,
) -> Vec<CountChange<K>>
where
    K: Clone + Ord,
{
    let keys = previous
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    keys.into_iter()
        .filter_map(|key| {
            let previous = previous.get(&key).copied().unwrap_or_default();
            let current = current.get(&key).copied().unwrap_or_default();
            CountChange::new(key, previous, current)
        })
        .collect()
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

    use crate::{
        Confidence, CountChangeDirection, DiagnosticChangeKind, DiagnosticModifiedAspect, EdgeKind,
        FactOrigin, GraphNode, NodeKind, ProducerId, Provenance, ResolutionState,
        SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphBuildDiff, SemanticReference,
        SemanticReferenceOutcome, SemanticReferenceStatistics,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance(source: &str) -> Provenance {
        Provenance::new(
            Some(id(source)),
            ProducerId::new("oneagent.graph.build-diff.tests"),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn graph_with_node(id_value: &str, name_value: &str) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id(id_value),
            name(name_value),
            NodeKind::Module,
        ));
        graph
    }

    fn diagnostic(message: &str) -> SemanticDiagnostic {
        SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            message,
            SemanticReference::Name(name("Missing")),
        )
        .with_source_node(id("source.node"))
        .with_expected_kinds(vec![NodeKind::Module])
    }

    fn statistics(outcomes: &[SemanticReferenceOutcome]) -> SemanticReferenceStatistics {
        let mut statistics = SemanticReferenceStatistics::new();
        for outcome in outcomes {
            statistics.record(*outcome, true);
        }
        statistics
    }

    #[test]
    fn empty_build_results_create_empty_diff() {
        let graph = SemanticGraph::new();
        let diff = SemanticGraphBuildDiff::between(
            &graph,
            &[],
            SemanticReferenceStatistics::new(),
            &graph,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert!(diff.is_empty());
        assert_eq!(diff.summary().total_changes(), 0);
    }

    #[test]
    fn graph_diff_is_included_in_build_diff_summary() {
        let previous = graph_with_node("node.previous", "Previous");
        let current = graph_with_node("node.current", "Current");
        let diff = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert_eq!(diff.graph().added_nodes().len(), 1);
        assert_eq!(diff.graph().removed_nodes().len(), 1);
        assert_eq!(diff.summary().node_changes(), 2);
    }

    #[test]
    fn diagnostics_are_added_removed_and_modified_by_identity() {
        let graph = SemanticGraph::new();
        let mut previous_diagnostic = diagnostic("previous message");
        let mut current_diagnostic =
            diagnostic("current message").with_provenance(vec![provenance("diagnostic.current")]);
        current_diagnostic = current_diagnostic.with_actual_kind(NodeKind::Function);
        previous_diagnostic =
            previous_diagnostic.with_provenance(vec![provenance("diagnostic.previous")]);
        let added = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            "added",
            SemanticReference::Name(name("Ambiguous")),
        );
        let removed = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::InvalidOwnerReference,
            "removed",
            SemanticReference::OwnedChild {
                owner: id("owner"),
                child: id("child"),
            },
        );

        let diff = SemanticGraphBuildDiff::between(
            &graph,
            &[removed.clone(), previous_diagnostic],
            SemanticReferenceStatistics::new(),
            &graph,
            &[current_diagnostic, added.clone()],
            SemanticReferenceStatistics::new(),
        );

        assert_eq!(diff.diagnostics().added().len(), 1);
        assert_eq!(diff.diagnostics().removed().len(), 1);
        assert_eq!(diff.diagnostics().modified().len(), 1);
        assert_eq!(
            diff.diagnostics().added()[0].kind(),
            DiagnosticChangeKind::Added
        );
        assert_eq!(
            diff.diagnostics().modified()[0].modified_aspects(),
            &[
                DiagnosticModifiedAspect::Message,
                DiagnosticModifiedAspect::ActualKind,
                DiagnosticModifiedAspect::Provenance
            ]
        );
        assert_eq!(diff.summary().diagnostic_changes(), 3);
    }

    #[test]
    fn diagnostics_input_order_does_not_affect_diff() {
        let graph = SemanticGraph::new();
        let first = diagnostic("first");
        let second = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            "second",
            SemanticReference::Name(name("Ambiguous")),
        );

        let left = SemanticGraphBuildDiff::between(
            &graph,
            &[first.clone(), second.clone()],
            SemanticReferenceStatistics::new(),
            &graph,
            &[],
            SemanticReferenceStatistics::new(),
        );
        let right = SemanticGraphBuildDiff::between(
            &graph,
            &[second, first],
            SemanticReferenceStatistics::new(),
            &graph,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn resolution_statistics_track_increase_and_decrease_without_underflow() {
        let graph = SemanticGraph::new();
        let previous_statistics = statistics(&[
            SemanticReferenceOutcome::Resolved,
            SemanticReferenceOutcome::Resolved,
        ]);
        let current_statistics = statistics(&[
            SemanticReferenceOutcome::MalformedFormat,
            SemanticReferenceOutcome::UnsupportedPrefix,
            SemanticReferenceOutcome::Resolved,
            SemanticReferenceOutcome::Unresolved,
            SemanticReferenceOutcome::Ambiguous,
        ]);

        let diff = SemanticGraphBuildDiff::between(
            &graph,
            &[],
            previous_statistics,
            &graph,
            &[],
            current_statistics,
        );

        let resolved = diff
            .resolution()
            .changed_metrics()
            .iter()
            .find(|change| *change.key() == crate::ResolutionStatisticsMetric::ResolvedReferences)
            .expect("resolved count must change");

        assert_eq!(resolved.previous(), 2);
        assert_eq!(resolved.current(), 1);
        assert_eq!(
            resolved.delta().direction(),
            CountChangeDirection::Decreased
        );
        assert_eq!(resolved.delta().magnitude(), 1);
        for metric in [
            crate::ResolutionStatisticsMetric::MalformedFormatReferences,
            crate::ResolutionStatisticsMetric::UnsupportedPrefixReferences,
        ] {
            let change = diff
                .resolution()
                .changed_metrics()
                .iter()
                .find(|change| *change.key() == metric)
                .expect("new typed outcome count must change");
            assert_eq!(change.previous(), 0);
            assert_eq!(change.current(), 1);
            assert_eq!(change.delta().direction(), CountChangeDirection::Increased);
        }
    }

    #[test]
    fn report_and_provenance_metrics_are_changed_deterministically() {
        let previous = graph_with_node("node.previous", "Previous");
        let mut current = SemanticGraph::new();
        current.insert_node(GraphNode::new_with_provenance(
            id("node.previous"),
            name("Previous"),
            NodeKind::Module,
            vec![provenance("node.previous")],
        ));
        current.insert_node(GraphNode::new(
            id("node.current"),
            name("Current"),
            NodeKind::Function,
        ));

        let diff = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert!(
            diff.report()
                .scalar_metrics()
                .iter()
                .any(|change| *change.key() == crate::GraphReportMetric::TotalNodes)
        );
        assert!(
            diff.report()
                .nodes_by_kind()
                .iter()
                .any(|change| *change.key() == NodeKind::Function)
        );
        assert!(diff
            .provenance()
            .changed_metrics()
            .iter()
            .any(|change| *change.key() == crate::ProvenanceCoverageMetric::NodesWithProvenance));
    }

    #[test]
    fn repeated_comparison_is_identical_and_does_not_mutate_inputs() {
        let previous = graph_with_node("node.previous", "Previous");
        let current = graph_with_node("node.current", "Current");
        let previous_report = previous.report();
        let current_report = current.report();

        let first = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );
        let second = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert_eq!(first, second);
        assert_eq!(previous.report(), previous_report);
        assert_eq!(current.report(), current_report);
    }

    #[test]
    fn reverse_comparison_inverts_direction() {
        let previous = graph_with_node("node.previous", "Previous");
        let current = graph_with_node("node.current", "Current");

        let forward = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );
        let reverse = SemanticGraphBuildDiff::between(
            &current,
            &[],
            SemanticReferenceStatistics::new(),
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert_eq!(
            forward.graph().added_nodes()[0].id().as_str(),
            "node.current"
        );
        assert_eq!(
            reverse.graph().added_nodes()[0].id().as_str(),
            "node.previous"
        );
    }

    #[test]
    fn edge_distribution_changes_are_reported() {
        let mut previous = SemanticGraph::new();
        let mut current = SemanticGraph::new();
        previous.insert_node(GraphNode::new(
            id("source"),
            name("Source"),
            NodeKind::Module,
        ));
        previous.insert_node(GraphNode::new(
            id("target"),
            name("Target"),
            NodeKind::Function,
        ));
        current.insert_node(GraphNode::new(
            id("source"),
            name("Source"),
            NodeKind::Module,
        ));
        current.insert_node(GraphNode::new(
            id("target"),
            name("Target"),
            NodeKind::Function,
        ));
        current
            .insert_edge(crate::GraphEdge::new(
                id("source"),
                id("target"),
                EdgeKind::Calls,
            ))
            .expect("edge must be valid");

        let diff = SemanticGraphBuildDiff::between(
            &previous,
            &[],
            SemanticReferenceStatistics::new(),
            &current,
            &[],
            SemanticReferenceStatistics::new(),
        );

        assert!(
            diff.report()
                .edges_by_kind()
                .iter()
                .any(|change| *change.key() == EdgeKind::Calls)
        );
    }
}
