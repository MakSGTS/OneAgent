//! Deterministic normalized repository change evidence.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Maximum number of normalized changes in one accepted set.
pub const MAX_REPOSITORY_CHANGES: usize = 10_000;

/// Maximum UTF-8 byte length of one accepted repository-relative path.
pub const MAX_REPOSITORY_CHANGE_PATH_BYTES: usize = 4_096;

/// Stable category for an invalid Git commit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitCommitIdErrorKind {
    /// The identifier is neither a SHA-1 nor a SHA-256 hexadecimal length.
    InvalidLength,
    /// The identifier contains a non-lowercase-hexadecimal byte.
    InvalidCharacter,
}

/// Redacted error produced while validating a Git commit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitCommitIdError {
    kind: GitCommitIdErrorKind,
}

impl GitCommitIdError {
    const fn new(kind: GitCommitIdErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> GitCommitIdErrorKind {
        self.kind
    }
}

impl Display for GitCommitIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.kind {
            GitCommitIdErrorKind::InvalidLength => {
                "Git commit identifier has an unsupported length"
            }
            GitCommitIdErrorKind::InvalidCharacter => {
                "Git commit identifier is not lowercase hexadecimal"
            }
        })
    }
}

impl Error for GitCommitIdError {}

/// Validated immutable Git commit object identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitCommitId(String);

impl GitCommitId {
    /// Creates a validated SHA-1 or SHA-256 Git object identifier.
    ///
    /// # Errors
    ///
    /// Returns [`GitCommitIdError`] for an unsupported length or character.
    pub fn new(value: impl Into<String>) -> Result<Self, GitCommitIdError> {
        let value = value.into();
        if !matches!(value.len(), 40 | 64) {
            return Err(GitCommitIdError::new(GitCommitIdErrorKind::InvalidLength));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(GitCommitIdError::new(
                GitCommitIdErrorKind::InvalidCharacter,
            ));
        }
        Ok(Self(value))
    }

    /// Returns the canonical lowercase hexadecimal value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for GitCommitId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable category for an invalid repository change path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryChangePathErrorKind {
    /// The supplied path is empty.
    Empty,
    /// The supplied path exceeds the accepted UTF-8 byte bound.
    TooLong,
    /// The supplied path is absolute or has a platform prefix.
    NotRelative,
    /// The supplied path contains a non-canonical component.
    NonCanonical,
    /// The supplied path contains a backslash separator.
    UnsupportedSeparator,
    /// The supplied path contains a NUL byte.
    Nul,
}

/// Redacted error produced while validating a repository change path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepositoryChangePathError {
    kind: RepositoryChangePathErrorKind,
}

impl RepositoryChangePathError {
    const fn new(kind: RepositoryChangePathErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> RepositoryChangePathErrorKind {
        self.kind
    }
}

impl Display for RepositoryChangePathError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.kind {
            RepositoryChangePathErrorKind::Empty => "Repository change path is empty",
            RepositoryChangePathErrorKind::TooLong => {
                "Repository change path exceeds the supported length"
            }
            RepositoryChangePathErrorKind::NotRelative => {
                "Repository change path is not a confined relative path"
            }
            RepositoryChangePathErrorKind::NonCanonical => {
                "Repository change path contains a non-canonical component"
            }
            RepositoryChangePathErrorKind::UnsupportedSeparator => {
                "Repository change path contains an unsupported separator"
            }
            RepositoryChangePathErrorKind::Nul => "Repository change path contains a NUL byte",
        })
    }
}

impl Error for RepositoryChangePathError {}

/// Validated UTF-8 repository-relative path with `/` separators.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepositoryChangePath(String);

