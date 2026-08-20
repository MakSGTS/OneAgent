//! Filesystem adapter for discovering `1C:Enterprise` workspaces.

use oneagent_designer_xml::{DesignerXmlDiscoveryError, is_designer_xml_project};
use oneagent_workspace::{DiscoveredConfiguration, WorkspaceDetector, WorkspaceFormat};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const EDT_PROJECT_FILE: &str = ".project";
const EDT_CONFIGURATION_FILE: &str = "src/Configuration/Configuration.mdo";

/// Filesystem-based workspace detector.
#[derive(Debug, Clone, Copy)]
pub struct FileSystemWorkspaceDetector {
    max_depth: usize,
}

impl FileSystemWorkspaceDetector {
    /// Creates a detector with the supplied recursion limit.
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    fn discover_directory(
        self,
        directory: &Path,
        depth: usize,
        results: &mut BTreeMap<PathBuf, WorkspaceFormat>,
    ) -> Result<(), DiscoveryError> {
        if depth > self.max_depth {
            return Ok(());
        }

        let edt = is_edt_project(directory);
        let designer_xml =
            is_designer_xml_project(directory).map_err(|source| DiscoveryError::DesignerXml {
                path: directory.to_path_buf(),
                source,
            })?;

        if edt && designer_xml {
            return Err(DiscoveryError::ConflictingFormatMarkers(
                directory.to_path_buf(),
            ));
        }

        if edt || designer_xml {
            results.insert(
                directory.to_path_buf(),
                if edt {
                    WorkspaceFormat::Edt
                } else {
                    WorkspaceFormat::DesignerXml
                },
            );
            return Ok(());
        }

        for entry in fs::read_dir(directory).map_err(|source| DiscoveryError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| DiscoveryError::ReadDirectoryEntry {
                path: directory.to_path_buf(),
                source,
            })?;

            let file_type = entry
                .file_type()
                .map_err(|source| DiscoveryError::ReadFileType {
                    path: entry.path(),
                    source,
                })?;

            if file_type.is_dir() && !is_ignored_directory(&entry.path()) {
                self.discover_directory(&entry.path(), depth + 1, results)?;
            }
        }

        Ok(())
    }
}

impl Default for FileSystemWorkspaceDetector {
    fn default() -> Self {
        Self::new(6)
    }
}

impl WorkspaceDetector for FileSystemWorkspaceDetector {
    fn discover(
        &self,
        root: &Path,
    ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>> {
        if !root.exists() {
            return Err(Box::new(DiscoveryError::RootDoesNotExist(
                root.to_path_buf(),
            )));
        }

        if !root.is_dir() {
            return Err(Box::new(DiscoveryError::RootIsNotDirectory(
                root.to_path_buf(),
            )));
        }

        let mut roots = BTreeMap::new();
        self.discover_directory(root, 0, &mut roots)?;

        Ok(roots
            .into_iter()
            .map(|(path, format)| DiscoveredConfiguration::new(path, format))
            .collect())
    }
}

fn is_edt_project(directory: &Path) -> bool {
    directory.join(EDT_PROJECT_FILE).is_file() && directory.join(EDT_CONFIGURATION_FILE).is_file()
}

fn is_ignored_directory(directory: &Path) -> bool {
    directory
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | ".idea" | ".vscode" | "target" | "node_modules"
            )
        })
}

