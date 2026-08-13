//! Common semantic diagnostics produced while building or resolving graph facts.

use oneagent_common::EntityId;
use std::cmp::Ordering;

use crate::{
    Confidence, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionError, ResolutionState,
    SemanticReference,
};

/// Stable machine-readable semantic diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticDiagnosticCode {
    /// A raw semantic reference does not have the required source format.
    ReferenceMalformedFormat,
    /// A raw semantic reference uses a source prefix unsupported by the producer.
    ReferenceUnsupportedPrefix,
    /// A semantic reference has no resolved target.
    ReferenceUnresolved,
    /// A semantic reference has multiple possible targets.
    ReferenceAmbiguous,
    /// A semantic reference resolved to a node with an incompatible kind.
    ReferenceIncompatibleKind,
    /// A semantic reference points to a child that is not owned by the supplied owner.
    ReferenceInvalidOwner,
    /// A semantic edge was requested more than once where that is considered diagnostic.
    DuplicateSemanticEdgeRequest,
}

impl SemanticDiagnosticCode {
    /// Returns the stable string representation of the diagnostic code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReferenceMalformedFormat => "semantic.reference.malformed_format",
            Self::ReferenceUnsupportedPrefix => "semantic.reference.unsupported_prefix",
            Self::ReferenceUnresolved => "semantic.reference.unresolved",
            Self::ReferenceAmbiguous => "semantic.reference.ambiguous",
            Self::ReferenceIncompatibleKind => "semantic.reference.incompatible_kind",
            Self::ReferenceInvalidOwner => "semantic.reference.invalid_owner",
            Self::DuplicateSemanticEdgeRequest => "semantic.edge.duplicate_request",
        }
    }
}

/// Severity assigned to a semantic diagnostic.
///
/// `Error` marks a semantic problem in input facts. It does not necessarily
/// make graph construction fatal when the build result can return a partial
/// graph alongside ordered diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticDiagnosticSeverity {
    /// Non-fatal diagnostic that should be visible to users.
    Warning,
    /// Semantic error that may still be recoverable by a tolerant pipeline.
    Error,
}

/// Typed semantic diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticDiagnosticKind {
    /// A raw semantic reference does not have the required source format.
    MalformedReferenceFormat,
    /// A raw semantic reference uses a source prefix unsupported by the producer.
    UnsupportedReferencePrefix,
    /// A semantic reference has no resolved target.
    UnresolvedTarget,
    /// A semantic reference has multiple possible targets.
    AmbiguousTarget,
    /// A semantic reference resolved to a node with an incompatible kind.
    IncompatibleTargetKind,
    /// A semantic owner-child relation is invalid.
    InvalidOwnerReference,
    /// A semantic edge request is duplicated where duplication is diagnostic.
    DuplicateSemanticEdgeRequest,
}

/// Structured semantic diagnostic with deterministic ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    code: SemanticDiagnosticCode,
    severity: SemanticDiagnosticSeverity,
    kind: SemanticDiagnosticKind,
    message: String,
    reference: SemanticReference,
    source_node: Option<EntityId>,
    expected_kinds: Vec<NodeKind>,
    actual_kind: Option<NodeKind>,
    candidates: Vec<EntityId>,
    provenance: Vec<Provenance>,
}

impl SemanticDiagnostic {
    /// Creates a structured semantic diagnostic.
    #[must_use]
    pub fn new(
        code: SemanticDiagnosticCode,
        severity: SemanticDiagnosticSeverity,
        kind: SemanticDiagnosticKind,
        message: impl Into<String>,
        reference: SemanticReference,
    ) -> Self {
        Self {
            code,
            severity,
            kind,
            message: message.into(),
            reference,
            source_node: None,
            expected_kinds: Vec::new(),
            actual_kind: None,
            candidates: Vec::new(),
            provenance: Vec::new(),
        }
    }

    /// Creates a diagnostic from a semantic resolution error.
    #[must_use]
    pub fn from_resolution_error(error: ResolutionError) -> Self {
        Self::from_resolution_error_with_reference(error, None)
    }

