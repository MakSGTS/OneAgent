//! Deterministic semantic Context Engine domain and request resolution.

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{EdgeId, EdgeKind, NodeId, NodeKind, Provenance, SemanticGraph};

/// Minimum accepted rendered-context budget in UTF-8 bytes.
pub const MIN_CONTEXT_BUDGET_BYTES: usize = 1;
/// Maximum accepted rendered-context budget in UTF-8 bytes.
pub const MAX_CONTEXT_BUDGET_BYTES: usize = 65_536;
/// Maximum number of raw request seeds.
pub const MAX_CONTEXT_SEEDS: usize = 16;
/// Maximum accepted traversal depth.
pub const MAX_CONTEXT_DEPTH: usize = 4;
/// Maximum accepted candidate count.
pub const MAX_CONTEXT_CANDIDATES: usize = 128;

/// Closed first-slice Context Engine intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextIntent {
    /// Explain bounded semantic graph evidence around the request seeds.
    Explain,
}

/// Seed used to select canonical graph nodes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextSeed {
    /// Select one exact node identifier with an optional kind constraint.
    Node {
        /// Stable node identifier.
        id: NodeId,
        /// Required kind when present.
        expected_kind: Option<NodeKind>,
    },
    /// Select one exact canonical name with an optional kind constraint.
    ExactName {
        /// Exact case-sensitive canonical name.
        name: EntityName,
        /// Required kind when present.
        expected_kind: Option<NodeKind>,
    },
}

impl ContextSeed {
    /// Creates an unconstrained exact-node seed.
    #[must_use]
    pub fn node(id: impl Into<NodeId>) -> Self {
        Self::Node {
            id: id.into(),
            expected_kind: None,
        }
    }

    /// Creates an exact-node seed constrained to `expected_kind`.
    #[must_use]
    pub fn node_with_kind(id: impl Into<NodeId>, expected_kind: NodeKind) -> Self {
        Self::Node {
            id: id.into(),
            expected_kind: Some(expected_kind),
        }
    }

    /// Creates an unconstrained exact-name seed.
    #[must_use]
    pub const fn exact_name(name: EntityName) -> Self {
        Self::ExactName {
            name,
            expected_kind: None,
        }
    }

    /// Creates an exact-name seed constrained to `expected_kind`.
    #[must_use]
    pub const fn exact_name_with_kind(name: EntityName, expected_kind: NodeKind) -> Self {
        Self::ExactName {
            name,
            expected_kind: Some(expected_kind),
        }
    }

    /// Returns the optional kind constraint.
    #[must_use]
    pub const fn expected_kind(&self) -> Option<NodeKind> {
        match self {
            Self::Node { expected_kind, .. } | Self::ExactName { expected_kind, .. } => {
                *expected_kind
            }
        }
    }
}

/// Direction used by Context Engine traversal policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextTraversalDirection {
    /// Follow graph edges from source to target.
    Outgoing,
    /// Follow graph edges from target to source.
    Incoming,
    /// Consider outgoing edges before incoming edges.
    Both,
}

/// Direction of one selected path step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextRelationDirection {
    /// The path follows the stored edge direction.
    Outgoing,
    /// The path follows the inverse query direction.
    Incoming,
}

/// Validated Context Engine policy input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPolicy {
    direction: ContextTraversalDirection,
    edge_kinds: BTreeSet<EdgeKind>,
    node_kinds: Option<BTreeSet<NodeKind>>,
    max_depth: usize,
    max_candidates: usize,
}

impl ContextPolicy {
    /// Creates policy input validated when a [`ContextRequest`] is constructed.
    #[must_use]
    pub const fn new(
        direction: ContextTraversalDirection,
        edge_kinds: BTreeSet<EdgeKind>,
        node_kinds: Option<BTreeSet<NodeKind>>,
        max_depth: usize,
        max_candidates: usize,
    ) -> Self {
        Self {
            direction,
            edge_kinds,
            node_kinds,
            max_depth,
            max_candidates,
        }
    }

    /// Returns the traversal direction.
    #[must_use]
    pub const fn direction(&self) -> ContextTraversalDirection {
        self.direction
    }

    /// Returns the accepted edge kinds.
    #[must_use]
    pub const fn edge_kinds(&self) -> &BTreeSet<EdgeKind> {
        &self.edge_kinds
    }

