use std::sync::atomic::{AtomicBool, Ordering};

use oneagent_analysis::change_impact::{
    CHANGE_IMPACT_MAX_DEPTH, ChangeImpactCancellationSignal, ChangeImpactCompleteness,
    ChangeImpactConfiguration, ChangeImpactErrorKind, ChangeImpactEvaluator,
    ChangeImpactPublicationId, ConfigurationImpactKind, MAX_CHANGE_IMPACT_CONFIGURATIONS,
    MAX_CHANGE_IMPACT_IDENTIFIER_BYTES, NeverCancelledChangeImpact,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, ImpactNodeAvailability, ImpactNodeStatus, ImpactReasonKind,
    NodeId, NodeKind, SemanticGraph, SemanticGraphQuery,
};

fn id(value: impl Into<String>) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn add_node(graph: &mut SemanticGraph, id_value: &str, name_value: &str, kind: NodeKind) {
    graph.insert_node(GraphNode::new(id(id_value), name(name_value), kind));
}

fn add_edge(graph: &mut SemanticGraph, source: &str, target: &str, kind: EdgeKind) {
    graph
        .insert_edge(GraphEdge::new(id(source), id(target), kind))
        .expect("edge endpoints must exist");
}

fn dependency_graph(
    target_name: &str,
    include_removed: bool,
    include_added: bool,
) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    add_node(
        &mut graph,
        "procedure.caller",
        "Caller",
        NodeKind::Procedure,
    );
    add_node(
        &mut graph,
        "function.target",
        target_name,
        NodeKind::Function,
    );
    add_edge(
        &mut graph,
        "procedure.caller",
        "function.target",
        EdgeKind::Calls,
    );
    if include_removed {
        add_node(
            &mut graph,
            "function.removed",
            "Removed",
            NodeKind::Function,
        );
    }
    if include_added {
        add_node(&mut graph, "function.added", "Added", NodeKind::Function);
    }
    graph
}

fn evaluate(
    previous: &[(EntityId, SemanticGraph)],
    current: &[(EntityId, SemanticGraph)],
) -> oneagent_analysis::change_impact::ChangeImpactReport {
    let previous = previous
        .iter()
        .map(|(id, graph)| ChangeImpactConfiguration::new(id, graph))
        .collect::<Vec<_>>();
    let current = current
        .iter()
        .map(|(id, graph)| ChangeImpactConfiguration::new(id, graph))
        .collect::<Vec<_>>();
    ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &previous,
            &current,
            &NeverCancelledChangeImpact,
        )
        .expect("report must build")
}

#[test]
fn empty_endpoints_produce_complete_adjacent_empty_report() {
    let report = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &[],
            &[],
            &NeverCancelledChangeImpact,
        )
        .expect("empty report must build");

    assert_eq!(report.previous_publication_id().get(), 1);
    assert_eq!(report.current_publication_id().get(), 2);
    assert_eq!(
        report.completeness(),
        ChangeImpactCompleteness::CompleteWithinConfiguredDepth
    );
    assert!(report.is_empty());
    assert_eq!(report.summary().total_configurations(), 0);
    assert_eq!(
        report.summary().configured_max_depth(),
        CHANGE_IMPACT_MAX_DEPTH
    );
}

#[test]
fn equal_graphs_produce_a_complete_empty_compared_transition() {
    let graph = dependency_graph("Target", false, false);
    let report = evaluate(
        &[(id("configuration.main"), graph.clone())],
        &[(id("configuration.main"), graph)],
    );
    let configuration = &report.configurations()[0];

    assert_eq!(configuration.kind(), ConfigurationImpactKind::Compared);
    assert!(configuration.result().is_empty());
    assert_eq!(configuration.summary().total_affected_nodes(), 0);
    assert_eq!(report.summary().compared_configurations(), 1);
    assert_eq!(report.summary().added_configurations(), 0);
    assert_eq!(report.summary().removed_configurations(), 0);
}

