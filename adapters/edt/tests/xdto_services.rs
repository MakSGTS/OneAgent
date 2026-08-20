use oneagent_common::EntityName;
use oneagent_edt::{EdtGraphError, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    EdgeKind, NodeKind, SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticReferenceCategory,
    SemanticReferenceRequestOutcome, xdto_type_id,
};
use oneagent_metadata::MetadataSpecificPayload;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

const PACKAGE_ID: &str = "package-id";
const HTTP_ID: &str = "http-id";
const WEB_ID: &str = "web-id";
const INTERNAL_NAMESPACE: &str = "urn:repository:package";

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
    let directory = object_directory(project, "XDTOPackages", directory_name);
    fs::create_dir_all(&directory).expect("additional XDTO directory must be created");
    fs::write(
        directory.join(format!("{directory_name}.mdo")),
        format!(
            r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}"><name>{name}</name><namespace>urn:repository:{directory_name}</namespace></mdclass:XDTOPackage>"#
        ),
    )
    .expect("additional XDTO descriptor must be written");
    fs::write(
        directory.join("Package.xdto"),
        format!(
            r#"<package xmlns="http://v8.1c.ru/8.1/xdto" targetNamespace="urn:repository:{directory_name}"><objectType name="Other"/></package>"#
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
        "Procedure HandleHttp()\nEndProcedure\n\nProcedure AlternateHttp()\nEndProcedure\n",
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
        "Procedure HandleWeb()\nEndProcedure\n",
    )
    .expect("Web module must be written");
}

fn build(project: &Path) -> oneagent_edt::EdtSemanticGraphBuildResult {
    FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project)
        .expect("generated XDTO/service project must build")
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
    write_http(invalid_owner_project.path(), "HandleWeb", Some("POST"));
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
