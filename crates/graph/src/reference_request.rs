//! Source-independent semantic reference request lifecycle and ledger.

use oneagent_common::EntityId;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use crate::{NodeKind, Provenance, ResolutionState, SemanticReference};

/// Stable identity of one semantic reference request.
///
/// Identity is derived only from the source node, semantic category, typed
/// target expression, and canonical expected-kind set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticReferenceRequestId(String);

impl SemanticReferenceRequestId {
    fn from_identity(identity: &ReferenceRequestIdentity) -> Self {
        let mut value = String::from("reference_request");
        push_component(&mut value, "source", identity.source_node.as_str());
        push_component(&mut value, "category", identity.category.as_str());
        push_component(
            &mut value,
            "reference",
            &reference_encoding(&identity.reference),
        );
        for kind in &identity.expected_kinds {
            push_component(&mut value, "expected_kind", &node_kind_encoding(*kind));
        }
        Self(value)
    }

    /// Returns the canonical string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier and returns its canonical string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SemanticReferenceRequestId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for SemanticReferenceRequestId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Source-independent semantic intent of a reference request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticReferenceCategory {
    /// Metadata type reference from an accepted semantic source.
    MetadataType,
    /// Callable symbol reference.
    Callable,
    /// Persistent query source reference.
    QuerySource,
    /// Persistent write target reference.
    WriteTarget,
    /// Protected resource reference.
    ProtectedResource,
    /// Subsystem membership reference.
    SubsystemMember,
    /// Metadata extension target reference.
    ExtensionTarget,
}

impl SemanticReferenceCategory {
    /// Returns the stable machine-readable category name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataType => "metadata_type",
            Self::Callable => "callable",
            Self::QuerySource => "query_source",
            Self::WriteTarget => "write_target",
            Self::ProtectedResource => "protected_resource",
            Self::SubsystemMember => "subsystem_member",
            Self::ExtensionTarget => "extension_target",
        }
    }
}

impl Display for SemanticReferenceCategory {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Typed lifecycle outcome of a semantic reference request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticReferenceRequestOutcome {
    /// Accepted request collected before resolution.
    Collected,
    /// Request resolved to exactly one compatible target.
    Resolved,
    /// Complete workspace has no matching target.
    MissingTarget,
    /// Explicitly partial workspace cannot complete resolution.
    PartialWorkspace,
    /// Multiple compatible targets remain.
    AmbiguousTarget,
    /// Inspected targets have incompatible semantic kinds.
    IncompatibleTargetKind,
    /// Known owner and child candidates violate ownership.
    InvalidOwnerReference,
}

impl SemanticReferenceRequestOutcome {
    /// Returns the stable machine-readable outcome name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Collected => "collected",
            Self::Resolved => "resolved",
            Self::MissingTarget => "missing_target",
            Self::PartialWorkspace => "partial_workspace",
            Self::AmbiguousTarget => "ambiguous_target",
            Self::IncompatibleTargetKind => "incompatible_target_kind",
            Self::InvalidOwnerReference => "invalid_owner_reference",
        }
    }
}

impl Display for SemanticReferenceRequestOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferenceRequestIdentity {
    source_node: EntityId,
    category: SemanticReferenceCategory,
    reference: SemanticReference,
    expected_kinds: Vec<NodeKind>,
}

impl ReferenceRequestIdentity {
    fn new(
        source_node: EntityId,
        category: SemanticReferenceCategory,
        reference: SemanticReference,
        expected_kinds: impl IntoIterator<Item = NodeKind>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        let mut expected_kinds = expected_kinds.into_iter().collect::<Vec<_>>();
        expected_kinds.sort();
        expected_kinds.dedup();
        if expected_kinds.is_empty() {
            return Err(SemanticReferenceRequestError::MissingExpectedKinds);
        }

        Ok(Self {
            source_node,
            category,
            reference,
            expected_kinds,
        })
    }
}

/// Immutable source-independent semantic reference request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticReferenceRequest {
    id: SemanticReferenceRequestId,
    source_node: EntityId,
    category: SemanticReferenceCategory,
    reference: SemanticReference,
    expected_kinds: Vec<NodeKind>,
    candidates: Vec<EntityId>,
    state: ResolutionState,
    outcome: SemanticReferenceRequestOutcome,
    provenance: Vec<Provenance>,
}

impl SemanticReferenceRequest {
    /// Creates an accepted unresolved request with collection provenance.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty expected-kind set, missing provenance, or
    /// collection provenance with a state other than `Unresolved`.
    pub fn collected(
        source_node: EntityId,
        category: SemanticReferenceCategory,
        reference: SemanticReference,
        expected_kinds: impl IntoIterator<Item = NodeKind>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        let provenance = provenance.into_iter().collect::<Vec<_>>();
        validate_stage_provenance(&provenance, ResolutionState::Unresolved)?;
        let identity =
            ReferenceRequestIdentity::new(source_node, category, reference, expected_kinds)?;
        Self::new(
            identity,
            Vec::new(),
            ResolutionState::Unresolved,
            SemanticReferenceRequestOutcome::Collected,
            provenance,
        )
    }

