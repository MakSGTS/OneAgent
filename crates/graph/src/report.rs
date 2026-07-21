//! Deterministic semantic graph quality reports.

use std::collections::BTreeMap;

use crate::{
    EdgeKind, ResolutionError, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticDiagnosticSeverity, SemanticGraph,
};

use crate::NodeKind;

/// Deterministic aggregate report for a semantic graph build snapshot.
///
/// The report contains aggregated metrics only. It does not copy graph nodes,
/// graph edges or diagnostics, does not mutate the graph and does not perform
/// semantic resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGraphReport {
    graph: GraphSummary,
    nodes: NodeSummary,
    edges: EdgeSummary,
    diagnostics: DiagnosticSummary,
    resolution: SemanticReferenceStatistics,
    provenance: ProvenanceCoverageSummary,
}

impl SemanticGraphReport {
    /// Builds a graph-only report with no diagnostics or reference statistics.
    #[must_use]
    pub fn from_graph(graph: &SemanticGraph) -> Self {
        Self::from_graph_diagnostics_and_references(graph, &[], SemanticReferenceStatistics::new())
    }

    /// Builds a report from a graph and ordered or unordered diagnostics.
    ///
    /// Reference totals are not inferred from graph edges or diagnostics. Use
    /// [`Self::from_graph_diagnostics_and_references`] when the build pipeline
    /// preserves explicit reference outcome counters.
    #[must_use]
    pub fn from_graph_and_diagnostics(
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
    ) -> Self {
        Self::from_graph_diagnostics_and_references(
            graph,
            diagnostics,
            SemanticReferenceStatistics::new(),
        )
    }

    /// Builds a report from a graph, diagnostics and semantic reference statistics.
    ///
    /// Reference resolution rate is defined as `resolved / total`, where `total`
    /// is the number of semantic references explicitly processed by the build
    /// pipeline. A zero-reference build is represented as `0 / 0`; percentage
    /// formatting is intentionally left to presentation layers.
    #[must_use]
    pub fn from_graph_diagnostics_and_references(
        graph: &SemanticGraph,
        diagnostics: &[SemanticDiagnostic],
        reference_statistics: SemanticReferenceStatistics,
    ) -> Self {
        let nodes = NodeSummary::from_graph(graph);
        let edges = EdgeSummary::from_graph(graph);
        let diagnostic_summary = DiagnosticSummary::from_diagnostics(diagnostics);
        let graph_summary = GraphSummary::new(
            graph.node_count(),
            graph.edge_count(),
            diagnostic_summary.total(),
            diagnostic_summary.recoverable(),
        );
        let provenance = ProvenanceCoverageSummary::from_summaries(
            &nodes,
            &edges,
            &diagnostic_summary,
            reference_statistics,
        );

        Self {
            graph: graph_summary,
            nodes,
            edges,
            diagnostics: diagnostic_summary,
            resolution: reference_statistics,
            provenance,
        }
    }

    /// Returns graph-wide counters.
    #[must_use]
    pub const fn graph(&self) -> &GraphSummary {
        &self.graph
    }

    /// Returns node distribution and provenance counters.
    #[must_use]
    pub const fn nodes(&self) -> &NodeSummary {
        &self.nodes
    }

    /// Returns edge distribution, provenance and structural integrity counters.
    #[must_use]
    pub const fn edges(&self) -> &EdgeSummary {
        &self.edges
    }

    /// Returns diagnostic distribution and provenance counters.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticSummary {
        &self.diagnostics
    }

    /// Returns semantic reference resolution counters.
    #[must_use]
    pub const fn resolution(&self) -> &SemanticReferenceStatistics {
        &self.resolution
    }

    /// Returns combined provenance coverage counters.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceCoverageSummary {
        &self.provenance
    }
}

/// Graph-wide counters shared by graph-only and build reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphSummary {
    total_nodes: usize,
    total_edges: usize,
    total_diagnostics: usize,
    recoverable_diagnostics: usize,
}

impl GraphSummary {
    const fn new(
        total_nodes: usize,
        total_edges: usize,
        total_diagnostics: usize,
        recoverable_diagnostics: usize,
    ) -> Self {
        Self {
            total_nodes,
            total_edges,
            total_diagnostics,
            recoverable_diagnostics,
        }
    }

