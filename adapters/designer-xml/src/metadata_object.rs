//! Reader for accepted top-level Designer XML metadata descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::{CommonMetadataPayload, MetadataKind, MetadataPayload};
use quick_xml::NsReader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{DesignerXmlBuildScope, DesignerXmlDiscoveryError, is_designer_xml_project};

const METADATA_NAMESPACE: &str = "http://v8.1c.ru/8.3/MDClasses";
const CORE_NAMESPACE: &str = "http://v8.1c.ru/8.1/data/core";
const SUPPORTED_VERSION: &str = "2.20";

#[derive(Debug, Clone, Copy)]
pub(crate) struct FamilySpec {
    pub(crate) directory: &'static str,
    pub(crate) root: &'static str,
    pub(crate) kind: MetadataKind,
}

pub(crate) const ACCEPTED_FAMILIES: [FamilySpec; 20] = [
    FamilySpec {
        directory: "Catalogs",
        root: "Catalog",
        kind: MetadataKind::Catalog,
    },
    FamilySpec {
        directory: "Documents",
        root: "Document",
        kind: MetadataKind::Document,
    },
    FamilySpec {
        directory: "Enums",
        root: "Enum",
        kind: MetadataKind::Enumeration,
    },
    FamilySpec {
        directory: "CommonModules",
        root: "CommonModule",
        kind: MetadataKind::CommonModule,
    },
    FamilySpec {
        directory: "Reports",
        root: "Report",
        kind: MetadataKind::Report,
    },
    FamilySpec {
        directory: "DataProcessors",
        root: "DataProcessor",
        kind: MetadataKind::DataProcessor,
    },
    FamilySpec {
        directory: "InformationRegisters",
        root: "InformationRegister",
        kind: MetadataKind::InformationRegister,
    },
    FamilySpec {
        directory: "AccumulationRegisters",
        root: "AccumulationRegister",
        kind: MetadataKind::AccumulationRegister,
    },
    FamilySpec {
        directory: "AccountingRegisters",
        root: "AccountingRegister",
        kind: MetadataKind::AccountingRegister,
    },
    FamilySpec {
        directory: "BusinessProcesses",
        root: "BusinessProcess",
        kind: MetadataKind::BusinessProcess,
    },
    FamilySpec {
        directory: "Tasks",
        root: "Task",
        kind: MetadataKind::Task,
    },
    FamilySpec {
        directory: "Roles",
        root: "Role",
        kind: MetadataKind::Role,
    },
    FamilySpec {
        directory: "CommonCommands",
        root: "CommonCommand",
        kind: MetadataKind::Command,
    },
    FamilySpec {
        directory: "CommonForms",
        root: "CommonForm",
        kind: MetadataKind::CommonForm,
    },
    FamilySpec {
        directory: "CommonTemplates",
        root: "CommonTemplate",
        kind: MetadataKind::Template,
    },
    FamilySpec {
        directory: "HTTPServices",
        root: "HTTPService",
        kind: MetadataKind::HttpService,
    },
    FamilySpec {
        directory: "WebServices",
        root: "WebService",
        kind: MetadataKind::WebService,
    },
    FamilySpec {
        directory: "XDTOPackages",
        root: "XDTOPackage",
        kind: MetadataKind::XdtoPackage,
    },
    FamilySpec {
        directory: "Subsystems",
        root: "Subsystem",
        kind: MetadataKind::Subsystem,
    },
    FamilySpec {
        directory: "EventSubscriptions",
        root: "EventSubscription",
        kind: MetadataKind::EventSubscription,
    },
];

/// Adapter-local evidence for one parsed Designer XML descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerXmlSourceEvidence {
    artifact_path: PathBuf,
    xml_version: &'static str,
    raw_byte_len: u64,
}

impl DesignerXmlSourceEvidence {
    /// Returns the exact source artifact path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the accepted Designer XML serialization version.
    #[must_use]
    pub const fn xml_version(&self) -> &'static str {
        self.xml_version
    }

    /// Returns the number of raw source bytes read from the artifact.
    #[must_use]
    pub const fn raw_byte_len(&self) -> u64 {
        self.raw_byte_len
    }
}

/// Parsed source-independent content of one top-level Designer XML object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerXmlMetadataObjectDescriptor {
    id: EntityId,
    name: EntityName,
    kind: MetadataKind,
    payload: MetadataPayload,
    source: DesignerXmlSourceEvidence,
}

