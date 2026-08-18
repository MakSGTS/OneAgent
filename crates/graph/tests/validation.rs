use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, ProducerId,
    Provenance, ResolutionState, SemanticGraph, SemanticGraphQuery, SemanticGraphReport,
    SemanticGraphSchema, SemanticGraphValidationCode, SemanticGraphValidationIssueKind,
    SemanticGraphValidationSeverity, SemanticGraphValidator, SemanticReferenceOutcome,
    SemanticReferenceStatistics,
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

fn metadata_kinds() -> [MetadataKind; 23] {
    [
        MetadataKind::Configuration,
        MetadataKind::Subsystem,
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::Enumeration,
        MetadataKind::CommonModule,
        MetadataKind::Report,
        MetadataKind::DataProcessor,
        MetadataKind::InformationRegister,
        MetadataKind::AccumulationRegister,
        MetadataKind::AccountingRegister,
        MetadataKind::CalculationRegister,
        MetadataKind::BusinessProcess,
        MetadataKind::Task,
        MetadataKind::Role,
        MetadataKind::CommonForm,
        MetadataKind::Form,
        MetadataKind::Command,
        MetadataKind::Template,
        MetadataKind::HttpService,
        MetadataKind::WebService,
        MetadataKind::XdtoPackage,
        MetadataKind::Unknown,
    ]
}

