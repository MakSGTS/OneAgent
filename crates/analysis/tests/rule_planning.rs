use oneagent_analysis::rules::{
    RuleConfiguration, RuleDefinition, RuleEngineErrorKind, RuleId, RulePlan, RulePlanEntry,
    RuleRegistry, RuleSetting, RuleSettingValue,
};

fn id(value: &str) -> RuleId {
    RuleId::new(value).expect("test rule ID must be valid")
}

fn definition(value: &str, dependencies: &[&str]) -> RuleDefinition {
    RuleDefinition::new(id(value), dependencies.iter().copied().map(id))
        .expect("test definition must be valid")
}

fn plan_ids(plan: &RulePlan) -> Vec<&str> {
    plan.entries()
        .iter()
        .map(|entry| entry.rule_id().as_str())
        .collect()
}

#[test]
fn public_plan_uses_smallest_ready_rule_for_independent_branches_and_diamond() {
    let registry = RuleRegistry::new([
        definition("final", &["middle.z", "middle.a"]),
        definition("middle.z", &["root"]),
        definition("independent", &[]),
        definition("root", &[]),
        definition("middle.a", &["root"]),
    ])
    .expect("registry must pass");
    let plan = RulePlan::new(&registry, &RuleConfiguration::default()).expect("plan must pass");

    assert_eq!(
        plan_ids(&plan),
        ["independent", "root", "middle.a", "middle.z", "final"]
    );
}

#[test]
fn public_plan_is_equal_for_reordered_registration_dependency_and_setting_input() {
    let first = RuleRegistry::new([
        definition("c", &["b", "a"]),
        definition("b", &[]),
        definition("a", &[]),
    ])
    .expect("registry must pass");
    let second = RuleRegistry::new([
        definition("a", &[]),
        definition("c", &["a", "b", "a"]),
        definition("b", &[]),
    ])
    .expect("registry must pass");
    let first_configuration = RuleConfiguration::new([
        RuleSetting::new(id("c"), RuleSettingValue::Enabled),
        RuleSetting::new(id("a"), RuleSettingValue::Disabled),
    ])
    .expect("configuration must pass");
    let second_configuration = RuleConfiguration::new([
        RuleSetting::new(id("a"), RuleSettingValue::Disabled),
        RuleSetting::new(id("c"), RuleSettingValue::Enabled),
    ])
    .expect("configuration must pass");

    assert_eq!(
        RulePlan::new(&first, &first_configuration).expect("plan must pass"),
        RulePlan::new(&second, &second_configuration).expect("plan must pass")
    );
}

#[test]
fn public_configuration_is_in_memory_default_enabled_and_keeps_disabled_rules_observable() {
    let registry = RuleRegistry::new([definition("dependent", &["root"]), definition("root", &[])])
        .expect("registry must pass");
    let configuration =
        RuleConfiguration::new([RuleSetting::new(id("root"), RuleSettingValue::Disabled)])
            .expect("configuration must pass");
    let plan = RulePlan::new(&registry, &configuration).expect("plan must pass");

    assert_eq!(
        plan.get(&id("root")).map(RulePlanEntry::setting),
        Some(RuleSettingValue::Disabled)
    );
    assert_eq!(
        plan.get(&id("dependent")).map(RulePlanEntry::setting),
        Some(RuleSettingValue::Enabled)
    );
    assert_eq!(
        plan.get(&id("dependent"))
            .expect("dependent must exist")
            .dependencies(),
        [id("root")]
    );
}

#[test]
fn public_plan_rejects_missing_self_cycle_and_unknown_configuration_atomically() {
    for (definitions, kind) in [
        (
            vec![definition("rule", &["missing"])],
            RuleEngineErrorKind::MissingDependency,
        ),
        (
            vec![definition("rule", &["rule"])],
            RuleEngineErrorKind::SelfDependency,
        ),
        (
            vec![
                definition("a", &["c"]),
                definition("b", &["a"]),
                definition("c", &["b"]),
            ],
            RuleEngineErrorKind::DependencyCycle,
        ),
    ] {
        let registry = RuleRegistry::new(definitions).expect("registry must pass");
        let error = RulePlan::new(&registry, &RuleConfiguration::default())
            .expect_err("invalid graph must fail");
        assert_eq!(error.kind(), kind);
    }

    let registry = RuleRegistry::new([definition("known", &[])]).expect("registry must pass");
    let configuration =
        RuleConfiguration::new([RuleSetting::new(id("unknown"), RuleSettingValue::Enabled)])
            .expect("configuration must pass");
    let error = RulePlan::new(&registry, &configuration).expect_err("unknown setting must fail");
    assert_eq!(error.kind(), RuleEngineErrorKind::UnknownConfiguredRule);
}

#[test]
fn public_configuration_duplicate_and_error_redaction_are_deterministic() {
    let sentinel = "private-rule-setting";
    for settings in [
        [
            RuleSetting::new(id(sentinel), RuleSettingValue::Enabled),
            RuleSetting::new(id(sentinel), RuleSettingValue::Enabled),
        ],
        [
            RuleSetting::new(id(sentinel), RuleSettingValue::Disabled),
            RuleSetting::new(id(sentinel), RuleSettingValue::Enabled),
        ],
    ] {
        let error = RuleConfiguration::new(settings).expect_err("duplicate must fail");
        assert_eq!(error.kind(), RuleEngineErrorKind::DuplicateSetting);
        assert!(!error.to_string().contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
    }
}

#[test]
fn public_empty_and_repeated_planning_are_complete_and_equal() {
    let registry = RuleRegistry::<RuleDefinition>::new([]).expect("empty registry must pass");
    let configuration = RuleConfiguration::default();
    let first = RulePlan::new(&registry, &configuration).expect("empty plan must pass");
    let second = RulePlan::new(&registry, &configuration).expect("empty plan must pass");

    assert!(first.is_empty());
    assert_eq!(first.len(), 0);
    assert_eq!(first, second);
}
