//! Reader for module files inside EDT metadata objects.

use oneagent_analysis::refactoring::{
    MAX_SOURCE_DOCUMENT_BYTES, SourceEvidenceAdmission, SourceEvidenceError,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Supported EDT module file kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtModuleKind {
    /// Object module.
    Object,
    /// Manager module.
    Manager,
    /// Common module implementation.
    Common,
    /// Subordinate form module implementation.
    Form,
    /// Common or subordinate command module implementation.
    Command,
}

impl EdtModuleKind {
    /// Returns a stable machine-readable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Object => "object_module",
            Self::Manager => "manager_module",
            Self::Common => "common_module",
            Self::Form => "form_module",
            Self::Command => "command_module",
        }
    }
}

/// Canonical owner family for a Form or Command module layout observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtModuleOwnerKind {
    /// Form declared inside a metadata object.
    Form,
    /// Command declared inside a metadata object.
    Command,
    /// Top-level Common Command metadata object.
    CommonCommand,
}

/// Typed result category for one inspected Form or Command module layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtModuleLayoutOutcomeKind {
    /// The exact accepted module artifact was found and read.
    Accepted,
    /// The optional module artifact is absent.
    Missing,
    /// The layout was rejected without synthesizing an owner.
    Rejected(EdtModuleLayoutRejectionReason),
}

/// Deterministic reason for rejecting a Form or Command module layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtModuleLayoutRejectionReason {
    /// A module directory has no matching parsed owner declaration.
    OrphanDirectory,
    /// A directory differs from the exact owner source name.
    NameMismatch,
    /// More than one parsed declaration claims the same source name and kind.
    DuplicateOwner,
    /// The directory role conflicts with the matching parsed owner kind.
    WrongOwnerKind,
    /// A known directory contains the wrong module file role.
    UnsupportedLayout,
}

/// Owner-aware parser observation for one accepted or rejected module layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtModuleLayoutObservation {
    owner_id: Option<EntityId>,
    owner_name: Option<EntityName>,
    owner_kind: Option<EdtModuleOwnerKind>,
    path: PathBuf,
    outcome: EdtModuleLayoutOutcomeKind,
    module: Option<EdtModuleDescriptor>,
}

impl EdtModuleLayoutObservation {
    /// Returns the canonical owner identifier when a parsed owner was matched.
    #[must_use]
    pub const fn owner_id(&self) -> Option<&EntityId> {
        self.owner_id.as_ref()
    }

    /// Returns the canonical owner name when a parsed owner was matched.
    #[must_use]
    pub const fn owner_name(&self) -> Option<&EntityName> {
        self.owner_name.as_ref()
    }

    /// Returns the accepted owner family when one was matched.
    #[must_use]
    pub const fn owner_kind(&self) -> Option<EdtModuleOwnerKind> {
        self.owner_kind
    }

    /// Returns the inspected source path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the typed observation outcome.
    #[must_use]
    pub const fn outcome(&self) -> EdtModuleLayoutOutcomeKind {
        self.outcome
    }

    /// Returns the accepted module descriptor, when present.
    #[must_use]
    pub const fn module(&self) -> Option<&EdtModuleDescriptor> {
        self.module.as_ref()
    }
}

/// Parsed EDT module descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtModuleDescriptor {
    id: EntityId,
    name: EntityName,
    kind: EdtModuleKind,
    path: PathBuf,
    raw_source: Option<Vec<u8>>,
}

impl EdtModuleDescriptor {
    /// Creates a module descriptor.
    #[must_use]
    pub const fn new(id: EntityId, name: EntityName, kind: EdtModuleKind, path: PathBuf) -> Self {
        Self {
            id,
            name,
            kind,
            path,
            raw_source: None,
        }
    }

    pub(crate) const fn new_with_raw_source(
        id: EntityId,
        name: EntityName,
        kind: EdtModuleKind,
        path: PathBuf,
        raw_source: Vec<u8>,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            path,
            raw_source: Some(raw_source),
        }
    }

    /// Returns the stable module identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the module name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the module kind.
    #[must_use]
    pub const fn kind(&self) -> EdtModuleKind {
        self.kind
    }

    /// Returns the source file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns exact bytes retained by production discovery.
    #[must_use]
    pub fn raw_source(&self) -> Option<&[u8]> {
        self.raw_source.as_deref()
    }
}

