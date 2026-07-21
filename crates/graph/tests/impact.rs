use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, ImpactAnalysisError,
    ImpactNodeAvailability, ImpactNodeStatus, ImpactReasonKind, ImpactSnapshot, NodeId, NodeKind,
    OwnershipImpactMode, ProducerId, Provenance, ProvenanceImpactMode, ResolutionState,
    SemanticGraph, SemanticGraphEdgeFilter, SemanticImpactAnalyzer, SemanticImpactOptions,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: &str) -> Provenance {
    Provenance::new(
        Some(id(source)),
        ProducerId::new("oneagent.graph.impact.tests"),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn add_node(graph: &mut SemanticGraph, id_value: &str, name_value: &str, kind: NodeKind) {
    graph.insert_node(GraphNode::new(id(id_value), name(name_value), kind));
}

fn add_node_with_provenance(
    graph: &mut SemanticGraph,
    id_value: &str,
    name_value: &str,
    source: &str,
) {
    graph.insert_node(GraphNode::new_with_provenance(
        id(id_value),
        name(name_value),
        NodeKind::Function,
        vec![provenance(source)],
    ));
}

fn add_edge(graph: &mut SemanticGraph, source: &str, target: &str, kind: EdgeKind) {
    graph
        .insert_edge(GraphEdge::new(id(source), id(target), kind))
        .expect("edge must be valid");
}

fn dependency_graph(callee_name: &str, reverse: bool) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    let nodes = [
        ("procedure.top", "Top", NodeKind::Procedure),
        ("procedure.caller", "Caller", NodeKind::Procedure),
        ("function.callee", callee_name, NodeKind::Function),
        ("metadata.document.sales", "Sales", NodeKind::Unknown),
        (
            "metadata.document.sales:attribute:Company",
            "Company",
            NodeKind::Attribute,
        ),
    ];

    if reverse {
        for (node, name, kind) in nodes.into_iter().rev() {
            add_node(&mut graph, node, name, kind);
        }
    } else {
        for (node, name, kind) in nodes {
            add_node(&mut graph, node, name, kind);
        }
    }

    let edges = [
        ("procedure.top", "procedure.caller", EdgeKind::Calls),
        ("procedure.caller", "function.callee", EdgeKind::Calls),
        (
            "metadata.document.sales",
            "metadata.document.sales:attribute:Company",
            EdgeKind::Contains,
        ),
    ];

    if reverse {
        for (source, target, kind) in edges.into_iter().rev() {
            add_edge(&mut graph, source, target, kind);
        }
    } else {
        for (source, target, kind) in edges {
            add_edge(&mut graph, source, target, kind);
        }
    }

    graph
}

#[test]
fn empty_diff_returns_empty_successful_result() {
    let previous = dependency_graph("Callee", false);
    let current = previous.clone();
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(2))
            .expect("empty impact must succeed");

    assert!(result.is_empty());
    assert_eq!(result.summary().total_affected_nodes(), 0);
    assert_eq!(result.summary().seed_node_changes(), 0);
    assert_eq!(result.summary().seed_edge_changes(), 0);
}

#[test]
fn modified_dependency_affects_direct_and_transitive_usages() {
    let previous = dependency_graph("Callee", false);
    let current = dependency_graph("CalleeRenamed", false);
    let diff = previous.diff(&current);
    let before_previous = previous.report();
    let before_current = current.report();

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(2))
            .expect("impact must succeed");

    let affected = result.affected_nodes();

    assert_eq!(previous.report(), before_previous);
    assert_eq!(current.report(), before_current);
    assert_eq!(
        affected
            .iter()
            .map(|node| (
                node.node_id().as_str().to_owned(),
                node.depth(),
                node.status()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "function.callee".to_owned(),
                0,
                ImpactNodeStatus::DirectlyChanged,
            ),
            (
                "procedure.caller".to_owned(),
                1,
                ImpactNodeStatus::TransitivelyAffected,
            ),
            (
                "procedure.top".to_owned(),
                2,
                ImpactNodeStatus::TransitivelyAffected,
            ),
        ]
    );
    assert_eq!(result.summary().seed_node_changes(), 1);
    assert_eq!(result.summary().directly_changed_nodes(), 1);
    assert_eq!(result.summary().transitively_affected_nodes(), 2);
    assert_eq!(result.summary().max_reached_depth(), 2);
    assert!(affected[1].reasons().iter().any(|reason| reason.kind()
        == ImpactReasonKind::DependencyPropagation
        && reason.edge_kind() == Some(EdgeKind::Calls)
        && reason.depth() == 1));
}