impl DesignerXmlMetadataObjectDescriptor {
    /// Returns the stable source UUID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact canonical metadata name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the accepted source-independent metadata kind.
    #[must_use]
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }

    /// Returns accepted source-independent metadata content.
    #[must_use]
    pub const fn payload(&self) -> &MetadataPayload {
        &self.payload
    }

    /// Returns adapter-local source evidence.
    #[must_use]
    pub const fn source(&self) -> &DesignerXmlSourceEvidence {
        &self.source
    }
}

/// Reads all accepted top-level metadata descriptors from a Designer XML root.
pub trait DesignerXmlMetadataObjectReader {
    /// Reads and canonically orders all accepted descriptors.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the root or any supplied accepted artifact is invalid.
    fn read_all(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<Vec<DesignerXmlMetadataObjectDescriptor>, DesignerXmlMetadataObjectError>;
}

/// Filesystem implementation of [`DesignerXmlMetadataObjectReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemDesignerXmlMetadataObjectReader;

impl DesignerXmlMetadataObjectReader for FileSystemDesignerXmlMetadataObjectReader {
    fn read_all(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<Vec<DesignerXmlMetadataObjectDescriptor>, DesignerXmlMetadataObjectError> {
        if !is_designer_xml_project(project_root)? {
            return Err(DesignerXmlMetadataObjectError::MarkersNotFound(
                project_root.to_path_buf(),
            ));
        }

        let mut descriptors = Vec::new();
        for family in ACCEPTED_FAMILIES {
            descriptors.extend(read_family(project_root, family)?);
        }

        descriptors.sort_by(|left, right| {
            (left.kind, &left.name, &left.id, &left.source.artifact_path).cmp(&(
                right.kind,
                &right.name,
                &right.id,
                &right.source.artifact_path,
            ))
        });
        validate_unique_descriptors(&descriptors)?;

        let _ = scope;
        Ok(descriptors)
    }
}

fn read_family(
    project_root: &Path,
    family: FamilySpec,
) -> Result<Vec<DesignerXmlMetadataObjectDescriptor>, DesignerXmlMetadataObjectError> {
    let family_path = project_root.join(family.directory);
    let metadata = match fs::symlink_metadata(&family_path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(DesignerXmlMetadataObjectError::InspectPath {
                path: family_path,
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(DesignerXmlMetadataObjectError::SymlinkArtifact(family_path));
    }
    if !metadata.file_type().is_dir() {
        return Err(DesignerXmlMetadataObjectError::FamilyNotDirectory(
            family_path,
        ));
    }

    let entries = fs::read_dir(&family_path).map_err(|source| {
        DesignerXmlMetadataObjectError::ReadDirectory {
            path: family_path.clone(),
            source,
        }
    })?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(
            |source| DesignerXmlMetadataObjectError::ReadDirectoryEntry {
                path: family_path.clone(),
                source,
            },
        )?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("xml") {
            continue;
        }
        let file_type =
            entry
                .file_type()
                .map_err(|source| DesignerXmlMetadataObjectError::InspectPath {
                    path: path.clone(),
                    source,
                })?;
        if file_type.is_symlink() {
            return Err(DesignerXmlMetadataObjectError::SymlinkArtifact(path));
        }
        if !file_type.is_file() {
            return Err(DesignerXmlMetadataObjectError::ArtifactNotRegularFile(path));
        }
        candidates.push(path);
    }
    candidates.sort();

    candidates
        .into_iter()
        .map(|path| read_descriptor(path, family))
        .collect()
}

fn read_descriptor(
    path: PathBuf,
    family: FamilySpec,
) -> Result<DesignerXmlMetadataObjectDescriptor, DesignerXmlMetadataObjectError> {
    let raw = fs::read(&path).map_err(|source| DesignerXmlMetadataObjectError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let raw_byte_len = raw
        .len()
        .try_into()
        .expect("usize must fit into u64 on supported targets");
    let parsed = parse_descriptor(&raw, &path, family)?;
    let filename = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| DesignerXmlMetadataObjectError::InvalidFilename(path.clone()))?
        .to_owned();
    if filename != parsed.name {
        return Err(DesignerXmlMetadataObjectError::FilenameNameMismatch {
            path,
            filename,
            declared: parsed.name,
        });
    }

    let id = EntityId::new(parsed.uuid)
        .map_err(|_| DesignerXmlMetadataObjectError::InvalidIdentifier(path.clone()))?;
    let name = EntityName::new(parsed.name)
        .map_err(|_| DesignerXmlMetadataObjectError::InvalidName(path.clone()))?;
    Ok(DesignerXmlMetadataObjectDescriptor {
        id,
        name,
        kind: family.kind,
        payload: MetadataPayload::new(CommonMetadataPayload::new(parsed.synonym), None),
        source: DesignerXmlSourceEvidence {
            artifact_path: path,
            xml_version: SUPPORTED_VERSION,
            raw_byte_len,
        },
    })
}

#[derive(Debug)]
struct ParsedDescriptor {
    uuid: String,
    name: String,
    synonym: Option<String>,
}

#[allow(clippy::too_many_lines)]
fn parse_descriptor(
    xml: &[u8],
    artifact_path: &Path,
    family: FamilySpec,
) -> Result<ParsedDescriptor, DesignerXmlMetadataObjectError> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut path = Vec::<String>::new();
    let mut direct_children = Vec::new();
    let mut uuid = None;
    let mut properties_count = 0_usize;
    let mut name_count = 0_usize;
    let mut name = None;
    let mut synonym = None;
    let mut root_seen = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let element = local_name(event.name().as_ref());
                if path.is_empty() {
                    validate_wrapper(&reader, &event, artifact_path)?;
                    root_seen = true;
                } else if path.len() == 1 {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    direct_children.push(element.clone());
                    if element == family.root {
                        uuid = Some(required_attribute(&reader, &event, "uuid", artifact_path)?);
                    }
                } else if path_equals(&path, &["MetaDataObject", family.root])
                    && element == "Properties"
                {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    properties_count += 1;
                } else if path_equals(&path, &["MetaDataObject", family.root, "Properties"])
                    && element == "Name"
                {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    name_count += 1;
                } else if is_accepted_synonym_element(&path, family.root, &element) {
                    require_element_namespace(
                        &reader,
                        &event,
                        synonym_namespace(&element),
                        artifact_path,
                    )?;
                }
                path.push(element);
            }
            Ok(Event::Empty(event)) => {
                let element = local_name(event.name().as_ref());
                if path.is_empty() {
                    validate_wrapper(&reader, &event, artifact_path)?;
                    root_seen = true;
                } else if path.len() == 1 {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    direct_children.push(element);
                } else if path_equals(&path, &["MetaDataObject", family.root])
                    && element == "Properties"
                {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    properties_count += 1;
                } else if path_equals(&path, &["MetaDataObject", family.root, "Properties"])
                    && element == "Name"
                {
                    require_element_namespace(&reader, &event, METADATA_NAMESPACE, artifact_path)?;
                    name_count += 1;
                } else if is_accepted_synonym_element(&path, family.root, &element) {
                    require_element_namespace(
                        &reader,
                        &event,
                        synonym_namespace(&element),
                        artifact_path,
                    )?;
                }
            }
            Ok(Event::Text(event)) => {
                let decoded = event.decode().map_err(|source| {
                    DesignerXmlMetadataObjectError::MalformedXml {
                        path: artifact_path.to_path_buf(),
                        message: source.to_string(),
                    }
                })?;
                let value = unescape(&decoded)
                    .map_err(|source| DesignerXmlMetadataObjectError::MalformedXml {
                        path: artifact_path.to_path_buf(),
                        message: source.to_string(),
                    })?
                    .into_owned();
                if path_equals(
                    &path,
                    &["MetaDataObject", family.root, "Properties", "Name"],
                ) {
                    if name.replace(value).is_some() {
                        return Err(DesignerXmlMetadataObjectError::DuplicateField {
                            path: artifact_path.to_path_buf(),
                            field: "Properties.Name",
                        });
                    }
                } else if is_direct_synonym_content(&path, family.root) && synonym.is_none() {
                    synonym = (!value.trim().is_empty()).then_some(value);
                }
            }
            Ok(Event::End(_)) => {
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(source) => {
                return Err(DesignerXmlMetadataObjectError::MalformedXml {
                    path: artifact_path.to_path_buf(),
                    message: source.to_string(),
                });
            }
        }
    }

