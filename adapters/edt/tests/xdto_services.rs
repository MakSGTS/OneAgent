use oneagent_common::EntityName;
use oneagent_edt::{EdtGraphError, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    EdgeKind, ImpactNodeStatus, NodeId, NodeKind, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticImpactAnalyzer, SemanticImpactOptions, SemanticReference, SemanticReferenceCategory,
    SemanticReferenceRequestOutcome, WebServiceParameterDirection, xdto_type_id,
};
use oneagent_metadata::{MetadataSpecificPayload, WebServiceXdtoPackage};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

const PACKAGE_ID: &str = "package-id";
const HTTP_ID: &str = "http-id";
const WEB_ID: &str = "web-id";
const INTERNAL_NAMESPACE: &str = "urn:repository:package";

fn production_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint13_xdto_services_project")
}

fn multiple_packages_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/multiple_xdto_packages_project")
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("fixture target directory must be created");
    for entry in fs::read_dir(source).expect("fixture directory must be readable") {
        let entry = entry.expect("fixture entry must be readable");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture entry type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("fixture artifact must be copied");
        }
    }
}

fn copied_production_fixture() -> TempDir {
    let target = tempdir().expect("temporary fixture project must be created");
    copy_tree(&production_fixture(), target.path());
    target
}

fn replace_fixture_fragment(path: &Path, old: &str, new: &str) {
    let source = fs::read_to_string(path).expect("fixture artifact must be readable");
    assert!(source.contains(old), "fixture fragment must exist: {old}");
    fs::write(path, source.replacen(old, new, 1)).expect("fixture artifact must be updated");
}

fn project() -> TempDir {
    let project = tempdir().expect("temporary project must be created");
    let configuration = project.path().join("src/Configuration");
    fs::create_dir_all(&configuration).expect("configuration directory must be created");
    fs::write(
        configuration.join("Configuration.mdo"),
        r#"<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="configuration-id"><name>Demo</name></mdclass:Configuration>"#,
    )
    .expect("configuration descriptor must be written");

    write_xdto(project.path());
    write_http(project.path(), "HandleHttp", Some("POST"));
    write_web(project.path());
    project
}

fn object_directory(project: &Path, family: &str, name: &str) -> PathBuf {
    project.join("src").join(family).join(name)
}

fn write_xdto(project: &Path) {
    let directory = object_directory(project, "XDTOPackages", "ExchangePackage");
    fs::create_dir_all(&directory).expect("XDTO directory must be created");
    fs::write(
        directory.join("ExchangePackage.mdo"),
        format!(
            r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{PACKAGE_ID}"><name>ExchangePackage</name><synonym><key>en</key><value>Exchange package</value></synonym><namespace>{INTERNAL_NAMESPACE}</namespace></mdclass:XDTOPackage>"#
        ),
    )
    .expect("XDTO descriptor must be written");
    fs::write(
        directory.join("Package.xdto"),
        format!(
            r#"<package xmlns="http://v8.1c.ru/8.1/xdto" targetNamespace="{INTERNAL_NAMESPACE}"><objectType name="Result"><property name="Deferred"/></objectType><valueType name="Status"/></package>"#
        ),
    )
    .expect("XDTO artifact must be written");
}

fn write_additional_xdto(project: &Path, directory_name: &str, id: &str, name: &str) {
    write_xdto_package(
        project,
        directory_name,
        id,
        name,
        &format!("urn:repository:{directory_name}"),
        "Other",
    );
}

fn write_xdto_package(
    project: &Path,
    directory_name: &str,
    id: &str,
    name: &str,
    namespace: &str,
    type_name: &str,
) {
    let directory = object_directory(project, "XDTOPackages", directory_name);
    fs::create_dir_all(&directory).expect("additional XDTO directory must be created");
    fs::write(
        directory.join(format!("{directory_name}.mdo")),
        format!(
            r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}"><name>{name}</name><namespace>{namespace}</namespace></mdclass:XDTOPackage>"#
        ),
    )
    .expect("additional XDTO descriptor must be written");
    fs::write(
        directory.join("Package.xdto"),
        format!(
            r#"<package xmlns="http://v8.1c.ru/8.1/xdto" targetNamespace="{namespace}"><objectType name="{type_name}"/></package>"#
        ),
    )
    .expect("additional XDTO artifact must be written");
}

fn write_catalog(project: &Path, name: &str, id: &str) {
    let directory = object_directory(project, "Catalogs", name);
    fs::create_dir_all(&directory).expect("Catalog directory must be created");
    fs::write(
        directory.join(format!("{name}.mdo")),
        format!(
            r#"<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}"><name>{name}</name></mdclass:Catalog>"#
        ),
    )
    .expect("Catalog descriptor must be written");
}

fn replace_web_package(project: &Path, package: &str) {
    let path =
        object_directory(project, "WebServices", "ExchangeService").join("ExchangeService.mdo");
    let descriptor = fs::read_to_string(&path).expect("Web descriptor must be readable");
    fs::write(
        path,
        descriptor.replace(
            "XDTOPackage.ExchangePackage",
            &format!("XDTOPackage.{package}"),
        ),
    )
    .expect("Web package reference must be rewritten");
}

