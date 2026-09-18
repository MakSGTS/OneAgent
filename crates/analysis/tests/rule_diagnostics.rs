use std::collections::BTreeSet;

use oneagent_analysis::diagnostics::{
    DiagnosticCategory, DiagnosticCode, DiagnosticDisposition, DiagnosticEngine,
    DiagnosticErrorKind, DiagnosticEvidence, DiagnosticFamily, DiagnosticFilter,
    DiagnosticIdentity, DiagnosticKind, DiagnosticPolicy, DiagnosticSeverity,
};
use oneagent_analysis::rules::{RuleDiagnostic, RuleDiagnosticCode, RuleId};
use oneagent_common::EntityId;
use oneagent_graph::{
    SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticDiagnosticSeverity,
    SemanticGraph, SemanticReference,
};

fn rule_id(value: &str) -> RuleId {
    RuleId::new(value).expect("test rule ID must be valid")
}

fn code(value: &str) -> RuleDiagnosticCode {
    RuleDiagnosticCode::new(value).expect("test code must be valid")
}

fn rule_diagnostic(rule: &str, code_value: &str, message: &str) -> RuleDiagnostic {
    RuleDiagnostic::new(
        rule_id(rule),
        code(code_value),
        DiagnosticSeverity::Warning,
        DiagnosticCategory::Semantic,
        message,
        [EntityId::new("node").expect("node ID must pass")],
    )
}

fn empty_validation() -> oneagent_graph::SemanticGraphValidationResult {
    SemanticGraph::new().validate()
}

#[test]
fn public_rule_vocabulary_identity_evidence_and_projection_are_typed_and_stable() {
    let diagnostic = rule_diagnostic("rule", "finding", "message");
    let report = DiagnosticEngine
        .build_with_rules(
            &[],
            &empty_validation(),
            std::slice::from_ref(&diagnostic),
            &DiagnosticPolicy::default(),
        )
        .expect("Rule report must pass");
    let finding = &report.findings()[0];

    assert_eq!(DiagnosticFamily::Rule.as_str(), "rule");
    assert_eq!(DiagnosticKind::Rule.as_str(), "rule_finding");
    assert_eq!(finding.family(), DiagnosticFamily::Rule);
    assert_eq!(finding.code(), &DiagnosticCode::Rule(code("finding")));
    assert_eq!(finding.kind(), DiagnosticKind::Rule);
    assert_eq!(finding.message(), "message");
    assert_eq!(finding.node_anchors(), diagnostic.node_anchors());
    assert!(matches!(
        finding.identity(),
        DiagnosticIdentity::Rule { rule_id, code, node_anchors }
            if rule_id.as_str() == "rule"
                && code.as_str() == "finding"
                && node_anchors == diagnostic.node_anchors()
    ));
    assert!(matches!(
        finding.evidence(),
        DiagnosticEvidence::Rule(evidence) if evidence == &diagnostic
    ));
    assert_eq!(
        report.summary().by_family().get(&DiagnosticFamily::Rule),
        Some(&1)
    );
}

#[test]
fn public_rule_exact_duplicates_collapse_and_identity_conflicts_fail_closed() {
    let first = rule_diagnostic("rule", "finding", "first");
    let equal = first.clone();
    let conflict = rule_diagnostic("rule", "finding", "second-private-message");
    let validation = empty_validation();

    let duplicate_report = DiagnosticEngine
        .build_with_rules(
            &[],
            &validation,
            &[first.clone(), equal],
            &DiagnosticPolicy::default(),
        )
        .expect("exact duplicates must collapse");
    assert_eq!(duplicate_report.findings().len(), 1);

    for diagnostics in [
        vec![first.clone(), conflict.clone()],
        vec![conflict.clone(), first.clone()],
    ] {
        let error = DiagnosticEngine
            .build_with_rules(&[], &validation, &diagnostics, &DiagnosticPolicy::default())
            .expect_err("identity conflict must fail");
        assert_eq!(error.kind(), DiagnosticErrorKind::ConflictingEvidence);
        assert!(!error.to_string().contains("second-private-message"));
        assert!(!format!("{error:?}").contains("second-private-message"));
    }
}

