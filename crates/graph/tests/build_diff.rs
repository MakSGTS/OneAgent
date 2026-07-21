use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    GraphNode, NodeKind, SemanticGraph, SemanticGraphBuildDiff, SemanticReferenceOutcome,
    SemanticReferenceStatistics,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

#[test]
fn public_build_diff_api_reuses_graph_diff_and_compares_statistics() {
    let mut previous = SemanticGraph::new();
    let mut current = SemanticGraph::new();
    let mut previous_statistics = SemanticReferenceStatistics::new();
    let mut current_statistics = SemanticReferenceStatistics::new();

    previous.insert_node(GraphNode::new(
        id("node.previous"),
        name("Previous"),
        NodeKind::Module,
    ));
    current.insert_node(GraphNode::new(
        id("node.current"),
        name("Current"),
        NodeKind::Function,
    ));
    previous_statistics.record(SemanticReferenceOutcome::Resolved, true);
    current_statistics.record(SemanticReferenceOutcome::Unresolved, true);

    let diff = SemanticGraphBuildDiff::between(
        &previous,
        &[],
        previous_statistics,
        &current,
        &[],
        current_statistics,
    );

    assert_eq!(diff.graph().added_nodes().len(), 1);
    assert_eq!(diff.graph().removed_nodes().len(), 1);
    assert!(!diff.resolution().changed_metrics().is_empty());
    assert_eq!(diff.summary().node_changes(), 2);
}