#[test]
fn result_is_repeatable_and_independent_from_insertion_order() {
    let previous = dependency_graph("Callee", false);
    let current = dependency_graph("CalleeRenamed", false);
    let previous_reversed = dependency_graph("Callee", true);
    let current_reversed = dependency_graph("CalleeRenamed", true);
    let diff = previous.diff(&current);
    let reversed_diff = previous_reversed.diff(&current_reversed);
    let options = SemanticImpactOptions::new(2);

    let first = SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &options)
        .expect("first impact must succeed");
    let second = SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &options)
        .expect("second impact must succeed");
    let reversed = SemanticImpactAnalyzer::analyze(
        &previous_reversed,
        &current_reversed,
        &reversed_diff,
        &options,
    )
    .expect("reversed impact must succeed");

    assert_eq!(first, second);
    assert_eq!(first, reversed);
    assert_eq!(diff, previous.diff(&current));
}

#[test]
fn maximum_depth_zero_returns_only_direct_seeds() {
    let previous = dependency_graph("Callee", false);
    let current = dependency_graph("CalleeRenamed", false);
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(0))
            .expect("impact must succeed");

    assert_eq!(result.affected_nodes().len(), 1);
    assert_eq!(
        result.affected_nodes()[0].node_id().as_str(),
        "function.callee"
    );
    assert_eq!(result.summary().max_reached_depth(), 0);
}

#[test]
fn added_and_removed_nodes_are_direct_seeds_in_expected_snapshots() {
    let previous = {
        let mut graph = SemanticGraph::new();
        add_node(&mut graph, "node.removed", "Removed", NodeKind::Function);
        graph
    };
    let current = {
        let mut graph = SemanticGraph::new();
        add_node(&mut graph, "node.added", "Added", NodeKind::Function);
        graph
    };
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("impact must succeed");

    assert_eq!(result.affected_nodes().len(), 2);
    assert_eq!(result.summary().removed_nodes(), 1);
    assert_eq!(result.summary().previous_only_nodes(), 1);
    assert_eq!(result.summary().current_nodes(), 1);
    assert!(result.affected_nodes().iter().any(|node| {
        node.node_id().as_str() == "node.removed"
            && node.status() == ImpactNodeStatus::Removed
            && node.availability() == ImpactNodeAvailability::PreviousOnly
            && node.primary_reason().snapshot() == ImpactSnapshot::Previous
    }));
    assert!(result.affected_nodes().iter().any(|node| {
        node.node_id().as_str() == "node.added"
            && node.status() == ImpactNodeStatus::DirectlyChanged
            && node.availability() == ImpactNodeAvailability::CurrentOnly
            && node.primary_reason().snapshot() == ImpactSnapshot::Current
    }));
}

#[test]
fn added_edge_creates_direct_endpoint_seeds() {
    let previous = {
        let mut graph = SemanticGraph::new();
        add_node(
            &mut graph,
            "procedure.caller",
            "Caller",
            NodeKind::Procedure,
        );
        add_node(&mut graph, "function.callee", "Callee", NodeKind::Function);
        graph
    };
    let current = {
        let mut graph = previous.clone();
        add_edge(
            &mut graph,
            "procedure.caller",
            "function.callee",
            EdgeKind::Calls,
        );
        graph
    };
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(0))
            .expect("impact must succeed");

    assert_eq!(result.summary().seed_edge_changes(), 1);
    assert_eq!(result.affected_nodes().len(), 2);
    assert!(result.affected_nodes().iter().all(|node| {
        node.status() == ImpactNodeStatus::DirectlyChanged
            && node
                .reasons()
                .iter()
                .any(|reason| reason.kind() == ImpactReasonKind::EdgeAdded)
    }));
}

#[test]
fn removed_edge_uses_previous_snapshot() {
    let previous = {
        let mut graph = SemanticGraph::new();
        add_node(
            &mut graph,
            "procedure.caller",
            "Caller",
            NodeKind::Procedure,
        );
        add_node(&mut graph, "function.callee", "Callee", NodeKind::Function);
        add_edge(
            &mut graph,
            "procedure.caller",
            "function.callee",
            EdgeKind::Calls,
        );
        graph
    };
    let current = {
        let mut graph = SemanticGraph::new();
        add_node(
            &mut graph,
            "procedure.caller",
            "Caller",
            NodeKind::Procedure,
        );
        add_node(&mut graph, "function.callee", "Callee", NodeKind::Function);
        graph
    };
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(0))
            .expect("impact must succeed");

    assert_eq!(result.summary().seed_edge_changes(), 1);
    assert!(result.affected_nodes().iter().all(|node| {
        node.primary_reason().kind() == ImpactReasonKind::EdgeRemoved
            && node.primary_reason().snapshot() == ImpactSnapshot::Previous
    }));
}