/// Errors produced during filesystem discovery.
#[derive(Debug)]
pub enum DiscoveryError {
    /// The supplied root does not exist.
    RootDoesNotExist(PathBuf),
    /// The supplied root is not a directory.
    RootIsNotDirectory(PathBuf),
    /// A directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A directory entry could not be read.
    ReadDirectoryEntry {
        /// Parent directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// File type metadata could not be read.
    ReadFileType {
        /// Entry path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A candidate Designer XML root is invalid.
    DesignerXml {
        /// Candidate root path.
        path: PathBuf,
        /// Format-specific inspection error.
        source: DesignerXmlDiscoveryError,
    },
    /// One directory has both accepted EDT and Designer XML markers.
    ConflictingFormatMarkers(PathBuf),
}

impl Display for DiscoveryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootDoesNotExist(path) => {
                write!(
                    formatter,
                    "workspace root does not exist: {}",
                    path.display()
                )
            }
            Self::RootIsNotDirectory(path) => {
                write!(
                    formatter,
                    "workspace root is not a directory: {}",
                    path.display()
                )
            }
            Self::ReadDirectory { path, source } => {
                write!(
                    formatter,
                    "failed to read directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadDirectoryEntry { path, source } => {
                write!(
                    formatter,
                    "failed to read an entry in {}: {source}",
                    path.display()
                )
            }
            Self::ReadFileType { path, source } => {
                write!(
                    formatter,
                    "failed to read file type for {}: {source}",
                    path.display()
                )
            }
            Self::DesignerXml { path, source } => write!(
                formatter,
                "invalid Designer XML project at {}: {source}",
                path.display()
            ),
            Self::ConflictingFormatMarkers(path) => write!(
                formatter,
                "workspace project has conflicting EDT and Designer XML markers: {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFileType { source, .. } => Some(source),
            Self::DesignerXml { source, .. } => Some(source),
            Self::RootDoesNotExist(_)
            | Self::RootIsNotDirectory(_)
            | Self::ConflictingFormatMarkers(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_workspace::{WorkspaceDetector, WorkspaceFormat};
    use std::fs;
    use tempfile::tempdir;

    use super::FileSystemWorkspaceDetector;

    const DESIGNER_DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;
    const DESIGNER_CONFIGURATION: &str = r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Configuration uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093"><Properties><Name>DNSWorldEdition</Name></Properties></Configuration></MetaDataObject>"#;

    fn write_designer_markers(project: &std::path::Path) {
        fs::write(project.join("ConfigDumpInfo.xml"), DESIGNER_DUMP_INFO)
            .expect("Designer dump marker must be created");
        fs::write(project.join("Configuration.xml"), DESIGNER_CONFIGURATION)
            .expect("Designer configuration marker must be created");
    }

    #[test]
    fn detects_edt_project() {
        let root = tempdir().expect("temporary directory must be created");
        let project = root.path().join("DemoConfiguration");

        fs::create_dir_all(project.join("src/Configuration"))
            .expect("EDT directory structure must be created");
        fs::write(project.join(".project"), "<projectDescription />")
            .expect("EDT project file must be created");
        fs::write(
            project.join("src/Configuration/Configuration.mdo"),
            "<mdclass:Configuration />",
        )
        .expect("configuration metadata file must be created");

        let configurations = FileSystemWorkspaceDetector::default()
            .discover(root.path())
            .expect("discovery must succeed");

        assert_eq!(configurations.len(), 1);
        assert_eq!(configurations[0].root_path(), project);
        assert_eq!(configurations[0].format(), WorkspaceFormat::Edt);
    }

    #[test]
    fn ignores_incomplete_edt_project() {
        let root = tempdir().expect("temporary directory must be created");
        let project = root.path().join("IncompleteProject");

        fs::create_dir_all(&project).expect("project directory must be created");
        fs::write(project.join(".project"), "<projectDescription />")
            .expect("EDT project file must be created");

        let configurations = FileSystemWorkspaceDetector::default()
            .discover(root.path())
            .expect("discovery must succeed");

        assert!(configurations.is_empty());
    }

    #[test]
    fn detects_designer_xml_project_and_stops_at_its_boundary() {
        let root = tempdir().expect("temporary directory must be created");
        let project = root.path().join("DesignerConfiguration");
        let nested_edt = project.join("nested");

        fs::create_dir_all(nested_edt.join("src/Configuration"))
            .expect("nested EDT structure must be created");
        write_designer_markers(&project);
        fs::write(nested_edt.join(".project"), "<projectDescription />")
            .expect("nested EDT marker must be created");
        fs::write(
            nested_edt.join("src/Configuration/Configuration.mdo"),
            "<mdclass:Configuration />",
        )
        .expect("nested EDT configuration must be created");

        let configurations = FileSystemWorkspaceDetector::default()
            .discover(root.path())
            .expect("discovery must succeed");

        assert_eq!(configurations.len(), 1);
        assert_eq!(configurations[0].root_path(), project);
        assert_eq!(configurations[0].format(), WorkspaceFormat::DesignerXml);
    }

    #[test]
    fn rejects_conflicting_edt_and_designer_markers() {
        let root = tempdir().expect("temporary directory must be created");
        let project = root.path().join("ConflictingConfiguration");

        fs::create_dir_all(project.join("src/Configuration"))
            .expect("EDT directory structure must be created");
        write_designer_markers(&project);
        fs::write(project.join(".project"), "<projectDescription />")
            .expect("EDT project file must be created");
        fs::write(
            project.join("src/Configuration/Configuration.mdo"),
            "<mdclass:Configuration />",
        )
        .expect("EDT configuration must be created");

        let error = FileSystemWorkspaceDetector::default()
            .discover(root.path())
            .expect_err("conflicting markers must fail");

        assert!(
            error
                .to_string()
                .contains("conflicting EDT and Designer XML markers")
        );
    }

    #[test]
    fn respects_depth_limit() {
        let root = tempdir().expect("temporary directory must be created");
        let project = root.path().join("one/two/three");

        fs::create_dir_all(project.join("src/Configuration"))
            .expect("EDT directory structure must be created");
        fs::write(project.join(".project"), "<projectDescription />")
            .expect("EDT project file must be created");
        fs::write(
            project.join("src/Configuration/Configuration.mdo"),
            "<mdclass:Configuration />",
        )
        .expect("configuration metadata file must be created");

        let configurations = FileSystemWorkspaceDetector::new(1)
            .discover(root.path())
            .expect("discovery must succeed");

        assert!(configurations.is_empty());
    }
}