    /// Creates a diagnostic from a semantic resolution error and optional source reference context.
    #[must_use]
    pub fn from_resolution_error_with_reference(
        error: ResolutionError,
        reference_context: Option<SemanticReference>,
    ) -> Self {
        match error {
            ResolutionError::MissingTarget { reference } => Self::new(
                SemanticDiagnosticCode::ReferenceUnresolved,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::UnresolvedTarget,
                "semantic reference target could not be resolved",
                reference_context.unwrap_or(reference),
            ),
            ResolutionError::AmbiguousTarget {
                reference,
                candidates,
            } => Self::new(
                SemanticDiagnosticCode::ReferenceAmbiguous,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::AmbiguousTarget,
                "semantic reference target is ambiguous",
                reference_context.unwrap_or(reference),
            )
            .with_candidates(candidates),
            ResolutionError::IncompatibleNodeKind {
                id,
                expected,
                actual,
            } => Self::new(
                SemanticDiagnosticCode::ReferenceIncompatibleKind,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::IncompatibleTargetKind,
                "semantic reference target has incompatible kind",
                reference_context
                    .unwrap_or_else(|| SemanticReference::NodeId(id.as_str().to_owned())),
            )
            .with_expected_kinds(expected)
            .with_actual_kind(actual),
            ResolutionError::InvalidOwnerReference { owner, child } => Self::new(
                SemanticDiagnosticCode::ReferenceInvalidOwner,
                SemanticDiagnosticSeverity::Error,
                SemanticDiagnosticKind::InvalidOwnerReference,
                "semantic reference owner relation is invalid",
                reference_context.unwrap_or(SemanticReference::OwnedChild { owner, child }),
            ),
        }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> SemanticDiagnosticCode {
        self.code
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> SemanticDiagnosticSeverity {
        self.severity
    }

    /// Returns the typed diagnostic kind.
    #[must_use]
    pub const fn kind(&self) -> SemanticDiagnosticKind {
        self.kind
    }

    /// Returns a short human-readable diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the semantic reference that produced this diagnostic.
    #[must_use]
    pub const fn reference(&self) -> &SemanticReference {
        &self.reference
    }

    /// Returns the source node related to this diagnostic, when known.
    #[must_use]
    pub const fn source_node(&self) -> Option<&EntityId> {
        self.source_node.as_ref()
    }

    /// Returns the expected target node kinds.
    #[must_use]
    pub fn expected_kinds(&self) -> &[NodeKind] {
        &self.expected_kinds
    }

    /// Returns the actual target node kind, when known.
    #[must_use]
    pub const fn actual_kind(&self) -> Option<NodeKind> {
        self.actual_kind
    }

    /// Returns deterministic candidate target identifiers.
    #[must_use]
    pub fn candidates(&self) -> &[EntityId] {
        &self.candidates
    }

    /// Returns provenance attached to the problematic reference.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Attaches the source node and returns the diagnostic.
    #[must_use]
    pub fn with_source_node(mut self, source_node: EntityId) -> Self {
        self.source_node = Some(source_node);
        self
    }

    /// Attaches expected target kinds and returns the diagnostic.
    #[must_use]
    pub fn with_expected_kinds(mut self, expected_kinds: Vec<NodeKind>) -> Self {
        self.expected_kinds = expected_kinds;
        self.expected_kinds.sort();
        self.expected_kinds.dedup();
        self
    }

    /// Attaches the actual target kind and returns the diagnostic.
    #[must_use]
    pub const fn with_actual_kind(mut self, actual_kind: NodeKind) -> Self {
        self.actual_kind = Some(actual_kind);
        self
    }

    /// Attaches candidate target identifiers and returns the diagnostic.
    #[must_use]
    pub fn with_candidates(mut self, mut candidates: Vec<EntityId>) -> Self {
        candidates.sort();
        candidates.dedup();
        self.candidates = candidates;
        self
    }

    /// Attaches source provenance and returns the diagnostic.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Vec<Provenance>) -> Self {
        self.provenance = provenance;
        self
    }

    fn provenance_key(&self) -> Vec<ProvenanceKey> {
        self.provenance
            .iter()
            .map(|provenance| ProvenanceKey {
                source: provenance.source().cloned(),
                producer: provenance.producer().clone(),
                origin: provenance.origin(),
                confidence: provenance.confidence(),
                resolution: provenance.resolution(),
            })
            .collect()
    }
}

impl PartialOrd for SemanticDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.provenance_key(),
            &self.source_node,
            self.code,
            self.kind,
            &self.reference,
            &self.expected_kinds,
            self.actual_kind,
            &self.candidates,
            &self.message,
        )
            .cmp(&(
                other.provenance_key(),
                &other.source_node,
                other.code,
                other.kind,
                &other.reference,
                &other.expected_kinds,
                other.actual_kind,
                &other.candidates,
                &other.message,
            ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProvenanceKey {
    source: Option<EntityId>,
    producer: ProducerId,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use crate::{
        Confidence, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionError, ResolutionState,
        SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticReference,
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
            ProducerId::new("oneagent.graph.tests"),
            FactOrigin::Resolved,
            Confidence::High,
            ResolutionState::Unresolved,
        )
    }

    #[test]
    fn diagnostic_code_is_stable() {
        assert_eq!(
            SemanticDiagnosticCode::ReferenceMalformedFormat.as_str(),
            "semantic.reference.malformed_format"
        );
        assert_eq!(
            SemanticDiagnosticCode::ReferenceUnsupportedPrefix.as_str(),
            "semantic.reference.unsupported_prefix"
        );
        assert_eq!(
            SemanticDiagnosticCode::ReferenceUnresolved.as_str(),
            "semantic.reference.unresolved"
        );
        assert_eq!(
            SemanticDiagnosticCode::ReferenceAmbiguous.as_str(),
            "semantic.reference.ambiguous"
        );
        assert_eq!(
            SemanticDiagnosticCode::ReferenceIncompatibleKind.as_str(),
            "semantic.reference.incompatible_kind"
        );
        assert_eq!(
            SemanticDiagnosticCode::ReferenceInvalidOwner.as_str(),
            "semantic.reference.invalid_owner"
        );
    }

    #[test]
    fn diagnostic_order_is_deterministic() {
        let first = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "first",
            SemanticReference::Name(name("MissingA")),
        )
        .with_source_node(id("source.a"))
        .with_provenance(vec![provenance("source.a")]);
        let second = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "second",
            SemanticReference::Name(name("MissingB")),
        )
        .with_source_node(id("source.b"))
        .with_expected_kinds(vec![NodeKind::Attribute])
        .with_provenance(vec![provenance("source.b")]);
        let mut diagnostics = vec![second.clone(), first.clone()];

        diagnostics.sort();

        assert_eq!(diagnostics, vec![first, second]);
    }

    #[test]
    fn diagnostic_from_resolution_error_preserves_kind_and_code() {
        let unresolved =
            SemanticDiagnostic::from_resolution_error(ResolutionError::MissingTarget {
                reference: SemanticReference::Name(name("Missing")),
            });
        let ambiguous =
            SemanticDiagnostic::from_resolution_error(ResolutionError::AmbiguousTarget {
                reference: SemanticReference::Name(name("Duplicate")),
                candidates: vec![id("target.b"), id("target.a")],
            });
        let incompatible =
            SemanticDiagnostic::from_resolution_error(ResolutionError::IncompatibleNodeKind {
                id: id("target.catalog"),
                expected: vec![NodeKind::Metadata(
                    oneagent_metadata::MetadataKind::Document,
                )],
                actual: NodeKind::Metadata(oneagent_metadata::MetadataKind::Catalog),
            });
        let invalid_owner =
            SemanticDiagnostic::from_resolution_error(ResolutionError::InvalidOwnerReference {
                owner: id("owner.a"),
                child: id("child.b"),
            });

        assert_eq!(
            unresolved.code(),
            SemanticDiagnosticCode::ReferenceUnresolved
        );
        assert_eq!(unresolved.kind(), SemanticDiagnosticKind::UnresolvedTarget);
        assert_eq!(unresolved.severity(), SemanticDiagnosticSeverity::Error);
        assert_eq!(ambiguous.code(), SemanticDiagnosticCode::ReferenceAmbiguous);
        assert_eq!(ambiguous.candidates(), &[id("target.a"), id("target.b")]);
        assert_eq!(
            incompatible.code(),
            SemanticDiagnosticCode::ReferenceIncompatibleKind
        );
        assert_eq!(
            incompatible.actual_kind(),
            Some(NodeKind::Metadata(oneagent_metadata::MetadataKind::Catalog))
        );
        assert_eq!(
            invalid_owner.code(),
            SemanticDiagnosticCode::ReferenceInvalidOwner
        );
        assert_eq!(
            invalid_owner.kind(),
            SemanticDiagnosticKind::InvalidOwnerReference
        );
    }
}
