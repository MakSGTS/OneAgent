use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, GraphNodePayload,
    HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, NodeId, NodeKind, NodeModifiedAspect,
    ProducerId, Provenance, ResolutionState, SemanticGraph, SemanticGraphSchema,
    SemanticGraphValidator, SemanticReference, SemanticReferenceCategory, SemanticReferenceRequest,
    SemanticReferenceRequestLedger, WebServiceOperationPayload, WebServiceParameterDirection,
    WebServiceParameterPayload, XdtoTypeKind, XdtoTypePayload, XdtoTypeReference, xdto_type_id,
};
use oneagent_metadata::{
    CommonMetadataPayload, HttpServiceMetadataPayload, MetadataKind, MetadataPayload,
    MetadataSpecificPayload, WebServiceMetadataPayload, WebServiceXdtoPackage,
    XdtoPackageMetadataPayload,
};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn provenance(source: &str, resolution: ResolutionState) -> Provenance {
    Provenance::new(
        Some(id(source)),
        ProducerId::new("oneagent.graph.xdto-service.tests"),
        if resolution == ResolutionState::Resolved {
            FactOrigin::Resolved
        } else {
            FactOrigin::Declared
        },
        Confidence::Exact,
        resolution,
    )
}

fn node_provenance(source: &str) -> Vec<Provenance> {
    vec![provenance(source, ResolutionState::NotApplicable)]
}

struct FixtureIds {
    package: EntityId,
    xdto_type: EntityId,
    http_service: EntityId,
    url_template: EntityId,
    http_method: EntityId,
    http_module: EntityId,
    http_function: EntityId,
    web_service: EntityId,
    operation: EntityId,
    parameter: EntityId,
    web_module: EntityId,
    web_function: EntityId,
}

impl FixtureIds {
    fn new() -> Self {
        let package = id("metadata.xdto.exchange");
        Self {
            xdto_type: xdto_type_id(&package, &name("Result"))
                .expect("XDTO type identity must be valid"),
            package,
            http_service: id("metadata.http.api"),
            url_template: id("uuid.http.url-template"),
            http_method: id("uuid.http.method"),
            http_module: id("metadata.http.api:common_module"),
            http_function: id("metadata.http.api:common_module:function:Handle"),
            web_service: id("metadata.web.exchange"),
            operation: id("uuid.web.operation"),
            parameter: id("uuid.web.parameter"),
            web_module: id("metadata.web.exchange:common_module"),
            web_function: id("metadata.web.exchange:common_module:function:Exchange"),
        }
    }
}

fn metadata_payload(kind: MetadataKind) -> MetadataPayload {
    let specific = match kind {
        MetadataKind::XdtoPackage => {
            MetadataSpecificPayload::XdtoPackage(XdtoPackageMetadataPayload::new("urn:exchange"))
        }
        MetadataKind::HttpService => {
            MetadataSpecificPayload::HttpService(HttpServiceMetadataPayload::new("api"))
        }
        MetadataKind::WebService => {
            MetadataSpecificPayload::WebService(WebServiceMetadataPayload::new(
                "urn:web",
                [WebServiceXdtoPackage::Repository(name("Exchange"))],
            ))
        }
        _ => panic!("unsupported fixture metadata kind"),
    };
    MetadataPayload::new(CommonMetadataPayload::empty(), Some(specific))
}

