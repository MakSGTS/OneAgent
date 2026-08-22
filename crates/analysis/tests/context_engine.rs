use std::collections::BTreeSet;

use oneagent_analysis::context::{
    ContextEngine, ContextError, ContextInclusionReason, ContextIntent, ContextPolicy,
    ContextPolicyField, ContextRelationDirection, ContextRequest, ContextSeed,
    ContextTraversalDirection, MAX_CONTEXT_BUDGET_BYTES, MAX_CONTEXT_CANDIDATES, MAX_CONTEXT_DEPTH,
    MAX_CONTEXT_SEEDS,
};
use oneagent_analysis::{AnalysisModule, SemanticAnalysisPipeline};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, ProducerId,
    Provenance, ResolutionState, SemanticGraph, SemanticGraphQuery,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: Option<&str>, producer: &str) -> Provenance {
    Provenance::new(
        source.map(id),
        ProducerId::new(producer),
        FactOrigin::Resolved,
        Confidence::High,
        ResolutionState::Resolved,
    )
}

fn node(node_id: &str, node_name: &str, kind: NodeKind) -> GraphNode {
    GraphNode::new(id(node_id), name(node_name), kind)
}

fn graph(
    nodes: &[(&str, &str, NodeKind)],
    edges: &[(&str, &str, EdgeKind)],
    reverse: bool,
) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    let node_iter: Box<dyn Iterator<Item = &(&str, &str, NodeKind)>> = if reverse {
        Box::new(nodes.iter().rev())
    } else {
        Box::new(nodes.iter())
    };
    for (node_id, node_name, kind) in node_iter {
        graph.insert_node(node(node_id, node_name, *kind));
    }

    let edge_iter: Box<dyn Iterator<Item = &(&str, &str, EdgeKind)>> = if reverse {
        Box::new(edges.iter().rev())
    } else {
        Box::new(edges.iter())
    };
    for (source, target, kind) in edge_iter {
        graph
            .insert_edge(GraphEdge::new(id(source), id(target), *kind))
            .expect("edge endpoints must exist");
    }
    graph
}

fn policy(
    direction: ContextTraversalDirection,
    edge_kinds: impl IntoIterator<Item = EdgeKind>,
    node_kinds: Option<BTreeSet<NodeKind>>,
    max_depth: usize,
    max_candidates: usize,
) -> ContextPolicy {
    ContextPolicy::new(
        direction,
        edge_kinds.into_iter().collect(),
        node_kinds,
        max_depth,
        max_candidates,
    )
}

fn request(seeds: Vec<ContextSeed>, budget: usize, policy: ContextPolicy) -> ContextRequest {
    ContextRequest::new(ContextIntent::Explain, seeds, budget, policy)
        .expect("request must be valid")
}

#[test]
fn public_request_validation_and_precedence_are_exact() {
    let invalid_policy = policy(
        ContextTraversalDirection::Both,
        [],
        Some(BTreeSet::new()),
        MAX_CONTEXT_DEPTH + 1,
        0,
    );
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            Vec::new(),
            0,
            invalid_policy.clone()
        ),
        Err(ContextError::InvalidBudget {
            value: 0,
            minimum: 1,
            maximum: MAX_CONTEXT_BUDGET_BYTES,
        })
    );
    assert!(matches!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed")],
            MAX_CONTEXT_BUDGET_BYTES + 1,
            ContextPolicy::default(),
        ),
        Err(ContextError::InvalidBudget { .. })
    ));
    assert_eq!(
        ContextRequest::new(ContextIntent::Explain, Vec::new(), 1, invalid_policy),
        Err(ContextError::InvalidPolicy {
            field: ContextPolicyField::MaxDepth,
            value: MAX_CONTEXT_DEPTH + 1,
            minimum: 0,
            maximum: MAX_CONTEXT_DEPTH,
        })
    );
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed")],
            1,
            policy(
                ContextTraversalDirection::Both,
                [EdgeKind::Calls],
                None,
                0,
                0
            ),
        ),
        Err(ContextError::InvalidPolicy {
            field: ContextPolicyField::MaxCandidates,
            value: 0,
            minimum: 1,
            maximum: MAX_CONTEXT_CANDIDATES,
        })
    );
    assert!(matches!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed")],
            1,
            policy(
                ContextTraversalDirection::Both,
                [EdgeKind::Calls],
                None,
                0,
                MAX_CONTEXT_CANDIDATES + 1,
            ),
        ),
        Err(ContextError::InvalidPolicy {
            field: ContextPolicyField::MaxCandidates,
            ..
        })
    ));
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed")],
            1,
            policy(ContextTraversalDirection::Both, [], None, 0, 1),
        ),
        Err(ContextError::EmptyEdgeKinds)
    );
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed")],
            1,
            policy(
                ContextTraversalDirection::Both,
                [EdgeKind::Calls],
                Some(BTreeSet::new()),
                0,
                1,
            ),
        ),
        Err(ContextError::EmptyNodeKinds)
    );
}

