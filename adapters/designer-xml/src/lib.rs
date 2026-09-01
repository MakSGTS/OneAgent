//! Adapter for hierarchical `1C:Enterprise Designer` XML sources.

mod coverage;
mod metadata_object;
mod module_reader;
mod semantic_graph;
mod source_evidence;

pub use coverage::{DesignerXmlSemanticCoverageRegistry, DesignerXmlSemanticCoverageReport};
pub use metadata_object::{
    DesignerXmlMetadataObjectDescriptor, DesignerXmlMetadataObjectError,
    DesignerXmlMetadataObjectReader, DesignerXmlSourceEvidence,
    FileSystemDesignerXmlMetadataObjectReader,
};
pub use module_reader::{
    DesignerXmlModuleDescriptor, DesignerXmlModuleError, DesignerXmlModuleKind,
    DesignerXmlModuleReader, DesignerXmlModuleSourceEvidence, FileSystemDesignerXmlModuleReader,
};
pub use semantic_graph::{
    DesignerXmlGraphError, DesignerXmlSemanticGraphBuildResult, DesignerXmlSemanticGraphBuilder,
    FileSystemDesignerXmlSemanticGraphBuilder,
};
pub use source_evidence::DesignerXmlSourceEvidenceError;

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::{CommonMetadataPayload, MetadataPayload};
use oneagent_workspace::{Configuration, WorkspaceFormat};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const DUMP_INFO_FILE: &str = "ConfigDumpInfo.xml";
const CONFIGURATION_FILE: &str = "Configuration.xml";
const DUMP_INFO_NAMESPACE: &str = "http://v8.1c.ru/8.3/xcf/dumpinfo";
const METADATA_NAMESPACE: &str = "http://v8.1c.ru/8.3/MDClasses";
const SUPPORTED_VERSION: &str = "2.20";

/// Completeness declared by the caller for one Designer XML build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignerXmlBuildScope {
    /// The supplied root represents one complete configuration dump.
    Complete,
    /// The supplied root deliberately contains a subset of configuration artifacts.
    Partial,
}

/// Inspects a directory for the accepted Designer XML project markers.
///
/// # Errors
///
/// Returns a typed error when `ConfigDumpInfo.xml` identifies a candidate root
/// whose required markers are missing, symlinked, unreadable, or incompatible.
pub fn is_designer_xml_project(root: &Path) -> Result<bool, DesignerXmlDiscoveryError> {
    let root_metadata =
        fs::symlink_metadata(root).map_err(|source| DesignerXmlDiscoveryError::InspectPath {
            path: root.to_path_buf(),
            source,
        })?;
    if root_metadata.file_type().is_symlink() {
        return Err(DesignerXmlDiscoveryError::SymlinkArtifact(
            root.to_path_buf(),
        ));
    }
    if !root_metadata.file_type().is_dir() {
        return Err(DesignerXmlDiscoveryError::NotDirectory(root.to_path_buf()));
    }

    let dump_info_path = root.join(DUMP_INFO_FILE);
    if !path_exists_without_following_symlinks(&dump_info_path)? {
        return Ok(false);
    }

    ensure_regular_file(&dump_info_path)?;
    let configuration_path = root.join(CONFIGURATION_FILE);
    if !path_exists_without_following_symlinks(&configuration_path)? {
        return Err(DesignerXmlDiscoveryError::MissingMarker(configuration_path));
    }
    ensure_regular_file(&configuration_path)?;

    let dump_info = read_file(&dump_info_path)?;
    let dump_version = parse_dump_info_marker(&dump_info, &dump_info_path)?;
    let configuration = read_file(&configuration_path)?;
    let configuration_version = parse_configuration_marker(&configuration, &configuration_path)?;

    if dump_version != configuration_version {
        return Err(DesignerXmlDiscoveryError::VersionMismatch {
            dump_info: dump_version,
            configuration: configuration_version,
        });
    }

    Ok(true)
}

