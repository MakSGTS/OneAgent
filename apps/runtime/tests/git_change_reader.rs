use oneagent_runtime::{GitRepositoryReadErrorKind, GitRepositoryReader, RepositoryChangeKind};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Repository {
    _temp: TempDir,
    root: PathBuf,
}

impl Repository {
    fn new(files: &[(&str, &str)]) -> Self {
        let temp = tempfile::tempdir().expect("temporary repository owner must be created");
        let root = temp.path().join("repository");
        fs::create_dir(&root).expect("repository root must be created");
        git_at(temp.path(), ["init", "repository"]);
        let repository = Self { _temp: temp, root };
        repository.configure();
        repository.write(".gitignore", "ignored.txt\n");
        for (path, content) in files {
            repository.write(path, content);
        }
        repository.git(["add", "--all"]);
        repository.commit("initial");
        repository
    }

    fn empty_unborn() -> Self {
        let temp = tempfile::tempdir().expect("temporary repository owner must be created");
        let root = temp.path().join("repository");
        fs::create_dir(&root).expect("repository root must be created");
        git_at(temp.path(), ["init", "repository"]);
        let repository = Self { _temp: temp, root };
        repository.configure();
        repository
    }

    fn configure(&self) {
        self.git(["config", "user.name", "OneAgent Test"]);
        self.git(["config", "user.email", "oneagent@example.invalid"]);
        self.git(["config", "commit.gpgsign", "false"]);
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent must be created");
        }
        fs::write(path, content).expect("fixture file must be written");
    }

    fn remove(&self, relative: &str) {
        fs::remove_file(self.root.join(relative)).expect("fixture file must be removed");
    }

    fn commit(&self, message: &str) {
        self.git(["commit", "-m", message]);
    }

    fn git<I, S>(&self, arguments: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        git_at(&self.root, arguments)
    }

    fn head(&self) -> String {
        let output = self.git(["rev-parse", "HEAD"]);
        String::from_utf8(output.stdout)
            .expect("HEAD output must be UTF-8")
            .trim()
            .to_owned()
    }
}