fn remove_web_package(project: &Path) {
    let path =
        object_directory(project, "WebServices", "ExchangeService").join("ExchangeService.mdo");
    let descriptor = fs::read_to_string(&path).expect("Web descriptor must be readable");
    fs::write(
        path,
        descriptor.replace(
            r#"<xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages>"#,
            "",
        ),
    )
    .expect("Web package declaration must be removed");
}

fn write_http(project: &Path, handler: &str, http_method: Option<&str>) {
    let directory = object_directory(project, "HTTPServices", "Api");
    fs::create_dir_all(&directory).expect("HTTP directory must be created");
    let http_method = http_method
        .map(|value| format!("<httpMethod>{value}</httpMethod>"))
        .unwrap_or_default();
    fs::write(
        directory.join("Api.mdo"),
        format!(
            r#"<mdclass:HTTPService xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{HTTP_ID}"><name>Api</name><rootURL>api</rootURL><urlTemplates uuid="url-id"><name>Route</name><template>/route</template><methods uuid="method-id"><name>POST</name>{http_method}<handler>{handler}</handler></methods></urlTemplates></mdclass:HTTPService>"#
        ),
    )
    .expect("HTTP descriptor must be written");
    fs::write(
        directory.join("Module.bsl"),
        "Function HandleHttp()\nEndFunction\n\nFunction AlternateHttp()\nEndFunction\n",
    )
    .expect("HTTP module must be written");
}

fn write_web(project: &Path) {
    let directory = object_directory(project, "WebServices", "ExchangeService");
    fs::create_dir_all(&directory).expect("Web directory must be created");
    fs::write(
        directory.join("ExchangeService.mdo"),
        format!(
            r#"<mdclass:WebService xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://g5.1c.ru/v8/dt/mcore" xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{WEB_ID}"><name>ExchangeService</name><namespace>urn:web</namespace><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages><operations uuid="operation-id"><name>Exchange</name><xdtoReturningValueType><name>Result</name><nsUri>{INTERNAL_NAMESPACE}</nsUri></xdtoReturningValueType><procedureName>HandleWeb</procedureName><parameters uuid="parameter-id"><name>External</name><xdtoValueType><name>string</name><nsUri>http://www.w3.org/2001/XMLSchema</nsUri></xdtoValueType></parameters></operations></mdclass:WebService>"#
        ),
    )
    .expect("Web descriptor must be written");
    fs::write(
        directory.join("Module.bsl"),
        "Function HandleWeb()\nEndFunction\n\nFunction ForeignHttpHandler()\nEndFunction\n",
    )
    .expect("Web module must be written");
}

fn build(project: &Path) -> oneagent_edt::EdtSemanticGraphBuildResult {
    FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project)
        .expect("generated XDTO/service project must build")
}

fn assert_failed_xdto_type_request(
    result: &oneagent_edt::EdtSemanticGraphBuildResult,
    outcome: SemanticReferenceRequestOutcome,
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
) {
    let request = result
        .reference_requests()
        .iter()
        .find(|request| {
            request.source_node().as_str() == "operation-id"
                && request.category() == SemanticReferenceCategory::XdtoType
        })
        .expect("XDTO type request must exist");
    assert_eq!(request.outcome(), outcome);
    assert_eq!(request.provenance().len(), 2);
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == code
            && diagnostic.kind() == kind
            && diagnostic.source_node() == Some(request.source_node())
            && diagnostic.candidates() == request.candidates()
    }));
    assert!(!result.graph().edges().any(|edge| {
        edge.source() == request.source_node()
            && edge.kind() == EdgeKind::References
            && result
                .graph()
                .node(edge.target())
                .is_some_and(|target| target.kind() == NodeKind::XdtoType)
    }));
    assert!(
        result.validate().is_valid(),
        "issues: {:?}",
        result.validate().issues()
    );
}

fn assert_metadata_payloads(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    let package = result
        .graph()
        .node(&oneagent_common::EntityId::new(PACKAGE_ID).unwrap())
        .unwrap();
    let package_payload = package
        .metadata_payload()
        .and_then(oneagent_metadata::MetadataPayload::specific)
        .expect("XDTO metadata payload must exist");
    assert!(matches!(
        package_payload,
        MetadataSpecificPayload::XdtoPackage(payload) if payload.namespace() == INTERNAL_NAMESPACE
    ));
    let http = result
        .graph()
        .node(&oneagent_common::EntityId::new(HTTP_ID).unwrap())
        .unwrap();
    assert!(matches!(
        http.metadata_payload().and_then(oneagent_metadata::MetadataPayload::specific),
        Some(MetadataSpecificPayload::HttpService(payload)) if payload.root_url() == "api"
    ));
    let web = result
        .graph()
        .node(&oneagent_common::EntityId::new(WEB_ID).unwrap())
        .unwrap();
    assert!(matches!(
        web.metadata_payload().and_then(oneagent_metadata::MetadataPayload::specific),
        Some(MetadataSpecificPayload::WebService(payload))
            if payload.namespace() == "urn:web" && payload.xdto_packages().len() == 1
    ));
}

