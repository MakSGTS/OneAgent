use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    EdgeId, EdgeKind, GraphEdge, GraphNode, GraphNodePayload, NodeId, NodeKind, SemanticGraph,
    SemanticGraphEdgeFilter, SemanticGraphQuery, SemanticGraphTraversalDirection,
    SemanticGraphTraversalOptions,
};
use oneagent_metadata::{
    CommonMetadataPayload, MetadataKind, MetadataMemberPayload, MetadataPayload,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

const fn const_query(graph: &SemanticGraph) -> SemanticGraphQuery<'_> {
    SemanticGraphQuery::new(graph)
}

fn node_ids<'graph>(nodes: impl IntoIterator<Item = &'graph GraphNode>) -> Vec<EntityId> {
    nodes.into_iter().map(|node| node.id().clone()).collect()
}

fn sorted_edge_ids<'graph>(edges: impl IntoIterator<Item = &'graph GraphEdge>) -> Vec<EdgeId> {
    let mut ids = edges
        .into_iter()
        .map(|edge| {
            SemanticGraphQuery::edge_id(
                &node_id(edge.source().as_str()),
                &node_id(edge.target().as_str()),
                edge.kind(),
            )
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

struct QueryFixtureIds {
    configuration: EntityId,
    document_sales: EntityId,
    document_returns: EntityId,
    module: EntityId,
    post: EntityId,
    validate: EntityId,
    calculate: EntityId,
    company: EntityId,
}

impl QueryFixtureIds {
    fn new() -> Self {
        Self {
            configuration: id("configuration.main"),
            document_sales: id("metadata.document.sales"),
            document_returns: id("metadata.document.returns"),
            module: id("metadata.document.sales:object_module"),
            post: id("metadata.document.sales:object_module:procedure:Post"),
            validate: id("metadata.document.sales:object_module:procedure:Validate"),
            calculate: id("metadata.document.sales:object_module:function:Calculate"),
            company: id("metadata.document.sales:attribute:Company"),
        }
    }
}

fn graph_fixture(reverse: bool) -> SemanticGraph {
    let ids = QueryFixtureIds::new();
    let mut graph = SemanticGraph::new();

    insert_nodes(&mut graph, &ids, reverse);
    insert_edges(&mut graph, ids, reverse);

    graph
}

fn insert_nodes(graph: &mut SemanticGraph, ids: &QueryFixtureIds, reverse: bool) {
    let nodes = [
        GraphNode::new(
            ids.configuration.clone(),
            name("Configuration"),
            NodeKind::Metadata(MetadataKind::Configuration),
        ),
        GraphNode::new(
            ids.document_sales.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ),
        GraphNode::new(
            ids.document_returns.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ),
        GraphNode::new(ids.module.clone(), name("ObjectModule"), NodeKind::Module),
        GraphNode::new(ids.post.clone(), name("Post"), NodeKind::Procedure),
        GraphNode::new(ids.validate.clone(), name("Validate"), NodeKind::Procedure),
        GraphNode::new(ids.calculate.clone(), name("Calculate"), NodeKind::Function),
        GraphNode::new(ids.company.clone(), name("Company"), NodeKind::Attribute),
    ];

    if reverse {
        for node in nodes.into_iter().rev() {
            graph.insert_node(node);
        }
    } else {
        for node in nodes {
            graph.insert_node(node);
        }
    }
}

fn insert_edges(graph: &mut SemanticGraph, ids: QueryFixtureIds, reverse: bool) {
    let edges = [
        GraphEdge::new(
            ids.configuration.clone(),
            ids.document_sales.clone(),
            EdgeKind::Contains,
        ),
        GraphEdge::new(
            ids.configuration,
            ids.document_returns.clone(),
            EdgeKind::Contains,
        ),
        GraphEdge::new(
            ids.document_sales.clone(),
            ids.module.clone(),
            EdgeKind::Contains,
        ),
        GraphEdge::new(
            ids.document_sales.clone(),
            ids.company.clone(),
            EdgeKind::Contains,
        ),
        GraphEdge::new(ids.module.clone(), ids.post.clone(), EdgeKind::Contains),
        GraphEdge::new(ids.module.clone(), ids.validate.clone(), EdgeKind::Contains),
        GraphEdge::new(ids.module, ids.calculate.clone(), EdgeKind::Contains),
        GraphEdge::new(ids.post.clone(), ids.validate.clone(), EdgeKind::Calls),
        GraphEdge::new(ids.post.clone(), ids.calculate.clone(), EdgeKind::Calls),
        GraphEdge::new(ids.post.clone(), ids.document_returns, EdgeKind::References),
        GraphEdge::new(ids.post.clone(), ids.company, EdgeKind::Reads),
        GraphEdge::new(ids.validate.clone(), ids.post.clone(), EdgeKind::Calls),
        GraphEdge::new(ids.calculate, ids.post, EdgeKind::DependsOn),
    ];

    if reverse {
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
fn const_query_construction_preserves_empty_snapshot_behavior() {
    let graph = SemanticGraph::new();
    let query = const_query(&graph);

    assert!(query.nodes().is_empty());
    assert!(query.edges().is_empty());
    assert!(query.nodes_by_name(&name("Missing")).is_empty());
    assert!(query.outgoing_edges(&node_id("missing")).is_empty());
    assert!(query.owner(&node_id("missing")).is_none());
}

#[test]
fn indexed_primitives_match_independent_canonical_scans() {
    let graph = graph_fixture(false);
    let ids = QueryFixtureIds::new();
    let query = graph.query();
    let repeated = graph.query();
    let cloned = query.clone();
    let post = node_id(ids.post.as_str());
    let company = node_id(ids.company.as_str());

    assert_eq!(query.node_by_entity_id(&ids.post), graph.node(&ids.post));
    assert_eq!(node_ids(query.nodes()), node_ids(graph.nodes()));
    assert_eq!(
        node_ids(query.nodes_by_kind(NodeKind::Procedure)),
        node_ids(
            graph
                .nodes()
                .filter(|node| node.kind() == NodeKind::Procedure)
        )
    );
    assert_eq!(
        node_ids(query.nodes_by_name(&name("Sales"))),
        node_ids(graph.nodes().filter(|node| node.name() == &name("Sales")))
    );
    assert_eq!(
        node_ids(query.nodes_by_name_and_kind(
            &name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        )),
        node_ids(graph.nodes().filter(|node| {
            node.name() == &name("Sales")
                && node.kind() == NodeKind::Metadata(MetadataKind::Document)
        }))
    );

    let all_edge_ids = sorted_edge_ids(graph.edges());
    assert_eq!(sorted_edge_ids(query.edges()), all_edge_ids);
    assert!(all_edge_ids.iter().all(|id| query.edge(id).is_some()));
    assert_eq!(
        sorted_edge_ids(query.edges_by_kind(EdgeKind::Calls)),
        sorted_edge_ids(graph.edges().filter(|edge| edge.kind() == EdgeKind::Calls))
    );
    assert_eq!(
        sorted_edge_ids(query.outgoing_edges(&post)),
        sorted_edge_ids(graph.outgoing(&ids.post))
    );
    assert_eq!(
        sorted_edge_ids(query.incoming_edges(&company)),
        sorted_edge_ids(graph.incoming(&ids.company))
    );
    assert_eq!(
        sorted_edge_ids(query.outgoing_edges_by_kind(&post, EdgeKind::Calls)),
        sorted_edge_ids(
            graph
                .outgoing(&ids.post)
                .into_iter()
                .filter(|edge| edge.kind() == EdgeKind::Calls)
        )
    );
    assert_eq!(
        sorted_edge_ids(query.incoming_edges_by_kind(&company, EdgeKind::Contains)),
        sorted_edge_ids(
            graph
                .incoming(&ids.company)
                .into_iter()
                .filter(|edge| edge.kind() == EdgeKind::Contains)
        )
    );

    assert_eq!(node_ids(query.nodes()), node_ids(repeated.nodes()));
    assert_eq!(
        sorted_edge_ids(query.edges()),
        sorted_edge_ids(cloned.edges())
    );
}

#[test]
fn indexed_containment_matches_multiple_owner_canonical_scans() {
    let mut graph = graph_fixture(false);
    let ids = QueryFixtureIds::new();
    graph
        .insert_edge(GraphEdge::new(
            ids.document_returns.clone(),
            ids.company.clone(),
            EdgeKind::Contains,
        ))
        .expect("multiple ownership must remain representable");

    let query = graph.query();
    let company = node_id(ids.company.as_str());
    let configuration = node_id(ids.configuration.as_str());
    let mut canonical_owners = graph
        .incoming(&ids.company)
        .into_iter()
        .filter(|edge| edge.kind() == EdgeKind::Contains)
        .filter_map(|edge| graph.node(edge.source()))
        .collect::<Vec<_>>();
    canonical_owners.sort_by_key(|node| node.id());
    canonical_owners.dedup_by_key(|node| node.id());

    assert_eq!(node_ids(query.owners(&company)), node_ids(canonical_owners));
    assert!(query.owner(&company).is_none());
    assert_eq!(
        sorted_edge_ids(query.owner_edges(&company)),
        sorted_edge_ids(
            graph
                .incoming(&ids.company)
                .into_iter()
                .filter(|edge| edge.kind() == EdgeKind::Contains)
        )
    );

    let mut canonical_children = graph
        .outgoing(&ids.configuration)
        .into_iter()
        .filter(|edge| edge.kind() == EdgeKind::Contains)
        .filter_map(|edge| graph.node(edge.target()))
        .collect::<Vec<_>>();
    canonical_children.sort_by_key(|node| node.id());
    canonical_children.dedup_by_key(|node| node.id());
    assert_eq!(
        node_ids(query.children(&configuration)),
        node_ids(canonical_children)
    );
}

#[test]
fn node_lookup_and_exact_name_queries_are_deterministic() {
    let graph = graph_fixture(false);
    let query = graph.query();
    let post_id = node_id("metadata.document.sales:object_module:procedure:Post");

    let before_report = graph.report();
    let post = query.node(&post_id).expect("post node must exist");
    let repeated = query.node(&post_id).expect("post node must still exist");
    let sales = query.nodes_by_name(&name("Sales"));
    let procedures = query.nodes_by_name_and_kind(&name("Sales"), NodeKind::Procedure);

    assert_eq!(post.id().as_str(), post_id.as_str());
    assert_eq!(post.kind(), NodeKind::Procedure);
    assert_eq!(post, repeated);
    assert!(query.contains_node(&post_id));
    assert!(!query.contains_node(&node_id("metadata.document.missing")));
    assert!(query.node(&node_id("metadata.document.missing")).is_none());
    assert_eq!(sales.len(), 2);
    assert_eq!(sales[0].id().as_str(), "metadata.document.returns");
    assert_eq!(sales[1].id().as_str(), "metadata.document.sales");
    assert!(procedures.is_empty());
    assert!(query.nodes_by_name(&name("Sale")).is_empty());
    assert_eq!(graph.report(), before_report);
}

#[test]
fn kind_queries_do_not_depend_on_insertion_order() {
    let normal = graph_fixture(false);
    let reversed = graph_fixture(true);
    let normal_query = normal.query();
    let reversed_query = reversed.query();

    let normal_ids = normal_query
        .nodes_by_kind(NodeKind::Procedure)
        .into_iter()
        .map(|node| node.id().clone())
        .collect::<Vec<_>>();
    let reversed_ids = reversed_query
        .nodes_by_kind(NodeKind::Procedure)
        .into_iter()
        .map(|node| node.id().clone())
        .collect::<Vec<_>>();

    assert_eq!(normal_ids, reversed_ids);
    assert_eq!(
        normal_ids,
        vec![
            id("metadata.document.sales:object_module:procedure:Post"),
            id("metadata.document.sales:object_module:procedure:Validate"),
        ]
    );
    assert!(normal_query.nodes_by_kind(NodeKind::Query).is_empty());
}

#[test]
fn query_finds_access_right_nodes_by_kind() {
    let access_right_id =
        id("access_right:resource#23:metadata.document.sales;right#10:right.read");
    let mut graph = SemanticGraph::new();

    graph.insert_node(GraphNode::new(
        access_right_id.clone(),
        name("right.read on metadata.document.sales"),
        NodeKind::AccessRight,
    ));

    let nodes = graph.query().nodes_by_kind(NodeKind::AccessRight);

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id(), &access_right_id);
    assert_eq!(nodes[0].kind(), NodeKind::AccessRight);
}

#[test]
fn owner_and_children_queries_use_contains_edges_only() {
    let graph = graph_fixture(false);
    let query = graph.query();
    let module_id = node_id("metadata.document.sales:object_module");
    let post_id = node_id("metadata.document.sales:object_module:procedure:Post");
    let configuration_id = node_id("configuration.main");

    let owner = query.owner(&post_id).expect("post owner must exist");
    let owner_edges = query.owner_edges(&post_id);
    let children = query.children(&module_id);
    let procedure_children = query.children_by_kind(&module_id, NodeKind::Procedure);

    assert_eq!(owner.id().as_str(), module_id.as_str());
    assert_eq!(owner_edges.len(), 1);
    assert_eq!(owner_edges[0].kind(), EdgeKind::Contains);
    assert!(query.owner(&configuration_id).is_none());
    assert_eq!(
        children
            .into_iter()
            .map(|node| node.id().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec![
            "metadata.document.sales:object_module:function:Calculate",
            "metadata.document.sales:object_module:procedure:Post",
            "metadata.document.sales:object_module:procedure:Validate",
        ]
    );
    assert_eq!(procedure_children.len(), 2);
}

#[test]
fn edge_lookup_and_adjacency_queries_are_sorted_by_edge_id() {
    let graph = graph_fixture(false);
    let query = graph.query();
    let post_id = node_id("metadata.document.sales:object_module:procedure:Post");
    let calculate_id = node_id("metadata.document.sales:object_module:function:Calculate");
    let edge_id = SemanticGraphQuery::edge_id(&post_id, &calculate_id, EdgeKind::Calls);

    let edge = query.edge(&edge_id).expect("call edge must exist");
    let outgoing = query.outgoing_edges(&post_id);
    let incoming = query.incoming_edges(&calculate_id);
    let outgoing_calls = query.outgoing_edges_by_kind(&post_id, EdgeKind::Calls);
    let incoming_calls = query.incoming_edges_by_kind(&calculate_id, EdgeKind::Calls);

    assert!(query.contains_edge(&edge_id));
    assert!(!query.contains_edge(&oneagent_graph::EdgeId::new("edge:missing")));
    assert_eq!(edge.source().as_str(), post_id.as_str());
    assert_eq!(edge.target().as_str(), calculate_id.as_str());
    assert_eq!(edge.kind(), EdgeKind::Calls);
    assert_eq!(outgoing.len(), 4);
    assert_eq!(incoming.len(), 2);
    assert_eq!(outgoing_calls.len(), 2);
    assert_eq!(incoming_calls.len(), 1);
    assert!(query.outgoing_edges(&node_id("missing")).is_empty());
}

#[test]
fn neighbors_deduplicate_nodes_and_apply_edge_filters() {
    let mut graph = graph_fixture(false);
    let post = id("metadata.document.sales:object_module:procedure:Post");
    let calculate = id("metadata.document.sales:object_module:function:Calculate");

    graph
        .insert_edge(GraphEdge::new(
            post.clone(),
            calculate.clone(),
            EdgeKind::References,
        ))
        .expect("second semantic relation must be valid");

    let query = graph.query();
    let post_id = node_id(post.as_str());
    let calculate_id = node_id(calculate.as_str());
    let downstream = query.downstream_neighbors(&post_id);
    let calls = query.downstream_neighbors_by_kind(&post_id, EdgeKind::Calls);
    let upstream = query.upstream_neighbors(&calculate_id);

    assert_eq!(
        downstream
            .into_iter()
            .map(|node| node.id().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec![
            "metadata.document.returns",
            "metadata.document.sales:attribute:Company",
            "metadata.document.sales:object_module:function:Calculate",
            "metadata.document.sales:object_module:procedure:Validate",
        ]
    );
    assert_eq!(calls.len(), 2);
    assert!(upstream.iter().any(|node| node.id() == &post));
}

#[test]
fn dependencies_and_usages_follow_dependency_edge_policy() {
    let graph = graph_fixture(false);
    let query = graph.query();
    let post_id = node_id("metadata.document.sales:object_module:procedure:Post");
    let calculate_id = node_id("metadata.document.sales:object_module:function:Calculate");

    let dependencies = query.direct_dependencies(&post_id);
    let call_dependencies = query.direct_dependencies_by_kind(&post_id, EdgeKind::Calls);
    let ownership_dependencies = query.direct_dependencies_by_kind(&post_id, EdgeKind::Contains);
    let usages = query.direct_usages(&calculate_id);

    assert!(SemanticGraphQuery::is_dependency_edge_kind(EdgeKind::Calls));
    assert!(!SemanticGraphQuery::is_dependency_edge_kind(
        EdgeKind::Contains
    ));
    assert!(!SemanticGraphQuery::is_dependency_edge_kind(
        EdgeKind::Includes
    ));
    assert_eq!(dependencies.len(), 4);
    assert_eq!(call_dependencies.len(), 2);
    assert!(ownership_dependencies.is_empty());
    assert_eq!(usages.len(), 1);
    assert_eq!(usages[0].node().id().as_str(), post_id.as_str());
    assert_eq!(usages[0].edge().kind(), EdgeKind::Calls);
    assert_eq!(
        usages[0].direction(),
        SemanticGraphTraversalDirection::Upstream
    );
}

#[test]
fn opens_participates_in_generic_filters_dependencies_usages_and_traversal() {
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
        .insert_edge(GraphEdge::new(
            procedure.clone(),
            form.clone(),
            EdgeKind::Opens,
        ))
        .expect("Opens edge must be stored");

    let query = graph.query();
    let procedure_id = node_id(procedure.as_str());
    let form_id = node_id(form.as_str());
    let traversal = query.traverse(
        &procedure_id,
        &SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 1)
            .with_edge_filter(SemanticGraphEdgeFilter::only(EdgeKind::Opens)),
    );

    assert!(SemanticGraphQuery::is_dependency_edge_kind(EdgeKind::Opens));
    assert_eq!(
        query
            .downstream_neighbors_by_kind(&procedure_id, EdgeKind::Opens)
            .len(),
        1
    );
    assert_eq!(query.direct_dependencies(&procedure_id).len(), 1);
    assert_eq!(
        query
            .direct_dependencies_by_kind(&procedure_id, EdgeKind::Opens)
            .len(),
        1
    );
    assert_eq!(query.direct_usages(&form_id).len(), 1);
    assert_eq!(
        query.direct_usages(&form_id)[0].edge().kind(),
        EdgeKind::Opens
    );
    assert_eq!(traversal.len(), 1);
    assert_eq!(traversal[0].node_id(), &form_id);
}

#[test]
fn bounded_breadth_first_traversal_handles_cycles_and_depth() {
    let normal = graph_fixture(false);
    let reversed = graph_fixture(true);
    let start = node_id("metadata.document.sales:object_module:procedure:Post");
    let depth_zero_include =
        SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 0)
            .with_include_start(true);
    let depth_zero_exclude =
        SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 0);
    let depth_one =
        SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 1)
            .with_edge_filter(SemanticGraphEdgeFilter::any([
                EdgeKind::Calls,
                EdgeKind::DependsOn,
            ]));
    let depth_two =
        SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 2)
            .with_edge_filter(SemanticGraphEdgeFilter::any([
                EdgeKind::Calls,
                EdgeKind::DependsOn,
            ]));

    let normal_depth_two = normal.query().traverse(&start, &depth_two);
    let reversed_depth_two = reversed.query().traverse(&start, &depth_two);

    assert_eq!(
        normal.query().traverse(&start, &depth_zero_include).len(),
        1
    );
    assert!(
        normal
            .query()
            .traverse(&start, &depth_zero_exclude)
            .is_empty()
    );
    assert_eq!(normal.query().traverse(&start, &depth_one).len(), 2);
    assert_eq!(
        normal_depth_two
            .iter()
            .map(|node| (node.node_id().as_str().to_owned(), node.depth()))
            .collect::<Vec<_>>(),
        vec![
            (
                "metadata.document.sales:object_module:function:Calculate".to_owned(),
                1,
            ),
            (
                "metadata.document.sales:object_module:procedure:Validate".to_owned(),
                1,
            ),
        ]
    );
    assert_eq!(normal_depth_two, reversed_depth_two);
    assert!(
        normal_depth_two
            .iter()
            .all(|node| node.via_edge().is_some())
    );
    assert!(
        normal
            .query()
            .traverse(&node_id("missing"), &depth_two)
            .is_empty()
    );
}

#[test]
fn upstream_traversal_and_self_loop_are_deterministic() {
    let mut graph = graph_fixture(false);
    let post = id("metadata.document.sales:object_module:procedure:Post");

    graph
        .insert_edge(GraphEdge::new(
            post.clone(),
            post.clone(),
            EdgeKind::DependsOn,
        ))
        .expect("dependency self-loop is representable");

    let query = graph.query();
    let post_id = node_id(post.as_str());
    let upstream_options =
        SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Upstream, 2)
            .with_edge_kind(EdgeKind::Calls);
    let incoming = query.incoming_edges(&post_id);
    let outgoing = query.outgoing_edges(&post_id);
    let upstream = query.traverse(&post_id, &upstream_options);

    assert!(incoming.iter().any(|edge| edge.source() == edge.target()));
    assert!(outgoing.iter().any(|edge| edge.source() == edge.target()));
    assert_eq!(
        query
            .downstream_neighbors_by_kind(&post_id, EdgeKind::DependsOn)
            .len(),
        1
    );
    assert!(
        upstream
            .iter()
            .all(|node| node.node_id().as_str() != post_id.as_str())
    );
    assert_eq!(upstream[0].depth(), 1);
}