#[allow(clippy::too_many_lines)]
fn graph(explicit_http_method: Option<&str>, reverse: bool) -> SemanticGraph {
    let ids = FixtureIds::new();
    let type_reference = XdtoTypeReference::new("urn:exchange", name("Result"));
    let nodes = vec![
        GraphNode::new_with_payload_and_provenance(
            ids.package.clone(),
            name("Exchange"),
            NodeKind::Metadata(MetadataKind::XdtoPackage),
            GraphNodePayload::Metadata(metadata_payload(MetadataKind::XdtoPackage)),
            node_provenance("source.package"),
        )
        .expect("XDTO Package payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.xdto_type.clone(),
            name("Result"),
            NodeKind::XdtoType,
            GraphNodePayload::XdtoType(XdtoTypePayload::new(XdtoTypeKind::Object)),
            node_provenance("source.xdto-type"),
        )
        .expect("XDTO Type payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.http_service.clone(),
            name("Api"),
            NodeKind::Metadata(MetadataKind::HttpService),
            GraphNodePayload::Metadata(metadata_payload(MetadataKind::HttpService)),
            node_provenance("source.http-service"),
        )
        .expect("HTTP Service payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.url_template.clone(),
            name("Resource"),
            NodeKind::HttpServiceUrlTemplate,
            GraphNodePayload::HttpServiceUrlTemplate(HttpServiceUrlTemplatePayload::new("/{id}")),
            node_provenance("source.url-template"),
        )
        .expect("URL Template payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.http_method.clone(),
            name("POST"),
            NodeKind::HttpServiceMethod,
            GraphNodePayload::HttpServiceMethod(HttpServiceMethodPayload::new(
                explicit_http_method.map(name),
            )),
            node_provenance("source.http-method"),
        )
        .expect("HTTP Method payload must be valid"),
        GraphNode::new_with_provenance(
            ids.http_module.clone(),
            name("Module"),
            NodeKind::Module,
            node_provenance("source.http-module"),
        ),
        GraphNode::new_with_provenance(
            ids.http_function.clone(),
            name("Handle"),
            NodeKind::Function,
            node_provenance("source.http-function"),
        ),
        GraphNode::new_with_payload_and_provenance(
            ids.web_service.clone(),
            name("ExchangeService"),
            NodeKind::Metadata(MetadataKind::WebService),
            GraphNodePayload::Metadata(metadata_payload(MetadataKind::WebService)),
            node_provenance("source.web-service"),
        )
        .expect("Web Service payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.operation.clone(),
            name("Exchange"),
            NodeKind::WebServiceOperation,
            GraphNodePayload::WebServiceOperation(WebServiceOperationPayload::new(
                type_reference.clone(),
                Some(true),
            )),
            node_provenance("source.web-operation"),
        )
        .expect("Web Operation payload must be valid"),
        GraphNode::new_with_payload_and_provenance(
            ids.parameter.clone(),
            name("Request"),
            NodeKind::WebServiceParameter,
            GraphNodePayload::WebServiceParameter(WebServiceParameterPayload::new(
                type_reference,
                None,
                Some(WebServiceParameterDirection::InOut),
            )),
            node_provenance("source.web-parameter"),
        )
        .expect("Web Parameter payload must be valid"),
        GraphNode::new_with_provenance(
            ids.web_module.clone(),
            name("Module"),
            NodeKind::Module,
            node_provenance("source.web-module"),
        ),
        GraphNode::new_with_provenance(
            ids.web_function.clone(),
            name("Exchange"),
            NodeKind::Function,
            node_provenance("source.web-function"),
        ),
    ];
    let edges = vec![
        (
            ids.package,
            ids.xdto_type,
            EdgeKind::Contains,
            "edge.package-type",
        ),
        (
            ids.http_service.clone(),
            ids.url_template.clone(),
            EdgeKind::Contains,
            "edge.http-url",
        ),
        (
            ids.url_template,
            ids.http_method.clone(),
            EdgeKind::Contains,
            "edge.url-method",
        ),
        (
            ids.http_service,
            ids.http_module.clone(),
            EdgeKind::Contains,
            "edge.http-module",
        ),
        (
            ids.http_module,
            ids.http_function.clone(),
            EdgeKind::Contains,
            "edge.http-function",
        ),
        (
            ids.web_service.clone(),
            ids.operation.clone(),
            EdgeKind::Contains,
            "edge.web-operation",
        ),
        (
            ids.operation.clone(),
            ids.parameter.clone(),
            EdgeKind::Contains,
            "edge.operation-parameter",
        ),
        (
            ids.web_service.clone(),
            ids.web_module.clone(),
            EdgeKind::Contains,
            "edge.web-module",
        ),
        (
            ids.web_module,
            ids.web_function.clone(),
            EdgeKind::Contains,
            "edge.web-function",
        ),
        (
            ids.web_service,
            id("metadata.xdto.exchange"),
            EdgeKind::References,
            "edge.web-package",
        ),
        (
            ids.operation.clone(),
            xdto_type_id(&id("metadata.xdto.exchange"), &name("Result"))
                .expect("XDTO type identity must be valid"),
            EdgeKind::References,
            "edge.operation-type",
        ),
        (
            ids.parameter,
            xdto_type_id(&id("metadata.xdto.exchange"), &name("Result"))
                .expect("XDTO type identity must be valid"),
            EdgeKind::References,
            "edge.parameter-type",
        ),
        (
            ids.http_method.clone(),
            ids.http_function.clone(),
            EdgeKind::References,
            "edge.http-handler-reference",
        ),
        (
            ids.http_method,
            ids.http_function,
            EdgeKind::Triggers,
            "edge.http-trigger",
        ),
        (
            ids.operation.clone(),
            ids.web_function.clone(),
            EdgeKind::References,
            "edge.web-handler-reference",
        ),
        (
            ids.operation,
            ids.web_function,
            EdgeKind::Triggers,
            "edge.web-trigger",
        ),
    ];

    let mut graph = SemanticGraph::new();
    if reverse {
        for node in nodes.into_iter().rev() {
            graph.insert_node(node);
        }
        for (source, target, kind, evidence) in edges.into_iter().rev() {
            graph
                .insert_edge(GraphEdge::new_with_provenance(
                    source,
                    target,
                    kind,
                    node_provenance(evidence),
                ))
                .expect("edge endpoints must exist");
        }
    } else {
        for node in nodes {
            graph.insert_node(node);
        }
        for (source, target, kind, evidence) in edges {
            graph
                .insert_edge(GraphEdge::new_with_provenance(
                    source,
                    target,
                    kind,
                    node_provenance(evidence),
                ))
                .expect("edge endpoints must exist");
        }
    }
    graph
}

