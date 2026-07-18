//! Reader for module files inside EDT metadata objects.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// Supported EDT module file kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdtModuleKind {
    /// Object module.
    Object,
    /// Manager module.
    Manager,
    /// Common module implementation.
    Common,
}

impl EdtModuleKind {
    /// Returns a stable machine-readable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Object => "object_module",
            Self::Manager => "manager_module",
            Self::Common => "common_module",
        }
    }
}

/// Parsed EDT module descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtModuleDescriptor {
    id: EntityId,
    name: EntityName,
    kind: EdtModuleKind,
    path: PathBuf,
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
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError>;
}

/// Filesystem implementation of [`EdtModuleReader`].
#[derive(Debug, Default)]
pub struct FileSystemEdtModuleReader;

impl EdtModuleReader for FileSystemEdtModuleReader {
    fn read_modules(
        &self,
        object_id: &EntityId,
        object_directory: &Path,
    ) -> Result<Vec<EdtModuleDescriptor>, EdtModuleError> {
        let candidates = [
            ("ObjectModule.bsl", EdtModuleKind::Object, "ObjectModule"),
            ("ManagerModule.bsl", EdtModuleKind::Manager, "ManagerModule"),
            ("Module.bsl", EdtModuleKind::Common, "Module"),
        ];

        let mut modules = Vec::new();

        for (file_name, kind, display_name) in candidates {
            let path = object_directory.join(file_name);

            if !path.is_file() {
                continue;
            }

            fs::read_to_string(&path).map_err(|source| EdtModuleError::ReadFile {
                path: path.clone(),
                source,
            })?;

            let id = EntityId::new(format!("{}:{}", object_id.as_str(), kind.as_str()))
                .map_err(|_| EdtModuleError::InvalidIdentifier)?;

            let name = EntityName::new(display_name).map_err(|_| EdtModuleError::InvalidName)?;

            modules.push(EdtModuleDescriptor::new(id, name, kind, path));
        }

        Ok(modules)
    }
}

/// Error produced while reading EDT modules.
#[derive(Debug)]
pub enum EdtModuleError {
    /// A module file could not be read.
    ReadFile {
        /// Module path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A module identifier could not be created.
    InvalidIdentifier,
    /// A module name could not be created.
    InvalidName,
}

impl Display for EdtModuleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read EDT module {}: {source}",
                    path.display()
                )
            }
            Self::InvalidIdentifier => formatter.write_str("EDT module identifier is invalid"),
            Self::InvalidName => formatter.write_str("EDT module name is invalid"),
        }
    }
}

impl std::error::Error for EdtModuleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            Self::InvalidIdentifier | Self::InvalidName => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;
    use std::fs;
    use tempfile::tempdir;

    use super::{EdtModuleKind, EdtModuleReader, FileSystemEdtModuleReader};

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
            .read_modules(&object_id, root.path())
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