#[test]
fn provenance_only_changes_are_direct_only_by_default() {
    let mut previous = SemanticGraph::new();
    let mut current = SemanticGraph::new();

    add_node(
        &mut previous,
        "procedure.caller",
        "Caller",
        NodeKind::Procedure,
    );
    add_node(
        &mut current,
        "procedure.caller",
        "Caller",
        NodeKind::Procedure,
    );
    add_node_with_provenance(&mut previous, "function.callee", "Callee", "source.old");
    add_node_with_provenance(&mut current, "function.callee", "Callee", "source.new");
    add_edge(
        &mut previous,
        "procedure.caller",
        "function.callee",
        EdgeKind::Calls,
    );
    add_edge(
        &mut current,
        "procedure.caller",
        "function.callee",
        EdgeKind::Calls,
    );

    let diff = previous.diff(&current);

    let direct_only =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(2))
            .expect("direct-only impact must succeed");
    let propagated = SemanticImpactAnalyzer::analyze(
        &previous,
        &current,
        &diff,
        &SemanticImpactOptions::new(2).with_provenance_mode(ProvenanceImpactMode::Propagate),
    )
    .expect("propagated impact must succeed");

    assert_eq!(direct_only.affected_nodes().len(), 1);
    assert_eq!(
        direct_only.affected_nodes()[0].node_id().as_str(),
        "function.callee"
    );
    assert!(
        propagated
            .affected_nodes()
            .iter()
            .any(|node| node.node_id().as_str() == "procedure.caller" && node.depth() == 1)
    );
}

#[test]
fn edge_filter_limits_dependency_propagation() {
    let previous = dependency_graph("Callee", false);
    let current = dependency_graph("CalleeRenamed", false);
    let diff = previous.diff(&current);

    let result = SemanticImpactAnalyzer::analyze(
        &previous,
        &current,
        &diff,
        &SemanticImpactOptions::new(2)
            .with_edge_filter(SemanticGraphEdgeFilter::only(EdgeKind::References)),
    )
    .expect("impact must succeed");

    assert_eq!(result.affected_nodes().len(), 1);
    assert_eq!(
        result.affected_nodes()[0].node_id().as_str(),
        "function.callee"
    );
}

#[test]
fn ownership_propagation_is_explicitly_configured() {
    let previous = dependency_graph("Callee", false);
    let mut current = dependency_graph("Callee", false);

    current.insert_node(GraphNode::new(
        id("metadata.document.sales:attribute:Company"),
        name("CompanyRenamed"),
        NodeKind::Attribute,
    ));

    let diff = previous.diff(&current);
    let disabled =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("impact must succeed");
    let child_to_owner = SemanticImpactAnalyzer::analyze(
        &previous,
        &current,
        &diff,
        &SemanticImpactOptions::new(1).with_ownership_mode(OwnershipImpactMode::ChildToOwner),
    )
    .expect("impact must succeed");

    assert_eq!(disabled.affected_nodes().len(), 1);
    assert!(child_to_owner.affected_nodes().iter().any(|node| {
        node.node_id().as_str() == "metadata.document.sales"
            && node
                .reasons()
                .iter()
                .any(|reason| reason.kind() == ImpactReasonKind::OwnershipPropagation)
    }));
}

#[test]
fn dependency_cycle_and_self_loop_do_not_duplicate_nodes() {
    let previous = {
        let mut graph = SemanticGraph::new();
        add_node(&mut graph, "procedure.a", "A", NodeKind::Procedure);
        add_node(&mut graph, "procedure.b", "B", NodeKind::Procedure);
        add_edge(&mut graph, "procedure.a", "procedure.b", EdgeKind::Calls);
        add_edge(&mut graph, "procedure.b", "procedure.a", EdgeKind::Calls);
        add_edge(
            &mut graph,
            "procedure.a",
            "procedure.a",
            EdgeKind::DependsOn,
        );
        graph
    };
    let mut current = previous.clone();
    current.insert_node(GraphNode::new(
        id("procedure.b"),
        name("BRenamed"),
        NodeKind::Procedure,
    ));
    let diff = previous.diff(&current);

    let result =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(3))
            .expect("impact must succeed");

    assert_eq!(result.affected_nodes().len(), 2);
    assert_eq!(
        result
            .affected_nodes()
            .iter()
            .map(|node| node.node_id().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["procedure.a", "procedure.b"]
    );
}

#[test]
fn inconsistent_diff_returns_typed_error() {
    let previous = SemanticGraph::new();
    let current_with_added = {
        let mut graph = SemanticGraph::new();
        add_node(&mut graph, "node.added", "Added", NodeKind::Function);
        graph
    };
    let wrong_current = SemanticGraph::new();
    let diff = previous.diff(&current_with_added);

    let error = SemanticImpactAnalyzer::analyze(
        &previous,
        &wrong_current,
        &diff,
        &SemanticImpactOptions::new(1),
    )
    .expect_err("missing added seed must fail");

    assert_eq!(
        error,
        ImpactAnalysisError::MissingSeedNode {
            node: node_id("node.added"),
            snapshot: ImpactSnapshot::Current,
        }
    );
}
