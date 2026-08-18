use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
    ReferenceRequestChangeKind, ReferenceRequestModifiedAspect, ResolutionState,
    SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticDiagnosticSeverity,
    SemanticGraph, SemanticGraphBuildDiff, SemanticGraphBuildSnapshot, SemanticGraphReport,
    SemanticGraphValidationCode, SemanticGraphValidator, SemanticReference,
    SemanticReferenceCategory, SemanticReferenceOutcome, SemanticReferenceRequest,
    SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome, SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: &str, resolution: ResolutionState) -> Provenance {
    Provenance::new(
        Some(id(source)),
        ProducerId::new(format!("oneagent.graph.reference-request-build.{source}")),
        if resolution == ResolutionState::Unresolved {
            FactOrigin::Declared
        } else {
            FactOrigin::Resolved
        },
        Confidence::Exact,
        resolution,
    )
}

fn collected(source: &str, target: &str) -> SemanticReferenceRequest {
    SemanticReferenceRequest::collected(
        id(source),
        SemanticReferenceCategory::MetadataType,
        SemanticReference::Name(name(target)),
        [NodeKind::Metadata(MetadataKind::Catalog)],
        [provenance(
            &format!("{source}.collection"),
            ResolutionState::Unresolved,
        )],
    )
    .expect("request must be valid")
}

fn terminal_requests() -> SemanticReferenceRequestLedger {
    let resolved = collected("source.resolved", "Resolved")
        .into_resolved(
            id("target.resolved"),
            NodeKind::Metadata(MetadataKind::Catalog),
            [provenance(
                "source.resolved.resolution",
                ResolutionState::Resolved,
            )],
        )
        .expect("resolved request must be valid");
    let missing = collected("source.missing", "Missing")
        .into_missing_target([provenance(
            "source.missing.resolution",
            ResolutionState::Unresolved,
        )])
        .expect("missing request must be valid");
    let partial = collected("source.partial", "Partial")
        .into_partial_workspace(
            [],
            [provenance(
                "source.partial.resolution",
                ResolutionState::Partial,
            )],
        )
        .expect("partial request must be valid");
    let ambiguous = collected("source.ambiguous", "Ambiguous")
        .into_ambiguous_target(
            [id("target.ambiguous.b"), id("target.ambiguous.a")],
            [provenance(
                "source.ambiguous.resolution",
                ResolutionState::Ambiguous,
            )],
        )
        .expect("ambiguous request must be valid");
    let incompatible = collected("source.incompatible", "Incompatible")
        .into_incompatible_target_kind(
            [id("target.incompatible")],
            [provenance(
                "source.incompatible.resolution",
                ResolutionState::Unresolved,
            )],
        )
        .expect("incompatible request must be valid");
    let invalid_owner = collected("source.invalid-owner", "InvalidOwner")
        .into_invalid_owner_reference(
            [id("target.invalid-owner")],
            [provenance(
                "source.invalid-owner.resolution",
                ResolutionState::Unresolved,
            )],
        )
        .expect("invalid-owner request must be valid");

    SemanticReferenceRequestLedger::from_requests([
        invalid_owner,
        incompatible,
        ambiguous,
        partial,
        missing,
        resolved,
    ])
    .expect("terminal ledger must be valid")
}

