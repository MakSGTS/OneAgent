use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use oneagent_analysis::diagnostics::{
    DiagnosticCategory, DiagnosticEngine, DiagnosticPolicy, DiagnosticSeverity,
};
use oneagent_analysis::rules::{
    MAX_RULE_DIAGNOSTICS_PER_RULE, NeverCancelled, Rule, RuleCancellationSignal, RuleConfiguration,
    RuleContext, RuleDefinition, RuleDiagnostic, RuleDiagnosticCode, RuleEngine,
    RuleEngineErrorKind, RuleEvaluation, RuleExecutionReport, RuleFailureCode, RuleId, RulePlan,
    RuleRegistration, RuleRegistry, RuleSetting, RuleSettingValue, RuleStatus,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, FactOrigin, GraphNode, NodeKind, ProducerId, Provenance, ResolutionState,
    SemanticGraph,
};

type Evaluator =
    dyn Fn(&RuleContext<'_>, &dyn RuleCancellationSignal) -> RuleEvaluation + Send + Sync;

struct ConformanceRule {
    definition: RuleDefinition,
    evaluator: Arc<Evaluator>,
}

impl RuleRegistration for ConformanceRule {
    fn definition(&self) -> &RuleDefinition {
        &self.definition
    }
}

impl Rule for ConformanceRule {
    fn evaluate(
        &self,
        context: &RuleContext<'_>,
        cancellation: &dyn RuleCancellationSignal,
    ) -> RuleEvaluation {
        (self.evaluator)(context, cancellation)
    }
}

#[derive(Clone)]
struct AtomicCancellation {
    cancelled: Arc<AtomicBool>,
}

impl RuleCancellationSignal for AtomicCancellation {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

fn rule_id(value: &str) -> RuleId {
    RuleId::new(value).expect("test rule ID must be valid")
}

fn entity_id(value: &str) -> EntityId {
    EntityId::new(value).expect("test entity ID must be valid")
}

fn rule<F>(value: &str, dependencies: &[&str], evaluator: F) -> Arc<dyn Rule>
where
    F: Fn(&RuleContext<'_>, &dyn RuleCancellationSignal) -> RuleEvaluation + Send + Sync + 'static,
{
    Arc::new(ConformanceRule {
        definition: RuleDefinition::new(rule_id(value), dependencies.iter().copied().map(rule_id))
            .expect("test definition must be valid"),
        evaluator: Arc::new(evaluator),
    })
}

fn completed_rule(value: &str, dependencies: &[&str]) -> Arc<dyn Rule> {
    rule(value, dependencies, |_, _| {
        RuleEvaluation::Completed(Vec::new())
    })
}

fn diagnostic(rule: &str, code: &str, message: &str, anchors: &[&str]) -> RuleDiagnostic {
    RuleDiagnostic::new(
        rule_id(rule),
        RuleDiagnosticCode::new(code).expect("test code must be valid"),
        DiagnosticSeverity::Warning,
        DiagnosticCategory::Semantic,
        message,
        anchors.iter().copied().map(entity_id),
    )
}

fn execute(
    registry: &RuleRegistry<Arc<dyn Rule>>,
    configuration: &RuleConfiguration,
    graph: &SemanticGraph,
    cancellation: &dyn RuleCancellationSignal,
) -> Result<RuleExecutionReport, oneagent_analysis::rules::RuleEngineError> {
    let validation = graph.validate();
    let base = DiagnosticEngine
        .build(&[], &validation, &DiagnosticPolicy::default())
        .expect("base diagnostics must be valid");
    let plan = RulePlan::new(registry, configuration).expect("test plan must be valid");
    let context = RuleContext::new(graph, &validation, &base);
    RuleEngine.execute(registry, &plan, configuration, &context, cancellation)
}

fn status_map(report: &RuleExecutionReport) -> Vec<(&str, RuleStatus)> {
    report
        .results()
        .iter()
        .map(|result| (result.rule_id().as_str(), result.status()))
        .collect()
}

#[test]
fn public_empty_execution_is_complete_and_reconciled() {
    let registry = RuleRegistry::<Arc<dyn Rule>>::new([]).expect("empty registry must pass");
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect("empty execution must pass");

    assert!(report.results().is_empty());
    assert!(report.diagnostics().is_empty());
    assert_eq!(report.summary().total(), 0);
    for status in [
        RuleStatus::Disabled,
        RuleStatus::NotApplicable,
        RuleStatus::Completed,
        RuleStatus::Blocked,
        RuleStatus::Failed,
        RuleStatus::Cancelled,
    ] {
        assert_eq!(report.summary().status_count(status), 0);
    }
}

#[test]
fn public_execution_distinguishes_completed_disabled_not_applicable_failed_and_blocked() {
    let registry = RuleRegistry::new([
        completed_rule("a.completed", &[]),
        rule("b.not-applicable", &["a.completed"], |_, _| {
            RuleEvaluation::NotApplicable
        }),
        rule("c.failed", &[], |_, _| {
            RuleEvaluation::Failed(RuleFailureCode::new("rule_failure").expect("code must pass"))
        }),
        completed_rule("d.blocked", &["b.not-applicable"]),
        completed_rule("e.disabled", &[]),
        completed_rule("f.independent", &[]),
    ])
    .expect("registry must pass");
    let configuration = RuleConfiguration::new([RuleSetting::new(
        rule_id("e.disabled"),
        RuleSettingValue::Disabled,
    )])
    .expect("configuration must pass");
    let report = execute(
        &registry,
        &configuration,
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect("execution must pass");

    assert_eq!(
        status_map(&report),
        [
            ("a.completed", RuleStatus::Completed),
            ("b.not-applicable", RuleStatus::NotApplicable),
            ("c.failed", RuleStatus::Failed),
            ("d.blocked", RuleStatus::Blocked),
            ("e.disabled", RuleStatus::Disabled),
            ("f.independent", RuleStatus::Completed),
        ]
    );
    assert_eq!(report.summary().total(), 6);
    assert_eq!(report.summary().status_count(RuleStatus::Completed), 2);
    assert_eq!(
        report.results()[2]
            .failure_code()
            .map(RuleFailureCode::as_str),
        Some("rule_failure")
    );
}

#[test]
fn public_failure_blocks_dependents_and_preserves_independent_diamond_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let independent_calls = Arc::clone(&calls);
    let registry = RuleRegistry::new([
        completed_rule("a.root", &[]),
        rule("b.failure", &["a.root"], |_, _| {
            RuleEvaluation::Failed(RuleFailureCode::new("failed").expect("code must pass"))
        }),
        rule("c.independent", &["a.root"], move |_, _| {
            independent_calls.fetch_add(1, Ordering::SeqCst);
            RuleEvaluation::Completed(Vec::new())
        }),
        completed_rule("d.diamond", &["b.failure", "c.independent"]),
    ])
    .expect("registry must pass");
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect("execution must pass");

    assert_eq!(
        status_map(&report),
        [
            ("a.root", RuleStatus::Completed),
            ("b.failure", RuleStatus::Failed),
            ("c.independent", RuleStatus::Completed),
            ("d.diamond", RuleStatus::Blocked),
        ]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn public_context_exposes_only_canonical_borrowed_evidence() {
    let observed = Arc::new(AtomicUsize::new(0));
    let rule_observed = Arc::clone(&observed);
    let registry = RuleRegistry::new([rule("context", &[], move |context, cancellation| {
        assert_eq!(context.graph().node_count(), 1);
        assert!(!context.validation().issues().is_empty());
        assert_eq!(context.base_diagnostics().summary().total(), 1);
        assert!(!cancellation.is_cancelled());
        rule_observed.fetch_add(1, Ordering::SeqCst);
        RuleEvaluation::Completed(Vec::new())
    })])
    .expect("registry must pass");
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new(
        entity_id("node"),
        EntityName::new("Node").expect("name must pass"),
        NodeKind::Unknown,
    ));

    execute(
        &registry,
        &RuleConfiguration::default(),
        &graph,
        &NeverCancelled,
    )
    .expect("execution must pass");
    assert_eq!(observed.load(Ordering::SeqCst), 1);
}

#[test]
fn public_preexisting_cancellation_marks_every_rule_and_executes_nothing() {
    let calls = Arc::new(AtomicUsize::new(0));
    let first_calls = Arc::clone(&calls);
    let second_calls = Arc::clone(&calls);
    let registry = RuleRegistry::new([
        rule("a", &[], move |_, _| {
            first_calls.fetch_add(1, Ordering::SeqCst);
            RuleEvaluation::Completed(Vec::new())
        }),
        rule("b", &["a"], move |_, _| {
            second_calls.fetch_add(1, Ordering::SeqCst);
            RuleEvaluation::Completed(Vec::new())
        }),
    ])
    .expect("registry must pass");
    let cancellation = AtomicCancellation {
        cancelled: Arc::new(AtomicBool::new(true)),
    };
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &cancellation,
    )
    .expect("cancelled execution must be complete");

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(
        report
            .results()
            .iter()
            .all(|result| result.status() == RuleStatus::Cancelled)
    );
    assert_eq!(report.summary().status_count(RuleStatus::Cancelled), 2);
}

#[test]
fn public_post_evaluation_cancellation_discards_output_and_cancels_remaining_rules() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let rule_cancelled = Arc::clone(&cancelled);
    let later_calls = Arc::new(AtomicUsize::new(0));
    let rule_later_calls = Arc::clone(&later_calls);
    let registry = RuleRegistry::new([
        rule("a", &[], move |_, cancellation| {
            assert!(!cancellation.is_cancelled());
            rule_cancelled.store(true, Ordering::SeqCst);
            RuleEvaluation::Completed(vec![diagnostic("a", "finding", "discarded", &[])])
        }),
        rule("b", &[], move |_, _| {
            rule_later_calls.fetch_add(1, Ordering::SeqCst);
            RuleEvaluation::Completed(Vec::new())
        }),
    ])
    .expect("registry must pass");
    let cancellation = AtomicCancellation { cancelled };
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &cancellation,
    )
    .expect("cancelled execution must be complete");

    assert!(report.diagnostics().is_empty());
    assert_eq!(later_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        status_map(&report),
        [("a", RuleStatus::Cancelled), ("b", RuleStatus::Cancelled)]
    );
}

#[test]
fn public_diagnostics_are_canonical_deduplicated_and_provenance_backed() {
    let finding = diagnostic("rule", "finding", "message", &["node", "node"]);
    let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
        RuleEvaluation::Completed(vec![finding.clone(), finding.clone()])
    })])
    .expect("registry must pass");
    let provenance = Provenance::new(
        None,
        ProducerId::new("test.rule"),
        FactOrigin::Derived,
        Confidence::Exact,
        ResolutionState::Resolved,
    );
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new_with_provenance(
        entity_id("node"),
        EntityName::new("Node").expect("name must pass"),
        NodeKind::Unknown,
        vec![provenance],
    ));
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &graph,
        &NeverCancelled,
    )
    .expect("execution must pass");

    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.results()[0].diagnostic_count(), 1);
    assert_eq!(report.diagnostics()[0].node_anchors(), [entity_id("node")]);
    assert_eq!(report.diagnostics()[0].observed_provenance_count(), 1);
}

