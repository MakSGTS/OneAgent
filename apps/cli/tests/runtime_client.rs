use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Output};
use std::time::Duration;

use oneagent_runtime::{
    App, BoxError, ConfigurationProvider, GraphQueryService, HttpService, LifecycleState,
    RuntimeConfig, RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask,
    WorkspaceService,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const CLI: &str = env!("CARGO_BIN_EXE_oneagent-cli");
const TEST_TIMEOUT: Duration = Duration::from_secs(5);
const DESIGNER_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const EDT_ID: &str = "50000000-0000-0000-0000-000000000000";

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
    address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("OneAgent Runtime", "cli-client-test")
            .with_workspace_root(self.workspace_root.clone())
            .with_http_bind_address(self.address))
    }
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../runtime/tests/fixtures/workspace_service")
}

fn copy_fixture() -> TempDir {
    let root = tempfile::tempdir().expect("temporary Workspace must be created");
    copy_tree(&fixture_root(), root.path());
    root
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

fn startup_gate(
    started: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
) -> impl RuntimeService {
    move |context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            started.send(()).expect("gate startup must be observed");
            release.await.expect("gate startup must be released");
            let mut cancellation = context.cancellation();
            let task: ServiceTask = Box::pin(async move {
                cancellation.cancelled().await;
                Ok(())
            });
            Ok(task)
        })
    }
}

fn observed_task_start<S>(service: S, started: oneshot::Sender<()>) -> impl RuntimeService
where
    S: RuntimeService,
{
    move |context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            let service_task = Box::new(service).start(context).await?;
            let task: ServiceTask = Box::pin(async move {
                started.send(()).expect("HTTP task start must be observed");
                service_task.await
            });
            Ok(task)
        })
    }
}

async fn wait_for_address(receiver: &mut watch::Receiver<Option<SocketAddr>>) -> SocketAddr {
    loop {
        if let Some(address) = *receiver.borrow() {
            return address;
        }
        timeout(TEST_TIMEOUT, receiver.changed())
            .await
            .expect("HTTP address wait must not hang")
            .expect("HTTP service must retain address ownership");
    }
}

async fn wait_for_lifecycle(
    receiver: &mut watch::Receiver<LifecycleState>,
    expected: LifecycleState,
) {
    while *receiver.borrow() != expected {
        timeout(TEST_TIMEOUT, receiver.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("application must retain lifecycle ownership");
    }
}

async fn cli(args: Vec<String>) -> Output {
    timeout(
        TEST_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            ProcessCommand::new(CLI)
                .args(args)
                .output()
                .expect("CLI process must run")
        }),
    )
    .await
    .expect("CLI process must not hang")
    .expect("CLI blocking task must join")
}

fn address_args(address: SocketAddr, command: &[&str]) -> Vec<String> {
    let mut args = vec!["--address".to_owned(), address.to_string()];
    args.extend(command.iter().map(ToString::to_string));
    args
}

fn assert_exit(output: &Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "output: {output:?}");
}

fn success_json(output: &Output) -> Value {
    assert_exit(output, 0);
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));
    serde_json::from_slice(&output.stdout).expect("successful CLI output must be Runtime JSON")
}

fn server_error_json(output: &Output) -> Value {
    assert_exit(output, 4);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.ends_with(b"\n"));
    serde_json::from_slice(&output.stderr).expect("server failure must preserve Runtime JSON")
}

async fn running_configuration_matrix(address: SocketAddr) -> [Value; 2] {
    let configurations =
        success_json(&cli(address_args(address, &["configurations", "--limit", "100"])).await);
    assert_eq!(
        configurations["configurations"]
            .as_array()
            .expect("configuration list must be an array")
            .iter()
            .map(|value| value["id"].as_str().expect("ID must be a string"))
            .collect::<Vec<_>>(),
        [DESIGNER_ID, EDT_ID]
    );
    assert_eq!(configurations["truncated"], false);
    let repeated =
        success_json(&cli(address_args(address, &["configurations", "--limit", "100"])).await);
    assert_eq!(repeated, configurations);

    let limited =
        success_json(&cli(address_args(address, &["configurations", "--limit", "1"])).await);
    assert_eq!(limited["configurations"].as_array().map(Vec::len), Some(1));
    assert_eq!(limited["truncated"], true);
    [configurations, limited]
}

