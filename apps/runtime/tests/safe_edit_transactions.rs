use oneagent_analysis::refactoring::{RefactoringFamily, RefactoringRequest, SourceOccurrenceKind};
use oneagent_runtime::{
    App, BoxError, ConfigurationProvider, RuntimeConfig, WorkspaceEditCancellation,
    WorkspaceEditCause, WorkspaceEditHandle, WorkspaceEditOutcome, WorkspaceEditOwnership,
    WorkspaceEditRecovery, WorkspaceService, WorkspaceSnapshot, WorkspaceSnapshotBuilder,
};
use oneagent_tool_policy::{
    ActorId, ActorScope, PolicyRevision, RuleAction, ToolEffect, ToolPolicy, ToolRequestId,
    ToolRule, ToolScope,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

pub(crate) fn fixture(format: &str) -> tempfile::TempDir {
    let root = repository();
    let temporary_parent = root.join("local-artifacts/codex-runs/sprint-41/task-5/tmp");
    fs::create_dir_all(&temporary_parent).unwrap();
    let temporary = tempfile::tempdir_in(temporary_parent).unwrap();
    copy_tree(
        &root
            .join("adapters/designer-xml/tests/fixtures/sprint14_conformance")
            .join(format),
        temporary.path(),
    );
    let runtime = root
        .join("apps/runtime/tests/fixtures/workspace_service")
        .join(format);
    if format == "designer" {
        for name in ["Configuration.xml", "ConfigDumpInfo.xml"] {
            fs::copy(runtime.join(name), temporary.path().join(name)).unwrap();
        }
    } else {
        let configuration = fs::read_to_string(runtime.join("src/Configuration/Configuration.mdo"))
            .unwrap()
            .replace(
                "50000000-0000-0000-0000-000000000000",
                "408a41e7-907a-4fb3-8999-83d1e8b6e093",
            )
            .replace("WritesFixture", "DNSWorldEdition");
        fs::write(
            temporary.path().join("src/Configuration/Configuration.mdo"),
            configuration,
        )
        .unwrap();
    }
    let (descriptor, source, second_descriptor, second_source) = if format == "designer" {
        (
            "CommonModules/DynamicSecurityOverridable.xml",
            "CommonModules/DynamicSecurityOverridable/Ext/Module.bsl",
            "CommonModules/SecondaryCaller.xml",
            "CommonModules/SecondaryCaller/Ext/Module.bsl",
        )
    } else {
        (
            "src/CommonModules/DynamicSecurityOverridable/DynamicSecurityOverridable.mdo",
            "src/CommonModules/DynamicSecurityOverridable/Module.bsl",
            "src/CommonModules/SecondaryCaller/SecondaryCaller.mdo",
            "src/CommonModules/SecondaryCaller/Module.bsl",
        )
    };
    let descriptor = fs::read_to_string(temporary.path().join(descriptor))
        .unwrap()
        .replace("DynamicSecurityOverridable", "SecondaryCaller")
        .replace(
            "dc24575c-a787-411d-93bd-494271291d73",
            "dc24575c-a787-411d-93bd-494271291d74",
        );
    fs::create_dir_all(temporary.path().join(second_descriptor).parent().unwrap()).unwrap();
    fs::write(temporary.path().join(second_descriptor), descriptor).unwrap();
    let original = fs::read_to_string(temporary.path().join(source)).unwrap();
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let declaration = "Procedure ExerciseSecurityCollection(SecurityCollection)";
    let caller_start = original.find(declaration).unwrap();
    let caller_end = original[caller_start..].find("EndProcedure").unwrap()
        + caller_start
        + "EndProcedure".len();
    let caller = original[caller_start..caller_end].replace(
        "    FillSecurityCollection(SecurityCollection);",
        "    DynamicSecurityOverridable.FillSecurityCollection(SecurityCollection);",
    );
    fs::create_dir_all(temporary.path().join(second_source).parent().unwrap()).unwrap();
    let bom = if original.starts_with('\u{feff}') {
        "\u{feff}"
    } else {
        ""
    };
    fs::write(
        temporary.path().join(second_source),
        format!("{bom}{caller}{newline}"),
    )
    .unwrap();
    // Derive an internal call from the tracked caller rather than inventing grammar.
    let derived = original.replacen(&format!("Export{newline}{newline}EndProcedure"), &format!("Export{newline}{newline}    ExerciseSecurityCollection(SecurityCollection);{newline}{newline}EndProcedure"), 1);
    fs::write(temporary.path().join(source), derived).unwrap();
    temporary
}

pub(crate) fn policy(action: RuleAction) -> ToolPolicy {
    ToolPolicy::new(
        PolicyRevision::new("edit-test-1").unwrap(),
        vec![ToolRule::new(
            ActorScope::Any,
            ToolScope::Any,
            ToolEffect::LocalMutation,
            action,
        )],
    )
    .unwrap()
}

pub(crate) fn add_query_fixture(root: &Path, format: &str, disposition: &str) {
    let path = if format == "edt" {
        "src/CommonModules/DynamicSecurityOverridable/Module.bsl"
    } else {
        "CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"
    };
    let source = fs::read_to_string(root.join(path)).unwrap();
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let tracked = fs::read_to_string(repository().join(
        "adapters/edt/tests/fixtures/reads_project/src/Documents/QueryHost/ObjectModule.bsl",
    ))
    .unwrap();
    let start = tracked.find("    Query = New Query;").unwrap();
    let end = tracked[start..].find("EndProcedure").unwrap() + start;
    let mut body = tracked[start..end].to_owned();
    if disposition == "resolved" {
        if format == "edt" {
            copy_tree(
                &repository()
                    .join("adapters/edt/tests/fixtures/reads_project/src/Catalogs/Products"),
                &root.join("src/Catalogs/Products"),
            );
        }
    } else if disposition == "unsupported" {
        let text = fs::read_to_string(
            repository().join("crates/bsl/tests/fixtures/query_language/unsupported_join_en.query"),
        )
        .unwrap();
        let literal = text.trim().replace('"', "\"\"").replace('\n', "\n    |");
        body = body.replace("SELECT Ref FROM Catalog.Products", &literal);
    } else if disposition == "malformed" {
        body = body.replace("SELECT Ref FROM Catalog.Products", "SELECT Ref FROM");
    }
    // Preserve source encoding; another callable deliberately owns the same binding.
    let body = body.replace('\n', newline);
    let source = source.replacen("EndProcedure", &format!("{body}EndProcedure"), 1);
    let source = source.replacen(
        "Procedure ExerciseSecurityCollection(SecurityCollection)",
        &format!("Procedure ExerciseSecurityCollection(SecurityCollection){newline}{body}"),
        1,
    );
    fs::write(root.join(path), source).unwrap();
}
pub(crate) fn actor() -> ActorId {
    ActorId::new("fixture-owner").unwrap()
}
pub(crate) fn request_id() -> ToolRequestId {
    ToolRequestId::new("fixture-edit").unwrap()
}
pub(crate) fn request(snapshot: &WorkspaceSnapshot, desired: &str) -> RefactoringRequest {
    let configuration = &snapshot.configurations()[0];
    let target = configuration
        .source_evidence()
        .documents()
        .iter()
        .flat_map(oneagent_analysis::refactoring::SourceDocument::occurrences)
        .find(|o| {
            o.kind() == SourceOccurrenceKind::Declaration && o.token() == "FillSecurityCollection"
        })
        .unwrap();
    RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        snapshot.publication_id(),
        configuration.configuration_id().clone(),
        target.mapped_target_id().unwrap().clone(),
        desired,
    )
    .unwrap()
}