fn web_packages(node: &oneagent_graph::GraphNode) -> &[WebServiceXdtoPackage] {
    match node
        .metadata_payload()
        .and_then(oneagent_metadata::MetadataPayload::specific)
    {
        Some(MetadataSpecificPayload::WebService(payload)) => payload.xdto_packages(),
        _ => panic!("Web Service metadata payload must exist"),
    }
}

fn assert_live_metadata_payloads(graph: &oneagent_graph::SemanticGraph) {
    let package = graph
        .node(&oneagent_common::EntityId::new("a69525c7-27ff-48df-b26b-325ba580a53e").unwrap())
        .expect("internal XDTO Package must exist");
    assert!(matches!(
        package
            .metadata_payload()
            .and_then(oneagent_metadata::MetadataPayload::specific),
        Some(MetadataSpecificPayload::XdtoPackage(payload))
            if payload.namespace() == "http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange"
    ));
    let http = graph
        .node(&oneagent_common::EntityId::new("c913a3d3-5fa2-4919-b304-e65731508ab1").unwrap())
        .expect("HTTP Service must exist");
    assert!(matches!(
        http.metadata_payload()
            .and_then(oneagent_metadata::MetadataPayload::specific),
        Some(MetadataSpecificPayload::HttpService(payload)) if payload.root_url() == "kiosk"
    ));
    let internal = graph
        .node(&oneagent_common::EntityId::new("cb3a5c5b-7bdc-4e12-96f1-11b1213b6853").unwrap())
        .expect("internal Web Service must exist");
    let external = graph
        .node(&oneagent_common::EntityId::new("a4ed8b24-bd23-45a7-9f34-61b25b91d0c6").unwrap())
        .expect("external Web Service must exist");
    let absent = graph
        .node(&oneagent_common::EntityId::new("a4e56049-4de9-40a3-8197-a5c84a11f516").unwrap())
        .expect("package-free Web Service must exist");
    assert_eq!(
        web_packages(internal),
        [WebServiceXdtoPackage::Repository(
            EntityName::new("EnterpriseDataExchange_1_0_1_1").unwrap()
        )]
    );
    assert_eq!(
        web_packages(external),
        [WebServiceXdtoPackage::ExternalNamespace(
            "http://v8.1c.ru/8.1/data/core".to_owned()
        )]
    );
    assert!(web_packages(absent).is_empty());
}

#[test]
fn live_derived_fixture_is_consumer_visible_and_deterministic() {
    let fixture = production_fixture();
    let first = build(&fixture);
    let repeated = build(&fixture);
    let graph = first.graph();
    let query = graph.query();
    assert_live_metadata_payloads(graph);

    assert!(first.diagnostics().is_empty());
    assert!(first.validate().is_valid());
    assert!(first.diff(&repeated).is_empty());
    assert_eq!(first.report(), repeated.report());
    assert_eq!(first.reference_requests(), repeated.reference_requests());

    for (kind, count) in [
        (NodeKind::XdtoType, 8),
        (NodeKind::HttpServiceUrlTemplate, 2),
        (NodeKind::HttpServiceMethod, 2),
        (NodeKind::WebServiceOperation, 3),
        (NodeKind::WebServiceParameter, 5),
    ] {
        let nodes = query.nodes_by_kind(kind);
        assert_eq!(nodes.len(), count, "unexpected fixture count for {kind:?}");
        assert!(nodes.iter().all(|node| {
            !node.provenance().is_empty() && query.owner(&NodeId::new(node.id().as_str())).is_some()
        }));
    }

    let explicit_post = graph
        .node(&oneagent_common::EntityId::new("78487ea6-9ec6-43ad-ac83-459cbd463f77").unwrap())
        .and_then(oneagent_graph::GraphNode::http_service_method_payload)
        .expect("explicit POST payload must exist");
    let implicit_get = graph
        .node(&oneagent_common::EntityId::new("fa155059-6d3c-4f2a-ba01-2820db6fcbf3").unwrap())
        .and_then(oneagent_graph::GraphNode::http_service_method_payload)
        .expect("implicit GET payload must exist");
    assert_eq!(
        explicit_post.http_method().map(EntityName::as_str),
        Some("POST")
    );
    assert!(implicit_get.http_method().is_none());

    let out = graph
        .node(&oneagent_common::EntityId::new("1acf34e6-c428-4130-82f0-5160dfcc0858").unwrap())
        .and_then(oneagent_graph::GraphNode::web_service_parameter_payload)
        .expect("Out parameter payload must exist");
    let in_out = graph
        .node(&oneagent_common::EntityId::new("7ed6bd83-b61d-4db4-8549-acacbdda6ca2").unwrap())
        .and_then(oneagent_graph::GraphNode::web_service_parameter_payload)
        .expect("InOut parameter payload must exist");
    assert_eq!(out.direction(), Some(WebServiceParameterDirection::Out));
    assert_eq!(
        in_out.direction(),
        Some(WebServiceParameterDirection::InOut)
    );

    assert_eq!(first.reference_requests().len(), 7);
    assert!(first.reference_requests().iter().all(|request| {
        request.outcome() == SemanticReferenceRequestOutcome::Resolved
            && !request.provenance().is_empty()
    }));
    assert_eq!(query.edges_by_kind(EdgeKind::References).len(), 7);
    assert_eq!(query.edges_by_kind(EdgeKind::Triggers).len(), 5);
    assert!(query.edges_by_kind(EdgeKind::Triggers).iter().all(|edge| {
        graph
            .node(edge.target())
            .is_some_and(|target| target.kind() == NodeKind::Function)
    }));
}

