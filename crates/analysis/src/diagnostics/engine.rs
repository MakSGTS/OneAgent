//! Deterministic orchestration over Graph-owned diagnostic evidence.

use oneagent_graph::{SemanticDiagnostic, SemanticGraphValidationResult};

use super::{
    DiagnosticError, DiagnosticErrorKind, DiagnosticFinding, DiagnosticPolicy, DiagnosticReport,
    MAX_SEMANTIC_DIAGNOSTICS, MAX_VALIDATION_ISSUES, validate_count,
};

/// Stateless deterministic Diagnostics Engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct DiagnosticEngine;

impl DiagnosticEngine {
    /// Builds one complete bounded report from immutable Graph-owned evidence.
    ///
    /// The caller supplies the validation result. This method does not execute
    /// graph validation, inspect a graph, read source, or derive locations.
    ///
    /// # Errors
    ///
    /// Returns a closed bounded error without a partial report when an input,
    /// finding, normalized result, or identity collision violates the accepted
    /// diagnostic domain contract.
    pub fn build(
        &self,
        semantic_diagnostics: &[SemanticDiagnostic],
        validation: &SemanticGraphValidationResult,
        policy: &DiagnosticPolicy,
    ) -> Result<DiagnosticReport, DiagnosticError> {
        validate_input_counts(semantic_diagnostics.len(), validation.issues().len())?;

        let mut findings = Vec::with_capacity(
            semantic_diagnostics
                .len()
                .checked_add(validation.issues().len())
                .unwrap_or(MAX_SEMANTIC_DIAGNOSTICS + MAX_VALIDATION_ISSUES),
        );

        let mut semantic_inputs = semantic_diagnostics.iter().collect::<Vec<_>>();
        semantic_inputs.sort();
        for diagnostic in semantic_inputs {
            findings.push(DiagnosticFinding::from_semantic(diagnostic, policy)?);
        }

        let mut validation_inputs = validation.issues().iter().collect::<Vec<_>>();
        validation_inputs.sort();
        for issue in validation_inputs {
            findings.push(DiagnosticFinding::from_validation(issue, policy)?);
        }

        DiagnosticReport::new(findings)
    }
}

