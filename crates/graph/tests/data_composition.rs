use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, DataCompositionFieldPayload, DataCompositionSchemaPayload, DataSetKind,
    DataSetPayload, EdgeKind, FactOrigin, GraphEdge, GraphNode, GraphNodePayload, NodeId, NodeKind,
    NodeModifiedAspect, ProducerId, Provenance, ResolutionState, SemanticGraph,
    SemanticImpactAnalyzer, SemanticImpactOptions, data_composition_field_id, data_set_id,
    data_set_query_id,
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
        ProducerId::new("oneagent.graph.data_composition.tests"),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn graph(
    main: bool,
    data_set_kind: DataSetKind,
    data_source: Option<&str>,
    field_path: &str,
    reverse: bool,
) -> SemanticGraph {
    let report_id = id("metadata.report.sales");
    let schema_id = id("schema-uuid");
    let data_set_id =
        data_set_id(&schema_id, &name("Sales")).expect("Data Set identity must be representable");
    let field_id = data_composition_field_id(&data_set_id, &name("Product"))
        .expect("Field identity must be representable");
    let query_id = data_set_query_id(&data_set_id).expect("Query identity must be representable");
    let nodes = [
        GraphNode::new_with_provenance(
            report_id.clone(),
            name("SalesReport"),
            NodeKind::Metadata(MetadataKind::Report),
            vec![provenance("report")],
        ),
        GraphNode::new_with_payload_and_provenance(
            schema_id.clone(),
            name("MainDataCompositionSchema"),
            NodeKind::DataCompositionSchema,
            GraphNodePayload::DataCompositionSchema(DataCompositionSchemaPayload::new(main)),
            vec![provenance("schema")],
        )
        .expect("Schema payload must be compatible"),
        GraphNode::new_with_payload_and_provenance(
            data_set_id.clone(),
            name("Sales"),
            NodeKind::DataSet,
            GraphNodePayload::DataSet(
                DataSetPayload::new(data_set_kind, data_source.map(name))
                    .expect("Data Set payload must be valid"),
            ),
            vec![provenance("data_set")],
        )
        .expect("Data Set payload must be compatible"),
        GraphNode::new_with_payload_and_provenance(
            field_id.clone(),
            name("Product"),
            NodeKind::DataCompositionField,
            GraphNodePayload::DataCompositionField(DataCompositionFieldPayload::new(name(
                field_path,
            ))),
            vec![provenance("field")],
        )
        .expect("Field payload must be compatible"),
        GraphNode::new_with_provenance(
            query_id.clone(),
            name("Query"),
            NodeKind::Query,
            vec![provenance("query")],
        ),
    ];
    let edges = [
        GraphEdge::new_with_provenance(
            report_id,
            schema_id,
            EdgeKind::Contains,
            vec![provenance("report_schema")],
        ),
        GraphEdge::new_with_provenance(
            id("schema-uuid"),
            data_set_id.clone(),
            EdgeKind::Contains,
            vec![provenance("schema_data_set")],
        ),
        GraphEdge::new_with_provenance(
            data_set_id.clone(),
            field_id,
            EdgeKind::Contains,
            vec![provenance("data_set_field")],
        ),
        GraphEdge::new_with_provenance(
            data_set_id,
            query_id,
            EdgeKind::Contains,
            vec![provenance("data_set_query")],
        ),
    ];
    let mut graph = SemanticGraph::new();
    if reverse {
        for node in nodes.into_iter().rev() {
            graph.insert_node(node);
        }
        for edge in edges.into_iter().rev() {
            graph.insert_edge(edge).expect("edge endpoints must exist");
        }
    } else {
        for node in nodes {
            graph.insert_node(node);
        }
        for edge in edges {
            graph.insert_edge(edge).expect("edge endpoints must exist");
        }
    }
    graph
}

#[test]
fn data_composition_graph_is_valid_queryable_reported_and_order_independent() {
    let normal = graph(
        true,
        DataSetKind::Query,
        Some("DataSource1"),
        "Products.Ref",
        false,
    );
    let reversed = graph(
        true,
        DataSetKind::Query,
        Some("DataSource1"),
        "Products.Ref",
        true,
    );
    let query = normal.query();
    let schema = query.nodes_by_kind(NodeKind::DataCompositionSchema);
    let data_sets = query.nodes_by_kind(NodeKind::DataSet);
    let fields = query.nodes_by_kind(NodeKind::DataCompositionField);
    let query_nodes = query.nodes_by_kind(NodeKind::Query);

    assert!(normal.diff(&reversed).is_empty());
    assert_eq!(normal.report(), reversed.report());
    assert!(normal.validate().is_valid());
    assert_eq!(schema.len(), 1);
    assert_eq!(data_sets.len(), 1);
    assert_eq!(fields.len(), 1);
    assert_eq!(query_nodes.len(), 1);
    assert_eq!(
        query
            .owner(&NodeId::new(schema[0].id().as_str()))
            .expect("Report must own Schema")
            .kind(),
        NodeKind::Metadata(MetadataKind::Report)
    );
    assert_eq!(
        query
            .children_by_kind(
                &NodeId::new(data_sets[0].id().as_str()),
                NodeKind::DataCompositionField,
            )
            .len(),
        1
    );
    assert_eq!(
        query
            .children_by_kind(&NodeId::new(data_sets[0].id().as_str()), NodeKind::Query)
            .len(),
        1
    );
    assert!(
        query
            .direct_dependencies(&NodeId::new(data_sets[0].id().as_str()))
            .is_empty()
    );
    assert_eq!(normal.report().nodes().total(), 5);
    assert_eq!(normal.report().edges().by_kind()[&EdgeKind::Contains], 4);
}

#[test]
fn data_composition_payload_changes_preserve_identity_and_modify_content() {
    let before = graph(
        false,
        DataSetKind::Query,
        Some("DataSource1"),
        "Products.Ref",
        false,
    );
    let main_changed = graph(
        true,
        DataSetKind::Query,
        Some("DataSource1"),
        "Products.Ref",
        false,
    );
    let kind_changed = graph(
        false,
        DataSetKind::Object,
        Some("DataSource1"),
        "Products.Ref",
        false,
    );
    let source_changed = graph(
        false,
        DataSetKind::Query,
        Some("DataSource2"),
        "Products.Ref",
        false,
    );
    let field_changed = graph(
        false,
        DataSetKind::Query,
        Some("DataSource1"),
        "Products.Code",
        false,
    );

    for diff in [
        before.diff(&main_changed),
        before.diff(&kind_changed),
        before.diff(&source_changed),
        before.diff(&field_changed),
    ] {
        assert!(diff.added_nodes().is_empty());
        assert!(diff.removed_nodes().is_empty());
        assert_eq!(diff.modified_nodes().len(), 1);
        assert_eq!(
            diff.modified_nodes()[0].modified_aspects(),
            &[NodeModifiedAspect::SemanticContent]
        );
    }

    let diff = before.diff(&main_changed);
    let impact = SemanticImpactAnalyzer::analyze(
        &before,
        &main_changed,
        &diff,
        &SemanticImpactOptions::new(4),
    )
    .expect("Data Composition impact analysis must succeed");
    assert_eq!(impact.affected_nodes().len(), 1);
    assert_eq!(impact.affected_nodes()[0].node_id().as_str(), "schema-uuid");
}