#[test]
fn retail_derived_multiple_packages_are_complete_canonical_and_deterministic() {
    let fixture = multiple_packages_fixture();
    let first = build(&fixture);
    let repeated = build(&fixture);
    let graph = first.graph();
    let query = graph.query();
    let equipment = graph
        .node(&oneagent_common::EntityId::new("c1568f1c-25ab-4328-8e77-e0e84788f10f").unwrap())
        .expect("EquipmentService must exist");
    let mobile = graph
        .node(&oneagent_common::EntityId::new("e8f6bb7f-d65d-4a9d-8fe8-ab0ab80664bf").unwrap())
        .expect("MobileService must exist");

    assert_eq!(
        web_packages(equipment),
        [
            WebServiceXdtoPackage::Repository(EntityName::new("EquipmentService").unwrap()),
            WebServiceXdtoPackage::Repository(EntityName::new("EquipmentService_1_0_0_6").unwrap()),
            WebServiceXdtoPackage::Repository(EntityName::new("EquipmentService_1_0_0_7").unwrap()),
            WebServiceXdtoPackage::Repository(EntityName::new("EquipmentService_2_0_0_3").unwrap()),
        ]
    );
    assert_eq!(
        web_packages(mobile),
        [
            WebServiceXdtoPackage::Repository(EntityName::new("MobileClientIntegration").unwrap()),
            WebServiceXdtoPackage::ExternalNamespace("http://v8.1c.ru/8.1/data/core".to_owned()),
        ]
    );

    let package_requests = first
        .reference_requests()
        .iter()
        .filter(|request| request.category() == SemanticReferenceCategory::XdtoPackage)
        .collect::<Vec<_>>();
    assert_eq!(package_requests.len(), 6);
    assert!(package_requests.iter().all(|request| {
        request.outcome() == SemanticReferenceRequestOutcome::Resolved
            && request.expected_kinds()
                == [NodeKind::Metadata(
                    oneagent_metadata::MetadataKind::XdtoPackage,
                )]
            && request.candidates().len() == 1
    }));
    let equipment_package_names = package_requests
        .iter()
        .filter(|request| request.source_node() == equipment.id())
        .map(|request| match request.reference() {
            SemanticReference::Name(name) => name.as_str(),
            other => panic!("unexpected package reference: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        equipment_package_names,
        [
            "EquipmentService",
            "EquipmentService_1_0_0_6",
            "EquipmentService_1_0_0_7",
            "EquipmentService_2_0_0_3",
        ]
    );
    assert_eq!(
        query
            .outgoing_edges_by_kind(&NodeId::new(equipment.id().as_str()), EdgeKind::References,)
            .len(),
        4
    );

    let type_request = first
        .reference_requests()
        .iter()
        .find(|request| {
            request.source_node().as_str() == "027aff36-61b0-4934-a707-8431b83f5898"
                && request.category() == SemanticReferenceCategory::XdtoType
        })
        .expect("SiteExchange2 return type request must exist");
    let commerce_205_type = xdto_type_id(
        &oneagent_common::EntityId::new("188a3368-9a46-49f6-81b3-c5b50a91f36b").unwrap(),
        &EntityName::new("КоммерческаяИнформация").unwrap(),
    )
    .expect("CommerceML205a type ID must be valid");
    assert_eq!(type_request.candidates(), [commerce_205_type]);
    assert_eq!(
        type_request.outcome(),
        SemanticReferenceRequestOutcome::Resolved
    );

    assert!(first.diagnostics().is_empty());
    assert_eq!(first.reference_statistics().resolved(), 10);
    assert_eq!(query.edges_by_kind(EdgeKind::References).len(), 10);
    assert_eq!(query.edges_by_kind(EdgeKind::Triggers).len(), 3);
    assert!(first.validate().is_valid());
    assert!(first.diff(&repeated).is_empty());
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert_eq!(first.report(), repeated.report());
}

fn mutate_production_fixture(project: &Path) {
    let currency = object_directory(project, "XDTOPackages", "CurrencyRates").join("Package.xdto");
    replace_fixture_fragment(
        &currency,
        "objectType name=\"Rate\"",
        "objectType name=\"Quote\"",
    );

    let http = object_directory(project, "HTTPServices", "Site").join("Site.mdo");
    replace_fixture_fragment(&http, "/{version}/order/create/", "/{version}/order/new/");
    replace_fixture_fragment(
        &http,
        "<httpMethod>POST</httpMethod>",
        "<httpMethod>PATCH</httpMethod>",
    );

    let web = object_directory(project, "WebServices", "EnterpriseDataExchange_1_0_1_1")
        .join("EnterpriseDataExchange_1_0_1_1.mdo");
    replace_fixture_fragment(
        &web,
        "<value>XDTOPackage.EnterpriseDataExchange_1_0_1_1</value>",
        "<value>http://v8.1c.ru/8.1/data/core</value>",
    );
    replace_fixture_fragment(&web, "core:ReferenceValue", "core:StringValue");
    replace_fixture_fragment(
        &web,
        "http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange",
        "http://www.w3.org/2001/XMLSchema",
    );
    replace_fixture_fragment(
        &web,
        "<name>PrepareDataOperationResult</name>",
        "<name>string</name>",
    );
    replace_fixture_fragment(
        &web,
        "<transferDirection>Out</transferDirection>",
        "<transferDirection>InOut</transferDirection>",
    );
    replace_fixture_fragment(
        &web,
        "<procedureName>GetPrepareDataToExportResult</procedureName>",
        "<procedureName>AlternatePrepareResult</procedureName>",
    );
    let module = object_directory(project, "WebServices", "EnterpriseDataExchange_1_0_1_1")
        .join("Module.bsl");
    replace_fixture_fragment(
        &module,
        "EndFunction",
        "EndFunction\n\nFunction AlternatePrepareResult()\n\tReturn Undefined;\nEndFunction",
    );

    fs::remove_dir_all(object_directory(project, "WebServices", "InterfaceVersion"))
        .expect("external Web Service must be removable in the copied fixture");
}

#[test]
fn live_derived_diff_reports_validation_and_impact_cover_boundary_transitions() {
    let fixture = copied_production_fixture();
    let before = build(fixture.path());
    mutate_production_fixture(fixture.path());
    let current = build(fixture.path());
    let diff = before.diff(&current);

    assert!(before.validate().is_valid());
    assert!(current.validate().is_valid());
    assert!(diff.graph().modified_nodes().len() >= 4);
    assert!(diff.graph().added_nodes().iter().any(|change| {
        change.new_state().is_some_and(|node| {
            node.name().as_str() == "Quote" && node.kind() == NodeKind::XdtoType
        })
    }));
    assert!(diff.graph().removed_nodes().iter().any(|change| {
        change
            .old()
            .is_some_and(|node| node.name().as_str() == "Rate" && node.kind() == NodeKind::XdtoType)
    }));
    assert!(!diff.reference_requests().added().is_empty());
    assert!(!diff.reference_requests().removed().is_empty());
    assert!(!diff.graph().added_edges().is_empty());
    assert!(!diff.graph().removed_edges().is_empty());
    assert_ne!(before.report(), current.report());

    let impact = SemanticImpactAnalyzer::analyze(
        before.graph(),
        current.graph(),
        diff.graph(),
        &SemanticImpactOptions::new(2),
    )
    .expect("fixture boundary impact must succeed");
    assert!(impact.affected_nodes().iter().any(|node| {
        node.status() == ImpactNodeStatus::Removed
            && node.node_id().as_str() == "65efaa10-3239-4f0f-a08e-88c89d9d8d5a"
    }));
}

#[test]
fn deferred_xdto_property_changes_leave_complete_indexes_unchanged() {
    let fixture = copied_production_fixture();
    let before = build(fixture.path());
    let currency =
        object_directory(fixture.path(), "XDTOPackages", "CurrencyRates").join("Package.xdto");
    replace_fixture_fragment(
        &currency,
        "</objectType>",
        "  <property name=\"DeferredOnly\"/>\n  </objectType>",
    );
    let current = build(fixture.path());

    assert!(before.graph().diff(current.graph()).is_empty());
    assert_eq!(
        before.graph().query().nodes().len(),
        current.graph().query().nodes().len()
    );
    assert_eq!(before.reference_requests(), current.reference_requests());
    assert!(current.validate().is_valid());
}

#[test]
fn production_builder_emits_payloads_children_requests_and_resolved_relations() {
    let project = project();
    let first = build(project.path());
    let repeated = build(project.path());
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert_eq!(first.diagnostics(), repeated.diagnostics());
    assert!(
        first.validate().is_valid(),
        "issues: {:?}",
        first.validate().issues()
    );
    assert!(first.diagnostics().is_empty());

    let graph = first.graph();
    assert_metadata_payloads(&first);

    let type_id = xdto_type_id(
        &oneagent_common::EntityId::new(PACKAGE_ID).unwrap(),
        &EntityName::new("Result").unwrap(),
    )
    .unwrap();
    assert_eq!(graph.node(&type_id).unwrap().kind(), NodeKind::XdtoType);
    assert_eq!(graph.nodes_by_kind(NodeKind::XdtoType).len(), 2);
    for (id, kind) in [
        ("url-id", NodeKind::HttpServiceUrlTemplate),
        ("method-id", NodeKind::HttpServiceMethod),
        ("operation-id", NodeKind::WebServiceOperation),
        ("parameter-id", NodeKind::WebServiceParameter),
    ] {
        assert_eq!(
            graph
                .node(&oneagent_common::EntityId::new(id).unwrap())
                .unwrap()
                .kind(),
            kind
        );
    }

    assert_eq!(first.reference_requests().len(), 4);
    assert_eq!(
        first
            .reference_requests()
            .iter()
            .filter(|request| request.category() == SemanticReferenceCategory::Callable)
            .count(),
        2
    );
    assert!(first.reference_requests().iter().all(|request| {
        if request.category() != SemanticReferenceCategory::Callable {
            return true;
        }
        match request.source_node().as_str() {
            "method-id" | "operation-id" => request.expected_kinds() == [NodeKind::Function],
            _ => false,
        }
    }));
    assert_eq!(
        first
            .reference_requests()
            .iter()
            .filter(|request| request.category() == SemanticReferenceCategory::XdtoPackage)
            .count(),
        1
    );
    assert_eq!(
        first
            .reference_requests()
            .iter()
            .filter(|request| request.category() == SemanticReferenceCategory::XdtoType)
            .count(),
        1
    );
    assert!(first.reference_requests().iter().all(|request| {
        request.outcome() == SemanticReferenceRequestOutcome::Resolved
            && request.provenance().len() == 2
    }));
    assert_eq!(
        graph
            .query()
            .edges_by_kind(EdgeKind::Triggers)
            .into_iter()
            .filter(|edge| {
                matches!(
                    graph
                        .node(edge.source())
                        .map(oneagent_graph::GraphNode::kind),
                    Some(NodeKind::HttpServiceMethod | NodeKind::WebServiceOperation)
                )
            })
            .count(),
        2
    );
    assert!(graph.nodes_by_kind(NodeKind::Unknown).is_empty());
}

#[test]
fn missing_handler_is_a_terminal_request_diagnostic_without_relation() {
    let project = project();
    write_http(project.path(), "MissingHandler", None);
    let result = build(project.path());
    let request = result
        .reference_requests()
        .iter()
        .find(|request| {
            request.source_node().as_str() == "method-id"
                && request.category() == SemanticReferenceCategory::Callable
        })
        .expect("HTTP handler request must exist");
    assert_eq!(
        request.outcome(),
        SemanticReferenceRequestOutcome::MissingTarget
    );
    assert!(request.candidates().is_empty());
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.source_node() == Some(request.source_node())
            && diagnostic.reference() == request.reference()
    }));
    assert!(!result.graph().edges().any(|edge| {
        edge.source().as_str() == "method-id"
            && matches!(edge.kind(), EdgeKind::References | EdgeKind::Triggers)
    }));
    assert!(
        result.validate().is_valid(),
        "issues: {:?}",
        result.validate().issues()
    );
}

