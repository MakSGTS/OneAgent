use oneagent_graph::SemanticGraphDiff;
use oneagent_runtime::{
    App, AppBuilder, BoxError, ConfigurationProvider, GitRepositoryReader, GraphQueryLimit,
    GraphQueryService, RepositoryChange, RepositoryChangeKind, RuntimeConfig,
    WorkspaceCacheLoadOutcome, WorkspaceCacheWriteOutcome, WorkspaceChangeInputHandle,
    WorkspaceChangeSubmissionOutcome, WorkspaceService, WorkspaceSnapshot,
    WorkspaceUpdateFailureKind, WorkspaceUpdatePhase, WorkspaceUpdateStatus,
};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
struct TestConfigurationProvider {
    workspace_root: PathBuf,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(
            RuntimeConfig::new("OneAgent Runtime", "git-change-workspace-test")
                .with_workspace_root(self.workspace_root.clone()),
        )
    }
}

struct GitWorkspace {
    _temp: TempDir,
    root: PathBuf,
}

impl GitWorkspace {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary Git Workspace owner must be created");
        let root = temp.path().join("workspace");
        fs::create_dir(&root).expect("temporary Git Workspace root must be created");
        copy_tree(&fixture_root(), &root);
        fs::write(root.join(".gitignore"), ".oneagent/\n")
            .expect("cache ignore rule must be written");
        fs::write(root.join("tracked-remove.txt"), "remove\n")
            .expect("tracked removal fixture must be written");
        fs::write(root.join("tracked-move.txt"), "move\n")
            .expect("tracked move fixture must be written");
        git_at(temp.path(), ["init", "workspace"]);
        git_at(&root, ["config", "user.name", "OneAgent Test"]);
        git_at(&root, ["config", "user.email", "oneagent@example.invalid"]);
        git_at(&root, ["config", "commit.gpgsign", "false"]);
        git_at(&root, ["add", "--all"]);
        git_at(&root, ["commit", "-m", "initial"]);
        Self { _temp: temp, root }
    }

    fn git<I, S>(&self, arguments: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        git_at(&self.root, arguments)
    }
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
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture entry type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).expect("fixture file must be copied");
        }
    }
}

fn git_at<I, S>(directory: &Path, arguments: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            directory.join("missing-global-git-config"),
        )
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .expect("local Git executable must start");
    assert!(
        output.status.success(),
        "fixture Git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn configured_builder(root: impl Into<PathBuf>) -> AppBuilder {
    let provider = TestConfigurationProvider {
        workspace_root: root.into(),
    };
    App::builder()
        .configure(&provider)
        .expect("Git Workspace test configuration must load")
}

fn replace_exact(path: &Path, before: &str, after: &str) {
    let source = fs::read_to_string(path).expect("source must be readable");
    assert_eq!(source.matches(before).count(), 1);
    fs::write(path, source.replace(before, after)).expect("source replacement must be written");
}