fn node_kinds() -> Vec<NodeKind> {
    let mut kinds = metadata_kinds().map(NodeKind::Metadata).to_vec();
    kinds.extend([
        NodeKind::Module,
        NodeKind::Procedure,
        NodeKind::Function,
        NodeKind::Query,
        NodeKind::Form,
        NodeKind::Command,
        NodeKind::Attribute,
        NodeKind::StandardAttribute,
        NodeKind::TabularSection,
        NodeKind::Dimension,
        NodeKind::Resource,
        NodeKind::Measure,
        NodeKind::Role,
        NodeKind::AccessRight,
        NodeKind::Subsystem,
        NodeKind::Unknown,
    ]);
    kinds
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

fn insert_accumulation_register(graph: &mut SemanticGraph) -> EntityId {
    let register_id = id("metadata.accumulation_register.cash_account_balance");

    graph.insert_node(GraphNode::new_with_provenance(
        register_id.clone(),
        name("CashAccountBalance"),
        NodeKind::Metadata(MetadataKind::AccumulationRegister),
        vec![provenance(
            "metadata.accumulation_register.cash_account_balance",
        )],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            id("configuration.main"),
            register_id.clone(),
            EdgeKind::Contains,
            vec![provenance("configuration.main")],
        ))
        .expect("accumulation register ownership edge must be stored");

    register_id
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
fn tabular_section_can_own_attribute() {
    let ids = FixtureIds::new();
    let tabular_section_id = id("metadata.document.sales:tabular_section:Products");
    let nested_attribute_id = id("metadata.document.sales:tabular_section:Products:attribute:Item");
    let mut graph = valid_graph(false);

    graph.insert_node(GraphNode::new_with_provenance(
        tabular_section_id.clone(),
        name("Products"),
        NodeKind::TabularSection,
        vec![provenance("metadata.document.sales")],
    ));
    graph.insert_node(GraphNode::new_with_provenance(
        nested_attribute_id.clone(),
        name("Item"),
        NodeKind::Attribute,
        vec![provenance("metadata.document.sales")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.document,
            tabular_section_id.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("TabularSection ownership edge must be stored");
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            tabular_section_id,
            nested_attribute_id,
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("nested Attribute ownership edge must be stored");

    let result = graph.validate();

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
fn reads_schema_accepts_first_slice_metadata_targets() {
    let schema = SemanticGraphSchema;

    for metadata_kind in [MetadataKind::Catalog, MetadataKind::InformationRegister] {
        assert!(schema.allows(
            NodeKind::Query,
            EdgeKind::Reads,
            NodeKind::Metadata(metadata_kind),
        ));
    }
}

#[test]
fn reads_schema_rejects_every_non_query_source_kind() {
    let schema = SemanticGraphSchema;

    for source_kind in node_kinds()
        .into_iter()
        .filter(|kind| *kind != NodeKind::Query)
    {
        assert!(
            !schema.allows(
                source_kind,
                EdgeKind::Reads,
                NodeKind::Metadata(MetadataKind::Catalog),
            ),
            "Reads unexpectedly accepts source kind {source_kind:?}",
        );
    }
}

#[test]
fn reads_schema_rejects_metadata_targets_outside_allowlist() {
    let schema = SemanticGraphSchema;

    for metadata_kind in metadata_kinds().into_iter().filter(|kind| {
        !matches!(
            kind,
            MetadataKind::Catalog | MetadataKind::InformationRegister
        )
    }) {
        assert!(
            !schema.allows(
                NodeKind::Query,
                EdgeKind::Reads,
                NodeKind::Metadata(metadata_kind),
            ),
            "Reads unexpectedly accepts metadata target kind {metadata_kind:?}",
        );
    }
}

#[test]
fn reads_schema_rejects_metadata_member_targets() {
    let schema = SemanticGraphSchema;

    for target_kind in [
        NodeKind::Attribute,
        NodeKind::StandardAttribute,
        NodeKind::TabularSection,
        NodeKind::Dimension,
        NodeKind::Resource,
        NodeKind::Measure,
    ] {
        assert!(!schema.allows(NodeKind::Query, EdgeKind::Reads, target_kind));
    }
}

#[test]
fn reads_schema_rejects_flat_semantic_targets() {
    let schema = SemanticGraphSchema;

    for target_kind in [
        NodeKind::Module,
        NodeKind::Procedure,
        NodeKind::Function,
        NodeKind::Query,
        NodeKind::Form,
        NodeKind::Command,
        NodeKind::Role,
        NodeKind::AccessRight,
        NodeKind::Subsystem,
    ] {
        assert!(!schema.allows(NodeKind::Query, EdgeKind::Reads, target_kind));
    }
}

#[test]
fn reads_schema_rejects_unknown_targets() {
    let schema = SemanticGraphSchema;

    for target_kind in [NodeKind::Unknown, NodeKind::Metadata(MetadataKind::Unknown)] {
        assert!(!schema.allows(NodeKind::Query, EdgeKind::Reads, target_kind));
    }
}

#[test]
fn reads_graph_validation_accepts_both_first_slice_target_kinds() {
    let query_id = id("query.reads");
    let catalog_id = id("metadata.catalog.products");
    let information_register_id = id("metadata.information_register.objects");
    let mut graph = SemanticGraph::new();

    for (node_id, node_name, node_kind) in [
        (query_id.clone(), "ReadsQuery", NodeKind::Query),
        (
            catalog_id.clone(),
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        ),
        (
            information_register_id.clone(),
            "Objects",
            NodeKind::Metadata(MetadataKind::InformationRegister),
        ),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            node_id,
            name(node_name),
            node_kind,
            vec![provenance("query.reads")],
        ));
    }
    for target_id in [catalog_id, information_register_id] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                query_id.clone(),
                target_id,
                EdgeKind::Reads,
                vec![provenance("query.reads")],
            ))
            .expect("Reads edge must be stored");
    }

    let result = graph.validate();

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn reads_graph_validation_reports_invalid_endpoint_contract() {
    let query_id = id("query.invalid_reads");
    let source_document_id = id("metadata.document.source");
    let target_document_id = id("metadata.document.target");
    let catalog_id = id("metadata.catalog.products");
    let mut graph = SemanticGraph::new();

    for (node_id, node_name, node_kind) in [
        (query_id.clone(), "InvalidReadsQuery", NodeKind::Query),
        (
            source_document_id.clone(),
            "SourceDocument",
            NodeKind::Metadata(MetadataKind::Document),
        ),
        (
            target_document_id.clone(),
            "TargetDocument",
            NodeKind::Metadata(MetadataKind::Document),
        ),
        (
            catalog_id.clone(),
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        ),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            node_id,
            name(node_name),
            node_kind,
            vec![provenance("query.invalid_reads")],
        ));
    }
    for (source_id, target_id) in [
        (query_id, target_document_id),
        (source_document_id, catalog_id),
    ] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source_id,
                target_id,
                EdgeKind::Reads,
                vec![provenance("query.invalid_reads")],
            ))
            .expect("storage only validates endpoint existence");
    }

    let result = graph.validate();
    let invalid_endpoints = result
        .issues()
        .iter()
        .filter(|issue| {
            issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
                && issue.severity() == SemanticGraphValidationSeverity::Error
                && issue.edge_kind() == Some(EdgeKind::Reads)
        })
        .collect::<Vec<_>>();

    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 2);
    assert_eq!(invalid_endpoints.len(), 2);
    assert!(invalid_endpoints.iter().any(|issue| {
        issue.source_kind() == Some(NodeKind::Query)
            && issue.target_kind() == Some(NodeKind::Metadata(MetadataKind::Document))
    }));
    assert!(invalid_endpoints.iter().any(|issue| {
        issue.source_kind() == Some(NodeKind::Metadata(MetadataKind::Document))
            && issue.target_kind() == Some(NodeKind::Metadata(MetadataKind::Catalog))
    }));
}