#[test]
fn report_preserves_direct_transitive_removed_availability_and_reasons() {
    let previous = dependency_graph("Target", true, false);
    let current = dependency_graph("RenamedTarget", false, true);
    let report = evaluate(
        &[(id("configuration.main"), previous)],
        &[(id("configuration.main"), current)],
    );
    let configuration = &report.configurations()[0];
    let affected = configuration.result().affected_nodes();
    let target = affected
        .iter()
        .find(|node| node.node_id().as_str() == "function.target")
        .expect("changed target must be present");
    let caller = affected
        .iter()
        .find(|node| node.node_id().as_str() == "procedure.caller")
        .expect("transitive caller must be present");
    let removed = affected
        .iter()
        .find(|node| node.node_id().as_str() == "function.removed")
        .expect("removed node must be present");
    let added = affected
        .iter()
        .find(|node| node.node_id().as_str() == "function.added")
        .expect("added node must be present");

    assert_eq!(target.status(), ImpactNodeStatus::DirectlyChanged);
    assert_eq!(target.availability(), ImpactNodeAvailability::Both);
    assert!(
        target
            .reasons()
            .iter()
            .any(|reason| reason.kind() == ImpactReasonKind::NodeModified)
    );
    assert_eq!(caller.status(), ImpactNodeStatus::TransitivelyAffected);
    assert_eq!(caller.availability(), ImpactNodeAvailability::Both);
    assert!(caller.reasons().iter().any(|reason| {
        reason.kind() == ImpactReasonKind::DependencyPropagation
            && reason.edge_kind() == Some(EdgeKind::Calls)
    }));
    assert_eq!(removed.status(), ImpactNodeStatus::Removed);
    assert_eq!(removed.availability(), ImpactNodeAvailability::PreviousOnly);
    assert_eq!(added.status(), ImpactNodeStatus::DirectlyChanged);
    assert_eq!(added.availability(), ImpactNodeAvailability::CurrentOnly);
    assert_eq!(configuration.summary().directly_changed_nodes(), 2);
    assert_eq!(configuration.summary().transitively_affected_nodes(), 1);
    assert_eq!(configuration.summary().removed_nodes(), 1);
    assert_eq!(configuration.summary().previous_only_nodes(), 1);
    assert_eq!(configuration.summary().current_nodes(), 3);
    assert_eq!(configuration.summary().total_affected_nodes(), 4);
    assert_eq!(report.summary().total_affected_nodes(), 4);
}

#[test]
fn added_and_removed_configurations_use_empty_graph_endpoints_and_canonical_order() {
    let mut previous_graph = SemanticGraph::new();
    add_node(
        &mut previous_graph,
        "function.previous",
        "Previous",
        NodeKind::Function,
    );
    let mut current_graph = SemanticGraph::new();
    add_node(
        &mut current_graph,
        "function.current",
        "Current",
        NodeKind::Function,
    );
    let report = evaluate(
        &[(id("configuration.z_removed"), previous_graph)],
        &[(id("configuration.a_added"), current_graph)],
    );

    assert_eq!(
        report
            .configurations()
            .iter()
            .map(|item| (item.configuration_id().as_str(), item.kind()))
            .collect::<Vec<_>>(),
        vec![
            ("configuration.a_added", ConfigurationImpactKind::Added),
            ("configuration.z_removed", ConfigurationImpactKind::Removed),
        ]
    );
    let added = report
        .configuration(&id("configuration.a_added"))
        .expect("added Configuration must be queryable");
    let removed = report
        .configuration(&id("configuration.z_removed"))
        .expect("removed Configuration must be queryable");
    assert_eq!(
        added.result().affected_nodes()[0].availability(),
        ImpactNodeAvailability::CurrentOnly
    );
    assert_eq!(
        removed.result().affected_nodes()[0].availability(),
        ImpactNodeAvailability::PreviousOnly
    );
    assert_eq!(report.summary().added_configurations(), 1);
    assert_eq!(report.summary().removed_configurations(), 1);
}

