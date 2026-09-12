//! Bounded cooperative filesystem transaction primitives; no publication authority.

use oneagent_analysis::safe_edit::SafeEditProjectionAdmission;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

pub(super) const MAX_ENTRIES: usize = 16_384;
pub(super) const MAX_PATH: usize = 4_096;
pub(super) const MAX_PATHS: usize = 4_194_304;
pub(super) const MAX_FILE: usize = 8_388_608;
pub(super) const MAX_BASELINE: usize = 67_108_864;
pub(super) const MAX_BUFFERS: usize = 268_435_456;
pub(super) const MAX_DOCUMENT: usize = 1_048_576;
pub(super) const MAX_EDITED: usize = 8_388_608;
pub(super) const MAX_FILES: usize = 64;
pub(super) const MAX_OPERATIONS: usize = 4_096;
pub(super) const CACHE_PATH: &str = ".oneagent/cache/workspace-v1.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditIoError {
    Bounds,
    Confinement,
    Changed,
    Io,
}
type Result<T> = std::result::Result<T, EditIoError>;

#[cfg(test)]
pub(super) mod faults {
    use std::cell::RefCell;
    thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
    type FaultAction = (&'static str, Box<dyn FnOnce()>);
    #[derive(Default)]
    struct State {
        action: Option<FaultAction>,
        failures: Vec<(&'static str, usize)>,
        unwinds: Vec<(&'static str, usize)>,
        events: Vec<&'static str>,
    }
    pub(in crate::workspace) fn set(failures: Vec<(&'static str, usize)>) {
        STATE.with(|state| {
            *state.borrow_mut() = State {
                failures,
                events: Vec::new(),
                action: None,
                unwinds: Vec::new(),
            }
        });
    }
    pub(in crate::workspace) fn events() -> Vec<&'static str> {
        STATE.with(|state| state.borrow().events.clone())
    }
    pub(in crate::workspace) fn unwind(points: Vec<(&'static str, usize)>) {
        STATE.with(|state| state.borrow_mut().unwinds = points);
    }
    pub(in crate::workspace) fn record(name: &'static str) {
        STATE.with(|state| state.borrow_mut().events.push(name));
    }
    pub(in crate::workspace) fn action(at: &'static str, action: impl FnOnce() + 'static) {
        STATE.with(|state| state.borrow_mut().action = Some((at, Box::new(action))));
    }
    pub(super) fn hit(name: &'static str) -> super::Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.events.push(name);
            if state.action.as_ref().is_some_and(|(at, _)| *at == name) {
                let (_, action) = state.action.take().unwrap();
                action();
            }
            let ordinal = state.events.iter().filter(|event| **event == name).count();
            assert!(
                !state.unwinds.contains(&(name, ordinal)),
                "SECRET_UNWIND /absolute/private/path SECRET_SOURCE_TOKEN"
            );
            if state.failures.contains(&(name, ordinal)) {
                super::io(Err(std::io::Error::other(
                    "SECRET_IO_ERROR /absolute/private/path SECRET_SOURCE_TOKEN",
                )))
            } else {
                Ok(())
            }
        })
    }
}

#[cfg_attr(not(test), allow(clippy::unnecessary_wraps))]
fn checkpoint(name: &'static str) -> Result<()> {
    #[cfg(test)]
    faults::hit(name)?;
    let _ = name;
    Ok(())
}

fn io<T>(value: std::io::Result<T>) -> Result<T> {
    value.map_err(|_| EditIoError::Io)
}
fn ensure(value: bool, error: EditIoError) -> Result<()> {
    if value { Ok(()) } else { Err(error) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Identity {
    device: u64,
    inode: u64,
}

#[cfg(unix)]
fn identity(metadata: &fs::Metadata) -> Result<Identity> {
    use std::os::unix::fs::MetadataExt;
    ensure(
        metadata.is_dir() || metadata.nlink() == 1,
        EditIoError::Confinement,
    )?;
    Ok(Identity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}
#[cfg(not(unix))]
fn identity(_metadata: &fs::Metadata) -> Result<Identity> {
    Err(EditIoError::Confinement)
}

#[cfg(unix)]
fn mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}
#[cfg(not(unix))]
fn mode(_metadata: &fs::Metadata) -> u32 {
    0
}

#[derive(Clone)]
struct Entry {
    identity: Identity,
    mode: u32,
    bytes: Option<Arc<[u8]>>,
}

impl Entry {
    fn equivalent(&self, other: &Self) -> bool {
        self.mode == other.mode && self.bytes == other.bytes
    }
}

/// Exact bounded publication source observation. Clones share immutable raw bytes.
#[derive(Clone)]
pub(super) struct EditBaseline {
    root: PathBuf,
    entries: BTreeMap<PathBuf, Entry>,
    raw_bytes: usize,
}

/// Checks budgets before allocations or disk creation.
#[derive(Debug, Default)]
pub(super) struct EditIoBudget {
    entries: usize,
    paths: usize,
    bytes: usize,
}

impl EditIoBudget {
    fn entry(&mut self, path: &Path) -> Result<()> {
        self.entry_length(path.as_os_str().len())
    }

    fn entry_length(&mut self, length: usize) -> Result<()> {
        ensure(length <= MAX_PATH, EditIoError::Bounds)?;
        let entries = self.entries.checked_add(1).ok_or(EditIoError::Bounds)?;
        let paths = self.paths.checked_add(length).ok_or(EditIoError::Bounds)?;
        ensure(
            entries <= MAX_ENTRIES && paths <= MAX_PATHS,
            EditIoError::Bounds,
        )?;
        self.entries = entries;
        self.paths = paths;
        Ok(())
    }

    fn file(&mut self, length: u64) -> Result<usize> {
        let length = usize::try_from(length).map_err(|_| EditIoError::Bounds)?;
        let total = self.bytes.checked_add(length).ok_or(EditIoError::Bounds)?;
        ensure(
            length <= MAX_FILE && total <= MAX_BASELINE,
            EditIoError::Bounds,
        )?;
        self.bytes = total;
        Ok(length)
    }

    pub(super) fn buffers(lengths: &[usize]) -> Result<()> {
        let total = lengths
            .iter()
            .try_fold(0usize, |n, &v| n.checked_add(v))
            .ok_or(EditIoError::Bounds)?;
        ensure(total <= MAX_BUFFERS, EditIoError::Bounds)
    }
}

impl EditBaseline {
    pub(super) fn capture(root: &Path, owned: &[OwnedEditFile]) -> Result<Self> {
        let canonical = io(root.canonicalize())?;
        ensure(canonical == root, EditIoError::Confinement)?;
        let mut result = Self {
            root: root.to_owned(),
            entries: BTreeMap::new(),
            raw_bytes: 0,
        };
        let mut budget = EditIoBudget::default();
        budget.entry(Path::new(""))?;
        result.scan(Path::new(""), owned, &mut budget)?;
        result.raw_bytes = budget.bytes;
        Ok(result)
    }

    fn scan(
        &mut self,
        relative: &Path,
        owned: &[OwnedEditFile],
        budget: &mut EditIoBudget,
    ) -> Result<()> {
        let metadata = validate_path(&self.root, relative)?;
        let file_identity = identity(&metadata)?;
        if let Some(entry) = owned
            .iter()
            .find(|entry| entry.path == relative && entry.present)
        {
            ensure(
                Some(file_identity) == entry.identity && metadata.is_file(),
                EditIoError::Confinement,
            )?;
            return Ok(());
        }
        if relative == Path::new(CACHE_PATH) {
            ensure(metadata.is_file(), EditIoError::Confinement)?;
            return Ok(());
        }
        let bytes = if metadata.is_file() {
            let length = budget.file(metadata.len())?;
            Some(Arc::from(read_checked(
                &self.root,
                relative,
                file_identity,
                length,
            )?))
        } else {
            None
        };
        self.entries.insert(
            relative.to_owned(),
            Entry {
                identity: file_identity,
                mode: mode(&metadata),
                bytes,
            },
        );
        if metadata.is_dir() {
            for entry in io(fs::read_dir(self.root.join(relative)))? {
                let entry = io(entry)?;
                let name = entry.file_name();
                let length = relative
                    .as_os_str()
                    .len()
                    .checked_add(name.len())
                    .and_then(|length| {
                        length.checked_add(usize::from(!relative.as_os_str().is_empty()))
                    })
                    .ok_or(EditIoError::Bounds)?;
                budget.entry_length(length)?;
                // Admit the full joined path before allocating or retaining it.
                let child = relative.join(name);
                self.scan(&child, owned, budget)?;
            }
        }
        Ok(())
    }

    pub(super) fn raw_bytes(&self) -> usize {
        self.raw_bytes
    }
    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn bytes(&self, path: &Path) -> Result<Arc<[u8]>> {
        self.entries
            .get(path)
            .and_then(|entry| entry.bytes.clone())
            .ok_or(EditIoError::Confinement)
    }

    pub(super) fn contains_directory(&self, path: &Path) -> bool {
        self.entries
            .get(path)
            .is_some_and(|entry| entry.bytes.is_none())
    }

    pub(super) fn equals(&self, other: &Self) -> bool {
        self.equivalent(other)
            && self.entries.iter().all(|(path, entry)| {
                other
                    .entries
                    .get(path)
                    .is_some_and(|other| entry.identity == other.identity)
            })
    }

    pub(super) fn equivalent(&self, other: &Self) -> bool {
        self.root == other.root
            && self.entries.len() == other.entries.len()
            && self.entries.iter().all(|(path, entry)| {
                other
                    .entries
                    .get(path)
                    .is_some_and(|other| entry.equivalent(other))
            })
    }

    pub(super) fn verify_tree(
        &self,
        replacements: &BTreeMap<PathBuf, Arc<[u8]>>,
        owned: &[OwnedEditFile],
    ) -> Result<Self> {
        EditIoBudget::buffers(&[self.raw_bytes, MAX_BASELINE, MAX_FILE, MAX_EDITED * 2])?;
        let actual = Self::capture(&self.root, owned)?;
        ensure(
            self.entries.len() == actual.entries.len(),
            EditIoError::Changed,
        )?;
        for (path, expected) in &self.entries {
            let observed = actual.entries.get(path).ok_or(EditIoError::Changed)?;
            if let Some(bytes) = replacements.get(path) {
                ensure(
                    observed.bytes.as_ref() == Some(bytes) && observed.mode == expected.mode,
                    EditIoError::Changed,
                )?;
            } else {
                ensure(
                    expected.equivalent(observed) && expected.identity == observed.identity,
                    EditIoError::Changed,
                )?;
            }
        }
        Ok(actual)
    }
}

fn validate_path(root: &Path, relative: &Path) -> Result<fs::Metadata> {
    ensure(
        relative
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        EditIoError::Confinement,
    )?;
    let mut path = root.to_owned();
    let mut metadata = io(fs::symlink_metadata(&path))?;
    ensure(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        EditIoError::Confinement,
    )?;
    for component in relative.components() {
        ensure(metadata.is_dir(), EditIoError::Confinement)?;
        path.push(component);
        metadata = io(fs::symlink_metadata(&path))?;
        ensure(
            !metadata.file_type().is_symlink() && (metadata.is_file() || metadata.is_dir()),
            EditIoError::Confinement,
        )?;
    }
    ensure(
        io(path.canonicalize())?.starts_with(root),
        EditIoError::Confinement,
    )?;
    identity(&metadata)?;
    Ok(metadata)
}

fn read_checked(
    root: &Path,
    relative: &Path,
    expected: Identity,
    length: usize,
) -> Result<Vec<u8>> {
    checkpoint("read")?;
    let metadata = validate_path(root, relative)?;
    ensure(
        metadata.is_file() && identity(&metadata)? == expected && metadata.len() == length as u64,
        EditIoError::Changed,
    )?;
    ensure(length <= MAX_FILE, EditIoError::Bounds)?;
    let file = io(File::open(root.join(relative)))?;
    ensure(
        identity(&io(file.metadata())?)? == expected,
        EditIoError::Changed,
    )?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| EditIoError::Bounds)?;
    ensure(bytes.capacity() == length, EditIoError::Bounds)?;
    bytes.resize(length, 0);
    let mut reader = file;
    io(reader.read_exact(&mut bytes))?;
    checkpoint("read_filled")?;
    let mut extra = [0u8; 1];
    ensure(
        bytes.len() == length && io(reader.read(&mut extra))? == 0,
        EditIoError::Changed,
    )?;
    ensure(
        identity(&validate_path(root, relative)?)? == expected,
        EditIoError::Changed,
    )?;
    Ok(bytes)
}

/// Exact created-file ownership, never a prefix-based cleanup permission.
pub(super) struct OwnedEditFile {
    path: PathBuf,
    // A successful create is retained even when descriptor identity acquisition
    // fails. Unknown identity never authorizes removal or tree exclusion.
    identity: Option<Identity>,
    present: bool,
}

struct Replacement {
    path: PathBuf,
    original: Arc<[u8]>,
    result: Arc<[u8]>,
    original_identity: Identity,
    permissions: fs::Permissions,
    stage: usize,
    backup: usize,
    attempted: bool,
}

/// Same production algorithm is used for apply, reversal and checked recovery.
pub(super) struct EditIo {
    baseline: EditBaseline,
    replacements: Vec<Replacement>,
    owned: Vec<OwnedEditFile>,
    attempt: u64,
    pub(super) admission: SafeEditProjectionAdmission,
}

impl EditIo {
    pub(super) fn admission(baseline: &EditBaseline) -> Result<SafeEditProjectionAdmission> {
        let raw = baseline
            .raw_bytes
            .checked_add(MAX_BASELINE * 2)
            .and_then(|bytes| bytes.checked_add(MAX_EDITED * 2))
            .and_then(|bytes| bytes.checked_add(MAX_FILE))
            .ok_or(EditIoError::Bounds)?;
        SafeEditProjectionAdmission::new(raw).map_err(|_| EditIoError::Bounds)
    }

    #[cfg(test)]
    pub(super) fn new(
        baseline: EditBaseline,
        results: BTreeMap<PathBuf, Arc<[u8]>>,
        attempt: u64,
    ) -> Result<Self> {
        let admission = Self::admission(&baseline)?;
        Self::with_admission(baseline, results, attempt, admission)
    }

    pub(super) fn with_admission(
        baseline: EditBaseline,
        results: BTreeMap<PathBuf, Arc<[u8]>>,
        attempt: u64,
        admission: SafeEditProjectionAdmission,
    ) -> Result<Self> {
        ensure(
            !results.is_empty() && results.len() <= MAX_FILES,
            EditIoError::Bounds,
        )?;
        let mut originals = 0usize;
        let mut outputs = 0usize;
        let mut identities = BTreeSet::new();
        // Complete accounting before collecting replacement or recovery records.
        for (path, result) in &results {
            let source = baseline.entries.get(path).ok_or(EditIoError::Confinement)?;
            let original = source.bytes.as_ref().ok_or(EditIoError::Confinement)?;
            ensure(
                original.len() <= MAX_DOCUMENT && result.len() <= MAX_DOCUMENT,
                EditIoError::Bounds,
            )?;
            originals = originals
                .checked_add(original.len())
                .ok_or(EditIoError::Bounds)?;
            outputs = outputs
                .checked_add(result.len())
                .ok_or(EditIoError::Bounds)?;
            ensure(identities.insert(source.identity), EditIoError::Confinement)?;
        }
        ensure(
            originals <= MAX_EDITED && outputs <= MAX_EDITED,
            EditIoError::Bounds,
        )?;
        // The same reservation already includes both verification scans,
        // read scratch, result/recovery bytes and the live frozen projection.
        ensure(
            admission.retained_bytes() >= Self::admission(&baseline)?.retained_bytes(),
            EditIoError::Bounds,
        )?;
        let mut replacements = Vec::with_capacity(results.len());
        for (path, result) in results {
            let source = baseline
                .entries
                .get(&path)
                .ok_or(EditIoError::Confinement)?;
            let permissions = validate_path(&baseline.root, &path)?.permissions();
            replacements.push(Replacement {
                path,
                original: source.bytes.clone().ok_or(EditIoError::Confinement)?,
                result,
                original_identity: source.identity,
                permissions,
                stage: 0,
                backup: 0,
                attempted: false,
            });
        }
        Ok(Self {
            baseline,
            replacements,
            owned: Vec::new(),
            attempt,
            admission,
        })
    }

    pub(super) fn retained_files(&self) -> usize {
        self.owned.iter().filter(|f| f.present).count()
    }
    pub(super) fn attempted(&self) -> bool {
        self.replacements.iter().any(|r| r.attempted)
    }
    pub(super) fn results(&self) -> BTreeMap<PathBuf, Arc<[u8]>> {
        self.replacements
            .iter()
            .map(|r| (r.path.clone(), Arc::clone(&r.result)))
            .collect()
    }
    pub(super) fn originals(&self) -> BTreeMap<PathBuf, Arc<[u8]>> {
        self.replacements
            .iter()
            .map(|r| (r.path.clone(), Arc::clone(&r.original)))
            .collect()
    }

    fn create_owned(
        &mut self,
        path: PathBuf,
        bytes: &[u8],
        permissions: &fs::Permissions,
    ) -> Result<usize> {
        let parent = path.parent().ok_or(EditIoError::Confinement)?;
        validate_path(&self.baseline.root, parent)?;
        let mut options = OpenOptions::new();
        // Register without allocation immediately after a successful create.
        // Recovery may recreate each removed backup once; reserve before I/O.
        self.owned.try_reserve(1).map_err(|_| EditIoError::Bounds)?;
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        checkpoint("create")?;
        let opened = options.open(self.baseline.root.join(&path));
        #[cfg(test)]
        if opened
            .as_ref()
            .is_err_and(|error| error.kind() == std::io::ErrorKind::AlreadyExists)
        {
            faults::record("create_already_exists");
        }
        let mut file = io(opened)?;
        let index = self.owned.len();
        self.owned.push(OwnedEditFile {
            path,
            identity: None,
            present: true,
        });
        checkpoint("created_metadata")?;
        let metadata = io(file.metadata())?;
        checkpoint("created_identity")?;
        let entry_identity = identity(&metadata)?;
        self.owned[index].identity = Some(entry_identity);
        checkpoint("write")?;
        io(file.write_all(bytes))?;
        checkpoint("permissions")?;
        io(file.set_permissions(permissions.clone()))?;
        checkpoint("sync")?;
        io(file.sync_all())?;
        drop(file);
        checkpoint("close_observation")?;
        checkpoint("readback")?;
        ensure(
            read_checked(
                &self.baseline.root,
                &self.owned[index].path,
                entry_identity,
                bytes.len(),
            )? == bytes,
            EditIoError::Changed,
        )?;
        Ok(index)
    }

    pub(super) fn stage_all(&mut self) -> Result<()> {
        ensure(
            self.replacements
                .len()
                .checked_mul(2)
                .is_some_and(|n| n <= 128),
            EditIoError::Bounds,
        )?;
        let disk = self
            .replacements
            .iter()
            .try_fold(0usize, |n, r| {
                n.checked_add(r.original.len())?.checked_add(r.result.len())
            })
            .ok_or(EditIoError::Bounds)?;
        ensure(disk <= 16_777_216, EditIoError::Bounds)?;
        let observed = self.baseline.verify_tree(&BTreeMap::new(), &self.owned)?;
        ensure(self.baseline.equals(&observed), EditIoError::Changed)?;
        drop(observed);
        for index in 0..self.replacements.len() {
            let replacement = &self.replacements[index];
            let parent = replacement.path.parent().ok_or(EditIoError::Confinement)?;
            let stage = parent.join(format!(".oneagent-edit-{}-{index}-result", self.attempt));
            let backup = parent.join(format!(".oneagent-edit-{}-{index}-backup", self.attempt));
            let result = Arc::clone(&replacement.result);
            let original = Arc::clone(&replacement.original);
            let permissions = replacement.permissions.clone();
            self.replacements[index].stage = self.create_owned(stage, &result, &permissions)?;
            self.replacements[index].backup = self.create_owned(backup, &original, &permissions)?;
        }
        Ok(())
    }

    pub(super) fn replace_checked(&mut self, cancelled: impl Fn() -> bool) -> Result<()> {
        let observed = self.baseline.verify_tree(&BTreeMap::new(), &self.owned)?;
        ensure(self.baseline.equals(&observed), EditIoError::Changed)?;
        for replacement in &self.replacements {
            for (index, bytes) in [
                (replacement.stage, &replacement.result),
                (replacement.backup, &replacement.original),
            ] {
                let owned = &self.owned[index];
                ensure(
                    read_checked(
                        &self.baseline.root,
                        &owned.path,
                        owned.identity.ok_or(EditIoError::Confinement)?,
                        bytes.len(),
                    )? == bytes.as_ref(),
                    EditIoError::Changed,
                )?;
            }
        }
        for replacement in &mut self.replacements {
            ensure(!cancelled(), EditIoError::Changed)?;
            ensure(
                read_checked(
                    &self.baseline.root,
                    &replacement.path,
                    replacement.original_identity,
                    replacement.original.len(),
                )? == replacement.original.as_ref(),
                EditIoError::Changed,
            )?;
            let stage = &mut self.owned[replacement.stage];
            ensure(
                read_checked(
                    &self.baseline.root,
                    &stage.path,
                    stage.identity.ok_or(EditIoError::Confinement)?,
                    replacement.result.len(),
                )? == replacement.result.as_ref(),
                EditIoError::Changed,
            )?;
            checkpoint("replace_before")?;
            replacement.attempted = true;
            let result = fs::rename(
                self.baseline.root.join(&stage.path),
                self.baseline.root.join(&replacement.path),
            );
            // A failed rename is ambiguous. Recovery observes the exact target.
            if result.is_ok() {
                stage.present = false;
            }
            checkpoint("replace_after")?;
            io(result)?;
        }
        Ok(())
    }

    pub(super) fn verify_results(&self) -> Result<EditBaseline> {
        self.baseline.verify_tree(&self.results(), &self.owned)
    }

    pub(super) fn cleanup_owned(&mut self) -> Result<()> {
        let mut failed = false;
        for entry in &mut self.owned {
            if !entry.present {
                continue;
            }
            let result = validate_path(&self.baseline.root, &entry.path).and_then(|m| {
                ensure(
                    Some(identity(&m)?) == entry.identity && m.is_file(),
                    EditIoError::Confinement,
                )?;
                checkpoint("cleanup")?;
                io(fs::remove_file(self.baseline.root.join(&entry.path)))
            });
            if result.is_ok() {
                entry.present = false;
                checkpoint("cleanup_after")?;
            } else {
                failed = true;
            }
        }
        ensure(!failed, EditIoError::Io)
    }

    pub(super) fn restore_checked(&mut self) -> Result<EditBaseline> {
        let mut failed = false;
        for index in (0..self.replacements.len()).rev() {
            if !self.replacements[index].attempted {
                continue;
            }
            if self.restore_one(index).is_err() {
                failed = true;
            }
        }
        if !failed && self.cleanup_owned().is_err() {
            failed = true;
        }
        checkpoint("restore_verify")?;
        let originals = self.originals();
        let restored = self.baseline.verify_tree(&originals, &self.owned);
        if !restored
            .as_ref()
            .is_ok_and(|state| self.baseline.equivalent(state))
        {
            failed = true;
        }
        ensure(!failed, EditIoError::Io)?;
        restored
    }

    pub(super) fn clean_unattempted(&mut self) -> Result<EditBaseline> {
        self.cleanup_owned()?;
        let observed = self.baseline.verify_tree(&BTreeMap::new(), &self.owned)?;
        ensure(self.baseline.equals(&observed), EditIoError::Changed)?;
        ensure(self.retained_files() == 0, EditIoError::Io)?;
        Ok(observed)
    }

    fn restore_one(&mut self, index: usize) -> Result<()> {
        checkpoint("restore_check")?;
        let replacement = &self.replacements[index];
        let metadata = validate_path(&self.baseline.root, &replacement.path)?;
        let current_identity = identity(&metadata)?;
        let current = read_checked(
            &self.baseline.root,
            &replacement.path,
            current_identity,
            usize::try_from(metadata.len()).map_err(|_| EditIoError::Bounds)?,
        )?;
        if current == replacement.original.as_ref() {
            ensure(
                mode(&metadata) == self.baseline.entries[&replacement.path].mode,
                EditIoError::Changed,
            )?;
            return Ok(());
        }
        ensure(
            current == replacement.result.as_ref()
                && Some(current_identity) == self.owned[replacement.stage].identity,
            EditIoError::Changed,
        )?;
        let original = Arc::clone(&replacement.original);
        let permissions = replacement.permissions.clone();
        let target = replacement.path.clone();
        let mut backup_index = replacement.backup;
        if !self.owned[backup_index].present {
            let path = self.owned[backup_index].path.clone();
            backup_index = self.create_owned(path, &original, &permissions)?;
            self.replacements[index].backup = backup_index;
        }
        let backup = &mut self.owned[backup_index];
        ensure(
            read_checked(
                &self.baseline.root,
                &backup.path,
                backup.identity.ok_or(EditIoError::Confinement)?,
                original.len(),
            )? == original.as_ref(),
            EditIoError::Changed,
        )?;
        ensure(
            identity(&validate_path(&self.baseline.root, &target)?)? == current_identity,
            EditIoError::Changed,
        )?;
        checkpoint("restore_before")?;
        io(fs::rename(
            self.baseline.root.join(&backup.path),
            self.baseline.root.join(&target),
        ))?;
        backup.present = false;
        checkpoint("restore_after")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Named matrix oracles enumerate complete boundary and ordinal tables.
    #![allow(clippy::too_many_lines)]
    use super::*;

    fn fixture() -> (tempfile::TempDir, EditIo) {
        faults::set(Vec::new());
        let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../local-artifacts/codex-runs/sprint-41/task-5/tmp");
        fs::create_dir_all(&parent).unwrap();
        let root = tempfile::tempdir_in(parent.canonicalize().unwrap()).unwrap();
        let source = include_bytes!(
            "../../../../adapters/designer-xml/tests/fixtures/sprint14_conformance/edt/src/CommonModules/DynamicSecurityOverridable/Module.bsl"
        );
        let result = String::from_utf8(source.to_vec())
            .unwrap()
            .replace("FillSecurityCollection", "ChangedSecurityCollection");
        let mut outputs = BTreeMap::new();
        for path in ["a.bsl", "b.bsl"] {
            fs::write(root.path().join(path), source).unwrap();
            outputs.insert(PathBuf::from(path), Arc::from(result.as_bytes()));
        }
        fs::write(root.path().join("sentinel"), b"untouched").unwrap();
        let baseline = EditBaseline::capture(&root.path().canonicalize().unwrap(), &[]).unwrap();
        let io = EditIo::new(baseline, outputs, 7).unwrap();
        (root, io)
    }

    #[test]
    fn controlled_unwind_io_ordinals_preserve_ownership() {
        fn route(io: &mut EditIo) -> Result<()> {
            io.stage_all()?;
            io.replace_checked(|| false)?;
            io.verify_results()?;
            io.cleanup_owned()?;
            io.verify_results()?;
            Ok(())
        }
        let (_root, mut control) = fixture();
        faults::set(Vec::new());
        route(&mut control).unwrap();
        let mutation_events = faults::events().len();
        control.restore_checked().unwrap();
        let trace = faults::events();
        let mut counts = BTreeMap::new();
        let ordinals: Vec<_> = trace
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let ordinal = counts.entry(*point).or_insert(0);
                *ordinal += 1;
                (index, *point, *ordinal)
            })
            .collect();
        for required in [
            "create",
            "created_metadata",
            "created_identity",
            "write",
            "permissions",
            "sync",
            "close_observation",
            "readback",
            "read",
            "replace_before",
            "replace_after",
            "cleanup",
            "cleanup_after",
            "restore_check",
            "restore_before",
            "restore_after",
            "restore_verify",
        ] {
            assert!(
                counts.contains_key(required),
                "missing real boundary {required}"
            );
        }
        assert_eq!(
            counts["create"], 6,
            "four staged entries and two recreated backups"
        );
        for reverse in [false, true] {
            for &(at, point, ordinal) in &ordinals {
                let (root, mut io) = fixture();
                if reverse {
                    io.stage_all().unwrap();
                    io.replace_checked(|| false).unwrap();
                    io.cleanup_owned().unwrap();
                    let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
                    io = EditIo::new(baseline, io.originals(), 8).unwrap();
                }
                let original = io.baseline.clone();
                faults::set(Vec::new());
                faults::unwind(vec![(point, ordinal)]);
                // Test retains the same production owner across each injected
                // boundary; production containment is independently exercised by E.
                let mutation =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| route(&mut io)));
                if at < mutation_events {
                    assert!(mutation.is_err(), "{point}/{ordinal}/{reverse}");
                } else {
                    mutation.unwrap().unwrap();
                }
                let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if io.attempted() {
                        io.restore_checked()
                    } else {
                        io.clean_unattempted()
                    }
                }));
                if at >= mutation_events {
                    assert!(recovered.is_err(), "recovery {point}/{ordinal}/{reverse}");
                }
                if recovered.is_ok_and(|result| result.is_ok()) {
                    assert!(original.equivalent(&EditBaseline::capture(root.path(), &[]).unwrap()));
                    assert_eq!(io.retained_files(), 0);
                }
                let present = fs::read_dir(root.path())
                    .unwrap()
                    .filter(|entry| {
                        entry
                            .as_ref()
                            .unwrap()
                            .file_name()
                            .to_string_lossy()
                            .starts_with(".oneagent-edit-")
                    })
                    .count();
                assert_eq!(
                    io.retained_files(),
                    present,
                    "exact ownership {point}/{ordinal}/{reverse}"
                );
                let events = faults::events();
                assert!(events.iter().filter(|event| **event == point).count() >= ordinal);
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
                faults::set(Vec::new());
            }
        }
    }

    #[test]
    fn scan_bounds_precede_retention() {
        {
            let root = tempfile::tempdir().unwrap();
            let mut relative = PathBuf::new();
            for _ in 0..128 {
                relative.push("d");
                fs::create_dir(root.path().join(&relative)).unwrap();
            }
            fs::write(root.path().join(&relative).join("leaf"), b"deep raw bytes").unwrap();
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            assert_eq!(baseline.entries.len(), 130);
            assert_eq!(baseline.raw_bytes(), 14);
            assert_eq!(
                baseline.bytes(&relative.join("leaf")).unwrap().as_ref(),
                b"deep raw bytes"
            );
            // Exercise the real recursive scanner with a nearly exhausted path
            // reservation: the first rejected joined path is never retained.
            let mut constrained = EditBaseline {
                root: baseline.root.clone(),
                entries: BTreeMap::new(),
                raw_bytes: 0,
            };
            let mut budget = EditIoBudget {
                paths: MAX_PATHS - 3,
                ..Default::default()
            };
            budget.entry(Path::new("")).unwrap();
            assert_eq!(
                constrained.scan(Path::new(""), &[], &mut budget),
                Err(EditIoError::Bounds)
            );
            assert_eq!(constrained.entries.len(), 2);
            assert!(!constrained.entries.contains_key(Path::new("d/d")));
            assert_eq!(budget.paths, MAX_PATHS - 2);
        }
        for ignored in [".git", ".oneagent", "node_modules", "target"] {
            let (root, _) = fixture();
            fs::create_dir(root.path().join(ignored)).unwrap();
            let path = root.path().join(ignored).join("unrecognized-source");
            fs::write(&path, b"original").unwrap();
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            assert_eq!(
                baseline
                    .bytes(&PathBuf::from(ignored).join("unrecognized-source"))
                    .unwrap()
                    .as_ref(),
                b"original"
            );
            fs::write(path, b"changed").unwrap();
            assert!(baseline.verify_tree(&BTreeMap::new(), &[]).is_err());
        }
        {
            let (root, _) = fixture();
            fs::create_dir_all(root.path().join(".oneagent/cache")).unwrap();
            let path = root.path().join(CACHE_PATH);
            fs::write(&path, b"cache").unwrap();
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            assert!(baseline.bytes(Path::new(CACHE_PATH)).is_err());
            fs::write(&path, b"different cache").unwrap();
            assert!(baseline.equals(&EditBaseline::capture(root.path(), &[]).unwrap()));
            fs::remove_file(&path).unwrap();
            fs::create_dir(path).unwrap();
            assert!(matches!(
                EditBaseline::capture(root.path(), &[]),
                Err(EditIoError::Confinement)
            ));
        }
        {
            let (root, _) = fixture();
            for name in ["a.bsl", "b.bsl", "sentinel"] {
                fs::remove_file(root.path().join(name)).unwrap();
            }
            for index in 0..MAX_BASELINE / MAX_FILE {
                fs::File::create(root.path().join(format!("raw-{index}")))
                    .unwrap()
                    .set_len(MAX_FILE as u64)
                    .unwrap();
            }
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            assert_eq!(baseline.raw_bytes(), MAX_BASELINE);
            assert_eq!(baseline.bytes(Path::new("raw-0")).unwrap().len(), MAX_FILE);
            drop(baseline);
            fs::write(root.path().join("raw-extra"), b"x").unwrap();
            assert!(matches!(
                EditBaseline::capture(root.path(), &[]),
                Err(EditIoError::Bounds)
            ));
        }
        let (root, io) = fixture();
        let target = root.path().join("a.bsl");
        let original = &io.replacements[0];
        faults::set(Vec::new());
        faults::action("read_filled", move || {
            use std::io::Write;
            OpenOptions::new()
                .append(true)
                .open(target)
                .unwrap()
                .write_all(b"x")
                .unwrap();
        });
        assert_eq!(
            read_checked(
                root.path(),
                Path::new("a.bsl"),
                original.original_identity,
                original.original.len()
            ),
            Err(EditIoError::Changed)
        );
        assert_eq!(
            faults::events()
                .iter()
                .filter(|event| **event == "read_filled")
                .count(),
            1
        );
        let (root, _) = fixture();
        for index in 0..MAX_ENTRIES - 4 {
            fs::write(root.path().join(format!("wide-{index}")), b"").unwrap();
        }
        let exact = EditBaseline::capture(root.path(), &[]).unwrap();
        assert_eq!(exact.entries.len(), MAX_ENTRIES);
        fs::write(root.path().join("one-over"), b"").unwrap();
        assert!(matches!(
            EditBaseline::capture(root.path(), &[]),
            Err(EditIoError::Bounds)
        ));
        let mut budget = EditIoBudget::default();
        for _ in 0..MAX_ENTRIES {
            budget.entry(Path::new("")).unwrap();
        }
        assert_eq!(budget.entry(Path::new("")), Err(EditIoError::Bounds));
        let mut budget = EditIoBudget::default();
        let exact = "a".repeat(MAX_PATH);
        for _ in 0..MAX_PATHS / MAX_PATH {
            budget.entry(Path::new(&exact)).unwrap();
        }
        assert_eq!(budget.entry(Path::new("a")), Err(EditIoError::Bounds));
        assert_eq!(budget.paths, MAX_PATHS);
        assert_eq!(budget.entries, MAX_PATHS / MAX_PATH);
        budget.paths = usize::MAX;
        assert_eq!(budget.entry_length(1), Err(EditIoError::Bounds));
        budget.paths = 0;
        budget.entries = usize::MAX;
        assert_eq!(budget.entry_length(0), Err(EditIoError::Bounds));
        assert_eq!(
            EditIoBudget::default().entry(Path::new(&"a".repeat(MAX_PATH + 1))),
            Err(EditIoError::Bounds)
        );
        let mut budget = EditIoBudget::default();
        for _ in 0..MAX_BASELINE / MAX_FILE {
            assert_eq!(budget.file(MAX_FILE as u64), Ok(MAX_FILE));
        }
        assert_eq!(budget.file(1), Err(EditIoError::Bounds));
        assert_eq!(
            EditIoBudget::default().file(MAX_FILE as u64 + 1),
            Err(EditIoError::Bounds)
        );
        budget.bytes = usize::MAX;
        assert_eq!(budget.file(1), Err(EditIoError::Bounds));
        let (root, io) = fixture();
        fs::write(root.path().join("a.bsl"), b"grown").unwrap();
        assert!(io.baseline.verify_tree(&BTreeMap::new(), &[]).is_err());
    }

    #[test]
    fn original_one_over_document_bound_rejects_admissible_result() {
        // SourceDocument constructors already reject this size; the private
        // filesystem owner can independently exercise its original-byte guard
        // without forging a semantic document or an executable planner result.
        for original_length in [MAX_DOCUMENT, MAX_DOCUMENT + 1] {
            let (root, _) = fixture();
            let path = PathBuf::from("a.bsl");
            let mut original = fs::read(root.path().join(&path)).unwrap();
            original.extend_from_slice(b"\n//");
            original.resize(original_length, b' ');
            let result = String::from_utf8(original.clone())
                .unwrap()
                .replace("FillSecurityCollection", "X")
                .into_bytes();
            assert_eq!(original.len(), original_length);
            assert!(result.len() < MAX_DOCUMENT);
            assert!(original.len() < MAX_EDITED && result.len() < MAX_EDITED);
            fs::write(root.path().join(&path), &original).unwrap();
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            assert!(baseline.raw_bytes() < MAX_BASELINE);
            assert_eq!(baseline.bytes(&path).unwrap().as_ref(), original);
            let results = BTreeMap::from([(path.clone(), Arc::from(result))]);
            assert_eq!(results.len(), 1);
            let admission = EditIo::admission(&baseline).unwrap();
            faults::set(Vec::new());
            let admitted = EditIo::with_admission(baseline.clone(), results, 42, admission);
            if original_length == MAX_DOCUMENT {
                let mut io = admitted.unwrap();
                io.stage_all().unwrap();
                assert_eq!(io.retained_files(), 2);
                assert!(!io.attempted());
                io.cleanup_owned().unwrap();
                assert_eq!(io.retained_files(), 0);
            } else {
                assert!(matches!(admitted, Err(EditIoError::Bounds)));
                assert!(
                    faults::events().is_empty(),
                    "no stage, write or replacement was entered"
                );
            }
            assert_eq!(fs::read(root.path().join(&path)).unwrap(), original);
            assert!(baseline.equals(&EditBaseline::capture(root.path(), &[]).unwrap()));
        }
    }

    #[test]
    fn buffer_and_disk_bounds_precede_allocation() {
        for (count, original_extra, result_extra) in
            [(64, 0, 0), (65, 0, 0), (64, 1, 0), (64, 0, 1)]
        {
            let (root, _) = fixture();
            let bytes = vec![b' '; MAX_EDITED / 64];
            let mut results = BTreeMap::new();
            for index in 0..count {
                let path = PathBuf::from(format!("bounded-{index}.bsl"));
                let mut original = bytes.clone();
                let mut result = bytes.clone();
                if index == 0 {
                    original.extend(std::iter::repeat_n(b' ', original_extra));
                    result.extend(std::iter::repeat_n(b' ', result_extra));
                }
                fs::write(root.path().join(&path), original).unwrap();
                results.insert(path, Arc::from(result));
            }
            let baseline =
                EditBaseline::capture(&root.path().canonicalize().unwrap(), &[]).unwrap();
            faults::set(Vec::new());
            let io = EditIo::new(baseline, results, 42);
            if count == 64 && original_extra + result_extra == 0 {
                let mut io = io.unwrap();
                io.stage_all().unwrap();
                assert_eq!(io.retained_files(), 128);
                assert_eq!(
                    io.replacements
                        .iter()
                        .map(|r| r.original.len() + r.result.len())
                        .sum::<usize>(),
                    16_777_216
                );
                io.cleanup_owned().unwrap();
                io.replacements[0].result = Arc::from(vec![b' '; bytes.len() + 1]);
                faults::set(Vec::new());
                assert_eq!(io.stage_all(), Err(EditIoError::Bounds));
                assert!(!faults::events().contains(&"create"));
            } else {
                assert!(matches!(io, Err(EditIoError::Bounds)));
                assert!(!faults::events().contains(&"create"));
            }
        }
        assert_eq!(EditIoBudget::buffers(&[MAX_BUFFERS]), Ok(()));
        assert_eq!(
            EditIoBudget::buffers(&[MAX_BUFFERS, 1]),
            Err(EditIoError::Bounds)
        );
        assert_eq!(
            EditIoBudget::buffers(&[usize::MAX, 1]),
            Err(EditIoError::Bounds)
        );
        let (_root, io) = fixture();
        let baseline = io.baseline.clone();
        let mut results = io.results();
        results.insert(
            PathBuf::from("a.bsl"),
            Arc::from(vec![b'a'; MAX_DOCUMENT + 1]),
        );
        assert!(matches!(
            EditIo::new(baseline, results, 8),
            Err(EditIoError::Bounds)
        ));
        assert!(!faults::events().contains(&"create"));
    }

    #[test]
    fn every_read_ordinal_fails_through_real_io_routes() {
        fn run(io: &mut EditIo, phase: usize) -> Result<()> {
            match phase {
                0 => io.stage_all(),
                1 => io.replace_checked(|| false),
                2 | 4 => io.verify_results().map(|_| ()),
                3 => io.restore_checked().map(|_| ()),
                _ => unreachable!(),
            }
        }
        fn prepared(phase: usize) -> (tempfile::TempDir, EditIo) {
            let (root, mut io) = fixture();
            if phase > 0 {
                io.stage_all().unwrap();
            }
            if phase > 1 {
                io.replace_checked(|| false).unwrap();
            }
            if phase >= 3 {
                io.cleanup_owned().unwrap();
            }
            faults::set(vec![]);
            (root, io)
        }
        let mut reached = 0;
        for phase in 0..5 {
            let (_root, mut io) = prepared(phase);
            run(&mut io, phase).unwrap();
            let events = faults::events();
            for point in ["read", "read_filled"] {
                let count = events.iter().filter(|event| **event == point).count();
                assert!(count > 0, "unobserved read route {phase}/{point}");
                for ordinal in 1..=count {
                    let (root, mut io) = prepared(phase);
                    faults::set(vec![(point, ordinal)]);
                    assert_eq!(
                        run(&mut io, phase),
                        Err(EditIoError::Io),
                        "{phase}/{point}/{ordinal}"
                    );
                    assert!(
                        faults::events()
                            .iter()
                            .filter(|event| **event == point)
                            .count()
                            >= ordinal
                    );
                    assert_eq!(
                        fs::read(root.path().join("sentinel")).unwrap(),
                        b"untouched"
                    );
                    assert!(io.retained_files() <= 4);
                    reached += 1;
                }
            }
        }
        eprintln!("safe-edit read ordinal cases: {reached}");
        faults::set(vec![]);
    }

    #[test]
    fn staging_faults_preserve_sources() {
        for bytes in [b"short".as_slice(), b"corrupt output".as_slice()] {
            let (root, mut io) = fixture();
            let path = root.path().join(".oneagent-edit-7-0-result");
            let bytes = bytes.to_vec();
            faults::action("readback", move || fs::write(path, bytes).unwrap());
            assert!(io.stage_all().is_err());
            assert!(!io.attempted());
            io.cleanup_owned().unwrap();
            assert!(
                io.baseline
                    .equals(&EditBaseline::capture(root.path(), &[]).unwrap())
            );
        }
        for point in [
            "create",
            "write",
            "permissions",
            "sync",
            "close_observation",
            "readback",
        ] {
            for ordinal in 1..=4 {
                let (_root, mut io) = fixture();
                let baseline = io.baseline.clone();
                faults::set(vec![(point, ordinal)]);
                assert!(io.stage_all().is_err(), "{point}/{ordinal}");
                assert!(!io.attempted());
                assert!(!faults::events().contains(&"replace_before"));
                faults::set(Vec::new());
                io.cleanup_owned().unwrap();
                assert!(baseline.equals(&EditBaseline::capture(baseline.root(), &[]).unwrap()));
                assert_eq!(io.retained_files(), 0);
            }
        }
        for (name, creates) in [("result", 1), ("backup", 2)] {
            let (root, previous) = fixture();
            let collision = root.path().join(format!(".oneagent-edit-7-0-{name}"));
            fs::write(&collision, b"unowned collision sentinel").unwrap();
            let identity_before = identity(&fs::metadata(&collision).unwrap()).unwrap();
            // The occupied name belongs to the complete before inventory, so
            // the staging baseline check must pass before create_new rejects it.
            let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
            let mut io = EditIo::new(baseline.clone(), previous.results(), 7).unwrap();
            faults::set(Vec::new());
            assert_eq!(io.stage_all(), Err(EditIoError::Io));
            assert_eq!(
                faults::events()
                    .iter()
                    .filter(|e| **e == "create_already_exists")
                    .count(),
                1
            );
            assert_eq!(
                faults::events().iter().filter(|e| **e == "create").count(),
                creates
            );
            assert_eq!(io.retained_files(), creates - 1);
            assert!(!io.attempted());
            assert!(!faults::events().contains(&"replace_before"));
            // One occupied-name attempt exhausts this algorithm: no retry,
            // alternate name, truncation or adoption of the unowned file.
            io.cleanup_owned().unwrap();
            assert_eq!(io.retained_files(), 0);
            assert_eq!(fs::read(&collision).unwrap(), b"unowned collision sentinel");
            assert_eq!(
                identity(&fs::metadata(&collision).unwrap()).unwrap(),
                identity_before
            );
            assert!(baseline.equals(&EditBaseline::capture(root.path(), &[]).unwrap()));
        }
    }

    #[test]
    fn created_file_identity_failures_retain_exact_ownership() {
        for point in ["created_metadata", "created_identity"] {
            for ordinal in 1..=4 {
                let (root, mut io) = fixture();
                faults::set(vec![(point, ordinal)]);
                assert_eq!(io.stage_all(), Err(EditIoError::Io));
                assert_eq!(io.retained_files(), ordinal);
                assert!(!io.attempted());
                assert!(!faults::events().contains(&"replace_before"));
                let unknown = &io.owned[ordinal - 1];
                assert!(unknown.identity.is_none());
                let path = root.path().join(&unknown.path);
                assert_eq!(fs::read(&path).unwrap(), b"");
                let observed_identity = identity(&fs::metadata(&path).unwrap()).unwrap();
                assert_eq!(io.cleanup_owned(), Err(EditIoError::Io));
                assert_eq!(io.retained_files(), 1);
                assert_eq!(fs::read(&path).unwrap(), b"");
                assert_eq!(
                    identity(&fs::metadata(&path).unwrap()).unwrap(),
                    observed_identity
                );
                assert!(
                    io.baseline
                        .verify_tree(&BTreeMap::new(), &io.owned)
                        .is_err()
                );
                for replacement in &io.replacements {
                    assert_eq!(
                        fs::read(root.path().join(&replacement.path)).unwrap(),
                        replacement.original.as_ref()
                    );
                }
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
                // Even a now-readable pathname cannot supply the missing
                // created identity, nor authorize deletion of a replacement.
                fs::rename(&path, root.path().join("saved-created-entry")).unwrap();
                fs::write(&path, b"unowned replacement").unwrap();
                assert_eq!(io.cleanup_owned(), Err(EditIoError::Io));
                assert_eq!(io.retained_files(), 1);
                assert_eq!(fs::read(&path).unwrap(), b"unowned replacement");
            }
            for ordinal in 1..=2 {
                let (root, mut io) = fixture();
                io.stage_all().unwrap();
                io.replace_checked(|| false).unwrap();
                io.cleanup_owned().unwrap();
                faults::set(vec![(point, ordinal)]);
                assert!(io.restore_checked().is_err());
                assert_eq!(
                    faults::events()
                        .iter()
                        .filter(|event| **event == point)
                        .count(),
                    2
                );
                assert_eq!(io.retained_files(), 1);
                let unknown = io.owned.iter().find(|entry| entry.present).unwrap();
                assert!(unknown.identity.is_none());
                assert_eq!(fs::read(root.path().join(&unknown.path)).unwrap(), b"");
                for (index, replacement) in io.replacements.iter().enumerate() {
                    let expected = if index == 2 - ordinal {
                        &replacement.result
                    } else {
                        &replacement.original
                    };
                    assert_eq!(
                        fs::read(root.path().join(&replacement.path)).unwrap(),
                        expected.as_ref()
                    );
                }
                assert_eq!(io.cleanup_owned(), Err(EditIoError::Io));
                assert_eq!(io.retained_files(), 1);
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
            }
        }
        // Positive controls cover both original staging and recreated backups.
        let (root, mut io) = fixture();
        io.stage_all().unwrap();
        assert_eq!(io.retained_files(), 4);
        assert!(io.owned.iter().all(|entry| entry.identity.is_some()));
        io.replace_checked(|| false).unwrap();
        io.cleanup_owned().unwrap();
        let restored = io.restore_checked().unwrap();
        assert!(io.baseline.equivalent(&restored));
        assert_eq!(io.retained_files(), 0);
        assert!(
            io.baseline
                .equivalent(&EditBaseline::capture(root.path(), &[]).unwrap())
        );
    }

    #[cfg(unix)]
    #[test]
    fn created_file_identity_rejection_never_adopts_an_alias() {
        let (root, mut io) = fixture();
        let path = root.path().join(".oneagent-edit-7-0-result");
        let alias = root.path().join("unowned-alias");
        let created = path.clone();
        let linked = alias.clone();
        faults::action("created_metadata", move || {
            fs::hard_link(created, linked).unwrap();
        });
        assert_eq!(io.stage_all(), Err(EditIoError::Confinement));
        assert_eq!(io.retained_files(), 1);
        assert!(io.owned[0].identity.is_none());
        assert!(!io.attempted());
        assert_eq!(io.cleanup_owned(), Err(EditIoError::Io));
        assert_eq!(fs::read(&path).unwrap(), b"");
        assert_eq!(fs::read(&alias).unwrap(), b"");
        fs::remove_file(alias).unwrap();
        assert_eq!(io.cleanup_owned(), Err(EditIoError::Io));
        assert_eq!(io.retained_files(), 1);
        assert!(path.exists());
    }

    #[test]
    fn replacement_ordinals_classify_ambiguous_failure() {
        for point in ["replace_before", "replace_after"] {
            for ordinal in 1..=2 {
                let (_root, mut io) = fixture();
                io.stage_all().unwrap();
                faults::set(vec![(point, ordinal)]);
                assert!(io.replace_checked(|| false).is_err());
                faults::set(Vec::new());
                let restored = io.restore_checked().unwrap();
                assert!(io.baseline.equivalent(&restored));
                assert_eq!(io.retained_files(), 0);
            }
        }
        let (root, mut io) = fixture();
        io.stage_all().unwrap();
        let calls = std::cell::Cell::new(0);
        assert!(
            io.replace_checked(|| {
                calls.set(calls.get() + 1);
                if calls.get() == 2 {
                    fs::write(root.path().join("b.bsl"), b"external edit").unwrap();
                }
                false
            })
            .is_err()
        );
        assert_eq!(calls.get(), 2);
        assert!(io.attempted());
        assert!(io.restore_checked().is_err());
        assert_eq!(
            fs::read(root.path().join("b.bsl")).unwrap(),
            b"external edit"
        );
        assert_eq!(
            fs::read(root.path().join("a.bsl")).unwrap(),
            io.replacements[0].original.as_ref()
        );
    }

    #[test]
    fn recovery_ordinals_preserve_unrelated_edits() {
        for point in [
            "create",
            "write",
            "permissions",
            "sync",
            "close_observation",
            "readback",
        ] {
            for ordinal in 1..=2 {
                let (root, mut io) = fixture();
                io.stage_all().unwrap();
                io.replace_checked(|| false).unwrap();
                io.cleanup_owned().unwrap();
                faults::set(vec![(point, ordinal)]);
                assert!(
                    io.restore_checked().is_err(),
                    "recreated backup {point}/{ordinal}"
                );
                assert!(
                    faults::events()
                        .iter()
                        .filter(|event| **event == point)
                        .count()
                        >= ordinal
                );
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
                assert!(io.retained_files() <= 4);
                faults::set(Vec::new());
            }
        }
        for point in [
            "restore_check",
            "restore_before",
            "restore_after",
            "cleanup",
        ] {
            for ordinal in 1..=2 {
                let (root, mut io) = fixture();
                io.stage_all().unwrap();
                if point == "cleanup" {
                    faults::set(vec![("replace_after", 1)]);
                    assert!(io.replace_checked(|| false).is_err());
                } else {
                    io.replace_checked(|| false).unwrap();
                }
                faults::set(vec![(point, ordinal)]);
                let result = io.restore_checked();
                assert!(
                    faults::events()
                        .iter()
                        .filter(|event| **event == point)
                        .count()
                        >= ordinal,
                    "unreached {point}/{ordinal}"
                );
                assert!(result.is_err(), "{point}/{ordinal}");
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
                faults::set(Vec::new());
            }
        }
        let (root, mut io) = fixture();
        io.stage_all().unwrap();
        io.replace_checked(|| false).unwrap();
        fs::write(root.path().join("a.bsl"), b"external change").unwrap();
        assert!(io.restore_checked().is_err());
        assert_eq!(
            fs::read(root.path().join("a.bsl")).unwrap(),
            b"external change"
        );
    }

    #[cfg(unix)]
    #[test]
    fn confinement_rechecked_at_every_io_boundary() {
        use std::os::unix::fs::symlink;
        for boundary in ["scan", "stage", "replace", "restore", "cleanup"] {
            for swap in [
                "directory",
                "hard_link",
                "ancestor",
                "root",
                "owned_identity",
            ] {
                let (root, _) = fixture();
                let nested = root.path().join("nested");
                fs::create_dir(&nested).unwrap();
                fs::rename(root.path().join("a.bsl"), nested.join("a.bsl")).unwrap();
                let baseline = EditBaseline::capture(root.path(), &[]).unwrap();
                let mut io = EditIo::new(
                    baseline,
                    BTreeMap::from([(PathBuf::from("nested/a.bsl"), Arc::from(&b"new-a"[..]))]),
                    17,
                )
                .unwrap();
                if !matches!(boundary, "scan" | "stage") {
                    io.stage_all().unwrap();
                }
                if boundary == "restore" {
                    io.replace_checked(|| false).unwrap();
                }
                let target = if boundary == "cleanup" {
                    root.path()
                        .join(&io.owned.iter().find(|o| o.present).unwrap().path)
                } else {
                    nested.join("a.bsl")
                };
                let saved_root = root.path().with_extension("saved-root");
                match swap {
                    "ancestor" => {
                        fs::rename(&nested, root.path().join("saved-nested")).unwrap();
                        symlink(root.path().join("saved-nested"), &nested).unwrap();
                    }
                    "root" => {
                        fs::rename(root.path(), &saved_root).unwrap();
                        symlink(&saved_root, root.path()).unwrap();
                    }
                    "directory" => {
                        fs::remove_file(&target).unwrap();
                        fs::create_dir(&target).unwrap();
                    }
                    "hard_link" => {
                        fs::hard_link(&target, root.path().join("unrelated-alias")).unwrap();
                    }
                    "owned_identity" => {
                        // Preserve the previous inode at another name, preventing
                        // allocator reuse from making the substitution vacuous.
                        fs::rename(&target, root.path().join("saved-entry")).unwrap();
                        fs::write(&target, b"third-party-secret").unwrap();
                    }
                    _ => unreachable!(),
                }
                let result = match boundary {
                    "scan" => EditBaseline::capture(root.path(), &[]).and_then(|actual| {
                        ensure(io.baseline.equals(&actual), EditIoError::Changed)
                    }),
                    "stage" => io.stage_all(),
                    "replace" => io.replace_checked(|| false),
                    "restore" => io.restore_checked().map(|_| ()),
                    "cleanup" => io.cleanup_owned(),
                    _ => unreachable!(),
                };
                assert!(result.is_err(), "{boundary}/{swap}");
                if swap == "root" {
                    fs::remove_file(root.path()).unwrap();
                    fs::rename(&saved_root, root.path()).unwrap();
                }
                assert_eq!(
                    fs::read(root.path().join("sentinel")).unwrap(),
                    b"untouched"
                );
                if swap == "owned_identity" {
                    assert_eq!(fs::read(target).unwrap(), b"third-party-secret");
                }
            }
        }
        for boundary in ["scan", "stage", "replace", "restore", "cleanup"] {
            let (root, mut io) = fixture();
            let outside = root.path().join("sentinel");
            if boundary != "scan" && boundary != "stage" {
                io.stage_all().unwrap();
            }
            if boundary == "restore" {
                io.replace_checked(|| false).unwrap();
            }
            let path = if boundary == "cleanup" {
                root.path().join(&io.owned[0].path)
            } else {
                root.path().join("a.bsl")
            };
            fs::remove_file(&path).unwrap();
            symlink(&outside, &path).unwrap();
            let rejected = match boundary {
                "scan" => EditBaseline::capture(root.path(), &[]).is_err(),
                "stage" => io.stage_all().is_err(),
                "replace" => io.replace_checked(|| false).is_err(),
                "restore" => io.restore_checked().is_err(),
                "cleanup" => io.cleanup_owned().is_err(),
                _ => unreachable!(),
            };
            assert!(rejected, "{boundary}");
            assert_eq!(fs::read(outside).unwrap(), b"untouched");
        }
        let (root, mut io) = fixture();
        assert!(validate_path(root.path(), Path::new("../escape")).is_err());
        fs::hard_link(root.path().join("a.bsl"), root.path().join("alias")).unwrap();
        assert!(EditBaseline::capture(root.path(), &[]).is_err());
        fs::remove_file(root.path().join("alias")).unwrap();
        io.stage_all().unwrap();
        fs::remove_file(root.path().join("a.bsl")).unwrap();
        symlink(root.path().join("sentinel"), root.path().join("a.bsl")).unwrap();
        assert!(io.replace_checked(|| false).is_err());
        io.cleanup_owned().unwrap();
        assert_eq!(
            fs::read(root.path().join("sentinel")).unwrap(),
            b"untouched"
        );
    }
}