    /// Creates a request proven partial at collection time.
    ///
    /// # Errors
    ///
    /// Returns an error when identity, provenance, or partial lifecycle
    /// invariants are not satisfied.
    pub fn partial_workspace(
        source_node: EntityId,
        category: SemanticReferenceCategory,
        reference: SemanticReference,
        expected_kinds: impl IntoIterator<Item = NodeKind>,
        candidates: impl IntoIterator<Item = EntityId>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        let provenance = provenance.into_iter().collect::<Vec<_>>();
        validate_stage_provenance(&provenance, ResolutionState::Partial)?;
        let identity =
            ReferenceRequestIdentity::new(source_node, category, reference, expected_kinds)?;
        Self::new(
            identity,
            candidates.into_iter().collect(),
            ResolutionState::Partial,
            SemanticReferenceRequestOutcome::PartialWorkspace,
            provenance,
        )
    }

    /// Resolves a collected request to one compatible candidate.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request, the candidate
    /// kind is incompatible, or resolver provenance is invalid.
    pub fn into_resolved(
        self,
        candidate: EntityId,
        candidate_kind: NodeKind,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        if !self.expected_kinds.contains(&candidate_kind) {
            return Err(
                SemanticReferenceRequestError::IncompatibleResolvedCandidate {
                    actual: candidate_kind,
                    expected: self.expected_kinds.clone(),
                },
            );
        }
        self.transition(
            SemanticReferenceRequestOutcome::Resolved,
            vec![candidate],
            ResolutionState::Resolved,
            provenance,
        )
    }

    /// Completes a collected request as missing in a complete workspace.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request or resolver
    /// provenance is invalid.
    pub fn into_missing_target(
        self,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        self.transition(
            SemanticReferenceRequestOutcome::MissingTarget,
            Vec::new(),
            ResolutionState::Unresolved,
            provenance,
        )
    }

    /// Completes a collected request as partial in an incomplete workspace.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request or resolver
    /// provenance is invalid.
    pub fn into_partial_workspace(
        self,
        candidates: impl IntoIterator<Item = EntityId>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        self.transition(
            SemanticReferenceRequestOutcome::PartialWorkspace,
            candidates.into_iter().collect(),
            ResolutionState::Partial,
            provenance,
        )
    }

    /// Completes a collected request with multiple compatible candidates.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request, fewer than two
    /// distinct candidates are supplied, or resolver provenance is invalid.
    pub fn into_ambiguous_target(
        self,
        candidates: impl IntoIterator<Item = EntityId>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        self.transition(
            SemanticReferenceRequestOutcome::AmbiguousTarget,
            candidates.into_iter().collect(),
            ResolutionState::Ambiguous,
            provenance,
        )
    }

    /// Completes a collected request with incompatible inspected targets.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request, no candidate is
    /// supplied, or resolver provenance is invalid.
    pub fn into_incompatible_target_kind(
        self,
        candidates: impl IntoIterator<Item = EntityId>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        self.transition(
            SemanticReferenceRequestOutcome::IncompatibleTargetKind,
            candidates.into_iter().collect(),
            ResolutionState::Unresolved,
            provenance,
        )
    }

    /// Completes a collected request with an invalid owner relationship.
    ///
    /// # Errors
    ///
    /// Returns an error when this is not a collected request, no candidate is
    /// supplied, or resolver provenance is invalid.
    pub fn into_invalid_owner_reference(
        self,
        candidates: impl IntoIterator<Item = EntityId>,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        self.transition(
            SemanticReferenceRequestOutcome::InvalidOwnerReference,
            candidates.into_iter().collect(),
            ResolutionState::Unresolved,
            provenance,
        )
    }

    fn new(
        identity: ReferenceRequestIdentity,
        mut candidates: Vec<EntityId>,
        state: ResolutionState,
        outcome: SemanticReferenceRequestOutcome,
        provenance: Vec<Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        candidates.sort();
        candidates.dedup();
        validate_lifecycle(state, outcome, candidates.len())?;
        let provenance = normalized_provenance(provenance)?;
        let id = SemanticReferenceRequestId::from_identity(&identity);

        Ok(Self {
            id,
            source_node: identity.source_node,
            category: identity.category,
            reference: identity.reference,
            expected_kinds: identity.expected_kinds,
            candidates,
            state,
            outcome,
            provenance,
        })
    }

    fn transition(
        self,
        outcome: SemanticReferenceRequestOutcome,
        candidates: Vec<EntityId>,
        state: ResolutionState,
        provenance: impl IntoIterator<Item = Provenance>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        if self.outcome != SemanticReferenceRequestOutcome::Collected {
            return Err(SemanticReferenceRequestError::InvalidTransition {
                from: self.outcome,
                to: outcome,
            });
        }
        let resolver_provenance = provenance.into_iter().collect::<Vec<_>>();
        validate_stage_provenance(&resolver_provenance, state)?;
        let mut combined_provenance = self.provenance;
        combined_provenance.extend(resolver_provenance);
        let identity = ReferenceRequestIdentity {
            source_node: self.source_node,
            category: self.category,
            reference: self.reference,
            expected_kinds: self.expected_kinds,
        };
        Self::new(identity, candidates, state, outcome, combined_provenance)
    }

    /// Returns the stable request identity.
    #[must_use]
    pub const fn id(&self) -> &SemanticReferenceRequestId {
        &self.id
    }