impl RepositoryChangePath {
    /// Creates a confined canonical repository-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryChangePathError`] when the path violates the
    /// accepted encoding, component, separator, or length contract.
    pub fn new(value: impl Into<String>) -> Result<Self, RepositoryChangePathError> {
        let value = value.into();
        if value.is_empty() {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::Empty,
            ));
        }
        if value.len() > MAX_REPOSITORY_CHANGE_PATH_BYTES {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::TooLong,
            ));
        }
        if value.contains('\0') {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::Nul,
            ));
        }
        if value.contains('\\') {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::UnsupportedSeparator,
            ));
        }
        if value.starts_with('/') || has_windows_prefix(&value) {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::NotRelative,
            ));
        }
        if value.ends_with('/')
            || value
                .split('/')
                .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return Err(RepositoryChangePathError::new(
                RepositoryChangePathErrorKind::NonCanonical,
            ));
        }
        Ok(Self(value))
    }

    /// Returns the canonical forward-slash path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RepositoryChangePath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn has_windows_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Closed normalized repository change vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RepositoryChangeKind {
    /// A tracked path is present only in the current worktree.
    Added,
    /// A tracked path has changed content without a type change.
    Modified,
    /// A tracked path is present only in the baseline.
    Deleted,
    /// A tracked path changed its repository entry type or mode class.
    TypeChanged,
    /// A current non-ignored path is not tracked by the baseline or index.
    Untracked,
}

/// Stable category for an invalid change status/path combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryChangeErrorKind {
    /// The selected kind requires no previous path.
    UnexpectedPreviousPath,
    /// The selected kind requires a previous path.
    MissingPreviousPath,
    /// The selected kind requires no current path.
    UnexpectedCurrentPath,
    /// The selected kind requires a current path.
    MissingCurrentPath,
    /// Modified or type-changed evidence must retain one exact path.
    MismatchedPaths,
}

/// Redacted error produced while constructing one normalized change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepositoryChangeError {
    kind: RepositoryChangeErrorKind,
}

impl RepositoryChangeError {
    const fn new(kind: RepositoryChangeErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> RepositoryChangeErrorKind {
        self.kind
    }
}

impl Display for RepositoryChangeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.kind {
            RepositoryChangeErrorKind::UnexpectedPreviousPath => {
                "Repository change has an unexpected previous path"
            }
            RepositoryChangeErrorKind::MissingPreviousPath => {
                "Repository change is missing its previous path"
            }
            RepositoryChangeErrorKind::UnexpectedCurrentPath => {
                "Repository change has an unexpected current path"
            }
            RepositoryChangeErrorKind::MissingCurrentPath => {
                "Repository change is missing its current path"
            }
            RepositoryChangeErrorKind::MismatchedPaths => {
                "Repository change requires equal previous and current paths"
            }
        })
    }
}

impl Error for RepositoryChangeError {}

/// One validated normalized repository change.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepositoryChange {
    kind: RepositoryChangeKind,
    previous_path: Option<RepositoryChangePath>,
    current_path: Option<RepositoryChangePath>,
}

impl RepositoryChange {
    /// Creates a change with the accepted kind/path shape.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryChangeError`] when the path presence or equality
    /// disagrees with the selected kind.
    pub fn new(
        kind: RepositoryChangeKind,
        previous_path: Option<RepositoryChangePath>,
        current_path: Option<RepositoryChangePath>,
    ) -> Result<Self, RepositoryChangeError> {
        validate_change_paths(kind, previous_path.as_ref(), current_path.as_ref())?;
        Ok(Self {
            kind,
            previous_path,
            current_path,
        })
    }

    /// Returns the normalized change kind.
    #[must_use]
    pub const fn kind(&self) -> RepositoryChangeKind {
        self.kind
    }

    /// Returns the baseline path when the change has one.
    #[must_use]
    pub const fn previous_path(&self) -> Option<&RepositoryChangePath> {
        self.previous_path.as_ref()
    }

    /// Returns the current path when the change has one.
    #[must_use]
    pub const fn current_path(&self) -> Option<&RepositoryChangePath> {
        self.current_path.as_ref()
    }

    /// Returns the current path when present, otherwise the previous path.
    ///
    /// # Panics
    ///
    /// Panics only if the validated change-path invariant is broken internally.
    #[must_use]
    pub fn effective_path(&self) -> &RepositoryChangePath {
        self.current_path
            .as_ref()
            .or(self.previous_path.as_ref())
            .expect("validated repository changes always contain one path")
    }
}

