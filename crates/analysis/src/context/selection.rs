use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use oneagent_graph::{
    EdgeId, EdgeKind, GraphEdge, NodeId, Provenance, SemanticGraph, SemanticGraphQuery,
};

use super::{
    ContextCandidate, ContextInclusionReason, ContextPathStep, ContextRelationDirection,
    ContextSelection, ContextTraversalDirection, ResolvedContextRequest,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StepKey {
    edge_priority: u8,
    direction: ContextRelationDirection,
    edge_id: EdgeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateKey {
    depth: usize,
    steps: Vec<StepKey>,
    seed_id: NodeId,
    node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateState {
    key: CandidateKey,
    path: Vec<ContextPathStep>,
}

struct Expansion {
    step_key: StepKey,
    step: ContextPathStep,
    node_id: NodeId,
}

pub(super) fn select_candidates(
    graph: &SemanticGraph,
    request: &ResolvedContextRequest,
) -> ContextSelection {
    let query = graph.query();
    let mut best = BTreeMap::<NodeId, CandidateState>::new();
    let mut pending = BinaryHeap::<Reverse<(CandidateKey, NodeId)>>::new();

    for seed_id in request.seeds() {
        let key = CandidateKey {
            depth: 0,
            steps: Vec::new(),
            seed_id: seed_id.clone(),
            node_id: seed_id.clone(),
        };
        best.insert(
            seed_id.clone(),
            CandidateState {
                key: key.clone(),
                path: Vec::new(),
            },
        );
        pending.push(Reverse((key, seed_id.clone())));
    }

    while let Some(Reverse((key, node_id))) = pending.pop() {
        let Some(state) = best.get(&node_id) else {
            continue;
        };
        if state.key != key || key.depth == request.policy().max_depth() {
            continue;
        }

        let parent_path = state.path.clone();
        for expansion in expansions(&query, request, &node_id) {
            let mut steps = key.steps.clone();
            steps.push(expansion.step_key);
            let next_key = CandidateKey {
                depth: key.depth + 1,
                steps,
                seed_id: key.seed_id.clone(),
                node_id: expansion.node_id.clone(),
            };

            if best
                .get(&expansion.node_id)
                .is_some_and(|existing| existing.key <= next_key)
            {
                continue;
            }

            let mut path = parent_path.clone();
            path.push(expansion.step);
            best.insert(
                expansion.node_id.clone(),
                CandidateState {
                    key: next_key.clone(),
                    path,
                },
            );
            pending.push(Reverse((next_key, expansion.node_id)));
        }
    }

    let mut ordered = best
        .into_iter()
        .filter_map(|(node_id, state)| {
            let node = query.node(&node_id)?;
            let reason = if state.key.depth == 0 {
                ContextInclusionReason::Seed
            } else {
                ContextInclusionReason::Related
            };
            Some((
                state.key.clone(),
                ContextCandidate {
                    node_id,
                    name: node.name().clone(),
                    kind: node.kind(),
                    provenance: canonical_provenance(node.provenance()),
                    depth: state.key.depth,
                    seed_id: state.key.seed_id,
                    path: state.path,
                    reason,
                },
            ))
        })
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));

    let candidate_omitted = ordered
        .len()
        .saturating_sub(request.policy().max_candidates());
    ordered.truncate(request.policy().max_candidates());

    ContextSelection {
        intent: request.intent(),
        seeds: request.seeds().to_vec(),
        candidates: ordered
            .into_iter()
            .map(|(_, candidate)| candidate)
            .collect(),
        budget: request.budget(),
        policy: request.policy().clone(),
        candidate_omitted,
    }
}

fn expansions(
    query: &SemanticGraphQuery<'_>,
    request: &ResolvedContextRequest,
    node_id: &NodeId,
) -> Vec<Expansion> {
    let mut expansions = Vec::new();
    if matches!(
        request.policy().direction(),
        ContextTraversalDirection::Outgoing | ContextTraversalDirection::Both
    ) {
        expansions.extend(
            query
                .outgoing_edges(node_id)
                .into_iter()
                .filter_map(|edge| {
                    expansion(
                        query,
                        request,
                        edge,
                        ContextRelationDirection::Outgoing,
                        NodeId::new(edge.target().as_str()),
                    )
                }),
        );
    }
    if matches!(
        request.policy().direction(),
        ContextTraversalDirection::Incoming | ContextTraversalDirection::Both
    ) {
        expansions.extend(
            query
                .incoming_edges(node_id)
                .into_iter()
                .filter_map(|edge| {
                    expansion(
                        query,
                        request,
                        edge,
                        ContextRelationDirection::Incoming,
                        NodeId::new(edge.source().as_str()),
                    )
                }),
        );
    }

    expansions.sort_by(|left, right| {
        (&left.step_key, &left.node_id).cmp(&(&right.step_key, &right.node_id))
    });
    expansions
}

fn expansion(
    query: &SemanticGraphQuery<'_>,
    request: &ResolvedContextRequest,
    edge: &GraphEdge,
    direction: ContextRelationDirection,
    node_id: NodeId,
) -> Option<Expansion> {
    if !request.policy().edge_kinds().contains(&edge.kind()) {
        return None;
    }
    let node = query.node(&node_id)?;
    if !request.policy().allows_node(node.kind()) {
        return None;
    }

    let edge_id = SemanticGraphQuery::edge_id(
        &NodeId::new(edge.source().as_str()),
        &NodeId::new(edge.target().as_str()),
        edge.kind(),
    );
    Some(Expansion {
        step_key: StepKey {
            edge_priority: edge_priority(edge.kind()),
            direction,
            edge_id: edge_id.clone(),
        },
        step: ContextPathStep {
            direction,
            edge_kind: edge.kind(),
            edge_id,
            provenance: canonical_provenance(edge.provenance()),
        },
        node_id,
    })
}

const fn edge_priority(kind: EdgeKind) -> u8 {
    match kind {
        EdgeKind::Contains => 0,
        EdgeKind::Calls => 1,
        EdgeKind::References => 2,
        EdgeKind::Reads => 3,
        EdgeKind::Writes => 4,
        EdgeKind::DependsOn => 5,
        EdgeKind::Opens => 6,
        EdgeKind::Triggers => 7,
        EdgeKind::Includes => 8,
        EdgeKind::Extends => 9,
        EdgeKind::Grants => 10,
    }
}

fn canonical_provenance(provenance: &[Provenance]) -> Vec<Provenance> {
    let mut canonical = provenance.to_vec();
    canonical.sort_by(|left, right| {
        (
            left.source().map(oneagent_common::EntityId::as_str),
            left.producer().as_str(),
            left.origin(),
            left.confidence(),
            left.resolution(),
        )
            .cmp(&(
                right.source().map(oneagent_common::EntityId::as_str),
                right.producer().as_str(),
                right.origin(),
                right.confidence(),
                right.resolution(),
            ))
    });
    canonical.dedup();
    canonical
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, ProducerId,
        Provenance, ResolutionState, SemanticGraph,
    };

    use super::super::{
        ContextEngine, ContextInclusionReason, ContextIntent, ContextPolicy, ContextRequest,
        ContextSeed, ContextTraversalDirection,
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

    fn node(node_id: &str, kind: NodeKind) -> GraphNode {
        GraphNode::new(id(node_id), name(node_id), kind)
    }

    fn request(
        seeds: Vec<&str>,
        direction: ContextTraversalDirection,
        edge_kinds: BTreeSet<EdgeKind>,
        node_kinds: Option<BTreeSet<NodeKind>>,
        max_depth: usize,
        max_candidates: usize,
    ) -> ContextRequest {
        ContextRequest::new(
            ContextIntent::Explain,
            seeds.into_iter().map(ContextSeed::node).collect(),
            65_536,
            ContextPolicy::new(direction, edge_kinds, node_kinds, max_depth, max_candidates),
        )
        .expect("request must be valid")
    }

    fn insert_edge(graph: &mut SemanticGraph, source: &str, target: &str, kind: EdgeKind) {
        graph
            .insert_edge(GraphEdge::new(id(source), id(target), kind))
            .expect("edge endpoints must exist");
    }

    #[test]
    fn selection_applies_edge_priority_before_edge_identity() {
        let mut graph = SemanticGraph::new();
        for node_id in ["seed", "calls", "contains"] {
            graph.insert_node(node(node_id, NodeKind::Procedure));
        }
        insert_edge(&mut graph, "seed", "calls", EdgeKind::Calls);
        insert_edge(&mut graph, "seed", "contains", EdgeKind::Contains);
        let request = request(
            vec!["seed"],
            ContextTraversalDirection::Outgoing,
            BTreeSet::from([EdgeKind::Calls, EdgeKind::Contains]),
            None,
            1,
            3,
        );

        let selection = ContextEngine
            .select_candidates(&graph, &request)
            .expect("selection must succeed");

        assert_eq!(
            selection
                .candidates()
                .iter()
                .map(|candidate| candidate.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["seed", "contains", "calls"]
        );
        assert_eq!(
            selection.candidates()[1].path()[0].edge_kind(),
            EdgeKind::Contains
        );
    }

    #[test]
    fn selection_orders_every_accepted_edge_kind_by_closed_priority() {
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
        graph.insert_node(node("seed", NodeKind::Procedure));
        for (priority, kind) in priorities.into_iter().enumerate() {
            let target = format!("target.{priority:02}");
            graph.insert_node(node(&target, NodeKind::Function));
            insert_edge(&mut graph, "seed", &target, kind);
        }
        let request = request(
            vec!["seed"],
            ContextTraversalDirection::Outgoing,
            priorities.into_iter().collect(),
            None,
            1,
            12,
        );

        let selection = ContextEngine
            .select_candidates(&graph, &request)
            .expect("selection must succeed");

        assert_eq!(selection.candidates().len(), 12);
        assert_eq!(
            selection
                .candidates()
                .iter()
                .skip(1)
                .map(|candidate| candidate.path()[0].edge_kind())
                .collect::<Vec<_>>(),
            priorities
        );
    }

    #[test]
    fn both_direction_prefers_outgoing_path_to_the_same_node() {
        let mut graph = SemanticGraph::new();
        graph.insert_node(node("seed", NodeKind::Procedure));
        graph.insert_node(node("related", NodeKind::Function));
        insert_edge(&mut graph, "seed", "related", EdgeKind::Calls);
        insert_edge(&mut graph, "related", "seed", EdgeKind::Calls);
        let request = request(
            vec!["seed"],
            ContextTraversalDirection::Both,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            1,
            2,
        );

        let selection = ContextEngine
            .select_candidates(&graph, &request)
            .expect("selection must succeed");

        assert_eq!(
            selection.candidates()[1].path()[0].direction(),
            super::super::ContextRelationDirection::Outgoing
        );
    }

    #[test]
    fn incoming_selection_honors_zero_and_maximum_depth() {
        let mut graph = SemanticGraph::new();
        for index in 0..=5 {
            graph.insert_node(node(&format!("node.{index}"), NodeKind::Procedure));
        }
        for index in 0..5 {
            insert_edge(
                &mut graph,
                &format!("node.{}", index + 1),
                &format!("node.{index}"),
                EdgeKind::Calls,
            );
        }
        let zero_depth = request(
            vec!["node.0"],
            ContextTraversalDirection::Incoming,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            0,
            6,
        );
        let maximum_depth = request(
            vec!["node.0"],
            ContextTraversalDirection::Incoming,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            4,
            6,
        );

        let zero = ContextEngine
            .select_candidates(&graph, &zero_depth)
            .expect("selection must succeed");
        let maximum = ContextEngine
            .select_candidates(&graph, &maximum_depth)
            .expect("selection must succeed");

        assert_eq!(zero.candidates().len(), 1);
        assert_eq!(maximum.candidates().len(), 5);
        assert_eq!(maximum.candidates()[4].node_id().as_str(), "node.4");
        assert_eq!(maximum.candidates()[4].depth(), 4);
        assert_eq!(maximum.candidate_omitted(), 0);
    }

    #[test]
    fn selection_enforces_depth_node_filter_cycles_and_candidate_limit() {
        let mut graph = SemanticGraph::new();
        for (node_id, kind) in [
            ("seed", NodeKind::Procedure),
            ("allowed", NodeKind::Function),
            ("filtered", NodeKind::Module),
            ("deep", NodeKind::Function),
        ] {
            graph.insert_node(node(node_id, kind));
        }
        insert_edge(&mut graph, "seed", "allowed", EdgeKind::Calls);
        insert_edge(&mut graph, "seed", "filtered", EdgeKind::Calls);
        insert_edge(&mut graph, "allowed", "seed", EdgeKind::Calls);
        insert_edge(&mut graph, "allowed", "deep", EdgeKind::Calls);
        let request = request(
            vec!["seed"],
            ContextTraversalDirection::Outgoing,
            BTreeSet::from([EdgeKind::Calls]),
            Some(BTreeSet::from([NodeKind::Procedure, NodeKind::Function])),
            2,
            2,
        );

        let selection = ContextEngine
            .select_candidates(&graph, &request)
            .expect("selection must succeed");

        assert_eq!(
            selection
                .candidates()
                .iter()
                .map(|candidate| candidate.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["seed", "allowed"]
        );
        assert_eq!(selection.candidate_omitted(), 1);
        assert!(selection.candidate_truncated());
    }

    #[test]
    fn selection_keeps_the_canonical_best_path_across_seed_and_edge_order() {
        fn build(reverse: bool) -> SemanticGraph {
            let mut graph = SemanticGraph::new();
            let mut nodes = vec!["seed.b", "target", "seed.a"];
            let mut edges = vec![
                ("seed.b", "target", EdgeKind::Calls),
                ("seed.a", "target", EdgeKind::Calls),
            ];
            if reverse {
                nodes.reverse();
                edges.reverse();
            }
            for node_id in nodes {
                graph.insert_node(node(node_id, NodeKind::Procedure));
            }
            for (source, target, kind) in edges {
                insert_edge(&mut graph, source, target, kind);
            }
            graph
        }

        let first_request = request(
            vec!["seed.b", "seed.a"],
            ContextTraversalDirection::Outgoing,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            1,
            3,
        );
        let second_request = request(
            vec!["seed.a", "seed.b"],
            ContextTraversalDirection::Outgoing,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            1,
            3,
        );

        let first = ContextEngine
            .select_candidates(&build(false), &first_request)
            .expect("selection must succeed");
        let second = ContextEngine
            .select_candidates(&build(true), &second_request)
            .expect("selection must succeed");

        assert_eq!(first, second);
        assert_eq!(first.candidates()[2].node_id(), &NodeId::new("target"));
        assert_eq!(first.candidates()[2].seed_id(), &NodeId::new("seed.a"));
        assert_eq!(
            first.candidates()[2].reason(),
            ContextInclusionReason::Related
        );
    }

    #[test]
    fn selection_canonicalizes_and_deduplicates_provenance() {
        let early = provenance(Some("source.a"), "producer.a");
        let late = provenance(Some("source.z"), "producer.z");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new_with_provenance(
            id("seed"),
            name("Seed"),
            NodeKind::Procedure,
            vec![late.clone(), early.clone(), early.clone()],
        ));
        graph.insert_node(node("related", NodeKind::Function));
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                id("seed"),
                id("related"),
                EdgeKind::Calls,
                vec![late.clone(), early.clone(), early.clone()],
            ))
            .expect("edge endpoints must exist");
        let request = request(
            vec!["seed"],
            ContextTraversalDirection::Outgoing,
            BTreeSet::from([EdgeKind::Calls]),
            None,
            1,
            2,
        );

        let selection = ContextEngine
            .select_candidates(&graph, &request)
            .expect("selection must succeed");

        assert_eq!(
            selection.candidates()[0].provenance(),
            &[early.clone(), late.clone()]
        );
        assert_eq!(
            selection.candidates()[1].path()[0].provenance(),
            &[early, late]
        );
        assert_eq!(
            selection.candidates()[0].reason(),
            ContextInclusionReason::Seed
        );
    }
}