    /// Returns the semantic source node identity.
    #[must_use]
    pub const fn source_node(&self) -> &EntityId {
        &self.source_node
    }

    /// Returns the semantic request category.
    #[must_use]
    pub const fn category(&self) -> SemanticReferenceCategory {
        self.category
    }

    /// Returns the canonical typed target expression.
    #[must_use]
    pub const fn reference(&self) -> &SemanticReference {
        &self.reference
    }

    /// Returns canonical accepted target kinds.
    #[must_use]
    pub fn expected_kinds(&self) -> &[NodeKind] {
        &self.expected_kinds
    }

    /// Returns canonical candidate identities.
    #[must_use]
    pub fn candidates(&self) -> &[EntityId] {
        &self.candidates
    }

    /// Returns the current resolution state.
    #[must_use]
    pub const fn state(&self) -> ResolutionState {
        self.state
    }

    /// Returns the typed lifecycle outcome.
    #[must_use]
    pub const fn outcome(&self) -> SemanticReferenceRequestOutcome {
        self.outcome
    }

    /// Returns canonical collection and resolver provenance.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    fn has_same_identity_content(&self, other: &Self) -> bool {
        self.source_node == other.source_node
            && self.category == other.category
            && self.reference == other.reference
            && self.expected_kinds == other.expected_kinds
    }

    fn has_same_terminal_content(&self, other: &Self) -> bool {
        self.candidates == other.candidates
            && self.state == other.state
            && self.outcome == other.outcome
    }
}

/// Checked invariant or ledger aggregation error for reference requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticReferenceRequestError {
    /// A semantic request must declare at least one expected target kind.
    MissingExpectedKinds,
    /// A semantic request or lifecycle stage must retain provenance.
    MissingProvenance,
    /// Request provenance cannot use `NotApplicable` resolution state.
    NotApplicableProvenance,
    /// Stage provenance does not match the state produced by the stage.
    InvalidStageProvenance {
        /// Required stage resolution state.
        expected: ResolutionState,
        /// Observed provenance resolution state.
        actual: ResolutionState,
    },
    /// Lifecycle state and typed outcome contradict each other.
    InvalidLifecycle {
        /// Supplied resolution state.
        state: ResolutionState,
        /// Supplied typed outcome.
        outcome: SemanticReferenceRequestOutcome,
    },
    /// Candidate cardinality contradicts the typed outcome.
    InvalidCandidateCount {
        /// Typed request outcome.
        outcome: SemanticReferenceRequestOutcome,
        /// Actual canonical candidate count.
        actual: usize,
        /// Human-readable cardinality requirement.
        required: &'static str,
    },
    /// Resolved target kind is not one of the accepted kinds.
    IncompatibleResolvedCandidate {
        /// Actual resolved target kind.
        actual: NodeKind,
        /// Canonical expected target kinds.
        expected: Vec<NodeKind>,
    },
    /// Only collected requests may enter a terminal state.
    InvalidTransition {
        /// Existing outcome.
        from: SemanticReferenceRequestOutcome,
        /// Requested terminal outcome.
        to: SemanticReferenceRequestOutcome,
    },
    /// Two values share an identifier but disagree on immutable identity data.
    ConflictingIdentity {
        /// Colliding request identifier.
        id: SemanticReferenceRequestId,
    },
    /// Duplicate observations disagree on terminal lifecycle content.
    ConflictingTerminalContent {
        /// Conflicting request identifier.
        id: SemanticReferenceRequestId,
    },
}

impl Display for SemanticReferenceRequestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExpectedKinds => {
                formatter.write_str("semantic reference request must declare expected kinds")
            }
            Self::MissingProvenance => {
                formatter.write_str("semantic reference request must retain provenance")
            }
            Self::NotApplicableProvenance => formatter
                .write_str("semantic reference request provenance cannot be not applicable"),
            Self::InvalidStageProvenance { expected, actual } => write!(
                formatter,
                "semantic reference request stage provenance is {actual:?}; expected {expected:?}"
            ),
            Self::InvalidLifecycle { state, outcome } => write!(
                formatter,
                "semantic reference request outcome {outcome} is incompatible with {state:?} state"
            ),
            Self::InvalidCandidateCount {
                outcome,
                actual,
                required,
            } => write!(
                formatter,
                "semantic reference request outcome {outcome} requires {required}; got {actual} candidates"
            ),
            Self::IncompatibleResolvedCandidate { actual, expected } => write!(
                formatter,
                "resolved semantic reference candidate has kind {actual:?}; expected one of {expected:?}"
            ),
            Self::InvalidTransition { from, to } => write!(
                formatter,
                "semantic reference request cannot transition from {from} to {to}"
            ),
            Self::ConflictingIdentity { id } => write!(
                formatter,
                "semantic reference request `{id}` has conflicting identity content"
            ),
            Self::ConflictingTerminalContent { id } => write!(
                formatter,
                "semantic reference request `{id}` has conflicting terminal content"
            ),
        }
    }
}

impl std::error::Error for SemanticReferenceRequestError {}

/// Deterministically ordered semantic reference request ledger.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SemanticReferenceRequestLedger {
    requests: Vec<SemanticReferenceRequest>,
}