    /// Returns the number of graph nodes.
    #[must_use]
    pub const fn total_nodes(self) -> usize {
        self.total_nodes
    }

    /// Returns the number of graph edges.
    #[must_use]
    pub const fn total_edges(self) -> usize {
        self.total_edges
    }

    /// Returns the number of diagnostics included in the report.
    #[must_use]
    pub const fn total_diagnostics(self) -> usize {
        self.total_diagnostics
    }

    /// Returns the number of recoverable diagnostics in a successful build result.
    #[must_use]
    pub const fn recoverable_diagnostics(self) -> usize {
        self.recoverable_diagnostics
    }
}

/// Deterministic node aggregate metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSummary {
    total: usize,
    by_kind: BTreeMap<NodeKind, usize>,
    with_provenance: usize,
    without_provenance: usize,
}

impl NodeSummary {
    fn from_graph(graph: &SemanticGraph) -> Self {
        let mut by_kind = BTreeMap::new();
        let mut with_provenance = 0;
        let mut without_provenance = 0;

        for node in graph.nodes() {
            *by_kind.entry(node.kind()).or_default() += 1;
            if node.provenance().is_empty() {
                without_provenance += 1;
            } else {
                with_provenance += 1;
            }
        }

        Self {
            total: graph.node_count(),
            by_kind,
            with_provenance,
            without_provenance,
        }
    }

    /// Returns the total number of nodes.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns deterministic node counts by typed node kind.
    #[must_use]
    pub const fn by_kind(&self) -> &BTreeMap<NodeKind, usize> {
        &self.by_kind
    }

    /// Returns the number of nodes with at least one provenance record.
    #[must_use]
    pub const fn with_provenance(&self) -> usize {
        self.with_provenance
    }

    /// Returns the number of nodes without provenance records.
    #[must_use]
    pub const fn without_provenance(&self) -> usize {
        self.without_provenance
    }
}

/// Deterministic edge aggregate metrics and structural integrity counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeSummary {
    total: usize,
    by_kind: BTreeMap<EdgeKind, usize>,
    with_provenance: usize,
    without_provenance: usize,
    missing_sources: usize,
    missing_targets: usize,
}

impl EdgeSummary {
    fn from_graph(graph: &SemanticGraph) -> Self {
        let mut by_kind = BTreeMap::new();
        let mut with_provenance = 0;
        let mut without_provenance = 0;
        let mut missing_sources = 0;
        let mut missing_targets = 0;

        for edge in graph.edges() {
            *by_kind.entry(edge.kind()).or_default() += 1;
            if edge.provenance().is_empty() {
                without_provenance += 1;
            } else {
                with_provenance += 1;
            }
            if graph.node(edge.source()).is_none() {
                missing_sources += 1;
            }
            if graph.node(edge.target()).is_none() {
                missing_targets += 1;
            }
        }

        Self {
            total: graph.edge_count(),
            by_kind,
            with_provenance,
            without_provenance,
            missing_sources,
            missing_targets,
        }
    }

    /// Returns the total number of edges.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns deterministic edge counts by typed edge kind.
    #[must_use]
    pub const fn by_kind(&self) -> &BTreeMap<EdgeKind, usize> {
        &self.by_kind
    }

    /// Returns the number of edges with at least one provenance record.
    #[must_use]
    pub const fn with_provenance(&self) -> usize {
        self.with_provenance
    }

    /// Returns the number of edges without provenance records.
    #[must_use]
    pub const fn without_provenance(&self) -> usize {
        self.without_provenance
    }

    /// Returns the number of edges whose source node is absent.
    #[must_use]
    pub const fn missing_sources(&self) -> usize {
        self.missing_sources
    }

    /// Returns the number of edges whose target node is absent.
    #[must_use]
    pub const fn missing_targets(&self) -> usize {
        self.missing_targets
    }
}