#[test]
fn public_conflicting_or_invalid_rule_output_fails_only_that_rule() {
    let cases = [
        vec![
            diagnostic("rule", "code", "first", &[]),
            diagnostic("rule", "code", "second", &[]),
        ],
        vec![diagnostic("other", "code", "wrong producer", &[])],
        vec![diagnostic("rule", "code", "missing anchor", &["missing"])],
        vec![diagnostic("rule", "code", &"x".repeat(4_097), &[])],
    ];

    for candidates in cases {
        let registry = RuleRegistry::new([
            rule("rule", &[], move |_, _| {
                RuleEvaluation::Completed(candidates.clone())
            }),
            completed_rule("z.independent", &[]),
        ])
        .expect("registry must pass");
        let report = execute(
            &registry,
            &RuleConfiguration::default(),
            &SemanticGraph::new(),
            &NeverCancelled,
        )
        .expect("invalid rule output must be a complete failed result");
        assert_eq!(report.results()[0].status(), RuleStatus::Failed);
        assert_eq!(
            report.results()[0]
                .failure_code()
                .map(RuleFailureCode::as_str),
            Some("invalid_rule_output")
        );
        assert_eq!(report.results()[1].status(), RuleStatus::Completed);
        assert!(report.diagnostics().is_empty());
    }
}