async fn running_graph_matrix(address: SocketAddr) -> [Value; 5] {
    let designer_node = success_json(
        &cli(address_args(
            address,
            &[
                "node",
                "--configuration-id",
                DESIGNER_ID,
                "--node-id",
                DESIGNER_ID,
            ],
        ))
        .await,
    );
    assert_eq!(designer_node["configuration_id"], DESIGNER_ID);
    assert_eq!(designer_node["node"]["name"], "DNSWorldEdition");
    assert_eq!(designer_node["node"]["metadata_kind"], "configuration");

    let edt_node = success_json(
        &cli(address_args(
            address,
            &["node", "--node-id", EDT_ID, "--configuration-id", EDT_ID],
        ))
        .await,
    );
    assert_eq!(edt_node["configuration_id"], EDT_ID);
    assert_eq!(edt_node["node"]["name"], "WritesFixture");

    let relations = success_json(
        &cli(address_args(
            address,
            &[
                "relations",
                "--configuration-id",
                DESIGNER_ID,
                "--node-id",
                DESIGNER_ID,
                "--direction",
                "outgoing",
                "--edge-kind",
                "contains",
                "--limit",
                "100",
            ],
        ))
        .await,
    );
    assert_eq!(relations["configuration_id"], DESIGNER_ID);
    assert_eq!(relations["direction"], "outgoing");
    assert_eq!(relations["edge_kind"], "contains");
    assert_eq!(relations["relations"].as_array().map(Vec::len), Some(1));

    let traversal = success_json(
        &cli(address_args(
            address,
            &[
                "traverse",
                "--configuration-id",
                DESIGNER_ID,
                "--node-id",
                DESIGNER_ID,
                "--direction",
                "outgoing",
                "--max-depth",
                "1",
                "--edge-kind",
                "contains",
                "--include-start",
                "--limit",
                "100",
            ],
        ))
        .await,
    );
    assert_eq!(traversal["configuration_id"], DESIGNER_ID);
    assert_eq!(traversal["include_start"], true);
    assert_eq!(traversal["nodes"].as_array().map(Vec::len), Some(2));
    assert_eq!(traversal["nodes"][0]["depth"], 0);
    assert_eq!(traversal["nodes"][1]["depth"], 1);

    let missing = server_error_json(
        &cli(address_args(
            address,
            &[
                "node",
                "--configuration-id",
                "missing",
                "--node-id",
                "missing",
            ],
        ))
        .await,
    );
    assert_eq!(
        missing,
        json!({"error": {"code": "configuration_not_found", "message": "configuration was not found"}})
    );
    [designer_node, edt_node, relations, traversal, missing]
}