#[test]
fn public_request_seed_validation_is_exact() {
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            Vec::new(),
            1,
            ContextPolicy::default(),
        ),
        Err(ContextError::InvalidSeedCount {
            value: 0,
            minimum: 1,
            maximum: MAX_CONTEXT_SEEDS,
        })
    );
    assert!(matches!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("seed"); MAX_CONTEXT_SEEDS + 1],
            1,
            ContextPolicy::default(),
        ),
        Err(ContextError::InvalidSeedCount { .. })
    ));
    assert_eq!(
        ContextRequest::new(
            ContextIntent::Explain,
            vec![ContextSeed::node("  ")],
            1,
            ContextPolicy::default(),
        ),
        Err(ContextError::InvalidSeedIdentifier {
            value: "  ".to_owned(),
        })
    );
}

#[test]
fn public_request_accepted_bounds_and_defaults_are_exact() {
    let minimum = request(
        vec![ContextSeed::node("seed")],
        1,
        policy(
            ContextTraversalDirection::Outgoing,
            [EdgeKind::Calls],
            None,
            0,
            1,
        ),
    );
    assert_eq!(minimum.budget().bytes(), 1);
    assert_eq!(minimum.policy().max_depth(), 0);
    assert_eq!(minimum.policy().max_candidates(), 1);

    let maximum_seeds = (0..MAX_CONTEXT_SEEDS)
        .map(|index| ContextSeed::node(format!("seed.{index}")))
        .collect();
    let maximum = request(
        maximum_seeds,
        MAX_CONTEXT_BUDGET_BYTES,
        policy(
            ContextTraversalDirection::Incoming,
            [EdgeKind::Calls],
            None,
            MAX_CONTEXT_DEPTH,
            MAX_CONTEXT_CANDIDATES,
        ),
    );
    assert_eq!(maximum.seeds().len(), MAX_CONTEXT_SEEDS);
    assert_eq!(maximum.budget().bytes(), MAX_CONTEXT_BUDGET_BYTES);

    let defaults = ContextPolicy::default();
    assert_eq!(defaults.direction(), ContextTraversalDirection::Both);
    assert_eq!(defaults.max_depth(), 2);
    assert_eq!(defaults.max_candidates(), 32);
    assert_eq!(defaults.edge_kinds().len(), 11);
    assert!(defaults.node_kinds().is_none());
}