#[test]
fn xdto_service_graph_is_valid_queryable_reported_and_order_independent() {
    let normal = graph(Some("POST"), false);
    let reversed = graph(Some("POST"), true);
    let query = normal.query();
    let ids = FixtureIds::new();

    assert!(normal.diff(&reversed).is_empty());
    assert_eq!(normal.report(), reversed.report());
    assert!(
        normal.validate().is_valid(),
        "issues: {:?}",
        normal.validate().issues()
    );
    for kind in [
        NodeKind::XdtoType,
        NodeKind::HttpServiceUrlTemplate,
        NodeKind::HttpServiceMethod,
        NodeKind::WebServiceOperation,
        NodeKind::WebServiceParameter,
    ] {
        assert_eq!(query.nodes_by_kind(kind).len(), 1);
    }
    assert_eq!(
        query
            .owner(&NodeId::new(ids.http_method.as_str()))
            .expect("HTTP Method must have one owner")
            .kind(),
        NodeKind::HttpServiceUrlTemplate
    );
    assert_eq!(
        query
            .outgoing_edges_by_kind(&NodeId::new(ids.operation.as_str()), EdgeKind::Triggers)
            .len(),
        1
    );
    assert_eq!(normal.report().nodes().total(), 12);
    assert_eq!(normal.report().edges().by_kind()[&EdgeKind::Contains], 9);
    assert_eq!(normal.report().edges().by_kind()[&EdgeKind::References], 5);
    assert_eq!(normal.report().edges().by_kind()[&EdgeKind::Triggers], 2);
}

#[test]
fn xdto_service_content_changes_preserve_identity_and_modify_semantic_content() {
    let absent = graph(None, false);
    let explicit = graph(Some("POST"), false);
    let method_id = FixtureIds::new().http_method;
    let diff = absent.diff(&explicit);
    let change = diff
        .modified_nodes()
        .iter()
        .find(|change| change.id().as_str() == method_id.as_str())
        .expect("HTTP Method content change must be modified");

    assert_eq!(diff.summary().nodes_modified(), 1);
    assert_eq!(
        change.modified_aspects(),
        &[NodeModifiedAspect::SemanticContent]
    );
    assert_eq!(
        change.old().expect("old snapshot must exist").id(),
        change.new_state().expect("new snapshot must exist").id()
    );
}