impl SemanticReferenceRequestLedger {
    /// Creates an empty request ledger.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Aggregates requests in deterministic identity order.
    ///
    /// # Errors
    ///
    /// Returns an error when duplicate identities disagree on immutable or
    /// terminal content.
    pub fn from_requests(
        requests: impl IntoIterator<Item = SemanticReferenceRequest>,
    ) -> Result<Self, SemanticReferenceRequestError> {
        let mut ledger = Self::new();
        for request in requests {
            ledger.insert(request)?;
        }
        Ok(ledger)
    }

    /// Inserts or provenance-merges one canonical request observation.
    ///
    /// Returns `true` when a new identity was inserted and `false` when
    /// equivalent request content was merged.
    ///
    /// # Errors
    ///
    /// Returns an error for an identity collision or conflicting terminal
    /// content.
    pub fn insert(
        &mut self,
        request: SemanticReferenceRequest,
    ) -> Result<bool, SemanticReferenceRequestError> {
        match self
            .requests
            .binary_search_by(|candidate| candidate.id.cmp(&request.id))
        {
            Ok(index) => {
                let existing = &self.requests[index];
                if !existing.has_same_identity_content(&request) {
                    return Err(SemanticReferenceRequestError::ConflictingIdentity {
                        id: request.id,
                    });
                }
                if !existing.has_same_terminal_content(&request) {
                    return Err(SemanticReferenceRequestError::ConflictingTerminalContent {
                        id: request.id,
                    });
                }
                let mut provenance = existing.provenance.clone();
                provenance.extend(request.provenance);
                self.requests[index].provenance = normalized_provenance(provenance)?;
                Ok(false)
            }
            Err(index) => {
                self.requests.insert(index, request);
                Ok(true)
            }
        }
    }

    /// Returns requests in stable request-identity order.
    #[must_use]
    pub fn requests(&self) -> &[SemanticReferenceRequest] {
        &self.requests
    }

    /// Returns the number of canonical requests.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.requests.len()
    }

    /// Returns whether the ledger is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Creates an immutable request-ledger query view.
    #[must_use]
    pub fn query(&self) -> SemanticReferenceRequestQuery<'_> {
        SemanticReferenceRequestQuery {
            requests: &self.requests,
        }
    }
}

/// Immutable query view over a canonical request ledger.
#[derive(Debug, Clone, Copy)]
pub struct SemanticReferenceRequestQuery<'ledger> {
    requests: &'ledger [SemanticReferenceRequest],
}

impl<'ledger> SemanticReferenceRequestQuery<'ledger> {
    /// Returns all requests in stable identity order.
    #[must_use]
    pub const fn all(self) -> &'ledger [SemanticReferenceRequest] {
        self.requests
    }

    /// Finds a request by stable identity.
    #[must_use]
    pub fn request(
        self,
        id: &SemanticReferenceRequestId,
    ) -> Option<&'ledger SemanticReferenceRequest> {
        self.requests
            .binary_search_by(|request| request.id.cmp(id))
            .ok()
            .map(|index| &self.requests[index])
    }

    /// Returns requests with the supplied semantic category.
    #[must_use]
    pub fn by_category(
        self,
        category: SemanticReferenceCategory,
    ) -> Vec<&'ledger SemanticReferenceRequest> {
        self.filter(|request| request.category == category)
    }

    /// Returns requests originating from the supplied semantic node.
    #[must_use]
    pub fn by_source(self, source_node: &EntityId) -> Vec<&'ledger SemanticReferenceRequest> {
        self.filter(|request| request.source_node == *source_node)
    }

    /// Returns requests with the supplied lifecycle outcome.
    #[must_use]
    pub fn by_outcome(
        self,
        outcome: SemanticReferenceRequestOutcome,
    ) -> Vec<&'ledger SemanticReferenceRequest> {
        self.filter(|request| request.outcome == outcome)
    }

    /// Returns requests in the supplied resolution state.
    #[must_use]
    pub fn by_state(self, state: ResolutionState) -> Vec<&'ledger SemanticReferenceRequest> {
        self.filter(|request| request.state == state)
    }

    /// Returns requests accepting the supplied target kind.
    #[must_use]
    pub fn by_expected_kind(self, kind: NodeKind) -> Vec<&'ledger SemanticReferenceRequest> {
        self.filter(|request| request.expected_kinds.contains(&kind))
    }

    fn filter(
        self,
        predicate: impl Fn(&SemanticReferenceRequest) -> bool,
    ) -> Vec<&'ledger SemanticReferenceRequest> {
        self.requests
            .iter()
            .filter(|request| predicate(request))
            .collect()
    }
}

