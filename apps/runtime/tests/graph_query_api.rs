use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use oneagent_runtime::{
    App, AppBuilder, BoxError, ConfigurationProvider, GraphQueryService, HttpService,
    LifecycleState, RuntimeConfig, RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask,
    WorkspaceService, WorkspaceSnapshot,
};
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const DESIGNER_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const DESIGNER_MODULE_ID: &str = "dc24575c-a787-411d-93bd-494271291d73";
const EDT_ID: &str = "50000000-0000-0000-0000-000000000000";
const EDT_DOCUMENT_ID: &str = "ed647f67-f8fe-476b-8823-8d52b365ab20";

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
    http_bind_address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(
            RuntimeConfig::new("OneAgent Runtime", "graph-query-api-test")
                .with_workspace_root(self.workspace_root.clone())
                .with_http_bind_address(self.http_bind_address),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("fixture destination must be created");
    let mut entries = fs::read_dir(source)
        .expect("fixture source must be readable")
        .map(|entry| entry.expect("fixture entry must be readable"))
        .collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let destination = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture entry type must be readable")
            .is_dir()
        {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).expect("fixture file must be copied");
        }
    }
}

fn copy_fixture() -> tempfile::TempDir {
    let temporary = tempdir().expect("temporary Workspace root must be created");
    copy_tree(&fixture_root(), temporary.path());
    temporary
}

fn configured_builder(root: impl Into<PathBuf>) -> AppBuilder {
    let provider = TestConfigurationProvider {
        workspace_root: root.into(),
        http_bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    };
    App::builder()
        .configure(&provider)
        .expect("Graph Query API test configuration must load")
}

async fn wait_for_address(receiver: &mut watch::Receiver<Option<SocketAddr>>) -> SocketAddr {
    loop {
        if let Some(address) = *receiver.borrow() {
            return address;
        }
        timeout(Duration::from_secs(1), receiver.changed())
            .await
            .expect("HTTP address wait must not hang")
            .expect("HTTP service must retain address ownership");
    }
}

async fn wait_for_snapshot(
    receiver: &mut watch::Receiver<Option<Arc<WorkspaceSnapshot>>>,
) -> Arc<WorkspaceSnapshot> {
    loop {
        if let Some(snapshot) = receiver.borrow().clone() {
            return snapshot;
        }
        timeout(Duration::from_secs(1), receiver.changed())
            .await
            .expect("Workspace snapshot wait must not hang")
            .expect("Workspace service must retain snapshot ownership");
    }
}

async fn wait_for_snapshot_clear(receiver: &mut watch::Receiver<Option<Arc<WorkspaceSnapshot>>>) {
    while receiver.borrow().is_some() {
        let _ = timeout(Duration::from_secs(1), receiver.changed())
            .await
            .expect("Workspace snapshot cleanup must not hang");
    }
    assert!(receiver.borrow_and_update().is_none());
    assert!(
        receiver.changed().await.is_err(),
        "Workspace sender must not survive App::run"
    );
}

