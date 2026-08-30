//! Source-independent rule identity, definitions, and registration.

use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

/// Maximum accepted rule identifier size in ASCII bytes.
pub const MAX_RULE_ID_BYTES: usize = 128;
/// Maximum accepted rule diagnostic code size in ASCII bytes.
pub const MAX_RULE_DIAGNOSTIC_CODE_BYTES: usize = 128;
/// Maximum number of input registrations accepted by one registry.
pub const MAX_RULE_REGISTRATIONS: usize = 4_096;
/// Maximum number of unique dependencies accepted by one rule definition.
pub const MAX_RULE_DEPENDENCIES: usize = 256;

/// Closed Rules Engine construction failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleEngineErrorKind {
    /// A rule identifier violated the accepted grammar or byte bound.
    InvalidRuleId,
    /// A rule diagnostic code violated the accepted grammar or byte bound.
    InvalidRuleDiagnosticCode,
    /// A rule definition contains too many unique dependencies.
    TooManyRuleDependencies,
    /// A registry contains too many input registrations.
    TooManyRuleRegistrations,
    /// Equal definitions were registered under one rule identifier.
    DuplicateRule,
    /// Different definitions were registered under one rule identifier.
    ConflictingRule,
}

/// Bounded redacted Rules Engine construction failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleEngineError {
    kind: RuleEngineErrorKind,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl RuleEngineError {
    const fn new(kind: RuleEngineErrorKind) -> Self {
        Self {
            kind,
            actual: None,
            maximum: None,
        }
    }

    const fn bounded(kind: RuleEngineErrorKind, actual: usize, maximum: usize) -> Self {
        Self {
            kind,
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> RuleEngineErrorKind {
        self.kind
    }

    /// Returns the observed rejected count for a bounded-count failure.
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

impl Display for RuleEngineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.actual, self.maximum) {
            (Some(actual), Some(maximum)) => write!(
                formatter,
                "rules engine rejected a bounded count: kind={:?}, actual={actual}, maximum={maximum}",
                self.kind
            ),
            _ => write!(
                formatter,
                "rules engine rejected registration data: kind={:?}",
                self.kind
            ),
        }
    }
}

impl std::error::Error for RuleEngineError {}

fn validate_identifier(
    value: &str,
    maximum: usize,
    kind: RuleEngineErrorKind,
) -> Result<(), RuleEngineError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > maximum || !value.is_ascii() {
        return Err(RuleEngineError::new(kind));
    }

    let is_component = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    let is_separator = |byte: u8| matches!(byte, b'.' | b'-' | b'_');

    if !is_component(bytes[0]) || !is_component(bytes[bytes.len() - 1]) {
        return Err(RuleEngineError::new(kind));
    }

    let mut previous_was_separator = false;
    for &byte in bytes {
        let separator = is_separator(byte);
        if (!is_component(byte) && !separator) || (separator && previous_was_separator) {
            return Err(RuleEngineError::new(kind));
        }
        previous_was_separator = separator;
    }

    Ok(())
}

/// Stable globally scoped rule identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(Box<str>);

