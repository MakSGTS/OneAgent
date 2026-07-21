use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticGraph, SemanticGraphReport, SemanticGraphValidationCode,
    SemanticGraphValidationIssueKind, SemanticGraphValidationSeverity, SemanticGraphValidator,
    SemanticReferenceOutcome, SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: &str) -> Provenance {
    Provenance::new(
        Some(id(source)),
        ProducerId::new("oneagent.graph.validation.tests"),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

struct FixtureIds {
    configuration: EntityId,
    document: EntityId,
    module: EntityId,
    procedure: EntityId,
    function: EntityId,
    attribute: EntityId,
}

impl FixtureIds {
    fn new() -> Self {
        Self {
            configuration: id("configuration.main"),
            document: id("metadata.document.sales"),
            module: id("metadata.document.sales:object_module"),
            procedure: id("metadata.document.sales:object_module:procedure:Post"),
            function: id("metadata.document.sales:object_module:function:Calculate"),
            attribute: id("metadata.document.sales:attribute:Company"),
        }
    }
}

fn valid_graph(reverse_order: bool) -> SemanticGraph {
    let ids = FixtureIds::new();
    let mut graph = SemanticGraph::new();

    insert_valid_nodes(&mut graph, &ids, reverse_order);
    insert_valid_edges(&mut graph, ids, reverse_order);

    graph
}

fn insert_valid_nodes(graph: &mut SemanticGraph, ids: &FixtureIds, reverse_order: bool) {
    let nodes = [
        GraphNode::new_with_provenance(
            ids.configuration.clone(),
            name("Configuration"),
            NodeKind::Metadata(MetadataKind::Configuration),
            vec![provenance("configuration.main")],
        ),
        GraphNode::new_with_provenance(
            ids.document.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
            vec![provenance("metadata.document.sales")],
        ),
        GraphNode::new_with_provenance(
            ids.module.clone(),
            name("ObjectModule"),
            NodeKind::Module,
            vec![provenance("metadata.document.sales:object_module")],
        ),
        GraphNode::new_with_provenance(
            ids.procedure.clone(),
            name("Post"),
            NodeKind::Procedure,
            vec![provenance("metadata.document.sales:object_module")],
        ),
        GraphNode::new_with_provenance(
            ids.function.clone(),
            name("Calculate"),
            NodeKind::Function,
            vec![provenance("metadata.document.sales:object_module")],
        ),
        GraphNode::new_with_provenance(
            ids.attribute.clone(),
            name("Company"),
            NodeKind::Attribute,
            vec![provenance("metadata.document.sales")],
        ),
    ];

    if reverse_order {
        for node in nodes.into_iter().rev() {
            graph.insert_node(node);
        }
    } else {
        for node in nodes {
            graph.insert_node(node);
        }
    }
}

fn insert_valid_edges(graph: &mut SemanticGraph, ids: FixtureIds, reverse_order: bool) {
    let edges = [
        GraphEdge::new_with_provenance(
            ids.configuration,
            ids.document.clone(),
            EdgeKind::Contains,
            vec![provenance("configuration.main")],
        ),
        GraphEdge::new_with_provenance(
            ids.document.clone(),
            ids.module.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ),
        GraphEdge::new_with_provenance(
            ids.document,
            ids.attribute,
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ),
        GraphEdge::new_with_provenance(
            ids.module.clone(),
            ids.procedure.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales:object_module")],
        ),
        GraphEdge::new_with_provenance(
            ids.module,
            ids.function.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales:object_module")],
        ),
        GraphEdge::new_with_provenance(
            ids.procedure,
            ids.function,
            EdgeKind::Calls,
            vec![provenance("metadata.document.sales:object_module")],
        ),
    ];

    if reverse_order {
        for edge in edges.into_iter().rev() {
            graph.insert_edge(edge).expect("edge must be valid");
        }
    } else {
        for edge in edges {
            graph.insert_edge(edge).expect("edge must be valid");
        }
    }
}

#[test]
fn empty_graph_is_valid() {
    let graph = SemanticGraph::new();
    let result = graph.validate();

    assert!(result.is_valid());
    assert_eq!(result.summary().total(), 0);
    assert_eq!(result.error_count(), 0);
}

#[test]
fn valid_graph_has_no_issues() {
    let graph = valid_graph(false);
    let result = SemanticGraphValidator::new().validate(&graph);

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn validation_is_deterministic_across_repeated_runs_and_insertion_order() {
    let normal = valid_graph(false);
    let reversed = valid_graph(true);

    let first = normal.validate();
    let second = normal.validate();
    let reversed_result = reversed.validate();

    assert_eq!(first, second);
    assert_eq!(first, reversed_result);
}

#[test]
fn warning_only_missing_provenance_keeps_graph_valid() {
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new(
        id("metadata.document.sales"),
        name("Sales"),
        NodeKind::Metadata(MetadataKind::Document),
    ));

    let result = graph.validate();

    assert!(result.is_valid());
    assert_eq!(result.error_count(), 0);
    assert_eq!(result.warning_count(), 1);
    assert_eq!(
        result.issues()[0].code(),
        SemanticGraphValidationCode::MissingNodeProvenance
    );
    assert_eq!(
        result.summary().by_kind()[&SemanticGraphValidationIssueKind::Provenance],
        1
    );
}

#[test]
fn invalid_edge_endpoint_combination_is_error() {
    let source_id = id("metadata.document.sales");
    let target_id = id("metadata.document.sales:attribute:Company");
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new_with_provenance(
        source_id.clone(),
        name("Sales"),
        NodeKind::Metadata(MetadataKind::Document),
        vec![provenance("metadata.document.sales")],
    ));
    graph.insert_node(GraphNode::new_with_provenance(
        target_id.clone(),
        name("Company"),
        NodeKind::Attribute,
        vec![provenance("metadata.document.sales")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            source_id.clone(),
            target_id,
            EdgeKind::Calls,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("storage only validates endpoint existence");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 2);
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
            && issue.severity() == SemanticGraphValidationSeverity::Error
            && issue.source_kind() == Some(NodeKind::Metadata(MetadataKind::Document))
            && issue.target_kind() == Some(NodeKind::Attribute)
            && issue.edge_kind() == Some(EdgeKind::Calls)
    }));
}