#[test]
fn public_node_anchor_bound_accepts_exact_and_fails_one_over() {
    let exact_anchors = (0..256)
        .map(|index| format!("node{index:03}"))
        .collect::<Vec<_>>();
    let exact_diagnostic = RuleDiagnostic::new(
        rule_id("rule"),
        RuleDiagnosticCode::new("finding").expect("code must pass"),
        DiagnosticSeverity::Warning,
        DiagnosticCategory::Semantic,
        "exact anchors",
        exact_anchors.iter().map(|value| entity_id(value)),
    );
    let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
        RuleEvaluation::Completed(vec![exact_diagnostic.clone()])
    })])
    .expect("registry must pass");
    let mut graph = SemanticGraph::new();
    for value in &exact_anchors {
        graph.insert_node(GraphNode::new(
            entity_id(value),
            EntityName::new(value).expect("name must pass"),
            NodeKind::Unknown,
        ));
    }
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &graph,
        &NeverCancelled,
    )
    .expect("exact anchor bound must pass");
    assert_eq!(report.results()[0].status(), RuleStatus::Completed);
    assert_eq!(report.diagnostics()[0].node_anchors().len(), 256);

    let over = RuleDiagnostic::new(
        rule_id("rule"),
        RuleDiagnosticCode::new("finding").expect("code must pass"),
        DiagnosticSeverity::Warning,
        DiagnosticCategory::Semantic,
        "over anchors",
        (0..257).map(|index| entity_id(&format!("node{index:03}"))),
    );
    let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
        RuleEvaluation::Completed(vec![over.clone()])
    })])
    .expect("registry must pass");
    let report = execute(
        &registry,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect("over anchor output must produce a complete failed result");
    assert_eq!(report.results()[0].status(), RuleStatus::Failed);
    assert!(report.diagnostics().is_empty());
}