fn validate_change_paths(
    kind: RepositoryChangeKind,
    previous_path: Option<&RepositoryChangePath>,
    current_path: Option<&RepositoryChangePath>,
) -> Result<(), RepositoryChangeError> {
    match kind {
        RepositoryChangeKind::Added | RepositoryChangeKind::Untracked => {
            if previous_path.is_some() {
                return Err(RepositoryChangeError::new(
                    RepositoryChangeErrorKind::UnexpectedPreviousPath,
                ));
            }
            if current_path.is_none() {
                return Err(RepositoryChangeError::new(
                    RepositoryChangeErrorKind::MissingCurrentPath,
                ));
            }
        }
        RepositoryChangeKind::Deleted => {
            if previous_path.is_none() {
                return Err(RepositoryChangeError::new(
                    RepositoryChangeErrorKind::MissingPreviousPath,
                ));
            }
            if current_path.is_some() {
                return Err(RepositoryChangeError::new(
                    RepositoryChangeErrorKind::UnexpectedCurrentPath,
                ));
            }
        }
        RepositoryChangeKind::Modified | RepositoryChangeKind::TypeChanged => {
            let previous_path = previous_path.ok_or_else(|| {
                RepositoryChangeError::new(RepositoryChangeErrorKind::MissingPreviousPath)
            })?;
            let current_path = current_path.ok_or_else(|| {
                RepositoryChangeError::new(RepositoryChangeErrorKind::MissingCurrentPath)
            })?;
            if previous_path != current_path {
                return Err(RepositoryChangeError::new(
                    RepositoryChangeErrorKind::MismatchedPaths,
                ));
            }
        }
    }
    Ok(())
}

impl Ord for RepositoryChange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.effective_path()
            .cmp(other.effective_path())
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.previous_path.cmp(&other.previous_path))
            .then_with(|| self.current_path.cmp(&other.current_path))
    }
}

impl PartialOrd for RepositoryChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Closed current endpoint kind for the first local Git slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GitCurrentEndpoint {
    /// The visible local worktree state.
    Worktree,
}

/// Closed completeness statement for one Git change set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GitChangeCompleteness {
    /// Tracked final-worktree and non-ignored untracked paths are included.
    TrackedAndUntrackedNonIgnored,
}

/// Stable failure category for complete change-set normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitChangeSetErrorKind {
    /// The normalized set exceeds [`MAX_REPOSITORY_CHANGES`].
    TooManyChanges,
    /// Two non-identical changes claim one effective path.
    ConflictingChange,
}

/// Redacted error produced while constructing one complete Git change set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitChangeSetError {
    kind: GitChangeSetErrorKind,
    normalized_count: usize,
}

impl GitChangeSetError {
    const fn new(kind: GitChangeSetErrorKind, normalized_count: usize) -> Self {
        Self {
            kind,
            normalized_count,
        }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> GitChangeSetErrorKind {
        self.kind
    }

    /// Returns the bounded normalized count observed before rejection.
    #[must_use]
    pub const fn normalized_count(self) -> usize {
        self.normalized_count
    }
}

impl Display for GitChangeSetError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            GitChangeSetErrorKind::TooManyChanges => write!(
                formatter,
                "Git change set contains {} normalized changes, exceeding the supported limit",
                self.normalized_count
            ),
            GitChangeSetErrorKind::ConflictingChange => {
                formatter.write_str("Git change set contains conflicting changes")
            }
        }
    }
}

impl Error for GitChangeSetError {}

/// Complete deterministic normalized evidence for `HEAD` to one local worktree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitChangeSet {
    baseline: GitCommitId,
    current: GitCurrentEndpoint,
    completeness: GitChangeCompleteness,
    changes: Box<[RepositoryChange]>,
}

impl GitChangeSet {
    /// Normalizes one directional Git change set.
    ///
    /// Exact duplicate changes collapse before the normalized count bound is
    /// checked. Non-identical records for one effective path are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`GitChangeSetError`] for an over-bound or conflicting set.
    pub fn new(
        baseline: GitCommitId,
        changes: impl IntoIterator<Item = RepositoryChange>,
    ) -> Result<Self, GitChangeSetError> {
        let mut changes = changes.into_iter().collect::<Vec<_>>();
        changes.sort_unstable();
        changes.dedup();
        if changes.len() > MAX_REPOSITORY_CHANGES {
            return Err(GitChangeSetError::new(
                GitChangeSetErrorKind::TooManyChanges,
                changes.len(),
            ));
        }
        if changes
            .windows(2)
            .any(|pair| pair[0].effective_path() == pair[1].effective_path())
        {
            return Err(GitChangeSetError::new(
                GitChangeSetErrorKind::ConflictingChange,
                changes.len(),
            ));
        }
        Ok(Self {
            baseline,
            current: GitCurrentEndpoint::Worktree,
            completeness: GitChangeCompleteness::TrackedAndUntrackedNonIgnored,
            changes: changes.into_boxed_slice(),
        })
    }

