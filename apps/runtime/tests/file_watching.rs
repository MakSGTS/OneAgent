use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use oneagent_runtime::{
    App, AppBuilder, BoxError, ConfigurationProvider, GraphQueryService, HttpService,
    LifecycleState, RuntimeConfig, WorkspaceService, WorkspaceSnapshot, WorkspaceUpdateFailureKind,
    WorkspaceUpdatePhase, WorkspaceUpdateStatus,
};
use serde_json::Value;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const DESIGNER_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const EDT_ID: &str = "50000000-0000-0000-0000-000000000000";

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
    http_bind_address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("OneAgent Runtime", "file-watching-test")
            .with_workspace_root(self.workspace_root.clone())
            .with_http_bind_address(self.http_bind_address))
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
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture file type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("fixture file must be copied");
        }
    }
}

fn replace_exact(path: &Path, before: &str, after: &str) {
    let source = fs::read_to_string(path).expect("source must be readable");
    assert_eq!(
        source.matches(before).count(),
        1,
        "replacement input must occur exactly once"
    );
    fs::write(path, source.replace(before, after)).expect("source replacement must be written");
}

fn configured_builder(root: impl Into<PathBuf>) -> AppBuilder {
    let provider = TestConfigurationProvider {
        workspace_root: root.into(),
        http_bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    };
    App::builder()
        .configure(&provider)
        .expect("File Watching test configuration must load")
}

async fn wait_for_address(receiver: &mut watch::Receiver<Option<SocketAddr>>) -> SocketAddr {
    loop {
        if let Some(address) = *receiver.borrow() {
            return address;
        }
        timeout(Duration::from_secs(5), receiver.changed())
            .await
            .expect("HTTP address wait must not hang")
            .expect("HTTP service must retain address ownership");
    }
}