#[test]
fn public_seed_resolution_distinguishes_every_outcome_without_partial_state() {
    let graph = graph(
        &[
            ("node.a", "Shared", NodeKind::Procedure),
            ("node.b", "Shared", NodeKind::Function),
            ("node.c", "Unique", NodeKind::Module),
        ],
        &[],
        false,
    );
    let engine = ContextEngine;

    let resolved = engine
        .resolve_request(
            &graph,
            &request(
                vec![
                    ContextSeed::node_with_kind("node.c", NodeKind::Module),
                    ContextSeed::exact_name_with_kind(name("Shared"), NodeKind::Function),
                    ContextSeed::node("node.c"),
                ],
                4_096,
                ContextPolicy::default(),
            ),
        )
        .expect("seeds must resolve");
    assert_eq!(
        resolved.seeds(),
        &[NodeId::new("node.b"), NodeId::new("node.c")]
    );

    let missing = request(
        vec![ContextSeed::exact_name(name("Missing"))],
        4_096,
        ContextPolicy::default(),
    );
    assert!(matches!(
        engine.resolve_request(&graph, &missing),
        Err(ContextError::MissingSeed { .. })
    ));

    let ambiguous = request(
        vec![ContextSeed::exact_name(name("Shared"))],
        4_096,
        ContextPolicy::default(),
    );
    assert_eq!(
        engine.resolve_request(&graph, &ambiguous),
        Err(ContextError::AmbiguousSeed {
            seed: ContextSeed::exact_name(name("Shared")),
            candidates: vec![NodeId::new("node.a"), NodeId::new("node.b")],
        })
    );

    let incompatible = request(
        vec![ContextSeed::node_with_kind("node.a", NodeKind::Function)],
        4_096,
        ContextPolicy::default(),
    );
    assert!(matches!(
        engine.resolve_request(&graph, &incompatible),
        Err(ContextError::IncompatibleSeed { .. })
    ));

    let policy_incompatible = request(
        vec![ContextSeed::node("node.a")],
        4_096,
        policy(
            ContextTraversalDirection::Both,
            [EdgeKind::Calls],
            Some(BTreeSet::from([NodeKind::Function])),
            1,
            2,
        ),
    );
    assert!(matches!(
        engine.resolve_request(&graph, &policy_incompatible),
        Err(ContextError::IncompatibleSeed { .. })
    ));

    let too_many = request(
        vec![ContextSeed::node("node.a"), ContextSeed::node("node.b")],
        4_096,
        policy(
            ContextTraversalDirection::Both,
            [EdgeKind::Calls],
            None,
            1,
            1,
        ),
    );
    assert_eq!(
        engine.resolve_request(&graph, &too_many),
        Err(ContextError::TooManyUniqueSeeds {
            value: 2,
            maximum: 1,
        })
    );
}

#[test]
fn public_selection_preserves_an_empty_neighborhood_seed() {
    let graph = graph(&[("seed", "Seed", NodeKind::Procedure)], &[], false);
    let selection = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                4_096,
                ContextPolicy::default(),
            ),
        )
        .expect("empty neighborhood must select its seed");

    assert_eq!(selection.candidates().len(), 1);
    assert_eq!(
        selection.candidates()[0].reason(),
        ContextInclusionReason::Seed
    );
}

#[test]
fn public_selection_orders_every_accepted_edge_kind() {
    let priorities = [
        EdgeKind::Contains,
        EdgeKind::Calls,
        EdgeKind::References,
        EdgeKind::Reads,
        EdgeKind::Writes,
        EdgeKind::DependsOn,
        EdgeKind::Opens,
        EdgeKind::Triggers,
        EdgeKind::Includes,
        EdgeKind::Extends,
        EdgeKind::Grants,
    ];
    let mut graph = SemanticGraph::new();
    graph.insert_node(node("seed", "Seed", NodeKind::Procedure));
    for (index, kind) in priorities.into_iter().enumerate() {
        let target = format!("target.{index:02}");
        graph.insert_node(node(&target, &target, NodeKind::Function));
        graph
            .insert_edge(GraphEdge::new(id("seed"), id(&target), kind))
            .expect("edge endpoints must exist");
    }
    let outgoing = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                65_536,
                policy(ContextTraversalDirection::Outgoing, priorities, None, 1, 12),
            ),
        )
        .expect("selection must succeed");
    assert_eq!(
        outgoing
            .candidates()
            .iter()
            .skip(1)
            .map(|candidate| candidate.path()[0].edge_kind())
            .collect::<Vec<_>>(),
        priorities
    );
}