#[test]
fn request_aware_report_derives_every_terminal_outcome_once() {
    let graph = SemanticGraph::new();
    let ledger = terminal_requests();
    let report =
        SemanticGraphReport::from_graph_diagnostics_and_reference_requests(&graph, &[], &ledger);

    assert_eq!(report.resolution().total(), 6);
    assert_eq!(report.resolution().resolved(), 1);
    assert_eq!(report.resolution().unresolved(), 2);
    assert_eq!(report.resolution().ambiguous(), 1);
    assert_eq!(report.resolution().incompatible_target_kind(), 1);
    assert_eq!(report.resolution().invalid_owner_reference(), 1);
    assert_eq!(report.resolution().with_provenance(), 6);

    let collected =
        SemanticReferenceRequestLedger::from_requests([collected("source.collected", "Collected")])
            .expect("collected ledger must be valid");
    assert!(SemanticReferenceStatistics::from_reference_requests(&collected).is_empty());

    let mut legacy = SemanticReferenceStatistics::new();
    legacy.record(SemanticReferenceOutcome::MalformedFormat, true);
    let transitional =
        SemanticGraphReport::from_graph_diagnostics_reference_requests_and_legacy_observations(
            &graph,
            &[],
            &ledger,
            legacy,
        );
    assert_eq!(transitional.resolution().total(), 7);
    assert_eq!(transitional.resolution().malformed_format(), 1);
}

#[test]
fn request_diff_is_identity_based_deterministic_and_reports_mutable_aspects() {
    let evolving = collected("source.evolving", "Evolving");
    let previous_evolving = evolving
        .clone()
        .into_missing_target([provenance(
            "source.evolving.missing",
            ResolutionState::Unresolved,
        )])
        .expect("missing request must be valid");
    let current_evolving = evolving
        .into_resolved(
            id("target.evolving"),
            NodeKind::Metadata(MetadataKind::Catalog),
            [provenance(
                "source.evolving.resolved",
                ResolutionState::Resolved,
            )],
        )
        .expect("resolved request must be valid");
    assert_eq!(previous_evolving.id(), current_evolving.id());

    let removed = collected("source.removed", "Removed")
        .into_missing_target([provenance(
            "source.removed.resolution",
            ResolutionState::Unresolved,
        )])
        .expect("removed request must be valid");
    let added = collected("source.added", "Added")
        .into_missing_target([provenance(
            "source.added.resolution",
            ResolutionState::Unresolved,
        )])
        .expect("added request must be valid");
    let previous =
        SemanticReferenceRequestLedger::from_requests([removed.clone(), previous_evolving.clone()])
            .expect("previous ledger must be valid");
    let current =
        SemanticReferenceRequestLedger::from_requests([added.clone(), current_evolving.clone()])
            .expect("current ledger must be valid");
    let graph = SemanticGraph::new();

    let first = SemanticGraphBuildDiff::between_with_reference_requests(
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &previous),
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &current),
    );
    let repeated = SemanticGraphBuildDiff::between_with_reference_requests(
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &previous),
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &current),
    );

    assert_eq!(first, repeated);
    assert_eq!(first.reference_requests().summary().added(), 1);
    assert_eq!(first.reference_requests().summary().removed(), 1);
    assert_eq!(first.reference_requests().summary().modified(), 1);
    assert_eq!(first.summary().reference_request_changes(), 3);
    assert_eq!(
        first.reference_requests().added()[0].kind(),
        ReferenceRequestChangeKind::Added
    );
    assert_eq!(first.reference_requests().added()[0].id(), added.id());
    assert_eq!(first.reference_requests().removed()[0].id(), removed.id());
    assert_eq!(
        first.reference_requests().modified()[0].modified_aspects(),
        &[
            ReferenceRequestModifiedAspect::Candidates,
            ReferenceRequestModifiedAspect::State,
            ReferenceRequestModifiedAspect::Outcome,
            ReferenceRequestModifiedAspect::Provenance,
        ]
    );
}