#[test]
fn writes_schema_accepts_procedure_to_accumulation_register() {
    let schema = SemanticGraphSchema;

    assert!(schema.allows(
        NodeKind::Procedure,
        EdgeKind::Writes,
        NodeKind::Metadata(MetadataKind::AccumulationRegister),
    ));
}

#[test]
fn writes_schema_rejects_every_non_procedure_source_kind() {
    let schema = SemanticGraphSchema;
    let kinds = node_kinds();

    for source_kind in kinds
        .iter()
        .copied()
        .filter(|kind| *kind != NodeKind::Procedure)
    {
        for target_kind in &kinds {
            assert!(
                !schema.allows(source_kind, EdgeKind::Writes, *target_kind),
                "Writes unexpectedly accepts {source_kind:?} -> {target_kind:?}",
            );
        }
    }
}

#[test]
fn writes_schema_rejects_every_non_accumulation_register_target_kind() {
    let schema = SemanticGraphSchema;

    for target_kind in node_kinds()
        .into_iter()
        .filter(|kind| *kind != NodeKind::Metadata(MetadataKind::AccumulationRegister))
    {
        assert!(
            !schema.allows(NodeKind::Procedure, EdgeKind::Writes, target_kind),
            "Writes unexpectedly accepts Procedure -> {target_kind:?}",
        );
    }
}

#[test]
fn writes_graph_accepts_provenance_backed_procedure_to_accumulation_register_edge() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);
    let register_id = insert_accumulation_register(&mut graph);

    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.procedure,
            register_id,
            EdgeKind::Writes,
            vec![provenance("metadata.document.sales:object_module#writes")],
        ))
        .expect("Writes edge must be stored");

    let result = graph.validate();

    assert!(result.is_valid());
    assert!(result.issues().is_empty());
}

#[test]
fn writes_graph_rejects_invalid_endpoints_with_exact_deterministic_context() {
    let ids = FixtureIds::new();
    let mut graph = valid_graph(false);
    let register_id = insert_accumulation_register(&mut graph);
    let invalid_edges = [
        (
            ids.function,
            register_id,
            NodeKind::Function,
            NodeKind::Metadata(MetadataKind::AccumulationRegister),
        ),
        (
            ids.procedure,
            ids.document,
            NodeKind::Procedure,
            NodeKind::Metadata(MetadataKind::Document),
        ),
    ];

    for (source_id, target_id, _, _) in &invalid_edges {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source_id.clone(),
                target_id.clone(),
                EdgeKind::Writes,
                vec![provenance("metadata.document.sales:object_module#writes")],
            ))
            .expect("storage only validates endpoint existence");
    }

    let result = graph.validate();
    let repeated = graph.validate();
    let invalid_endpoint_issues = result
        .issues()
        .iter()
        .filter(|issue| {
            issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
                && issue.edge_kind() == Some(EdgeKind::Writes)
        })
        .collect::<Vec<_>>();

    assert_eq!(result, repeated);
    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 2);
    assert_eq!(result.warning_count(), 0);
    assert_eq!(invalid_endpoint_issues.len(), 2);

    for (source_id, target_id, source_kind, target_kind) in invalid_edges {
        let edge_id = SemanticGraphQuery::edge_id(
            &NodeId::new(source_id.as_str()),
            &NodeId::new(target_id.as_str()),
            EdgeKind::Writes,
        );
        let issue = invalid_endpoint_issues
            .iter()
            .find(|issue| issue.edge_id() == Some(&edge_id))
            .expect("invalid Writes edge must retain its exact identity");
        let mut expected_nodes = vec![source_id, target_id];
        expected_nodes.sort();

        assert_eq!(issue.severity(), SemanticGraphValidationSeverity::Error);
        assert_eq!(issue.kind(), SemanticGraphValidationIssueKind::Semantic);
        assert_eq!(issue.nodes(), expected_nodes);
        assert_eq!(issue.source_kind(), Some(source_kind));
        assert_eq!(issue.target_kind(), Some(target_kind));
        assert_eq!(issue.invariant(), "edge endpoint schema");
        assert_eq!(
            issue.provenance(),
            [provenance("metadata.document.sales:object_module#writes")]
        );
    }
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
fn includes_accepts_only_subsystem_to_first_slice_metadata_kinds() {
    let schema = SemanticGraphSchema;
    let allowed = [
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::Enumeration,
        MetadataKind::CommonModule,
        MetadataKind::Report,
        MetadataKind::DataProcessor,
        MetadataKind::InformationRegister,
        MetadataKind::AccumulationRegister,
        MetadataKind::AccountingRegister,
        MetadataKind::CalculationRegister,
        MetadataKind::BusinessProcess,
        MetadataKind::Task,
        MetadataKind::Role,
        MetadataKind::Command,
        MetadataKind::CommonForm,
        MetadataKind::Template,
        MetadataKind::HttpService,
        MetadataKind::WebService,
        MetadataKind::XdtoPackage,
    ];

    for metadata_kind in allowed {
        assert!(schema.allows(
            NodeKind::Subsystem,
            EdgeKind::Includes,
            NodeKind::Metadata(metadata_kind),
        ));
    }

    for target_kind in [
        NodeKind::Metadata(MetadataKind::Configuration),
        NodeKind::Metadata(MetadataKind::Subsystem),
        NodeKind::Metadata(MetadataKind::Form),
        NodeKind::Metadata(MetadataKind::Unknown),
        NodeKind::Role,
        NodeKind::Subsystem,
        NodeKind::Unknown,
    ] {
        assert!(!schema.allows(NodeKind::Subsystem, EdgeKind::Includes, target_kind));
    }
    for source_kind in [
        NodeKind::Metadata(MetadataKind::Subsystem),
        NodeKind::Role,
        NodeKind::Unknown,
    ] {
        assert!(!schema.allows(
            source_kind,
            EdgeKind::Includes,
            NodeKind::Metadata(MetadataKind::Document),
        ));
    }
}

