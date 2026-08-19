use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph, SemanticGraphReport,
    SemanticReferenceOutcome, SemanticReferenceStatistics,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

#[test]
fn public_report_api_aggregates_graph_and_reference_statistics() {
    let module_id = id("module.sales");
    let procedure_id = id("procedure.post");
    let mut graph = SemanticGraph::new();
    let mut statistics = SemanticReferenceStatistics::new();

    graph.insert_node(GraphNode::new(
        module_id.clone(),
        name("Sales"),
        NodeKind::Module,
    ));
    graph.insert_node(GraphNode::new(
        procedure_id.clone(),
        name("Post"),
        NodeKind::Procedure,
    ));
    graph
        .insert_edge(GraphEdge::new(module_id, procedure_id, EdgeKind::Contains))
        .expect("edge must be valid");
    statistics.record(SemanticReferenceOutcome::Resolved, true);

    let report =
        SemanticGraphReport::from_graph_diagnostics_and_references(&graph, &[], statistics);

    assert_eq!(report.graph().total_nodes(), 2);
    assert_eq!(report.graph().total_edges(), 1);
    assert_eq!(report.nodes().by_kind()[&NodeKind::Module], 1);
    assert_eq!(report.edges().by_kind()[&EdgeKind::Contains], 1);
    assert_eq!(report.resolution().total(), 1);
    assert_eq!(report.resolution().resolved(), 1);
}

#[test]
fn report_counts_opens_edges_by_kind() {
    let procedure = id("procedure.open");
    let form = id("form.document");
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new(
        procedure.clone(),
        name("Open"),
        NodeKind::Procedure,
    ));
    graph.insert_node(GraphNode::new(
        form.clone(),
        name("DocumentForm"),
        NodeKind::Form,
    ));
    graph
        .insert_edge(GraphEdge::new(procedure, form, EdgeKind::Opens))
        .expect("Opens edge must be stored");

    let report = SemanticGraphReport::from_graph(&graph);

    assert_eq!(report.graph().total_edges(), 1);
    assert_eq!(report.edges().by_kind()[&EdgeKind::Opens], 1);
}
