//! Reader for accepted Designer XML BSL module layouts.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::metadata_object::{ACCEPTED_FAMILIES, FamilySpec};
use crate::{
    DesignerXmlBuildScope, DesignerXmlDiscoveryError, DesignerXmlMetadataObjectDescriptor,
    is_designer_xml_project,
};

/// Accepted Designer XML module roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DesignerXmlModuleKind {
    /// Object module.
    Object,
    /// Manager module.
    Manager,
    /// Common Module implementation.
    Common,
}

impl DesignerXmlModuleKind {
    /// Returns the stable module identity suffix.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Object => "object_module",
            Self::Manager => "manager_module",
            Self::Common => "common_module",
        }
    }

    const fn filename(self) -> &'static str {
        match self {
            Self::Object => "ObjectModule.bsl",
            Self::Manager => "ManagerModule.bsl",
            Self::Common => "Module.bsl",
        }
    }
}

/// Adapter-local raw evidence for one accepted Designer BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerXmlModuleSourceEvidence {
    artifact_path: PathBuf,
    raw_source: Vec<u8>,
}

impl DesignerXmlModuleSourceEvidence {
    /// Returns the exact accepted source path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the raw bytes before BOM or line-ending normalization.
    #[must_use]
    pub fn raw_source(&self) -> &[u8] {
        &self.raw_source
    }
}

/// One normalized Designer BSL module joined to its canonical metadata owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerXmlModuleDescriptor {
    id: EntityId,
    name: EntityName,
    kind: DesignerXmlModuleKind,
    owner_id: EntityId,
    owner_name: EntityName,
    owner_kind: MetadataKind,
    source_text: String,
    source: DesignerXmlModuleSourceEvidence,
}

impl DesignerXmlModuleDescriptor {
    /// Returns the stable `<owner-uuid>:<role-suffix>` identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the compatible module name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the accepted module role.
    #[must_use]
    pub const fn kind(&self) -> DesignerXmlModuleKind {
        self.kind
    }

    /// Returns the canonical metadata owner identifier.
    #[must_use]
    pub const fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }

    /// Returns the canonical metadata owner name.
    #[must_use]
    pub const fn owner_name(&self) -> &EntityName {
        &self.owner_name
    }

    /// Returns the canonical metadata owner kind.
    #[must_use]
    pub const fn owner_kind(&self) -> MetadataKind {
        self.owner_kind
    }

    /// Returns UTF-8 source after the accepted BOM and line-ending normalization.
    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// Returns adapter-local raw source evidence.
    #[must_use]
    pub const fn source(&self) -> &DesignerXmlModuleSourceEvidence {
        &self.source
    }
}

/// Reads accepted Designer XML BSL modules for parsed metadata owners.
pub trait DesignerXmlModuleReader {
    /// Reads, joins, normalizes, and canonically orders accepted module roles.
    ///
    /// # Errors
    ///
    /// Returns a typed error for any structurally invalid supplied accepted layout.
    fn read_modules(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
        owners: &[DesignerXmlMetadataObjectDescriptor],
    ) -> Result<Vec<DesignerXmlModuleDescriptor>, DesignerXmlModuleError>;
}

/// Filesystem implementation of [`DesignerXmlModuleReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemDesignerXmlModuleReader;

impl DesignerXmlModuleReader for FileSystemDesignerXmlModuleReader {
    fn read_modules(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
        owners: &[DesignerXmlMetadataObjectDescriptor],
    ) -> Result<Vec<DesignerXmlModuleDescriptor>, DesignerXmlModuleError> {
        if !is_designer_xml_project(project_root)? {
            return Err(DesignerXmlModuleError::MarkersNotFound(
                project_root.to_path_buf(),
            ));
        }
        let owners = assemble_owners(project_root, owners)?;
        let mut modules = Vec::new();
        for family in ACCEPTED_FAMILIES {
            collect_family_modules(project_root, family, &owners, &mut modules)?;
        }
        modules.sort_by(|left, right| {
            (
                &left.owner_id,
                left.kind,
                &left.id,
                &left.source.artifact_path,
            )
                .cmp(&(
                    &right.owner_id,
                    right.kind,
                    &right.id,
                    &right.source.artifact_path,
                ))
        });

        let _ = scope;
        Ok(modules)
    }
}

