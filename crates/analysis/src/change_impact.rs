//! Bounded product change-impact reports over adjacent semantic publications.
//!
//! Graph remains authoritative for directional diff and impact traversal. This
//! module only matches Configuration identities, invokes the canonical Graph
//! operations, validates product bounds, and reconciles an immutable report.

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use oneagent_common::EntityId;
use oneagent_graph::{
    ImpactCompleteness, ImpactNodeAvailability, ImpactNodeStatus, SemanticGraph,
    SemanticImpactAnalyzer, SemanticImpactOptions, SemanticImpactResult,
};

/// Maximum number of unique Configurations admitted in either endpoint.
pub const MAX_CHANGE_IMPACT_CONFIGURATIONS: usize = 4_096;
/// Maximum UTF-8 bytes in a Configuration, node, or edge identifier.
pub const MAX_CHANGE_IMPACT_IDENTIFIER_BYTES: usize = 4_096;
/// Maximum affected nodes retained by one complete product report.
pub const MAX_CHANGE_IMPACT_AFFECTED_NODES: usize = 65_536;
/// Maximum reasons retained for one affected node.
pub const MAX_CHANGE_IMPACT_REASONS_PER_NODE: usize = 256;
/// Maximum reasons retained by one complete product report.
pub const MAX_CHANGE_IMPACT_REASONS: usize = 262_144;
/// Fixed maximum traversal depth of the complete product report.
pub const CHANGE_IMPACT_MAX_DEPTH: usize = 4;

/// Process-local identity of one successful Workspace publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangeImpactPublicationId(u64);

impl ChangeImpactPublicationId {
    /// Returns the first publication identity of a fresh service run.
    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Creates a non-zero process-local publication identity.
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    /// Returns the numeric process-local identity.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Result<Self, ChangeImpactError> {
        self.0
            .checked_add(1)
            .and_then(Self::new)
            .ok_or_else(ChangeImpactError::summary_overflow)
    }
}

/// Borrowed canonical Configuration evidence for one complete endpoint.
#[derive(Debug, Clone, Copy)]
pub struct ChangeImpactConfiguration<'evidence> {
    configuration_id: &'evidence EntityId,
    graph: &'evidence SemanticGraph,
}

impl<'evidence> ChangeImpactConfiguration<'evidence> {
    /// Creates one borrowed Configuration input.
    #[must_use]
    pub const fn new(
        configuration_id: &'evidence EntityId,
        graph: &'evidence SemanticGraph,
    ) -> Self {
        Self {
            configuration_id,
            graph,
        }
    }

    /// Returns the canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(self) -> &'evidence EntityId {
        self.configuration_id
    }

    /// Returns the complete immutable semantic graph.
    #[must_use]
    pub const fn graph(self) -> &'evidence SemanticGraph {
        self.graph
    }
}

/// Minimal cooperative cancellation observation boundary.
pub trait ChangeImpactCancellationSignal: Send + Sync {
    /// Returns whether report construction cancellation was requested.
    fn is_cancelled(&self) -> bool;
}

/// Cancellation signal that never requests cancellation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverCancelledChangeImpact;

impl ChangeImpactCancellationSignal for NeverCancelledChangeImpact {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Configuration membership across adjacent publications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfigurationImpactKind {
    /// The canonical Configuration identity exists in both publications.
    Compared,
    /// The identity exists only in the current publication.
    Added,
    /// The identity exists only in the previous publication.
    Removed,
}

impl ConfigurationImpactKind {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compared => "compared",
            Self::Added => "added",
            Self::Removed => "removed",
        }
    }
}

/// Product-level completeness of one admitted report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChangeImpactCompleteness {
    /// Every admitted Configuration and Graph result is present through depth four.
    CompleteWithinConfiguredDepth,
}

impl ChangeImpactCompleteness {
    /// Returns the stable public string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CompleteWithinConfiguredDepth => "complete_within_configured_depth",
        }
    }
}