    if !root_seen {
        return Err(DesignerXmlMetadataObjectError::WrongRoot {
            path: artifact_path.to_path_buf(),
            expected: "MetaDataObject",
        });
    }
    if direct_children.as_slice() != [family.root] {
        return Err(DesignerXmlMetadataObjectError::RootKindMismatch {
            path: artifact_path.to_path_buf(),
            expected: family.root,
            actual: direct_children,
        });
    }
    if properties_count != 1 {
        return Err(DesignerXmlMetadataObjectError::PropertiesCount {
            path: artifact_path.to_path_buf(),
            count: properties_count,
        });
    }
    if name_count != 1 {
        return Err(DesignerXmlMetadataObjectError::NameCount {
            path: artifact_path.to_path_buf(),
            count: name_count,
        });
    }
    let uuid = required_non_empty(uuid, artifact_path, "uuid")?;
    let name = required_non_empty(name, artifact_path, "Properties.Name")?;

    Ok(ParsedDescriptor {
        uuid,
        name,
        synonym,
    })
}

fn validate_wrapper(
    reader: &NsReader<&[u8]>,
    event: &BytesStart<'_>,
    path: &Path,
) -> Result<(), DesignerXmlMetadataObjectError> {
    if local_name(event.name().as_ref()) != "MetaDataObject" {
        return Err(DesignerXmlMetadataObjectError::WrongRoot {
            path: path.to_path_buf(),
            expected: "MetaDataObject",
        });
    }
    require_element_namespace(reader, event, METADATA_NAMESPACE, path)?;
    let namespace = required_attribute(reader, event, "xmlns", path)?;
    if namespace != METADATA_NAMESPACE {
        return Err(DesignerXmlMetadataObjectError::WrongNamespace {
            path: path.to_path_buf(),
            expected: METADATA_NAMESPACE,
            actual: namespace,
        });
    }
    let version = required_attribute(reader, event, "version", path)?;
    if version != SUPPORTED_VERSION {
        return Err(DesignerXmlMetadataObjectError::UnsupportedVersion {
            path: path.to_path_buf(),
            version,
        });
    }
    Ok(())
}

