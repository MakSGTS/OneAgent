use std::collections::BTreeSet;

use oneagent_analysis::diagnostics::{
    DiagnosticDisposition, DiagnosticEngine, DiagnosticErrorKind, DiagnosticFamily,
    DiagnosticFilter, DiagnosticIdentity, DiagnosticPolicy, DiagnosticSeverity,
    MAX_SEMANTIC_DIAGNOSTICS,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    GraphNode, NodeKind, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn diagnostic(source: &str, severity: SemanticDiagnosticSeverity) -> SemanticDiagnostic {
    SemanticDiagnostic::new(
        SemanticDiagnosticCode::ReferenceUnresolved,
        severity,
        SemanticDiagnosticKind::UnresolvedTarget,
        "semantic reference target could not be resolved",
        SemanticReference::NodeId("metadata.target".to_owned()),
    )
    .with_source_node(id(source))
}

fn validation_graph() -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new(
        id("metadata.validation"),
        name("Validation"),
        NodeKind::Unknown,
    ));
    graph
}

#[test]
fn public_engine_builds_equal_complete_mixed_reports_from_reordered_input() {
    let first = diagnostic("metadata.first", SemanticDiagnosticSeverity::Warning);
    let second = diagnostic("metadata.second", SemanticDiagnosticSeverity::Error);
    let validation = validation_graph().validate();

    let expected = DiagnosticEngine
        .build(
            &[first.clone(), second.clone(), first.clone()],
            &validation,
            &DiagnosticPolicy::default(),
        )
        .expect("mixed report must build");
    let actual = DiagnosticEngine
        .build(&[second, first], &validation, &DiagnosticPolicy::default())
        .expect("reordered mixed report must build");

    assert_eq!(expected, actual);
    assert_eq!(actual.summary().total(), 3);
    assert_eq!(actual.summary().active(), 3);
    assert_eq!(actual.summary().suppressed(), 0);
    assert_eq!(actual.findings()[0].severity(), DiagnosticSeverity::Error);
    assert_eq!(
        actual
            .summary()
            .by_family()
            .get(&DiagnosticFamily::Semantic),
        Some(&2)
    );
    assert_eq!(
        actual
            .summary()
            .by_family()
            .get(&DiagnosticFamily::Validation),
        Some(&1)
    );
}

#[test]
fn public_engine_retains_exact_suppression_and_filters_without_changing_summary() {
    let active = diagnostic("metadata.active", SemanticDiagnosticSeverity::Error);
    let suppressed = diagnostic("metadata.suppressed", SemanticDiagnosticSeverity::Warning);
    let policy = DiagnosticPolicy::new(BTreeSet::from([DiagnosticIdentity::from_semantic(
        &suppressed,
    )]))
    .expect("single exact suppression must be valid");
    let validation = SemanticGraph::new().validate();
    let report = DiagnosticEngine
        .build(&[suppressed, active], &validation, &policy)
        .expect("suppressed report must build");
    let active_filter = DiagnosticFilter::new(
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::from([DiagnosticDisposition::Active]),
    );

    assert_eq!(report.summary().total(), 2);
    assert_eq!(report.summary().active(), 1);
    assert_eq!(report.summary().suppressed(), 1);
    assert_eq!(report.filtered(&active_filter).count(), 1);
    assert_eq!(report.summary().total(), 2);
}

#[test]
fn public_engine_rejects_one_over_semantic_input_without_partial_report() {
    let diagnostic = diagnostic("metadata.source", SemanticDiagnosticSeverity::Error);
    let diagnostics = vec![diagnostic; MAX_SEMANTIC_DIAGNOSTICS + 1];
    let validation = SemanticGraph::new().validate();

    let error = DiagnosticEngine
        .build(&diagnostics, &validation, &DiagnosticPolicy::default())
        .expect_err("one-over semantic input must fail");

    assert_eq!(
        error.kind(),
        DiagnosticErrorKind::TooManySemanticDiagnostics
    );
    assert_eq!(error.actual(), Some(MAX_SEMANTIC_DIAGNOSTICS + 1));
    assert_eq!(error.maximum(), Some(MAX_SEMANTIC_DIAGNOSTICS));
}