#[test]
fn public_equal_local_codes_from_different_rules_remain_distinct_findings() {
    let report = DiagnosticEngine
        .build_with_rules(
            &[],
            &empty_validation(),
            &[
                rule_diagnostic("rule.b", "finding", "same"),
                rule_diagnostic("rule.a", "finding", "same"),
            ],
            &DiagnosticPolicy::default(),
        )
        .expect("cross-rule findings must pass");

    assert_eq!(report.findings().len(), 2);
    let rule_ids = report
        .findings()
        .iter()
        .map(|finding| match finding.identity() {
            DiagnosticIdentity::Rule { rule_id, .. } => rule_id.as_str(),
            _ => panic!("expected Rule identity"),
        })
        .collect::<Vec<_>>();
    assert_eq!(rule_ids, ["rule.a", "rule.b"]);
}

#[test]
fn public_rule_identity_suppression_and_filtering_preserve_complete_summary() {
    let diagnostic = rule_diagnostic("rule", "finding", "suppressed");
    let identity = DiagnosticIdentity::Rule {
        rule_id: diagnostic.rule_id().clone(),
        code: diagnostic.code().clone(),
        node_anchors: diagnostic.node_anchors().to_vec(),
    };
    let policy =
        DiagnosticPolicy::new(BTreeSet::from([identity])).expect("suppression policy must pass");
    let report = DiagnosticEngine
        .build_with_rules(&[], &empty_validation(), &[diagnostic], &policy)
        .expect("suppressed Rule report must pass");
    let active_filter = DiagnosticFilter::new(
        BTreeSet::from([DiagnosticFamily::Rule]),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::from([DiagnosticDisposition::Active]),
    );

    assert_eq!(report.summary().total(), 1);
    assert_eq!(report.summary().active(), 0);
    assert_eq!(report.summary().suppressed(), 1);
    assert_eq!(report.filtered(&active_filter).count(), 0);
    assert_eq!(
        report.findings()[0].disposition(),
        DiagnosticDisposition::Suppressed
    );
}

#[test]
fn public_rule_findings_share_mixed_family_order_and_summary_boundary() {
    let semantic = SemanticDiagnostic::new(
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticSeverity::Error,
        SemanticDiagnosticKind::UnresolvedTarget,
        "semantic",
        SemanticReference::NodeId("target".to_owned()),
    );
    let report = DiagnosticEngine
        .build_with_rules(
            &[semantic],
            &empty_validation(),
            &[rule_diagnostic("rule", "finding", "rule")],
            &DiagnosticPolicy::default(),
        )
        .expect("mixed report must pass");

    assert_eq!(report.findings().len(), 2);
    assert_eq!(report.findings()[0].family(), DiagnosticFamily::Semantic);
    assert_eq!(report.findings()[1].family(), DiagnosticFamily::Rule);
    assert_eq!(report.summary().total(), 2);
    assert_eq!(report.summary().by_family().values().sum::<usize>(), 2);
}

#[test]
fn public_existing_build_remains_equal_to_build_with_empty_rule_input() {
    let semantic = SemanticDiagnostic::new(
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticSeverity::Warning,
        SemanticDiagnosticKind::UnresolvedTarget,
        "semantic",
        SemanticReference::NodeId("target".to_owned()),
    );
    let validation = empty_validation();
    let policy = DiagnosticPolicy::default();

    assert_eq!(
        DiagnosticEngine
            .build(std::slice::from_ref(&semantic), &validation, &policy)
            .expect("legacy build must pass"),
        DiagnosticEngine
            .build_with_rules(&[semantic], &validation, &[], &policy)
            .expect("empty Rule build must pass")
    );
}