/// Checked counters for one Configuration transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigurationImpactSummary {
    seed_node_changes: usize,
    seed_edge_changes: usize,
    directly_changed_nodes: usize,
    transitively_affected_nodes: usize,
    removed_nodes: usize,
    previous_only_nodes: usize,
    current_nodes: usize,
    max_reached_depth: usize,
    total_affected_nodes: usize,
    configured_max_depth: usize,
}

impl ConfigurationImpactSummary {
    fn from_graph(result: &SemanticImpactResult) -> Result<Self, ChangeImpactError> {
        if result.completeness() != ImpactCompleteness::CompleteWithinRequestedDepth {
            return Err(ChangeImpactError::inconsistent_graph_evidence());
        }
        let graph = result.summary();
        let status_total = checked_add3(
            graph.directly_changed_nodes(),
            graph.transitively_affected_nodes(),
            graph.removed_nodes(),
        )?;
        let availability_total = graph
            .previous_only_nodes()
            .checked_add(graph.current_nodes())
            .ok_or_else(ChangeImpactError::summary_overflow)?;
        if status_total != graph.total_affected_nodes()
            || availability_total != graph.total_affected_nodes()
            || graph.total_affected_nodes() != result.affected_nodes().len()
            || graph.requested_max_depth() != CHANGE_IMPACT_MAX_DEPTH
            || graph.max_reached_depth() > CHANGE_IMPACT_MAX_DEPTH
        {
            return Err(ChangeImpactError::inconsistent_graph_evidence());
        }

        Ok(Self {
            seed_node_changes: graph.seed_node_changes(),
            seed_edge_changes: graph.seed_edge_changes(),
            directly_changed_nodes: graph.directly_changed_nodes(),
            transitively_affected_nodes: graph.transitively_affected_nodes(),
            removed_nodes: graph.removed_nodes(),
            previous_only_nodes: graph.previous_only_nodes(),
            current_nodes: graph.current_nodes(),
            max_reached_depth: graph.max_reached_depth(),
            total_affected_nodes: graph.total_affected_nodes(),
            configured_max_depth: graph.requested_max_depth(),
        })
    }

    /// Returns the number of changed node seeds.
    #[must_use]
    pub const fn seed_node_changes(self) -> usize {
        self.seed_node_changes
    }

    /// Returns the number of changed edge seeds.
    #[must_use]
    pub const fn seed_edge_changes(self) -> usize {
        self.seed_edge_changes
    }

    /// Returns directly changed affected nodes.
    #[must_use]
    pub const fn directly_changed_nodes(self) -> usize {
        self.directly_changed_nodes
    }

    /// Returns transitively affected nodes.
    #[must_use]
    pub const fn transitively_affected_nodes(self) -> usize {
        self.transitively_affected_nodes
    }

    /// Returns removed affected nodes.
    #[must_use]
    pub const fn removed_nodes(self) -> usize {
        self.removed_nodes
    }

    /// Returns affected nodes available only in the previous graph.
    #[must_use]
    pub const fn previous_only_nodes(self) -> usize {
        self.previous_only_nodes
    }

    /// Returns affected nodes available in the current graph.
    #[must_use]
    pub const fn current_nodes(self) -> usize {
        self.current_nodes
    }

    /// Returns the maximum reached Graph traversal depth.
    #[must_use]
    pub const fn max_reached_depth(self) -> usize {
        self.max_reached_depth
    }

    /// Returns the total unique affected-node count.
    #[must_use]
    pub const fn total_affected_nodes(self) -> usize {
        self.total_affected_nodes
    }

    /// Returns the fixed complete report depth.
    #[must_use]
    pub const fn configured_max_depth(self) -> usize {
        self.configured_max_depth
    }
}

/// One immutable Configuration transition and canonical Graph result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationImpact {
    configuration_id: EntityId,
    kind: ConfigurationImpactKind,
    result: SemanticImpactResult,
    summary: ConfigurationImpactSummary,
}

impl ConfigurationImpact {
    /// Returns the canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the Configuration transition kind.
    #[must_use]
    pub const fn kind(&self) -> ConfigurationImpactKind {
        self.kind
    }

