use std::fmt::Write;

use oneagent_graph::{EdgeKind, NodeKind};

use super::{
    ContextBundle, ContextCandidate, ContextError, ContextInclusionReason, ContextItem,
    ContextRelationDirection, ContextSelection,
};

pub(super) fn assemble(selection: ContextSelection) -> Result<ContextBundle, ContextError> {
    let rendered_candidates = selection
        .candidates
        .iter()
        .map(|candidate| {
            render_candidate(candidate).map(|fragment| {
                let cost_bytes = fragment.len();
                (candidate, fragment, cost_bytes)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let required_seed_bytes = rendered_candidates
        .iter()
        .filter(|(candidate, _, _)| candidate.reason == ContextInclusionReason::Seed)
        .try_fold(0_usize, |total, (_, _, cost)| checked_add(total, *cost))?;
    if required_seed_bytes > selection.budget.bytes() {
        return Err(ContextError::InsufficientBudget {
            required: required_seed_bytes,
            available: selection.budget.bytes(),
        });
    }

    let mut items = Vec::new();
    let mut rendered = String::new();
    let mut used_bytes = 0_usize;
    let mut budget_omitted = 0_usize;
    let mut related_blocked = false;

    for (candidate, fragment, cost_bytes) in rendered_candidates {
        if candidate.reason == ContextInclusionReason::Related {
            if related_blocked {
                budget_omitted = checked_add(budget_omitted, 1)?;
                continue;
            }
            let next_used = checked_add(used_bytes, cost_bytes)?;
            if next_used > selection.budget.bytes() {
                related_blocked = true;
                budget_omitted = checked_add(budget_omitted, 1)?;
                continue;
            }
            used_bytes = next_used;
        } else {
            used_bytes = checked_add(used_bytes, cost_bytes)?;
        }

        rendered.push_str(&fragment);
        items.push(ContextItem {
            node_id: candidate.node_id.clone(),
            name: candidate.name.clone(),
            kind: candidate.kind,
            provenance: candidate.provenance.clone(),
            depth: candidate.depth,
            seed_id: candidate.seed_id.clone(),
            path: candidate.path.clone(),
            reason: candidate.reason,
            fragment,
            cost_bytes,
        });
    }

    if rendered.len() != used_bytes {
        return Err(ContextError::CostOverflow);
    }
    let remaining_bytes = selection
        .budget
        .bytes()
        .checked_sub(used_bytes)
        .ok_or(ContextError::CostOverflow)?;

    Ok(ContextBundle {
        intent: selection.intent,
        seeds: selection.seeds,
        items,
        budget: selection.budget,
        used_bytes,
        remaining_bytes,
        candidate_omitted: selection.candidate_omitted,
        budget_omitted,
        rendered,
    })
}

fn checked_add(left: usize, right: usize) -> Result<usize, ContextError> {
    left.checked_add(right).ok_or(ContextError::CostOverflow)
}

fn render_candidate(candidate: &ContextCandidate) -> Result<String, ContextError> {
    let mut fragment = String::new();
    let node_id = candidate.node_id.as_str();
    let name = candidate.name.as_str();
    writeln!(
        fragment,
        "node kind={} id={}:{} name={}:{}",
        node_kind(candidate.kind),
        node_id.len(),
        node_id,
        name.len(),
        name
    )
    .map_err(|_| ContextError::CostOverflow)?;

    let seed_id = candidate.seed_id.as_str();
    match candidate.reason {
        ContextInclusionReason::Seed => {
            writeln!(
                fragment,
                "reason seed={}:{} depth=0",
                seed_id.len(),
                seed_id
            )
            .map_err(|_| ContextError::CostOverflow)?;
        }
        ContextInclusionReason::Related => {
            write!(
                fragment,
                "reason seed={}:{} depth={} path={}:",
                seed_id.len(),
                seed_id,
                candidate.depth,
                candidate.path.len()
            )
            .map_err(|_| ContextError::CostOverflow)?;
            for (index, step) in candidate.path.iter().enumerate() {
                if index > 0 {
                    fragment.push(';');
                }
                let edge_id = step.edge_id.as_str();
                write!(
                    fragment,
                    "{},{},{}:{}",
                    relation_direction(step.direction),
                    edge_kind(step.edge_kind),
                    edge_id.len(),
                    edge_id
                )
                .map_err(|_| ContextError::CostOverflow)?;
            }
            fragment.push('\n');
        }
    }

    Ok(fragment)
}

fn node_kind(kind: NodeKind) -> String {
    match kind {
        NodeKind::Metadata(kind) => format!("metadata.{}", kind.as_str()),
        NodeKind::Module => "module".to_owned(),
        NodeKind::Procedure => "procedure".to_owned(),
        NodeKind::Function => "function".to_owned(),
        NodeKind::Query => "query".to_owned(),
        NodeKind::DataCompositionSchema => "data_composition_schema".to_owned(),
        NodeKind::DataSet => "data_set".to_owned(),
        NodeKind::DataCompositionField => "data_composition_field".to_owned(),
        NodeKind::XdtoType => "xdto_type".to_owned(),
        NodeKind::HttpServiceUrlTemplate => "http_service_url_template".to_owned(),
        NodeKind::HttpServiceMethod => "http_service_method".to_owned(),
        NodeKind::WebServiceOperation => "web_service_operation".to_owned(),
        NodeKind::WebServiceParameter => "web_service_parameter".to_owned(),
        NodeKind::Form => "form".to_owned(),
        NodeKind::Command => "command".to_owned(),
        NodeKind::Attribute => "attribute".to_owned(),
        NodeKind::StandardAttribute => "standard_attribute".to_owned(),
        NodeKind::TabularSection => "tabular_section".to_owned(),
        NodeKind::Dimension => "dimension".to_owned(),
        NodeKind::Resource => "resource".to_owned(),
        NodeKind::Measure => "measure".to_owned(),
        NodeKind::Role => "role".to_owned(),
        NodeKind::AccessRight => "access_right".to_owned(),
        NodeKind::Subsystem => "subsystem".to_owned(),
        NodeKind::Unknown => "unknown".to_owned(),
    }
}

const fn edge_kind(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Calls => "calls",
        EdgeKind::References => "references",
        EdgeKind::Reads => "reads",
        EdgeKind::Writes => "writes",
        EdgeKind::Grants => "grants",
        EdgeKind::Includes => "includes",
        EdgeKind::Extends => "extends",
        EdgeKind::DependsOn => "depends_on",
        EdgeKind::Opens => "opens",
        EdgeKind::Triggers => "triggers",
    }
}

const fn relation_direction(direction: ContextRelationDirection) -> &'static str {
    match direction {
        ContextRelationDirection::Outgoing => "outgoing",
        ContextRelationDirection::Incoming => "incoming",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        EdgeKind, GraphEdge, GraphNode, NodeId, NodeKind, SemanticGraph, SemanticGraphQuery,
    };

    use super::super::{
        ContextEngine, ContextError, ContextIntent, ContextPolicy, ContextRequest, ContextSeed,
        ContextTraversalDirection,
    };
    use super::{checked_add, edge_kind, node_kind, relation_direction};

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn graph(nodes: &[(&str, &str, NodeKind)], edges: &[(&str, &str, EdgeKind)]) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        for (node_id, node_name, kind) in nodes {
            graph.insert_node(GraphNode::new(id(node_id), name(node_name), *kind));
        }
        for (source, target, kind) in edges {
            graph
                .insert_edge(GraphEdge::new(id(source), id(target), *kind))
                .expect("edge endpoints must exist");
        }
        graph
    }

    fn request(seeds: Vec<&str>, budget: usize, max_candidates: usize) -> ContextRequest {
        ContextRequest::new(
            ContextIntent::Explain,
            seeds.into_iter().map(ContextSeed::node).collect(),
            budget,
            ContextPolicy::new(
                ContextTraversalDirection::Both,
                BTreeSet::from([EdgeKind::Contains, EdgeKind::Calls]),
                None,
                2,
                max_candidates,
            ),
        )
        .expect("request must be valid")
    }

    #[test]
    fn seed_fragment_has_exact_two_line_rendering_and_accounting() {
        let graph = graph(&[("seed", "Seed", NodeKind::Procedure)], &[]);
        let request = request(vec!["seed"], 65_536, 1);

        let bundle = ContextEngine
            .build(&graph, &request)
            .expect("bundle must build");
        let expected = "node kind=procedure id=4:seed name=4:Seed\nreason seed=4:seed depth=0\n";

        assert_eq!(bundle.rendered(), expected);
        assert_eq!(bundle.items()[0].fragment(), expected);
        assert_eq!(bundle.items()[0].cost_bytes(), expected.len());
        assert_eq!(bundle.used_bytes(), expected.len());
        assert_eq!(bundle.remaining_bytes(), 65_536 - expected.len());
        assert!(!bundle.candidate_truncated());
        assert!(!bundle.budget_truncated());
    }

    #[test]
    fn unicode_lengths_and_exact_budget_are_utf8_bytes() {
        let graph = graph(&[("узел", "Имя", NodeKind::Function)], &[]);
        let large = request(vec!["узел"], 65_536, 1);
        let measured = ContextEngine
            .build(&graph, &large)
            .expect("bundle must build");
        let expected = "node kind=function id=8:узел name=6:Имя\nreason seed=8:узел depth=0\n";
        assert_eq!(measured.rendered(), expected);

        let exact = request(vec!["узел"], expected.len(), 1);
        let bundle = ContextEngine
            .build(&graph, &exact)
            .expect("exact budget must build");
        assert_eq!(bundle.used_bytes(), expected.len());
        assert_eq!(bundle.remaining_bytes(), 0);
    }

    #[test]
    fn one_byte_short_mandatory_budget_fails_without_partial_bundle() {
        let graph = graph(&[("seed", "Seed", NodeKind::Procedure)], &[]);
        let measured = ContextEngine
            .build(&graph, &request(vec!["seed"], 65_536, 1))
            .expect("bundle must build")
            .used_bytes();

        assert_eq!(
            ContextEngine.build(&graph, &request(vec!["seed"], measured - 1, 1)),
            Err(ContextError::InsufficientBudget {
                required: measured,
                available: measured - 1,
            })
        );
    }

    #[test]
    fn related_admission_is_a_whole_fragment_prefix_with_explicit_omissions() {
        let graph = graph(
            &[
                ("seed", "Seed", NodeKind::Procedure),
                ("a", "A", NodeKind::Function),
                ("b", "B", NodeKind::Function),
            ],
            &[
                ("seed", "a", EdgeKind::Calls),
                ("seed", "b", EdgeKind::Calls),
            ],
        );
        let full = ContextEngine
            .build(&graph, &request(vec!["seed"], 65_536, 3))
            .expect("bundle must build");
        let prefix_budget = full.items()[0].cost_bytes() + full.items()[1].cost_bytes();

        let prefix = ContextEngine
            .build(&graph, &request(vec!["seed"], prefix_budget, 3))
            .expect("prefix must build");

        assert_eq!(prefix.items().len(), 2);
        assert_eq!(prefix.items()[1].node_id().as_str(), "a");
        assert_eq!(prefix.used_bytes(), prefix_budget);
        assert_eq!(prefix.remaining_bytes(), 0);
        assert_eq!(prefix.budget_omitted(), 1);
        assert!(prefix.budget_truncated());
        assert_eq!(
            prefix.rendered(),
            format!(
                "{}{}",
                prefix.items()[0].fragment(),
                prefix.items()[1].fragment()
            )
        );
    }

    #[test]
    fn candidate_and_budget_omissions_remain_distinct() {
        let graph = graph(
            &[
                ("seed", "Seed", NodeKind::Procedure),
                ("a", "A", NodeKind::Function),
                ("b", "B", NodeKind::Function),
            ],
            &[
                ("seed", "a", EdgeKind::Calls),
                ("seed", "b", EdgeKind::Calls),
            ],
        );
        let seed_only = ContextEngine
            .build(&graph, &request(vec!["seed"], 65_536, 1))
            .expect("seed bundle must build")
            .used_bytes();
        let bundle = ContextEngine
            .build(&graph, &request(vec!["seed"], seed_only, 2))
            .expect("bundle must build");

        assert_eq!(bundle.items().len(), 1);
        assert_eq!(bundle.candidate_omitted(), 1);
        assert_eq!(bundle.budget_omitted(), 1);
    }

    #[test]
    fn equivalent_reordered_graphs_and_repeated_builds_are_equal() {
        let first = graph(
            &[
                ("seed", "Seed", NodeKind::Procedure),
                ("target", "Target", NodeKind::Function),
            ],
            &[("seed", "target", EdgeKind::Calls)],
        );
        let second = graph(
            &[
                ("target", "Target", NodeKind::Function),
                ("seed", "Seed", NodeKind::Procedure),
            ],
            &[("seed", "target", EdgeKind::Calls)],
        );
        let request = request(vec!["seed"], 65_536, 2);

        let expected = ContextEngine
            .build(&first, &request)
            .expect("bundle must build");
        let edge_id = SemanticGraphQuery::edge_id(
            &NodeId::new("seed"),
            &NodeId::new("target"),
            EdgeKind::Calls,
        );
        assert_eq!(
            expected.items()[1].fragment(),
            format!(
                "node kind=function id=6:target name=6:Target\nreason seed=4:seed depth=1 path=1:outgoing,calls,{}:{}\n",
                edge_id.as_str().len(),
                edge_id
            )
        );
        assert_eq!(
            ContextEngine
                .build(&first, &request)
                .expect("repeat must build"),
            expected
        );
        assert_eq!(
            ContextEngine
                .build(&second, &request)
                .expect("reordered graph must build"),
            expected
        );
    }

    #[test]
    fn kind_and_direction_vocabularies_are_closed_and_exact() {
        assert_eq!(node_kind(NodeKind::Module), "module");
        assert_eq!(
            node_kind(NodeKind::DataCompositionSchema),
            "data_composition_schema"
        );
        assert_eq!(node_kind(NodeKind::Unknown), "unknown");
        assert_eq!(
            [
                EdgeKind::Contains,
                EdgeKind::Calls,
                EdgeKind::References,
                EdgeKind::Reads,
                EdgeKind::Writes,
                EdgeKind::Grants,
                EdgeKind::Includes,
                EdgeKind::Extends,
                EdgeKind::DependsOn,
                EdgeKind::Opens,
                EdgeKind::Triggers,
            ]
            .map(edge_kind),
            [
                "contains",
                "calls",
                "references",
                "reads",
                "writes",
                "grants",
                "includes",
                "extends",
                "depends_on",
                "opens",
                "triggers",
            ]
        );
        assert_eq!(
            [
                super::super::ContextRelationDirection::Outgoing,
                super::super::ContextRelationDirection::Incoming,
            ]
            .map(relation_direction),
            ["outgoing", "incoming"]
        );
    }

    #[test]
    fn checked_accounting_reports_overflow() {
        assert_eq!(checked_add(usize::MAX, 1), Err(ContextError::CostOverflow));
    }
}