fn validate_input_counts(
    semantic_diagnostics: usize,
    validation_issues: usize,
) -> Result<(), DiagnosticError> {
    validate_count(
        DiagnosticErrorKind::TooManySemanticDiagnostics,
        semantic_diagnostics,
        MAX_SEMANTIC_DIAGNOSTICS,
    )?;
    validate_count(
        DiagnosticErrorKind::TooManyValidationIssues,
        validation_issues,
        MAX_VALIDATION_ISSUES,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        GraphNode, NodeKind, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    };

    use super::{DiagnosticEngine, validate_input_counts};
    use crate::diagnostics::{
        DiagnosticDisposition, DiagnosticErrorKind, DiagnosticFamily, DiagnosticIdentity,
        DiagnosticPolicy, DiagnosticSeverity, MAX_SEMANTIC_DIAGNOSTICS, MAX_VALIDATION_ISSUES,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn diagnostic(
        source: Option<&str>,
        severity: SemanticDiagnosticSeverity,
        message: &str,
    ) -> SemanticDiagnostic {
        let diagnostic = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            severity,
            SemanticDiagnosticKind::UnresolvedTarget,
            message,
            SemanticReference::NodeId("metadata.target".to_owned()),
        );
        source.map_or(diagnostic.clone(), |source| {
            diagnostic.with_source_node(id(source))
        })
    }

    fn empty_validation() -> oneagent_graph::SemanticGraphValidationResult {
        SemanticGraph::new().validate()
    }

    fn provenance_validation_issue() -> oneagent_graph::SemanticGraphValidationIssue {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id("metadata.validation"),
            name("Validation"),
            NodeKind::Unknown,
        ));
        graph
            .validate()
            .issues()
            .first()
            .expect("missing provenance must produce an issue")
            .clone()
    }

    fn provenance_validation() -> oneagent_graph::SemanticGraphValidationResult {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id("metadata.validation"),
            name("Validation"),
            NodeKind::Unknown,
        ));
        graph.validate()
    }

    #[test]
    fn empty_inputs_create_a_complete_empty_report() {
        let report = DiagnosticEngine
            .build(&[], &empty_validation(), &DiagnosticPolicy::default())
            .expect("empty inputs must be valid");

        assert!(report.findings().is_empty());
        assert_eq!(report.summary().total(), 0);
    }

    #[test]
    fn mixed_inputs_preserve_families_and_canonical_error_first_order() {
        let semantic = diagnostic(
            Some("metadata.semantic"),
            SemanticDiagnosticSeverity::Error,
            "semantic error",
        );
        let report = DiagnosticEngine
            .build(
                &[semantic],
                &provenance_validation(),
                &DiagnosticPolicy::default(),
            )
            .expect("mixed inputs must be valid");

        assert_eq!(report.findings().len(), 2);
        assert_eq!(report.findings()[0].family(), DiagnosticFamily::Semantic);
        assert_eq!(report.findings()[0].severity(), DiagnosticSeverity::Error);
        assert_eq!(report.findings()[1].family(), DiagnosticFamily::Validation);
        assert_eq!(report.findings()[1].severity(), DiagnosticSeverity::Warning);
        assert_eq!(report.summary().by_family().values().sum::<usize>(), 2);
    }

    #[test]
    fn reordered_duplicates_and_repeated_builds_are_equal() {
        let first = diagnostic(
            Some("metadata.a"),
            SemanticDiagnosticSeverity::Warning,
            "first",
        );
        let second = diagnostic(
            Some("metadata.b"),
            SemanticDiagnosticSeverity::Error,
            "second",
        );
        let validation = empty_validation();
        let policy = DiagnosticPolicy::default();

        let expected = DiagnosticEngine
            .build(
                &[second.clone(), first.clone(), first.clone()],
                &validation,
                &policy,
            )
            .expect("duplicates must normalize");
        let actual = DiagnosticEngine
            .build(&[first, second], &validation, &policy)
            .expect("reordered inputs must normalize");
        let repeated = DiagnosticEngine
            .build(&actual_semantic_evidence(&actual), &validation, &policy)
            .expect("repeated evidence must normalize");

        assert_eq!(expected, actual);
        assert_eq!(actual, repeated);
        assert_eq!(actual.summary().total(), 2);
    }

    #[test]
    fn exact_suppression_is_applied_to_both_families_once() {
        let semantic = diagnostic(
            Some("metadata.semantic"),
            SemanticDiagnosticSeverity::Error,
            "semantic",
        );
        let validation = provenance_validation();
        let validation_identity = DiagnosticIdentity::from_validation(
            validation
                .issues()
                .first()
                .expect("validation issue must exist"),
        )
        .expect("validation identity must fit bounds");
        let policy = DiagnosticPolicy::new(BTreeSet::from([
            DiagnosticIdentity::from_semantic(&semantic),
            validation_identity,
        ]))
        .expect("two suppressions must be valid");

        let report = DiagnosticEngine
            .build(&[semantic], &validation, &policy)
            .expect("suppressed inputs must remain reportable");

        assert_eq!(report.summary().active(), 0);
        assert_eq!(report.summary().suppressed(), 2);
        assert!(
            report
                .findings()
                .iter()
                .all(|finding| finding.disposition() == DiagnosticDisposition::Suppressed)
        );
    }

    #[test]
    fn same_identity_with_different_content_fails_independently_of_order() {
        let first = diagnostic(
            Some("metadata.source"),
            SemanticDiagnosticSeverity::Error,
            "first-secret-marker",
        );
        let second = diagnostic(
            Some("metadata.source"),
            SemanticDiagnosticSeverity::Warning,
            "second-secret-marker",
        );
        let validation = empty_validation();

        for diagnostics in [
            vec![first.clone(), second.clone()],
            vec![second.clone(), first.clone()],
        ] {
            let error = DiagnosticEngine
                .build(&diagnostics, &validation, &DiagnosticPolicy::default())
                .expect_err("identity collision must fail");
            assert_eq!(error.kind(), DiagnosticErrorKind::ConflictingEvidence);
            assert!(!error.to_string().contains("secret-marker"));
            assert!(!format!("{error:?}").contains("secret-marker"));
        }
    }

    #[test]
    fn missing_optional_source_anchor_remains_unavailable() {
        let semantic = diagnostic(None, SemanticDiagnosticSeverity::Error, "unlocated");
        let report = DiagnosticEngine
            .build(
                &[semantic],
                &empty_validation(),
                &DiagnosticPolicy::default(),
            )
            .expect("missing source anchor is valid evidence");

        assert!(report.findings()[0].node_anchors().is_empty());
    }

    #[test]
    fn input_count_bounds_accept_exact_and_reject_one_over_in_precedence_order() {
        assert!(validate_input_counts(MAX_SEMANTIC_DIAGNOSTICS, MAX_VALIDATION_ISSUES).is_ok());

        let semantic_error =
            validate_input_counts(MAX_SEMANTIC_DIAGNOSTICS + 1, MAX_VALIDATION_ISSUES + 1)
                .expect_err("semantic count has precedence");
        assert_eq!(
            semantic_error.kind(),
            DiagnosticErrorKind::TooManySemanticDiagnostics
        );

        let validation_error =
            validate_input_counts(MAX_SEMANTIC_DIAGNOSTICS, MAX_VALIDATION_ISSUES + 1)
                .expect_err("validation one-over must fail");
        assert_eq!(
            validation_error.kind(),
            DiagnosticErrorKind::TooManyValidationIssues
        );
    }

    #[test]
    fn validation_fixture_is_canonical_and_repeatable() {
        let first = provenance_validation_issue();
        let second = provenance_validation_issue();
        assert_eq!(first, second);
    }

    fn actual_semantic_evidence(
        report: &crate::diagnostics::DiagnosticReport,
    ) -> Vec<SemanticDiagnostic> {
        report
            .findings()
            .iter()
            .filter_map(|finding| match finding.evidence() {
                crate::diagnostics::DiagnosticEvidence::Semantic(diagnostic) => {
                    Some(diagnostic.clone())
                }
                crate::diagnostics::DiagnosticEvidence::Validation(_) => None,
            })
            .collect()
    }
}
