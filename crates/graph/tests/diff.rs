use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{GraphChangeKind, GraphNode, NodeKind, SemanticGraph, SemanticGraphDiff};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

#[test]
fn public_diff_api_compares_graph_snapshots_directionally() {
    let mut old = SemanticGraph::new();
    let mut new = SemanticGraph::new();

    old.insert_node(GraphNode::new(
        id("node.old"),
        name("Old"),
        NodeKind::Module,
    ));
    new.insert_node(GraphNode::new(
        id("node.new"),
        name("New"),
        NodeKind::Function,
    ));

    let diff = SemanticGraphDiff::between(&old, &new);
    let convenience = old.diff(&new);

    assert_eq!(diff, convenience);
    assert_eq!(diff.added_nodes().len(), 1);
    assert_eq!(diff.added_nodes()[0].kind(), GraphChangeKind::Added);
    assert_eq!(diff.added_nodes()[0].id().as_str(), "node.new");
    assert_eq!(diff.removed_nodes().len(), 1);
    assert_eq!(diff.removed_nodes()[0].kind(), GraphChangeKind::Removed);
    assert_eq!(diff.removed_nodes()[0].id().as_str(), "node.old");
    assert_eq!(diff.summary().total_changes(), 2);
}