fn configuration_names(snapshot: &WorkspaceSnapshot) -> Vec<String> {
    snapshot
        .configurations()
        .iter()
        .map(|configuration| configuration.configuration_name().as_str().to_owned())
        .collect()
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

async fn submit_after_current_update(
    input: &WorkspaceChangeInputHandle,
    updates: &mut watch::Receiver<WorkspaceUpdateStatus>,
    change_set: oneagent_runtime::GitChangeSet,
) -> WorkspaceUpdateStatus {
    let settled = wait_for_update(updates, |status| {
        matches!(
            status.phase(),
            WorkspaceUpdatePhase::Watching | WorkspaceUpdatePhase::Failed
        )
    })
    .await;
    assert_eq!(
        input.submit(change_set),
        WorkspaceChangeSubmissionOutcome::Accepted
    );
    wait_for_update(updates, |status| {
        status.attempt() > settled.attempt()
            && matches!(
                status.phase(),
                WorkspaceUpdatePhase::Watching | WorkspaceUpdatePhase::Failed
            )
    })
    .await
}

async fn build_equivalent_ordered_end_state(
    repository: &GitWorkspace,
    reverse_order: bool,
) -> (Arc<WorkspaceSnapshot>, Vec<RepositoryChange>) {
    let workspace = WorkspaceService::new();
    let input = workspace.change_input_handle();
    let observer = workspace.snapshot_observer();
    let mut snapshots = observer.subscribe();
    let updates = workspace.update_observer();
    let mut update_changes = updates.subscribe();
    let app = configured_builder(&repository.root)
        .register_service("workspace", workspace)
        .expect("ordered Workspace service must register")
        .build()
        .expect("ordered application must build");
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    wait_for_update(&mut update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;
    wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldEdition", "WritesFixture"]
    })
    .await;

    let edt_configuration = repository
        .root
        .join("edt/src/Configuration/Configuration.mdo");
    if reverse_order {
        repository.git(["mv", "tracked-move.txt", "moved.txt"]);
        fs::remove_file(repository.root.join("tracked-remove.txt"))
            .expect("reverse-order tracked removal must succeed");
        fs::write(repository.root.join("added.txt"), "added\n")
            .expect("reverse-order addition must be written");
        repository.git(["add", "added.txt"]);
        replace_exact(
            &edt_configuration,
            "<name>WritesFixture</name>",
            "<name>WritesOrderEquivalent</name>",
        );
    } else {
        replace_exact(
            &edt_configuration,
            "<name>WritesFixture</name>",
            "<name>WritesOrderEquivalent</name>",
        );
        fs::write(repository.root.join("added.txt"), "added\n")
            .expect("forward-order addition must be written");
        repository.git(["add", "added.txt"]);
        fs::remove_file(repository.root.join("tracked-remove.txt"))
            .expect("forward-order tracked removal must succeed");
        repository.git(["mv", "tracked-move.txt", "moved.txt"]);
    }

    let change_set = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("equivalent ordered repository state must be readable");
    let normalized_changes = change_set.changes().to_vec();
    submit_after_current_update(&input, &mut update_changes, change_set).await;
    let snapshot = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldEdition", "WritesOrderEquivalent"]
    })
    .await;

    shutdown_sender
        .send(())
        .expect("ordered Workspace shutdown must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("ordered Workspace shutdown must not hang")
        .expect("ordered Workspace task must join")
        .expect("ordered requested shutdown must succeed");
    (snapshot, normalized_changes)
}