#[test]
fn public_selection_covers_directions_depth_filters_and_limits() {
    let graph = graph(
        &[
            ("seed", "Seed", NodeKind::Procedure),
            ("incoming", "Incoming", NodeKind::Function),
            ("outgoing", "Outgoing", NodeKind::Function),
            ("filtered", "Filtered", NodeKind::Module),
            ("edge.filtered", "Edge Filtered", NodeKind::Function),
        ],
        &[
            ("incoming", "seed", EdgeKind::Calls),
            ("seed", "outgoing", EdgeKind::Calls),
            ("seed", "filtered", EdgeKind::Calls),
            ("seed", "edge.filtered", EdgeKind::Contains),
        ],
        false,
    );
    let incoming = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                65_536,
                policy(
                    ContextTraversalDirection::Incoming,
                    [EdgeKind::Calls],
                    None,
                    1,
                    2,
                ),
            ),
        )
        .expect("selection must succeed");
    assert_eq!(incoming.candidates()[1].node_id().as_str(), "incoming");
    assert_eq!(
        incoming.candidates()[1].path()[0].direction(),
        ContextRelationDirection::Incoming
    );

    let both = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                65_536,
                policy(
                    ContextTraversalDirection::Both,
                    [EdgeKind::Calls],
                    Some(BTreeSet::from([NodeKind::Procedure, NodeKind::Function])),
                    1,
                    3,
                ),
            ),
        )
        .expect("selection must succeed");
    assert_eq!(
        both.candidates()
            .iter()
            .map(|candidate| candidate.node_id().as_str())
            .collect::<Vec<_>>(),
        vec!["seed", "outgoing", "incoming"]
    );

    let zero_depth = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                65_536,
                policy(
                    ContextTraversalDirection::Both,
                    [EdgeKind::Calls],
                    Some(BTreeSet::from([NodeKind::Procedure, NodeKind::Function])),
                    0,
                    1,
                ),
            ),
        )
        .expect("selection must succeed");
    assert_eq!(zero_depth.candidates().len(), 1);

    let limited = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                65_536,
                policy(
                    ContextTraversalDirection::Outgoing,
                    [EdgeKind::Calls],
                    Some(BTreeSet::from([NodeKind::Procedure, NodeKind::Function])),
                    MAX_CONTEXT_DEPTH,
                    1,
                ),
            ),
        )
        .expect("selection must succeed");
    assert_eq!(limited.candidates().len(), 1);
    assert_eq!(limited.candidate_omitted(), 1);
}

#[test]
fn public_selection_reaches_maximum_depth_and_stops() {
    let nodes = [
        ("node.0", "Node 0", NodeKind::Procedure),
        ("node.1", "Node 1", NodeKind::Procedure),
        ("node.2", "Node 2", NodeKind::Procedure),
        ("node.3", "Node 3", NodeKind::Procedure),
        ("node.4", "Node 4", NodeKind::Procedure),
        ("node.5", "Node 5", NodeKind::Procedure),
    ];
    let edges = [
        ("node.0", "node.1", EdgeKind::Calls),
        ("node.1", "node.2", EdgeKind::Calls),
        ("node.2", "node.3", EdgeKind::Calls),
        ("node.3", "node.4", EdgeKind::Calls),
        ("node.4", "node.5", EdgeKind::Calls),
    ];
    let graph = graph(&nodes, &edges, false);
    let selection = ContextEngine
        .select_candidates(
            &graph,
            &request(
                vec![ContextSeed::node("node.0")],
                65_536,
                policy(
                    ContextTraversalDirection::Outgoing,
                    [EdgeKind::Calls],
                    None,
                    MAX_CONTEXT_DEPTH,
                    6,
                ),
            ),
        )
        .expect("selection must succeed");

    assert_eq!(selection.candidates().len(), 5);
    assert_eq!(selection.candidates()[4].node_id().as_str(), "node.4");
    assert_eq!(selection.candidates()[4].depth(), MAX_CONTEXT_DEPTH);
}