/// Deterministic diagnostic aggregate metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSummary {
    total: usize,
    recoverable: usize,
    by_code: BTreeMap<SemanticDiagnosticCode, usize>,
    by_severity: BTreeMap<SemanticDiagnosticSeverity, usize>,
    by_kind: BTreeMap<SemanticDiagnosticKind, usize>,
    with_provenance: usize,
    without_provenance: usize,
}

impl DiagnosticSummary {
    /// Builds deterministic diagnostic counters from diagnostics in any order.
    #[must_use]
    pub fn from_diagnostics(diagnostics: &[SemanticDiagnostic]) -> Self {
        let mut by_code = BTreeMap::new();
        let mut by_severity = BTreeMap::new();
        let mut by_kind = BTreeMap::new();
        let mut with_provenance = 0;
        let mut without_provenance = 0;

        for diagnostic in diagnostics {
            *by_code.entry(diagnostic.code()).or_default() += 1;
            *by_severity.entry(diagnostic.severity()).or_default() += 1;
            *by_kind.entry(diagnostic.kind()).or_default() += 1;
            if diagnostic.provenance().is_empty() {
                without_provenance += 1;
            } else {
                with_provenance += 1;
            }
        }

        Self {
            total: diagnostics.len(),
            recoverable: diagnostics.len(),
            by_code,
            by_severity,
            by_kind,
            with_provenance,
            without_provenance,
        }
    }

    /// Returns the total number of diagnostics.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns the number of recoverable diagnostics in a successful build result.
    #[must_use]
    pub const fn recoverable(&self) -> usize {
        self.recoverable
    }

    /// Returns deterministic diagnostic counts by stable diagnostic code.
    #[must_use]
    pub const fn by_code(&self) -> &BTreeMap<SemanticDiagnosticCode, usize> {
        &self.by_code
    }

    /// Returns deterministic diagnostic counts by severity.
    #[must_use]
    pub const fn by_severity(&self) -> &BTreeMap<SemanticDiagnosticSeverity, usize> {
        &self.by_severity
    }

    /// Returns deterministic diagnostic counts by semantic problem kind.
    #[must_use]
    pub const fn by_kind(&self) -> &BTreeMap<SemanticDiagnosticKind, usize> {
        &self.by_kind
    }

    /// Returns the number of diagnostics with at least one provenance record.
    #[must_use]
    pub const fn with_provenance(&self) -> usize {
        self.with_provenance
    }

    /// Returns the number of diagnostics without provenance records.
    #[must_use]
    pub const fn without_provenance(&self) -> usize {
        self.without_provenance
    }
}

/// Outcome of one processed semantic reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticReferenceOutcome {
    /// The reference was resolved to one target.
    Resolved,
    /// The reference did not resolve to any target.
    Unresolved,
    /// The reference resolved to multiple possible targets.
    Ambiguous,
    /// The reference target kind did not match the expected kind.
    IncompatibleTargetKind,
    /// The reference owner-child relation is invalid.
    InvalidOwnerReference,
    /// The reference requested a duplicate edge where duplication is diagnostic.
    DuplicateEdgeRequest,
}

impl SemanticReferenceOutcome {
    /// Maps a semantic resolution error to the matching reference outcome.
    #[must_use]
    pub const fn from_resolution_error(error: &ResolutionError) -> Self {
        match error {
            ResolutionError::MissingTarget { .. } => Self::Unresolved,
            ResolutionError::AmbiguousTarget { .. } => Self::Ambiguous,
            ResolutionError::IncompatibleNodeKind { .. } => Self::IncompatibleTargetKind,
            ResolutionError::InvalidOwnerReference { .. } => Self::InvalidOwnerReference,
        }
    }
}

/// Immutable counters for semantic references processed by a build pipeline.
///
/// These counters are accumulated while references are processed. They are not
/// reconstructed from graph edges or diagnostics, because references may be
/// deduplicated, unresolved or represented by diagnostics instead of edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticReferenceStatistics {
    total: usize,
    resolved: usize,
    unresolved: usize,
    ambiguous: usize,
    incompatible_target_kind: usize,
    invalid_owner_reference: usize,
    duplicate_edge_request: usize,
    with_provenance: usize,
    without_provenance: usize,
}

