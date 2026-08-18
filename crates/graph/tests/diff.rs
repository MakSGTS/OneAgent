use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    GraphChangeKind, GraphNode, GraphNodePayload, NodeKind, NodeModifiedAspect, SemanticGraph,
    SemanticGraphDiff,
};
use oneagent_metadata::{CommonMetadataPayload, MetadataKind, MetadataPayload};

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

#[test]
fn payload_only_change_preserves_identity_and_modifies_semantic_content() {
    let node_id = id("metadata.catalog.products");
    let metadata_kind = NodeKind::Metadata(MetadataKind::Catalog);
    let node = |synonym: &str| {
        GraphNode::new_with_payload(
            node_id.clone(),
            name("Products"),
            metadata_kind,
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(Some(synonym.to_owned())),
                None,
            )),
        )
        .expect("Catalog common payload must be valid")
    };
    let mut old = SemanticGraph::new();
    let mut new = SemanticGraph::new();
    old.insert_node(node("Products"));
    new.insert_node(node("Goods"));

    let diff = SemanticGraphDiff::between(&old, &new);
    let repeated = SemanticGraphDiff::between(&old, &new);

    assert_eq!(diff, repeated);
    assert!(diff.added_nodes().is_empty());
    assert!(diff.removed_nodes().is_empty());
    assert_eq!(diff.modified_nodes().len(), 1);
    assert_eq!(
        diff.modified_nodes()[0].modified_aspects(),
        &[NodeModifiedAspect::SemanticContent]
    );
    assert_eq!(diff.modified_nodes()[0].id().as_str(), node_id.as_str());
    assert_eq!(
        diff.modified_nodes()[0]
            .new_state()
            .expect("new snapshot must exist")
            .payload()
            .metadata()
            .expect("metadata payload must exist")
            .common()
            .synonym(),
        Some("Goods")
    );
}