fn provenance() -> Provenance {
    Provenance::new(
        None,
        ProducerId::new("test.rule"),
        FactOrigin::Derived,
        Confidence::Exact,
        ResolutionState::Resolved,
    )
}

#[test]
fn public_observed_provenance_bound_accepts_exact_and_fails_one_over() {
    for (count, expected) in [(256, RuleStatus::Completed), (257, RuleStatus::Failed)] {
        let candidate = diagnostic("rule", "finding", "provenance", &["node"]);
        let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
            RuleEvaluation::Completed(vec![candidate.clone()])
        })])
        .expect("registry must pass");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new_with_provenance(
            entity_id("node"),
            EntityName::new("Node").expect("name must pass"),
            NodeKind::Unknown,
            vec![provenance(); count],
        ));
        let report = execute(
            &registry,
            &RuleConfiguration::default(),
            &graph,
            &NeverCancelled,
        )
        .expect("provenance bound must produce a complete result");
        assert_eq!(report.results()[0].status(), expected);
        if expected == RuleStatus::Completed {
            assert_eq!(report.diagnostics()[0].observed_provenance_count(), 256);
        } else {
            assert!(report.diagnostics().is_empty());
        }
    }
}

fn many_diagnostics(rule: &str, count: usize) -> Vec<RuleDiagnostic> {
    (0..count)
        .map(|index| diagnostic(rule, &format!("code{index:04}"), "message", &[]))
        .collect()
}

#[test]
fn public_per_rule_diagnostic_bound_accepts_exact_and_fails_one_over_as_invalid_output() {
    for (count, expected) in [
        (MAX_RULE_DIAGNOSTICS_PER_RULE, RuleStatus::Completed),
        (MAX_RULE_DIAGNOSTICS_PER_RULE + 1, RuleStatus::Failed),
    ] {
        let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
            RuleEvaluation::Completed(many_diagnostics("rule", count))
        })])
        .expect("registry must pass");
        let report = execute(
            &registry,
            &RuleConfiguration::default(),
            &SemanticGraph::new(),
            &NeverCancelled,
        )
        .expect("execution must be complete");
        assert_eq!(report.results()[0].status(), expected);
        assert_eq!(
            report.diagnostics().len(),
            if expected == RuleStatus::Completed {
                count
            } else {
                0
            }
        );
    }
}

fn aggregate_registry(rule_count: usize) -> RuleRegistry<Arc<dyn Rule>> {
    RuleRegistry::new((0..rule_count).map(|index| {
        let name = format!("rule{index:02}");
        let diagnostic_rule = name.clone();
        rule(&name, &[], move |_, _| {
            RuleEvaluation::Completed(many_diagnostics(
                &diagnostic_rule,
                MAX_RULE_DIAGNOSTICS_PER_RULE,
            ))
        })
    }))
    .expect("aggregate registry must pass")
}