    /// Returns the pinned baseline commit identity.
    #[must_use]
    pub const fn baseline(&self) -> &GitCommitId {
        &self.baseline
    }

    /// Returns the closed current endpoint kind.
    #[must_use]
    pub const fn current(&self) -> GitCurrentEndpoint {
        self.current
    }

    /// Returns the explicit included-layer completeness statement.
    #[must_use]
    pub const fn completeness(&self) -> GitChangeCompleteness {
        self.completeness
    }

    /// Returns changes in canonical total order.
    #[must_use]
    pub const fn changes(&self) -> &[RepositoryChange] {
        &self.changes
    }

    /// Returns `true` when no accepted repository change is present.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GitChangeSet, GitChangeSetErrorKind, GitCommitId, GitCommitIdErrorKind,
        MAX_REPOSITORY_CHANGE_PATH_BYTES, MAX_REPOSITORY_CHANGES, RepositoryChange,
        RepositoryChangeErrorKind, RepositoryChangeKind, RepositoryChangePath,
        RepositoryChangePathErrorKind,
    };

    const SHA1: &str = "0123456789abcdef0123456789abcdef01234567";

    fn path(value: &str) -> RepositoryChangePath {
        RepositoryChangePath::new(value).expect("test path must be valid")
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
        .expect("test change must be valid")
    }

    #[test]
    fn commit_ids_accept_sha1_and_sha256_lowercase_hex() {
        let sha1 = GitCommitId::new(SHA1).expect("SHA-1 must be accepted");
        let sha256 = GitCommitId::new("a".repeat(64)).expect("SHA-256 must be accepted");
        assert_eq!(sha1.as_str(), SHA1);
        assert_eq!(sha256.as_str(), "a".repeat(64));
    }

    #[test]
    fn commit_ids_reject_length_and_character_without_echoing_input() {
        let short = GitCommitId::new("secret").expect_err("short input must fail");
        let upper = GitCommitId::new("A".repeat(40)).expect_err("uppercase input must fail");
        assert_eq!(short.kind(), GitCommitIdErrorKind::InvalidLength);
        assert_eq!(upper.kind(), GitCommitIdErrorKind::InvalidCharacter);
        assert!(!short.to_string().contains("secret"));
        assert!(!upper.to_string().contains('A'));
    }

    #[test]
    fn paths_accept_confined_utf8_at_exact_bound() {
        let exact = "a".repeat(MAX_REPOSITORY_CHANGE_PATH_BYTES);
        let accepted = RepositoryChangePath::new(exact.clone()).expect("exact bound must pass");
        assert_eq!(accepted.as_str(), exact);
        assert_eq!(
            path("src/Документ/Module.bsl").as_str(),
            "src/Документ/Module.bsl"
        );
    }

    #[test]
    fn paths_reject_empty_over_bound_absolute_prefix_and_noncanonical_components() {
        let cases = [
            ("", RepositoryChangePathErrorKind::Empty),
            ("/root", RepositoryChangePathErrorKind::NotRelative),
            ("C:/root", RepositoryChangePathErrorKind::NotRelative),
            (
                "src\\file",
                RepositoryChangePathErrorKind::UnsupportedSeparator,
            ),
            ("src//file", RepositoryChangePathErrorKind::NonCanonical),
            ("src/./file", RepositoryChangePathErrorKind::NonCanonical),
            ("src/../file", RepositoryChangePathErrorKind::NonCanonical),
            ("src/file/", RepositoryChangePathErrorKind::NonCanonical),
            ("src/\0file", RepositoryChangePathErrorKind::Nul),
        ];
        for (value, kind) in cases {
            let error = RepositoryChangePath::new(value).expect_err("invalid path must fail");
            assert_eq!(error.kind(), kind);
            if !value.is_empty() {
                assert!(!error.to_string().contains(value));
            }
        }
        let over = RepositoryChangePath::new("a".repeat(MAX_REPOSITORY_CHANGE_PATH_BYTES + 1))
            .expect_err("one-over path must fail");
        assert_eq!(over.kind(), RepositoryChangePathErrorKind::TooLong);
    }

    #[test]
    fn every_change_kind_accepts_only_its_path_shape() {
        for kind in [
            RepositoryChangeKind::Added,
            RepositoryChangeKind::Modified,
            RepositoryChangeKind::Deleted,
            RepositoryChangeKind::TypeChanged,
            RepositoryChangeKind::Untracked,
        ] {
            assert_eq!(change(kind, "src/file").kind(), kind);
        }

        let missing = RepositoryChange::new(RepositoryChangeKind::Added, None, None)
            .expect_err("added change without current path must fail");
        assert_eq!(
            missing.kind(),
            RepositoryChangeErrorKind::MissingCurrentPath
        );
        let unexpected = RepositoryChange::new(
            RepositoryChangeKind::Deleted,
            Some(path("old")),
            Some(path("new")),
        )
        .expect_err("deleted change with current path must fail");
        assert_eq!(
            unexpected.kind(),
            RepositoryChangeErrorKind::UnexpectedCurrentPath
        );
        let mismatch = RepositoryChange::new(
            RepositoryChangeKind::Modified,
            Some(path("old")),
            Some(path("new")),
        )
        .expect_err("modified paths must match");
        assert_eq!(mismatch.kind(), RepositoryChangeErrorKind::MismatchedPaths);
    }

    #[test]
    fn change_sets_are_canonical_across_reorder_and_exact_duplicates() {
        let baseline = GitCommitId::new(SHA1).expect("baseline must be valid");
        let first = GitChangeSet::new(
            baseline.clone(),
            [
                change(RepositoryChangeKind::Deleted, "z"),
                change(RepositoryChangeKind::Modified, "a"),
                change(RepositoryChangeKind::Modified, "a"),
            ],
        )
        .expect("first set must be valid");
        let second = GitChangeSet::new(
            baseline,
            [
                change(RepositoryChangeKind::Modified, "a"),
                change(RepositoryChangeKind::Deleted, "z"),
            ],
        )
        .expect("second set must be valid");
        assert_eq!(first, second);
        assert_eq!(first.changes().len(), 2);
        assert_eq!(first.changes()[0].effective_path().as_str(), "a");
        assert_eq!(first.changes()[1].effective_path().as_str(), "z");
    }

    #[test]
    fn change_sets_reject_conflicting_same_path_without_path_leak() {
        let error = GitChangeSet::new(
            GitCommitId::new(SHA1).expect("baseline must be valid"),
            [
                change(RepositoryChangeKind::Modified, "private/secret.bsl"),
                change(RepositoryChangeKind::Deleted, "private/secret.bsl"),
            ],
        )
        .expect_err("conflicting changes must fail");
        assert_eq!(error.kind(), GitChangeSetErrorKind::ConflictingChange);
        assert!(!error.to_string().contains("private"));
    }

    #[test]
    fn change_sets_accept_exact_count_and_reject_one_over() {
        let baseline = GitCommitId::new(SHA1).expect("baseline must be valid");
        let exact = (0..MAX_REPOSITORY_CHANGES)
            .map(|index| change(RepositoryChangeKind::Untracked, &format!("p/{index:05}")))
            .collect::<Vec<_>>();
        let accepted =
            GitChangeSet::new(baseline.clone(), exact).expect("exact normalized count must pass");
        assert_eq!(accepted.changes().len(), MAX_REPOSITORY_CHANGES);

        let over = (0..=MAX_REPOSITORY_CHANGES)
            .map(|index| change(RepositoryChangeKind::Untracked, &format!("p/{index:05}")))
            .collect::<Vec<_>>();
        let error = GitChangeSet::new(baseline, over).expect_err("one-over set must fail");
        assert_eq!(error.kind(), GitChangeSetErrorKind::TooManyChanges);
        assert_eq!(error.normalized_count(), MAX_REPOSITORY_CHANGES + 1);
    }

    #[test]
    fn empty_sets_are_complete_for_the_declared_repository_layers() {
        let set = GitChangeSet::new(GitCommitId::new(SHA1).expect("baseline must be valid"), [])
            .expect("empty set must be valid");
        assert!(set.is_empty());
        assert_eq!(set.changes(), []);
    }
}