async fn wait_for_lifecycle(
    lifecycle: &mut watch::Receiver<LifecycleState>,
    expected: LifecycleState,
) {
    while *lifecycle.borrow() != expected {
        timeout(Duration::from_secs(5), lifecycle.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("Runtime must retain lifecycle ownership");
    }
}

async fn wait_for_update(
    updates: &mut watch::Receiver<WorkspaceUpdateStatus>,
    predicate: impl Fn(WorkspaceUpdateStatus) -> bool,
) -> WorkspaceUpdateStatus {
    loop {
        let status = *updates.borrow_and_update();
        if predicate(status) {
            return status;
        }
        timeout(Duration::from_secs(5), updates.changed())
            .await
            .expect("Workspace update wait must not hang")
            .expect("Workspace service must retain update ownership");
    }
}

async fn wait_for_snapshot(
    snapshots: &mut watch::Receiver<Option<Arc<WorkspaceSnapshot>>>,
    predicate: impl Fn(&WorkspaceSnapshot) -> bool,
) -> Arc<WorkspaceSnapshot> {
    loop {
        if let Some(snapshot) = snapshots.borrow_and_update().clone()
            && predicate(&snapshot)
        {
            return snapshot;
        }
        timeout(Duration::from_secs(5), snapshots.changed())
            .await
            .expect("Workspace snapshot wait must not hang")
            .expect("Workspace service must retain snapshot ownership");
    }
}

async fn wait_for_watch_closed<T>(receiver: &mut watch::Receiver<T>) {
    loop {
        match timeout(Duration::from_secs(5), receiver.changed())
            .await
            .expect("watch closure wait must not hang")
        {
            Ok(()) => {}
            Err(_) => return,
        }
    }
}

fn configuration_names(snapshot: &WorkspaceSnapshot) -> Vec<String> {
    snapshot
        .configurations()
        .iter()
        .map(|configuration| configuration.configuration_name().as_str().to_owned())
        .collect()
}

fn assert_diagnostic_snapshots_complete(snapshot: &WorkspaceSnapshot) {
    for configuration in snapshot.configurations() {
        assert!(configuration.validation().is_valid());
        assert_eq!(
            configuration.diagnostic_report().summary().total(),
            configuration.diagnostic_report().findings().len()
        );
        assert_eq!(configuration.diagnostic_report().summary().suppressed(), 0);
        assert!(configuration.rule_execution_report().results().is_empty());
        assert!(
            configuration
                .rule_execution_report()
                .diagnostics()
                .is_empty()
        );
        assert_eq!(configuration.rule_execution_report().summary().total(), 0);
    }
}

async fn request(address: SocketAddr, target: &str) -> RawResponse {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(address))
        .await
        .expect("HTTP connect must not hang")
        .expect("HTTP listener must accept loopback connections");
    let request = format!(
        "GET {target} HTTP/1.1\r\nHost: localhost\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .await
        .expect("HTTP request must write");

    let mut bytes = Vec::new();
    timeout(Duration::from_secs(5), stream.read_to_end(&mut bytes))
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

async fn configuration_list(address: SocketAddr) -> Value {
    let response = request(address, "/api/v1/configurations").await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    serde_json::from_str(&response.body).expect("configuration response must be JSON")
}

fn wire_configuration_names(response: &Value) -> Vec<&str> {
    response["configurations"]
        .as_array()
        .expect("configurations must be an array")
        .iter()
        .map(|configuration| {
            configuration["name"]
                .as_str()
                .expect("configuration name must be a string")
        })
        .collect()
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn public_file_watching_rebuilds_recovers_and_keeps_graph_queries_atomic() {
    let root = copy_fixture();
    let moved = tempfile::tempdir().expect("temporary moved-root parent must be created");
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshots = observer.subscribe();
    let update_observer = workspace.update_observer();
    let mut updates = update_observer.subscribe();
    let graph_query = GraphQueryService::new(observer.clone());
    let http = HttpService::with_graph_query(graph_query);
    let mut addresses = http.subscribe_bound_address();
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

    let address = wait_for_address(&mut addresses).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    let initial_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;
    assert_eq!(initial_status.attempt(), 1);
    assert_eq!(initial_status.published(), 1);
    let initial = wait_for_snapshot(&mut snapshots, |snapshot| snapshot.len() == 2).await;
    assert_diagnostic_snapshots_complete(&initial);
    assert_eq!(
        configuration_names(&initial),
        ["DNSWorldEdition", "WritesFixture"]
    );
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["DNSWorldEdition", "WritesFixture"]
    );

    replace_exact(
        &root.path().join("designer/Configuration.xml"),
        "<Name>DNSWorldEdition</Name>",
        "<Name>DNSWorldWatched</Name>",
    );
    let first_rebuild = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Rebuilding
            && status.attempt() == initial_status.attempt() + 1
    })
    .await;
    replace_exact(
        &root.path().join("edt/src/Configuration/Configuration.mdo"),
        "<name>WritesFixture</name>",
        "<name>WritesWatched</name>",
    );
    let modified_snapshot = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldWatched", "WritesWatched"]
    })
    .await;
    assert_diagnostic_snapshots_complete(&modified_snapshot);
    assert_eq!(
        configuration_names(&initial),
        ["DNSWorldEdition", "WritesFixture"]
    );
    assert_eq!(
        configuration_names(&modified_snapshot),
        ["DNSWorldWatched", "WritesWatched"]
    );
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["DNSWorldWatched", "WritesWatched"]
    );
    let followed_up = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
            && status.attempt() == first_rebuild.attempt() + 1
            && status.published() == initial_status.published() + 2
    })
    .await;

    let moved_designer = moved.path().join("designer");
    fs::rename(root.path().join("designer"), &moved_designer)
        .expect("Designer root removal must succeed");
    let removed = wait_for_snapshot(&mut snapshots, |snapshot| snapshot.len() == 1).await;
    assert_diagnostic_snapshots_complete(&removed);
    let removed_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
            && status.published() == followed_up.published() + 1
    })
    .await;
    assert_eq!(
        removed.configurations()[0].configuration_id().as_str(),
        EDT_ID
    );
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["WritesWatched"]
    );

    let renamed_designer = root.path().join("designer-renamed");
    fs::rename(&moved_designer, &renamed_designer).expect("Designer root addition must succeed");
    let renamed = wait_for_snapshot(&mut snapshots, |snapshot| snapshot.len() == 2).await;
    assert_diagnostic_snapshots_complete(&renamed);
    let renamed_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
            && status.published() == removed_status.published() + 1
    })
    .await;
    assert_eq!(renamed_status.attempt(), removed_status.attempt() + 1);
    let designer = renamed
        .configurations()
        .iter()
        .find(|configuration| configuration.configuration_id().as_str() == DESIGNER_ID)
        .expect("renamed Designer configuration must be present");
    assert!(designer.root_path().ends_with("designer-renamed"));

    let edt_configuration = root.path().join("edt/src/Configuration/Configuration.mdo");
    fs::write(&edt_configuration, "<broken>").expect("invalid EDT source must be written");
    let failed = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Failed
    })
    .await;
    assert_eq!(
        failed.failure(),
        Some(WorkspaceUpdateFailureKind::SemanticBuild)
    );
    let retained = observer
        .snapshot()
        .expect("last valid snapshot must be retained");
    assert_eq!(
        configuration_names(&retained),
        ["DNSWorldWatched", "WritesWatched"]
    );
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["DNSWorldWatched", "WritesWatched"]
    );

    let repaired_source =
        fs::read_to_string(fixture_root().join("edt/src/Configuration/Configuration.mdo"))
            .expect("tracked EDT source must be readable")
            .replace("<name>WritesFixture</name>", "<name>WritesRecovered</name>");
    fs::write(&edt_configuration, repaired_source).expect("repaired EDT source must be written");
    let recovered = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldWatched", "WritesRecovered"]
    })
    .await;
    assert_diagnostic_snapshots_complete(&recovered);
    assert_eq!(
        configuration_names(&recovered),
        ["DNSWorldWatched", "WritesRecovered"]
    );
    let recovered_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.published() > failed.published()
    })
    .await;
    assert!(recovered_status.attempt() > failed.attempt());
    assert_eq!(recovered_status.failure(), None);

    let moved_workspace = moved.path().join("workspace");
    fs::rename(root.path(), &moved_workspace).expect("Workspace root removal must succeed");
    let observation_failed = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Failed
            && status.failure() == Some(WorkspaceUpdateFailureKind::Observation)
    })
    .await;
    assert_eq!(observation_failed.attempt(), recovered_status.attempt());
    assert_eq!(observation_failed.published(), recovered_status.published());
    let readiness = request(address, "/health/ready").await;
    assert_eq!(readiness.status, 200);
    assert_eq!(readiness.body, r#"{"status":"ready"}"#);
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["DNSWorldWatched", "WritesRecovered"]
    );

    fs::rename(&moved_workspace, root.path()).expect("Workspace root recovery must succeed");
    let observation_recovered = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
            && status.attempt() == observation_failed.attempt() + 1
            && status.published() == observation_failed.published() + 1
    })
    .await;
    assert_eq!(observation_recovered.failure(), None);
    assert_eq!(
        wire_configuration_names(&configuration_list(address).await),
        ["DNSWorldWatched", "WritesRecovered"]
    );

    shutdown_sender.send(()).expect("shutdown must be observed");
    timeout(Duration::from_secs(5), run)
        .await
        .expect("File Watching Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert_eq!(
        update_observer.status().phase(),
        WorkspaceUpdatePhase::Stopped
    );
    assert!(observer.snapshot().is_none());
    wait_for_watch_closed(&mut snapshots).await;
    wait_for_watch_closed(&mut updates).await;
    assert_eq!(*addresses.borrow(), None);
    let rebound = TcpListener::bind(address)
        .await
        .expect("HTTP listener address must be released");
    drop(rebound);
}