#[test]
fn includes_self_loop_is_rejected_defensively() {
    let subsystem_id = id("metadata.subsystem.sales:subsystem");
    let mut graph = SemanticGraph::new();
    graph.insert_node(GraphNode::new_with_provenance(
        subsystem_id.clone(),
        name("Sales"),
        NodeKind::Subsystem,
        vec![provenance("metadata.subsystem.sales")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            subsystem_id.clone(),
            subsystem_id,
            EdgeKind::Includes,
            vec![provenance("metadata.subsystem.sales#content")],
        ))
        .expect("storage allows self-loop");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::ForbiddenSelfLoop
            && issue.edge_kind() == Some(EdgeKind::Includes)
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
fn module_cannot_own_attribute() {
    let ids = FixtureIds::new();
    let nested_attribute_id = id("metadata.document.sales:attribute:WrongOwner");
    let mut graph = valid_graph(false);

    graph.insert_node(GraphNode::new_with_provenance(
        nested_attribute_id.clone(),
        name("WrongOwner"),
        NodeKind::Attribute,
        vec![provenance("metadata.document.sales")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.module,
            nested_attribute_id,
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales:object_module")],
        ))
        .expect("storage only validates endpoint existence");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::InvalidEdgeEndpoints
            && issue.source_kind() == Some(NodeKind::Module)
            && issue.target_kind() == Some(NodeKind::Attribute)
            && issue.edge_kind() == Some(EdgeKind::Contains)
    }));
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::InvalidOwner
            && issue.source_kind() == Some(NodeKind::Module)
            && issue.target_kind() == Some(NodeKind::Attribute)
    }));
}

#[test]
fn attribute_with_multiple_valid_owners_is_error() {
    let ids = FixtureIds::new();
    let tabular_section_id = id("metadata.document.sales:tabular_section:Products");
    let mut graph = valid_graph(false);

    graph.insert_node(GraphNode::new_with_provenance(
        tabular_section_id.clone(),
        name("Products"),
        NodeKind::TabularSection,
        vec![provenance("metadata.document.sales")],
    ));
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            ids.document,
            tabular_section_id.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("TabularSection ownership edge must be stored");
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            tabular_section_id,
            ids.attribute.clone(),
            EdgeKind::Contains,
            vec![provenance("metadata.document.sales")],
        ))
        .expect("second valid ownership edge must be stored");

    let result = graph.validate();

    assert!(!result.is_valid());
    assert!(result.issues().iter().any(|issue| {
        issue.code() == SemanticGraphValidationCode::MultipleOwners
            && issue.nodes().contains(&ids.attribute)
            && issue.edge_kind() == Some(EdgeKind::Contains)
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