/// Port for loading a source-independent configuration from Designer XML.
pub trait DesignerXmlConfigurationLoader {
    /// Loads the root configuration descriptor with an explicit completeness scope.
    ///
    /// # Errors
    ///
    /// Returns an error when markers or accepted configuration content are invalid.
    fn load(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<Configuration, DesignerXmlLoadError>;
}

/// Filesystem implementation of [`DesignerXmlConfigurationLoader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemDesignerXmlConfigurationLoader;

impl FileSystemDesignerXmlConfigurationLoader {
    /// Loads the configuration and its accepted common metadata payload.
    ///
    /// # Errors
    ///
    /// Returns an error when project markers or configuration XML are invalid.
    pub fn load_with_payload(
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<(Configuration, MetadataPayload), DesignerXmlLoadError> {
        if !is_designer_xml_project(project_root)? {
            return Err(DesignerXmlLoadError::MarkersNotFound(
                project_root.to_path_buf(),
            ));
        }

        let configuration_path = project_root.join(CONFIGURATION_FILE);
        let xml = read_file(&configuration_path)?;
        let descriptor = parse_configuration(&xml, &configuration_path)?;
        let id =
            EntityId::new(descriptor.uuid).map_err(|_| DesignerXmlLoadError::InvalidIdentifier)?;
        let name =
            EntityName::new(descriptor.name).map_err(|_| DesignerXmlLoadError::InvalidName)?;
        let payload = MetadataPayload::new(CommonMetadataPayload::new(descriptor.synonym), None);

        let _ = scope;
        Ok((
            Configuration::new(id, name, project_root, WorkspaceFormat::DesignerXml),
            payload,
        ))
    }
}

impl DesignerXmlConfigurationLoader for FileSystemDesignerXmlConfigurationLoader {
    fn load(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<Configuration, DesignerXmlLoadError> {
        Self::load_with_payload(project_root, scope).map(|(configuration, _)| configuration)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ConfigurationDescriptor {
    uuid: String,
    name: String,
    synonym: Option<String>,
}

fn parse_dump_info_marker(xml: &[u8], path: &Path) -> Result<String, DesignerXmlDiscoveryError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) => {
                require_root(&reader, &event, "ConfigDumpInfo", DUMP_INFO_NAMESPACE, path)?;
                let format = required_attribute(&reader, &event, "format", path)?;
                if format != "Hierarchical" {
                    return Err(DesignerXmlDiscoveryError::UnsupportedFormat(format));
                }
                return supported_version(&reader, &event, path);
            }
            Ok(Event::Eof) => {
                return Err(DesignerXmlDiscoveryError::WrongRoot {
                    path: path.to_path_buf(),
                    expected: "ConfigDumpInfo",
                });
            }
            Ok(_) => {}
            Err(source) => return Err(malformed_xml(path, &source)),
        }
    }
}

fn parse_configuration_marker(
    xml: &[u8],
    path: &Path,
) -> Result<String, DesignerXmlDiscoveryError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut root_seen = false;
    let mut depth = 0_usize;
    let mut direct_configuration_count = 0_usize;
    let mut version = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if depth == 1 {
                    require_root(&reader, &event, "MetaDataObject", METADATA_NAMESPACE, path)?;
                    version = Some(supported_version(&reader, &event, path)?);
                    root_seen = true;
                } else if depth == 2 && local_name(event.name().as_ref()) == "Configuration" {
                    direct_configuration_count += 1;
                }
            }
            Ok(Event::Empty(event)) => {
                if depth == 1 && local_name(event.name().as_ref()) == "Configuration" {
                    direct_configuration_count += 1;
                }
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(source) => return Err(malformed_xml(path, &source)),
        }
    }

    if !root_seen {
        return Err(DesignerXmlDiscoveryError::WrongRoot {
            path: path.to_path_buf(),
            expected: "MetaDataObject",
        });
    }
    if direct_configuration_count != 1 {
        return Err(DesignerXmlDiscoveryError::ConfigurationRootCount(
            direct_configuration_count,
        ));
    }

    version.ok_or_else(|| DesignerXmlDiscoveryError::MissingAttribute {
        path: path.to_path_buf(),
        attribute: "version",
    })
}