fn validate_lifecycle(
    state: ResolutionState,
    outcome: SemanticReferenceRequestOutcome,
    candidate_count: usize,
) -> Result<(), SemanticReferenceRequestError> {
    let expected_state = match outcome {
        SemanticReferenceRequestOutcome::Collected
        | SemanticReferenceRequestOutcome::MissingTarget
        | SemanticReferenceRequestOutcome::IncompatibleTargetKind
        | SemanticReferenceRequestOutcome::InvalidOwnerReference => ResolutionState::Unresolved,
        SemanticReferenceRequestOutcome::Resolved => ResolutionState::Resolved,
        SemanticReferenceRequestOutcome::PartialWorkspace => ResolutionState::Partial,
        SemanticReferenceRequestOutcome::AmbiguousTarget => ResolutionState::Ambiguous,
    };
    if state != expected_state || state == ResolutionState::NotApplicable {
        return Err(SemanticReferenceRequestError::InvalidLifecycle { state, outcome });
    }

    let valid_candidate_count = match outcome {
        SemanticReferenceRequestOutcome::Collected
        | SemanticReferenceRequestOutcome::MissingTarget => candidate_count == 0,
        SemanticReferenceRequestOutcome::Resolved => candidate_count == 1,
        SemanticReferenceRequestOutcome::PartialWorkspace => true,
        SemanticReferenceRequestOutcome::AmbiguousTarget => candidate_count >= 2,
        SemanticReferenceRequestOutcome::IncompatibleTargetKind
        | SemanticReferenceRequestOutcome::InvalidOwnerReference => candidate_count >= 1,
    };

    if valid_candidate_count {
        Ok(())
    } else {
        Err(SemanticReferenceRequestError::InvalidCandidateCount {
            outcome,
            actual: candidate_count,
            required: candidate_requirement(outcome),
        })
    }
}

const fn candidate_requirement(outcome: SemanticReferenceRequestOutcome) -> &'static str {
    match outcome {
        SemanticReferenceRequestOutcome::Collected
        | SemanticReferenceRequestOutcome::MissingTarget => "no candidates",
        SemanticReferenceRequestOutcome::Resolved => "exactly one candidate",
        SemanticReferenceRequestOutcome::PartialWorkspace => "any canonical candidates",
        SemanticReferenceRequestOutcome::AmbiguousTarget => "at least two candidates",
        SemanticReferenceRequestOutcome::IncompatibleTargetKind
        | SemanticReferenceRequestOutcome::InvalidOwnerReference => "at least one candidate",
    }
}

fn validate_stage_provenance(
    provenance: &[Provenance],
    expected: ResolutionState,
) -> Result<(), SemanticReferenceRequestError> {
    if provenance.is_empty() {
        return Err(SemanticReferenceRequestError::MissingProvenance);
    }
    for value in provenance {
        if value.resolution() == ResolutionState::NotApplicable {
            return Err(SemanticReferenceRequestError::NotApplicableProvenance);
        }
        if value.resolution() != expected {
            return Err(SemanticReferenceRequestError::InvalidStageProvenance {
                expected,
                actual: value.resolution(),
            });
        }
    }
    Ok(())
}

fn normalized_provenance(
    mut provenance: Vec<Provenance>,
) -> Result<Vec<Provenance>, SemanticReferenceRequestError> {
    if provenance.is_empty() {
        return Err(SemanticReferenceRequestError::MissingProvenance);
    }
    if provenance
        .iter()
        .any(|value| value.resolution() == ResolutionState::NotApplicable)
    {
        return Err(SemanticReferenceRequestError::NotApplicableProvenance);
    }
    provenance.sort_by(compare_provenance);
    provenance.dedup();
    Ok(provenance)
}

fn compare_provenance(left: &Provenance, right: &Provenance) -> Ordering {
    (
        left.source(),
        left.producer(),
        left.origin(),
        left.confidence(),
        left.resolution(),
    )
        .cmp(&(
            right.source(),
            right.producer(),
            right.origin(),
            right.confidence(),
            right.resolution(),
        ))
}

fn push_component(target: &mut String, label: &str, value: &str) {
    use std::fmt::Write as _;

    write!(target, ";{label}#{}:{value}", value.len())
        .expect("writing deterministic identity to a String must succeed");
}

fn reference_encoding(reference: &SemanticReference) -> String {
    let mut value = String::new();
    match reference {
        SemanticReference::Raw(raw) => {
            value.push_str("raw");
            push_component(&mut value, "value", raw);
        }
        SemanticReference::NodeId(id) => {
            value.push_str("node_id");
            push_component(&mut value, "value", id);
        }
        SemanticReference::Name(name) => {
            value.push_str("name");
            push_component(&mut value, "value", name.as_str());
        }
        SemanticReference::Child { owner, name } => {
            value.push_str("child");
            push_component(&mut value, "owner", owner.as_str());
            push_component(&mut value, "name", name.as_str());
        }
        SemanticReference::Owner { child } => {
            value.push_str("owner");
            push_component(&mut value, "child", child.as_str());
        }
        SemanticReference::OwnedChild { owner, child } => {
            value.push_str("owned_child");
            push_component(&mut value, "owner", owner.as_str());
            push_component(&mut value, "child", child.as_str());
        }
    }
    value
}