type OwnerKey = (&'static str, String);

fn assemble_owners<'a>(
    project_root: &Path,
    owners: &'a [DesignerXmlMetadataObjectDescriptor],
) -> Result<BTreeMap<OwnerKey, &'a DesignerXmlMetadataObjectDescriptor>, DesignerXmlModuleError> {
    let mut assembled = BTreeMap::new();
    for owner in owners {
        let family = ACCEPTED_FAMILIES
            .iter()
            .find(|family| family.kind == owner.kind())
            .ok_or_else(|| DesignerXmlModuleError::UnsupportedOwnerKind {
                kind: owner.kind(),
                path: owner.source().artifact_path().to_path_buf(),
            })?;
        let expected_path = project_root
            .join(family.directory)
            .join(format!("{}.xml", owner.name().as_str()));
        if owner.source().artifact_path() != expected_path {
            return Err(DesignerXmlModuleError::OwnerPathMismatch {
                expected: expected_path,
                actual: owner.source().artifact_path().to_path_buf(),
            });
        }
        let key = (family.directory, owner.name().as_str().to_owned());
        if assembled.insert(key, owner).is_some() {
            return Err(DesignerXmlModuleError::DuplicateOwner {
                kind: owner.kind(),
                name: owner.name().clone(),
            });
        }
    }
    Ok(assembled)
}

fn collect_family_modules(
    project_root: &Path,
    family: FamilySpec,
    owners: &BTreeMap<OwnerKey, &DesignerXmlMetadataObjectDescriptor>,
    modules: &mut Vec<DesignerXmlModuleDescriptor>,
) -> Result<(), DesignerXmlModuleError> {
    let family_path = project_root.join(family.directory);
    let family_metadata = match fs::symlink_metadata(&family_path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(inspect_error(family_path, source)),
    };
    if family_metadata.file_type().is_symlink() {
        return Err(DesignerXmlModuleError::SymlinkArtifact(family_path));
    }
    if !family_metadata.file_type().is_dir() {
        return Err(DesignerXmlModuleError::FamilyNotDirectory(family_path));
    }

    let mut directories = direct_directories(&family_path)?;
    directories.sort_by(|left, right| left.0.cmp(&right.0));
    for (directory_name, directory_path) in directories {
        let Some(role_paths) = accepted_role_paths(&directory_path)? else {
            continue;
        };
        let exact_key = (family.directory, directory_name.clone());
        let owner = owners.get(&exact_key).copied();
        if owner.is_none() {
            let case_match = owners.iter().find(|((owner_family, owner_name), _)| {
                *owner_family == family.directory
                    && owner_name.eq_ignore_ascii_case(&directory_name)
            });
            if case_match.is_some() {
                return Err(DesignerXmlModuleError::OwnerNameMismatch(directory_path));
            }
            return Err(DesignerXmlModuleError::OrphanOwner(directory_path));
        }
        let owner = owner.expect("owner presence was checked");
        for (kind, path) in role_paths {
            if kind == DesignerXmlModuleKind::Common && owner.kind() != MetadataKind::CommonModule {
                return Err(DesignerXmlModuleError::WrongOwnerKind {
                    path,
                    role: kind,
                    actual: owner.kind(),
                });
            }
            modules.push(read_module(owner, kind, path)?);
        }
    }
    Ok(())
}

fn direct_directories(
    family_path: &Path,
) -> Result<Vec<(String, PathBuf)>, DesignerXmlModuleError> {
    let entries =
        fs::read_dir(family_path).map_err(|source| DesignerXmlModuleError::ReadDirectory {
            path: family_path.to_path_buf(),
            source,
        })?;
    let mut directories = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| DesignerXmlModuleError::ReadDirectoryEntry {
            path: family_path.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| inspect_error(path.clone(), source))?;
        if file_type.is_symlink() {
            return Err(DesignerXmlModuleError::SymlinkArtifact(path));
        }
        if !file_type.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| DesignerXmlModuleError::InvalidPath(path.clone()))?;
        directories.push((name, path));
    }
    Ok(directories)
}