async fn run_production_matrix_once() -> Vec<Value> {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let graph_query = GraphQueryService::new(workspace.snapshot_observer());
    let http = HttpService::with_graph_query(graph_query);
    let mut address_observer = http.subscribe_bound_address();
    let (http_task_started_sender, http_task_started) = oneshot::channel();
    let (gate_started_sender, gate_started) = oneshot::channel();
    let (gate_release_sender, gate_release) = oneshot::channel();
    let provider = TestConfigurationProvider {
        workspace_root: root.path().to_owned(),
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    };
    let app = App::builder()
        .configure(&provider)
        .expect("CLI test configuration must load")
        .register_service("http", observed_task_start(http, http_task_started_sender))
        .expect("HTTP service must register")
        .register_service("gate", startup_gate(gate_started_sender, gate_release))
        .expect("startup gate must register")
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("CLI test application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let address = wait_for_address(&mut address_observer).await;
    timeout(TEST_TIMEOUT, gate_started)
        .await
        .expect("startup gate must not hang")
        .expect("startup gate must be observed");
    timeout(TEST_TIMEOUT, http_task_started)
        .await
        .expect("HTTP task start must not hang")
        .expect("HTTP task start must be observed");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);

    let live = success_json(&cli(address_args(address, &["health", "live"])).await);
    assert_eq!(live, json!({"status": "alive"}));
    let not_ready = server_error_json(&cli(address_args(address, &["health", "ready"])).await);
    assert_eq!(not_ready, json!({"status": "not_ready"}));
    let query_not_ready =
        server_error_json(&cli(address_args(address, &["configurations", "--limit", "100"])).await);
    assert_eq!(
        query_not_ready,
        json!({"error": {"code": "runtime_not_ready", "message": "runtime is not ready"}})
    );

    gate_release_sender
        .send(())
        .expect("startup gate release must be observed");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

    let ready = success_json(&cli(address_args(address, &["health", "ready"])).await);
    assert_eq!(ready, json!({"status": "ready"}));

    let [configurations, limited] = running_configuration_matrix(address).await;
    let [designer_node, edt_node, relations, traversal, missing] =
        running_graph_matrix(address).await;

    shutdown_sender
        .send(())
        .expect("Runtime shutdown must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested Runtime shutdown must succeed");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert_eq!(*address_observer.borrow(), None);
    let rebound = tokio::net::TcpListener::bind(address)
        .await
        .expect("Runtime listener address must be released");
    drop(rebound);

    vec![
        live,
        not_ready,
        query_not_ready,
        ready,
        configurations,
        limited,
        designer_node,
        edt_node,
        relations,
        traversal,
        missing,
    ]
}

#[test]
fn public_executable_reports_help_version_usage_transport_and_protocol() {
    let help = ProcessCommand::new(CLI)
        .arg("--help")
        .output()
        .expect("help process must run");
    assert_exit(&help, 0);
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8_lossy(&help.stdout).starts_with("OneAgent Runtime client\n"));

    let version = ProcessCommand::new(CLI)
        .arg("--version")
        .output()
        .expect("version process must run");
    assert_exit(&version, 0);
    assert_eq!(version.stdout, b"oneagent-cli 0.1.0\n");
    assert!(version.stderr.is_empty());

    let usage = ProcessCommand::new(CLI)
        .output()
        .expect("usage process must run");
    assert_exit(&usage, 2);
    assert!(usage.stdout.is_empty());
    assert_eq!(
        usage.stderr,
        b"oneagent-cli: usage_error: invalid command line\nTry 'oneagent-cli --help' for usage.\n"
    );

    let transport = ProcessCommand::new(CLI)
        .args(["--address", "127.0.0.1:0", "health", "live"])
        .output()
        .expect("transport-failure process must run");
    assert_exit(&transport, 3);
    assert!(transport.stdout.is_empty());
    assert_eq!(
        transport.stderr,
        b"oneagent-cli: transport_error: failed to communicate with runtime\n"
    );

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("controlled malformed-response listener must bind");
    let address = listener.local_addr().expect("address must be readable");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("CLI must connect");
        let mut request = [0_u8; 1024];
        let read = std::io::Read::read(&mut stream, &mut request).expect("request must read");
        assert_ne!(read, 0);
        std::io::Write::write_all(
            &mut stream,
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\ninvalid",
        )
        .expect("malformed response must write");
    });
    let protocol = ProcessCommand::new(CLI)
        .args(["--address", &address.to_string(), "health", "live"])
        .output()
        .expect("protocol-failure process must run");
    assert_exit(&protocol, 5);
    assert!(protocol.stdout.is_empty());
    assert_eq!(
        protocol.stderr,
        b"oneagent-cli: protocol_error: runtime response is invalid\n"
    );
    server.join().expect("controlled server must join");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn public_cli_consumes_every_production_operation_and_repeats_fresh_runs() {
    let first = run_production_matrix_once().await;
    let second = run_production_matrix_once().await;
    assert_eq!(first, second);
}
