//! Workspace and configuration model for `OneAgent`.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataTree;
use std::path::{Path, PathBuf};

/// Source representation of a `1C:Enterprise` project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceFormat {
    /// `1C:EDT` project exported to source files.
    Edt,
    /// Designer XML export.
    DesignerXml,
    /// Extension project.
    Extension,
    /// Source format has not yet been detected.
    Unknown,
}

/// A discovered `1C:Enterprise` configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
    id: EntityId,
    name: EntityName,
    root_path: PathBuf,
    format: WorkspaceFormat,
    metadata: MetadataTree,
}

impl Configuration {
    /// Creates a configuration model.
    #[must_use]
    pub fn new(
        id: EntityId,
        name: EntityName,
        root_path: impl Into<PathBuf>,
        format: WorkspaceFormat,
    ) -> Self {
        Self {
            id,
            name,
            root_path: root_path.into(),
            format,
            metadata: MetadataTree::new(),
        }
    }

    /// Returns the configuration identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the configuration name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the root directory.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Returns the detected source format.
    #[must_use]
    pub const fn format(&self) -> WorkspaceFormat {
        self.format
    }

    /// Returns the semantic metadata tree.
    #[must_use]
    pub const fn metadata(&self) -> &MetadataTree {
        &self.metadata
    }

    /// Returns mutable access to the semantic metadata tree.
    #[must_use]
    pub const fn metadata_mut(&mut self) -> &mut MetadataTree {
        &mut self.metadata
    }
}

/// A local `OneAgent` workspace containing one or more configurations.
#[derive(Debug, Clone)]
pub struct Workspace {
    root_path: PathBuf,
    configurations: Vec<Configuration>,
}

impl Workspace {
    /// Creates an empty workspace rooted at the supplied path.
    #[must_use]
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            configurations: Vec::new(),
        }
    }

    /// Returns the workspace root directory.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Adds a configuration.
    pub fn add_configuration(&mut self, configuration: Configuration) {
        self.configurations.push(configuration);
    }

    /// Returns all configurations.
    #[must_use]
    pub fn configurations(&self) -> &[Configuration] {
        &self.configurations
    }

    /// Finds a configuration by identifier.
    #[must_use]
    pub fn configuration(&self, id: &EntityId) -> Option<&Configuration> {
        self.configurations
            .iter()
            .find(|configuration| configuration.id() == id)
    }

    /// Returns the number of configurations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.configurations.len()
    }

    /// Returns `true` when the workspace has no configurations.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.configurations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{Configuration, Workspace, WorkspaceFormat};

    #[test]
    fn workspace_finds_configuration_by_identifier() {
        let configuration_id =
            EntityId::new("configuration.main").expect("identifier must be valid");
        let configuration = Configuration::new(
            configuration_id.clone(),
            EntityName::new("Main").expect("name must be valid"),
            "/tmp/main",
            WorkspaceFormat::Edt,
        );

        let mut workspace = Workspace::new("/tmp");
        workspace.add_configuration(configuration);

        assert_eq!(
            workspace
                .configuration(&configuration_id)
                .expect("configuration must exist")
                .name()
                .as_str(),
            "Main"
        );
    }
}

/// Result of discovering a configuration on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredConfiguration {
    root_path: PathBuf,
    format: WorkspaceFormat,
}

impl DiscoveredConfiguration {
    /// Creates a discovery result.
    #[must_use]
    pub fn new(root_path: impl Into<PathBuf>, format: WorkspaceFormat) -> Self {
        Self {
            root_path: root_path.into(),
            format,
        }
    }

    /// Returns the detected configuration root.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Returns the detected source format.
    #[must_use]
    pub const fn format(&self) -> WorkspaceFormat {
        self.format
    }
}

/// Port for discovering `1C:Enterprise` configurations.
///
/// Implementations belong to adapter crates and may use the filesystem,
/// IDE APIs or remote services.
pub trait WorkspaceDetector {
    /// Discovers configurations below the supplied root.
    ///
    /// # Errors
    ///
    /// Returns an implementation-specific error when discovery cannot complete.
    fn discover(
        &self,
        root: &Path,
    ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>;
}
