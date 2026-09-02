use std::convert::Infallible;
use std::fs;
use std::future::pending;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use oneagent_analysis::refactoring::{
    NeverCancelledRefactoring, RefactoringCancellationSignal, RefactoringErrorKind,
    RefactoringFamily, RefactoringRequest, SourceOccurrenceKind, WorkspacePublicationId,
};
use oneagent_runtime::{
    App, AppBuilder, BoxError, ConfigurationProvider, HttpService, LifecycleState, RuntimeConfig,
    RuntimeError, RuntimeErrorKind, RuntimeService, ServiceContext, ServiceStartFuture,
    ServiceTask, WorkspaceBuildError, WorkspaceBuildErrorKind, WorkspaceService, WorkspaceSnapshot,
    WorkspaceSnapshotBuilder,
};
use oneagent_workspace::WorkspaceFormat;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const DESIGNER_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const EDT_ID: &str = "50000000-0000-0000-0000-000000000000";

struct PlannerCancellation(AtomicBool);

impl RefactoringCancellationSignal for PlannerCancellation {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
    http_bind_address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("OneAgent Runtime", "test")
            .with_workspace_root(self.workspace_root.clone())
            .with_http_bind_address(self.http_bind_address))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotObservation {
    configurations: Vec<ConfigurationObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigurationObservation {
    id: String,
    name: String,
    format: WorkspaceFormat,
    nodes: usize,
    edges: usize,
    diagnostics: usize,
    validation_issues: usize,
    normalized_findings: usize,
    suppressed_findings: usize,
    requests: usize,
    reference_total: usize,
    reference_resolved: usize,
    reference_unresolved: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: u16,
    body: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service")
}

fn configured_builder(root: impl Into<PathBuf>) -> AppBuilder {
    let provider = TestConfigurationProvider {
        workspace_root: root.into(),
        http_bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    };
    App::builder()
        .configure(&provider)
        .expect("test configuration must load")
}

fn observe(snapshot: &WorkspaceSnapshot) -> SnapshotObservation {
    SnapshotObservation {
        configurations: snapshot
            .configurations()
            .iter()
            .map(|configuration| {
                assert_eq!(
                    configuration.report().graph().total_nodes(),
                    configuration.graph().node_count()
                );
                assert_eq!(
                    configuration.report().graph().total_edges(),
                    configuration.graph().edge_count()
                );
                assert!(configuration.graph().validate().is_valid());
                assert!(configuration.validation().is_valid());
                assert_eq!(
                    configuration.diagnostic_report().summary().total(),
                    configuration.diagnostic_report().findings().len()
                );
                assert!(configuration.rule_execution_report().results().is_empty());
                assert!(
                    configuration
                        .rule_execution_report()
                        .diagnostics()
                        .is_empty()
                );
                assert_eq!(configuration.rule_execution_report().summary().total(), 0);

                ConfigurationObservation {
                    id: configuration.configuration_id().as_str().to_owned(),
                    name: configuration.configuration_name().as_str().to_owned(),
                    format: configuration.format(),
                    nodes: configuration.graph().node_count(),
                    edges: configuration.graph().edge_count(),
                    diagnostics: configuration.diagnostics().len(),
                    validation_issues: configuration.validation().issues().len(),
                    normalized_findings: configuration.diagnostic_report().summary().total(),
                    suppressed_findings: configuration.diagnostic_report().summary().suppressed(),
                    requests: configuration.reference_requests().len(),
                    reference_total: configuration.reference_statistics().total(),
                    reference_resolved: configuration.reference_statistics().resolved(),
                    reference_unresolved: configuration.reference_statistics().unresolved(),
                }
            })
            .collect(),
    }
}

async fn wait_for_snapshot(
    snapshot: &mut watch::Receiver<Option<Arc<WorkspaceSnapshot>>>,
) -> Arc<WorkspaceSnapshot> {
    loop {
        if let Some(snapshot) = snapshot.borrow().clone() {
            return snapshot;
        }
        timeout(Duration::from_secs(1), snapshot.changed())
            .await
            .expect("Workspace snapshot wait must not hang")
            .expect("Workspace service must retain snapshot ownership");
    }
}

async fn wait_for_snapshot_clear(snapshot: &mut watch::Receiver<Option<Arc<WorkspaceSnapshot>>>) {
    while snapshot.borrow().is_some() {
        let _ = timeout(Duration::from_secs(1), snapshot.changed())
            .await
            .expect("Workspace snapshot cleanup must not hang");
    }
    assert!(snapshot.borrow_and_update().is_none());
    assert!(
        snapshot.changed().await.is_err(),
        "Workspace snapshot sender must not survive App::run"
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

async fn request(address: SocketAddr, path: &str) -> Response {
    let mut stream = timeout(Duration::from_secs(1), TcpStream::connect(address))
        .await
        .expect("HTTP connect must not hang")
        .expect("HTTP listener must accept loopback connections");
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("HTTP request must write");
    let mut bytes = Vec::new();
    timeout(Duration::from_secs(1), stream.read_to_end(&mut bytes))
        .await
        .expect("HTTP response must not hang")
        .expect("HTTP response must read");
    let response = std::str::from_utf8(&bytes).expect("HTTP response must be UTF-8");
    let (head, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response must contain a header terminator");
    let status = head
        .lines()
        .next()
        .expect("HTTP response must contain a status line")
        .split_whitespace()
        .nth(1)
        .expect("HTTP status line must contain a code")
        .parse()
        .expect("HTTP status code must be numeric");
    Response {
        status,
        body: body.to_owned(),
    }
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

fn copy_fixture() -> tempfile::TempDir {
    let temporary = tempdir().expect("temporary Workspace root must be created");
    copy_tree(&fixture_root(), temporary.path());
    temporary
}

fn refactoring_request(
    snapshot: &WorkspaceSnapshot,
    configuration_id: &str,
    current_name: &str,
    desired_name: &str,
    publication_id: WorkspacePublicationId,
) -> RefactoringRequest {
    let configuration_id = oneagent_common::EntityId::new(configuration_id)
        .expect("fixture Configuration ID must be valid");
    let configuration = snapshot
        .configuration(&configuration_id)
        .expect("fixture Configuration must be published");
    let target = configuration
        .source_evidence()
        .documents()
        .iter()
        .flat_map(oneagent_analysis::refactoring::SourceDocument::occurrences)
        .find(|occurrence| {
            occurrence.kind() == SourceOccurrenceKind::Declaration
                && occurrence.token() == current_name
        })
        .and_then(oneagent_analysis::refactoring::SourceOccurrence::mapped_target_id)
        .expect("fixture declaration must map uniquely")
        .clone();
    RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        publication_id,
        configuration_id,
        target,
        desired_name,
    )
    .expect("fixture refactoring request must be valid")
}

fn workspace_source(error: &RuntimeError) -> &WorkspaceBuildError {
    std::error::Error::source(error)
        .and_then(|source| source.downcast_ref::<WorkspaceBuildError>())
        .expect("named Workspace startup failure must preserve its typed source")
}

async fn assert_workspace_start_failure(root: &Path, expected: WorkspaceBuildErrorKind) {
    let service = WorkspaceService::new();
    let observer = service.snapshot_observer();
    let app = configured_builder(root)
        .register_service("workspace", service)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let lifecycle = app.subscribe_lifecycle();

    let error = timeout(
        Duration::from_secs(1),
        app.run(pending::<Result<(), Infallible>>()),
    )
    .await
    .expect("Workspace startup failure must not hang")
    .expect_err("invalid Workspace must fail startup");

    assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
    assert_eq!(error.service_name(), Some("workspace"));
    assert_eq!(workspace_source(&error).kind(), expected);
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopped);
    assert!(observer.snapshot().is_none());
}

async fn run_mixed_workspace_once() -> SnapshotObservation {
    let root = copy_fixture();
    let service = WorkspaceService::new();
    let observer = service.snapshot_observer();
    let mut snapshot_changes = observer.subscribe();
    let app = configured_builder(root.path())
        .register_service("workspace", service)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let snapshot = wait_for_snapshot(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    let observation = observe(&snapshot);
    assert!(observer.snapshot().is_some());

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("Workspace Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_snapshot_clear(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert!(observer.snapshot().is_none());

    observation
}

#[tokio::test]
async fn public_workspace_builds_both_production_formats_deterministically() {
    let first = run_mixed_workspace_once().await;
    let repeated = run_mixed_workspace_once().await;

    assert_eq!(first, repeated);
    assert_eq!(first.configurations.len(), 2);
    assert_eq!(
        first.configurations[0],
        ConfigurationObservation {
            id: DESIGNER_ID.to_owned(),
            name: "DNSWorldEdition".to_owned(),
            format: WorkspaceFormat::DesignerXml,
            nodes: 4,
            edges: 3,
            diagnostics: 0,
            validation_issues: 0,
            normalized_findings: 0,
            suppressed_findings: 0,
            requests: 0,
            reference_total: 0,
            reference_resolved: 0,
            reference_unresolved: 0,
        }
    );
    assert_eq!(first.configurations[1].id, EDT_ID);
    assert_eq!(first.configurations[1].name, "WritesFixture");
    assert_eq!(first.configurations[1].format, WorkspaceFormat::Edt);
    assert_eq!(first.configurations[1].nodes, 13);
    assert_eq!(first.configurations[1].edges, 14);
    assert_eq!(first.configurations[1].diagnostics, 3);
    assert_eq!(first.configurations[1].validation_issues, 0);
    assert_eq!(first.configurations[1].normalized_findings, 3);
    assert_eq!(first.configurations[1].suppressed_findings, 0);
    assert_eq!(first.configurations[1].requests, 1);
    assert_eq!(first.configurations[1].reference_total, 5);
    assert_eq!(first.configurations[1].reference_resolved, 2);
    assert_eq!(first.configurations[1].reference_unresolved, 3);
}

#[test]
fn public_workspace_builds_each_configuration_at_the_workspace_root() {
    for directory in ["edt", "designer"] {
        let root = tempdir().expect("temporary direct-root Workspace must be created");
        copy_tree(&fixture_root().join(directory), root.path());
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect("Configuration at the Workspace root must build");
        assert_eq!(snapshot.len(), 1);
        let configuration = &snapshot.configurations()[0];
        assert_eq!(configuration.root_path(), root.path());
        assert!(
            configuration
                .source_evidence()
                .documents()
                .iter()
                .all(|document| !document.path().path().as_str().starts_with(directory))
        );
    }
}

#[test]
fn public_workspace_plans_repeatedly_from_retained_edt_and_designer_publications() {
    let root = copy_fixture();
    let snapshot = Arc::new(
        WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect("tracked Workspace fixture must build with source evidence"),
    );
    assert_eq!(snapshot.len(), 2);
    assert!(snapshot.configurations().iter().all(|configuration| {
        configuration.source_evidence().configuration_id() == configuration.configuration_id()
            && !configuration.source_evidence().documents().is_empty()
    }));

    let edt_request = refactoring_request(
        &snapshot,
        EDT_ID,
        "Posting",
        "PostingRenamed",
        snapshot.publication_id(),
    );
    let designer_request = refactoring_request(
        &snapshot,
        DESIGNER_ID,
        "FillSecurityCollection",
        "FillSecurityCollectionRenamed",
        snapshot.publication_id(),
    );
    let edt_plan = snapshot
        .plan_refactoring(&edt_request, &NeverCancelledRefactoring)
        .expect("EDT Workspace plan must succeed");
    let designer_plan = snapshot
        .plan_refactoring(&designer_request, &NeverCancelledRefactoring)
        .expect("Designer Workspace plan must succeed");
    assert_eq!(edt_plan.plan().operations().len(), 1);
    assert_eq!(designer_plan.plan().operations().len(), 1);
    assert_eq!(edt_plan.preview().entries().len(), 1);
    assert_eq!(designer_plan.preview().entries().len(), 1);
    assert_eq!(
        snapshot
            .plan_refactoring(&edt_request, &NeverCancelledRefactoring)
            .expect("repeated EDT Workspace plan must succeed"),
        edt_plan
    );
    assert_eq!(
        snapshot
            .plan_refactoring(&designer_request, &NeverCancelledRefactoring)
            .expect("repeated Designer Workspace plan must succeed"),
        designer_plan
    );

    let edt_path = root
        .path()
        .join("edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl");
    let designer_path = root
        .path()
        .join("designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl");
    let edt_source = fs::read(&edt_path).expect("EDT source must be readable before mutation");
    fs::write(
        &edt_path,
        String::from_utf8(edt_source.clone())
            .expect("EDT fixture must be UTF-8")
            .replace(
                "Procedure Posting()",
                "Procedure Posting() // changed after publication",
            ),
    )
    .expect("EDT source mutation must succeed");
    fs::remove_file(&designer_path).expect("Designer source removal must succeed");
    let renamed_edt_path = edt_path.with_extension("published-away");
    fs::rename(&edt_path, &renamed_edt_path)
        .expect("published EDT source must become unreadable at its original path");
    assert!(fs::read(&edt_path).is_err());
    assert!(fs::read(&designer_path).is_err());

    assert_eq!(
        snapshot
            .plan_refactoring(&edt_request, &NeverCancelledRefactoring)
            .expect("retained EDT publication must not reread source"),
        edt_plan
    );
    assert_eq!(
        snapshot
            .plan_refactoring(&designer_request, &NeverCancelledRefactoring)
            .expect("retained Designer publication must not reread source"),
        designer_plan
    );
}

#[test]
fn public_workspace_refactoring_failures_are_atomic_and_preserve_the_snapshot() {
    let root = copy_fixture();
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(root.path())
        .expect("tracked Workspace fixture must build with source evidence");
    let request = refactoring_request(
        &snapshot,
        EDT_ID,
        "Posting",
        "PostingRenamed",
        snapshot.publication_id(),
    );
    let before = snapshot
        .plan_refactoring(&request, &NeverCancelledRefactoring)
        .expect("baseline plan must succeed");

    let cancelled = PlannerCancellation(AtomicBool::new(true));
    let cancelled_error = snapshot
        .plan_refactoring(&request, &cancelled)
        .expect_err("cancelled Workspace planning must expose no result");
    assert_eq!(cancelled_error.kind(), RefactoringErrorKind::Cancelled);

    let stale_request = refactoring_request(
        &snapshot,
        EDT_ID,
        "Posting",
        "PostingRenamed",
        WorkspacePublicationId::new(snapshot.publication_id().get() + 1)
            .expect("successor publication ID must be valid"),
    );
    let stale_error = snapshot
        .plan_refactoring(&stale_request, &NeverCancelledRefactoring)
        .expect_err("stale Workspace publication must expose no result");
    assert_eq!(
        stale_error.kind(),
        RefactoringErrorKind::PublicationMismatch
    );

    let missing_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        snapshot.publication_id(),
        oneagent_common::EntityId::new("configuration.missing")
            .expect("missing Configuration ID must be valid"),
        oneagent_common::EntityId::new("target.missing").expect("missing target ID must be valid"),
        "Renamed",
    )
    .expect("missing Configuration request shape must be valid");
    let missing_error = snapshot
        .plan_refactoring(&missing_request, &NeverCancelledRefactoring)
        .expect_err("missing Workspace Configuration must expose no result");
    assert_eq!(
        missing_error.kind(),
        RefactoringErrorKind::ConfigurationNotFound
    );

    assert_eq!(
        snapshot
            .plan_refactoring(&request, &NeverCancelledRefactoring)
            .expect("valid Workspace state must survive failed requests"),
        before
    );
}

#[tokio::test]
async fn public_workspace_accepts_empty_root_and_repeats_fresh_runs() {
    let root = tempdir().expect("temporary empty Workspace must be created");

    for _ in 0..2 {
        let service = WorkspaceService::new();
        let observer = service.snapshot_observer();
        let mut snapshot_changes = observer.subscribe();
        let app = configured_builder(root.path())
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));

        assert!(wait_for_snapshot(&mut snapshot_changes).await.is_empty());
        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("empty Workspace shutdown must not hang")
            .expect("Runtime task must join")
            .expect("empty Workspace shutdown must succeed");
        wait_for_snapshot_clear(&mut snapshot_changes).await;
        assert!(observer.snapshot().is_none());
    }
}

#[tokio::test]
async fn public_workspace_rejects_invalid_roots_and_conflicting_markers() {
    let parent = tempdir().expect("temporary parent must be created");
    let missing = parent.path().join("missing");
    assert_workspace_start_failure(&missing, WorkspaceBuildErrorKind::ObservationFailed).await;

    let file = parent.path().join("not-a-directory");
    fs::write(&file, "not a Workspace").expect("non-directory root must be created");
    assert_workspace_start_failure(&file, WorkspaceBuildErrorKind::ObservationFailed).await;

    let conflict = parent.path().join("conflicting-workspace");
    copy_tree(&fixture_root().join("designer"), &conflict);
    fs::copy(
        fixture_root().join("edt/.project"),
        conflict.join(".project"),
    )
    .expect("EDT marker must be copied into Designer root");
    fs::create_dir_all(conflict.join("src/Configuration"))
        .expect("EDT Configuration directory must be created in conflicting root");
    fs::copy(
        fixture_root().join("edt/src/Configuration/Configuration.mdo"),
        conflict.join("src/Configuration/Configuration.mdo"),
    )
    .expect("EDT Configuration marker must be copied into conflicting root");
    assert_workspace_start_failure(&conflict, WorkspaceBuildErrorKind::DiscoveryFailed).await;
}

#[tokio::test]
async fn public_workspace_rejects_duplicate_identity_without_publication() {
    let temporary = tempdir().expect("temporary Workspace copy must be created");
    copy_tree(&fixture_root(), temporary.path());
    let configuration_path = temporary
        .path()
        .join("edt/src/Configuration/Configuration.mdo");
    let configuration = fs::read_to_string(&configuration_path)
        .expect("EDT Configuration mutation source must be readable")
        .replace(EDT_ID, DESIGNER_ID);
    fs::write(&configuration_path, configuration)
        .expect("duplicate EDT Configuration identity must be written");

    assert_workspace_start_failure(
        temporary.path(),
        WorkspaceBuildErrorKind::DuplicateConfigurationIdentity,
    )
    .await;
}

#[tokio::test]
async fn public_workspace_keeps_later_adapter_failure_atomic() {
    let temporary = tempdir().expect("temporary Workspace copy must be created");
    copy_tree(&fixture_root(), temporary.path());
    fs::write(
        temporary
            .path()
            .join("edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl"),
        "Procedure MissingParenthesis",
    )
    .expect("fatal EDT BSL mutation must be written");

    assert_workspace_start_failure(
        temporary.path(),
        WorkspaceBuildErrorKind::SemanticBuildFailed,
    )
    .await;
}

#[tokio::test]
async fn public_workspace_snapshot_and_health_follow_owned_lifecycle() {
    let root = copy_fixture();
    let http = HttpService::new();
    let mut address = http.subscribe_bound_address();
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshot_changes = observer.subscribe();
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

    let actual_address = wait_for_address(&mut address).await;
    let snapshot = wait_for_snapshot(&mut snapshot_changes).await;
    timeout(Duration::from_secs(1), start_attempted)
        .await
        .expect("post-Workspace gate startup must not hang")
        .expect("post-Workspace gate must attempt startup");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);
    assert_eq!(snapshot.len(), 2);
    assert_eq!(request(actual_address, "/health/live").await.status, 200);
    assert_eq!(
        request(actual_address, "/health/ready").await,
        Response {
            status: 503,
            body: "{\"status\":\"not_ready\"}".to_owned(),
        }
    );

    start_release_sender
        .send(())
        .expect("post-Workspace startup gate must be released");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    assert_eq!(
        request(actual_address, "/health/ready").await,
        Response {
            status: 200,
            body: "{\"status\":\"ready\"}".to_owned(),
        }
    );

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(Duration::from_secs(1), stopping)
        .await
        .expect("reverse cleanup gate must not hang")
        .expect("reverse cleanup gate must observe cancellation");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopping);
    assert!(observer.snapshot().is_some());
    assert_eq!(
        request(actual_address, "/health/ready").await,
        Response {
            status: 503,
            body: "{\"status\":\"not_ready\"}".to_owned(),
        }
    );

    stop_release_sender
        .send(())
        .expect("reverse cleanup gate must be released");
    timeout(Duration::from_secs(1), run)
        .await
        .expect("Workspace and HTTP shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_snapshot_clear(&mut snapshot_changes).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
    assert_eq!(*address.borrow(), None);
    assert!(observer.snapshot().is_none());
}