#[test]
fn payload_and_handler_target_changes_are_observable_without_source_identity_changes() {
    let project = project();
    let before = build(project.path());
    write_http(project.path(), "AlternateHttp", None);
    let after = build(project.path());
    let diff = before.diff(&after);
    assert!(diff.graph().modified_nodes().iter().any(|change| {
        change.id().as_str() == "method-id"
            && change
                .modified_aspects()
                .contains(&oneagent_graph::NodeModifiedAspect::SemanticContent)
    }));
    assert!(!diff.reference_requests().added().is_empty());
    assert!(!diff.reference_requests().removed().is_empty());
    assert!(after.validate().is_valid());
}

#[test]
fn malformed_xdto_artifact_is_a_fatal_build_error() {
    let project = project();
    fs::write(
        object_directory(project.path(), "XDTOPackages", "ExchangePackage").join("Package.xdto"),
        "<package",
    )
    .expect("malformed artifact must be written");
    assert!(matches!(
        FileSystemEdtSemanticGraphBuilder.build_graph_with_diagnostics(project.path()),
        Err(EdtGraphError::XdtoPackage(_))
    ));
}

#[test]
fn multiple_package_siblings_resolve_independently_and_malformed_members_are_fatal() {
    const ORIGINAL: &str = r#"<xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages>"#;
    const FIRST_ORDER: &str = r#"<xdtoPackages xsi:type="core:StringValue"><value>urn:external:z</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.MissingPackage</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages>"#;
    const SECOND_ORDER: &str = r#"<xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages><xdtoPackages xsi:type="core:StringValue"><value>urn:external:z</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.ExchangePackage</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.MissingPackage</value></xdtoPackages>"#;

    let project = project();
    let descriptor = object_directory(project.path(), "WebServices", "ExchangeService")
        .join("ExchangeService.mdo");
    replace_fixture_fragment(&descriptor, ORIGINAL, FIRST_ORDER);
    let first = build(project.path());
    let web = first
        .graph()
        .node(&oneagent_common::EntityId::new(WEB_ID).unwrap())
        .expect("Web Service must exist");
    assert_eq!(
        web_packages(web),
        [
            WebServiceXdtoPackage::Repository(EntityName::new("ExchangePackage").unwrap()),
            WebServiceXdtoPackage::Repository(EntityName::new("MissingPackage").unwrap()),
            WebServiceXdtoPackage::ExternalNamespace("urn:external:z".to_owned()),
        ]
    );
    let package_requests = first
        .reference_requests()
        .iter()
        .filter(|request| request.category() == SemanticReferenceCategory::XdtoPackage)
        .collect::<Vec<_>>();
    assert_eq!(package_requests.len(), 2);
    assert_eq!(
        package_requests
            .iter()
            .filter(|request| request.outcome() == SemanticReferenceRequestOutcome::Resolved)
            .count(),
        1
    );
    assert_eq!(
        package_requests
            .iter()
            .filter(|request| request.outcome() == SemanticReferenceRequestOutcome::MissingTarget)
            .count(),
        1
    );
    assert_eq!(first.reference_statistics().unresolved(), 1);
    assert_eq!(first.diagnostics().len(), 1);
    assert!(first.validate().is_valid());

    replace_fixture_fragment(&descriptor, FIRST_ORDER, SECOND_ORDER);
    let reordered = build(project.path());
    assert!(first.diff(&reordered).is_empty());
    assert_eq!(first.reference_requests(), reordered.reference_requests());
    assert_eq!(first.diagnostics(), reordered.diagnostics());
    assert_eq!(first.report(), reordered.report());

    replace_fixture_fragment(&descriptor, "core:StringValue", "core:UnsupportedValue");
    assert!(matches!(
        FileSystemEdtSemanticGraphBuilder.build_graph_with_diagnostics(project.path()),
        Err(EdtGraphError::ServiceDescriptor(_))
    ));
}