#[test]
fn public_selection_is_cycle_safe_provenance_backed_and_reorder_invariant() {
    fn fixture(reverse: bool) -> SemanticGraph {
        let early = provenance(Some("source.a"), "producer.a");
        let late = provenance(Some("source.z"), "producer.z");
        let mut graph = SemanticGraph::new();
        let mut nodes = vec![
            GraphNode::new_with_provenance(
                id("seed.a"),
                name("Seed A"),
                NodeKind::Procedure,
                vec![late.clone(), early.clone(), early.clone()],
            ),
            node("seed.b", "Seed B", NodeKind::Procedure),
            node("middle", "Middle", NodeKind::Function),
            node("target", "Target", NodeKind::Function),
        ];
        let mut edges = vec![
            GraphEdge::new_with_provenance(
                id("seed.a"),
                id("middle"),
                EdgeKind::Calls,
                vec![late.clone(), early.clone(), early.clone()],
            ),
            GraphEdge::new(id("seed.b"), id("middle"), EdgeKind::Calls),
            GraphEdge::new(id("middle"), id("target"), EdgeKind::Calls),
            GraphEdge::new(id("target"), id("seed.a"), EdgeKind::Calls),
        ];
        if reverse {
            nodes.reverse();
            edges.reverse();
        }
        for node in nodes {
            graph.insert_node(node);
        }
        for edge in edges {
            graph.insert_edge(edge).expect("edge endpoints must exist");
        }
        graph
    }

    let first_request = request(
        vec![ContextSeed::node("seed.b"), ContextSeed::node("seed.a")],
        65_536,
        policy(
            ContextTraversalDirection::Outgoing,
            [EdgeKind::Calls],
            None,
            4,
            4,
        ),
    );
    let second_request = request(
        vec![ContextSeed::node("seed.a"), ContextSeed::node("seed.b")],
        65_536,
        policy(
            ContextTraversalDirection::Outgoing,
            [EdgeKind::Calls],
            None,
            4,
            4,
        ),
    );
    let first_graph = fixture(false);
    let second_graph = fixture(true);

    let first = ContextEngine
        .select_candidates(&first_graph, &first_request)
        .expect("selection must succeed");
    let repeated = ContextEngine
        .select_candidates(&first_graph, &first_request)
        .expect("selection must repeat");
    let reordered = ContextEngine
        .select_candidates(&second_graph, &second_request)
        .expect("reordered selection must succeed");

    assert_eq!(first, repeated);
    assert_eq!(first, reordered);
    assert_eq!(first.candidates().len(), 4);
    assert_eq!(first.candidates()[2].node_id().as_str(), "middle");
    assert_eq!(first.candidates()[2].seed_id().as_str(), "seed.a");
    assert_eq!(first.candidates()[3].node_id().as_str(), "target");
    assert_eq!(first.candidates()[3].depth(), 2);
    assert_eq!(
        first.candidates()[0]
            .provenance()
            .iter()
            .map(|value| value.producer().as_str())
            .collect::<Vec<_>>(),
        vec!["producer.a", "producer.z"]
    );
    assert_eq!(
        first.candidates()[2].path()[0]
            .provenance()
            .iter()
            .map(|value| value.producer().as_str())
            .collect::<Vec<_>>(),
        vec!["producer.a", "producer.z"]
    );
}

