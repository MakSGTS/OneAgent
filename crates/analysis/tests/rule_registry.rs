use oneagent_analysis::rules::{
    RuleDefinition, RuleDiagnosticCode, RuleEngineErrorKind, RuleId, RuleRegistration, RuleRegistry,
};
use std::sync::Arc;

fn id(value: &str) -> RuleId {
    RuleId::new(value).expect("test rule ID must be valid")
}

fn definition(value: &str, dependencies: &[&str]) -> RuleDefinition {
    RuleDefinition::new(id(value), dependencies.iter().copied().map(id))
        .expect("test definition must be valid")
}

#[derive(Debug)]
struct SharedRegistration {
    definition: RuleDefinition,
    behavior_marker: &'static str,
}

impl RuleRegistration for SharedRegistration {
    fn definition(&self) -> &RuleDefinition {
        &self.definition
    }
}

#[test]
fn public_identity_and_definition_contract_is_source_independent() {
    let rule_id = RuleId::new("semantic.unresolved-call").expect("rule ID must pass");
    let code = RuleDiagnosticCode::new("unresolved_call").expect("code must pass");
    let definition =
        RuleDefinition::new(rule_id.clone(), [id("semantic.base")]).expect("definition must pass");

    assert_eq!(rule_id.as_str(), "semantic.unresolved-call");
    assert_eq!(code.as_str(), "unresolved_call");
    assert_eq!(definition.id(), &rule_id);
    assert_eq!(definition.dependencies(), [id("semantic.base")]);
}

#[test]
fn public_registry_enumeration_is_stable_across_input_reordering_and_repetition() {
    let expected = RuleRegistry::new([
        definition("rule.c", &["rule.a"]),
        definition("rule.a", &[]),
        definition("rule.b", &[]),
    ])
    .expect("registry must pass");

    for registrations in [
        vec![
            definition("rule.a", &[]),
            definition("rule.b", &[]),
            definition("rule.c", &["rule.a"]),
        ],
        vec![
            definition("rule.b", &[]),
            definition("rule.c", &["rule.a"]),
            definition("rule.a", &[]),
        ],
    ] {
        let actual = RuleRegistry::new(registrations).expect("registry must pass");
        assert_eq!(actual, expected);
        assert_eq!(
            actual
                .registrations()
                .iter()
                .map(|entry| entry.id().as_str())
                .collect::<Vec<_>>(),
            ["rule.a", "rule.b", "rule.c"]
        );
    }
}

#[test]
fn public_registry_owns_shared_registration_objects_without_comparing_behavior() {
    let first = Arc::new(SharedRegistration {
        definition: definition("rule", &[]),
        behavior_marker: "first",
    });
    let second = Arc::new(SharedRegistration {
        definition: definition("rule", &[]),
        behavior_marker: "second",
    });

    let first_registry = RuleRegistry::new([Arc::clone(&first)]).expect("registry must pass");
    let second_registry = RuleRegistry::new([Arc::clone(&second)]).expect("registry must pass");
    assert_eq!(first_registry, second_registry);
    assert_eq!(
        first_registry
            .get(&id("rule"))
            .expect("registration must exist")
            .behavior_marker,
        "first"
    );

    let error = RuleRegistry::new([first, second]).expect_err("duplicate must fail");
    assert_eq!(error.kind(), RuleEngineErrorKind::DuplicateRule);
}

#[test]
fn public_registry_conflict_classification_is_independent_from_input_order() {
    for registrations in [
        vec![definition("rule", &["a"]), definition("rule", &["b"])],
        vec![definition("rule", &["b"]), definition("rule", &["a"])],
    ] {
        let error = RuleRegistry::new(registrations).expect_err("conflict must fail");
        assert_eq!(error.kind(), RuleEngineErrorKind::ConflictingRule);
        assert_eq!(error.actual(), None);
        assert_eq!(error.maximum(), None);
    }
}

#[test]
fn public_errors_never_echo_rejected_identity_or_registered_behavior() {
    let identity_sentinel = "private-rule!";
    let behavior_sentinel = "private-behavior-marker";
    let invalid = RuleId::new(identity_sentinel).expect_err("identity must fail");
    let registration = Arc::new(SharedRegistration {
        definition: definition("rule", &[]),
        behavior_marker: behavior_sentinel,
    });
    let duplicate = RuleRegistry::new([Arc::clone(&registration), registration])
        .expect_err("duplicate must fail");

    for error in [invalid, duplicate] {
        let display = error.to_string();
        let debug = format!("{error:?}");
        assert!(!display.contains(identity_sentinel));
        assert!(!debug.contains(identity_sentinel));
        assert!(!display.contains(behavior_sentinel));
        assert!(!debug.contains(behavior_sentinel));
    }
}