#[test]
fn duplicate_request_provenance_is_merged_without_double_counting() {
    let base = collected("source.duplicate", "Duplicate");
    let first = base
        .clone()
        .into_missing_target([provenance(
            "source.duplicate.first",
            ResolutionState::Unresolved,
        )])
        .expect("first request must be valid");
    let duplicate = base
        .into_missing_target([provenance(
            "source.duplicate.second",
            ResolutionState::Unresolved,
        )])
        .expect("duplicate request must be valid");
    let previous = SemanticReferenceRequestLedger::from_requests([first.clone()])
        .expect("previous ledger must be valid");
    let current = SemanticReferenceRequestLedger::from_requests([duplicate, first])
        .expect("duplicate provenance must merge");
    let graph = SemanticGraph::new();

    let diff = SemanticGraphBuildDiff::between_with_reference_requests(
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &previous),
        SemanticGraphBuildSnapshot::from_reference_requests(&graph, &[], &current),
    );

    assert_eq!(current.requests().len(), 1);
    assert_eq!(
        SemanticReferenceStatistics::from_reference_requests(&current).total(),
        1
    );
    assert_eq!(diff.reference_requests().modified().len(), 1);
    assert_eq!(
        diff.reference_requests().modified()[0].modified_aspects(),
        &[ReferenceRequestModifiedAspect::Provenance]
    );
}

fn projection_graph(ledger: &SemanticReferenceRequestLedger) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    for request in ledger.requests() {
        graph.insert_node(GraphNode::new_with_provenance(
            request.source_node().clone(),
            name(request.source_node().as_str()),
            NodeKind::Unknown,
            vec![provenance(
                &format!("{}.node", request.source_node().as_str()),
                ResolutionState::NotApplicable,
            )],
        ));
    }
    for (candidate, kind) in [
        ("target.resolved", NodeKind::Metadata(MetadataKind::Catalog)),
        (
            "target.ambiguous.a",
            NodeKind::Metadata(MetadataKind::Catalog),
        ),
        (
            "target.ambiguous.b",
            NodeKind::Metadata(MetadataKind::Catalog),
        ),
        ("target.incompatible", NodeKind::Role),
        (
            "target.invalid-owner",
            NodeKind::Metadata(MetadataKind::Catalog),
        ),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            id(candidate),
            name(candidate),
            kind,
            vec![provenance(
                &format!("{candidate}.node"),
                ResolutionState::NotApplicable,
            )],
        ));
    }
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            id("source.resolved"),
            id("target.resolved"),
            EdgeKind::References,
            vec![provenance(
                "source.resolved.edge",
                ResolutionState::Resolved,
            )],
        ))
        .expect("resolved projection must be valid");
    graph
}

fn failed_diagnostics(ledger: &SemanticReferenceRequestLedger) -> Vec<SemanticDiagnostic> {
    ledger
        .requests()
        .iter()
        .filter_map(|request| {
            let (code, kind) = match request.outcome() {
                SemanticReferenceRequestOutcome::MissingTarget => (
                    SemanticDiagnosticCode::ReferenceUnresolved,
                    SemanticDiagnosticKind::UnresolvedTarget,
                ),
                SemanticReferenceRequestOutcome::AmbiguousTarget => (
                    SemanticDiagnosticCode::ReferenceAmbiguous,
                    SemanticDiagnosticKind::AmbiguousTarget,
                ),
                SemanticReferenceRequestOutcome::IncompatibleTargetKind => (
                    SemanticDiagnosticCode::ReferenceIncompatibleKind,
                    SemanticDiagnosticKind::IncompatibleTargetKind,
                ),
                SemanticReferenceRequestOutcome::InvalidOwnerReference => (
                    SemanticDiagnosticCode::ReferenceInvalidOwner,
                    SemanticDiagnosticKind::InvalidOwnerReference,
                ),
                SemanticReferenceRequestOutcome::Collected
                | SemanticReferenceRequestOutcome::Resolved
                | SemanticReferenceRequestOutcome::PartialWorkspace => return None,
            };
            Some(
                SemanticDiagnostic::new(
                    code,
                    SemanticDiagnosticSeverity::Error,
                    kind,
                    "request projection",
                    request.reference().clone(),
                )
                .with_source_node(request.source_node().clone())
                .with_expected_kinds(request.expected_kinds().to_vec())
                .with_candidates(request.candidates().to_vec())
                .with_provenance(request.provenance().to_vec()),
            )
        })
        .collect()
}