/// Discovers module files in an EDT metadata object directory.
pub trait EdtModuleReader {
    /// Reads known module files.
    ///
    /// # Errors
    ///
    /// Returns an error when a discovered module file cannot be read.
    fn read_modules(
        &self,
        object_id: &EntityId,
        object_name: &EntityName,
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError>;

    /// Reads owner-aware subordinate Form, subordinate Command, and Common
    /// Command module layout observations without emitting graph facts.
    ///
    /// # Errors
    ///
    /// Returns an error when a discovered directory or module file cannot be
    /// read. Missing optional modules and rejected layouts are returned as
    /// typed observations.
    fn read_form_command_modules(
        &self,
        object: &crate::EdtMetadataObjectDescriptor,
        children: &[crate::EdtMetadataChildDescriptor],
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleLayoutObservation>, EdtModuleError>;
}

/// Filesystem implementation of [`EdtModuleReader`].
#[derive(Debug, Default)]
pub struct FileSystemEdtModuleReader;

#[derive(Debug)]
pub(crate) struct EdtSourceManifest {
    admitted_metadata: BTreeMap<PathBuf, fs::Metadata>,
}

impl EdtSourceManifest {
    pub(crate) fn from_paths(mut paths: Vec<PathBuf>) -> Result<Self, EdtModuleError> {
        paths.sort();
        if let Some(duplicate) = paths.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(EdtModuleError::DuplicateAdmissionPath(duplicate[0].clone()));
        }
        let mut admission = SourceEvidenceAdmission::new(paths.len())?;
        let mut admitted_metadata = BTreeMap::new();
        for path in paths {
            let metadata = inspect_optional_module(&path)?
                .ok_or_else(|| EdtModuleError::ChangedDuringCapture(path.clone()))?;
            let length = admitted_length(&path, &metadata)?;
            admission.admit_document(length)?;
            admitted_metadata.insert(path, metadata);
        }
        admission.finish()?;
        Ok(Self { admitted_metadata })
    }

    pub(crate) fn finish(self) -> Result<(), EdtModuleError> {
        self.admitted_metadata
            .into_keys()
            .next()
            .map_or(Ok(()), |path| {
                Err(EdtModuleError::ChangedDuringCapture(path))
            })
    }

    fn take_metadata(&mut self, path: &Path) -> Result<fs::Metadata, EdtModuleError> {
        self.admitted_metadata
            .remove(path)
            .ok_or_else(|| EdtModuleError::ChangedDuringCapture(path.to_path_buf()))
    }
}

enum SourceCapture<'a> {
    Unbounded,
    Discover(&'a mut Vec<PathBuf>),
    Manifest(&'a mut EdtSourceManifest),
}

enum RetainedSource {
    Discovered,
    Captured(Vec<u8>),
}

impl EdtModuleReader for FileSystemEdtModuleReader {
    fn read_modules(
        &self,
        object_id: &EntityId,
        object_name: &EntityName,
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError> {
        Self::read_modules_internal(
            object_id,
            object_name,
            object_directory,
            &mut SourceCapture::Unbounded,
        )
    }

    fn read_form_command_modules(
        &self,
        object: &crate::EdtMetadataObjectDescriptor,
        children: &[crate::EdtMetadataChildDescriptor],
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleLayoutObservation>, EdtModuleError> {
        Self::read_form_command_modules_internal(
            object,
            children,
            object_directory,
            &mut SourceCapture::Unbounded,
        )
    }
}

impl FileSystemEdtModuleReader {
    pub(crate) fn discover_modules(
        object_id: &EntityId,
        object_name: &EntityName,
        object_directory: &Path,
        paths: &mut Vec<PathBuf>,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError> {
        Self::read_modules_internal(
            object_id,
            object_name,
            object_directory,
            &mut SourceCapture::Discover(paths),
        )
    }

    pub(crate) fn discover_form_command_modules(
        object: &crate::EdtMetadataObjectDescriptor,
        children: &[crate::EdtMetadataChildDescriptor],
        object_directory: &Path,
        paths: &mut Vec<PathBuf>,
    ) -> Result<Vec<EdtModuleLayoutObservation>, EdtModuleError> {
        Self::read_form_command_modules_internal(
            object,
            children,
            object_directory,
            &mut SourceCapture::Discover(paths),
        )
    }

    pub(crate) fn read_modules_with_manifest(
        object_id: &EntityId,
        object_name: &EntityName,
        object_directory: &Path,
        manifest: &mut EdtSourceManifest,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError> {
        Self::read_modules_internal(
            object_id,
            object_name,
            object_directory,
            &mut SourceCapture::Manifest(manifest),
        )
    }

    pub(crate) fn read_form_command_modules_with_manifest(
        object: &crate::EdtMetadataObjectDescriptor,
        children: &[crate::EdtMetadataChildDescriptor],
        object_directory: &Path,
        manifest: &mut EdtSourceManifest,
    ) -> Result<Vec<EdtModuleLayoutObservation>, EdtModuleError> {
        Self::read_form_command_modules_internal(
            object,
            children,
            object_directory,
            &mut SourceCapture::Manifest(manifest),
        )
    }

    fn read_modules_internal(
        object_id: &EntityId,
        object_name: &EntityName,
        object_directory: &Path,
        capture: &mut SourceCapture<'_>,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError> {
        let candidates = [
            ("ObjectModule.bsl", EdtModuleKind::Object),
            ("ManagerModule.bsl", EdtModuleKind::Manager),
            ("Module.bsl", EdtModuleKind::Common),
        ];

        let mut modules = Vec::new();

        for (file_name, kind) in candidates {
            let path = object_directory.join(file_name);
            let Some(raw_source) = read_optional_module(&path, capture)? else {
                continue;
            };

            let id = EntityId::new(format!("{}:{}", object_id.as_str(), kind.as_str()))
                .map_err(|_| EdtModuleError::InvalidIdentifier)?;

            let name = match kind {
                EdtModuleKind::Object => {
                    EntityName::new("ObjectModule").map_err(|_| EdtModuleError::InvalidName)?
                }
                EdtModuleKind::Manager => {
                    EntityName::new("ManagerModule").map_err(|_| EdtModuleError::InvalidName)?
                }
                EdtModuleKind::Common => object_name.clone(),
                EdtModuleKind::Form | EdtModuleKind::Command => {
                    return Err(EdtModuleError::InvalidName);
                }
            };

            modules.push(match raw_source {
                RetainedSource::Captured(raw_source) => {
                    EdtModuleDescriptor::new_with_raw_source(id, name, kind, path, raw_source)
                }
                RetainedSource::Discovered => EdtModuleDescriptor::new(id, name, kind, path),
            });
        }

        Ok(modules)
    }

    fn read_form_command_modules_internal(
        object: &crate::EdtMetadataObjectDescriptor,
        children: &[crate::EdtMetadataChildDescriptor],
        object_directory: &Path,
        capture: &mut SourceCapture<'_>,
    ) -> Result<Vec<EdtModuleLayoutObservation>, EdtModuleError> {
        let mut observations = Vec::new();

        collect_child_layouts(
            object_directory,
            "Forms",
            "Module.bsl",
            "CommandModule.bsl",
            crate::EdtMetadataChildKind::Form,
            EdtModuleOwnerKind::Form,
            EdtModuleKind::Form,
            "FormModule",
            children,
            &mut observations,
            capture,
        )?;
        collect_child_layouts(
            object_directory,
            "Commands",
            "CommandModule.bsl",
            "Module.bsl",
            crate::EdtMetadataChildKind::Command,
            EdtModuleOwnerKind::Command,
            EdtModuleKind::Command,
            "CommandModule",
            children,
            &mut observations,
            capture,
        )?;

        collect_common_command_layout(object, object_directory, &mut observations, capture)?;
        sort_observations(&mut observations);
        Ok(observations)
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_child_layouts(
    object_directory: &Path,
    directory_name: &str,
    expected_file_name: &str,
    alternate_file_name: &str,
    expected_child_kind: crate::EdtMetadataChildKind,
    owner_kind: EdtModuleOwnerKind,
    module_kind: EdtModuleKind,
    module_name: &str,
    children: &[crate::EdtMetadataChildDescriptor],
    observations: &mut Vec<EdtModuleLayoutObservation>,
    capture: &mut SourceCapture<'_>,
) -> Result<(), EdtModuleError> {
    let root = object_directory.join(directory_name);
    let directories = direct_directories(&root)?;
    let mut declarations = BTreeMap::<String, Vec<&crate::EdtMetadataChildDescriptor>>::new();
    let mut opposite_names = BTreeSet::new();

    for child in children {
        if child.kind() == expected_child_kind {
            declarations
                .entry(child.name().as_str().to_owned())
                .or_default()
                .push(child);
        } else if matches!(
            (expected_child_kind, child.kind()),
            (
                crate::EdtMetadataChildKind::Form,
                crate::EdtMetadataChildKind::Command
            ) | (
                crate::EdtMetadataChildKind::Command,
                crate::EdtMetadataChildKind::Form
            )
        ) {
            opposite_names.insert(child.name().as_str().to_owned());
        }
    }

    let mut handled_directories = BTreeSet::new();
    for (name, owners) in &declarations {
        let expected_directory = root.join(name);
        if owners.len() > 1 {
            observations.push(rejected_observation(
                None,
                owner_kind,
                expected_directory,
                EdtModuleLayoutRejectionReason::DuplicateOwner,
            ));
            continue;
        }
        let owner = owners[0];
        let exact = directories.iter().find(|(actual, _)| actual == name);
        let mismatched = directories
            .iter()
            .find(|(actual, _)| actual.eq_ignore_ascii_case(name) && actual != name);
        let Some((_, directory)) = exact else {
            if let Some((_, directory)) = mismatched {
                handled_directories.insert(directory.clone());
                observations.push(rejected_observation(
                    Some(owner),
                    owner_kind,
                    directory.clone(),
                    EdtModuleLayoutRejectionReason::NameMismatch,
                ));
            } else {
                observations.push(missing_observation(
                    owner,
                    owner_kind,
                    expected_directory.join(expected_file_name),
                ));
            }
            continue;
        };

        handled_directories.insert(directory.clone());
        observations.push(classify_owned_layout(
            owner.id(),
            owner.name(),
            owner_kind,
            module_kind,
            module_name,
            directory,
            expected_file_name,
            alternate_file_name,
            capture,
        )?);
    }

    for (name, directory) in directories {
        if handled_directories.contains(&directory) || declarations.contains_key(&name) {
            continue;
        }
        let reason = if opposite_names.contains(&name) {
            EdtModuleLayoutRejectionReason::WrongOwnerKind
        } else {
            EdtModuleLayoutRejectionReason::OrphanDirectory
        };
        observations.push(rejected_observation(None, owner_kind, directory, reason));
    }

    Ok(())
}

fn collect_common_command_layout(
    object: &crate::EdtMetadataObjectDescriptor,
    object_directory: &Path,
    observations: &mut Vec<EdtModuleLayoutObservation>,
    capture: &mut SourceCapture<'_>,
) -> Result<(), EdtModuleError> {
    let command_path = object_directory.join("CommandModule.bsl");
    if object.kind() != MetadataKind::Command {
        if command_path.exists() {
            observations.push(rejected_observation(
                None,
                EdtModuleOwnerKind::CommonCommand,
                command_path,
                EdtModuleLayoutRejectionReason::WrongOwnerKind,
            ));
        }
        return Ok(());
    }

    let directory_name = object_directory
        .file_name()
        .map(|name| name.to_string_lossy());
    if directory_name.as_deref() != Some(object.name().as_str()) {
        observations.push(rejected_observation(
            None,
            EdtModuleOwnerKind::CommonCommand,
            object_directory.to_path_buf(),
            EdtModuleLayoutRejectionReason::NameMismatch,
        ));
        return Ok(());
    }

    observations.push(classify_owned_layout(
        object.id(),
        object.name(),
        EdtModuleOwnerKind::CommonCommand,
        EdtModuleKind::Command,
        "CommandModule",
        object_directory,
        "CommandModule.bsl",
        "Module.bsl",
        capture,
    )?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn classify_owned_layout(
    owner_id: &EntityId,
    owner_name: &EntityName,
    owner_kind: EdtModuleOwnerKind,
    module_kind: EdtModuleKind,
    module_name: &str,
    directory: &Path,
    expected_file_name: &str,
    alternate_file_name: &str,
    capture: &mut SourceCapture<'_>,
) -> Result<EdtModuleLayoutObservation, EdtModuleError> {
    let path = directory.join(expected_file_name);
    if let Some(raw_source) = read_optional_module(&path, capture)? {
        let id = EntityId::new(format!("{}:{}", owner_id.as_str(), module_kind.as_str()))
            .map_err(|_| EdtModuleError::InvalidIdentifier)?;
        let name = EntityName::new(module_name).map_err(|_| EdtModuleError::InvalidName)?;
        let module = match raw_source {
            RetainedSource::Captured(raw_source) => EdtModuleDescriptor::new_with_raw_source(
                id,
                name,
                module_kind,
                path.clone(),
                raw_source,
            ),
            RetainedSource::Discovered => {
                EdtModuleDescriptor::new(id, name, module_kind, path.clone())
            }
        };
        return Ok(EdtModuleLayoutObservation {
            owner_id: Some(owner_id.clone()),
            owner_name: Some(owner_name.clone()),
            owner_kind: Some(owner_kind),
            path,
            outcome: EdtModuleLayoutOutcomeKind::Accepted,
            module: Some(module),
        });
    }

    if directory.join(alternate_file_name).exists() {
        return Ok(EdtModuleLayoutObservation {
            owner_id: Some(owner_id.clone()),
            owner_name: Some(owner_name.clone()),
            owner_kind: Some(owner_kind),
            path: directory.to_path_buf(),
            outcome: EdtModuleLayoutOutcomeKind::Rejected(
                EdtModuleLayoutRejectionReason::UnsupportedLayout,
            ),
            module: None,
        });
    }

    Ok(EdtModuleLayoutObservation {
        owner_id: Some(owner_id.clone()),
        owner_name: Some(owner_name.clone()),
        owner_kind: Some(owner_kind),
        path,
        outcome: EdtModuleLayoutOutcomeKind::Missing,
        module: None,
    })
}

fn missing_observation(
    owner: &crate::EdtMetadataChildDescriptor,
    owner_kind: EdtModuleOwnerKind,
    path: PathBuf,
) -> EdtModuleLayoutObservation {
    EdtModuleLayoutObservation {
        owner_id: Some(owner.id().clone()),
        owner_name: Some(owner.name().clone()),
        owner_kind: Some(owner_kind),
        path,
        outcome: EdtModuleLayoutOutcomeKind::Missing,
        module: None,
    }
}

fn rejected_observation(
    owner: Option<&crate::EdtMetadataChildDescriptor>,
    owner_kind: EdtModuleOwnerKind,
    path: PathBuf,
    reason: EdtModuleLayoutRejectionReason,
) -> EdtModuleLayoutObservation {
    EdtModuleLayoutObservation {
        owner_id: owner.map(|owner| owner.id().clone()),
        owner_name: owner.map(|owner| owner.name().clone()),
        owner_kind: owner.map(|_| owner_kind),
        path,
        outcome: EdtModuleLayoutOutcomeKind::Rejected(reason),
        module: None,
    }
}

fn direct_directories(root: &Path) -> Result<Vec<(String, PathBuf)>, EdtModuleError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(root).map_err(|source| EdtModuleError::ReadDirectory {
        path: root.to_path_buf(),
        source,
    })?;
    let mut directories = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| EdtModuleError::ReadDirectory {
            path: root.to_path_buf(),
            source,
        })?;
        if entry
            .file_type()
            .map_err(|source| EdtModuleError::ReadDirectory {
                path: root.to_path_buf(),
                source,
            })?
            .is_dir()
        {
            directories.push((
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            ));
        }
    }
    directories.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    Ok(directories)
}

fn sort_observations(observations: &mut [EdtModuleLayoutObservation]) {
    observations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.outcome.cmp(&right.outcome))
            .then(left.owner_id.cmp(&right.owner_id))
    });
}

fn read_optional_module(
    path: &Path,
    capture: &mut SourceCapture<'_>,
) -> Result<Option<RetainedSource>, EdtModuleError> {
    let Some(path_metadata) = inspect_optional_module(path)? else {
        return Ok(None);
    };
    if let SourceCapture::Discover(paths) = capture {
        paths.push(path.to_path_buf());
        return Ok(Some(RetainedSource::Discovered));
    }
    let admitted_metadata = match capture {
        SourceCapture::Manifest(manifest) => Some(manifest.take_metadata(path)?),
        SourceCapture::Unbounded => None,
        SourceCapture::Discover(_) => unreachable!("discovery returned before capture"),
    };
    if admitted_metadata.is_some() {
        admitted_length(path, &path_metadata)?;
    }
    if admitted_metadata
        .as_ref()
        .is_some_and(|admitted| source_metadata_changed(admitted, &path_metadata))
    {
        return Err(EdtModuleError::ChangedDuringCapture(path.to_path_buf()));
    }
    let expected_length = admitted_metadata
        .as_ref()
        .map_or(path_metadata.len(), fs::Metadata::len);

    let mut file = fs::File::open(path).map_err(|source| EdtModuleError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let before = file.metadata().map_err(|source| EdtModuleError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    if source_metadata_changed(&path_metadata, &before)
        || admitted_metadata
            .as_ref()
            .is_some_and(|admitted| source_metadata_changed(admitted, &before))
        || before.len() != expected_length
    {
        return Err(EdtModuleError::ChangedDuringCapture(path.to_path_buf()));
    }
    let capacity =
        usize::try_from(before.len()).map_err(|_| EdtModuleError::SourceBoundExceeded {
            path: path.to_path_buf(),
            actual: before.len(),
            maximum: MAX_SOURCE_DOCUMENT_BYTES,
        })?;
    let mut raw_source = Vec::with_capacity(capacity);
    file.by_ref()
        .take(if admitted_metadata.is_some() {
            MAX_SOURCE_DOCUMENT_BYTES as u64 + 1
        } else {
            u64::MAX
        })
        .read_to_end(&mut raw_source)
        .map_err(|source| EdtModuleError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
    let captured_len = u64::try_from(raw_source.len()).unwrap_or(u64::MAX);
    if admitted_metadata.is_some() && raw_source.len() > MAX_SOURCE_DOCUMENT_BYTES {
        return Err(EdtModuleError::SourceBoundExceeded {
            path: path.to_path_buf(),
            actual: captured_len,
            maximum: MAX_SOURCE_DOCUMENT_BYTES,
        });
    }
    let after = fs::symlink_metadata(path).map_err(|source| EdtModuleError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    if source_metadata_changed(&before, &after) || captured_len != expected_length {
        return Err(EdtModuleError::ChangedDuringCapture(path.to_path_buf()));
    }
    std::str::from_utf8(&raw_source).map_err(|source| EdtModuleError::ReadFile {
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
    })?;
    Ok(Some(RetainedSource::Captured(raw_source)))
}

fn inspect_optional_module(path: &Path) -> Result<Option<fs::Metadata>, EdtModuleError> {
    let path_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(EdtModuleError::ReadFile {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if path_metadata.file_type().is_symlink() {
        return Err(EdtModuleError::SymlinkArtifact(path.to_path_buf()));
    }
    if !path_metadata.file_type().is_file() {
        return Err(EdtModuleError::ArtifactNotRegularFile(path.to_path_buf()));
    }
    Ok(Some(path_metadata))
}

fn admitted_length(path: &Path, metadata: &fs::Metadata) -> Result<usize, EdtModuleError> {
    if metadata.len() > MAX_SOURCE_DOCUMENT_BYTES as u64 {
        return Err(EdtModuleError::SourceBoundExceeded {
            path: path.to_path_buf(),
            actual: metadata.len(),
            maximum: MAX_SOURCE_DOCUMENT_BYTES,
        });
    }
    usize::try_from(metadata.len()).map_err(|_| EdtModuleError::SourceBoundExceeded {
        path: path.to_path_buf(),
        actual: metadata.len(),
        maximum: MAX_SOURCE_DOCUMENT_BYTES,
    })
}

fn source_metadata_changed(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    let changed = !after.file_type().is_file()
        || after.file_type().is_symlink()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        changed || before.dev() != after.dev() || before.ino() != after.ino()
    }
    #[cfg(not(unix))]
    {
        changed
    }
}

/// Error produced while reading EDT modules.
#[derive(Debug)]
pub enum EdtModuleError {
    /// A module layout directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A module file could not be read.
    ReadFile {
        /// Module path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A known module path is a symlink.
    SymlinkArtifact(PathBuf),
    /// A known module path is not a regular file.
    ArtifactNotRegularFile(PathBuf),
    /// A module exceeds the inclusive raw source bound.
    SourceBoundExceeded {
        /// Module path.
        path: PathBuf,
        /// Observed byte count.
        actual: u64,
        /// Accepted maximum.
        maximum: usize,
    },
    /// A module changed while exact bytes were captured.
    ChangedDuringCapture(PathBuf),
    /// Source-evidence document or aggregate admission failed before reading.
    Admission(SourceEvidenceError),
    /// Preflight discovered the same accepted module path more than once.
    DuplicateAdmissionPath(PathBuf),
    /// A module identifier could not be created.
    InvalidIdentifier,
    /// A module name could not be created.
    InvalidName,
}

impl From<SourceEvidenceError> for EdtModuleError {
    fn from(value: SourceEvidenceError) -> Self {
        Self::Admission(value)
    }
}

impl Display for EdtModuleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadDirectory { path, source } => {
                write!(
                    formatter,
                    "failed to read EDT module directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read EDT module {}: {source}",
                    path.display()
                )
            }
            Self::SymlinkArtifact(path) => {
                write!(formatter, "EDT module is a symlink: {}", path.display())
            }
            Self::ArtifactNotRegularFile(path) => write!(
                formatter,
                "EDT module is not a regular file: {}",
                path.display()
            ),
            Self::SourceBoundExceeded {
                path,
                actual,
                maximum,
            } => write!(
                formatter,
                "EDT module {} exceeds source byte bound: actual={actual}, maximum={maximum}",
                path.display()
            ),
            Self::ChangedDuringCapture(path) => write!(
                formatter,
                "EDT module changed during capture: {}",
                path.display()
            ),
            Self::Admission(source) => write!(formatter, "EDT source admission failed: {source}"),
            Self::DuplicateAdmissionPath(path) => write!(
                formatter,
                "duplicate EDT source admission path: {}",
                path.display()
            ),
            Self::InvalidIdentifier => formatter.write_str("EDT module identifier is invalid"),
            Self::InvalidName => formatter.write_str("EDT module name is invalid"),
        }
    }
}

impl std::error::Error for EdtModuleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. } | Self::ReadFile { source, .. } => Some(source),
            Self::Admission(source) => Some(source),
            Self::SymlinkArtifact(_)
            | Self::ArtifactNotRegularFile(_)
            | Self::SourceBoundExceeded { .. }
            | Self::ChangedDuringCapture(_)
            | Self::DuplicateAdmissionPath(_)
            | Self::InvalidIdentifier
            | Self::InvalidName => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_analysis::refactoring::{
        MAX_SOURCE_BYTES_PER_CONFIGURATION, MAX_SOURCE_DOCUMENT_BYTES,
        MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION, SourceEvidenceErrorKind,
    };
    use oneagent_common::{EntityId, EntityName};
    use std::fs;
    use tempfile::tempdir;

    use super::{
        EdtModuleError, EdtModuleKind, EdtModuleReader, EdtSourceManifest,
        FileSystemEdtModuleReader, SourceCapture, read_optional_module, source_metadata_changed,
    };

    fn assert_admission_bound(error: &EdtModuleError, actual: usize, maximum: usize) {
        let EdtModuleError::Admission(error) = error else {
            panic!("reader must preserve source-evidence admission failure");
        };
        assert_eq!(error.kind(), SourceEvidenceErrorKind::BoundExceeded);
        assert_eq!(error.actual(), Some(actual));
        assert_eq!(error.maximum(), Some(maximum));
    }

    #[test]
    fn production_preflight_rejects_document_count_before_file_inspection() {
        let root = tempdir().expect("temporary directory must be created");
        let paths = (0..=MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION)
            .map(|index| root.path().join(format!("missing-{index}.bsl")))
            .collect();
        let error = EdtSourceManifest::from_paths(paths)
            .expect_err("one-over count must fail before paths are inspected");
        assert_admission_bound(
            &error,
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION + 1,
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION,
        );
    }

    #[test]
    fn production_preflight_rejects_aggregate_before_raw_source_retention() {
        let root = tempdir().expect("temporary directory must be created");
        let mut paths = Vec::new();
        for index in 0..(MAX_SOURCE_BYTES_PER_CONFIGURATION / MAX_SOURCE_DOCUMENT_BYTES) {
            let path = root.path().join(format!("module-{index}.bsl"));
            let file = fs::File::create(&path).expect("sparse fixture must be created");
            file.set_len(MAX_SOURCE_DOCUMENT_BYTES as u64)
                .expect("sparse fixture length must be set");
            paths.push(path);
        }
        let one_over = root.path().join("one-over.bsl");
        fs::write(&one_over, b" ").expect("one-over fixture must be created");
        paths.push(one_over);
        let error = EdtSourceManifest::from_paths(paths)
            .expect_err("one-over aggregate must fail during metadata preflight");
        assert_admission_bound(
            &error,
            MAX_SOURCE_BYTES_PER_CONFIGURATION + 1,
            MAX_SOURCE_BYTES_PER_CONFIGURATION,
        );
    }

    #[test]
    fn production_reader_binds_preflight_metadata_to_opened_file() {
        let root = tempdir().expect("temporary directory must be created");
        let path = root.path().join("Module.bsl");
        fs::write(&path, b"old").expect("fixture must be created");
        let mut manifest =
            EdtSourceManifest::from_paths(vec![path.clone()]).expect("preflight must succeed");
        fs::remove_file(&path).expect("admitted fixture must be removed");
        fs::write(&path, b"new").expect("replacement fixture must be created");

        assert!(matches!(
            read_optional_module(&path, &mut SourceCapture::Manifest(&mut manifest)),
            Err(EdtModuleError::ChangedDuringCapture(changed)) if changed == path
        ));
    }

    #[test]
    fn changed_during_capture_metadata_is_rejected() {
        let root = tempdir().expect("temporary directory must be created");
        let path = root.path().join("Module.bsl");
        fs::write(&path, "Procedure A()\nEndProcedure\n").expect("source must be written");
        let before = fs::metadata(&path).expect("initial metadata must be readable");
        fs::write(&path, "Procedure Changed()\nEndProcedure\n").expect("source must change");
        let after = fs::metadata(&path).expect("changed metadata must be readable");
        assert!(source_metadata_changed(&before, &after));
    }

    #[test]
    fn reads_known_module_files() {
        let root = tempdir().expect("temporary directory must be created");

        fs::write(
            root.path().join("ObjectModule.bsl"),
            "Procedure Test()\nEndProcedure",
        )
        .expect("object module must be created");

        fs::write(
            root.path().join("ManagerModule.bsl"),
            "Function Test()\nEndFunction",
        )
        .expect("manager module must be created");

        let object_id = EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
            .expect("identifier must be valid");

        let reader = FileSystemEdtModuleReader;

        let modules = reader
            .read_modules(
                &object_id,
                &EntityName::new("Sales").expect("name must be valid"),
                root.path(),
            )
            .expect("modules must load");

        assert_eq!(modules.len(), 2);

        assert!(
            modules
                .iter()
                .any(|module| module.kind() == EdtModuleKind::Object)
        );

        assert!(
            modules
                .iter()
                .any(|module| module.kind() == EdtModuleKind::Manager)
        );
    }
}