fn accepted_role_paths(
    owner_directory: &Path,
) -> Result<Option<Vec<(DesignerXmlModuleKind, PathBuf)>>, DesignerXmlModuleError> {
    let ext_path = owner_directory.join("Ext");
    let metadata = match fs::symlink_metadata(&ext_path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(inspect_error(ext_path, source)),
    };
    if metadata.file_type().is_symlink() {
        return Err(DesignerXmlModuleError::SymlinkArtifact(ext_path));
    }
    if !metadata.file_type().is_dir() {
        return Err(DesignerXmlModuleError::ExtensionNotDirectory(ext_path));
    }

    let entries =
        fs::read_dir(&ext_path).map_err(|source| DesignerXmlModuleError::ReadDirectory {
            path: ext_path.clone(),
            source,
        })?;
    let mut candidates = BTreeMap::<DesignerXmlModuleKind, Vec<(String, PathBuf)>>::new();
    for entry in entries {
        let entry = entry.map_err(|source| DesignerXmlModuleError::ReadDirectoryEntry {
            path: ext_path.clone(),
            source,
        })?;
        let path = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| DesignerXmlModuleError::InvalidPath(path.clone()))?;
        let Some(kind) = role_for_case_insensitive_filename(&name) else {
            continue;
        };
        let file_type = entry
            .file_type()
            .map_err(|source| inspect_error(path.clone(), source))?;
        if file_type.is_symlink() {
            return Err(DesignerXmlModuleError::SymlinkArtifact(path));
        }
        if !file_type.is_file() {
            return Err(DesignerXmlModuleError::ArtifactNotRegularFile(path));
        }
        candidates.entry(kind).or_default().push((name, path));
    }

    let mut roles = Vec::new();
    for kind in [
        DesignerXmlModuleKind::Object,
        DesignerXmlModuleKind::Manager,
        DesignerXmlModuleKind::Common,
    ] {
        let Some(paths) = candidates.remove(&kind) else {
            continue;
        };
        let path = select_role_path(kind, paths)?;
        roles.push((kind, path));
    }
    Ok((!roles.is_empty()).then_some(roles))
}

fn select_role_path(
    kind: DesignerXmlModuleKind,
    mut paths: Vec<(String, PathBuf)>,
) -> Result<PathBuf, DesignerXmlModuleError> {
    paths.sort();
    if paths.len() > 1 {
        return Err(DesignerXmlModuleError::DuplicateRole {
            role: kind,
            paths: paths.into_iter().map(|(_, path)| path).collect(),
        });
    }
    let (actual_name, path) = paths.pop().expect("candidate list is non-empty");
    if actual_name != kind.filename() {
        return Err(DesignerXmlModuleError::RolePathMismatch { role: kind, path });
    }
    Ok(path)
}

fn role_for_case_insensitive_filename(name: &str) -> Option<DesignerXmlModuleKind> {
    [
        DesignerXmlModuleKind::Object,
        DesignerXmlModuleKind::Manager,
        DesignerXmlModuleKind::Common,
    ]
    .into_iter()
    .find(|kind| name.eq_ignore_ascii_case(kind.filename()))
}