#[test]
fn exact_duplicates_reordering_and_repetition_produce_one_equal_transition() {
    let previous_a = dependency_graph("Before", false, false);
    let previous_b = previous_a.clone();
    let current_a = dependency_graph("After", false, false);
    let current_b = current_a.clone();
    let configuration_id = id("configuration.main");
    let first_previous = [
        ChangeImpactConfiguration::new(&configuration_id, &previous_a),
        ChangeImpactConfiguration::new(&configuration_id, &previous_b),
    ];
    let first_current = [
        ChangeImpactConfiguration::new(&configuration_id, &current_a),
        ChangeImpactConfiguration::new(&configuration_id, &current_b),
    ];
    let second_previous = [
        ChangeImpactConfiguration::new(&configuration_id, &previous_b),
        ChangeImpactConfiguration::new(&configuration_id, &previous_a),
    ];
    let second_current = [
        ChangeImpactConfiguration::new(&configuration_id, &current_b),
        ChangeImpactConfiguration::new(&configuration_id, &current_a),
    ];

    let build = |previous: &[ChangeImpactConfiguration<'_>],
                 current: &[ChangeImpactConfiguration<'_>]| {
        ChangeImpactEvaluator
            .evaluate(
                ChangeImpactPublicationId::initial(),
                previous,
                current,
                &NeverCancelledChangeImpact,
            )
            .expect("duplicate report must build")
    };
    let first = build(&first_previous, &first_current);
    let repeated = build(&first_previous, &first_current);
    let reordered = build(&second_previous, &second_current);

    assert_eq!(first, repeated);
    assert_eq!(first, reordered);
    assert_eq!(first.configurations().len(), 1);
}

#[test]
fn conflicting_configuration_fails_closed_without_echoing_sensitive_values() {
    let first = dependency_graph("First", false, false);
    let second = dependency_graph("Second", false, false);
    let sensitive_id = id("/secret/repository/configuration.main");
    let inputs = [
        ChangeImpactConfiguration::new(&sensitive_id, &first),
        ChangeImpactConfiguration::new(&sensitive_id, &second),
    ];

    let error = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &inputs,
            &[],
            &NeverCancelledChangeImpact,
        )
        .expect_err("conflicting content must fail");
    let display = error.to_string();

    assert_eq!(
        error.kind(),
        ChangeImpactErrorKind::ConflictingConfiguration
    );
    assert!(!display.contains("secret"));
    assert!(!display.contains("repository"));
    assert!(!display.contains("configuration.main"));
}

#[test]
fn identifier_bound_accepts_exact_and_rejects_one_over_before_report_cloning() {
    let exact_id = id("a".repeat(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES));
    let over_id = id("b".repeat(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES + 1));
    let graph = SemanticGraph::new();
    let exact = [ChangeImpactConfiguration::new(&exact_id, &graph)];
    let over = [ChangeImpactConfiguration::new(&over_id, &graph)];

    assert!(
        ChangeImpactEvaluator
            .evaluate(
                ChangeImpactPublicationId::initial(),
                &exact,
                &[],
                &NeverCancelledChangeImpact,
            )
            .is_ok()
    );
    let error = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &over,
            &[],
            &NeverCancelledChangeImpact,
        )
        .expect_err("one-over identifier must fail");
    assert_eq!(error.kind(), ChangeImpactErrorKind::IdentifierTooLarge);
    assert_eq!(error.actual(), Some(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES + 1));
    assert_eq!(error.maximum(), Some(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES));

    let mut over_graph = SemanticGraph::new();
    over_graph.insert_node(GraphNode::new(
        over_id,
        name("OverBoundNode"),
        NodeKind::Function,
    ));
    let configuration_id = id("configuration.main");
    let over_node = [ChangeImpactConfiguration::new(
        &configuration_id,
        &over_graph,
    )];
    let node_error = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &[],
            &over_node,
            &NeverCancelledChangeImpact,
        )
        .expect_err("one-over node identifier must fail");
    assert_eq!(node_error.kind(), ChangeImpactErrorKind::IdentifierTooLarge);
}