impl Default for SemanticReferenceStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticReferenceStatistics {
    /// Creates empty semantic reference statistics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total: 0,
            resolved: 0,
            unresolved: 0,
            ambiguous: 0,
            incompatible_target_kind: 0,
            invalid_owner_reference: 0,
            duplicate_edge_request: 0,
            with_provenance: 0,
            without_provenance: 0,
        }
    }

    /// Records one processed semantic reference.
    ///
    /// `has_provenance` describes whether the processed reference has source
    /// provenance in the build pipeline, not whether a resolved graph edge was
    /// eventually inserted.
    pub fn record(&mut self, outcome: SemanticReferenceOutcome, has_provenance: bool) {
        self.total += 1;
        match outcome {
            SemanticReferenceOutcome::Resolved => self.resolved += 1,
            SemanticReferenceOutcome::Unresolved => self.unresolved += 1,
            SemanticReferenceOutcome::Ambiguous => self.ambiguous += 1,
            SemanticReferenceOutcome::IncompatibleTargetKind => {
                self.incompatible_target_kind += 1;
            }
            SemanticReferenceOutcome::InvalidOwnerReference => {
                self.invalid_owner_reference += 1;
            }
            SemanticReferenceOutcome::DuplicateEdgeRequest => {
                self.duplicate_edge_request += 1;
            }
        }

        if has_provenance {
            self.with_provenance += 1;
        } else {
            self.without_provenance += 1;
        }
    }

    /// Returns the total number of processed references.
    #[must_use]
    pub const fn total(self) -> usize {
        self.total
    }

    /// Returns the number of successfully resolved references.
    #[must_use]
    pub const fn resolved(self) -> usize {
        self.resolved
    }

    /// Returns the number of references with no resolved target.
    #[must_use]
    pub const fn unresolved(self) -> usize {
        self.unresolved
    }

    /// Returns the number of ambiguous references.
    #[must_use]
    pub const fn ambiguous(self) -> usize {
        self.ambiguous
    }

    /// Returns the number of references resolved to an incompatible target kind.
    #[must_use]
    pub const fn incompatible_target_kind(self) -> usize {
        self.incompatible_target_kind
    }

    /// Returns the number of invalid owner-child references.
    #[must_use]
    pub const fn invalid_owner_reference(self) -> usize {
        self.invalid_owner_reference
    }

    /// Returns the number of duplicate edge requests recorded as reference outcomes.
    #[must_use]
    pub const fn duplicate_edge_request(self) -> usize {
        self.duplicate_edge_request
    }

    /// Returns the number of processed references with provenance.
    #[must_use]
    pub const fn with_provenance(self) -> usize {
        self.with_provenance
    }

    /// Returns the number of processed references without provenance.
    #[must_use]
    pub const fn without_provenance(self) -> usize {
        self.without_provenance
    }

    /// Returns the sum of all recorded resolution outcomes.
    #[must_use]
    pub const fn outcome_total(self) -> usize {
        self.resolved
            + self.unresolved
            + self.ambiguous
            + self.incompatible_target_kind
            + self.invalid_owner_reference
            + self.duplicate_edge_request
    }

    /// Returns `true` when no semantic references were processed.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.total == 0
    }

    /// Returns the deterministic resolved-to-total ratio.
    #[must_use]
    pub const fn resolution_rate(self) -> ResolutionRate {
        ResolutionRate::new(self.resolved, self.total)
    }
}

/// Rational semantic reference resolution rate.
///
/// The denominator is the number of processed references. When no references
/// were processed, the rate is represented as `0 / 0` and
/// [`Self::is_defined`] returns `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolutionRate {
    numerator: usize,
    denominator: usize,
}

impl ResolutionRate {
    const fn new(numerator: usize, denominator: usize) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns the number of resolved references.
    #[must_use]
    pub const fn numerator(self) -> usize {
        self.numerator
    }

    /// Returns the number of processed references.
    #[must_use]
    pub const fn denominator(self) -> usize {
        self.denominator
    }

    /// Returns `true` when the denominator is non-zero.
    #[must_use]
    pub const fn is_defined(self) -> bool {
        self.denominator != 0
    }
}