    /// Returns the complete Graph-owned impact result.
    #[must_use]
    pub const fn result(&self) -> &SemanticImpactResult {
        &self.result
    }

    /// Returns checked counters for this transition.
    #[must_use]
    pub const fn summary(&self) -> ConfigurationImpactSummary {
        self.summary
    }
}

/// Checked aggregate counters for one complete product report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChangeImpactSummary {
    total_configurations: usize,
    compared_configurations: usize,
    added_configurations: usize,
    removed_configurations: usize,
    seed_node_changes: usize,
    seed_edge_changes: usize,
    directly_changed_nodes: usize,
    transitively_affected_nodes: usize,
    removed_nodes: usize,
    previous_only_nodes: usize,
    current_nodes: usize,
    max_reached_depth: usize,
    total_affected_nodes: usize,
    configured_max_depth: usize,
}

impl ChangeImpactSummary {
    fn from_configurations(
        configurations: &[ConfigurationImpact],
    ) -> Result<Self, ChangeImpactError> {
        let mut summary = Self {
            configured_max_depth: CHANGE_IMPACT_MAX_DEPTH,
            ..Self::default()
        };
        for configuration in configurations {
            summary.total_configurations = checked_increment(summary.total_configurations)?;
            match configuration.kind() {
                ConfigurationImpactKind::Compared => {
                    summary.compared_configurations =
                        checked_increment(summary.compared_configurations)?;
                }
                ConfigurationImpactKind::Added => {
                    summary.added_configurations = checked_increment(summary.added_configurations)?;
                }
                ConfigurationImpactKind::Removed => {
                    summary.removed_configurations =
                        checked_increment(summary.removed_configurations)?;
                }
            }
            let value = configuration.summary();
            summary.seed_node_changes =
                checked_add(summary.seed_node_changes, value.seed_node_changes())?;
            summary.seed_edge_changes =
                checked_add(summary.seed_edge_changes, value.seed_edge_changes())?;
            summary.directly_changed_nodes = checked_add(
                summary.directly_changed_nodes,
                value.directly_changed_nodes(),
            )?;
            summary.transitively_affected_nodes = checked_add(
                summary.transitively_affected_nodes,
                value.transitively_affected_nodes(),
            )?;
            summary.removed_nodes = checked_add(summary.removed_nodes, value.removed_nodes())?;
            summary.previous_only_nodes =
                checked_add(summary.previous_only_nodes, value.previous_only_nodes())?;
            summary.current_nodes = checked_add(summary.current_nodes, value.current_nodes())?;
            summary.total_affected_nodes =
                checked_add(summary.total_affected_nodes, value.total_affected_nodes())?;
            summary.max_reached_depth = summary.max_reached_depth.max(value.max_reached_depth());
        }

        let transition_total = checked_add3(
            summary.compared_configurations,
            summary.added_configurations,
            summary.removed_configurations,
        )?;
        let status_total = checked_add3(
            summary.directly_changed_nodes,
            summary.transitively_affected_nodes,
            summary.removed_nodes,
        )?;
        let availability_total = checked_add(summary.previous_only_nodes, summary.current_nodes)?;
        if transition_total != summary.total_configurations
            || status_total != summary.total_affected_nodes
            || availability_total != summary.total_affected_nodes
        {
            return Err(ChangeImpactError::inconsistent_graph_evidence());
        }
        Ok(summary)
    }

    /// Returns the number of Configuration transitions.
    #[must_use]
    pub const fn total_configurations(self) -> usize {
        self.total_configurations
    }

    /// Returns matched Configuration transitions.
    #[must_use]
    pub const fn compared_configurations(self) -> usize {
        self.compared_configurations
    }

    /// Returns current-only Configuration transitions.
    #[must_use]
    pub const fn added_configurations(self) -> usize {
        self.added_configurations
    }

    /// Returns previous-only Configuration transitions.
    #[must_use]
    pub const fn removed_configurations(self) -> usize {
        self.removed_configurations
    }

    /// Returns aggregate node seed changes.
    #[must_use]
    pub const fn seed_node_changes(self) -> usize {
        self.seed_node_changes
    }