#[test]
fn request_aware_validation_accepts_complete_terminal_projections() {
    let ledger = terminal_requests();
    let graph = projection_graph(&ledger);
    let diagnostics = failed_diagnostics(&ledger);

    let result = SemanticGraphValidator::new().validate_build_result_with_reference_requests(
        &graph,
        &diagnostics,
        &ledger,
    );

    assert!(result.is_valid(), "issues: {:?}", result.issues());
}

#[test]
fn request_aware_validation_reports_lifecycle_node_and_projection_mismatches() {
    let collected_request = collected("source.absent", "Collected");
    let incompatible_ambiguous = collected("source.ambiguous", "Ambiguous")
        .into_ambiguous_target(
            [id("candidate.absent"), id("candidate.role")],
            [provenance(
                "source.ambiguous.resolution",
                ResolutionState::Ambiguous,
            )],
        )
        .expect("ambiguous request must be valid");
    let ledger =
        SemanticReferenceRequestLedger::from_requests([collected_request, incompatible_ambiguous])
            .expect("ledger must be valid");
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new(
        id("source.ambiguous"),
        name("Source"),
        NodeKind::Unknown,
    ));
    graph.insert_node(GraphNode::new(
        id("candidate.role"),
        name("Role"),
        NodeKind::Role,
    ));
    graph
        .insert_edge(GraphEdge::new(
            id("source.ambiguous"),
            id("candidate.role"),
            EdgeKind::References,
        ))
        .expect("unexpected projection edge must be insertable");

    let result = SemanticGraphValidator::new().validate_build_result_with_reference_requests(
        &graph,
        &[],
        &ledger,
    );
    let codes = result
        .issues()
        .iter()
        .map(oneagent_graph::SemanticGraphValidationIssue::code)
        .collect::<Vec<_>>();

    for code in [
        SemanticGraphValidationCode::NonTerminalReferenceRequest,
        SemanticGraphValidationCode::MissingReferenceRequestSource,
        SemanticGraphValidationCode::MissingReferenceRequestCandidate,
        SemanticGraphValidationCode::IncompatibleReferenceRequestCandidate,
        SemanticGraphValidationCode::UnexpectedReferenceRequestEdgeProjection,
        SemanticGraphValidationCode::MissingReferenceRequestDiagnosticProjection,
    ] {
        assert!(codes.contains(&code), "missing validation code {code:?}");
    }
    assert!(
        result
            .issues()
            .iter()
            .any(|issue| issue.reference_request_id().is_some())
    );
}

#[test]
fn request_aware_validation_reports_missing_resolved_edge_and_failed_diagnostic() {
    let ledger = terminal_requests();
    let graph = {
        let mut graph = projection_graph(&ledger);
        let mut without_edge = SemanticGraph::new();
        for node in graph.nodes().cloned() {
            without_edge.insert_node(node);
        }
        graph = without_edge;
        graph
    };
    let result = SemanticGraphValidator::new().validate_build_result_with_reference_requests(
        &graph,
        &[],
        &ledger,
    );

    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::MissingReferenceRequestEdgeProjection
    }));
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::MissingReferenceRequestDiagnosticProjection
    }));
}

#[test]
fn request_aware_validation_rejects_statistics_report_not_derived_from_ledger() {
    let ledger = terminal_requests();
    let graph = projection_graph(&ledger);
    let diagnostics = failed_diagnostics(&ledger);
    let incompatible_report = SemanticGraphReport::from_graph_diagnostics_and_references(
        &graph,
        &diagnostics,
        SemanticReferenceStatistics::new(),
    );

    let result = SemanticGraphValidator::new()
        .validate_build_result_with_reference_requests_and_report(
            &graph,
            &diagnostics,
            &ledger,
            SemanticReferenceStatistics::new(),
            &incompatible_report,
        );

    assert!(
        result
            .issues()
            .iter()
            .any(|issue| { issue.code() == SemanticGraphValidationCode::InconsistentReport })
    );
}