#[test]
fn public_aggregate_diagnostic_bound_accepts_exact_and_rejects_one_over_without_report() {
    let exact = aggregate_registry(16);
    let exact_report = execute(
        &exact,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect("exact aggregate must pass");
    assert_eq!(exact_report.diagnostics().len(), 65_536);

    let over = aggregate_registry(17);
    let error = execute(
        &over,
        &RuleConfiguration::default(),
        &SemanticGraph::new(),
        &NeverCancelled,
    )
    .expect_err("over aggregate must fail without a report");
    assert_eq!(error.kind(), RuleEngineErrorKind::TooManyRuleDiagnostics);
    assert_eq!(error.actual(), Some(69_632));
    assert_eq!(error.maximum(), Some(65_536));
}

#[test]
fn public_reordered_and_repeated_execution_is_equal_and_resource_free() {
    let first = RuleRegistry::new([
        completed_rule("c", &["a", "b"]),
        completed_rule("a", &[]),
        completed_rule("b", &[]),
    ])
    .expect("registry must pass");
    let second = RuleRegistry::new([
        completed_rule("b", &[]),
        completed_rule("c", &["b", "a"]),
        completed_rule("a", &[]),
    ])
    .expect("registry must pass");
    let configuration = RuleConfiguration::default();
    let graph = SemanticGraph::new();
    let expected =
        execute(&first, &configuration, &graph, &NeverCancelled).expect("execution must pass");
    assert_eq!(
        execute(&second, &configuration, &graph, &NeverCancelled).expect("execution must pass"),
        expected
    );
    assert_eq!(
        execute(&first, &configuration, &graph, &NeverCancelled).expect("execution must pass"),
        expected
    );
}

#[test]
fn public_failure_code_and_execution_errors_are_closed_and_redacted() {
    let sentinel = "private-invalid-code!";
    let invalid = RuleFailureCode::new(sentinel).expect_err("failure code must fail");
    assert_eq!(invalid.kind(), RuleEngineErrorKind::InvalidRuleFailureCode);
    assert!(!invalid.to_string().contains(sentinel));
    assert!(!format!("{invalid:?}").contains(sentinel));

    let registry = RuleRegistry::new([completed_rule("rule", &[])]).expect("registry must pass");
    let original_configuration = RuleConfiguration::default();
    let plan = RulePlan::new(&registry, &original_configuration).expect("plan must pass");
    let changed_configuration = RuleConfiguration::new([RuleSetting::new(
        rule_id("rule"),
        RuleSettingValue::Disabled,
    )])
    .expect("configuration must pass");
    let graph = SemanticGraph::new();
    let validation = graph.validate();
    let base = DiagnosticEngine
        .build(&[], &validation, &DiagnosticPolicy::default())
        .expect("base diagnostics must pass");
    let context = RuleContext::new(&graph, &validation, &base);
    let error = RuleEngine
        .execute(
            &registry,
            &plan,
            &changed_configuration,
            &context,
            &NeverCancelled,
        )
        .expect_err("mismatched plan must fail");
    assert_eq!(error.kind(), RuleEngineErrorKind::InvalidRulePlan);
}

#[test]
fn public_context_rejects_rule_evidence_in_the_base_diagnostic_report() {
    let calls = Arc::new(AtomicUsize::new(0));
    let rule_calls = Arc::clone(&calls);
    let registry = RuleRegistry::new([rule("rule", &[], move |_, _| {
        rule_calls.fetch_add(1, Ordering::SeqCst);
        RuleEvaluation::Completed(Vec::new())
    })])
    .expect("registry must pass");
    let configuration = RuleConfiguration::default();
    let plan = RulePlan::new(&registry, &configuration).expect("plan must pass");
    let graph = SemanticGraph::new();
    let validation = graph.validate();
    let base = DiagnosticEngine
        .build_with_rules(
            &[],
            &validation,
            &[diagnostic("rule", "finding", "already present", &[])],
            &DiagnosticPolicy::default(),
        )
        .expect("Rule-capable report must pass");
    let context = RuleContext::new(&graph, &validation, &base);

    let error = RuleEngine
        .execute(&registry, &plan, &configuration, &context, &NeverCancelled)
        .expect_err("Rule-bearing base report must fail");
    assert_eq!(error.kind(), RuleEngineErrorKind::InvalidRuleContext);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