    /// Returns the optional accepted node-kind set.
    #[must_use]
    pub const fn node_kinds(&self) -> Option<&BTreeSet<NodeKind>> {
        self.node_kinds.as_ref()
    }

    /// Returns the strict traversal depth bound.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the total candidate bound, including seeds.
    #[must_use]
    pub const fn max_candidates(&self) -> usize {
        self.max_candidates
    }

    fn validate(&self) -> Result<(), ContextError> {
        if self.max_depth > MAX_CONTEXT_DEPTH {
            return Err(ContextError::InvalidPolicy {
                field: ContextPolicyField::MaxDepth,
                value: self.max_depth,
                minimum: 0,
                maximum: MAX_CONTEXT_DEPTH,
            });
        }
        if !(1..=MAX_CONTEXT_CANDIDATES).contains(&self.max_candidates) {
            return Err(ContextError::InvalidPolicy {
                field: ContextPolicyField::MaxCandidates,
                value: self.max_candidates,
                minimum: 1,
                maximum: MAX_CONTEXT_CANDIDATES,
            });
        }
        if self.edge_kinds.is_empty() {
            return Err(ContextError::EmptyEdgeKinds);
        }
        if self.node_kinds.as_ref().is_some_and(BTreeSet::is_empty) {
            return Err(ContextError::EmptyNodeKinds);
        }

        Ok(())
    }

    fn allows_node(&self, kind: NodeKind) -> bool {
        self.node_kinds
            .as_ref()
            .is_none_or(|kinds| kinds.contains(&kind))
    }
}

impl Default for ContextPolicy {
    fn default() -> Self {
        Self::new(
            ContextTraversalDirection::Both,
            BTreeSet::from([
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
            ]),
            None,
            2,
            32,
        )
    }
}

/// Validated rendered-context budget in UTF-8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextBudget(usize);

impl ContextBudget {
    fn new(bytes: usize) -> Result<Self, ContextError> {
        if !(MIN_CONTEXT_BUDGET_BYTES..=MAX_CONTEXT_BUDGET_BYTES).contains(&bytes) {
            return Err(ContextError::InvalidBudget {
                value: bytes,
                minimum: MIN_CONTEXT_BUDGET_BYTES,
                maximum: MAX_CONTEXT_BUDGET_BYTES,
            });
        }

        Ok(Self(bytes))
    }

    /// Returns the accepted UTF-8 byte budget.
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.0
    }
}

/// Validated Context Engine request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRequest {
    intent: ContextIntent,
    seeds: Vec<ContextSeed>,
    budget: ContextBudget,
    policy: ContextPolicy,
}

impl ContextRequest {
    /// Validates and creates a Context Engine request.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid budget, policy, seed count, or node ID.
    pub fn new(
        intent: ContextIntent,
        seeds: Vec<ContextSeed>,
        budget_bytes: usize,
        policy: ContextPolicy,
    ) -> Result<Self, ContextError> {
        let budget = ContextBudget::new(budget_bytes)?;
        policy.validate()?;

        if !(1..=MAX_CONTEXT_SEEDS).contains(&seeds.len()) {
            return Err(ContextError::InvalidSeedCount {
                value: seeds.len(),
                minimum: 1,
                maximum: MAX_CONTEXT_SEEDS,
            });
        }

        for seed in &seeds {
            if let ContextSeed::Node { id, .. } = seed {
                EntityId::new(id.as_str()).map_err(|_| ContextError::InvalidSeedIdentifier {
                    value: id.as_str().to_owned(),
                })?;
            }
        }

        Ok(Self {
            intent,
            seeds,
            budget,
            policy,
        })
    }

    /// Returns the request intent.
    #[must_use]
    pub const fn intent(&self) -> ContextIntent {
        self.intent
    }

    /// Returns the raw request seeds in caller order.
    #[must_use]
    pub fn seeds(&self) -> &[ContextSeed] {
        &self.seeds
    }

    /// Returns the validated rendered UTF-8 byte budget.
    #[must_use]
    pub const fn budget(&self) -> ContextBudget {
        self.budget
    }

    /// Returns the validated selection policy.
    #[must_use]
    pub const fn policy(&self) -> &ContextPolicy {
        &self.policy
    }
}

/// Canonical request after all seeds resolve successfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedContextRequest {
    intent: ContextIntent,
    seeds: Vec<NodeId>,
    budget: ContextBudget,
    policy: ContextPolicy,
}