impl RuleId {
    /// Creates a rule identifier without normalization.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`RuleEngineError`] when the value violates the
    /// accepted grammar or byte bound.
    pub fn new(value: impl Into<String>) -> Result<Self, RuleEngineError> {
        let value = value.into();
        validate_identifier(
            &value,
            MAX_RULE_ID_BYTES,
            RuleEngineErrorKind::InvalidRuleId,
        )?;
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact accepted rule identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RuleId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Stable rule-local diagnostic code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleDiagnosticCode(Box<str>);

impl RuleDiagnosticCode {
    /// Creates a rule diagnostic code without normalization.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`RuleEngineError`] when the value violates the
    /// accepted grammar or byte bound.
    pub fn new(value: impl Into<String>) -> Result<Self, RuleEngineError> {
        let value = value.into();
        validate_identifier(
            &value,
            MAX_RULE_DIAGNOSTIC_CODE_BYTES,
            RuleEngineErrorKind::InvalidRuleDiagnosticCode,
        )?;
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact accepted diagnostic code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RuleDiagnosticCode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Immutable rule metadata used for registration and dependency planning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleDefinition {
    id: RuleId,
    dependencies: Vec<RuleId>,
}

impl RuleDefinition {
    /// Creates a canonical rule definition.
    ///
    /// Dependencies are sorted and exact repeats collapse. A self-dependency
    /// is retained for complete-registry validation during rule planning.
    ///
    /// # Errors
    ///
    /// Returns a bounded [`RuleEngineError`] when more than
    /// [`MAX_RULE_DEPENDENCIES`] unique dependencies are supplied.
    pub fn new(
        id: RuleId,
        dependencies: impl IntoIterator<Item = RuleId>,
    ) -> Result<Self, RuleEngineError> {
        let mut canonical = BTreeSet::new();
        for dependency in dependencies {
            canonical.insert(dependency);
            if canonical.len() > MAX_RULE_DEPENDENCIES {
                return Err(RuleEngineError::bounded(
                    RuleEngineErrorKind::TooManyRuleDependencies,
                    MAX_RULE_DEPENDENCIES + 1,
                    MAX_RULE_DEPENDENCIES,
                ));
            }
        }
        Ok(Self {
            id,
            dependencies: canonical.into_iter().collect(),
        })
    }

    /// Returns the globally scoped rule identifier.
    #[must_use]
    pub const fn id(&self) -> &RuleId {
        &self.id
    }

    /// Returns required rule identifiers in canonical ascending order.
    #[must_use]
    pub fn dependencies(&self) -> &[RuleId] {
        &self.dependencies
    }
}

/// Source-independent registration metadata exposed by a registry entry.
pub trait RuleRegistration: Send + Sync {
    /// Returns the immutable definition that identifies this registration.
    fn definition(&self) -> &RuleDefinition;
}

impl RuleRegistration for RuleDefinition {
    fn definition(&self) -> &RuleDefinition {
        self
    }
}

impl<T> RuleRegistration for Arc<T>
where
    T: RuleRegistration + ?Sized,
{
    fn definition(&self) -> &RuleDefinition {
        self.as_ref().definition()
    }
}

/// Immutable deterministic registry of source-independent rule registrations.
pub struct RuleRegistry<R> {
    registrations: Vec<R>,
}

impl<R> RuleRegistry<R>
where
    R: RuleRegistration,
{
    /// Constructs and validates a complete immutable registry.
    ///
    /// Accepted registrations are exposed in ascending complete [`RuleId`]
    /// order independently from input order.
    ///
    /// # Errors
    ///
    /// Returns a closed error for an over-limit registry, an equal duplicate,
    /// or a conflicting definition with the same identifier. No partial
    /// registry is returned.
    pub fn new(registrations: impl IntoIterator<Item = R>) -> Result<Self, RuleEngineError> {
        let mut canonical = Vec::new();
        for registration in registrations {
            if canonical.len() == MAX_RULE_REGISTRATIONS {
                return Err(RuleEngineError::bounded(
                    RuleEngineErrorKind::TooManyRuleRegistrations,
                    MAX_RULE_REGISTRATIONS + 1,
                    MAX_RULE_REGISTRATIONS,
                ));
            }
            canonical.push(registration);
        }

        canonical.sort_by(|left, right| left.definition().cmp(right.definition()));
        let mut group_start = 0;
        while group_start < canonical.len() {
            let group_id = canonical[group_start].definition().id();
            let mut group_end = group_start + 1;
            while group_end < canonical.len() && canonical[group_end].definition().id() == group_id
            {
                group_end += 1;
            }
            if group_end - group_start > 1 {
                let first = canonical[group_start].definition();
                let last = canonical[group_end - 1].definition();
                let kind = if first == last {
                    RuleEngineErrorKind::DuplicateRule
                } else {
                    RuleEngineErrorKind::ConflictingRule
                };
                return Err(RuleEngineError::new(kind));
            }
            group_start = group_end;
        }

        Ok(Self {
            registrations: canonical,
        })
    }

    /// Returns whether the registry contains no registrations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// Returns the number of registrations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// Returns all registrations in ascending complete rule-ID order.
    #[must_use]
    pub fn registrations(&self) -> &[R] {
        &self.registrations
    }

    /// Returns the registration for an exact rule identifier.
    #[must_use]
    pub fn get(&self, id: &RuleId) -> Option<&R> {
        self.registrations
            .binary_search_by(|registration| registration.definition().id().cmp(id))
            .ok()
            .map(|index| &self.registrations[index])
    }
}

impl<R> Debug for RuleRegistry<R>
where
    R: RuleRegistration,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuleRegistry")
            .field(
                "definitions",
                &self
                    .registrations
                    .iter()
                    .map(RuleRegistration::definition)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<R, S> PartialEq<RuleRegistry<S>> for RuleRegistry<R>
where
    R: RuleRegistration,
    S: RuleRegistration,
{
    fn eq(&self, other: &RuleRegistry<S>) -> bool {
        self.registrations
            .iter()
            .map(RuleRegistration::definition)
            .eq(other.registrations.iter().map(RuleRegistration::definition))
    }
}

impl<R> Eq for RuleRegistry<R> where R: RuleRegistration {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_RULE_DEPENDENCIES, MAX_RULE_DIAGNOSTIC_CODE_BYTES, MAX_RULE_ID_BYTES,
        MAX_RULE_REGISTRATIONS, RuleDefinition, RuleDiagnosticCode, RuleEngineErrorKind, RuleId,
        RuleRegistration, RuleRegistry,
    };

    fn id(value: impl Into<String>) -> RuleId {
        RuleId::new(value).expect("test rule ID must be valid")
    }

    fn definition(value: &str, dependencies: &[&str]) -> RuleDefinition {
        RuleDefinition::new(id(value), dependencies.iter().copied().map(id))
            .expect("test definition must be valid")
    }

    #[test]
    fn identifiers_accept_exact_grammar_and_boundaries() {
        for value in ["a", "a0", "a.b", "a-b", "a_b", "0.a-b_c9"] {
            let rule_id = RuleId::new(value).expect("rule ID must pass");
            let code = RuleDiagnosticCode::new(value).expect("code must pass");
            assert_eq!(rule_id.as_str(), value);
            assert_eq!(code.as_str(), value);
        }
        assert!(RuleId::new("a".repeat(MAX_RULE_ID_BYTES)).is_ok());
        assert!(RuleDiagnosticCode::new("a".repeat(MAX_RULE_DIAGNOSTIC_CODE_BYTES)).is_ok());
    }

    #[test]
    fn identifiers_reject_every_invalid_grammar_class_and_one_over_bound() {
        for value in [
            "",
            ".a",
            "a.",
            "-a",
            "a-",
            "_a",
            "a_",
            "a..b",
            "a-_b",
            "A",
            "a B",
            "a/b",
            "a\n",
            "правило",
        ] {
            assert_eq!(
                RuleId::new(value).expect_err("rule ID must fail").kind(),
                RuleEngineErrorKind::InvalidRuleId
            );
            assert_eq!(
                RuleDiagnosticCode::new(value)
                    .expect_err("code must fail")
                    .kind(),
                RuleEngineErrorKind::InvalidRuleDiagnosticCode
            );
        }
        assert!(RuleId::new("a".repeat(MAX_RULE_ID_BYTES + 1)).is_err());
        assert!(RuleDiagnosticCode::new("a".repeat(MAX_RULE_DIAGNOSTIC_CODE_BYTES + 1)).is_err());
    }

    #[test]
    fn identifiers_have_exact_equality_total_order_and_display() {
        let first = id("a.rule");
        let equal = id("a.rule");
        let second = id("b-rule");
        assert_eq!(first, equal);
        assert!(first < second);
        assert_eq!(first.to_string(), "a.rule");

        let code = RuleDiagnosticCode::new("finding_code").expect("code must pass");
        assert_eq!(code.to_string(), "finding_code");
    }

    #[test]
    fn definition_canonicalizes_dependencies_and_retains_self_dependency() {
        let definition = definition("rule", &["z", "a", "z", "rule"]);
        let dependencies = definition
            .dependencies()
            .iter()
            .map(RuleId::as_str)
            .collect::<Vec<_>>();
        assert_eq!(dependencies, ["a", "rule", "z"]);
        assert_eq!(definition.definition(), &definition);
    }

    #[test]
    fn definition_accepts_exact_unique_dependency_bound_and_rejects_one_over() {
        let exact = (0..MAX_RULE_DEPENDENCIES)
            .map(|index| id(format!("dependency{index:03}")))
            .collect::<Vec<_>>();
        assert!(RuleDefinition::new(id("rule"), exact).is_ok());

        let over = (0..=MAX_RULE_DEPENDENCIES)
            .map(|index| id(format!("dependency{index:03}")))
            .collect::<Vec<_>>();
        let error = RuleDefinition::new(id("rule"), over).expect_err("definition must fail");
        assert_eq!(error.kind(), RuleEngineErrorKind::TooManyRuleDependencies);
        assert_eq!(error.actual(), Some(MAX_RULE_DEPENDENCIES + 1));
        assert_eq!(error.maximum(), Some(MAX_RULE_DEPENDENCIES));
    }

    #[test]
    fn registry_is_empty_single_or_canonically_ordered() {
        let empty = RuleRegistry::<RuleDefinition>::new([]).expect("empty registry must pass");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let single =
            RuleRegistry::new([definition("single", &[])]).expect("single registry must pass");
        assert_eq!(single.get(&id("single")), single.registrations().first());

        let multiple = RuleRegistry::new([
            definition("z", &[]),
            definition("a", &[]),
            definition("m", &[]),
        ])
        .expect("registry must pass");
        let ids = multiple
            .registrations()
            .iter()
            .map(|entry| entry.id().as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["a", "m", "z"]);
        assert!(multiple.get(&id("missing")).is_none());
    }

    #[test]
    fn registry_rejects_duplicate_and_conflicting_definitions() {
        let duplicate = RuleRegistry::new([definition("rule", &["a"]), definition("rule", &["a"])])
            .expect_err("duplicate must fail");
        assert_eq!(duplicate.kind(), RuleEngineErrorKind::DuplicateRule);

        let conflict = RuleRegistry::new([definition("rule", &["a"]), definition("rule", &["b"])])
            .expect_err("conflict must fail");
        assert_eq!(conflict.kind(), RuleEngineErrorKind::ConflictingRule);
    }

    #[test]
    fn registry_classifies_mixed_same_id_groups_as_conflicts() {
        for registrations in [
            [
                definition("rule", &["a"]),
                definition("rule", &["a"]),
                definition("rule", &["b"]),
            ],
            [
                definition("rule", &["b"]),
                definition("rule", &["a"]),
                definition("rule", &["a"]),
            ],
        ] {
            let error = RuleRegistry::new(registrations).expect_err("conflict must fail");
            assert_eq!(error.kind(), RuleEngineErrorKind::ConflictingRule);
        }
    }

    #[test]
    fn registry_accepts_exact_registration_bound_and_rejects_one_over() {
        let exact = (0..MAX_RULE_REGISTRATIONS)
            .map(|index| definition(&format!("rule{index:04}"), &[]))
            .collect::<Vec<_>>();
        assert_eq!(
            RuleRegistry::new(exact)
                .expect("exact registry must pass")
                .len(),
            MAX_RULE_REGISTRATIONS
        );

        let over =
            (0..=MAX_RULE_REGISTRATIONS).map(|index| definition(&format!("rule{index:04}"), &[]));
        let error = RuleRegistry::new(over).expect_err("over-limit registry must fail");
        assert_eq!(error.kind(), RuleEngineErrorKind::TooManyRuleRegistrations);
        assert_eq!(error.actual(), Some(MAX_RULE_REGISTRATIONS + 1));
        assert_eq!(error.maximum(), Some(MAX_RULE_REGISTRATIONS));
    }

    #[test]
    fn registry_equality_and_debug_compare_only_canonical_definitions() {
        let first = RuleRegistry::new([definition("b", &["a"]), definition("a", &[])])
            .expect("registry must pass");
        let second = RuleRegistry::new([definition("a", &[]), definition("b", &["a"])])
            .expect("registry must pass");
        assert_eq!(first, second);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }

    #[test]
    fn all_error_formats_are_redacted_and_bounded() {
        let sentinel = "sensitive-rejected-rule";
        let invalid = RuleId::new(format!("{sentinel}!")).expect_err("invalid sentinel must fail");
        let duplicate = RuleRegistry::new([definition("rule", &[]), definition("rule", &[])])
            .expect_err("duplicate must fail");
        for error in [invalid, duplicate] {
            assert!(!format!("{error}").contains(sentinel));
            assert!(!format!("{error:?}").contains(sentinel));
        }
    }
}