#[tokio::test]
async fn public_git_input_publishes_equal_complete_end_states_across_operation_orders() {
    let repository = GitWorkspace::new();
    let (forward, forward_changes) = build_equivalent_ordered_end_state(&repository, false).await;
    repository.git(["reset", "--hard", "HEAD"]);
    repository.git(["clean", "-fd"]);
    let cache_root = repository.root.join(".oneagent");
    if cache_root.exists() {
        fs::remove_dir_all(cache_root).expect("ordered test cache must reset with its fixture");
    }
    let (reverse, reverse_changes) = build_equivalent_ordered_end_state(&repository, true).await;

    assert_eq!(forward_changes, reverse_changes);
    assert_eq!(configuration_names(&forward), configuration_names(&reverse));
    assert_eq!(forward.len(), reverse.len());
    for (forward_configuration, reverse_configuration) in forward
        .configurations()
        .iter()
        .zip(reverse.configurations())
    {
        assert_eq!(
            forward_configuration
                .root_path()
                .strip_prefix(forward.root_path())
                .expect("forward project root must remain confined"),
            reverse_configuration
                .root_path()
                .strip_prefix(reverse.root_path())
                .expect("reverse project root must remain confined")
        );
        assert_eq!(
            forward_configuration.format(),
            reverse_configuration.format()
        );
        assert_eq!(
            forward_configuration.configuration_id(),
            reverse_configuration.configuration_id()
        );
        assert_eq!(
            forward_configuration.configuration_name(),
            reverse_configuration.configuration_name()
        );
        assert!(
            SemanticGraphDiff::between(
                forward_configuration.graph(),
                reverse_configuration.graph(),
            )
            .is_empty()
        );
        assert_eq!(
            forward_configuration.diagnostics(),
            reverse_configuration.diagnostics()
        );
        assert_eq!(
            forward_configuration.reference_requests(),
            reverse_configuration.reference_requests()
        );
        assert_eq!(
            forward_configuration.reference_statistics(),
            reverse_configuration.reference_statistics()
        );
        assert_eq!(
            forward_configuration.report(),
            reverse_configuration.report()
        );
        assert_eq!(
            forward_configuration.validation(),
            reverse_configuration.validation()
        );
        assert_eq!(
            forward_configuration.rule_execution_report(),
            reverse_configuration.rule_execution_report()
        );
        assert_eq!(
            forward_configuration.diagnostic_report(),
            reverse_configuration.diagnostic_report()
        );
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn public_git_input_drives_complete_atomic_rebuild_failure_recovery_and_graph_queries() {
    let repository = GitWorkspace::new();
    let reader = GitRepositoryReader::new();
    let empty = reader
        .read(&repository.root)
        .await
        .expect("clean fixture repository must be readable");
    assert!(empty.is_empty());

    let workspace = WorkspaceService::new();
    let input = workspace.change_input_handle();
    let observer = workspace.snapshot_observer();
    let mut snapshots = observer.subscribe();
    let updates = workspace.update_observer();
    let mut update_changes = updates.subscribe();
    let cache = workspace.cache_observer();
    let graph_query = GraphQueryService::new(observer.clone());
    let app = configured_builder(&repository.root)
        .register_service("workspace", workspace)
        .expect("Workspace service must register")
        .build()
        .expect("application must build");
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let initial_status = wait_for_update(&mut update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;
    let initial = wait_for_snapshot(&mut snapshots, |snapshot| snapshot.len() == 2).await;
    assert_eq!(
        configuration_names(&initial),
        ["DNSWorldEdition", "WritesFixture"]
    );
    assert_eq!(
        input.submit(empty),
        WorkspaceChangeSubmissionOutcome::IgnoredEmpty
    );
    assert_eq!(updates.status(), initial_status);

    let edt_configuration = repository
        .root
        .join("edt/src/Configuration/Configuration.mdo");
    replace_exact(
        &edt_configuration,
        "<name>WritesFixture</name>",
        "<name>WritesGitInput</name>",
    );
    let modified = reader
        .read(&repository.root)
        .await
        .expect("modified repository must be readable");
    assert!(modified.changes().iter().any(|change| {
        change.kind() == RepositoryChangeKind::Modified
            && change.effective_path().as_str() == "edt/src/Configuration/Configuration.mdo"
    }));
    let modified_status = submit_after_current_update(&input, &mut update_changes, modified).await;
    let modified_snapshot = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldEdition", "WritesGitInput"]
    })
    .await;
    assert_eq!(
        configuration_names(&initial),
        ["DNSWorldEdition", "WritesFixture"]
    );
    assert_eq!(
        configuration_names(&modified_snapshot),
        ["DNSWorldEdition", "WritesGitInput"]
    );

    fs::write(repository.root.join("added.txt"), "added\n").expect("added fixture must be written");
    repository.git(["add", "added.txt"]);
    fs::remove_file(repository.root.join("tracked-remove.txt"))
        .expect("tracked removal fixture must be removed");
    repository.git(["mv", "tracked-move.txt", "moved.txt"]);
    let matrix = reader
        .read(&repository.root)
        .await
        .expect("repository operation matrix must be readable");
    let matrix_kinds = matrix
        .changes()
        .iter()
        .map(|change| (change.effective_path().as_str(), change.kind()))
        .collect::<Vec<_>>();
    assert!(matrix_kinds.contains(&("added.txt", RepositoryChangeKind::Added)));
    assert!(matrix_kinds.contains(&("moved.txt", RepositoryChangeKind::Added)));
    assert!(matrix_kinds.contains(&("tracked-move.txt", RepositoryChangeKind::Deleted)));
    assert!(matrix_kinds.contains(&("tracked-remove.txt", RepositoryChangeKind::Deleted)));
    let matrix_status = submit_after_current_update(&input, &mut update_changes, matrix).await;
    assert!(matrix_status.attempt() > modified_status.attempt());
    let equal_snapshot = observer
        .snapshot()
        .expect("complete snapshot must remain published");
    assert_eq!(
        configuration_names(&equal_snapshot),
        ["DNSWorldEdition", "WritesGitInput"]
    );

    fs::write(&edt_configuration, "<broken>").expect("invalid source must be written");
    let invalid = reader
        .read(&repository.root)
        .await
        .expect("invalid semantic source remains valid Git evidence");
    submit_after_current_update(&input, &mut update_changes, invalid).await;
    let failed = wait_for_update(&mut update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Failed
            && status.failure() == Some(WorkspaceUpdateFailureKind::SemanticBuild)
    })
    .await;
    assert_eq!(failed.published(), matrix_status.published());
    assert_eq!(
        configuration_names(
            &observer
                .snapshot()
                .expect("last valid snapshot must be retained")
        ),
        ["DNSWorldEdition", "WritesGitInput"]
    );

    let repaired =
        fs::read_to_string(fixture_root().join("edt/src/Configuration/Configuration.mdo"))
            .expect("tracked repair source must be readable")
            .replace("<name>WritesFixture</name>", "<name>WritesRecovered</name>");
    fs::write(&edt_configuration, repaired).expect("repaired source must be written");
    let recovery = reader
        .read(&repository.root)
        .await
        .expect("repaired repository must be readable");
    submit_after_current_update(&input, &mut update_changes, recovery).await;
    let recovered = wait_for_snapshot(&mut snapshots, |snapshot| {
        configuration_names(snapshot) == ["DNSWorldEdition", "WritesRecovered"]
    })
    .await;
    assert_eq!(
        graph_query
            .configurations(GraphQueryLimit::default())
            .expect("published graph configurations must be queryable")
            .configurations()
            .iter()
            .map(oneagent_runtime::GraphQueryConfiguration::name)
            .collect::<Vec<_>>(),
        ["DNSWorldEdition", "WritesRecovered"]
    );
    assert_eq!(
        configuration_names(&recovered),
        ["DNSWorldEdition", "WritesRecovered"]
    );
    assert_eq!(
        cache.status().write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );

    shutdown_sender.send(()).expect("shutdown must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Workspace shutdown must not hang")
        .expect("Workspace task must join")
        .expect("requested shutdown must succeed");
    assert!(observer.snapshot().is_none());
    assert_eq!(updates.status().phase(), WorkspaceUpdatePhase::Stopped);
    assert_eq!(
        input.submit(
            reader
                .read(&repository.root)
                .await
                .expect("repository remains readable after shutdown")
        ),
        WorkspaceChangeSubmissionOutcome::Closed
    );
}