impl ResolvedContextRequest {
    /// Returns the request intent.
    #[must_use]
    pub const fn intent(&self) -> ContextIntent {
        self.intent
    }

    /// Returns unique resolved seeds in stable node-ID order.
    #[must_use]
    pub fn seeds(&self) -> &[NodeId] {
        &self.seeds
    }

    /// Returns the validated rendered UTF-8 byte budget.
    #[must_use]
    pub const fn budget(&self) -> ContextBudget {
        self.budget
    }

    /// Returns the validated selection policy.
    #[must_use]
    pub const fn policy(&self) -> &ContextPolicy {
        &self.policy
    }
}

/// Stable path step retained for Context explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPathStep {
    direction: ContextRelationDirection,
    edge_kind: EdgeKind,
    edge_id: EdgeId,
    provenance: Vec<Provenance>,
}

impl ContextPathStep {
    /// Returns the relation direction.
    #[must_use]
    pub const fn direction(&self) -> ContextRelationDirection {
        self.direction
    }

    /// Returns the graph edge kind.
    #[must_use]
    pub const fn edge_kind(&self) -> EdgeKind {
        self.edge_kind
    }

    /// Returns the stable edge identifier.
    #[must_use]
    pub const fn edge_id(&self) -> &EdgeId {
        &self.edge_id
    }

    /// Returns canonicalized edge provenance.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }
}

/// Typed reason for including a Context item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextInclusionReason {
    /// The item is a resolved request seed.
    Seed,
    /// The item is related to a seed through its retained path.
    Related,
}

/// One admitted semantic Context item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextItem {
    node_id: NodeId,
    name: EntityName,
    kind: NodeKind,
    provenance: Vec<Provenance>,
    depth: usize,
    seed_id: NodeId,
    path: Vec<ContextPathStep>,
    reason: ContextInclusionReason,
    fragment: String,
    cost_bytes: usize,
}

impl ContextItem {
    /// Returns the canonical node identifier.
    #[must_use]
    pub const fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns the exact canonical node name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns canonicalized node provenance.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Returns the selected graph depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the seed that selected this item.
    #[must_use]
    pub const fn seed_id(&self) -> &NodeId {
        &self.seed_id
    }

    /// Returns the selected path from the seed.
    #[must_use]
    pub fn path(&self) -> &[ContextPathStep] {
        &self.path
    }

    /// Returns the typed inclusion reason.
    #[must_use]
    pub const fn reason(&self) -> ContextInclusionReason {
        self.reason
    }

    /// Returns the exact rendered semantic fragment.
    #[must_use]
    pub fn fragment(&self) -> &str {
        &self.fragment
    }

    /// Returns the rendered UTF-8 byte cost.
    #[must_use]
    pub const fn cost_bytes(&self) -> usize {
        self.cost_bytes
    }
}

/// Owned deterministic Context Engine result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundle {
    intent: ContextIntent,
    seeds: Vec<NodeId>,
    items: Vec<ContextItem>,
    budget: ContextBudget,
    used_bytes: usize,
    remaining_bytes: usize,
    candidate_omitted: usize,
    budget_omitted: usize,
    rendered: String,
}

impl ContextBundle {
    /// Returns the request intent.
    #[must_use]
    pub const fn intent(&self) -> ContextIntent {
        self.intent
    }

    /// Returns resolved seed IDs in stable order.
    #[must_use]
    pub fn seeds(&self) -> &[NodeId] {
        &self.seeds
    }

    /// Returns admitted Context items in rendering order.
    #[must_use]
    pub fn items(&self) -> &[ContextItem] {
        &self.items
    }

    /// Returns the requested rendered UTF-8 byte budget.
    #[must_use]
    pub const fn budget(&self) -> ContextBudget {
        self.budget
    }

    /// Returns admitted rendered bytes.
    #[must_use]
    pub const fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    /// Returns unused rendered bytes.
    #[must_use]
    pub const fn remaining_bytes(&self) -> usize {
        self.remaining_bytes
    }

    /// Returns whether candidate-limit truncation occurred.
    #[must_use]
    pub const fn candidate_truncated(&self) -> bool {
        self.candidate_omitted > 0
    }

    /// Returns the exact candidate-limit omission count.
    #[must_use]
    pub const fn candidate_omitted(&self) -> usize {
        self.candidate_omitted
    }