#[test]
fn public_bundle_rendering_budget_and_omission_contract_is_exact() {
    let graph = graph(
        &[
            ("seed", "Узел", NodeKind::Procedure),
            ("a", "A", NodeKind::Function),
            ("b", "B", NodeKind::Function),
        ],
        &[
            ("seed", "a", EdgeKind::Calls),
            ("seed", "b", EdgeKind::Calls),
        ],
        false,
    );
    let full_request = request(
        vec![ContextSeed::node("seed")],
        65_536,
        policy(
            ContextTraversalDirection::Outgoing,
            [EdgeKind::Calls],
            None,
            1,
            3,
        ),
    );
    let full = ContextEngine
        .build(&graph, &full_request)
        .expect("bundle must build");
    let seed_fragment = "node kind=procedure id=4:seed name=8:Узел\nreason seed=4:seed depth=0\n";
    let edge_id =
        SemanticGraphQuery::edge_id(&NodeId::new("seed"), &NodeId::new("a"), EdgeKind::Calls);
    let related_fragment = format!(
        "node kind=function id=1:a name=1:A\nreason seed=4:seed depth=1 path=1:outgoing,calls,{}:{}\n",
        edge_id.as_str().len(),
        edge_id
    );
    assert_eq!(full.items()[0].fragment(), seed_fragment);
    assert_eq!(full.items()[1].fragment(), related_fragment);
    assert_eq!(full.rendered().len(), full.used_bytes());
    assert_eq!(full.remaining_bytes(), 65_536 - full.used_bytes());
    assert_eq!(full.items()[0].reason(), ContextInclusionReason::Seed);
    assert_eq!(full.items()[1].reason(), ContextInclusionReason::Related);

    let required_seed = full.items()[0].cost_bytes();
    let one_byte_short = request(
        vec![ContextSeed::node("seed")],
        required_seed - 1,
        policy(
            ContextTraversalDirection::Outgoing,
            [EdgeKind::Calls],
            None,
            1,
            3,
        ),
    );
    assert_eq!(
        ContextEngine.build(&graph, &one_byte_short),
        Err(ContextError::InsufficientBudget {
            required: required_seed,
            available: required_seed - 1,
        })
    );

    let prefix_budget = required_seed + full.items()[1].cost_bytes();
    let prefix = ContextEngine
        .build(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                prefix_budget,
                policy(
                    ContextTraversalDirection::Outgoing,
                    [EdgeKind::Calls],
                    None,
                    1,
                    3,
                ),
            ),
        )
        .expect("prefix bundle must build");
    assert_eq!(prefix.items().len(), 2);
    assert_eq!(prefix.used_bytes(), prefix_budget);
    assert_eq!(prefix.remaining_bytes(), 0);
    assert_eq!(prefix.budget_omitted(), 1);
    assert_eq!(prefix.candidate_omitted(), 0);

    let both_limits = ContextEngine
        .build(
            &graph,
            &request(
                vec![ContextSeed::node("seed")],
                required_seed,
                policy(
                    ContextTraversalDirection::Outgoing,
                    [EdgeKind::Calls],
                    None,
                    1,
                    2,
                ),
            ),
        )
        .expect("limited bundle must build");
    assert_eq!(both_limits.items().len(), 1);
    assert_eq!(both_limits.candidate_omitted(), 1);
    assert_eq!(both_limits.budget_omitted(), 1);
}

#[test]
fn public_context_engine_consumes_production_analysis_facts_without_source_text() {
    let module = AnalysisModule::new(
        id("module.sales"),
        name("Sales"),
        "Procedure Post()\n    FillMovements();\nEndProcedure\n\nProcedure FillMovements()\nEndProcedure\n",
    );
    let analysis = SemanticAnalysisPipeline
        .analyze(&[module])
        .expect("analysis must succeed");
    let request = request(
        vec![ContextSeed::node("module.sales:procedure:Post")],
        65_536,
        policy(
            ContextTraversalDirection::Both,
            [EdgeKind::Contains, EdgeKind::Calls],
            None,
            1,
            3,
        ),
    );

    let first = ContextEngine
        .build(analysis.graph(), &request)
        .expect("bundle must build");
    let repeated = ContextEngine
        .build(analysis.graph(), &request)
        .expect("bundle must repeat");

    assert_eq!(first, repeated);
    assert_eq!(
        first
            .items()
            .iter()
            .map(|item| item.node_id().as_str())
            .collect::<Vec<_>>(),
        vec![
            "module.sales:procedure:Post",
            "module.sales",
            "module.sales:procedure:FillMovements",
        ]
    );
    assert!(
        first
            .items()
            .iter()
            .all(|item| !item.provenance().is_empty())
    );
    assert!(
        first.items()[1]
            .path()
            .iter()
            .all(|step| !step.provenance().is_empty())
    );
    assert!(!first.rendered().contains("FillMovements();"));
    assert!(!first.rendered().contains("EndProcedure"));
}