fn parse_configuration(
    xml: &[u8],
    path: &Path,
) -> Result<ConfigurationDescriptor, DesignerXmlLoadError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut element_path = Vec::<String>::new();
    let mut uuid = None;
    let mut name = None;
    let mut synonym = None;
    let mut properties_count = 0_usize;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                element_path.push(local_name(event.name().as_ref()));
                if path_equals(&element_path, &["MetaDataObject", "Configuration"]) {
                    let value = required_attribute(&reader, &event, "uuid", path)?;
                    set_once(&mut uuid, value, "Configuration.uuid")?;
                } else if path_equals(
                    &element_path,
                    &["MetaDataObject", "Configuration", "Properties"],
                ) {
                    properties_count += 1;
                }
            }
            Ok(Event::Text(event)) => {
                let decoded =
                    event
                        .decode()
                        .map_err(|source| DesignerXmlLoadError::MalformedXml {
                            path: path.to_path_buf(),
                            message: source.to_string(),
                        })?;
                let value = unescape(&decoded)
                    .map_err(|source| DesignerXmlLoadError::MalformedXml {
                        path: path.to_path_buf(),
                        message: source.to_string(),
                    })?
                    .into_owned();

                if path_equals(
                    &element_path,
                    &["MetaDataObject", "Configuration", "Properties", "Name"],
                ) {
                    set_once(&mut name, value, "Configuration.Properties.Name")?;
                } else if is_direct_synonym_content(&element_path) && synonym.is_none() {
                    synonym = non_empty(value);
                }
            }
            Ok(Event::End(_)) => {
                element_path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(source) => {
                return Err(DesignerXmlLoadError::MalformedXml {
                    path: path.to_path_buf(),
                    message: source.to_string(),
                });
            }
        }
    }

    if properties_count != 1 {
        return Err(DesignerXmlLoadError::PropertiesCount(properties_count));
    }
    let uuid = required_non_empty(uuid, "Configuration.uuid")?;
    let name = required_non_empty(name, "Configuration.Properties.Name")?;

    Ok(ConfigurationDescriptor {
        uuid,
        name,
        synonym,
    })
}

fn is_direct_synonym_content(path: &[String]) -> bool {
    path_equals(
        path,
        &[
            "MetaDataObject",
            "Configuration",
            "Properties",
            "Synonym",
            "item",
            "content",
        ],
    )
}

fn path_equals(actual: &[String], expected: &[&str]) -> bool {
    actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
}

fn set_once(
    slot: &mut Option<String>,
    value: String,
    field: &'static str,
) -> Result<(), DesignerXmlLoadError> {
    if slot.replace(value).is_some() {
        return Err(DesignerXmlLoadError::DuplicateField(field));
    }
    Ok(())
}

fn required_non_empty(
    value: Option<String>,
    field: &'static str,
) -> Result<String, DesignerXmlLoadError> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or(DesignerXmlLoadError::MissingField(field))
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn require_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    expected_local_name: &'static str,
    expected_namespace: &'static str,
    path: &Path,
) -> Result<(), DesignerXmlDiscoveryError> {
    if local_name(event.name().as_ref()) != expected_local_name {
        return Err(DesignerXmlDiscoveryError::WrongRoot {
            path: path.to_path_buf(),
            expected: expected_local_name,
        });
    }

    let namespace = required_attribute(reader, event, "xmlns", path)?;
    if namespace != expected_namespace {
        return Err(DesignerXmlDiscoveryError::WrongNamespace {
            path: path.to_path_buf(),
            expected: expected_namespace,
            actual: namespace,
        });
    }
    Ok(())
}

fn supported_version(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    path: &Path,
) -> Result<String, DesignerXmlDiscoveryError> {
    let version = required_attribute(reader, event, "version", path)?;
    if version != SUPPORTED_VERSION {
        return Err(DesignerXmlDiscoveryError::UnsupportedVersion(version));
    }
    Ok(version)
}