    /// Returns whether budget truncation occurred.
    #[must_use]
    pub const fn budget_truncated(&self) -> bool {
        self.budget_omitted > 0
    }

    /// Returns the exact budget omission count.
    #[must_use]
    pub const fn budget_omitted(&self) -> usize {
        self.budget_omitted
    }

    /// Returns exact concatenated semantic Context rendering.
    #[must_use]
    pub fn rendered(&self) -> &str {
        &self.rendered
    }
}

/// Context policy field with numeric bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextPolicyField {
    /// Strict maximum traversal depth.
    MaxDepth,
    /// Total candidate maximum.
    MaxCandidates,
}

/// Typed Context Engine request or resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    /// Rendered byte budget is outside accepted bounds.
    InvalidBudget {
        /// Rejected value.
        value: usize,
        /// Inclusive minimum.
        minimum: usize,
        /// Inclusive maximum.
        maximum: usize,
    },
    /// Numeric policy value is outside accepted bounds.
    InvalidPolicy {
        /// Rejected field.
        field: ContextPolicyField,
        /// Rejected value.
        value: usize,
        /// Inclusive minimum.
        minimum: usize,
        /// Inclusive maximum.
        maximum: usize,
    },
    /// Policy edge-kind set is empty.
    EmptyEdgeKinds,
    /// Present policy node-kind set is empty.
    EmptyNodeKinds,
    /// Raw seed count is outside accepted bounds.
    InvalidSeedCount {
        /// Rejected value.
        value: usize,
        /// Inclusive minimum.
        minimum: usize,
        /// Inclusive maximum.
        maximum: usize,
    },
    /// Exact node seed has an invalid identifier.
    InvalidSeedIdentifier {
        /// Rejected raw identifier.
        value: String,
    },
    /// No graph node matches the canonical seed.
    MissingSeed {
        /// Missing canonical seed.
        seed: ContextSeed,
    },
    /// More than one graph node matches an exact-name seed.
    AmbiguousSeed {
        /// Ambiguous canonical seed.
        seed: ContextSeed,
        /// Compatible candidates in stable node-ID order.
        candidates: Vec<NodeId>,
    },
    /// A graph node exists but conflicts with seed or policy kinds.
    IncompatibleSeed {
        /// Incompatible canonical seed.
        seed: ContextSeed,
        /// Accepted kinds in stable enum order.
        expected: Vec<NodeKind>,
        /// Observed kinds in stable enum order.
        actual: Vec<NodeKind>,
    },
    /// Unique seed count exceeds the candidate limit.
    TooManyUniqueSeeds {
        /// Unique resolved seed count.
        value: usize,
        /// Policy candidate maximum.
        maximum: usize,
    },
    /// Mandatory seed fragments cannot fit the budget.
    InsufficientBudget {
        /// Required UTF-8 bytes.
        required: usize,
        /// Available UTF-8 bytes.
        available: usize,
    },
    /// Checked rendering or accounting arithmetic overflowed.
    CostOverflow,
    /// A future graph kind reached Context rendering without an accepted map.
    UnsupportedKind,
}

impl Display for ContextError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBudget {
                value,
                minimum,
                maximum,
            } => write!(
                formatter,
                "context budget {value} is outside {minimum}..={maximum} UTF-8 bytes"
            ),
            Self::InvalidPolicy {
                field,
                value,
                minimum,
                maximum,
            } => write!(
                formatter,
                "context policy {field} value {value} is outside {minimum}..={maximum}"
            ),
            Self::EmptyEdgeKinds => {
                formatter.write_str("context policy edge kinds must not be empty")
            }
            Self::EmptyNodeKinds => {
                formatter.write_str("context policy node kinds must not be empty when present")
            }
            Self::InvalidSeedCount {
                value,
                minimum,
                maximum,
            } => write!(
                formatter,
                "context seed count {value} is outside {minimum}..={maximum}"
            ),
            Self::InvalidSeedIdentifier { value } => {
                write!(formatter, "context seed identifier is invalid: {value:?}")
            }
            Self::MissingSeed { seed } => write!(formatter, "context seed is missing: {seed}"),
            Self::AmbiguousSeed { seed, candidates } => write!(
                formatter,
                "context seed is ambiguous: {seed} matched {} nodes",
                candidates.len()
            ),
            Self::IncompatibleSeed {
                seed,
                expected,
                actual,
            } => write!(
                formatter,
                "context seed is incompatible: {seed} expected {} kinds and observed {} kinds",
                expected.len(),
                actual.len()
            ),
            Self::TooManyUniqueSeeds { value, maximum } => write!(
                formatter,
                "context unique seed count {value} exceeds candidate maximum {maximum}"
            ),
            Self::InsufficientBudget {
                required,
                available,
            } => write!(
                formatter,
                "context budget is insufficient for mandatory seeds: required {required} bytes, available {available} bytes"
            ),
            Self::CostOverflow => formatter.write_str("context cost calculation overflowed"),
            Self::UnsupportedKind => formatter.write_str("context kind has no accepted rendering"),
        }
    }
}

