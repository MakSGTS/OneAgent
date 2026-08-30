use oneagent_runtime::{
    GitChangeCompleteness, GitChangeSet, GitChangeSetErrorKind, GitCommitId, GitCurrentEndpoint,
    MAX_REPOSITORY_CHANGES, RepositoryChange, RepositoryChangeKind, RepositoryChangePath,
    RepositoryChangePathErrorKind,
};

const SHA1: &str = "0123456789abcdef0123456789abcdef01234567";

fn path(value: &str) -> RepositoryChangePath {
    RepositoryChangePath::new(value).expect("public test path must be valid")
}

fn change(kind: RepositoryChangeKind, value: &str) -> RepositoryChange {
    let value = path(value);
    match kind {
        RepositoryChangeKind::Added | RepositoryChangeKind::Untracked => {
            RepositoryChange::new(kind, None, Some(value))
        }
        RepositoryChangeKind::Deleted => RepositoryChange::new(kind, Some(value), None),
        RepositoryChangeKind::Modified | RepositoryChangeKind::TypeChanged => {
            RepositoryChange::new(kind, Some(value.clone()), Some(value))
        }
    }
    .expect("public test change must be valid")
}

#[test]
fn public_domain_preserves_pinned_endpoints_and_explicit_completeness() {
    let set = GitChangeSet::new(GitCommitId::new(SHA1).expect("baseline must be valid"), [])
        .expect("empty set must be valid");

    assert_eq!(set.baseline().as_str(), SHA1);
    assert_eq!(set.current(), GitCurrentEndpoint::Worktree);
    assert_eq!(
        set.completeness(),
        GitChangeCompleteness::TrackedAndUntrackedNonIgnored
    );
    assert!(set.is_empty());
}

#[test]
fn public_domain_exposes_every_closed_status_with_accepted_paths() {
    let kinds = [
        RepositoryChangeKind::Added,
        RepositoryChangeKind::Modified,
        RepositoryChangeKind::Deleted,
        RepositoryChangeKind::TypeChanged,
        RepositoryChangeKind::Untracked,
    ];
    let set = GitChangeSet::new(
        GitCommitId::new(SHA1).expect("baseline must be valid"),
        kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| change(kind, &format!("src/{index}"))),
    )
    .expect("status set must be valid");

    assert_eq!(
        set.changes()
            .iter()
            .map(RepositoryChange::kind)
            .collect::<Vec<_>>(),
        kinds
    );
}

#[test]
fn public_domain_is_equal_across_input_order_and_duplicate_delivery() {
    let baseline = GitCommitId::new(SHA1).expect("baseline must be valid");
    let first = GitChangeSet::new(
        baseline.clone(),
        [
            change(RepositoryChangeKind::Deleted, "z/old"),
            change(RepositoryChangeKind::Added, "a/new"),
            change(RepositoryChangeKind::Added, "a/new"),
        ],
    )
    .expect("first set must be valid");
    let second = GitChangeSet::new(
        baseline,
        [
            change(RepositoryChangeKind::Added, "a/new"),
            change(RepositoryChangeKind::Deleted, "z/old"),
        ],
    )
    .expect("second set must be valid");

    assert_eq!(first, second);
    assert_eq!(first.changes()[0].effective_path().as_str(), "a/new");
    assert_eq!(first.changes()[1].effective_path().as_str(), "z/old");
}

#[test]
fn public_domain_rejects_escape_and_conflict_without_sensitive_values() {
    let escape =
        RepositoryChangePath::new("../private/secret.bsl").expect_err("escaping path must fail");
    assert_eq!(escape.kind(), RepositoryChangePathErrorKind::NonCanonical);
    assert!(!escape.to_string().contains("secret"));

    let conflict = GitChangeSet::new(
        GitCommitId::new(SHA1).expect("baseline must be valid"),
        [
            change(RepositoryChangeKind::Modified, "private/secret.bsl"),
            change(RepositoryChangeKind::Deleted, "private/secret.bsl"),
        ],
    )
    .expect_err("conflicting evidence must fail");
    assert_eq!(conflict.kind(), GitChangeSetErrorKind::ConflictingChange);
    assert!(!conflict.to_string().contains("private"));
}

#[test]
fn public_domain_enforces_exact_and_one_over_change_bounds() {
    let baseline = GitCommitId::new(SHA1).expect("baseline must be valid");
    let exact = (0..MAX_REPOSITORY_CHANGES)
        .map(|index| change(RepositoryChangeKind::Untracked, &format!("p/{index:05}")))
        .collect::<Vec<_>>();
    assert_eq!(
        GitChangeSet::new(baseline.clone(), exact)
            .expect("exact bound must pass")
            .changes()
            .len(),
        MAX_REPOSITORY_CHANGES
    );

    let over = (0..=MAX_REPOSITORY_CHANGES)
        .map(|index| change(RepositoryChangeKind::Untracked, &format!("p/{index:05}")))
        .collect::<Vec<_>>();
    let error = GitChangeSet::new(baseline, over).expect_err("one-over bound must fail");
    assert_eq!(error.kind(), GitChangeSetErrorKind::TooManyChanges);
    assert_eq!(error.normalized_count(), MAX_REPOSITORY_CHANGES + 1);
}