fn node_kind_encoding(kind: NodeKind) -> String {
    match kind {
        NodeKind::Metadata(metadata_kind) => format!("metadata.{}", metadata_kind.as_str()),
        NodeKind::Module => "module".to_owned(),
        NodeKind::Procedure => "procedure".to_owned(),
        NodeKind::Function => "function".to_owned(),
        NodeKind::Query => "query".to_owned(),
        NodeKind::DataCompositionSchema => "data_composition_schema".to_owned(),
        NodeKind::DataSet => "data_set".to_owned(),
        NodeKind::DataCompositionField => "data_composition_field".to_owned(),
        NodeKind::Form => "form".to_owned(),
        NodeKind::Command => "command".to_owned(),
        NodeKind::Attribute => "attribute".to_owned(),
        NodeKind::StandardAttribute => "standard_attribute".to_owned(),
        NodeKind::TabularSection => "tabular_section".to_owned(),
        NodeKind::Dimension => "dimension".to_owned(),
        NodeKind::Resource => "resource".to_owned(),
        NodeKind::Measure => "measure".to_owned(),
        NodeKind::Role => "role".to_owned(),
        NodeKind::AccessRight => "access_right".to_owned(),
        NodeKind::Subsystem => "subsystem".to_owned(),
        NodeKind::Unknown => "unknown".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;

    use crate::{Confidence, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionState};

    use super::{
        ReferenceRequestIdentity, SemanticReferenceCategory, SemanticReferenceRequest,
        SemanticReferenceRequestError, SemanticReferenceRequestLedger,
        SemanticReferenceRequestOutcome,
    };
    use crate::SemanticReference;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance(producer: &str, state: ResolutionState) -> Provenance {
        Provenance::new(
            Some(id(&format!("source:{producer}"))),
            ProducerId::new(producer),
            if state == ResolutionState::Unresolved {
                FactOrigin::Parsed
            } else {
                FactOrigin::Resolved
            },
            Confidence::Exact,
            state,
        )
    }

    fn collected(source: &str, target: &str) -> SemanticReferenceRequest {
        SemanticReferenceRequest::collected(
            id(source),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name(target)),
            [
                NodeKind::Metadata(MetadataKind::Document),
                NodeKind::Metadata(MetadataKind::Catalog),
                NodeKind::Metadata(MetadataKind::Catalog),
            ],
            [provenance("collector", ResolutionState::Unresolved)],
        )
        .expect("collected request must be valid")
    }

    fn raw_request(
        state: ResolutionState,
        outcome: SemanticReferenceRequestOutcome,
        candidates: Vec<EntityId>,
    ) -> Result<SemanticReferenceRequest, SemanticReferenceRequestError> {
        let identity = ReferenceRequestIdentity::new(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [NodeKind::Metadata(MetadataKind::Catalog)],
        )?;
        SemanticReferenceRequest::new(
            identity,
            candidates,
            state,
            outcome,
            vec![provenance("collector", ResolutionState::Unresolved)],
        )
    }

    #[test]
    fn identity_is_stable_and_excludes_terminal_content() {
        let collected = collected("source", "Target");
        let resolved = collected
            .clone()
            .into_resolved(
                id("target"),
                NodeKind::Metadata(MetadataKind::Catalog),
                [provenance("resolver", ResolutionState::Resolved)],
            )
            .expect("resolution must succeed");
        let missing = collected
            .clone()
            .into_missing_target([provenance("missing", ResolutionState::Unresolved)])
            .expect("missing outcome must succeed");

        assert_eq!(collected.id(), resolved.id());
        assert_eq!(collected.id(), missing.id());
        assert_ne!(collected, resolved);
        assert_ne!(resolved, missing);
        assert!(
            collected
                .id()
                .as_str()
                .contains("reference_request;source#6:source")
        );
    }

    #[test]
    fn identity_is_independent_of_expected_kind_order_and_duplicates() {
        let first = collected("source", "Target");
        let second = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [
                NodeKind::Metadata(MetadataKind::Catalog),
                NodeKind::Metadata(MetadataKind::Document),
            ],
            [provenance("collector", ResolutionState::Unresolved)],
        )
        .expect("request must be valid");

        assert_eq!(first.id(), second.id());
        assert_eq!(first.expected_kinds(), second.expected_kinds());
    }

    #[test]
    fn identity_changes_for_every_identity_component() {
        let base = collected("source", "Target");
        let other_source = collected("other", "Target");
        let other_reference = collected("source", "Other");
        let other_category = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::QuerySource,
            SemanticReference::Name(name("Target")),
            base.expected_kinds().iter().copied(),
            [provenance("collector", ResolutionState::Unresolved)],
        )
        .expect("request must be valid");
        let other_kinds = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [NodeKind::Metadata(MetadataKind::Catalog)],
            [provenance("collector", ResolutionState::Unresolved)],
        )
        .expect("request must be valid");

        for request in [other_source, other_reference, other_category, other_kinds] {
            assert_ne!(base.id(), request.id());
        }
    }

    #[test]
    fn lifecycle_factories_create_every_valid_outcome() {
        let base = collected("source", "Target");
        let resolved = base
            .clone()
            .into_resolved(
                id("target"),
                NodeKind::Metadata(MetadataKind::Catalog),
                [provenance("resolved", ResolutionState::Resolved)],
            )
            .expect("resolved outcome must be valid");
        let missing = base
            .clone()
            .into_missing_target([provenance("missing", ResolutionState::Unresolved)])
            .expect("missing outcome must be valid");
        let partial = base
            .clone()
            .into_partial_workspace(
                [id("known")],
                [provenance("partial", ResolutionState::Partial)],
            )
            .expect("partial outcome must be valid");
        let ambiguous = base
            .clone()
            .into_ambiguous_target(
                [id("two"), id("one"), id("two")],
                [provenance("ambiguous", ResolutionState::Ambiguous)],
            )
            .expect("ambiguous outcome must be valid");
        let incompatible = base
            .clone()
            .into_incompatible_target_kind(
                [id("wrong")],
                [provenance("incompatible", ResolutionState::Unresolved)],
            )
            .expect("incompatible outcome must be valid");
        let invalid_owner = base
            .clone()
            .into_invalid_owner_reference(
                [id("child"), id("owner")],
                [provenance("owner", ResolutionState::Unresolved)],
            )
            .expect("invalid owner outcome must be valid");

        assert_eq!(base.outcome(), SemanticReferenceRequestOutcome::Collected);
        assert_eq!(resolved.state(), ResolutionState::Resolved);
        assert_eq!(
            missing.outcome(),
            SemanticReferenceRequestOutcome::MissingTarget
        );
        assert_eq!(partial.state(), ResolutionState::Partial);
        assert_eq!(ambiguous.candidates(), &[id("one"), id("two")]);
        assert_eq!(
            incompatible.outcome(),
            SemanticReferenceRequestOutcome::IncompatibleTargetKind
        );
        assert_eq!(
            invalid_owner.outcome(),
            SemanticReferenceRequestOutcome::InvalidOwnerReference
        );
    }

    #[test]
    fn lifecycle_rejects_every_state_outcome_mismatch() {
        let cases = [
            (
                ResolutionState::Resolved,
                SemanticReferenceRequestOutcome::Collected,
            ),
            (
                ResolutionState::Unresolved,
                SemanticReferenceRequestOutcome::Resolved,
            ),
            (
                ResolutionState::Partial,
                SemanticReferenceRequestOutcome::MissingTarget,
            ),
            (
                ResolutionState::Resolved,
                SemanticReferenceRequestOutcome::PartialWorkspace,
            ),
            (
                ResolutionState::Unresolved,
                SemanticReferenceRequestOutcome::AmbiguousTarget,
            ),
            (
                ResolutionState::Ambiguous,
                SemanticReferenceRequestOutcome::IncompatibleTargetKind,
            ),
            (
                ResolutionState::NotApplicable,
                SemanticReferenceRequestOutcome::InvalidOwnerReference,
            ),
        ];

        for (state, outcome) in cases {
            let error = raw_request(state, outcome, Vec::new())
                .expect_err("contradictory lifecycle must fail");
            assert!(matches!(
                error,
                SemanticReferenceRequestError::InvalidLifecycle {
                    state: actual_state,
                    outcome: actual_outcome,
                } if actual_state == state && actual_outcome == outcome
            ));
        }
    }

    #[test]
    fn lifecycle_rejects_invalid_candidate_counts_and_kinds() {
        let invalid_counts = [
            (
                SemanticReferenceRequestOutcome::Collected,
                ResolutionState::Unresolved,
                vec![id("unexpected")],
            ),
            (
                SemanticReferenceRequestOutcome::Resolved,
                ResolutionState::Resolved,
                Vec::new(),
            ),
            (
                SemanticReferenceRequestOutcome::MissingTarget,
                ResolutionState::Unresolved,
                vec![id("unexpected")],
            ),
            (
                SemanticReferenceRequestOutcome::AmbiguousTarget,
                ResolutionState::Ambiguous,
                vec![id("only-one")],
            ),
            (
                SemanticReferenceRequestOutcome::IncompatibleTargetKind,
                ResolutionState::Unresolved,
                Vec::new(),
            ),
            (
                SemanticReferenceRequestOutcome::InvalidOwnerReference,
                ResolutionState::Unresolved,
                Vec::new(),
            ),
        ];
        for (outcome, state, candidates) in invalid_counts {
            assert!(matches!(
                raw_request(state, outcome, candidates),
                Err(SemanticReferenceRequestError::InvalidCandidateCount { .. })
            ));
        }

        let error = collected("source", "Target")
            .into_resolved(
                id("target"),
                NodeKind::Metadata(MetadataKind::InformationRegister),
                [provenance("resolver", ResolutionState::Resolved)],
            )
            .expect_err("incompatible candidate kind must fail");
        assert!(matches!(
            error,
            SemanticReferenceRequestError::IncompatibleResolvedCandidate { .. }
        ));
    }

    #[test]
    fn constructors_reject_missing_or_invalid_provenance_and_expected_kinds() {
        let missing_kinds = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [],
            [provenance("collector", ResolutionState::Unresolved)],
        );
        assert_eq!(
            missing_kinds.expect_err("expected kinds must be required"),
            SemanticReferenceRequestError::MissingExpectedKinds
        );

        let missing_provenance = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [NodeKind::Metadata(MetadataKind::Catalog)],
            [],
        );
        assert_eq!(
            missing_provenance.expect_err("provenance must be required"),
            SemanticReferenceRequestError::MissingProvenance
        );

        let invalid_provenance = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [NodeKind::Metadata(MetadataKind::Catalog)],
            [provenance("collector", ResolutionState::NotApplicable)],
        );
        assert_eq!(
            invalid_provenance.expect_err("not-applicable provenance must fail"),
            SemanticReferenceRequestError::NotApplicableProvenance
        );
    }

    #[test]
    fn provenance_and_candidates_are_sorted_and_deduplicated() {
        let request = SemanticReferenceRequest::collected(
            id("source"),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(name("Target")),
            [NodeKind::Metadata(MetadataKind::Catalog)],
            [
                provenance("z-collector", ResolutionState::Unresolved),
                provenance("a-collector", ResolutionState::Unresolved),
                provenance("z-collector", ResolutionState::Unresolved),
            ],
        )
        .expect("request must be valid")
        .into_ambiguous_target(
            [id("z-target"), id("a-target"), id("z-target")],
            [provenance("resolver", ResolutionState::Ambiguous)],
        )
        .expect("ambiguous request must be valid");

        assert_eq!(request.candidates(), &[id("a-target"), id("z-target")]);
        assert_eq!(request.provenance().len(), 3);
        assert_eq!(request.provenance()[0].producer().as_str(), "a-collector");
    }

    #[test]
    fn ledger_merges_duplicate_provenance_and_is_order_independent() {
        let first = collected("source-a", "A");
        let duplicate = SemanticReferenceRequest::collected(
            first.source_node().clone(),
            first.category(),
            first.reference().clone(),
            first.expected_kinds().iter().copied(),
            [provenance("second-collector", ResolutionState::Unresolved)],
        )
        .expect("duplicate request must be valid");
        let second = collected("source-b", "B");
        let normal = SemanticReferenceRequestLedger::from_requests([
            first.clone(),
            duplicate.clone(),
            second.clone(),
        ])
        .expect("ledger must aggregate");
        let reversed = SemanticReferenceRequestLedger::from_requests([second, duplicate, first])
            .expect("reordered ledger must aggregate");

        assert_eq!(normal, reversed);
        assert_eq!(normal.len(), 2);
        assert_eq!(normal.requests()[0].provenance().len(), 2);
    }

    #[test]
    fn ledger_rejects_identity_and_terminal_conflicts() {
        let base = collected("source", "Target");
        let resolved = base
            .clone()
            .into_resolved(
                id("target"),
                NodeKind::Metadata(MetadataKind::Catalog),
                [provenance("resolver", ResolutionState::Resolved)],
            )
            .expect("resolved request must be valid");
        let missing = base
            .clone()
            .into_missing_target([provenance("missing", ResolutionState::Unresolved)])
            .expect("missing request must be valid");
        let mut terminal_ledger = SemanticReferenceRequestLedger::new();
        terminal_ledger
            .insert(resolved)
            .expect("first terminal request must insert");
        assert!(matches!(
            terminal_ledger.insert(missing),
            Err(SemanticReferenceRequestError::ConflictingTerminalContent { .. })
        ));

        let mut conflicting_identity = base.clone();
        conflicting_identity.category = SemanticReferenceCategory::Callable;
        let mut identity_ledger = SemanticReferenceRequestLedger::new();
        identity_ledger
            .insert(base)
            .expect("base request must insert");
        assert!(matches!(
            identity_ledger.insert(conflicting_identity),
            Err(SemanticReferenceRequestError::ConflictingIdentity { .. })
        ));
    }

    #[test]
    fn query_filters_return_stable_request_order() {
        let metadata = collected("source-a", "Metadata")
            .into_missing_target([provenance("metadata", ResolutionState::Unresolved)])
            .expect("missing request must be valid");
        let query_source = SemanticReferenceRequest::collected(
            id("source-b"),
            SemanticReferenceCategory::QuerySource,
            SemanticReference::Raw("Catalog.Products".to_owned()),
            [NodeKind::Metadata(MetadataKind::Catalog)],
            [provenance("query", ResolutionState::Unresolved)],
        )
        .expect("query request must be valid")
        .into_partial_workspace([], [provenance("partial", ResolutionState::Partial)])
        .expect("partial request must be valid");
        let callable = SemanticReferenceRequest::collected(
            id("source-a"),
            SemanticReferenceCategory::Callable,
            SemanticReference::Name(name("Run")),
            [NodeKind::Procedure],
            [provenance("call", ResolutionState::Unresolved)],
        )
        .expect("call request must be valid")
        .into_resolved(
            id("procedure"),
            NodeKind::Procedure,
            [provenance("call-resolver", ResolutionState::Resolved)],
        )
        .expect("call request must resolve");
        let ledger = SemanticReferenceRequestLedger::from_requests([
            query_source.clone(),
            callable.clone(),
            metadata.clone(),
        ])
        .expect("ledger must be valid");
        let query = ledger.query();

        assert_eq!(query.request(metadata.id()), Some(&metadata));
        assert_eq!(
            query.by_category(SemanticReferenceCategory::Callable),
            vec![&callable]
        );
        assert_eq!(query.by_source(&id("source-a")).len(), 2);
        assert_eq!(
            query.by_outcome(SemanticReferenceRequestOutcome::MissingTarget),
            vec![&metadata]
        );
        assert_eq!(
            query.by_state(ResolutionState::Partial),
            vec![&query_source]
        );
        assert_eq!(
            query
                .by_expected_kind(NodeKind::Metadata(MetadataKind::Catalog))
                .len(),
            2
        );
        let ids = query
            .all()
            .iter()
            .map(|request| request.id().as_str())
            .collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