fn required_attribute(
    reader: &NsReader<&[u8]>,
    event: &BytesStart<'_>,
    name: &'static str,
    path: &Path,
) -> Result<String, DesignerXmlMetadataObjectError> {
    let mut value = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute =
            attribute.map_err(|source| DesignerXmlMetadataObjectError::MalformedXml {
                path: path.to_path_buf(),
                message: source.to_string(),
            })?;
        if attribute.key.as_ref() == name.as_bytes() {
            let decoded = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|source| DesignerXmlMetadataObjectError::MalformedXml {
                    path: path.to_path_buf(),
                    message: source.to_string(),
                })?
                .into_owned();
            if value.replace(decoded).is_some() {
                return Err(DesignerXmlMetadataObjectError::DuplicateAttribute {
                    path: path.to_path_buf(),
                    attribute: name,
                });
            }
        }
    }
    value.ok_or_else(|| DesignerXmlMetadataObjectError::MissingAttribute {
        path: path.to_path_buf(),
        attribute: name,
    })
}

fn require_element_namespace(
    reader: &NsReader<&[u8]>,
    event: &BytesStart<'_>,
    expected: &'static str,
    path: &Path,
) -> Result<(), DesignerXmlMetadataObjectError> {
    let (namespace, _) = reader.resolve_element(event.name());
    let actual = match namespace {
        ResolveResult::Bound(namespace) => String::from_utf8_lossy(namespace.as_ref()).into_owned(),
        ResolveResult::Unbound => String::new(),
        ResolveResult::Unknown(prefix) => {
            format!("unresolved prefix {}", String::from_utf8_lossy(&prefix))
        }
    };
    if actual != expected {
        return Err(DesignerXmlMetadataObjectError::WrongNamespace {
            path: path.to_path_buf(),
            expected,
            actual,
        });
    }
    Ok(())
}

fn required_non_empty(
    value: Option<String>,
    path: &Path,
    field: &'static str,
) -> Result<String, DesignerXmlMetadataObjectError> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| DesignerXmlMetadataObjectError::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

fn is_direct_synonym_content(path: &[String], root: &str) -> bool {
    path_equals(
        path,
        &[
            "MetaDataObject",
            root,
            "Properties",
            "Synonym",
            "item",
            "content",
        ],
    )
}