pub(crate) struct Provider(pub(crate) PathBuf);
impl ConfigurationProvider for Provider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("safe-edit-test", "test").with_workspace_root(self.0.clone()))
    }
}
async fn start(
    root: &Path,
    enabled: bool,
) -> (
    WorkspaceEditHandle,
    oneagent_runtime::WorkspaceSnapshotObserver,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<Result<(), oneagent_runtime::RuntimeError>>,
) {
    let service = if enabled {
        WorkspaceService::new().with_edit_policy(
            policy(RuleAction::RequireConfirmation),
            WorkspaceEditOwnership::ExclusiveCooperative,
        )
    } else {
        WorkspaceService::new()
    };
    start_service(root, service).await
}

pub(crate) async fn start_service<
    D: oneagent_workspace::WorkspaceDetector + Clone + Send + 'static,
>(
    root: &Path,
    service: WorkspaceService<D>,
) -> (
    WorkspaceEditHandle,
    oneagent_runtime::WorkspaceSnapshotObserver,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<Result<(), oneagent_runtime::RuntimeError>>,
) {
    let handle = service.edit_handle();
    let observer = service.snapshot_observer();
    let mut changes = observer.subscribe();
    let app = App::builder()
        .configure(&Provider(root.to_owned()))
        .unwrap()
        .register_service("workspace", service)
        .unwrap()
        .build()
        .unwrap();
    let (stop, signal) = oneshot::channel();
    let task = tokio::spawn(app.run(signal));
    tokio::time::timeout(Duration::from_secs(30), async {
        while changes.borrow().is_none() {
            changes.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    (handle, observer, stop, task)
}

#[tokio::test]
async fn apply_reversal_preserve_exact_bytes_and_old_arcs() {
    for format in ["edt", "designer"] {
        for desired in ["РасширенноеИмя", "Я"] {
            let root = fixture(format);
            if desired == "РасширенноеИмя" {
                let paths: Vec<_> = fs::read_dir(root.path())
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .collect();
                let custom = root.path().join("custom-configuration-root");
                fs::create_dir(&custom).unwrap();
                for path in paths {
                    fs::rename(&path, custom.join(path.file_name().unwrap())).unwrap();
                }
            }
            let built = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
            assert_eq!(built.len(), 1);
            assert_eq!(
                built.configurations()[0]
                    .source_evidence()
                    .documents()
                    .len(),
                2
            );
            let (handle, observer, stop, task) = start(root.path(), true).await;
            let old = observer.snapshot().unwrap();
            let original_sources = old.configurations()[0].source_evidence().clone();
            let (challenge, preview) = handle
                .prepare_apply(request(&old, desired), actor(), request_id())
                .await
                .unwrap();
            assert!(format!("{preview:?}").contains("RefactoringPreview"));
            let applied = handle
                .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                .await;
            let WorkspaceEditOutcome::Applied {
                current,
                previous,
                reversal: Some(receipt),
                files,
                ..
            } = applied
            else {
                panic!("apply failed: {applied:?}")
            };
            assert_eq!(previous, old.publication_id());
            assert_eq!(current.get(), previous.get() + 1);
            assert_eq!(files, 2);
            let applied_snapshot = observer.snapshot().unwrap();
            assert!(!Arc::ptr_eq(&old, &applied_snapshot));
            assert_eq!(old.configurations()[0].source_evidence(), &original_sources);
            let (challenge, _) = handle
                .prepare_reversal(receipt, actor(), request_id())
                .await
                .unwrap();
            let reversed = handle
                .checked_reversal(challenge.confirm(), WorkspaceEditCancellation::new())
                .await;
            assert!(
                matches!(
                    reversed,
                    WorkspaceEditOutcome::Applied { reversal: None, .. }
                ),
                "{reversed:?}"
            );
            let restored = observer.snapshot().unwrap();
            assert_eq!(restored.publication_id().get(), current.get() + 1);
            assert_eq!(
                restored.configurations()[0].source_evidence(),
                &original_sources
            );
            for document in original_sources.documents() {
                assert_eq!(
                    fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                    document.raw_content()
                );
            }
            stop.send(()).unwrap();
            tokio::time::timeout(Duration::from_secs(30), task)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
        }
    }
}

#[tokio::test]
async fn disabled_unready_stopped_and_foreign_services_reject() {
    let root = fixture("edt");
    let snapshot = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
    let unready = WorkspaceService::new()
        .with_edit_policy(
            policy(RuleAction::RequireConfirmation),
            WorkspaceEditOwnership::ExclusiveCooperative,
        )
        .edit_handle();
    assert_eq!(
        unready
            .prepare_apply(request(&snapshot, "Changed"), actor(), request_id())
            .await
            .unwrap_err(),
        WorkspaceEditCause::Unavailable
    );
    let (disabled, _, stop, task) = start(root.path(), false).await;
    assert_eq!(
        disabled
            .prepare_apply(request(&snapshot, "Changed"), actor(), request_id())
            .await
            .unwrap_err(),
        WorkspaceEditCause::Unavailable
    );
    stop.send(()).unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(
        disabled
            .prepare_apply(request(&snapshot, "Changed"), actor(), request_id())
            .await
            .unwrap_err(),
        WorkspaceEditCause::Stopped
    );
    let other_root = fixture("edt");
    let (first, first_observer, first_stop, first_task) = start(root.path(), true).await;
    let (other, other_observer, other_stop, other_task) = start(other_root.path(), true).await;
    let before = first_observer.snapshot().unwrap();
    let other_before = other_observer.snapshot().unwrap();
    assert_eq!(before.publication_id(), other_before.publication_id());
    let (challenge, _) = first
        .prepare_apply(request(&before, "Changed"), actor(), request_id())
        .await
        .unwrap();
    assert!(matches!(
        other
            .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
            .await,
        WorkspaceEditOutcome::Failed {
            cause: WorkspaceEditCause::AuthorizationMismatch,
            recovery: WorkspaceEditRecovery::NotNeeded,
            ..
        }
    ));
    for (root, before, observer) in [
        (root.path(), &before, &first_observer),
        (other_root.path(), &other_before, &other_observer),
    ] {
        assert!(Arc::ptr_eq(before, &observer.snapshot().unwrap()));
        for doc in before.configurations()[0].source_evidence().documents() {
            assert_eq!(
                fs::read(root.join(doc.path().path().as_str())).unwrap(),
                doc.raw_content()
            );
        }
    }
    first_stop.send(()).unwrap();
    other_stop.send(()).unwrap();
    first_task.await.unwrap().unwrap();
    other_task.await.unwrap().unwrap();
}

#[tokio::test]
async fn query_containing_targets_preserve_complete_nested_evidence() {
    for format in ["edt", "designer"] {
        for disposition in ["resolved", "missing", "unsupported", "malformed"] {
            for desired in ["РасширенноеИмя", "Я"] {
                let root = fixture(format);
                add_query_fixture(root.path(), format, disposition);
                let (handle, observer, stop, task) = start(root.path(), true).await;
                let before = observer.snapshot().unwrap();
                let (challenge, _) = handle
                    .prepare_apply(request(&before, desired), actor(), request_id())
                    .await
                    .unwrap();
                let outcome = handle
                    .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await;
                let WorkspaceEditOutcome::Applied {
                    reversal: Some(receipt),
                    ..
                } = outcome
                else {
                    panic!("{format}/{disposition}/{desired}: {outcome:?}");
                };
                let after = observer.snapshot().unwrap();
                if format == "edt" {
                    assert!(
                        before.configurations()[0]
                            .graph()
                            .nodes()
                            .any(|node| node.kind() == oneagent_graph::NodeKind::Query)
                    );
                    assert_eq!(
                        before.configurations()[0].reference_statistics(),
                        after.configurations()[0].reference_statistics()
                    );
                }
                let (challenge, _) = handle
                    .prepare_reversal(receipt, actor(), request_id())
                    .await
                    .unwrap();
                let outcome = handle
                    .checked_reversal(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await;
                assert!(
                    matches!(
                        outcome,
                        WorkspaceEditOutcome::Applied { reversal: None, .. }
                    ),
                    "{format}/{disposition}: {outcome:?}"
                );
                assert_eq!(
                    before.configurations()[0].source_evidence(),
                    observer.snapshot().unwrap().configurations()[0].source_evidence()
                );
                stop.send(()).unwrap();
                task.await.unwrap().unwrap();
            }
        }
    }
}

#[tokio::test]
async fn complete_baseline_staleness_rejects() {
    for changed in ["source", "metadata", "unknown"] {
        let root = fixture("edt");
        let (handle, observer, stop, task) = start(root.path(), true).await;
        let before = observer.snapshot().unwrap();
        let (challenge, _) = handle
            .prepare_apply(request(&before, "Changed"), actor(), request_id())
            .await
            .unwrap();
        let path = match changed {
            "source" => root
                .path()
                .join("src/CommonModules/SecondaryCaller/Module.bsl"),
            "metadata" => root.path().join("src/Configuration/Configuration.mdo"),
            _ => root.path().join(".ignored-unknown"),
        };
        let mut bytes = fs::read(&path).unwrap_or_default();
        bytes.extend_from_slice(b"\n ");
        fs::write(&path, &bytes).unwrap();
        let outcome = handle
            .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
            .await;
        assert!(
            matches!(
                outcome,
                WorkspaceEditOutcome::Failed {
                    cause: WorkspaceEditCause::SourceChanged,
                    ..
                }
            ),
            "{changed}: {outcome:?}"
        );
        assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
        assert_eq!(fs::read(path).unwrap(), bytes);
        stop.send(()).unwrap();
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn stale_reversal_and_reconfirmation_reject() {
    let root = fixture("edt");
    let other_root = fixture("edt");
    let (handle, observer, stop, task) = start(root.path(), true).await;
    let (other, other_observer, other_stop, other_task) = start(other_root.path(), true).await;
    let before = observer.snapshot().unwrap();
    let other_before = other_observer.snapshot().unwrap();
    assert_eq!(before.publication_id(), other_before.publication_id());
    let (challenge, _) = handle
        .prepare_apply(request(&before, "Changed"), actor(), request_id())
        .await
        .unwrap();
    let outcome = other
        .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
        .await;
    assert!(
        matches!(
            outcome,
            WorkspaceEditOutcome::Failed {
                cause: WorkspaceEditCause::AuthorizationMismatch,
                ..
            }
        ),
        "{outcome:?}"
    );
    assert!(Arc::ptr_eq(
        &other_before,
        &other_observer.snapshot().unwrap()
    ));
    let (challenge, _) = handle
        .prepare_apply(request(&before, "Changed"), actor(), request_id())
        .await
        .unwrap();
    let outcome = handle
        .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
        .await;
    let WorkspaceEditOutcome::Applied {
        reversal: Some(receipt),
        ..
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        other
            .prepare_reversal(receipt, actor(), request_id())
            .await
            .unwrap_err(),
        WorkspaceEditCause::AuthorizationMismatch
    );
    stop.send(()).unwrap();
    other_stop.send(()).unwrap();
    task.await.unwrap().unwrap();
    other_task.await.unwrap().unwrap();
    for kind in ["denied", "bytes", "intervening_apply", "fresh_confirmation"] {
        let root = fixture("edt");
        let edit_policy = if kind == "denied" {
            ToolPolicy::new(
                PolicyRevision::new("reverse-denied").unwrap(),
                vec![
                    ToolRule::new(
                        ActorScope::Any,
                        ToolScope::Exact(
                            oneagent_tool_policy::ToolId::new("oneagent.workspace.edit.apply")
                                .unwrap(),
                        ),
                        ToolEffect::LocalMutation,
                        RuleAction::RequireConfirmation,
                    ),
                    ToolRule::new(
                        ActorScope::Any,
                        ToolScope::Exact(
                            oneagent_tool_policy::ToolId::new("oneagent.workspace.edit.reverse")
                                .unwrap(),
                        ),
                        ToolEffect::LocalMutation,
                        RuleAction::Deny,
                    ),
                ],
            )
            .unwrap()
        } else {
            policy(RuleAction::RequireConfirmation)
        };
        let service = WorkspaceService::new()
            .with_edit_policy(edit_policy, WorkspaceEditOwnership::ExclusiveCooperative);
        let (handle, observer, stop, task) = start_service(root.path(), service).await;
        let before = observer.snapshot().unwrap();
        let (challenge, _) = handle
            .prepare_apply(request(&before, "Changed"), actor(), request_id())
            .await
            .unwrap();
        let WorkspaceEditOutcome::Applied {
            reversal: Some(receipt),
            ..
        } = handle
            .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
            .await
        else {
            panic!("apply prerequisite");
        };
        let applied = observer.snapshot().unwrap();
        if kind == "bytes" {
            let doc = &applied.configurations()[0].source_evidence().documents()[0];
            let mut bytes = doc.raw_content().to_vec();
            bytes.extend_from_slice(b"\n// external\n");
            fs::write(root.path().join(doc.path().path().as_str()), bytes).unwrap();
        }
        if kind == "intervening_apply" {
            let configuration = &applied.configurations()[0];
            let node = configuration
                .graph()
                .nodes()
                .find(|node| node.name().as_str() == "Changed")
                .unwrap();
            let request = RefactoringRequest::new(
                RefactoringFamily::BslCallableRenameV1,
                applied.publication_id(),
                configuration.configuration_id().clone(),
                node.id().clone(),
                "Again",
            )
            .unwrap();
            let (challenge, _) = handle
                .prepare_apply(request, actor(), request_id())
                .await
                .unwrap();
            assert!(matches!(
                handle
                    .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await,
                WorkspaceEditOutcome::Applied { .. }
            ));
        }
        let current = observer.snapshot().unwrap();
        let prepared = handle
            .prepare_reversal(receipt, actor(), request_id())
            .await;
        if kind == "fresh_confirmation" {
            let (challenge, _) = prepared.unwrap();
            assert!(
                Arc::ptr_eq(&applied, &observer.snapshot().unwrap()),
                "preparation has no mutation authority"
            );
            assert!(matches!(
                handle
                    .checked_reversal(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await,
                WorkspaceEditOutcome::Applied { reversal: None, .. }
            ));
            assert_eq!(
                observer.snapshot().unwrap().configurations()[0].source_evidence(),
                before.configurations()[0].source_evidence()
            );
        } else {
            let cause = prepared.unwrap_err();
            if kind == "denied" {
                assert_eq!(cause, WorkspaceEditCause::PolicyDenied);
            }
            if kind != "bytes" {
                assert!(Arc::ptr_eq(&current, &observer.snapshot().unwrap()));
            }
        }
        stop.send(()).unwrap();
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn exact_one_mib_document_remains_eligible() {
    let root = fixture("edt");
    let path = root
        .path()
        .join("src/CommonModules/DynamicSecurityOverridable/Module.bsl");
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"\n//");
    bytes.resize(1_048_576, b' ');
    fs::write(&path, &bytes).unwrap();
    let (handle, observer, stop, task) = start(root.path(), true).await;
    let before = observer.snapshot().unwrap();
    let (challenge, _) = handle
        .prepare_apply(
            request(&before, "KeepSecurityCollection"),
            actor(),
            request_id(),
        )
        .await
        .unwrap();
    let outcome = handle
        .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
        .await;
    assert!(
        matches!(outcome, WorkspaceEditOutcome::Applied { .. }),
        "{outcome:?}"
    );
    assert_eq!(fs::metadata(path).unwrap().len(), 1_048_576);
    stop.send(()).unwrap();
    task.await.unwrap().unwrap();
}