/// Combined provenance coverage metrics across report sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenanceCoverageSummary {
    nodes_with: usize,
    nodes_without: usize,
    edges_with: usize,
    edges_without: usize,
    diagnostics_with: usize,
    diagnostics_without: usize,
    references_with: usize,
    references_without: usize,
}

impl ProvenanceCoverageSummary {
    const fn from_summaries(
        nodes: &NodeSummary,
        edges: &EdgeSummary,
        diagnostics: &DiagnosticSummary,
        references: SemanticReferenceStatistics,
    ) -> Self {
        Self {
            nodes_with: nodes.with_provenance(),
            nodes_without: nodes.without_provenance(),
            edges_with: edges.with_provenance(),
            edges_without: edges.without_provenance(),
            diagnostics_with: diagnostics.with_provenance(),
            diagnostics_without: diagnostics.without_provenance(),
            references_with: references.with_provenance(),
            references_without: references.without_provenance(),
        }
    }

    /// Returns the number of nodes with provenance.
    #[must_use]
    pub const fn nodes_with_provenance(self) -> usize {
        self.nodes_with
    }

    /// Returns the number of nodes without provenance.
    #[must_use]
    pub const fn nodes_without_provenance(self) -> usize {
        self.nodes_without
    }

    /// Returns the number of edges with provenance.
    #[must_use]
    pub const fn edges_with_provenance(self) -> usize {
        self.edges_with
    }

    /// Returns the number of edges without provenance.
    #[must_use]
    pub const fn edges_without_provenance(self) -> usize {
        self.edges_without
    }

    /// Returns the number of diagnostics with provenance.
    #[must_use]
    pub const fn diagnostics_with_provenance(self) -> usize {
        self.diagnostics_with
    }

    /// Returns the number of diagnostics without provenance.
    #[must_use]
    pub const fn diagnostics_without_provenance(self) -> usize {
        self.diagnostics_without
    }

    /// Returns the number of processed references with provenance.
    #[must_use]
    pub const fn references_with_provenance(self) -> usize {
        self.references_with
    }

    /// Returns the number of processed references without provenance.
    #[must_use]
    pub const fn references_without_provenance(self) -> usize {
        self.references_without
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;

    use crate::{
        Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
        ResolutionError, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
        SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphReport,
        SemanticReference, SemanticReferenceOutcome, SemanticReferenceStatistics,
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
            ProducerId::new("oneagent.graph.report.tests"),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn diagnostic(
        code: SemanticDiagnosticCode,
        severity: SemanticDiagnosticSeverity,
        kind: SemanticDiagnosticKind,
        source: Option<&str>,
    ) -> SemanticDiagnostic {
        let diagnostic = SemanticDiagnostic::new(
            code,
            severity,
            kind,
            "diagnostic",
            SemanticReference::Name(name("Missing")),
        );

        if let Some(value) = source {
            diagnostic.with_provenance(vec![provenance(value)])
        } else {
            diagnostic
        }
    }

    #[test]
    fn empty_graph_creates_zero_report() {
        let graph = SemanticGraph::new();
        let report = SemanticGraphReport::from_graph(&graph);

        assert_eq!(report.graph().total_nodes(), 0);
        assert_eq!(report.graph().total_edges(), 0);
        assert_eq!(report.graph().total_diagnostics(), 0);
        assert!(report.nodes().by_kind().is_empty());
        assert!(report.edges().by_kind().is_empty());
        assert!(report.resolution().is_empty());
        assert_eq!(report.resolution().resolution_rate().numerator(), 0);
        assert_eq!(report.resolution().resolution_rate().denominator(), 0);
        assert!(!report.resolution().resolution_rate().is_defined());
    }

    #[test]
    fn nodes_are_counted_by_kind_and_provenance() {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new_with_provenance(
            id("module.sales"),
            name("SalesModule"),
            NodeKind::Module,
            vec![provenance("module.sales")],
        ));
        graph.insert_node(GraphNode::new(
            id("document.sales"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));
        graph.insert_node(GraphNode::new(
            id("procedure.post"),
            name("Post"),
            NodeKind::Procedure,
        ));

        let report = graph.report();

        assert_eq!(report.nodes().total(), 3);
        assert_eq!(report.nodes().by_kind()[&NodeKind::Module], 1);
        assert_eq!(
            report.nodes().by_kind()[&NodeKind::Metadata(MetadataKind::Document)],
            1
        );
        assert_eq!(report.nodes().by_kind()[&NodeKind::Procedure], 1);
        assert_eq!(report.nodes().with_provenance(), 1);
        assert_eq!(report.nodes().without_provenance(), 2);
    }

    #[test]
    fn edges_are_counted_by_kind_provenance_and_integrity() {
        let module_id = id("module.sales");
        let procedure_id = id("procedure.post");
        let query_id = id("query.balance");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            module_id.clone(),
            name("Module"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            procedure_id.clone(),
            name("Post"),
            NodeKind::Procedure,
        ));
        graph.insert_node(GraphNode::new(
            query_id.clone(),
            name("Balance"),
            NodeKind::Query,
        ));
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                module_id.clone(),
                procedure_id,
                EdgeKind::Contains,
                vec![provenance("edge.contains")],
            ))
            .expect("edge must be valid");
        graph
            .insert_edge(GraphEdge::new(module_id, query_id, EdgeKind::Reads))
            .expect("edge must be valid");