#[test]
fn schema_accepts_only_adr_0035_additive_pairs() {
    let schema = SemanticGraphSchema;
    let contains = [
        (
            NodeKind::Metadata(MetadataKind::XdtoPackage),
            NodeKind::XdtoType,
        ),
        (
            NodeKind::Metadata(MetadataKind::HttpService),
            NodeKind::HttpServiceUrlTemplate,
        ),
        (
            NodeKind::HttpServiceUrlTemplate,
            NodeKind::HttpServiceMethod,
        ),
        (
            NodeKind::Metadata(MetadataKind::WebService),
            NodeKind::WebServiceOperation,
        ),
        (NodeKind::WebServiceOperation, NodeKind::WebServiceParameter),
    ];
    let references = [
        (
            NodeKind::Metadata(MetadataKind::WebService),
            NodeKind::Metadata(MetadataKind::XdtoPackage),
        ),
        (NodeKind::WebServiceOperation, NodeKind::XdtoType),
        (NodeKind::WebServiceParameter, NodeKind::XdtoType),
        (NodeKind::HttpServiceMethod, NodeKind::Function),
        (NodeKind::WebServiceOperation, NodeKind::Function),
    ];

    for (source, target) in contains {
        assert!(schema.allows(source, EdgeKind::Contains, target));
        assert!(!schema.allows(target, EdgeKind::Contains, source));
    }
    for (source, target) in references {
        assert!(schema.allows(source, EdgeKind::References, target));
        assert!(!schema.allows(target, EdgeKind::References, source));
    }
    for (source, target) in [
        (NodeKind::HttpServiceMethod, NodeKind::Function),
        (NodeKind::WebServiceOperation, NodeKind::Function),
    ] {
        assert!(schema.allows(source, EdgeKind::Triggers, target));
        assert!(!schema.allows(target, EdgeKind::Triggers, source));
    }
}

fn resolved_web_handler_request(ids: &FixtureIds) -> SemanticReferenceRequest {
    SemanticReferenceRequest::collected(
        ids.operation.clone(),
        SemanticReferenceCategory::Callable,
        SemanticReference::Child {
            owner: ids.web_module.clone(),
            name: name("Exchange"),
        },
        [NodeKind::Function],
        [provenance(
            "request.web-handler.collection",
            ResolutionState::Unresolved,
        )],
    )
    .expect("Web handler request must be valid")
    .into_resolved(
        ids.web_function.clone(),
        NodeKind::Function,
        [provenance(
            "request.web-handler.resolution",
            ResolutionState::Resolved,
        )],
    )
    .expect("Web handler resolution must be valid")
}

#[test]
fn xdto_and_service_requests_use_stable_categories_and_reference_projections() {
    let graph = graph(Some("POST"), false);
    let ids = FixtureIds::new();
    let requests = [
        SemanticReferenceRequest::collected(
            ids.web_service.clone(),
            SemanticReferenceCategory::XdtoPackage,
            SemanticReference::Name(name("Exchange")),
            [NodeKind::Metadata(MetadataKind::XdtoPackage)],
            [provenance(
                "request.package.collection",
                ResolutionState::Unresolved,
            )],
        )
        .expect("package request must be valid")
        .into_resolved(
            ids.package.clone(),
            NodeKind::Metadata(MetadataKind::XdtoPackage),
            [provenance(
                "request.package.resolution",
                ResolutionState::Resolved,
            )],
        )
        .expect("package resolution must be valid"),
        SemanticReferenceRequest::collected(
            ids.operation.clone(),
            SemanticReferenceCategory::XdtoType,
            SemanticReference::Child {
                owner: ids.package.clone(),
                name: name("Result"),
            },
            [NodeKind::XdtoType],
            [provenance(
                "request.type.collection",
                ResolutionState::Unresolved,
            )],
        )
        .expect("type request must be valid")
        .into_resolved(
            ids.xdto_type.clone(),
            NodeKind::XdtoType,
            [provenance(
                "request.type.resolution",
                ResolutionState::Resolved,
            )],
        )
        .expect("type resolution must be valid"),
        SemanticReferenceRequest::collected(
            ids.http_method.clone(),
            SemanticReferenceCategory::Callable,
            SemanticReference::Child {
                owner: ids.http_module.clone(),
                name: name("Handle"),
            },
            [NodeKind::Function],
            [provenance(
                "request.handler.collection",
                ResolutionState::Unresolved,
            )],
        )
        .expect("handler request must be valid")
        .into_resolved(
            ids.http_function.clone(),
            NodeKind::Function,
            [provenance(
                "request.handler.resolution",
                ResolutionState::Resolved,
            )],
        )
        .expect("handler resolution must be valid"),
        resolved_web_handler_request(&ids),
    ];
    assert!(
        requests[0]
            .id()
            .as_str()
            .contains("category#12:xdto_package")
    );
    assert!(requests[1].id().as_str().contains("category#9:xdto_type"));
    let ledger = SemanticReferenceRequestLedger::from_requests(requests)
        .expect("request ledger must be valid");
    let validation = SemanticGraphValidator::new().validate_build_result_with_reference_requests(
        &graph,
        &[],
        &ledger,
    );

    assert!(validation.is_valid(), "issues: {:?}", validation.issues());
}