fn is_accepted_synonym_element(path: &[String], root: &str, element: &str) -> bool {
    (path_equals(path, &["MetaDataObject", root, "Properties"]) && element == "Synonym")
        || (path_equals(path, &["MetaDataObject", root, "Properties", "Synonym"])
            && element == "item")
        || (path_equals(
            path,
            &["MetaDataObject", root, "Properties", "Synonym", "item"],
        ) && matches!(element, "lang" | "content"))
}

fn synonym_namespace(element: &str) -> &'static str {
    if element == "Synonym" {
        METADATA_NAMESPACE
    } else {
        CORE_NAMESPACE
    }
}

fn path_equals(actual: &[String], expected: &[&str]) -> bool {
    actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

fn validate_unique_descriptors(
    descriptors: &[DesignerXmlMetadataObjectDescriptor],
) -> Result<(), DesignerXmlMetadataObjectError> {
    let mut keys = BTreeSet::new();
    let mut identities = BTreeMap::<&EntityId, &Path>::new();
    for descriptor in descriptors {
        if !keys.insert((descriptor.kind, &descriptor.name)) {
            return Err(DesignerXmlMetadataObjectError::DuplicateKey {
                kind: descriptor.kind,
                name: descriptor.name.clone(),
            });
        }
        if let Some(first_path) =
            identities.insert(&descriptor.id, descriptor.source.artifact_path())
        {
            return Err(DesignerXmlMetadataObjectError::DuplicateIdentifier {
                id: descriptor.id.clone(),
                first_path: first_path.to_path_buf(),
                second_path: descriptor.source.artifact_path.clone(),
            });
        }
    }
    Ok(())
}

/// Errors produced while enumerating or parsing Designer XML metadata objects.
#[derive(Debug)]
pub enum DesignerXmlMetadataObjectError {
    /// Project marker validation failed.
    Discovery(DesignerXmlDiscoveryError),
    /// The supplied root has no Designer XML marker pair.
    MarkersNotFound(PathBuf),
    /// A filesystem path could not be inspected.
    InspectPath {
        /// Inspected path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An accepted family path is a symlink.
    SymlinkArtifact(PathBuf),
    /// An accepted family path is not a directory.
    FamilyNotDirectory(PathBuf),
    /// An accepted direct XML artifact is not a regular file.
    ArtifactNotRegularFile(PathBuf),
    /// An accepted family directory could not be read.
    ReadDirectory {
        /// Family path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An accepted family directory entry could not be read.
    ReadDirectoryEntry {
        /// Family path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An accepted descriptor could not be read.
    ReadFile {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An accepted descriptor filename is not valid UTF-8.
    InvalidFilename(PathBuf),
    /// XML is malformed or cannot be decoded.
    MalformedXml {
        /// Descriptor path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// The descriptor wrapper has the wrong local root name.
    WrongRoot {
        /// Descriptor path.
        path: PathBuf,
        /// Required local root name.
        expected: &'static str,
    },
    /// The descriptor wrapper has the wrong default namespace.
    WrongNamespace {
        /// Descriptor path.
        path: PathBuf,
        /// Required namespace.
        expected: &'static str,
        /// Observed namespace.
        actual: String,
    },
    /// The descriptor uses an unsupported serialization version.
    UnsupportedVersion {
        /// Descriptor path.
        path: PathBuf,
        /// Observed version.
        version: String,
    },
    /// The descriptor has an incompatible set of direct kind children.
    RootKindMismatch {
        /// Descriptor path.
        path: PathBuf,
        /// Required direct kind child.
        expected: &'static str,
        /// Observed direct children in source order.
        actual: Vec<String>,
    },
    /// A required XML attribute is missing.
    MissingAttribute {
        /// Descriptor path.
        path: PathBuf,
        /// Required attribute.
        attribute: &'static str,
    },
    /// An XML attribute is repeated.
    DuplicateAttribute {
        /// Descriptor path.
        path: PathBuf,
        /// Repeated attribute.
        attribute: &'static str,
    },
    /// A metadata object has the wrong number of direct Properties containers.
    PropertiesCount {
        /// Descriptor path.
        path: PathBuf,
        /// Observed count.
        count: usize,
    },
    /// A metadata object has the wrong number of direct Name elements.
    NameCount {
        /// Descriptor path.
        path: PathBuf,
        /// Observed count.
        count: usize,
    },
    /// A scalar field produced more than one text value.
    DuplicateField {
        /// Descriptor path.
        path: PathBuf,
        /// Repeated field.
        field: &'static str,
    },
    /// A required field is absent or empty.
    MissingField {
        /// Descriptor path.
        path: PathBuf,
        /// Missing field.
        field: &'static str,
    },
    /// The filename and declared metadata name differ.
    FilenameNameMismatch {
        /// Descriptor path.
        path: PathBuf,
        /// Direct filename stem.
        filename: String,
        /// Declared exact name.
        declared: String,
    },
    /// The declared source UUID is empty.
    InvalidIdentifier(PathBuf),
    /// The declared metadata name is empty.
    InvalidName(PathBuf),
    /// Two descriptors have the same canonical kind/name key.
    DuplicateKey {
        /// Duplicated metadata kind.
        kind: MetadataKind,
        /// Duplicated exact name.
        name: EntityName,
    },
    /// Two descriptors declare the same stable source UUID.
    DuplicateIdentifier {
        /// Duplicated identifier.
        id: EntityId,
        /// First path in canonical order.
        first_path: PathBuf,
        /// Second path in canonical order.
        second_path: PathBuf,
    },
}

impl From<DesignerXmlDiscoveryError> for DesignerXmlMetadataObjectError {
    fn from(value: DesignerXmlDiscoveryError) -> Self {
        Self::Discovery(value)
    }
}

impl Display for DesignerXmlMetadataObjectError {
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
            Self::ArtifactNotRegularFile(path) => write!(
                formatter,
                "Designer XML metadata artifact is not a regular file: {}",
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
            Self::InvalidFilename(path) => write!(
                formatter,
                "Designer XML descriptor has an invalid filename: {}",
                path.display()
            ),
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
            Self::UnsupportedVersion { path, version } => write!(
                formatter,
                "unsupported Designer XML version {version} in {}",
                path.display()
            ),
            Self::RootKindMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "Designer XML descriptor {} has direct children {actual:?}, expected [{expected}]",
                path.display()
            ),
            Self::MissingAttribute { path, attribute } => write!(
                formatter,
                "Designer XML descriptor {} is missing {attribute}",
                path.display()
            ),
            Self::DuplicateAttribute { path, attribute } => write!(
                formatter,
                "Designer XML descriptor {} repeats {attribute}",
                path.display()
            ),
            Self::PropertiesCount { path, count } => write!(
                formatter,
                "Designer XML descriptor {} has {count} Properties containers, expected one",
                path.display()
            ),
            Self::NameCount { path, count } => write!(
                formatter,
                "Designer XML descriptor {} has {count} Name elements, expected one",
                path.display()
            ),
            Self::DuplicateField { path, field } => write!(
                formatter,
                "Designer XML descriptor {} repeats {field}",
                path.display()
            ),
            Self::MissingField { path, field } => write!(
                formatter,
                "Designer XML descriptor {} is missing non-empty {field}",
                path.display()
            ),
            Self::FilenameNameMismatch {
                path,
                filename,
                declared,
            } => write!(
                formatter,
                "Designer XML descriptor {} has filename {filename} but declares {declared}",
                path.display()
            ),
            Self::InvalidIdentifier(path) => write!(
                formatter,
                "Designer XML descriptor {} declares an invalid identifier",
                path.display()
            ),
            Self::InvalidName(path) => write!(
                formatter,
                "Designer XML descriptor {} declares an invalid name",
                path.display()
            ),
            Self::DuplicateKey { kind, name } => {
                write!(
                    formatter,
                    "duplicate Designer XML metadata key {kind}/{name}"
                )
            }
            Self::DuplicateIdentifier {
                id,
                first_path,
                second_path,
            } => write!(
                formatter,
                "duplicate Designer XML identifier {id} in {} and {}",
                first_path.display(),
                second_path.display()
            ),
        }
    }
}

impl std::error::Error for DesignerXmlMetadataObjectError {
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
        ACCEPTED_FAMILIES, DesignerXmlMetadataObjectError, DesignerXmlMetadataObjectReader,
        FileSystemDesignerXmlMetadataObjectReader,
    };
    use crate::DesignerXmlBuildScope;
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    const DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;
    const CONFIGURATION: &str = r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Configuration uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093"><Properties><Name>DNSWorldEdition</Name></Properties></Configuration></MetaDataObject>"#;
    const PRODUCTS: &[u8] = include_bytes!("../tests/fixtures/metadata/Catalogs/Products.xml");

    fn write_project(root: &Path) {
        fs::write(root.join("ConfigDumpInfo.xml"), DUMP_INFO).expect("dump marker must be created");
        fs::write(root.join("Configuration.xml"), CONFIGURATION)
            .expect("configuration marker must be created");
    }

    fn descriptor(root: &str, uuid: &str, name: &str, synonym: &str) -> String {
        format!(
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><{root} uuid="{uuid}"><Properties><Name>{name}</Name><Synonym><v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core"><v8:lang>en</v8:lang><v8:content>{synonym}</v8:content></v8:item></Synonym></Properties></{root}></MetaDataObject>"#
        )
    }

    #[test]
    fn parses_exact_real_catalog_fixture_with_provenance() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        fs::write(root.path().join("Catalogs/Products.xml"), PRODUCTS)
            .expect("fixture must be written");

        let objects = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect("real descriptor must parse");

        assert_eq!(objects.len(), 1);
        assert_eq!(
            objects[0].id().as_str(),
            "92bcb692-56c4-4199-bf7e-e33cdd76a310"
        );
        assert_eq!(objects[0].name().as_str(), "Products");
        assert_eq!(objects[0].kind(), MetadataKind::Catalog);
        assert_eq!(
            objects[0].payload().common().synonym(),
            Some("Номенклатура")
        );
        assert_eq!(objects[0].source().xml_version(), "2.20");
        assert_eq!(objects[0].source().raw_byte_len(), 76_819);
    }

    #[test]
    fn enumerates_all_twenty_families_in_canonical_order() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        for (index, family) in ACCEPTED_FAMILIES.iter().enumerate() {
            let name = format!("Object{index:02}");
            let family_path = root.path().join(family.directory);
            fs::create_dir(&family_path).expect("family must be created");
            fs::write(
                family_path.join(format!("{name}.xml")),
                descriptor(
                    family.root,
                    &format!("00000000-0000-0000-0000-{index:012}"),
                    &name,
                    &format!("Synonym {index}"),
                ),
            )
            .expect("descriptor must be written");
        }

        let first = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Complete)
            .expect("all accepted families must parse");
        let second = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Complete)
            .expect("repeated parsing must succeed");

        assert_eq!(first.len(), 20);
        assert_eq!(first, second);
        assert!(
            first
                .windows(2)
                .all(|pair| pair[0].kind() <= pair[1].kind())
        );
    }

    #[test]
    fn ignores_unknown_deferred_and_nested_artifacts() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir_all(root.path().join("CalculationRegisters/Future/Ext"))
            .expect("deferred family must be created");
        fs::write(
            root.path().join("CalculationRegisters/Future.xml"),
            descriptor(
                "CalculationRegister",
                "d255fbc0-033d-4531-8593-0dbd0881959c",
                "Future",
                "Future",
            ),
        )
        .expect("deferred descriptor must be written");
        fs::create_dir_all(root.path().join("Catalogs/Products/Ext"))
            .expect("nested artifact directory must be created");
        fs::write(
            root.path().join("Catalogs/Products/Ext/Form.xml"),
            "<unsupported />",
        )
        .expect("nested artifact must be written");

        let objects = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect("deferred artifacts must be ignored");

        assert!(objects.is_empty());
    }

    #[test]
    fn rejects_filename_name_and_root_kind_mismatches() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        fs::write(
            root.path().join("Catalogs/Other.xml"),
            descriptor(
                "Catalog",
                "92bcb692-56c4-4199-bf7e-e33cdd76a310",
                "Products",
                "Products",
            ),
        )
        .expect("descriptor must be written");

        let error = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect_err("name mismatch must fail");
        assert!(matches!(
            error,
            DesignerXmlMetadataObjectError::FilenameNameMismatch { .. }
        ));

        fs::write(
            root.path().join("Catalogs/Other.xml"),
            descriptor(
                "Document",
                "92bcb692-56c4-4199-bf7e-e33cdd76a310",
                "Other",
                "Other",
            ),
        )
        .expect("descriptor must be replaced");
        let error = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect_err("root kind mismatch must fail");
        assert!(matches!(
            error,
            DesignerXmlMetadataObjectError::RootKindMismatch { .. }
        ));
    }

    #[test]
    fn rejects_unsupported_version_and_foreign_kind_namespace() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        let path = root.path().join("Catalogs/Products.xml");
        let unsupported = descriptor(
            "Catalog",
            "92bcb692-56c4-4199-bf7e-e33cdd76a310",
            "Products",
            "Products",
        )
        .replace("version=\"2.20\"", "version=\"2.21\"");
        fs::write(&path, unsupported).expect("unsupported descriptor must be written");
        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::UnsupportedVersion { .. })
        ));

        let foreign = descriptor(
            "Catalog",
            "92bcb692-56c4-4199-bf7e-e33cdd76a310",
            "Products",
            "Products",
        )
        .replace("<Catalog uuid=", "<Catalog xmlns=\"urn:foreign\" uuid=");
        fs::write(&path, foreign).expect("foreign descriptor must be written");
        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::WrongNamespace { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_identifiers_in_both_scopes() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        for name in ["First", "Second"] {
            fs::write(
                root.path().join(format!("Catalogs/{name}.xml")),
                descriptor(
                    "Catalog",
                    "92bcb692-56c4-4199-bf7e-e33cdd76a310",
                    name,
                    name,
                ),
            )
            .expect("descriptor must be written");
        }

        for scope in [
            DesignerXmlBuildScope::Complete,
            DesignerXmlBuildScope::Partial,
        ] {
            let error = FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), scope)
                .expect_err("duplicate identity must fail");
            assert!(matches!(
                error,
                DesignerXmlMetadataObjectError::DuplicateIdentifier { .. }
            ));
        }
    }

    #[test]
    fn rejects_malformed_missing_and_repeated_fields() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        let path = root.path().join("Catalogs/Products.xml");

        fs::write(&path, "<MetaDataObject>").expect("malformed descriptor must be written");
        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::MissingAttribute { .. }
                | DesignerXmlMetadataObjectError::WrongNamespace { .. }
                | DesignerXmlMetadataObjectError::MalformedXml { .. })
        ));

        fs::write(
            &path,
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Catalog uuid="92bcb692-56c4-4199-bf7e-e33cdd76a310"><Properties><Name>Products</Name><Name>Products</Name></Properties></Catalog></MetaDataObject>"#,
        )
        .expect("repeated field descriptor must be written");
        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::NameCount { count: 2, .. }
                | DesignerXmlMetadataObjectError::DuplicateField { .. })
        ));

        fs::write(
            &path,
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Catalog uuid=""><Properties><Name>Products</Name></Properties></Catalog></MetaDataObject>"#,
        )
        .expect("missing identifier descriptor must be written");
        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::MissingField { field: "uuid", .. })
        ));
    }

    #[test]
    fn reordered_unrelated_elements_are_equivalent() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        let path = root.path().join("Catalogs/Products.xml");
        let first = r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Catalog uuid="92bcb692-56c4-4199-bf7e-e33cdd76a310"><InternalInfo/><Properties><Name>Products</Name><Synonym><v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core"><v8:content>Products</v8:content></v8:item></Synonym><Comment/></Properties></Catalog></MetaDataObject>"#;
        let second = r#"<MetaDataObject version="2.20" xmlns="http://v8.1c.ru/8.3/MDClasses"><Catalog uuid="92bcb692-56c4-4199-bf7e-e33cdd76a310"><Properties><Comment/><Name>Products</Name><Synonym><v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core"><v8:content>Products</v8:content></v8:item></Synonym></Properties><InternalInfo/></Catalog></MetaDataObject>"#;
        fs::write(&path, first).expect("first descriptor must be written");
        let first = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect("first descriptor must parse");
        fs::write(&path, second).expect("second descriptor must be written");
        let second = FileSystemDesignerXmlMetadataObjectReader
            .read_all(root.path(), DesignerXmlBuildScope::Partial)
            .expect("second descriptor must parse");

        assert_eq!(first[0].id(), second[0].id());
        assert_eq!(first[0].name(), second[0].name());
        assert_eq!(first[0].kind(), second[0].kind());
        assert_eq!(first[0].payload(), second[0].payload());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_accepted_artifact() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());
        fs::create_dir(root.path().join("Catalogs")).expect("family must be created");
        fs::write(root.path().join("outside.xml"), PRODUCTS)
            .expect("outside descriptor must be written");
        symlink(
            root.path().join("outside.xml"),
            root.path().join("Catalogs/Products.xml"),
        )
        .expect("symlink must be created");

        assert!(matches!(
            FileSystemDesignerXmlMetadataObjectReader
                .read_all(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlMetadataObjectError::SymlinkArtifact(_))
        ));
    }
}