        let report = SemanticGraphReport::from_graph(&graph);

        assert_eq!(report.edges().total(), 2);
        assert_eq!(report.edges().by_kind()[&EdgeKind::Contains], 1);
        assert_eq!(report.edges().by_kind()[&EdgeKind::Reads], 1);
        assert_eq!(report.edges().with_provenance(), 1);
        assert_eq!(report.edges().without_provenance(), 1);
        assert_eq!(report.edges().missing_sources(), 0);
        assert_eq!(report.edges().missing_targets(), 0);
    }

    #[test]
    fn diagnostics_are_counted_by_code_severity_kind_and_provenance() {
        let graph = SemanticGraph::new();
        let diagnostics = vec![
            diagnostic(
                SemanticDiagnosticCode::ReferenceUnresolved,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::UnresolvedTarget,
                Some("diagnostic.a"),
            ),
            diagnostic(
                SemanticDiagnosticCode::ReferenceAmbiguous,
                SemanticDiagnosticSeverity::Warning,
                SemanticDiagnosticKind::AmbiguousTarget,
                None,
            ),
            diagnostic(
                SemanticDiagnosticCode::ReferenceUnresolved,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::UnresolvedTarget,
                None,
            ),
        ];

        let report = SemanticGraphReport::from_graph_and_diagnostics(&graph, &diagnostics);

        assert_eq!(report.diagnostics().total(), 3);
        assert_eq!(report.diagnostics().recoverable(), 3);
        assert_eq!(
            report.diagnostics().by_code()[&SemanticDiagnosticCode::ReferenceUnresolved],
            2
        );
        assert_eq!(
            report.diagnostics().by_severity()[&SemanticDiagnosticSeverity::Error],
            2
        );
        assert_eq!(
            report.diagnostics().by_kind()[&SemanticDiagnosticKind::UnresolvedTarget],
            2
        );
        assert_eq!(report.diagnostics().with_provenance(), 1);
        assert_eq!(report.diagnostics().without_provenance(), 2);
    }

    #[test]
    fn reference_statistics_track_all_outcomes_and_rate() {
        let mut statistics = SemanticReferenceStatistics::new();
        statistics.record(SemanticReferenceOutcome::Resolved, true);
        statistics.record(SemanticReferenceOutcome::Unresolved, true);
        statistics.record(SemanticReferenceOutcome::Ambiguous, false);
        statistics.record(SemanticReferenceOutcome::IncompatibleTargetKind, true);
        statistics.record(SemanticReferenceOutcome::InvalidOwnerReference, false);
        statistics.record(SemanticReferenceOutcome::DuplicateEdgeRequest, true);

        assert_eq!(statistics.total(), 6);
        assert_eq!(statistics.resolved(), 1);
        assert_eq!(statistics.unresolved(), 1);
        assert_eq!(statistics.ambiguous(), 1);
        assert_eq!(statistics.incompatible_target_kind(), 1);
        assert_eq!(statistics.invalid_owner_reference(), 1);
        assert_eq!(statistics.duplicate_edge_request(), 1);
        assert_eq!(statistics.outcome_total(), statistics.total());
        assert_eq!(statistics.with_provenance(), 4);
        assert_eq!(statistics.without_provenance(), 2);
        assert_eq!(statistics.resolution_rate().numerator(), 1);
        assert_eq!(statistics.resolution_rate().denominator(), 6);
        assert!(statistics.resolution_rate().is_defined());
    }

    #[test]
    fn resolution_errors_map_to_reference_outcomes() {
        assert_eq!(
            SemanticReferenceOutcome::from_resolution_error(&ResolutionError::MissingTarget {
                reference: SemanticReference::Name(name("Missing"))
            }),
            SemanticReferenceOutcome::Unresolved
        );
        assert_eq!(
            SemanticReferenceOutcome::from_resolution_error(&ResolutionError::AmbiguousTarget {
                reference: SemanticReference::Name(name("Ambiguous")),
                candidates: vec![id("target.a")]
            }),
            SemanticReferenceOutcome::Ambiguous
        );
        assert_eq!(
            SemanticReferenceOutcome::from_resolution_error(
                &ResolutionError::IncompatibleNodeKind {
                    id: id("target.catalog"),
                    expected: vec![NodeKind::Metadata(MetadataKind::Document)],
                    actual: NodeKind::Metadata(MetadataKind::Catalog)
                }
            ),
            SemanticReferenceOutcome::IncompatibleTargetKind
        );
        assert_eq!(
            SemanticReferenceOutcome::from_resolution_error(
                &ResolutionError::InvalidOwnerReference {
                    owner: id("owner"),
                    child: id("child")
                }
            ),
            SemanticReferenceOutcome::InvalidOwnerReference
        );
    }

    #[test]
    fn report_is_deterministic_for_input_order_changes() {
        let graph = SemanticGraph::new();
        let first = diagnostic(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            Some("diagnostic.a"),
        );
        let second = diagnostic(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            Some("diagnostic.b"),
        );
        let mut left_statistics = SemanticReferenceStatistics::new();
        left_statistics.record(SemanticReferenceOutcome::Resolved, true);
        left_statistics.record(SemanticReferenceOutcome::Unresolved, true);
        let mut right_statistics = SemanticReferenceStatistics::new();
        right_statistics.record(SemanticReferenceOutcome::Unresolved, true);
        right_statistics.record(SemanticReferenceOutcome::Resolved, true);

        let left = SemanticGraphReport::from_graph_diagnostics_and_references(
            &graph,
            &[first.clone(), second.clone()],
            left_statistics,
        );
        let right = SemanticGraphReport::from_graph_diagnostics_and_references(
            &graph,
            &[second, first],
            right_statistics,
        );

        assert_eq!(left, right);
    }

    #[test]
    fn duplicate_edge_request_does_not_change_actual_edge_count() {
        let source_id = id("source");
        let target_id = id("target");
        let mut graph = SemanticGraph::new();
        let mut statistics = SemanticReferenceStatistics::new();

        graph.insert_node(GraphNode::new(
            source_id.clone(),
            name("Source"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            target_id.clone(),
            name("Target"),
            NodeKind::Function,
        ));
        graph
            .insert_edge(GraphEdge::new(
                source_id.clone(),
                target_id.clone(),
                EdgeKind::Calls,
            ))
            .expect("edge must be valid");
        graph
            .insert_edge(GraphEdge::new(source_id, target_id, EdgeKind::Calls))
            .expect("duplicate edge request must be accepted as no-op");
        statistics.record(SemanticReferenceOutcome::Resolved, true);
        statistics.record(SemanticReferenceOutcome::DuplicateEdgeRequest, true);

        let report =
            SemanticGraphReport::from_graph_diagnostics_and_references(&graph, &[], statistics);

        assert_eq!(report.edges().total(), 1);
        assert_eq!(report.resolution().duplicate_edge_request(), 1);
    }
}
