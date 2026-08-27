use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use oneagent_runtime::{
    App, AppBuilder, BoxError, ConfigurationProvider, GraphQueryConfigurationList, GraphQueryLimit,
    GraphQueryNodeResult, GraphQueryService, HttpService, LifecycleState, RuntimeConfig,
    WorkspaceCacheLoadOutcome, WorkspaceCacheStatus, WorkspaceCacheWriteOutcome, WorkspaceService,
    WorkspaceSnapshot, WorkspaceUpdatePhase, WorkspaceUpdateStatus,
};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const DESIGNER_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const EDT_ID: &str = "50000000-0000-0000-0000-000000000000";
const CACHE_RELATIVE_PATH: &str = ".oneagent/cache/workspace-v1.json";
const CACHE_TEMPORARY_RELATIVE_PATH: &str = ".oneagent/cache/workspace-v1.tmp";
const TEST_TIMEOUT: Duration = Duration::from_secs(5);
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
    http_bind_address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(
            RuntimeConfig::new("OneAgent Runtime", "persistent-cache-test")
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryObservation {
    configurations: GraphQueryConfigurationList,
    designer_configuration: GraphQueryNodeResult,
    edt_configuration: GraphQueryNodeResult,
    wire_configurations: RawResponse,
    live: RawResponse,
    ready: RawResponse,
}

#[derive(Debug)]
struct RunObservation {
    snapshot: Arc<WorkspaceSnapshot>,
    query: QueryObservation,
    cache: WorkspaceCacheStatus,
    updates: WorkspaceUpdateStatus,
}

#[derive(Debug, Clone, Copy)]
enum Corruption {
    Malformed,
    Truncated,
    Partial,
    Checksum,
    Semantic,
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

fn configured_builder(root: impl Into<PathBuf>) -> AppBuilder {
    let provider = TestConfigurationProvider {
        workspace_root: root.into(),
        http_bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    };
    App::builder()
        .configure(&provider)
        .expect("Persistent Cache test configuration must load")
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
    lifecycle: &mut watch::Receiver<LifecycleState>,
    expected: LifecycleState,
) {
    while *lifecycle.borrow() != expected {
        timeout(TEST_TIMEOUT, lifecycle.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("Runtime must retain lifecycle ownership");
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
        timeout(TEST_TIMEOUT, snapshots.changed())
            .await
            .expect("Workspace snapshot wait must not hang")
            .expect("Workspace service must retain snapshot ownership");
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
        timeout(TEST_TIMEOUT, updates.changed())
            .await
            .expect("Workspace update wait must not hang")
            .expect("Workspace service must retain update ownership");
    }
}

async fn wait_for_watch_closed<T>(receiver: &mut watch::Receiver<T>) {
    loop {
        match timeout(TEST_TIMEOUT, receiver.changed())
            .await
            .expect("watch closure wait must not hang")
        {
            Ok(()) => {}
            Err(_) => return,
        }
    }
}

async fn request(address: SocketAddr, target: &str) -> RawResponse {
    let mut stream = timeout(TEST_TIMEOUT, TcpStream::connect(address))
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
    timeout(TEST_TIMEOUT, stream.read_to_end(&mut bytes))
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
        .filter_map(|line| {
            let (name, value) = line
                .split_once(':')
                .expect("HTTP header must contain a colon");
            let name = name.to_ascii_lowercase();
            (name != "date").then(|| (name, value.trim().to_owned()))
        })
        .collect();
    RawResponse {
        status,
        headers,
        body: body.to_owned(),
    }
}

async fn observe_queries(graph_query: &GraphQueryService, address: SocketAddr) -> QueryObservation {
    let configurations = graph_query
        .configurations(GraphQueryLimit::default())
        .expect("published configurations must be queryable");
    let designer_configuration = graph_query
        .node(DESIGNER_ID, DESIGNER_ID)
        .expect("Designer Configuration node must be queryable");
    let edt_configuration = graph_query
        .node(EDT_ID, EDT_ID)
        .expect("EDT Configuration node must be queryable");
    QueryObservation {
        configurations,
        designer_configuration,
        edt_configuration,
        wire_configurations: request(address, "/api/v1/configurations").await,
        live: request(address, "/health/live").await,
        ready: request(address, "/health/ready").await,
    }
}

async fn run_once(root: &Path) -> RunObservation {
    let workspace = WorkspaceService::new();
    let snapshot_observer = workspace.snapshot_observer();
    let mut snapshots = snapshot_observer.subscribe();
    let cache_observer = workspace.cache_observer();
    let mut cache_changes = cache_observer.subscribe();
    let update_observer = workspace.update_observer();
    let mut updates = update_observer.subscribe();
    let graph_query = GraphQueryService::new(snapshot_observer.clone());
    let http = HttpService::with_graph_query(graph_query.clone());
    let mut addresses = http.subscribe_bound_address();
    let app = configured_builder(root)
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
    let snapshot = wait_for_snapshot(&mut snapshots, |_| true).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    let query = observe_queries(&graph_query, address).await;
    assert_eq!(query.live.status, 200);
    assert_eq!(query.live.body, "{\"status\":\"alive\"}");
    assert_eq!(query.ready.status, 200);
    assert_eq!(query.ready.body, "{\"status\":\"ready\"}");
    assert_eq!(query.wire_configurations.status, 200);
    let cache = cache_observer.status();
    let update = update_observer.status();
    assert_eq!(update.phase(), WorkspaceUpdatePhase::Watching);
    assert_eq!(update.attempt(), 1);
    assert_eq!(update.published(), 1);

    shutdown_sender.send(()).expect("shutdown must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    wait_for_watch_closed(&mut snapshots).await;
    wait_for_watch_closed(&mut updates).await;
    wait_for_watch_closed(&mut cache_changes).await;
    assert!(snapshot_observer.snapshot().is_none());
    assert_eq!(
        update_observer.status().phase(),
        WorkspaceUpdatePhase::Stopped
    );
    assert_eq!(*addresses.borrow(), None);
    let rebound = TcpListener::bind(address)
        .await
        .expect("HTTP listener address must be released");
    drop(rebound);

    RunObservation {
        snapshot,
        query,
        cache,
        updates: update,
    }
}

fn assert_snapshot_equivalent(expected: &WorkspaceSnapshot, actual: &WorkspaceSnapshot) {
    assert_eq!(expected.len(), actual.len());
    for (expected, actual) in expected
        .configurations()
        .iter()
        .zip(actual.configurations())
    {
        assert_eq!(expected.root_path(), actual.root_path());
        assert_eq!(expected.format(), actual.format());
        assert_eq!(expected.configuration_id(), actual.configuration_id());
        assert_eq!(expected.configuration_name(), actual.configuration_name());
        assert!(expected.graph().diff(actual.graph()).is_empty());
        assert_eq!(expected.diagnostics(), actual.diagnostics());
        assert_eq!(expected.reference_requests(), actual.reference_requests());
        assert_eq!(
            expected.reference_statistics(),
            actual.reference_statistics()
        );
        assert_eq!(expected.report(), actual.report());
        assert!(actual.graph().validate().is_valid());
    }
}

fn set_cache_version(root: &Path, field: &str, version: u64) {
    let path = root.join(CACHE_RELATIVE_PATH);
    let bytes = fs::read(&path).expect("cache candidate must be readable");
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("cache candidate must be JSON");
    value
        .as_object_mut()
        .expect("cache envelope must be an object")
        .insert(field.to_owned(), serde_json::Value::from(version));
    fs::write(
        path,
        serde_json::to_vec(&value).expect("mutated cache must encode"),
    )
    .expect("mutated cache must be written");
}

fn corrupt_cache(root: &Path, corruption: Corruption) {
    let path = root.join(CACHE_RELATIVE_PATH);
    let bytes = fs::read(&path).expect("cache candidate must be readable");
    let corrupted = match corruption {
        Corruption::Malformed => b"{".to_vec(),
        Corruption::Truncated => bytes[..bytes.len() / 2].to_vec(),
        Corruption::Partial => b"{}".to_vec(),
        Corruption::Checksum => replace_checksum(&bytes, "fnv1a64:0000000000000000"),
        Corruption::Semantic => semantically_invalid_cache(&bytes),
    };
    fs::write(path, corrupted).expect("corrupted cache must be written");
}

fn replace_checksum(bytes: &[u8], replacement: &str) -> Vec<u8> {
    let mut value = String::from_utf8(bytes.to_vec()).expect("cache bytes must be UTF-8");
    let prefix = "\"content_checksum\":\"";
    let start = value
        .find(prefix)
        .map(|index| index + prefix.len())
        .expect("cache checksum must exist");
    let end = value[start..]
        .find('"')
        .map(|index| start + index)
        .expect("cache checksum must terminate");
    value.replace_range(start..end, replacement);
    value.into_bytes()
}

fn semantically_invalid_cache(bytes: &[u8]) -> Vec<u8> {
    let mut value = String::from_utf8(bytes.to_vec()).expect("cache bytes must be UTF-8");
    let valid = "\"name\":\"DNSWorldEdition\"";
    assert_eq!(value.matches(valid).count(), 1);
    value = value.replace(valid, "\"name\":\"\"");
    let source = value
        .find("\"source\":")
        .expect("cache source field must exist");
    let mut content = Vec::with_capacity(value.len() - source + 1);
    content.push(b'{');
    content.extend_from_slice(&value.as_bytes()[source..]);
    let hash = content.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    replace_checksum(value.as_bytes(), &format!("fnv1a64:{hash:016x}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn public_persistent_cache_cold_and_warm_runs_are_complete_and_equivalent() {
    let root = copy_fixture();
    assert!(!root.path().join(CACHE_RELATIVE_PATH).exists());

    let cold = run_once(root.path()).await;
    assert_eq!(cold.cache.load(), WorkspaceCacheLoadOutcome::Missing);
    assert_eq!(cold.cache.write(), WorkspaceCacheWriteOutcome::Succeeded);
    assert_eq!(cold.query.configurations.configurations().len(), 2);
    assert!(root.path().join(CACHE_RELATIVE_PATH).is_file());
    assert!(!root.path().join(CACHE_TEMPORARY_RELATIVE_PATH).exists());

    let warm = run_once(root.path()).await;
    assert_eq!(warm.cache.load(), WorkspaceCacheLoadOutcome::Hit);
    assert_eq!(warm.cache.write(), WorkspaceCacheWriteOutcome::NotAttempted);
    assert_eq!(warm.updates.attempt(), 1);
    assert_eq!(warm.updates.published(), 1);
    assert_snapshot_equivalent(&cold.snapshot, &warm.snapshot);
    assert_eq!(cold.query, warm.query);
}

#[tokio::test(flavor = "multi_thread")]
async fn public_persistent_cache_invalidates_rejects_and_cleanly_recovers() {
    let root = copy_fixture();
    let cold = run_once(root.path()).await;
    let cache_path = root.path().join(CACHE_RELATIVE_PATH);
    let temporary_path = root.path().join(CACHE_TEMPORARY_RELATIVE_PATH);

    fs::write(&temporary_path, b"stale temporary cache")
        .expect("stale temporary cache must be created");
    replace_exact(
        &root.path().join("edt/src/Configuration/Configuration.mdo"),
        "WritesFixture",
        "WritesCacheInvalidated",
    );
    let source_changed = run_once(root.path()).await;
    assert_eq!(
        source_changed.cache.load(),
        WorkspaceCacheLoadOutcome::SourceChanged
    );
    assert_eq!(
        source_changed.cache.write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );
    assert!(!temporary_path.exists());
    assert_eq!(
        source_changed.snapshot.configurations()[1]
            .configuration_name()
            .as_str(),
        "WritesCacheInvalidated"
    );
    assert!(
        !cold.snapshot.configurations()[1]
            .graph()
            .diff(source_changed.snapshot.configurations()[1].graph())
            .is_empty()
    );

    for (field, version) in [
        ("schema_version", 0),
        ("schema_version", 2),
        ("semantic_version", 0),
        ("semantic_version", 3),
    ] {
        set_cache_version(root.path(), field, version);
        let recovered = run_once(root.path()).await;
        assert_eq!(
            recovered.cache.load(),
            WorkspaceCacheLoadOutcome::Incompatible
        );
        assert_eq!(
            recovered.cache.write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        assert_snapshot_equivalent(&source_changed.snapshot, &recovered.snapshot);
        assert_eq!(source_changed.query, recovered.query);
    }

    for corruption in [
        Corruption::Malformed,
        Corruption::Truncated,
        Corruption::Partial,
        Corruption::Checksum,
        Corruption::Semantic,
    ] {
        corrupt_cache(root.path(), corruption);
        let recovered = run_once(root.path()).await;
        assert_eq!(
            recovered.cache.load(),
            WorkspaceCacheLoadOutcome::Corrupt,
            "unexpected classification for {corruption:?}"
        );
        assert_eq!(
            recovered.cache.write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        assert_snapshot_equivalent(&source_changed.snapshot, &recovered.snapshot);
        assert_eq!(source_changed.query, recovered.query);
        assert!(cache_path.is_file());
        assert!(!temporary_path.exists());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn public_persistent_cache_write_failure_is_nonfatal_and_recoverable() {
    let root = copy_fixture();
    let owner = root.path().join(".oneagent");
    fs::write(&owner, b"wrong cache owner kind").expect("wrong-kind cache owner must be created");

    let failed = run_once(root.path()).await;
    assert_eq!(failed.cache.load(), WorkspaceCacheLoadOutcome::Unavailable);
    assert_eq!(failed.cache.write(), WorkspaceCacheWriteOutcome::Failed);
    assert_eq!(failed.query.configurations.configurations().len(), 2);
    assert!(owner.is_file());
    assert!(!root.path().join(CACHE_RELATIVE_PATH).exists());

    fs::remove_file(&owner).expect("wrong-kind cache owner must be removed");
    let repaired = run_once(root.path()).await;
    assert_eq!(repaired.cache.load(), WorkspaceCacheLoadOutcome::Missing);
    assert_eq!(
        repaired.cache.write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );
    assert_snapshot_equivalent(&failed.snapshot, &repaired.snapshot);
    assert_eq!(failed.query, repaired.query);

    let warm = run_once(root.path()).await;
    assert_eq!(warm.cache.load(), WorkspaceCacheLoadOutcome::Hit);
    assert_eq!(warm.cache.write(), WorkspaceCacheWriteOutcome::NotAttempted);
    assert_snapshot_equivalent(&repaired.snapshot, &warm.snapshot);
}

#[tokio::test(flavor = "multi_thread")]
#[allow(clippy::too_many_lines)] // One public lifecycle proves both replacements and warm reuse.
async fn public_persistent_cache_watched_replacements_are_atomic_and_warm_reusable() {
    let root = copy_fixture();
    let workspace = WorkspaceService::new();
    let snapshot_observer = workspace.snapshot_observer();
    let mut snapshots = snapshot_observer.subscribe();
    let cache_observer = workspace.cache_observer();
    let mut cache_changes = cache_observer.subscribe();
    let update_observer = workspace.update_observer();
    let mut updates = update_observer.subscribe();
    let graph_query = GraphQueryService::new(snapshot_observer.clone());
    let app = configured_builder(root.path())
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let initial = wait_for_snapshot(&mut snapshots, |_| true).await;
    let initial_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 1
    })
    .await;
    assert_eq!(initial_status.attempt(), 1);
    assert_eq!(
        cache_observer.status().load(),
        WorkspaceCacheLoadOutcome::Missing
    );
    assert_eq!(
        cache_observer.status().write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );
    let cache_path = root.path().join(CACHE_RELATIVE_PATH);
    let initial_cache = fs::read(&cache_path).expect("initial cache entry must be readable");

    replace_exact(
        &root.path().join("designer/Configuration.xml"),
        "DNSWorldEdition",
        "DNSWorldCached",
    );
    let first_replacement_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 2
    })
    .await;
    assert_eq!(first_replacement_status.attempt(), 2);
    let first_replacement = wait_for_snapshot(&mut snapshots, |snapshot| {
        snapshot.configurations()[0].configuration_name().as_str() == "DNSWorldCached"
    })
    .await;
    assert_eq!(
        initial.configurations()[0].configuration_name().as_str(),
        "DNSWorldEdition"
    );
    assert_ne!(
        fs::read(&cache_path).expect("replacement cache entry must be readable"),
        initial_cache
    );
    assert_eq!(
        graph_query
            .configurations(GraphQueryLimit::default())
            .expect("replacement must be queryable")
            .configurations()[0]
            .name(),
        "DNSWorldCached"
    );

    fs::write(
        root.path().join(".oneagent/cache/ignored-public-probe"),
        b"cache-owned state",
    )
    .expect("cache-owned probe must be written");
    replace_exact(
        &root.path().join("edt/src/Configuration/Configuration.mdo"),
        "WritesFixture",
        "WritesCached",
    );
    let second_replacement_status = wait_for_update(&mut updates, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 3
    })
    .await;
    assert_eq!(second_replacement_status.attempt(), 3);
    let second_replacement = wait_for_snapshot(&mut snapshots, |snapshot| {
        snapshot.configurations()[1].configuration_name().as_str() == "WritesCached"
    })
    .await;
    assert_eq!(
        first_replacement.configurations()[1]
            .configuration_name()
            .as_str(),
        "WritesFixture"
    );
    assert_eq!(
        cache_observer.status().write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );
    assert!(!root.path().join(CACHE_TEMPORARY_RELATIVE_PATH).exists());

    shutdown_sender.send(()).expect("shutdown must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_watch_closed(&mut snapshots).await;
    wait_for_watch_closed(&mut updates).await;
    wait_for_watch_closed(&mut cache_changes).await;
    assert!(snapshot_observer.snapshot().is_none());
    assert_eq!(
        update_observer.status().phase(),
        WorkspaceUpdatePhase::Stopped
    );

    let warm = run_once(root.path()).await;
    assert_eq!(warm.cache.load(), WorkspaceCacheLoadOutcome::Hit);
    assert_eq!(warm.cache.write(), WorkspaceCacheWriteOutcome::NotAttempted);
    assert_snapshot_equivalent(&second_replacement, &warm.snapshot);
    assert_eq!(
        warm.query.configurations.configurations()[0].name(),
        "DNSWorldCached"
    );
    assert_eq!(
        warm.query.configurations.configurations()[1].name(),
        "WritesCached"
    );
}