fn read_module(
    owner: &DesignerXmlMetadataObjectDescriptor,
    kind: DesignerXmlModuleKind,
    path: PathBuf,
) -> Result<DesignerXmlModuleDescriptor, DesignerXmlModuleError> {
    let raw_source = fs::read(&path).map_err(|source| DesignerXmlModuleError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let source_text = normalize_source(&raw_source)
        .map_err(|_| DesignerXmlModuleError::InvalidUtf8 { path: path.clone() })?;
    let id = EntityId::new(format!("{}:{}", owner.id().as_str(), kind.as_str()))
        .map_err(|_| DesignerXmlModuleError::InvalidIdentifier)?;
    let name = match kind {
        DesignerXmlModuleKind::Object => EntityName::new("ObjectModule"),
        DesignerXmlModuleKind::Manager => EntityName::new("ManagerModule"),
        DesignerXmlModuleKind::Common => EntityName::new(owner.name().as_str()),
    }
    .map_err(|_| DesignerXmlModuleError::InvalidName)?;

    Ok(DesignerXmlModuleDescriptor {
        id,
        name,
        kind,
        owner_id: owner.id().clone(),
        owner_name: owner.name().clone(),
        owner_kind: owner.kind(),
        source_text,
        source: DesignerXmlModuleSourceEvidence {
            artifact_path: path,
            raw_source,
        },
    })
}

fn normalize_source(raw_source: &[u8]) -> Result<String, std::str::Utf8Error> {
    let without_bom = raw_source
        .strip_prefix(&[0xef, 0xbb, 0xbf])
        .unwrap_or(raw_source);
    let source = std::str::from_utf8(without_bom)?;
    Ok(source.replace("\r\n", "\n").replace('\r', "\n"))
}

fn inspect_error(path: PathBuf, source: std::io::Error) -> DesignerXmlModuleError {
    DesignerXmlModuleError::InspectPath { path, source }
}

/// Errors produced while assembling or reading Designer XML modules.
#[derive(Debug)]
pub enum DesignerXmlModuleError {
    /// Project marker validation failed.
    Discovery(DesignerXmlDiscoveryError),
    /// The supplied root has no Designer XML marker pair.
    MarkersNotFound(PathBuf),
    /// A path could not be inspected.
    InspectPath {
        /// Inspected path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A required direct path is a symlink.
    SymlinkArtifact(PathBuf),
    /// An accepted family path is not a directory.
    FamilyNotDirectory(PathBuf),
    /// An owner `Ext` path is not a directory.
    ExtensionNotDirectory(PathBuf),
    /// A role path is not a regular file.
    ArtifactNotRegularFile(PathBuf),
    /// A directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A directory entry could not be read.
    ReadDirectoryEntry {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A module could not be read.
    ReadFile {
        /// Module path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A path component is not valid UTF-8.
    InvalidPath(PathBuf),
    /// A descriptor kind is outside the accepted top-level family registry.
    UnsupportedOwnerKind {
        /// Unsupported kind.
        kind: MetadataKind,
        /// Source descriptor path.
        path: PathBuf,
    },
    /// A descriptor path does not match its canonical owner join.
    OwnerPathMismatch {
        /// Required exact descriptor path.
        expected: PathBuf,
        /// Supplied descriptor path.
        actual: PathBuf,
    },
    /// More than one supplied descriptor claims an owner key.
    DuplicateOwner {
        /// Owner kind.
        kind: MetadataKind,
        /// Owner name.
        name: EntityName,
    },
    /// A module-bearing directory has no metadata owner.
    OrphanOwner(PathBuf),
    /// A module-bearing directory differs from its owner name by case.
    OwnerNameMismatch(PathBuf),
    /// A role is incompatible with the joined owner kind.
    WrongOwnerKind {
        /// Module path.
        path: PathBuf,
        /// Observed role.
        role: DesignerXmlModuleKind,
        /// Joined owner kind.
        actual: MetadataKind,
    },
    /// More than one path claims one role under an owner.
    DuplicateRole {
        /// Duplicated role.
        role: DesignerXmlModuleKind,
        /// Conflicting paths in canonical order.
        paths: Vec<PathBuf>,
    },
    /// A case-insensitive role filename differs from the exact accepted spelling.
    RolePathMismatch {
        /// Claimed role.
        role: DesignerXmlModuleKind,
        /// Mismatched path.
        path: PathBuf,
    },
    /// Module bytes are not valid UTF-8 after BOM handling.
    InvalidUtf8 {
        /// Module path.
        path: PathBuf,
    },
    /// Stable module identity construction failed.
    InvalidIdentifier,
    /// Compatible module name construction failed.
    InvalidName,
}

impl From<DesignerXmlDiscoveryError> for DesignerXmlModuleError {
    fn from(value: DesignerXmlDiscoveryError) -> Self {
        Self::Discovery(value)
    }
}

impl Display for DesignerXmlModuleError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discovery(source) => write!(formatter, "Designer XML discovery failed: {source}"),
            Self::MarkersNotFound(path) => write!(
                formatter,
                "Designer XML markers were not found at {}",
                path.display()
            ),
            Self::InspectPath { path, source } => {
                write!(formatter, "failed to inspect {}: {source}", path.display())
            }
            Self::SymlinkArtifact(path) => {
                write!(
                    formatter,
                    "Designer XML artifact is a symlink: {}",
                    path.display()
                )
            }
            Self::FamilyNotDirectory(path) => write!(
                formatter,
                "Designer XML metadata family is not a directory: {}",
                path.display()
            ),
            Self::ExtensionNotDirectory(path) => write!(
                formatter,
                "Designer XML module extension path is not a directory: {}",
                path.display()
            ),
            Self::ArtifactNotRegularFile(path) => write!(
                formatter,
                "Designer XML module artifact is not a regular file: {}",
                path.display()
            ),
            Self::ReadDirectory { path, source } | Self::ReadDirectoryEntry { path, source } => {
                write!(
                    formatter,
                    "failed to read directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::InvalidPath(path) => {
                write!(
                    formatter,
                    "Designer XML path is not valid UTF-8: {}",
                    path.display()
                )
            }
            Self::UnsupportedOwnerKind { kind, path } => write!(
                formatter,
                "Designer XML owner {} has unsupported kind {kind}",
                path.display()
            ),
            Self::OwnerPathMismatch { expected, actual } => write!(
                formatter,
                "Designer XML owner path {} does not match {}",
                actual.display(),
                expected.display()
            ),
            Self::DuplicateOwner { kind, name } => {
                write!(formatter, "duplicate Designer XML owner {kind}/{name}")
            }
            Self::OrphanOwner(path) => write!(
                formatter,
                "Designer XML module directory has no owner: {}",
                path.display()
            ),
            Self::OwnerNameMismatch(path) => write!(
                formatter,
                "Designer XML module directory mismatches owner name: {}",
                path.display()
            ),
            Self::WrongOwnerKind { path, role, actual } => write!(
                formatter,
                "Designer XML {} role at {} is incompatible with {actual}",
                role.as_str(),
                path.display()
            ),
            Self::DuplicateRole { role, paths } => write!(
                formatter,
                "duplicate Designer XML {} role paths: {paths:?}",
                role.as_str()
            ),
            Self::RolePathMismatch { role, path } => write!(
                formatter,
                "Designer XML {} role has mismatched path {}",
                role.as_str(),
                path.display()
            ),
            Self::InvalidUtf8 { path } => write!(
                formatter,
                "Designer XML module is not valid UTF-8: {}",
                path.display()
            ),
            Self::InvalidIdentifier => {
                formatter.write_str("invalid Designer XML module identifier")
            }
            Self::InvalidName => formatter.write_str("invalid Designer XML module name"),
        }
    }
}

impl std::error::Error for DesignerXmlModuleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            Self::InspectPath { source, .. }
            | Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesignerXmlModuleError, DesignerXmlModuleKind, DesignerXmlModuleReader,
        FileSystemDesignerXmlModuleReader, normalize_source, select_role_path,
    };
    use crate::{
        DesignerXmlBuildScope, DesignerXmlMetadataObjectReader,
        FileSystemDesignerXmlMetadataObjectReader,
    };
    use oneagent_bsl::{BslDeclarationExtractor, BslSymbolKind, LineBslDeclarationExtractor};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    const DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;
    const CONFIGURATION: &str = r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Configuration uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093"><Properties><Name>DNSWorldEdition</Name></Properties></Configuration></MetaDataObject>"#;
    const DESIGNER_MODULE: &[u8] =
        include_bytes!("../tests/fixtures/modules/designer/DynamicSecurityOverridable.bsl");
    const EDT_MODULE: &[u8] =
        include_bytes!("../tests/fixtures/modules/edt/DynamicSecurityOverridable.bsl");

    fn write_project(root: &Path) {
        fs::write(root.join("ConfigDumpInfo.xml"), DUMP_INFO).expect("dump marker must be created");
        fs::write(root.join("Configuration.xml"), CONFIGURATION)
            .expect("configuration marker must be created");
    }

    fn metadata_xml(root: &str, uuid: &str, name: &str) -> String {
        format!(
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><{root} uuid="{uuid}"><Properties><Name>{name}</Name></Properties></{root}></MetaDataObject>"#
        )
    }

    fn write_owner(root: &Path, family: &str, kind_root: &str, uuid: &str, name: &str) {
        fs::create_dir_all(root.join(family)).expect("family must be created");
        fs::write(
            root.join(family).join(format!("{name}.xml")),
            metadata_xml(kind_root, uuid, name),
        )
        .expect("owner descriptor must be written");
    }

    fn owners(root: &Path) -> Vec<crate::DesignerXmlMetadataObjectDescriptor> {
        FileSystemDesignerXmlMetadataObjectReader
            .read_all(root, DesignerXmlBuildScope::Partial)
            .expect("owners must parse")
    }

    #[test]
    fn normalizes_exact_paired_common_module_and_feeds_bsl_analyzer() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        write_owner(
            root.path(),
            "CommonModules",
            "CommonModule",
            "aee4de19-9300-4f9e-88e9-ec43983c719b",
            "DynamicSecurityOverridable",
        );
        fs::create_dir_all(
            root.path()
                .join("CommonModules/DynamicSecurityOverridable/Ext"),
        )
        .expect("module directory must be created");
        fs::write(
            root.path()
                .join("CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"),
            DESIGNER_MODULE,
        )
        .expect("Designer fixture must be written");

        let modules = FileSystemDesignerXmlModuleReader
            .read_modules(
                root.path(),
                DesignerXmlBuildScope::Partial,
                &owners(root.path()),
            )
            .expect("paired module must parse");

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].kind(), DesignerXmlModuleKind::Common);
        assert_eq!(modules[0].name().as_str(), "DynamicSecurityOverridable");
        assert_eq!(modules[0].source().raw_source(), DESIGNER_MODULE);
        assert_eq!(
            modules[0].source_text(),
            normalize_source(EDT_MODULE).expect("EDT fixture must be valid")
        );
        let symbols = LineBslDeclarationExtractor
            .extract(modules[0].id(), modules[0].source_text())
            .expect("existing BSL analyzer must accept normalized source");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].kind(), BslSymbolKind::Procedure);
        assert_eq!(symbols[0].name().as_str(), "FillSecurityCollection");
        assert!(symbols[0].is_exported());
    }

    #[test]
    fn reads_three_roles_canonically_and_repeatedly() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        write_owner(
            root.path(),
            "CommonModules",
            "CommonModule",
            "aee4de19-9300-4f9e-88e9-ec43983c719b",
            "Shared",
        );
        let ext = root.path().join("CommonModules/Shared/Ext");
        fs::create_dir_all(&ext).expect("module directory must be created");
        fs::write(
            ext.join("Module.bsl"),
            "Procedure Common()\r\nEndProcedure\r\n",
        )
        .expect("common module must be written");
        fs::write(
            ext.join("ManagerModule.bsl"),
            "Function Manager()\rReturn 1;\rEndFunction\r",
        )
        .expect("manager module must be written");
        fs::write(
            ext.join("ObjectModule.bsl"),
            b"\xef\xbb\xbfProcedure Object()\nEndProcedure\n",
        )
        .expect("object module must be written");
        let owners = owners(root.path());

        let first = FileSystemDesignerXmlModuleReader
            .read_modules(root.path(), DesignerXmlBuildScope::Complete, &owners)
            .expect("roles must parse");
        let second = FileSystemDesignerXmlModuleReader
            .read_modules(root.path(), DesignerXmlBuildScope::Complete, &owners)
            .expect("repeated read must parse");

        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
        assert_eq!(first[0].kind(), DesignerXmlModuleKind::Object);
        assert_eq!(first[1].kind(), DesignerXmlModuleKind::Manager);
        assert_eq!(first[2].kind(), DesignerXmlModuleKind::Common);
        assert!(
            first
                .iter()
                .all(|module| !module.source_text().contains('\r'))
        );
    }

    #[test]
    fn missing_optional_modules_create_no_placeholders() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        write_owner(
            root.path(),
            "Catalogs",
            "Catalog",
            "92bcb692-56c4-4199-bf7e-e33cdd76a310",
            "Products",
        );

        let modules = FileSystemDesignerXmlModuleReader
            .read_modules(
                root.path(),
                DesignerXmlBuildScope::Complete,
                &owners(root.path()),
            )
            .expect("missing optional roles must succeed");

        assert!(modules.is_empty());
    }

    #[test]
    fn rejects_orphan_wrong_kind_and_mismatched_role_paths() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir_all(root.path().join("Catalogs/Orphan/Ext"))
            .expect("orphan directory must be created");
        fs::write(
            root.path().join("Catalogs/Orphan/Ext/ObjectModule.bsl"),
            "Procedure Orphan()\nEndProcedure\n",
        )
        .expect("orphan module must be written");
        assert!(matches!(
            FileSystemDesignerXmlModuleReader.read_modules(
                root.path(),
                DesignerXmlBuildScope::Partial,
                &owners(root.path())
            ),
            Err(DesignerXmlModuleError::OrphanOwner(_))
        ));

        fs::write(
            root.path().join("Catalogs/Orphan.xml"),
            metadata_xml("Catalog", "92bcb692-56c4-4199-bf7e-e33cdd76a310", "Orphan"),
        )
        .expect("owner descriptor must be written");
        fs::write(
            root.path().join("Catalogs/Orphan/Ext/Module.bsl"),
            "Procedure WrongKind()\nEndProcedure\n",
        )
        .expect("wrong-kind module must be written");
        assert!(matches!(
            FileSystemDesignerXmlModuleReader.read_modules(
                root.path(),
                DesignerXmlBuildScope::Partial,
                &owners(root.path())
            ),
            Err(DesignerXmlModuleError::WrongOwnerKind { .. })
        ));

        fs::remove_file(root.path().join("Catalogs/Orphan/Ext/Module.bsl"))
            .expect("wrong-kind module must be removed");
        fs::rename(
            root.path().join("Catalogs/Orphan/Ext/ObjectModule.bsl"),
            root.path().join("Catalogs/Orphan/Ext/objectmodule.bsl"),
        )
        .expect("role path must be renamed");
        assert!(matches!(
            FileSystemDesignerXmlModuleReader.read_modules(
                root.path(),
                DesignerXmlBuildScope::Partial,
                &owners(root.path())
            ),
            Err(DesignerXmlModuleError::RolePathMismatch { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_role_candidates_deterministically() {
        let error = select_role_path(
            DesignerXmlModuleKind::Object,
            vec![
                (
                    "objectmodule.bsl".to_owned(),
                    "/source/objectmodule.bsl".into(),
                ),
                (
                    "ObjectModule.bsl".to_owned(),
                    "/source/ObjectModule.bsl".into(),
                ),
            ],
        )
        .expect_err("duplicate role candidates must fail");

        assert!(matches!(
            error,
            DesignerXmlModuleError::DuplicateRole { .. }
        ));
    }

    #[test]
    fn rejects_invalid_utf8_in_both_scopes() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        write_owner(
            root.path(),
            "Catalogs",
            "Catalog",
            "92bcb692-56c4-4199-bf7e-e33cdd76a310",
            "Products",
        );
        let ext = root.path().join("Catalogs/Products/Ext");
        fs::create_dir_all(&ext).expect("module directory must be created");
        let owners = owners(root.path());
        fs::write(ext.join("ObjectModule.bsl"), [0xff, 0xfe])
            .expect("invalid source must be written");
        for scope in [
            DesignerXmlBuildScope::Complete,
            DesignerXmlBuildScope::Partial,
        ] {
            assert!(matches!(
                FileSystemDesignerXmlModuleReader.read_modules(root.path(), scope, &owners),
                Err(DesignerXmlModuleError::InvalidUtf8 { .. })
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_module_artifact() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        write_owner(
            root.path(),
            "Catalogs",
            "Catalog",
            "92bcb692-56c4-4199-bf7e-e33cdd76a310",
            "Products",
        );
        let ext = root.path().join("Catalogs/Products/Ext");
        fs::create_dir_all(&ext).expect("module directory must be created");
        fs::write(root.path().join("outside.bsl"), "").expect("target must be written");
        symlink(
            root.path().join("outside.bsl"),
            ext.join("ObjectModule.bsl"),
        )
        .expect("module symlink must be created");

        assert!(matches!(
            FileSystemDesignerXmlModuleReader.read_modules(
                root.path(),
                DesignerXmlBuildScope::Partial,
                &owners(root.path())
            ),
            Err(DesignerXmlModuleError::SymlinkArtifact(_))
        ));
    }
}