#[test]
fn package_and_handler_failures_use_every_terminal_request_outcome() {
    let ambiguous_project = project();
    write_additional_xdto(
        ambiguous_project.path(),
        "DuplicatePackage",
        "package-id-2",
        "ExchangePackage",
    );
    let ambiguous = build(ambiguous_project.path());
    let ambiguous_request = ambiguous
        .reference_requests()
        .iter()
        .find(|request| request.category() == SemanticReferenceCategory::XdtoPackage)
        .expect("package request must exist");
    assert_eq!(
        ambiguous_request.outcome(),
        SemanticReferenceRequestOutcome::AmbiguousTarget
    );
    assert_eq!(ambiguous_request.candidates().len(), 2);
    assert!(
        ambiguous_request
            .candidates()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    assert_eq!(ambiguous.reference_statistics().ambiguous(), 1);
    assert!(ambiguous.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == SemanticDiagnosticCode::ReferenceAmbiguous
            && diagnostic.kind() == SemanticDiagnosticKind::AmbiguousTarget
            && diagnostic.source_node() == Some(ambiguous_request.source_node())
    }));

    let incompatible_project = project();
    write_catalog(incompatible_project.path(), "WrongPackage", "catalog-id");
    replace_web_package(incompatible_project.path(), "WrongPackage");
    let incompatible = build(incompatible_project.path());
    let incompatible_request = incompatible
        .reference_requests()
        .iter()
        .find(|request| request.category() == SemanticReferenceCategory::XdtoPackage)
        .expect("package request must exist");
    assert_eq!(
        incompatible_request.outcome(),
        SemanticReferenceRequestOutcome::IncompatibleTargetKind
    );
    assert_eq!(
        incompatible
            .reference_statistics()
            .incompatible_target_kind(),
        1
    );
    assert!(incompatible.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == SemanticDiagnosticCode::ReferenceIncompatibleKind
            && diagnostic.kind() == SemanticDiagnosticKind::IncompatibleTargetKind
            && diagnostic.source_node() == Some(incompatible_request.source_node())
    }));

    let invalid_owner_project = project();
    write_http(
        invalid_owner_project.path(),
        "ForeignHttpHandler",
        Some("POST"),
    );
    let invalid_owner = build(invalid_owner_project.path());
    let invalid_owner_request = invalid_owner
        .reference_requests()
        .iter()
        .find(|request| request.source_node().as_str() == "method-id")
        .expect("HTTP handler request must exist");
    assert_eq!(
        invalid_owner_request.outcome(),
        SemanticReferenceRequestOutcome::InvalidOwnerReference
    );
    assert_eq!(
        invalid_owner
            .reference_statistics()
            .invalid_owner_reference(),
        1
    );
    assert!(invalid_owner.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == SemanticDiagnosticCode::ReferenceInvalidOwner
            && diagnostic.kind() == SemanticDiagnosticKind::InvalidOwnerReference
            && diagnostic.source_node() == Some(invalid_owner_request.source_node())
    }));

    for result in [&ambiguous, &incompatible, &invalid_owner] {
        assert!(result.validate().is_valid());
        assert!(!result.graph().edges().any(|edge| {
            edge.source()
                == result
                    .reference_requests()
                    .iter()
                    .find(|request| request.outcome() != SemanticReferenceRequestOutcome::Resolved)
                    .expect("failed request must exist")
                    .source_node()
                && matches!(edge.kind(), EdgeKind::References | EdgeKind::Triggers)
        }));
    }
}