    /// Returns aggregate edge seed changes.
    #[must_use]
    pub const fn seed_edge_changes(self) -> usize {
        self.seed_edge_changes
    }

    /// Returns aggregate directly changed nodes.
    #[must_use]
    pub const fn directly_changed_nodes(self) -> usize {
        self.directly_changed_nodes
    }

    /// Returns aggregate transitively affected nodes.
    #[must_use]
    pub const fn transitively_affected_nodes(self) -> usize {
        self.transitively_affected_nodes
    }

    /// Returns aggregate removed nodes.
    #[must_use]
    pub const fn removed_nodes(self) -> usize {
        self.removed_nodes
    }

    /// Returns aggregate previous-only nodes.
    #[must_use]
    pub const fn previous_only_nodes(self) -> usize {
        self.previous_only_nodes
    }

    /// Returns aggregate nodes available in current graphs.
    #[must_use]
    pub const fn current_nodes(self) -> usize {
        self.current_nodes
    }

    /// Returns maximum reached depth across all Configuration results.
    #[must_use]
    pub const fn max_reached_depth(self) -> usize {
        self.max_reached_depth
    }

    /// Returns the aggregate affected-node count.
    #[must_use]
    pub const fn total_affected_nodes(self) -> usize {
        self.total_affected_nodes
    }

    /// Returns the fixed complete report depth.
    #[must_use]
    pub const fn configured_max_depth(self) -> usize {
        self.configured_max_depth
    }
}

/// Complete immutable product report for two adjacent publications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeImpactReport {
    previous_publication_id: ChangeImpactPublicationId,
    current_publication_id: ChangeImpactPublicationId,
    configurations: Vec<ConfigurationImpact>,
    summary: ChangeImpactSummary,
    completeness: ChangeImpactCompleteness,
}

impl ChangeImpactReport {
    /// Returns the predecessor publication identity.
    #[must_use]
    pub const fn previous_publication_id(&self) -> ChangeImpactPublicationId {
        self.previous_publication_id
    }

    /// Returns the current publication identity.
    #[must_use]
    pub const fn current_publication_id(&self) -> ChangeImpactPublicationId {
        self.current_publication_id
    }

    /// Returns Configuration transitions in canonical identity order.
    #[must_use]
    pub fn configurations(&self) -> &[ConfigurationImpact] {
        &self.configurations
    }

    /// Finds a Configuration transition by canonical identity.
    #[must_use]
    pub fn configuration(&self, id: &EntityId) -> Option<&ConfigurationImpact> {
        self.configurations
            .binary_search_by(|candidate| candidate.configuration_id().cmp(id))
            .ok()
            .map(|index| &self.configurations[index])
    }

    /// Returns checked aggregate counters.
    #[must_use]
    pub const fn summary(&self) -> ChangeImpactSummary {
        self.summary
    }

    /// Returns complete product-report status.
    #[must_use]
    pub const fn completeness(&self) -> ChangeImpactCompleteness {
        self.completeness
    }

    /// Returns `true` when no Configuration transition exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.configurations.is_empty()
    }
}

/// Stable closed failure classification for product report construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChangeImpactErrorKind {
    /// One endpoint contains too many unique Configurations.
    TooManyConfigurations,
    /// A retained Configuration, node, or edge identifier exceeds its bound.
    IdentifierTooLarge,
    /// The complete report contains too many affected nodes.
    TooManyAffectedNodes,
    /// One affected node contains too many reasons.
    TooManyReasonsForNode,
    /// The complete report contains too many reasons.
    TooManyReasons,
    /// Equal Configuration identities carry different graph content in one endpoint.
    ConflictingConfiguration,
    /// Canonical Graph output failed a product reconciliation invariant.
    InconsistentGraphEvidence,
    /// Checked publication or summary arithmetic overflowed.
    SummaryOverflow,
    /// Cooperative cancellation was observed.
    Cancelled,
}

