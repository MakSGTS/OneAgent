//! Provenance model for graph facts.

use oneagent_common::EntityId;
use std::fmt::{self, Display, Formatter};

/// Stable identifier of a graph fact producer.
///
/// A producer identifier must be deterministic and must not depend on random
/// values, process-specific state, or graph insertion order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProducerId(String);

impl ProducerId {
    /// Creates a producer identifier from its canonical string representation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the canonical string representation of the identifier.
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

impl AsRef<str> for ProducerId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for ProducerId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<String> for ProducerId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ProducerId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Origin of a graph fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactOrigin {
    /// The fact comes from a source declaration.
    Declared,
    /// The fact was parsed from source content.
    Parsed,
    /// The fact was produced by semantic resolution.
    Resolved,
    /// The fact was inferred by analysis.
    Derived,
    /// The fact refers to external knowledge.
    External,
}

/// Confidence level assigned to a graph fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    /// Exact confidence.
    Exact,
    /// High confidence.
    High,
    /// Medium confidence.
    Medium,
    /// Low confidence.
    Low,
    /// Unknown confidence.
    Unknown,
}

/// Resolution state of a graph fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionState {
    /// Resolution does not apply to this fact.
    NotApplicable,
    /// The reference is unresolved.
    Unresolved,
    /// The reference is partially resolved.
    Partial,
    /// The reference has multiple possible targets.
    Ambiguous,
    /// The reference is resolved.
    Resolved,
}

/// Provenance attached to a graph fact.
///
/// Source identity is currently represented by an optional [`EntityId`]
/// because `oneagent-common` does not yet provide source-specific identity or
/// span types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Provenance {
    source: Option<EntityId>,
    producer: ProducerId,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
}

impl Provenance {
    /// Creates provenance for a graph fact.
    #[must_use]
    pub const fn new(
        source: Option<EntityId>,
        producer: ProducerId,
        origin: FactOrigin,
        confidence: Confidence,
        resolution: ResolutionState,
    ) -> Self {
        Self {
            source,
            producer,
            origin,
            confidence,
            resolution,
        }
    }

    /// Returns the optional source entity identifier.
    #[must_use]
    pub const fn source(&self) -> Option<&EntityId> {
        self.source.as_ref()
    }

    /// Returns the fact producer identifier.
    #[must_use]
    pub const fn producer(&self) -> &ProducerId {
        &self.producer
    }

    /// Returns the fact origin.
    #[must_use]
    pub const fn origin(&self) -> FactOrigin {
        self.origin
    }

    /// Returns the confidence level.
    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// Returns the resolution state.
    #[must_use]
    pub const fn resolution(&self) -> ResolutionState {
        self.resolution
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use super::{Confidence, FactOrigin, ProducerId, Provenance, ResolutionState};

    fn source_id(value: &str) -> EntityId {
        EntityId::new(value).expect("source identifier must be valid")
    }

    #[test]
    fn producer_id_preserves_canonical_value() {
        let producer = ProducerId::new("oneagent.graph.metadata-contributor");

        assert_eq!(producer.as_str(), "oneagent.graph.metadata-contributor");
        assert_eq!(producer.to_string(), "oneagent.graph.metadata-contributor");
    }

    #[test]
    fn producer_id_supports_deterministic_ordering() {
        let first = ProducerId::new("oneagent.graph.bsl-declaration-contributor");
        let second = ProducerId::new("oneagent.graph.metadata-contributor");
        let mut producers = vec![second, first];

        producers.sort();

        assert_eq!(
            producers
                .into_iter()
                .map(ProducerId::into_inner)
                .collect::<Vec<_>>(),
            vec![
                "oneagent.graph.bsl-declaration-contributor",
                "oneagent.graph.metadata-contributor"
            ]
        );
    }

    #[test]
    fn provenance_preserves_fact_metadata() {
        let source = source_id("oneagent://source/modules/Sales.bsl");
        let producer = ProducerId::new("oneagent.graph.bsl-reference-contributor");
        let provenance = Provenance::new(
            Some(source.clone()),
            producer.clone(),
            FactOrigin::Resolved,
            Confidence::High,
            ResolutionState::Resolved,
        );

        assert_eq!(provenance.source(), Some(&source));
        assert_eq!(provenance.producer(), &producer);
        assert_eq!(provenance.origin(), FactOrigin::Resolved);
        assert_eq!(provenance.confidence(), Confidence::High);
        assert_eq!(provenance.resolution(), ResolutionState::Resolved);
    }

    #[test]
    fn provenance_allows_absent_source() {
        let provenance = Provenance::new(
            None,
            ProducerId::new("oneagent.graph.derived-dependency-analyzer"),
            FactOrigin::Derived,
            Confidence::Medium,
            ResolutionState::NotApplicable,
        );

        assert_eq!(provenance.source(), None);
        assert_eq!(provenance.origin(), FactOrigin::Derived);
        assert_eq!(provenance.confidence(), Confidence::Medium);
        assert_eq!(provenance.resolution(), ResolutionState::NotApplicable);
    }

    #[test]
    fn provenance_enums_include_required_states() {
        assert_eq!(FactOrigin::Declared, FactOrigin::Declared);
        assert_eq!(FactOrigin::Parsed, FactOrigin::Parsed);
        assert_eq!(FactOrigin::External, FactOrigin::External);

        assert_eq!(Confidence::Exact, Confidence::Exact);
        assert_eq!(Confidence::Low, Confidence::Low);
        assert_eq!(Confidence::Unknown, Confidence::Unknown);

        assert_eq!(ResolutionState::Unresolved, ResolutionState::Unresolved);
        assert_eq!(ResolutionState::Partial, ResolutionState::Partial);
        assert_eq!(ResolutionState::Ambiguous, ResolutionState::Ambiguous);
    }
}
