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
fn procedure_can_own_query_node() {
    let ids = FixtureIds::new();
    let query_id = id("metadata.document.sales:object_module:procedure:Post:query:Query");
    let mut graph = valid_graph(false);

    graph.insert_node(GraphNode::new_with_provenance(
        query_id.clone(),
        name("Query"),
        NodeKind::Query,
        vec![provenance("metadata.document.sales:object_module#query")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.procedure,
            query_id,
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales:object_module#query")],
        ))
        .expect("query ownership edge must be stored");

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
fn depends_on_accepts_first_slice_member_to_metadata_pairs() {
    let mut graph = valid_graph(false);
    let ids = FixtureIds::new();
    let dimension_id = id("metadata.register.stock:dimension:Product");
    let resource_id = id("metadata.register.stock:resource:Quantity");
    let register_id = id("metadata.register.stock");

    graph.insert_node(GraphNode::new_with_provenance(
        register_id.clone(),
        name("Stock"),
        NodeKind::Metadata(MetadataKind::AccumulationRegister),
        vec![provenance("metadata.register.stock")],
    ));
    for (member_id, member_name, member_kind) in [
        (dimension_id.clone(), "Product", NodeKind::Dimension),
        (resource_id.clone(), "Quantity", NodeKind::Resource),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            member_id.clone(),
            name(member_name),
            member_kind,
            vec![provenance("metadata.register.stock")],
        ));
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                register_id.clone(),
                member_id,
                EdgeKind::Contains,
                vec![provenance("metadata.register.stock")],
            ))
            .expect("ownership edge must be stored");
    }

    for source_id in [ids.attribute, dimension_id, resource_id] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source_id,
                ids.document.clone(),
                EdgeKind::DependsOn,
                vec![provenance("metadata.member.type")],
            ))
            .expect("depends_on edge must be stored");
    }

    let result = graph.validate();

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn depends_on_rejects_unrelated_endpoint_pairs() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);

    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.procedure.clone(),
            ids.document.clone(),
            EdgeKind::DependsOn,
            vec![provenance("metadata.document.sales:object_module")],
        ))
        .expect("storage only validates endpoint existence");
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.attribute,
            ids.function,
            EdgeKind::DependsOn,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("storage only validates endpoint existence");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 2);
    assert_eq!(
        result
            .issues()
            .iter()
            .filter(|issue| {
                issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
                    && issue.edge_kind() == Some(EdgeKind::DependsOn)
            })
            .count(),
        2
    );
}

#[test]
fn extends_accepts_same_metadata_kind_pairs() {
    let mut graph = valid_graph(false);
    let base_document_id = id("metadata.document.base");

    graph.insert_node(GraphNode::new_with_provenance(
        base_document_id.clone(),
        name("BaseSales"),
        NodeKind::Metadata(MetadataKind::Document),
        vec![provenance("metadata.document.base")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            id("configuration.main"),
            base_document_id.clone(),
            EdgeKind::Contains,
            vec![provenance("configuration.main")],
        ))
        .expect("metadata owner edge must be stored");
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            id("metadata.document.sales"),
            base_document_id,
            EdgeKind::Extends,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("extends edge must be stored");

    let result = graph.validate();

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn extends_rejects_unrelated_endpoint_pairs() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);
    let catalog_id = id("metadata.catalog.products");

    graph.insert_node(GraphNode::new_with_provenance(
        catalog_id.clone(),
        name("Products"),
        NodeKind::Metadata(MetadataKind::Catalog),
        vec![provenance("metadata.catalog.products")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.configuration.clone(),
            catalog_id.clone(),
            EdgeKind::Contains,
            vec![provenance("configuration.main")],
        ))
        .expect("metadata owner edge must be stored");
    for (source, target) in [
        (ids.document.clone(), catalog_id),
        (ids.document.clone(), ids.module.clone()),
        (ids.attribute.clone(), ids.document.clone()),
        (ids.procedure, ids.function),
    ] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source,
                target,
                EdgeKind::Extends,
                vec![provenance("metadata.extension")],
            ))
            .expect("storage only validates endpoint existence");
    }

    let result = graph.validate();

    assert!(!result.is_valid());
    assert_eq!(
        result
            .issues()
            .iter()
            .filter(|issue| {
                issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
                    && issue.edge_kind() == Some(EdgeKind::Extends)
            })
            .count(),
        4
    );
}

#[test]
fn extends_self_loop_is_error() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);

    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.document.clone(),
            ids.document,
            EdgeKind::Extends,
            vec![provenance("metadata.extension")],
        ))
        .expect("storage allows self-loop");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::ForbiddenSelfLoop
            && issue.edge_kind() == Some(EdgeKind::Extends)
    }));
}

#[test]
fn grants_accepts_role_to_access_right_pair() {
    let mut graph = valid_graph(false);
    let role_id = id("metadata.role.sales_manager:role");
    let access_right_id =
        id("access_right:resource#23:metadata.document.sales;right#10:right.read");

    graph.insert_node(GraphNode::new_with_provenance(
        role_id.clone(),
        name("SalesManager"),
        NodeKind::Role,
        vec![provenance("metadata.role.sales_manager")],
    ));
    graph.insert_node(GraphNode::new_with_provenance(
        access_right_id.clone(),
        name("right.read on metadata.document.sales"),
        NodeKind::AccessRight,
        vec![provenance("metadata.role.sales_manager#rights")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            role_id,
            access_right_id,
            EdgeKind::Grants,
            vec![provenance("metadata.role.sales_manager#rights")],
        ))
        .expect("grants edge must be stored");

    let result = graph.validate();

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn grants_rejects_unrelated_endpoint_pairs() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);
    let role_id = id("metadata.role.sales_manager:role");
    let metadata_role_id = id("metadata.role.sales_manager");
    let access_right_id =
        id("access_right:resource#23:metadata.document.sales;right#10:right.read");
    let unknown_id = id("unknown.access");
    let query_id = id("metadata.document.sales:object_module:procedure:Post:query:AccessQuery");

    for (node_id, node_name, node_kind) in [
        (role_id.clone(), "SalesManager", NodeKind::Role),
        (
            metadata_role_id.clone(),
            "SalesManager",
            NodeKind::Metadata(MetadataKind::Role),
        ),
        (
            access_right_id.clone(),
            "right.read on metadata.document.sales",
            NodeKind::AccessRight,
        ),
        (unknown_id.clone(), "UnknownAccess", NodeKind::Unknown),
        (query_id.clone(), "AccessQuery", NodeKind::Query),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            node_id,
            name(node_name),
            node_kind,
            vec![provenance("metadata.role.sales_manager#rights")],
        ));
    }
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.procedure.clone(),
            query_id.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales:object_module#query")],
        ))
        .expect("query ownership edge must be stored");

    for (source, target) in [
        (role_id.clone(), ids.document.clone()),
        (metadata_role_id, access_right_id.clone()),
        (role_id.clone(), role_id.clone()),
        (ids.procedure, access_right_id.clone()),
        (query_id, access_right_id.clone()),
        (unknown_id.clone(), access_right_id),
        (role_id, unknown_id),
    ] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source,
                target,
                EdgeKind::Grants,
                vec![provenance("metadata.role.sales_manager#rights")],
            ))
            .expect("storage only validates endpoint existence");
    }

    let result = graph.validate();

    assert!(!result.is_valid());
    assert_eq!(
        result
            .issues()
            .iter()
            .filter(|issue| {
                issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
                    && issue.edge_kind() == Some(EdgeKind::Grants)
            })
            .count(),
        7
    );
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