#[test]
fn canonical_edge_identifier_bound_applies_to_equal_graphs_without_reasons() {
    let configuration_id = id("configuration.main");
    let build = |target_bytes: usize| {
        let source = id("s".repeat(2_027));
        let target = id("t".repeat(target_bytes));
        let edge_id = SemanticGraphQuery::edge_id(
            &NodeId::new(source.as_str()),
            &NodeId::new(target.as_str()),
            EdgeKind::Calls,
        );
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            source.clone(),
            name("BoundedSource"),
            NodeKind::Procedure,
        ));
        graph.insert_node(GraphNode::new(
            target.clone(),
            name("BoundedTarget"),
            NodeKind::Function,
        ));
        graph
            .insert_edge(GraphEdge::new(source, target, EdgeKind::Calls))
            .expect("bounded edge endpoints must exist");
        (graph, edge_id)
    };

    let (exact_graph, exact_edge_id) = build(2_028);
    assert_eq!(
        exact_edge_id.as_str().len(),
        MAX_CHANGE_IMPACT_IDENTIFIER_BYTES
    );
    let exact = [ChangeImpactConfiguration::new(
        &configuration_id,
        &exact_graph,
    )];
    let report = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &exact,
            &exact,
            &NeverCancelledChangeImpact,
        )
        .expect("exact canonical edge identifier bound must be admitted");
    assert_eq!(report.summary().total_affected_nodes(), 0);

    let (over_graph, over_edge_id) = build(2_029);
    assert_eq!(
        over_edge_id.as_str().len(),
        MAX_CHANGE_IMPACT_IDENTIFIER_BYTES + 1
    );
    let over = [ChangeImpactConfiguration::new(
        &configuration_id,
        &over_graph,
    )];
    let error = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &over,
            &over,
            &NeverCancelledChangeImpact,
        )
        .expect_err("one-over canonical edge identifier must fail without report reasons");
    assert_eq!(error.kind(), ChangeImpactErrorKind::IdentifierTooLarge);
    assert_eq!(error.actual(), Some(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES + 1));
    assert_eq!(error.maximum(), Some(MAX_CHANGE_IMPACT_IDENTIFIER_BYTES));
}

#[test]
fn configuration_bound_accepts_exact_and_rejects_one_over_unique_identities() {
    let graph = SemanticGraph::new();
    let identifiers = (0..=MAX_CHANGE_IMPACT_CONFIGURATIONS)
        .map(|index| id(format!("configuration.{index:04}")))
        .collect::<Vec<_>>();
    let exact = identifiers[..MAX_CHANGE_IMPACT_CONFIGURATIONS]
        .iter()
        .map(|id| ChangeImpactConfiguration::new(id, &graph))
        .collect::<Vec<_>>();
    let over = identifiers
        .iter()
        .map(|id| ChangeImpactConfiguration::new(id, &graph))
        .collect::<Vec<_>>();

    let exact_report = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &exact,
            &[],
            &NeverCancelledChangeImpact,
        )
        .expect("exact Configuration bound must build");
    assert_eq!(
        exact_report.summary().total_configurations(),
        MAX_CHANGE_IMPACT_CONFIGURATIONS
    );

    let error = ChangeImpactEvaluator
        .evaluate(
            ChangeImpactPublicationId::initial(),
            &over,
            &[],
            &NeverCancelledChangeImpact,
        )
        .expect_err("one-over Configuration bound must fail");
    assert_eq!(error.kind(), ChangeImpactErrorKind::TooManyConfigurations);
    assert_eq!(error.actual(), Some(MAX_CHANGE_IMPACT_CONFIGURATIONS + 1));
}

struct Cancellation(AtomicBool);

impl ChangeImpactCancellationSignal for Cancellation {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[test]
fn cancellation_and_publication_overflow_fail_without_partial_report() {
    let cancelled = Cancellation(AtomicBool::new(true));
    let cancellation_error = ChangeImpactEvaluator
        .evaluate(ChangeImpactPublicationId::initial(), &[], &[], &cancelled)
        .expect_err("cancelled evaluation must fail");
    assert_eq!(cancellation_error.kind(), ChangeImpactErrorKind::Cancelled);

    let maximum = ChangeImpactPublicationId::new(u64::MAX).expect("maximum is non-zero");
    let overflow = ChangeImpactEvaluator
        .evaluate(maximum, &[], &[], &NeverCancelledChangeImpact)
        .expect_err("publication successor must overflow");
    assert_eq!(overflow.kind(), ChangeImpactErrorKind::SummaryOverflow);
}