fn git_at<I, S>(directory: &Path, arguments: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let isolated_global = directory.join("missing-global-git-config");
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", isolated_global)
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

fn paths_and_kinds(set: &oneagent_runtime::GitChangeSet) -> Vec<(&str, RepositoryChangeKind)> {
    set.changes()
        .iter()
        .map(|change| (change.effective_path().as_str(), change.kind()))
        .collect()
}

#[tokio::test]
async fn public_reader_maps_final_tracked_untracked_ignored_and_move_states() {
    let repository = Repository::new(&[
        ("modified.txt", "before\n"),
        ("combined.txt", "before\n"),
        ("deleted.txt", "before\n"),
        ("old.txt", "before\n"),
        ("copy-source.txt", "copy\n"),
        ("cancelled.txt", "before\n"),
    ]);
    let baseline = repository.head();

    repository.write("modified.txt", "after\n");
    repository.write("combined.txt", "staged\n");
    repository.git(["add", "combined.txt"]);
    repository.write("combined.txt", "final\n");
    repository.remove("deleted.txt");
    repository.git(["mv", "old.txt", "moved.txt"]);
    repository.write("added.txt", "added\n");
    repository.git(["add", "added.txt"]);
    fs::copy(
        repository.root.join("copy-source.txt"),
        repository.root.join("copied.txt"),
    )
    .expect("copy candidate must be created");
    repository.git(["add", "copied.txt"]);
    repository.write("cancelled.txt", "staged\n");
    repository.git(["add", "cancelled.txt"]);
    repository.write("cancelled.txt", "before\n");
    repository.write("untracked.txt", "untracked\n");
    repository.write("ignored.txt", "ignored\n");

    let reader = GitRepositoryReader::new();
    let first = reader
        .read(&repository.root)
        .await
        .expect("accepted repository state must be read");
    let second = reader
        .read(&repository.root)
        .await
        .expect("repeated repository state must be read");
    assert_eq!(first, second);
    assert_eq!(first.baseline().as_str(), baseline);
    assert_eq!(
        paths_and_kinds(&first),
        vec![
            ("added.txt", RepositoryChangeKind::Added),
            ("combined.txt", RepositoryChangeKind::Modified),
            ("copied.txt", RepositoryChangeKind::Added),
            ("deleted.txt", RepositoryChangeKind::Deleted),
            ("modified.txt", RepositoryChangeKind::Modified),
            ("moved.txt", RepositoryChangeKind::Added),
            ("old.txt", RepositoryChangeKind::Deleted),
            ("untracked.txt", RepositoryChangeKind::Untracked),
        ]
    );
}

#[tokio::test]
async fn public_reader_is_operation_order_independent_for_equal_end_states() {
    let repository = Repository::new(&[("a.txt", "before\n"), ("b.txt", "before\n")]);
    repository.write("a.txt", "after\n");
    repository.remove("b.txt");
    repository.write("c.txt", "new\n");
    let first = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("first end state must be read");

    repository.git(["reset", "--hard", "HEAD"]);
    repository.git(["clean", "-f"]);
    repository.write("c.txt", "new\n");
    repository.remove("b.txt");
    repository.write("a.txt", "after\n");
    let second = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("equivalent reordered end state must be read");

    assert_eq!(first, second);
}

#[tokio::test]
async fn public_reader_accepts_detached_and_linked_exact_root_worktrees() {
    let repository = Repository::new(&[("tracked.txt", "content\n")]);
    let reader = GitRepositoryReader::new();
    let attached = reader
        .read(&repository.root)
        .await
        .expect("attached worktree must be read");
    assert!(attached.is_empty());

    repository.git(["checkout", "--detach"]);
    let detached = reader
        .read(&repository.root)
        .await
        .expect("detached worktree must be read");
    assert_eq!(attached, detached);

    let linked = repository
        .root
        .parent()
        .expect("repository has a temporary parent")
        .join("linked");
    repository.git([
        OsStr::new("worktree"),
        OsStr::new("add"),
        OsStr::new("--detach"),
        linked.as_os_str(),
        OsStr::new("HEAD"),
    ]);
    let linked = reader
        .read(&linked)
        .await
        .expect("linked exact-root worktree must be read");
    assert_eq!(attached, linked);
}

#[tokio::test]
async fn public_reader_classifies_missing_file_non_repository_unborn_bare_and_mismatch() {
    let temp = tempfile::tempdir().expect("temporary error roots must be created");
    let reader = GitRepositoryReader::new();
    let missing = reader
        .read(temp.path().join("missing"))
        .await
        .expect_err("missing root must fail");
    assert_eq!(missing.kind(), GitRepositoryReadErrorKind::RootUnavailable);

    let file = temp.path().join("file");
    fs::write(&file, "content").expect("file root must be written");
    let error = reader.read(&file).await.expect_err("file root must fail");
    assert_eq!(error.kind(), GitRepositoryReadErrorKind::RootNotDirectory);

    let plain = temp.path().join("plain");
    fs::create_dir(&plain).expect("plain directory must be created");
    let error = reader
        .read(&plain)
        .await
        .expect_err("non-repository root must fail");
    assert_eq!(error.kind(), GitRepositoryReadErrorKind::NotRepository);

    let unborn = Repository::empty_unborn();
    let error = reader
        .read(&unborn.root)
        .await
        .expect_err("unborn repository must fail");
    assert_eq!(error.kind(), GitRepositoryReadErrorKind::MissingBaseline);

    let bare = temp.path().join("bare.git");
    git_at(temp.path(), ["init", "--bare", "bare.git"]);
    let error = reader
        .read(&bare)
        .await
        .expect_err("bare repository must fail");
    assert_eq!(error.kind(), GitRepositoryReadErrorKind::BareRepository);

    let repository = Repository::new(&[("nested/kept.txt", "content\n")]);
    let error = reader
        .read(repository.root.join("nested"))
        .await
        .expect_err("subdirectory root must not expand to its parent worktree");
    assert_eq!(
        error.kind(),
        GitRepositoryReadErrorKind::WorktreeRootMismatch
    );

    let nested = repository.root.join("nested-repository");
    git_at(&repository.root, ["init", "nested-repository"]);
    git_at(&nested, ["config", "user.name", "OneAgent Test"]);
    git_at(
        &nested,
        ["config", "user.email", "oneagent@example.invalid"],
    );
    fs::write(nested.join("tracked.txt"), "nested\n")
        .expect("nested repository fixture must be written");
    git_at(&nested, ["add", "tracked.txt"]);
    git_at(&nested, ["commit", "-m", "nested"]);
    let set = reader
        .read(&nested)
        .await
        .expect("nested repository is accepted at its exact worktree root");
    assert!(set.is_empty());
}

#[tokio::test]
async fn public_reader_rejects_conflicts_and_changed_gitlinks() {
    let conflict = Repository::new(&[("conflict.txt", "base\n")]);
    conflict.git(["checkout", "-b", "feature"]);
    conflict.write("conflict.txt", "feature\n");
    conflict.git(["add", "conflict.txt"]);
    conflict.commit("feature");
    conflict.git(["checkout", "master"]);
    conflict.write("conflict.txt", "main\n");
    conflict.git(["add", "conflict.txt"]);
    conflict.commit("main");
    let merge = Command::new("git")
        .current_dir(&conflict.root)
        .args(["merge", "feature"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", conflict.root.join("missing-global"))
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("conflicting merge must start");
    assert!(!merge.status.success());
    let error = GitRepositoryReader::new()
        .read(&conflict.root)
        .await
        .expect_err("unmerged repository must fail atomically");
    assert_eq!(
        error.kind(),
        GitRepositoryReadErrorKind::ConflictedRepository
    );

    let gitlink = Repository::new(&[("anchor.txt", "first\n")]);
    let first = gitlink.head();
    gitlink.write("anchor.txt", "second\n");
    gitlink.git(["add", "anchor.txt"]);
    gitlink.commit("second");
    let second = gitlink.head();
    gitlink.git([
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{first},submodule"),
    ]);
    gitlink.commit("gitlink baseline");
    gitlink.git([
        "update-index",
        "--cacheinfo",
        &format!("160000,{second},submodule"),
    ]);
    let error = GitRepositoryReader::new()
        .read(&gitlink.root)
        .await
        .expect_err("changed gitlink must fail");
    assert_eq!(
        error.kind(),
        GitRepositoryReadErrorKind::UnsupportedEntryKind
    );
}

#[cfg(unix)]
#[tokio::test]
async fn public_reader_maps_type_changes_and_rejects_non_utf8_paths() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    let repository = Repository::new(&[("typed", "regular\n")]);
    repository.remove("typed");
    symlink("target", repository.root.join("typed")).expect("fixture symlink must be created");
    let set = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect("regular-to-symlink change must be read");
    assert_eq!(
        paths_and_kinds(&set),
        vec![("typed", RepositoryChangeKind::TypeChanged)]
    );

    let invalid = std::ffi::OsString::from_vec(b"invalid-\xff".to_vec());
    if let Err(error) = fs::write(repository.root.join(invalid), "private") {
        eprintln!("non-UTF-8 fixture paths are unavailable in this environment: {error}");
        return;
    }
    let error = GitRepositoryReader::new()
        .read(&repository.root)
        .await
        .expect_err("non-UTF-8 repository path must fail");
    assert_eq!(
        error.kind(),
        GitRepositoryReadErrorKind::UnsupportedPathEncoding
    );
    assert!(!error.to_string().contains("private"));
}

#[tokio::test]
async fn public_reader_accepts_sha256_baselines_when_local_git_supports_them() {
    let temp = tempfile::tempdir().expect("SHA-256 repository owner must be created");
    let root = temp.path().join("sha256");
    fs::create_dir(&root).expect("SHA-256 repository root must be created");
    let output = Command::new("git")
        .current_dir(temp.path())
        .args(["init", "--object-format=sha256", "sha256"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", temp.path().join("missing-global"))
        .output()
        .expect("local Git executable must start");
    if !output.status.success() {
        eprintln!("SHA-256 Git repository format is unavailable in this environment");
        return;
    }
    git_at(&root, ["config", "user.name", "OneAgent Test"]);
    git_at(&root, ["config", "user.email", "oneagent@example.invalid"]);
    fs::write(root.join("tracked.txt"), "content\n").expect("SHA-256 fixture must be written");
    git_at(&root, ["add", "tracked.txt"]);
    git_at(&root, ["commit", "-m", "initial"]);

    let set = GitRepositoryReader::new()
        .read(&root)
        .await
        .expect("supported SHA-256 repository must be read");
    assert_eq!(set.baseline().as_str().len(), 64);
    assert!(set.is_empty());
}