#[test]
fn node_lookup_exposes_payload_without_changing_exact_name_behavior() {
    let metadata_id = id("metadata.catalog.products");
    let mut graph = SemanticGraph::new();
    graph.insert_node(
        GraphNode::new_with_payload(
            metadata_id.clone(),
            name("Products"),
            NodeKind::Metadata(MetadataKind::Catalog),
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(Some("Goods".to_owned())),
                None,
            )),
        )
        .expect("Catalog common payload must be valid"),
    );

    let query = graph.query();
    let node = query
        .node(&node_id(metadata_id.as_str()))
        .expect("metadata node must exist");

    assert_eq!(
        node.metadata_payload()
            .expect("metadata payload must exist")
            .common()
            .synonym(),
        Some("Goods")
    );
    assert_eq!(query.nodes_by_name(&name("Products")), vec![node]);
    assert!(query.nodes_by_name(&name("Goods")).is_empty());
}

#[test]
fn member_payload_is_visible_without_becoming_an_exact_name_index() {
    let attribute_id = id("metadata.document.sales:attribute:Company");
    let mut graph = SemanticGraph::new();
    graph.insert_node(
        GraphNode::new_with_payload(
            attribute_id.clone(),
            name("Company"),
            NodeKind::Attribute,
            GraphNodePayload::MetadataMember(MetadataMemberPayload::new(Some(
                "Organization".to_owned(),
            ))),
        )
        .expect("Attribute member payload must be valid"),
    );

    let query = graph.query();
    let node = query
        .node(&node_id(attribute_id.as_str()))
        .expect("Attribute node must exist");

    assert_eq!(
        node.metadata_member_payload()
            .expect("member payload must exist")
            .synonym(),
        Some("Organization")
    );
    assert_eq!(query.nodes_by_name(&name("Company")), vec![node]);
    assert!(query.nodes_by_name(&name("Organization")).is_empty());
}