/// Bounded redacted Change Impact domain error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeImpactError {
    kind: ChangeImpactErrorKind,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl ChangeImpactError {
    const fn bounded(kind: ChangeImpactErrorKind, actual: usize, maximum: usize) -> Self {
        Self {
            kind,
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    const fn closed(kind: ChangeImpactErrorKind) -> Self {
        Self {
            kind,
            actual: None,
            maximum: None,
        }
    }

    const fn conflicting_configuration() -> Self {
        Self::closed(ChangeImpactErrorKind::ConflictingConfiguration)
    }

    const fn inconsistent_graph_evidence() -> Self {
        Self::closed(ChangeImpactErrorKind::InconsistentGraphEvidence)
    }

    const fn summary_overflow() -> Self {
        Self::closed(ChangeImpactErrorKind::SummaryOverflow)
    }

    const fn cancelled() -> Self {
        Self::closed(ChangeImpactErrorKind::Cancelled)
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> ChangeImpactErrorKind {
        self.kind
    }

    /// Returns the rejected count for a bounded failure.
    #[must_use]
    pub const fn actual(self) -> Option<usize> {
        self.actual
    }

    /// Returns the accepted maximum for a bounded failure.
    #[must_use]
    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

impl Display for ChangeImpactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.actual, self.maximum) {
            (Some(actual), Some(maximum)) => write!(
                formatter,
                "change impact rejected a bounded count: kind={:?}, actual={actual}, maximum={maximum}",
                self.kind
            ),
            _ => write!(
                formatter,
                "change impact rejected evidence: kind={:?}",
                self.kind
            ),
        }
    }
}

impl std::error::Error for ChangeImpactError {}

/// Stateless evaluator for one adjacent complete semantic publication pair.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChangeImpactEvaluator;

impl ChangeImpactEvaluator {
    /// Builds one complete bounded immutable product report.
    ///
    /// The current publication identity is the checked successor of
    /// `previous_publication_id`. Exact duplicate Configuration inputs collapse;
    /// conflicting content for one identity fails closed.
    ///
    /// # Errors
    ///
    /// Returns a closed [`ChangeImpactError`] without a partial report when an
    /// input, Graph result, bound, reconciliation, or cancellation invariant
    /// fails.
    pub fn evaluate(
        &self,
        previous_publication_id: ChangeImpactPublicationId,
        previous: &[ChangeImpactConfiguration<'_>],
        current: &[ChangeImpactConfiguration<'_>],
        cancellation: &dyn ChangeImpactCancellationSignal,
    ) -> Result<ChangeImpactReport, ChangeImpactError> {
        observe_cancellation(cancellation)?;
        let current_publication_id = previous_publication_id.next()?;
        let previous = normalize_endpoint(previous, cancellation)?;
        let current = normalize_endpoint(current, cancellation)?;
        let empty = SemanticGraph::new();
        let mut configurations = Vec::with_capacity(previous.len().max(current.len()));
        let mut affected_nodes = 0usize;
        let mut reasons = 0usize;

        let mut identities = previous
            .keys()
            .chain(current.keys())
            .copied()
            .collect::<Vec<_>>();
        identities.sort();
        identities.dedup();

        for configuration_id in identities {
            observe_cancellation(cancellation)?;
            let (kind, previous_graph, current_graph) = match (
                previous.get(configuration_id),
                current.get(configuration_id),
            ) {
                (Some(previous), Some(current)) => {
                    (ConfigurationImpactKind::Compared, *previous, *current)
                }
                (Some(previous), None) => (ConfigurationImpactKind::Removed, *previous, &empty),
                (None, Some(current)) => (ConfigurationImpactKind::Added, &empty, *current),
                (None, None) => return Err(ChangeImpactError::inconsistent_graph_evidence()),
            };

            validate_graph_identifiers(previous_graph)?;
            validate_graph_identifiers(current_graph)?;
            let diff = previous_graph.diff(current_graph);
            let result = SemanticImpactAnalyzer::analyze(
                previous_graph,
                current_graph,
                &diff,
                &SemanticImpactOptions::new(CHANGE_IMPACT_MAX_DEPTH),
            )
            .map_err(|_| ChangeImpactError::inconsistent_graph_evidence())?;
            observe_cancellation(cancellation)?;
            validate_result(&result, &mut affected_nodes, &mut reasons)?;
            let summary = ConfigurationImpactSummary::from_graph(&result)?;
            configurations.push(ConfigurationImpact {
                configuration_id: configuration_id.clone(),
                kind,
                result,
                summary,
            });
        }

        observe_cancellation(cancellation)?;
        let summary = ChangeImpactSummary::from_configurations(&configurations)?;
        Ok(ChangeImpactReport {
            previous_publication_id,
            current_publication_id,
            configurations,
            summary,
            completeness: ChangeImpactCompleteness::CompleteWithinConfiguredDepth,
        })
    }
}

fn normalize_endpoint<'evidence>(
    configurations: &[ChangeImpactConfiguration<'evidence>],
    cancellation: &dyn ChangeImpactCancellationSignal,
) -> Result<BTreeMap<&'evidence EntityId, &'evidence SemanticGraph>, ChangeImpactError> {
    let mut normalized: BTreeMap<&EntityId, &SemanticGraph> = BTreeMap::new();
    for configuration in configurations {
        observe_cancellation(cancellation)?;
        validate_identifier(configuration.configuration_id().as_str())?;
        validate_graph_identifiers(configuration.graph())?;
        if let Some(existing) = normalized.get(configuration.configuration_id()) {
            if !existing.diff(configuration.graph()).is_empty() {
                return Err(ChangeImpactError::conflicting_configuration());
            }
        } else {
            if normalized.len() == MAX_CHANGE_IMPACT_CONFIGURATIONS {
                return Err(ChangeImpactError::bounded(
                    ChangeImpactErrorKind::TooManyConfigurations,
                    MAX_CHANGE_IMPACT_CONFIGURATIONS + 1,
                    MAX_CHANGE_IMPACT_CONFIGURATIONS,
                ));
            }
            normalized.insert(configuration.configuration_id(), configuration.graph());
        }
    }
    Ok(normalized)
}

fn validate_graph_identifiers(graph: &SemanticGraph) -> Result<(), ChangeImpactError> {
    for node in graph.nodes() {
        validate_identifier(node.id().as_str())?;
    }
    for edge in graph.edges() {
        validate_identifier(edge.source().as_str())?;
        validate_identifier(edge.target().as_str())?;
    }
    Ok(())
}

fn validate_result(
    result: &SemanticImpactResult,
    affected_nodes: &mut usize,
    reasons: &mut usize,
) -> Result<(), ChangeImpactError> {
    let next_nodes = checked_add(*affected_nodes, result.affected_nodes().len())?;
    validate_count(
        ChangeImpactErrorKind::TooManyAffectedNodes,
        next_nodes,
        MAX_CHANGE_IMPACT_AFFECTED_NODES,
    )?;

    let mut next_reasons = *reasons;
    for node in result.affected_nodes() {
        validate_identifier(node.node_id().as_str())?;
        validate_count(
            ChangeImpactErrorKind::TooManyReasonsForNode,
            node.reasons().len(),
            MAX_CHANGE_IMPACT_REASONS_PER_NODE,
        )?;
        next_reasons = checked_add(next_reasons, node.reasons().len())?;
        validate_count(
            ChangeImpactErrorKind::TooManyReasons,
            next_reasons,
            MAX_CHANGE_IMPACT_REASONS,
        )?;

        for reason in node.reasons() {
            if let Some(node_id) = reason.seed().node_id() {
                validate_identifier(node_id.as_str())?;
            }
            if let Some(edge_id) = reason.seed().edge_id() {
                validate_identifier(edge_id.as_str())?;
            }
            if let Some(source_node) = reason.source_node() {
                validate_identifier(source_node.as_str())?;
            }
            if let Some(edge_id) = reason.edge_id() {
                validate_identifier(edge_id.as_str())?;
            }
            if reason.depth() > CHANGE_IMPACT_MAX_DEPTH {
                return Err(ChangeImpactError::inconsistent_graph_evidence());
            }
        }

        if node.depth() > CHANGE_IMPACT_MAX_DEPTH
            || (node.status() == ImpactNodeStatus::Removed
                && node.availability() != ImpactNodeAvailability::PreviousOnly)
        {
            return Err(ChangeImpactError::inconsistent_graph_evidence());
        }
    }
    *affected_nodes = next_nodes;
    *reasons = next_reasons;
    Ok(())
}

fn validate_identifier(value: &str) -> Result<(), ChangeImpactError> {
    validate_count(
        ChangeImpactErrorKind::IdentifierTooLarge,
        value.len(),
        MAX_CHANGE_IMPACT_IDENTIFIER_BYTES,
    )
}

fn validate_count(
    kind: ChangeImpactErrorKind,
    actual: usize,
    maximum: usize,
) -> Result<(), ChangeImpactError> {
    if actual > maximum {
        Err(ChangeImpactError::bounded(kind, actual, maximum))
    } else {
        Ok(())
    }
}

fn observe_cancellation(
    cancellation: &dyn ChangeImpactCancellationSignal,
) -> Result<(), ChangeImpactError> {
    if cancellation.is_cancelled() {
        Err(ChangeImpactError::cancelled())
    } else {
        Ok(())
    }
}

fn checked_increment(value: usize) -> Result<usize, ChangeImpactError> {
    value
        .checked_add(1)
        .ok_or_else(ChangeImpactError::summary_overflow)
}

fn checked_add(left: usize, right: usize) -> Result<usize, ChangeImpactError> {
    left.checked_add(right)
        .ok_or_else(ChangeImpactError::summary_overflow)
}

fn checked_add3(first: usize, second: usize, third: usize) -> Result<usize, ChangeImpactError> {
    checked_add(checked_add(first, second)?, third)
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeImpactErrorKind, MAX_CHANGE_IMPACT_AFFECTED_NODES, MAX_CHANGE_IMPACT_CONFIGURATIONS,
        MAX_CHANGE_IMPACT_IDENTIFIER_BYTES, MAX_CHANGE_IMPACT_REASONS,
        MAX_CHANGE_IMPACT_REASONS_PER_NODE, checked_add, checked_increment, validate_count,
    };

    #[test]
    fn every_count_bound_accepts_exact_and_rejects_one_over() {
        for (kind, maximum) in [
            (
                ChangeImpactErrorKind::TooManyConfigurations,
                MAX_CHANGE_IMPACT_CONFIGURATIONS,
            ),
            (
                ChangeImpactErrorKind::IdentifierTooLarge,
                MAX_CHANGE_IMPACT_IDENTIFIER_BYTES,
            ),
            (
                ChangeImpactErrorKind::TooManyAffectedNodes,
                MAX_CHANGE_IMPACT_AFFECTED_NODES,
            ),
            (
                ChangeImpactErrorKind::TooManyReasonsForNode,
                MAX_CHANGE_IMPACT_REASONS_PER_NODE,
            ),
            (
                ChangeImpactErrorKind::TooManyReasons,
                MAX_CHANGE_IMPACT_REASONS,
            ),
        ] {
            assert!(validate_count(kind, maximum, maximum).is_ok());
            let error =
                validate_count(kind, maximum + 1, maximum).expect_err("one-over count must fail");
            assert_eq!(error.kind(), kind);
            assert_eq!(error.actual(), Some(maximum + 1));
            assert_eq!(error.maximum(), Some(maximum));
        }
    }

    #[test]
    fn checked_summary_arithmetic_rejects_overflow() {
        assert_eq!(checked_increment(0), Ok(1));
        assert_eq!(checked_add(1, 2), Ok(3));
        assert_eq!(
            checked_increment(usize::MAX)
                .expect_err("increment must overflow")
                .kind(),
            ChangeImpactErrorKind::SummaryOverflow
        );
        assert_eq!(
            checked_add(usize::MAX, 1)
                .expect_err("addition must overflow")
                .kind(),
            ChangeImpactErrorKind::SummaryOverflow
        );
    }
}