impl std::error::Error for ContextError {}

impl Display for ContextPolicyField {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::MaxDepth => "max_depth",
            Self::MaxCandidates => "max_candidates",
        })
    }
}

impl Display for ContextSeed {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node { id, expected_kind } => {
                write!(formatter, "node({id}")?;
                if let Some(kind) = expected_kind {
                    write!(formatter, ", {kind:?}")?;
                }
                formatter.write_str(")")
            }
            Self::ExactName {
                name,
                expected_kind,
            } => {
                write!(formatter, "exact_name({name}")?;
                if let Some(kind) = expected_kind {
                    write!(formatter, ", {kind:?}")?;
                }
                formatter.write_str(")")
            }
        }
    }
}

/// Stateless deterministic Context Engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct ContextEngine;

impl ContextEngine {
    /// Resolves and canonicalizes every request seed against one graph snapshot.
    ///
    /// # Errors
    ///
    /// Returns a typed missing, ambiguous, incompatible, or limit error without
    /// producing partial Context state.
    pub fn resolve_request(
        &self,
        graph: &SemanticGraph,
        request: &ContextRequest,
    ) -> Result<ResolvedContextRequest, ContextError> {
        let query = graph.query();
        let mut canonical_seeds = request.seeds.clone();
        canonical_seeds.sort();
        canonical_seeds.dedup();

        let mut resolved = BTreeSet::new();
        for seed in &canonical_seeds {
            let node = match seed {
                ContextSeed::Node { id, expected_kind } => {
                    let Some(node) = query.node(id) else {
                        return Err(ContextError::MissingSeed { seed: seed.clone() });
                    };
                    if expected_kind.is_some_and(|kind| kind != node.kind()) {
                        return Err(ContextError::IncompatibleSeed {
                            seed: seed.clone(),
                            expected: expected_kind.iter().copied().collect(),
                            actual: vec![node.kind()],
                        });
                    }
                    node
                }
                ContextSeed::ExactName {
                    name,
                    expected_kind,
                } => {
                    let all = query.nodes_by_name(name);
                    let compatible = all
                        .iter()
                        .copied()
                        .filter(|node| expected_kind.is_none_or(|kind| kind == node.kind()))
                        .collect::<Vec<_>>();

                    if compatible.is_empty() {
                        if all.is_empty() {
                            return Err(ContextError::MissingSeed { seed: seed.clone() });
                        }

                        let actual = all
                            .iter()
                            .map(|node| node.kind())
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect();
                        return Err(ContextError::IncompatibleSeed {
                            seed: seed.clone(),
                            expected: expected_kind.iter().copied().collect(),
                            actual,
                        });
                    }
                    if compatible.len() > 1 {
                        return Err(ContextError::AmbiguousSeed {
                            seed: seed.clone(),
                            candidates: compatible
                                .iter()
                                .map(|node| NodeId::new(node.id().as_str()))
                                .collect(),
                        });
                    }

                    compatible[0]
                }
            };

            if !request.policy.allows_node(node.kind()) {
                return Err(ContextError::IncompatibleSeed {
                    seed: seed.clone(),
                    expected: request
                        .policy
                        .node_kinds
                        .iter()
                        .flat_map(|kinds| kinds.iter().copied())
                        .collect(),
                    actual: vec![node.kind()],
                });
            }

            resolved.insert(NodeId::new(node.id().as_str()));
        }

        if resolved.len() > request.policy.max_candidates {
            return Err(ContextError::TooManyUniqueSeeds {
                value: resolved.len(),
                maximum: request.policy.max_candidates,
            });
        }

        Ok(ResolvedContextRequest {
            intent: request.intent,
            seeds: resolved.into_iter().collect(),
            budget: request.budget,
            policy: request.policy.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{GraphNode, NodeId, NodeKind, SemanticGraph};

    use super::{
        ContextEngine, ContextError, ContextIntent, ContextPolicy, ContextPolicyField,
        ContextRequest, ContextSeed, ContextTraversalDirection, MAX_CONTEXT_BUDGET_BYTES,
        MAX_CONTEXT_CANDIDATES, MAX_CONTEXT_DEPTH, MAX_CONTEXT_SEEDS,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn graph(nodes: &[(&str, &str, NodeKind)]) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        for (node_id, node_name, kind) in nodes {
            graph.insert_node(GraphNode::new(id(node_id), name(node_name), *kind));
        }
        graph
    }

    fn request(
        seeds: Vec<ContextSeed>,
        policy: ContextPolicy,
    ) -> Result<ContextRequest, ContextError> {
        ContextRequest::new(ContextIntent::Explain, seeds, 4_096, policy)
    }

    #[test]
    fn request_validation_uses_accepted_precedence() {
        let invalid_policy = ContextPolicy::new(
            ContextTraversalDirection::Both,
            BTreeSet::new(),
            None,
            MAX_CONTEXT_DEPTH + 1,
            0,
        );

        assert_eq!(
            ContextRequest::new(
                ContextIntent::Explain,
                Vec::new(),
                0,
                invalid_policy.clone()
            ),
            Err(ContextError::InvalidBudget {
                value: 0,
                minimum: 1,
                maximum: MAX_CONTEXT_BUDGET_BYTES,
            })
        );
        assert_eq!(
            ContextRequest::new(ContextIntent::Explain, Vec::new(), 1, invalid_policy),
            Err(ContextError::InvalidPolicy {
                field: ContextPolicyField::MaxDepth,
                value: MAX_CONTEXT_DEPTH + 1,
                minimum: 0,
                maximum: MAX_CONTEXT_DEPTH,
            })
        );
    }

    #[test]
    fn request_rejects_empty_sets_seed_bounds_and_invalid_node_id() {
        let empty_edges =
            ContextPolicy::new(ContextTraversalDirection::Both, BTreeSet::new(), None, 1, 1);
        assert_eq!(
            request(vec![ContextSeed::node("node")], empty_edges),
            Err(ContextError::EmptyEdgeKinds)
        );

        let empty_nodes = ContextPolicy::new(
            ContextTraversalDirection::Both,
            BTreeSet::from([oneagent_graph::EdgeKind::Calls]),
            Some(BTreeSet::new()),
            1,
            1,
        );
        assert_eq!(
            request(vec![ContextSeed::node("node")], empty_nodes),
            Err(ContextError::EmptyNodeKinds)
        );

        assert_eq!(
            request(Vec::new(), ContextPolicy::default()),
            Err(ContextError::InvalidSeedCount {
                value: 0,
                minimum: 1,
                maximum: MAX_CONTEXT_SEEDS,
            })
        );
        assert_eq!(
            request(vec![ContextSeed::node("   ")], ContextPolicy::default()),
            Err(ContextError::InvalidSeedIdentifier {
                value: "   ".to_owned(),
            })
        );
    }

    #[test]
    fn node_seeds_resolve_deduplicate_and_sort() {
        let graph = graph(&[
            ("node.b", "B", NodeKind::Function),
            ("node.a", "A", NodeKind::Procedure),
        ]);
        let request = request(
            vec![
                ContextSeed::node("node.b"),
                ContextSeed::node("node.a"),
                ContextSeed::node("node.b"),
            ],
            ContextPolicy::default(),
        )
        .expect("request must be valid");

        let resolved = ContextEngine
            .resolve_request(&graph, &request)
            .expect("seeds must resolve");

        assert_eq!(
            resolved.seeds(),
            &[NodeId::new("node.a"), NodeId::new("node.b")]
        );
    }

    #[test]
    fn exact_name_seeds_distinguish_missing_ambiguous_and_incompatible() {
        let graph = graph(&[
            ("node.a", "Shared", NodeKind::Procedure),
            ("node.b", "Shared", NodeKind::Function),
        ]);

        let missing = request(
            vec![ContextSeed::exact_name(name("Missing"))],
            ContextPolicy::default(),
        )
        .expect("request must be valid");
        assert!(matches!(
            ContextEngine.resolve_request(&graph, &missing),
            Err(ContextError::MissingSeed { .. })
        ));

        let ambiguous = request(
            vec![ContextSeed::exact_name(name("Shared"))],
            ContextPolicy::default(),
        )
        .expect("request must be valid");
        assert_eq!(
            ContextEngine.resolve_request(&graph, &ambiguous),
            Err(ContextError::AmbiguousSeed {
                seed: ContextSeed::exact_name(name("Shared")),
                candidates: vec![NodeId::new("node.a"), NodeId::new("node.b")],
            })
        );

        let incompatible = request(
            vec![ContextSeed::exact_name_with_kind(
                name("Shared"),
                NodeKind::Module,
            )],
            ContextPolicy::default(),
        )
        .expect("request must be valid");
        assert_eq!(
            ContextEngine.resolve_request(&graph, &incompatible),
            Err(ContextError::IncompatibleSeed {
                seed: ContextSeed::exact_name_with_kind(name("Shared"), NodeKind::Module),
                expected: vec![NodeKind::Module],
                actual: vec![NodeKind::Procedure, NodeKind::Function],
            })
        );
    }

    #[test]
    fn kind_constraints_and_policy_are_all_or_nothing() {
        let graph = graph(&[("node.a", "A", NodeKind::Procedure)]);
        let wrong_seed_kind = request(
            vec![ContextSeed::node_with_kind("node.a", NodeKind::Function)],
            ContextPolicy::default(),
        )
        .expect("request must be valid");
        assert!(matches!(
            ContextEngine.resolve_request(&graph, &wrong_seed_kind),
            Err(ContextError::IncompatibleSeed { .. })
        ));

        let policy = ContextPolicy::new(
            ContextTraversalDirection::Both,
            BTreeSet::from([oneagent_graph::EdgeKind::Calls]),
            Some(BTreeSet::from([NodeKind::Function])),
            1,
            1,
        );
        let policy_mismatch =
            request(vec![ContextSeed::node("node.a")], policy).expect("request must be valid");
        assert!(matches!(
            ContextEngine.resolve_request(&graph, &policy_mismatch),
            Err(ContextError::IncompatibleSeed { .. })
        ));
    }

    #[test]
    fn canonical_seed_order_makes_first_failure_repeatable() {
        let graph = SemanticGraph::new();
        let first = request(
            vec![
                ContextSeed::node("missing.z"),
                ContextSeed::node("missing.a"),
            ],
            ContextPolicy::default(),
        )
        .expect("request must be valid");
        let second = request(
            vec![
                ContextSeed::node("missing.a"),
                ContextSeed::node("missing.z"),
            ],
            ContextPolicy::default(),
        )
        .expect("request must be valid");

        assert_eq!(
            ContextEngine.resolve_request(&graph, &first),
            ContextEngine.resolve_request(&graph, &second)
        );
        assert_eq!(
            ContextEngine.resolve_request(&graph, &first),
            Err(ContextError::MissingSeed {
                seed: ContextSeed::node("missing.a"),
            })
        );
    }

    #[test]
    fn unique_seed_count_cannot_exceed_candidate_limit() {
        let graph = graph(&[
            ("node.a", "A", NodeKind::Procedure),
            ("node.b", "B", NodeKind::Function),
        ]);
        let policy = ContextPolicy::new(
            ContextTraversalDirection::Both,
            BTreeSet::from([oneagent_graph::EdgeKind::Calls]),
            None,
            1,
            1,
        );
        let request = request(
            vec![ContextSeed::node("node.a"), ContextSeed::node("node.b")],
            policy,
        )
        .expect("request must be valid");

        assert_eq!(
            ContextEngine.resolve_request(&graph, &request),
            Err(ContextError::TooManyUniqueSeeds {
                value: 2,
                maximum: 1,
            })
        );
    }

    #[test]
    fn default_policy_matches_the_accepted_limits_and_vocabulary() {
        let policy = ContextPolicy::default();

        assert_eq!(policy.direction(), ContextTraversalDirection::Both);
        assert_eq!(policy.edge_kinds().len(), 11);
        assert_eq!(policy.node_kinds(), None);
        assert_eq!(policy.max_depth(), 2);
        assert_eq!(policy.max_candidates(), 32);
        assert_eq!(MAX_CONTEXT_CANDIDATES, 128);
    }
}