#[test]
fn child_without_owner_is_error() {
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new_with_provenance(
        id("metadata.document.sales:attribute:Company"),
        name("Company"),
        NodeKind::Attribute,
        vec![provenance("metadata.document.sales")],
    ));

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::InvalidOwner
            && issue.invariant() == "mandatory owner edge"
    }));
}

#[test]
fn calls_self_loop_is_error() {
    let procedure_id = id("procedure.self");
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new_with_provenance(
        procedure_id.clone(),
        name("SelfCall"),
        NodeKind::Procedure,
        vec![provenance("procedure.self")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            procedure_id.clone(),
            procedure_id,
            EdgeKind::Calls,
            vec![provenance("procedure.self")],
        ))
        .expect("storage allows self-loop");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::ForbiddenSelfLoop
            && issue.edge_kind() == Some(EdgeKind::Calls)
    }));
}

#[test]
fn calls_cycle_is_not_owner_cycle() {
    let first_id = id("procedure.first");
    let second_id = id("procedure.second");
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new_with_provenance(
        first_id.clone(),
        name("First"),
        NodeKind::Procedure,
        vec![provenance("procedure.first")],
    ));
    graph.insert_node(GraphNode::new_with_provenance(
        second_id.clone(),
        name("Second"),
        NodeKind::Procedure,
        vec![provenance("procedure.second")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            first_id.clone(),
            second_id.clone(),
            EdgeKind::Calls,
            vec![provenance("procedure.first")],
        ))
        .expect("edge must be valid");
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            second_id,
            first_id,
            EdgeKind::Calls,
            vec![provenance("procedure.second")],
        ))
        .expect("edge must be valid");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(
        !result
            .issues()
            .iter()
            .any(|issue| issue.code() == SemanticGraphValidationCode::Cycle)
    );
}

#[test]
fn build_level_validation_accepts_consistent_statistics() {
    let graph = valid_graph(false);
    let mut statistics = SemanticReferenceStatistics::new();
    statistics.record(SemanticReferenceOutcome::Resolved, true);

    let result = SemanticGraphValidator::new().validate_build_result(&graph, &[], statistics);

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn build_level_validation_detects_report_mismatch() {
    let graph = valid_graph(false);
    let mismatched_report = SemanticGraphReport::from_graph(&SemanticGraph::new());

    let result = SemanticGraphValidator::new().validate_build_result_with_report(
        &graph,
        &[],
        SemanticReferenceStatistics::new(),
        &mismatched_report,
    );

    assert!(!result.is_valid());
    assert!(
        result
            .issues()
            .iter()
            .any(|issue| issue.code() == SemanticGraphValidationCode::InconsistentReport)
    );
}