#[tokio::test]
async fn public_git_input_preserves_cold_warm_cache_and_fresh_service_ownership() {
    let repository = GitWorkspace::new();

    let first = WorkspaceService::new();
    let first_cache = first.cache_observer();
    let first_input = first.change_input_handle();
    let first_updates = first.update_observer();
    let mut first_update_changes = first_updates.subscribe();
    let first_app = configured_builder(&repository.root)
        .register_service("workspace", first)
        .expect("first Workspace service must register")
        .build()
        .expect("first application must build");
    let (first_shutdown_sender, first_shutdown) = oneshot::channel::<()>();
    let first_run = tokio::spawn(first_app.run(first_shutdown));
    wait_for_update(&mut first_update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;
    assert_eq!(
        first_cache.status().load(),
        WorkspaceCacheLoadOutcome::Missing
    );
    assert_eq!(
        first_cache.status().write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );
    first_shutdown_sender
        .send(())
        .expect("first shutdown must be observed");
    timeout(TEST_TIMEOUT, first_run)
        .await
        .expect("first shutdown must not hang")
        .expect("first Runtime task must join")
        .expect("first requested shutdown must succeed");
    let clean = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("warm repository must be readable");
    assert_eq!(
        first_input.submit(clean),
        WorkspaceChangeSubmissionOutcome::IgnoredEmpty
    );

    let second = WorkspaceService::new();
    let second_cache = second.cache_observer();
    let second_input = second.change_input_handle();
    let second_updates = second.update_observer();
    let mut second_update_changes = second_updates.subscribe();
    let second_app = configured_builder(&repository.root)
        .register_service("workspace", second)
        .expect("second Workspace service must register")
        .build()
        .expect("second application must build");
    let (second_shutdown_sender, second_shutdown) = oneshot::channel::<()>();
    let second_run = tokio::spawn(second_app.run(second_shutdown));
    wait_for_update(&mut second_update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching
    })
    .await;
    assert_eq!(second_cache.status().load(), WorkspaceCacheLoadOutcome::Hit);

    fs::write(repository.root.join("untracked.txt"), "input\n")
        .expect("untracked input fixture must be written");
    let change_set = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("changed warm repository must be readable");
    let rebuilt =
        submit_after_current_update(&second_input, &mut second_update_changes, change_set).await;
    wait_for_update(&mut second_update_changes, |status| {
        status.phase() == WorkspaceUpdatePhase::Watching && status.attempt() >= rebuilt.attempt()
    })
    .await;
    assert_eq!(
        second_cache.status().write(),
        WorkspaceCacheWriteOutcome::Succeeded
    );

    second_shutdown_sender
        .send(())
        .expect("second shutdown must be observed");
    timeout(TEST_TIMEOUT, second_run)
        .await
        .expect("second shutdown must not hang")
        .expect("second Runtime task must join")
        .expect("second requested shutdown must succeed");
    assert_eq!(
        second_input.submit(
            GitRepositoryReader::new()
                .read(&repository.root)
                .await
                .expect("repository remains readable after repeated shutdown")
        ),
        WorkspaceChangeSubmissionOutcome::Closed
    );
}