async fn run_fresh_update_once() -> (Vec<String>, WorkspaceUpdateStatus) {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshots = observer.subscribe();
    let update_observer = workspace.update_observer();
    let mut updates = update_observer.subscribe();
    let app = configured_builder(root.path())
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));
    wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;

    replace_exact(
        &root.path().join("edt/src/Configuration/Configuration.mdo"),
        "<name>WritesFixture</name>",
        "<name>WritesRepeated</name>",
    );
    let snapshot = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldEdition", "WritesRepeated"]
    })
    .await;
    let status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.published() >= 2
    })
    .await;

    shutdown_sender.send(()).expect("shutdown must be observed");
    timeout(Duration::from_secs(5), run)
        .await
        .expect("fresh Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    assert!(observer.snapshot().is_none());
    wait_for_watch_closed(&mut snapshots).await;
    wait_for_watch_closed(&mut updates).await;
    (configuration_names(&snapshot), status)
}

#[tokio::test]
async fn public_file_watching_repeats_fresh_updates_without_shared_owners() {
    let first = run_fresh_update_once().await;
    let repeated = run_fresh_update_once().await;

    assert_eq!(first, repeated);
    assert_eq!(first.0, ["DNSWorldEdition", "WritesRepeated"]);
    assert_eq!(first.1.attempt(), 2);
    assert_eq!(first.1.published(), 2);
    assert_eq!(first.1.failure(), None);
}