#[test]
fn xdto_type_namespace_and_owner_failures_are_terminal_and_relation_free() {
    let missing_project = project();
    let missing_descriptor =
        object_directory(missing_project.path(), "WebServices", "ExchangeService")
            .join("ExchangeService.mdo");
    replace_fixture_fragment(
        &missing_descriptor,
        "<xdtoReturningValueType><name>Result</name>",
        "<xdtoReturningValueType><name>MissingResult</name>",
    );
    let missing = build(missing_project.path());
    assert_failed_xdto_type_request(
        &missing,
        SemanticReferenceRequestOutcome::MissingTarget,
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticKind::UnresolvedTarget,
    );
    assert_eq!(missing.reference_statistics().unresolved(), 1);

    let ambiguous_project = project();
    write_xdto_package(
        ambiguous_project.path(),
        "DuplicateNamespacePackage",
        "duplicate-package-id",
        "DuplicateNamespacePackage",
        INTERNAL_NAMESPACE,
        "Result",
    );
    let ambiguous = build(ambiguous_project.path());
    assert_failed_xdto_type_request(
        &ambiguous,
        SemanticReferenceRequestOutcome::AmbiguousTarget,
        SemanticDiagnosticCode::ReferenceAmbiguous,
        SemanticDiagnosticKind::AmbiguousTarget,
    );
    let ambiguous_request = ambiguous
        .reference_requests()
        .iter()
        .find(|request| {
            request.source_node().as_str() == "operation-id"
                && request.category() == SemanticReferenceCategory::XdtoType
        })
        .expect("ambiguous XDTO type request must exist");
    assert_eq!(ambiguous_request.candidates().len(), 2);
    assert!(
        ambiguous_request
            .candidates()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    assert_eq!(ambiguous.reference_statistics().ambiguous(), 1);

    let invalid_owner_project = project();
    let package = object_directory(
        invalid_owner_project.path(),
        "XDTOPackages",
        "ExchangePackage",
    )
    .join("Package.xdto");
    replace_fixture_fragment(&package, "name=\"Result\"", "name=\"OwnedResult\"");
    write_xdto_package(
        invalid_owner_project.path(),
        "ForeignPackage",
        "foreign-package-id",
        "ForeignPackage",
        "urn:repository:foreign",
        "Result",
    );
    let invalid_owner = build(invalid_owner_project.path());
    assert_failed_xdto_type_request(
        &invalid_owner,
        SemanticReferenceRequestOutcome::InvalidOwnerReference,
        SemanticDiagnosticCode::ReferenceInvalidOwner,
        SemanticDiagnosticKind::InvalidOwnerReference,
    );
    assert_eq!(
        invalid_owner
            .reference_statistics()
            .invalid_owner_reference(),
        1
    );
}

#[test]
fn reordered_xml_declarations_preserve_the_complete_build_result() {
    let project = project();
    let before = build(project.path());

    let xdto_path =
        object_directory(project.path(), "XDTOPackages", "ExchangePackage").join("Package.xdto");
    fs::write(
        xdto_path,
        format!(
            r#"<package targetNamespace="{INTERNAL_NAMESPACE}" xmlns="http://v8.1c.ru/8.1/xdto"><valueType name="Status"/><objectType name="Result"><property name="Deferred"/></objectType></package>"#
        ),
    )
    .expect("reordered XDTO artifact must be written");
    let http_path = object_directory(project.path(), "HTTPServices", "Api").join("Api.mdo");
    fs::write(
        http_path,
        format!(
            r#"<mdclass:HTTPService uuid="{HTTP_ID}" xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"><rootURL>api</rootURL><name>Api</name><urlTemplates uuid="url-id"><template>/route</template><name>Route</name><methods uuid="method-id"><handler>HandleHttp</handler><httpMethod>POST</httpMethod><name>POST</name></methods></urlTemplates></mdclass:HTTPService>"#
        ),
    )
    .expect("reordered HTTP descriptor must be written");

    let after = build(project.path());
    assert!(before.graph().diff(after.graph()).is_empty());
    assert_eq!(before.reference_requests(), after.reference_requests());
    assert_eq!(before.diagnostics(), after.diagnostics());
    assert_eq!(before.reference_statistics(), after.reference_statistics());
    assert_eq!(before.report(), after.report());
    assert_eq!(before.validate(), after.validate());
}

#[test]
fn absent_optional_web_package_emits_no_package_request_or_diagnostic() {
    let project = project();
    remove_web_package(project.path());
    let result = build(project.path());

    assert!(
        result
            .reference_requests()
            .iter()
            .all(|request| request.category() != SemanticReferenceCategory::XdtoPackage)
    );
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.reference_requests().len(), 3);
    assert!(result.validate().is_valid());
}