async fn wait_for_lifecycle(
    lifecycle: &mut watch::Receiver<LifecycleState>,
    expected: LifecycleState,
) {
    while *lifecycle.borrow() != expected {
        timeout(Duration::from_secs(1), lifecycle.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("Runtime must retain lifecycle ownership");
    }
}

async fn request(address: SocketAddr, method: &str, target: &str) -> RawResponse {
    let mut stream = timeout(Duration::from_secs(1), TcpStream::connect(address))
        .await
        .expect("HTTP connect must not hang")
        .expect("HTTP listener must accept loopback connections");
    let request = format!(
        "{method} {target} HTTP/1.1\r\nHost: localhost\r\nAccept: text/plain\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .await
        .expect("HTTP request must write");

    let mut bytes = Vec::new();
    timeout(Duration::from_secs(1), stream.read_to_end(&mut bytes))
        .await
        .expect("HTTP response must not hang")
        .expect("HTTP response must read");
    parse_response(&bytes)
}

fn parse_response(bytes: &[u8]) -> RawResponse {
    let response = std::str::from_utf8(bytes).expect("HTTP response must be UTF-8");
    let (head, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response must contain a header terminator");
    let mut lines = head.lines();
    let status = lines
        .next()
        .expect("HTTP response must contain a status line")
        .split_whitespace()
        .nth(1)
        .expect("HTTP status line must contain a code")
        .parse()
        .expect("HTTP status code must be numeric");
    let headers = lines
        .map(|line| {
            let (name, value) = line
                .split_once(':')
                .expect("HTTP header must contain a colon");
            (name.to_ascii_lowercase(), value.trim().to_owned())
        })
        .collect();
    RawResponse {
        status,
        headers,
        body: body.to_owned(),
    }
}

fn json_response(response: &RawResponse, expected_status: u16) -> Value {
    assert_eq!(response.status, expected_status);
    assert_eq!(
        response.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    serde_json::from_str(&response.body).expect("Graph Query response must be JSON")
}

fn assert_error(response: &RawResponse, status: u16, code: &'static str, message: &'static str) {
    assert_eq!(
        json_response(response, status),
        json!({"error": {"code": code, "message": message}})
    );
}

fn startup_gate(
    start_attempted: oneshot::Sender<()>,
    start_release: oneshot::Receiver<()>,
    stopping: oneshot::Sender<()>,
    stop_release: oneshot::Receiver<()>,
) -> impl RuntimeService {
    move |context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            start_attempted
                .send(())
                .expect("gate startup attempt must be observed");
            start_release
                .await
                .expect("gate startup release must be sent");
            let mut cancellation = context.cancellation();
            let task: ServiceTask = Box::pin(async move {
                cancellation.cancelled().await;
                stopping.send(()).expect("gate stopping must be observed");
                stop_release.await.expect("gate stop release must be sent");
                Ok(())
            });
            Ok(task)
        })
    }
}

async fn run_production_operations_once() -> Vec<Value> {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshot_changes = observer.subscribe();
    let graph_query = GraphQueryService::new(observer.clone());
    let http = HttpService::with_graph_query(graph_query);
    let mut address_changes = http.subscribe_bound_address();
    let app = configured_builder(root.path())
        .register_service("http", http)
        .expect("HTTP service must register")
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));
    let address = wait_for_address(&mut address_changes).await;
    let snapshot = wait_for_snapshot(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

    assert_eq!(snapshot.configurations().len(), 2);
    assert_eq!(
        snapshot.configurations()[0].configuration_id().as_str(),
        DESIGNER_ID
    );
    assert_eq!(
        snapshot.configurations()[1].configuration_id().as_str(),
        EDT_ID
    );

    let targets = [
        "/api/v1/configurations",
        &format!("/api/v1/graph/node?configuration_id={DESIGNER_ID}&node_id={DESIGNER_MODULE_ID}"),
        &format!("/api/v1/graph/node?configuration_id={EDT_ID}&node_id={EDT_DOCUMENT_ID}"),
        &format!(
            "/api/v1/graph/relations?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&edge_kind=contains"
        ),
        &format!(
            "/api/v1/graph/relations?configuration_id={EDT_ID}&node_id={EDT_ID}&direction=outgoing&edge_kind=contains&limit=1"
        ),
        &format!(
            "/api/v1/graph/traverse?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&edge_kind=contains&max_depth=4&include_start=true"
        ),
        &format!(
            "/api/v1/graph/traverse?configuration_id={EDT_ID}&node_id={EDT_ID}&direction=outgoing&edge_kind=contains&max_depth=1"
        ),
    ];
    let mut responses = Vec::new();
    for target in targets {
        responses.push(json_response(&request(address, "GET", target).await, 200));
    }

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("Graph Query Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_snapshot_clear(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert!(observer.snapshot().is_none());
    assert_eq!(*address_changes.borrow(), None);
    let rebound = TcpListener::bind(address)
        .await
        .expect("HTTP listener address must be released");
    drop(rebound);

    responses
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn public_graph_query_api_keeps_both_production_graphs_exact_and_repeatable() {
    let first = run_production_operations_once().await;
    let repeated = run_production_operations_once().await;
    assert_eq!(first, repeated);

    assert_eq!(
        first[0],
        json!({
            "configurations": [
                {
                    "id": DESIGNER_ID,
                    "name": "DNSWorldEdition",
                    "format": "designer_xml",
                    "node_count": 4,
                    "edge_count": 3
                },
                {
                    "id": EDT_ID,
                    "name": "WritesFixture",
                    "format": "edt",
                    "node_count": 13,
                    "edge_count": 14
                }
            ],
            "truncated": false
        })
    );
    assert_eq!(first[1]["configuration_id"], DESIGNER_ID);
    assert_eq!(first[1]["node"]["id"], DESIGNER_MODULE_ID);
    assert_eq!(first[1]["node"]["name"], "DynamicSecurityOverridable");
    assert_eq!(first[1]["node"]["kind"], "metadata");
    assert_eq!(first[1]["node"]["metadata_kind"], "common_module");
    assert_eq!(first[2]["configuration_id"], EDT_ID);
    assert_eq!(first[2]["node"]["id"], EDT_DOCUMENT_ID);
    assert_eq!(first[2]["node"]["name"], "RefundOfPaymentByOrder");
    assert_eq!(first[2]["node"]["metadata_kind"], "document");

    assert_eq!(first[3]["configuration_id"], DESIGNER_ID);
    assert_eq!(first[3]["node_id"], DESIGNER_ID);
    assert_eq!(first[3]["direction"], "outgoing");
    assert_eq!(first[3]["edge_kind"], "contains");
    assert_eq!(first[3]["truncated"], false);
    assert_eq!(
        first[3]["relations"]
            .as_array()
            .expect("relations must be an array")
            .len(),
        1
    );
    assert_eq!(first[3]["relations"][0]["source_node_id"], DESIGNER_ID);
    assert_eq!(
        first[3]["relations"][0]["target_node_id"],
        DESIGNER_MODULE_ID
    );
    assert_eq!(
        first[3]["relations"][0]["related_node"]["id"],
        DESIGNER_MODULE_ID
    );
    assert!(
        !first[3]["relations"][0]["edge_id"]
            .as_str()
            .expect("edge ID must be a string")
            .is_empty()
    );

    assert_eq!(first[4]["configuration_id"], EDT_ID);
    assert_eq!(
        first[4]["relations"]
            .as_array()
            .expect("relations must be an array")
            .len(),
        1
    );
    assert_eq!(first[4]["truncated"], true);
    assert_ne!(
        first[4]["relations"][0]["target_node_id"],
        DESIGNER_MODULE_ID
    );

    let designer_nodes = first[5]["nodes"]
        .as_array()
        .expect("nodes must be an array");
    assert_eq!(designer_nodes.len(), 4);
    assert_eq!(designer_nodes[0]["node"]["id"], DESIGNER_ID);
    assert_eq!(designer_nodes[0]["depth"], 0);
    assert_eq!(designer_nodes[0]["via_edge_id"], Value::Null);
    assert_eq!(designer_nodes[1]["node"]["id"], DESIGNER_MODULE_ID);
    assert_eq!(designer_nodes[1]["depth"], 1);
    assert_eq!(designer_nodes[3]["depth"], 3);
    assert_eq!(first[5]["max_depth"], 4);
    assert_eq!(first[5]["include_start"], true);

    let edt_nodes = first[6]["nodes"]
        .as_array()
        .expect("nodes must be an array");
    assert_eq!(edt_nodes.len(), 3);
    assert!(edt_nodes.iter().all(|node| node["depth"] == 1));
    assert_eq!(first[6]["include_start"], false);
    assert_eq!(first[6]["edge_kind"], "contains");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn public_graph_query_api_enforces_the_complete_wire_and_error_matrix() {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let graph_query = GraphQueryService::new(workspace.snapshot_observer());
    let http = HttpService::with_graph_query(graph_query);
    let mut address_changes = http.subscribe_bound_address();
    let app = configured_builder(root.path())
        .register_service("http", http)
        .expect("HTTP service must register")
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));
    let address = wait_for_address(&mut address_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

    let limited = json_response(
        &request(address, "GET", "/api/v1/configurations?limit=1").await,
        200,
    );
    assert_eq!(
        limited["configurations"].as_array().expect("array").len(),
        1
    );
    assert_eq!(limited["truncated"], true);
    let maximum_limit = json_response(
        &request(address, "GET", "/api/v1/configurations?limit=100").await,
        200,
    );
    assert_eq!(
        maximum_limit["configurations"]
            .as_array()
            .expect("configurations must be an array")
            .len(),
        2
    );
    assert_eq!(maximum_limit["truncated"], false);

    let empty_relations = json_response(
        &request(
            address,
            "GET",
            &format!(
                "/api/v1/graph/relations?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=incoming"
            ),
        )
        .await,
        200,
    );
    assert_eq!(empty_relations["edge_kind"], Value::Null);
    assert_eq!(empty_relations["relations"], json!([]));
    assert_eq!(empty_relations["truncated"], false);

    let empty_traversal = json_response(
        &request(
            address,
            "GET",
            &format!(
                "/api/v1/graph/traverse?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&max_depth=0"
            ),
        )
        .await,
        200,
    );
    assert_eq!(empty_traversal["edge_kind"], Value::Null);
    assert_eq!(empty_traversal["include_start"], false);
    assert_eq!(empty_traversal["nodes"], json!([]));

    let errors = [
        (
            format!("/api/v1/graph/node?configuration_id=missing&node_id={DESIGNER_ID}"),
            404,
            "configuration_not_found",
            "configuration was not found",
        ),
        (
            format!("/api/v1/graph/node?configuration_id={DESIGNER_ID}&node_id=missing"),
            404,
            "node_not_found",
            "node was not found",
        ),
        (
            format!("/api/v1/graph/node?configuration_id=%20&node_id={DESIGNER_ID}"),
            400,
            "invalid_identifier",
            "identifier must not be empty",
        ),
        (
            "/api/v1/graph/node?configuration_id=x".to_owned(),
            400,
            "invalid_query",
            "query parameters are invalid",
        ),
        (
            format!(
                "/api/v1/graph/node?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&node_id=x"
            ),
            400,
            "invalid_query",
            "query parameters are invalid",
        ),
        (
            "/api/v1/configurations?unknown=x".to_owned(),
            400,
            "invalid_query",
            "query parameters are invalid",
        ),
        (
            "/api/v1/graph/node?configuration_id=x&node_id=%FF".to_owned(),
            400,
            "invalid_query",
            "query parameters are invalid",
        ),
        (
            format!(
                "/api/v1/graph/relations?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=sideways"
            ),
            400,
            "unsupported_direction",
            "direction is unsupported",
        ),
        (
            format!(
                "/api/v1/graph/relations?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&edge_kind=dependency"
            ),
            400,
            "unsupported_edge_kind",
            "edge kind is unsupported",
        ),
        (
            "/api/v1/configurations?limit=0".to_owned(),
            400,
            "limit_out_of_range",
            "limit must be between 1 and 100",
        ),
        (
            "/api/v1/configurations?limit=101".to_owned(),
            400,
            "limit_out_of_range",
            "limit must be between 1 and 100",
        ),
        (
            "/api/v1/configurations?limit=1.0".to_owned(),
            400,
            "limit_out_of_range",
            "limit must be between 1 and 100",
        ),
        (
            "/api/v1/configurations?limit=184467440737095516160".to_owned(),
            400,
            "limit_out_of_range",
            "limit must be between 1 and 100",
        ),
        (
            format!(
                "/api/v1/graph/traverse?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&max_depth=5"
            ),
            400,
            "max_depth_out_of_range",
            "max_depth must be between 0 and 4",
        ),
        (
            format!(
                "/api/v1/graph/traverse?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&max_depth=1.0"
            ),
            400,
            "max_depth_out_of_range",
            "max_depth must be between 0 and 4",
        ),
        (
            format!(
                "/api/v1/graph/traverse?configuration_id={DESIGNER_ID}&node_id={DESIGNER_ID}&direction=outgoing&max_depth=1&include_start=yes"
            ),
            400,
            "invalid_boolean",
            "include_start must be true or false",
        ),
    ];
    for (target, status, code, message) in errors {
        assert_error(
            &request(address, "GET", &target).await,
            status,
            code,
            message,
        );
    }

    for path in [
        "/api/v1/configurations",
        "/api/v1/graph/node",
        "/api/v1/graph/relations",
        "/api/v1/graph/traverse",
    ] {
        for method in ["HEAD", "POST"] {
            let response = request(address, method, path).await;
            assert_eq!(response.status, 405);
            assert_eq!(
                response.headers.get("allow").map(String::as_str),
                Some("GET")
            );
            assert_eq!(response.body, "");
        }
    }
    for path in [
        "/api/v1/unknown",
        "/api/v1/configurations/",
        "/api/v1/graph/node/",
        "/api/v1/graph/relations/",
        "/api/v1/graph/traverse/",
    ] {
        let response = request(address, "GET", path).await;
        assert_eq!(response.status, 404);
        assert_eq!(response.body, "");
    }

    assert_eq!(
        request(address, "GET", "/health/live").await.body,
        "{\"status\":\"alive\"}"
    );
    assert_eq!(
        request(address, "GET", "/health/ready").await.body,
        "{\"status\":\"ready\"}"
    );

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("Graph Query Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn public_graph_query_api_is_lifecycle_gated_and_releases_all_resources() {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshot_changes = observer.subscribe();
    let graph_query = GraphQueryService::new(observer.clone());
    let http = HttpService::with_graph_query(graph_query);
    let mut address_changes = http.subscribe_bound_address();
    let (start_attempted_sender, start_attempted) = oneshot::channel();
    let (start_release_sender, start_release) = oneshot::channel();
    let (stopping_sender, stopping) = oneshot::channel();
    let (stop_release_sender, stop_release) = oneshot::channel();
    let gate = startup_gate(
        start_attempted_sender,
        start_release,
        stopping_sender,
        stop_release,
    );
    let app = configured_builder(root.path())
        .register_service("http", http)
        .expect("HTTP service must register")
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .register_service("gate", gate)
        .expect("lifecycle gate must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let address = wait_for_address(&mut address_changes).await;
    assert_eq!(
        wait_for_snapshot(&mut snapshot_changes)
            .await
            .configurations()
            .len(),
        2
    );
    timeout(Duration::from_secs(1), start_attempted)
        .await
        .expect("gate startup must not hang")
        .expect("gate startup must be attempted");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);
    assert_error(
        &request(address, "GET", "/api/v1/configurations").await,
        503,
        "runtime_not_ready",
        "runtime is not ready",
    );

    start_release_sender
        .send(())
        .expect("gate startup must be released");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    assert_eq!(
        json_response(
            &request(address, "GET", "/api/v1/configurations").await,
            200
        )["configurations"]
            .as_array()
            .expect("configurations must be an array")
            .len(),
        2
    );

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(Duration::from_secs(1), stopping)
        .await
        .expect("gate stopping must not hang")
        .expect("gate stopping must be observed");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopping);
    assert!(observer.snapshot().is_some());
    assert_error(
        &request(address, "GET", "/api/v1/configurations").await,
        503,
        "runtime_not_ready",
        "runtime is not ready",
    );

    stop_release_sender
        .send(())
        .expect("gate stopping must be released");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("gated Graph Query shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_snapshot_clear(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert!(observer.snapshot().is_none());
    assert_eq!(*address_changes.borrow(), None);
    let rebound = TcpListener::bind(address)
        .await
        .expect("HTTP listener address must be released");
    drop(rebound);

    let unpublished_workspace = WorkspaceService::new();
    let unavailable_query = GraphQueryService::new(unpublished_workspace.snapshot_observer());
    drop(unpublished_workspace);
    let http = HttpService::with_graph_query(unavailable_query);
    let mut unavailable_address = http.subscribe_bound_address();
    let app = configured_builder(fixture_root())
        .register_service("http", http)
        .expect("HTTP service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));
    let address = wait_for_address(&mut unavailable_address).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    assert_error(
        &request(address, "GET", "/api/v1/configurations").await,
        503,
        "workspace_unavailable",
        "workspace snapshot is unavailable",
    );
    shutdown_sender
        .send(())
        .expect("unavailable app shutdown must be observed");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("unavailable Graph Query shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
}