fn required_attribute(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &'static str,
    path: &Path,
) -> Result<String, DesignerXmlDiscoveryError> {
    let mut value = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute.map_err(|source| DesignerXmlDiscoveryError::MalformedXml {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;
        if attribute.key.as_ref() == name.as_bytes() {
            let decoded = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|source| DesignerXmlDiscoveryError::MalformedXml {
                    path: path.to_path_buf(),
                    message: source.to_string(),
                })?
                .into_owned();
            if value.replace(decoded).is_some() {
                return Err(DesignerXmlDiscoveryError::DuplicateAttribute {
                    path: path.to_path_buf(),
                    attribute: name,
                });
            }
        }
    }
    value.ok_or_else(|| DesignerXmlDiscoveryError::MissingAttribute {
        path: path.to_path_buf(),
        attribute: name,
    })
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

fn path_exists_without_following_symlinks(path: &Path) -> Result<bool, DesignerXmlDiscoveryError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(DesignerXmlDiscoveryError::InspectPath {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn ensure_regular_file(path: &Path) -> Result<(), DesignerXmlDiscoveryError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|source| DesignerXmlDiscoveryError::InspectPath {
            path: path.to_path_buf(),
            source,
        })?;
    if metadata.file_type().is_symlink() {
        return Err(DesignerXmlDiscoveryError::SymlinkArtifact(
            path.to_path_buf(),
        ));
    }
    if !metadata.file_type().is_file() {
        return Err(DesignerXmlDiscoveryError::NotRegularFile(
            path.to_path_buf(),
        ));
    }
    Ok(())
}

fn read_file(path: &Path) -> Result<Vec<u8>, DesignerXmlDiscoveryError> {
    fs::read(path).map_err(|source| DesignerXmlDiscoveryError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn malformed_xml(path: &Path, source: &quick_xml::Error) -> DesignerXmlDiscoveryError {
    DesignerXmlDiscoveryError::MalformedXml {
        path: path.to_path_buf(),
        message: source.to_string(),
    }
}

/// Errors produced while inspecting Designer XML project markers.
#[derive(Debug)]
pub enum DesignerXmlDiscoveryError {
    /// A candidate root is missing one required marker.
    MissingMarker(PathBuf),
    /// A path could not be inspected.
    InspectPath {
        /// Inspected path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A required artifact is a symlink.
    SymlinkArtifact(PathBuf),
    /// A required artifact is not a regular file.
    NotRegularFile(PathBuf),
    /// The supplied project root is not a directory.
    NotDirectory(PathBuf),
    /// A required artifact could not be read.
    ReadFile {
        /// Artifact path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// XML is malformed.
    MalformedXml {
        /// Artifact path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// The root element is incompatible.
    WrongRoot {
        /// Artifact path.
        path: PathBuf,
        /// Required local root name.
        expected: &'static str,
    },
    /// The default namespace is incompatible.
    WrongNamespace {
        /// Artifact path.
        path: PathBuf,
        /// Required namespace.
        expected: &'static str,
        /// Observed namespace.
        actual: String,
    },
    /// A required attribute is absent.
    MissingAttribute {
        /// Artifact path.
        path: PathBuf,
        /// Required attribute.
        attribute: &'static str,
    },
    /// A root attribute occurs more than once.
    DuplicateAttribute {
        /// Artifact path.
        path: PathBuf,
        /// Duplicated attribute.
        attribute: &'static str,
    },
    /// The dump format is not hierarchical.
    UnsupportedFormat(String),
    /// The XML format version is outside the first slice.
    UnsupportedVersion(String),
    /// Marker versions disagree.
    VersionMismatch {
        /// Dump marker version.
        dump_info: String,
        /// Configuration marker version.
        configuration: String,
    },
    /// The root wrapper has the wrong number of direct Configuration children.
    ConfigurationRootCount(usize),
}

impl Display for DesignerXmlDiscoveryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMarker(path) => {
                write!(
                    formatter,
                    "Designer XML marker is missing: {}",
                    path.display()
                )
            }
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
            Self::NotRegularFile(path) => write!(
                formatter,
                "Designer XML artifact is not a regular file: {}",
                path.display()
            ),
            Self::NotDirectory(path) => write!(
                formatter,
                "Designer XML project root is not a directory: {}",
                path.display()
            ),
            Self::ReadFile { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::MalformedXml { path, message } => {
                write!(
                    formatter,
                    "malformed Designer XML in {}: {message}",
                    path.display()
                )
            }
            Self::WrongRoot { path, expected } => write!(
                formatter,
                "Designer XML root in {} is not {expected}",
                path.display()
            ),
            Self::WrongNamespace {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "Designer XML namespace in {} is {actual}, expected {expected}",
                path.display()
            ),
            Self::MissingAttribute { path, attribute } => write!(
                formatter,
                "Designer XML root in {} is missing {attribute}",
                path.display()
            ),
            Self::DuplicateAttribute { path, attribute } => write!(
                formatter,
                "Designer XML root in {} repeats {attribute}",
                path.display()
            ),
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported Designer XML dump format: {format}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported Designer XML version: {version}")
            }
            Self::VersionMismatch {
                dump_info,
                configuration,
            } => write!(
                formatter,
                "Designer XML marker versions differ: {dump_info} and {configuration}"
            ),
            Self::ConfigurationRootCount(count) => write!(
                formatter,
                "Designer XML Configuration.xml has {count} direct Configuration roots"
            ),
        }
    }
}

impl std::error::Error for DesignerXmlDiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InspectPath { source, .. } | Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Errors produced while loading the Designer XML configuration descriptor.
#[derive(Debug)]
pub enum DesignerXmlLoadError {
    /// The directory does not have accepted Designer XML markers.
    MarkersNotFound(PathBuf),
    /// Project marker inspection failed.
    Discovery(DesignerXmlDiscoveryError),
    /// Configuration XML is malformed.
    MalformedXml {
        /// Configuration path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// One accepted field occurs more than once.
    DuplicateField(&'static str),
    /// The direct Properties container count is invalid.
    PropertiesCount(usize),
    /// One required field is absent or empty.
    MissingField(&'static str),
    /// The UUID cannot form a canonical entity identifier.
    InvalidIdentifier,
    /// The name cannot form a canonical entity name.
    InvalidName,
}

impl From<DesignerXmlDiscoveryError> for DesignerXmlLoadError {
    fn from(value: DesignerXmlDiscoveryError) -> Self {
        Self::Discovery(value)
    }
}

impl Display for DesignerXmlLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MarkersNotFound(path) => write!(
                formatter,
                "Designer XML project markers were not found in {}",
                path.display()
            ),
            Self::Discovery(source) => Display::fmt(source, formatter),
            Self::MalformedXml { path, message } => {
                write!(
                    formatter,
                    "malformed Designer XML in {}: {message}",
                    path.display()
                )
            }
            Self::DuplicateField(field) => {
                write!(formatter, "Designer XML field is duplicated: {field}")
            }
            Self::PropertiesCount(count) => write!(
                formatter,
                "Designer XML Configuration has {count} direct Properties containers"
            ),
            Self::MissingField(field) => {
                write!(formatter, "Designer XML field is missing or empty: {field}")
            }
            Self::InvalidIdentifier => formatter.write_str("Designer XML UUID is invalid"),
            Self::InvalidName => formatter.write_str("Designer XML name is invalid"),
        }
    }
}

impl std::error::Error for DesignerXmlLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesignerXmlBuildScope, DesignerXmlConfigurationLoader, DesignerXmlDiscoveryError,
        FileSystemDesignerXmlConfigurationLoader, is_designer_xml_project,
    };
    use oneagent_workspace::WorkspaceFormat;
    use std::fs;
    use tempfile::tempdir;

    // Source-derived unit reductions retain the accepted roots and fields from
    // OneAgent_DesignerXML/ConfigDumpInfo.xml (SHA-256 b0163f45...) and
    // OneAgent_DesignerXML/Configuration.xml (SHA-256 b7eed83a...). They are
    // parser tests, not the tracked production/conformance fixture owned by Task 7.
    const DUMP_INFO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20">
  <ConfigVersions />
</ConfigDumpInfo>
"#;
    const CONFIGURATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" xmlns:v8="http://v8.1c.ru/8.1/data/core" version="2.20">
  <Configuration uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093">
    <Properties>
      <Name>DNSWorldEdition</Name>
      <Synonym><v8:item><v8:lang>ru</v8:lang><v8:content>DNS: WE</v8:content></v8:item></Synonym>
    </Properties>
  </Configuration>
</MetaDataObject>
"#;

    fn write_project(root: &std::path::Path) {
        fs::write(root.join("ConfigDumpInfo.xml"), DUMP_INFO).expect("dump marker must be written");
        fs::write(root.join("Configuration.xml"), CONFIGURATION)
            .expect("configuration must be written");
    }

    #[test]
    fn detects_hierarchical_designer_xml_project() {
        let project = tempdir().expect("temporary project must be created");
        write_project(project.path());

        assert!(is_designer_xml_project(project.path()).expect("project must be inspected"));
    }

    #[test]
    fn ignores_directory_without_dump_marker() {
        let project = tempdir().expect("temporary project must be created");
        fs::write(project.path().join("Configuration.xml"), CONFIGURATION)
            .expect("configuration must be written");

        assert!(!is_designer_xml_project(project.path()).expect("directory must be inspected"));
    }

    #[test]
    fn rejects_candidate_without_configuration_marker() {
        let project = tempdir().expect("temporary project must be created");
        fs::write(project.path().join("ConfigDumpInfo.xml"), DUMP_INFO)
            .expect("dump marker must be written");

        assert!(matches!(
            is_designer_xml_project(project.path()),
            Err(DesignerXmlDiscoveryError::MissingMarker(_))
        ));
    }

    #[test]
    fn rejects_unsupported_dump_version() {
        let project = tempdir().expect("temporary project must be created");
        write_project(project.path());
        fs::write(
            project.path().join("ConfigDumpInfo.xml"),
            DUMP_INFO.replace("2.20", "2.19"),
        )
        .expect("changed dump marker must be written");

        assert!(matches!(
            is_designer_xml_project(project.path()),
            Err(DesignerXmlDiscoveryError::UnsupportedVersion(version)) if version == "2.19"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_marker() {
        use std::os::unix::fs::symlink;

        let project = tempdir().expect("temporary project must be created");
        let source = project.path().join("dump-info-source.xml");
        fs::write(&source, DUMP_INFO).expect("source marker must be written");
        symlink(&source, project.path().join("ConfigDumpInfo.xml"))
            .expect("marker symlink must be created");
        fs::write(project.path().join("Configuration.xml"), CONFIGURATION)
            .expect("configuration must be written");

        assert!(matches!(
            is_designer_xml_project(project.path()),
            Err(DesignerXmlDiscoveryError::SymlinkArtifact(_))
        ));
    }

    #[test]
    fn loads_source_independent_configuration_and_payload() {
        let project = tempdir().expect("temporary project must be created");
        write_project(project.path());

        let configuration = FileSystemDesignerXmlConfigurationLoader
            .load(project.path(), DesignerXmlBuildScope::Complete)
            .expect("configuration must load");
        let (_, payload) = FileSystemDesignerXmlConfigurationLoader::load_with_payload(
            project.path(),
            DesignerXmlBuildScope::Partial,
        )
        .expect("partial configuration must load");

        assert_eq!(
            configuration.id().as_str(),
            "408a41e7-907a-4fb3-8999-83d1e8b6e093"
        );
        assert_eq!(configuration.name().as_str(), "DNSWorldEdition");
        assert_eq!(configuration.format(), WorkspaceFormat::DesignerXml);
        assert_eq!(payload.common().synonym(), Some("DNS: WE"));
    }

    #[test]
    fn unrelated_element_order_and_repeated_loads_are_equivalent() {
        let canonical = tempdir().expect("canonical project must be created");
        let reordered = tempdir().expect("reordered project must be created");
        write_project(canonical.path());
        write_project(reordered.path());
        let reordered_configuration =
            CONFIGURATION.replace("<Name>DNSWorldEdition</Name>\n      <Synonym>", "<Synonym>");
        let reordered_configuration = reordered_configuration.replace(
            "</Synonym>\n    </Properties>",
            "</Synonym>\n      <Name>DNSWorldEdition</Name>\n    </Properties>",
        );
        fs::write(
            reordered.path().join("Configuration.xml"),
            reordered_configuration,
        )
        .expect("reordered configuration must be written");

        let first = FileSystemDesignerXmlConfigurationLoader::load_with_payload(
            canonical.path(),
            DesignerXmlBuildScope::Complete,
        )
        .expect("canonical configuration must load");
        let repeated = FileSystemDesignerXmlConfigurationLoader::load_with_payload(
            canonical.path(),
            DesignerXmlBuildScope::Complete,
        )
        .expect("canonical configuration must load repeatedly");
        let reordered = FileSystemDesignerXmlConfigurationLoader::load_with_payload(
            reordered.path(),
            DesignerXmlBuildScope::Complete,
        )
        .expect("reordered configuration must load");

        assert_eq!(first.0.id(), repeated.0.id());
        assert_eq!(first.0.name(), repeated.0.name());
        assert_eq!(first.1, repeated.1);
        assert_eq!(first.0.id(), reordered.0.id());
        assert_eq!(first.0.name(), reordered.0.name());
        assert_eq!(first.1, reordered.1);
    }

    #[test]
    fn rejects_duplicate_configuration_name() {
        let project = tempdir().expect("temporary project must be created");
        write_project(project.path());
        let duplicate = CONFIGURATION.replace(
            "<Name>DNSWorldEdition</Name>",
            "<Name>DNSWorldEdition</Name><Name>Duplicate</Name>",
        );
        fs::write(project.path().join("Configuration.xml"), duplicate)
            .expect("changed configuration must be written");

        let error = FileSystemDesignerXmlConfigurationLoader
            .load(project.path(), DesignerXmlBuildScope::Complete)
            .expect_err("duplicate name must fail");

        assert_eq!(
            error.to_string(),
            "Designer XML field is duplicated: Configuration.Properties.Name"
        );
    }
}
