//! Typed parser for EDT XDTO Package descriptors and schema artifacts.

use oneagent_common::EntityName;
use oneagent_graph::XdtoTypeKind;
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesCData, BytesRef, BytesStart, BytesText, Event};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::EdtMetadataObjectDescriptor;

const XDTO_DESCRIPTOR_ROOT: &str = "mdclass:XDTOPackage";
const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";
const XDTO_SCHEMA_ROOT: &str = "package";
const XDTO_SCHEMA_NAMESPACE: &str = "http://v8.1c.ru/8.1/xdto";
const XDTO_ARTIFACT_NAME: &str = "Package.xdto";

/// Parsed EDT XDTO Package descriptor and its joined schema artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtXdtoPackageDescriptor {
    metadata: EdtMetadataObjectDescriptor,
    namespace: String,
    artifact_path: PathBuf,
    types: Vec<EdtXdtoTypeDeclaration>,
    deferred: Vec<EdtXdtoDeferredObservation>,
}

impl EdtXdtoPackageDescriptor {
    /// Returns the already discovered top-level metadata descriptor.
    #[must_use]
    pub const fn metadata(&self) -> &EdtMetadataObjectDescriptor {
        &self.metadata
    }

    /// Returns the exact descriptor and schema target namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the joined `Package.xdto` path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns direct Value/Object declarations ordered by exact local name.
    #[must_use]
    pub fn types(&self) -> &[EdtXdtoTypeDeclaration] {
        &self.types
    }

    /// Returns canonical deferred source observations.
    #[must_use]
    pub fn deferred(&self) -> &[EdtXdtoDeferredObservation] {
        &self.deferred
    }

    /// Returns the total number of deferred occurrences of `kind`.
    #[must_use]
    pub fn deferred_occurrence_count(&self, kind: EdtXdtoDeferredKind) -> usize {
        self.deferred
            .iter()
            .filter(|observation| observation.kind == kind)
            .map(|observation| observation.occurrence_count)
            .sum()
    }
}

/// One accepted direct XDTO type declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdtXdtoTypeDeclaration {
    name: EntityName,
    kind: XdtoTypeKind,
}

impl EdtXdtoTypeDeclaration {
    /// Returns the exact decoded local type name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the direct XDTO type family.
    #[must_use]
    pub const fn kind(&self) -> XdtoTypeKind {
        self.kind
    }
}

/// Deferred XDTO source family outside the accepted direct-type slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtXdtoDeferredKind {
    /// Direct package import declaration.
    Import,
    /// Nested enumeration occurrence.
    Enumeration,
    /// Nested property occurrence.
    Property,
    /// Nested pattern occurrence.
    Pattern,
    /// Nested inline type definition occurrence.
    TypeDef,
    /// Unknown direct schema child retained without speculative semantics.
    UnknownDirectElement,
}

/// Canonical aggregate for deferred XDTO source occurrences.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdtXdtoDeferredObservation {
    kind: EdtXdtoDeferredKind,
    source_value: Option<String>,
    occurrence_count: usize,
}

impl EdtXdtoDeferredObservation {
    /// Returns the deferred source family.
    #[must_use]
    pub const fn kind(&self) -> EdtXdtoDeferredKind {
        self.kind
    }

    /// Returns an exact import namespace or direct element name when applicable.
    #[must_use]
    pub fn source_value(&self) -> Option<&str> {
        self.source_value.as_deref()
    }

    /// Returns the number of equivalent source occurrences.
    #[must_use]
    pub const fn occurrence_count(&self) -> usize {
        self.occurrence_count
    }
}

/// Reads one already discovered EDT XDTO Package and its schema artifact.
pub trait EdtXdtoPackageReader {
    /// Reads the XDTO-specific descriptor content and the joined `Package.xdto`.
    ///
    /// # Errors
    ///
    /// Returns a typed error for metadata-kind, filesystem, XML, namespace,
    /// cardinality, required-field, or duplicate-type failures.
    fn read(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtXdtoPackageDescriptor, EdtXdtoPackageError>;
}

/// Filesystem implementation of [`EdtXdtoPackageReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtXdtoPackageReader;

impl EdtXdtoPackageReader for FileSystemEdtXdtoPackageReader {
    fn read(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtXdtoPackageDescriptor, EdtXdtoPackageError> {
        if metadata.kind() != MetadataKind::XdtoPackage {
            return Err(EdtXdtoPackageError::UnexpectedMetadataKind(metadata.kind()));
        }

        let object_directory = object_directory(metadata)?;
        let descriptor_path = metadata.descriptor_path();
        let descriptor_xml = fs::read_to_string(descriptor_path).map_err(|source| {
            EdtXdtoPackageError::ReadDescriptor {
                path: descriptor_path.to_path_buf(),
                source,
            }
        })?;
        let descriptor_namespace = parse_descriptor_namespace(&descriptor_xml, metadata)?;

        let artifact_path = find_artifact(&object_directory)?;
        let artifact_xml = fs::read_to_string(&artifact_path).map_err(|source| {
            EdtXdtoPackageError::ReadArtifact {
                path: artifact_path.clone(),
                source,
            }
        })?;
        let schema = parse_schema(&artifact_xml)?;

        if descriptor_namespace != schema.namespace {
            return Err(EdtXdtoPackageError::NamespaceMismatch {
                descriptor: descriptor_namespace,
                artifact: schema.namespace,
            });
        }

        Ok(EdtXdtoPackageDescriptor {
            metadata: metadata.clone(),
            namespace: descriptor_namespace,
            artifact_path,
            types: schema.types,
            deferred: schema.deferred,
        })
    }
}

fn object_directory(
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<PathBuf, EdtXdtoPackageError> {
    let directory = metadata
        .descriptor_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if !directory.is_dir() {
        return Err(EdtXdtoPackageError::ObjectDirectoryNotFound(directory));
    }
    Ok(directory)
}

fn find_artifact(object_directory: &Path) -> Result<PathBuf, EdtXdtoPackageError> {
    let mut candidates = Vec::new();
    for entry in
        fs::read_dir(object_directory).map_err(|source| EdtXdtoPackageError::ReadDirectory {
            path: object_directory.to_path_buf(),
            source,
        })?
    {
        let entry = entry.map_err(|source| EdtXdtoPackageError::ReadDirectoryEntry {
            path: object_directory.to_path_buf(),
            source,
        })?;
        let file_type =
            entry
                .file_type()
                .map_err(|source| EdtXdtoPackageError::ReadDirectoryEntry {
                    path: object_directory.to_path_buf(),
                    source,
                })?;
        let path = entry.path();
        if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xdto"))
        {
            candidates.push(path);
        }
    }
    candidates.sort();

    match candidates.len() {
        0 => Err(EdtXdtoPackageError::ArtifactNotFound(
            object_directory.to_path_buf(),
        )),
        1 => {
            let artifact = candidates.remove(0);
            if artifact.file_name().and_then(|name| name.to_str()) != Some(XDTO_ARTIFACT_NAME) {
                return Err(EdtXdtoPackageError::UnexpectedArtifactName(artifact));
            }
            Ok(artifact)
        }
        _ => Err(EdtXdtoPackageError::MultipleArtifacts {
            directory: object_directory.to_path_buf(),
            candidates,
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DescriptorField {
    Name,
    Namespace,
}

impl DescriptorField {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Namespace => "namespace",
        }
    }
}

#[allow(clippy::too_many_lines)]
fn parse_descriptor_namespace(
    xml: &str,
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<String, EdtXdtoPackageError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut depth = 0_usize;
    let mut root_seen = false;
    let mut root_closed = false;
    let mut uuid = None;
    let mut name = None;
    let mut namespace = None;
    let mut pending = None::<(DescriptorField, String)>;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if depth == 0 {
                    if root_seen {
                        return Err(EdtXdtoPackageError::MalformedDescriptorXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    uuid = validate_descriptor_root(&reader, &event)?;
                    root_seen = true;
                } else if depth == 1 {
                    let element = xml_name(event.name().as_ref())
                        .map_err(EdtXdtoPackageError::MalformedDescriptorXml)?;
                    pending = match element.as_str() {
                        "name" => Some((DescriptorField::Name, String::new())),
                        "namespace" => Some((DescriptorField::Namespace, String::new())),
                        _ => None,
                    };
                }
                depth += 1;
            }
            Ok(Event::Empty(event)) => {
                if depth == 0 {
                    if root_seen {
                        return Err(EdtXdtoPackageError::MalformedDescriptorXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    uuid = validate_descriptor_root(&reader, &event)?;
                    root_seen = true;
                    root_closed = true;
                } else if depth == 1 {
                    let element = xml_name(event.name().as_ref())
                        .map_err(EdtXdtoPackageError::MalformedDescriptorXml)?;
                    match element.as_str() {
                        "name" => set_descriptor_field(
                            DescriptorField::Name,
                            String::new(),
                            &mut name,
                            &mut namespace,
                        )?,
                        "namespace" => set_descriptor_field(
                            DescriptorField::Namespace,
                            String::new(),
                            &mut name,
                            &mut namespace,
                        )?,
                        _ => {}
                    }
                }
            }
            Ok(Event::Text(event)) => append_descriptor_text(&event, &mut pending)?,
            Ok(Event::CData(event)) => append_descriptor_cdata(&event, &mut pending)?,
            Ok(Event::GeneralRef(event)) => append_descriptor_reference(&event, &mut pending)?,
            Ok(Event::End(_)) => {
                if depth == 2
                    && let Some((field, value)) = pending.take()
                {
                    set_descriptor_field(field, value, &mut name, &mut namespace)?;
                }
                if depth == 1 {
                    root_closed = true;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Err(source) => {
                return Err(EdtXdtoPackageError::MalformedDescriptorXml(
                    source.to_string(),
                ));
            }
        }
    }

    if !root_seen {
        return Err(EdtXdtoPackageError::MissingDescriptorRoot);
    }
    if !root_closed {
        return Err(EdtXdtoPackageError::MalformedDescriptorXml(
            "unexpected end of file before the XDTO Package root was closed".to_owned(),
        ));
    }

    let uuid = uuid.ok_or(EdtXdtoPackageError::MissingDescriptorUuid)?;
    if uuid.is_empty() {
        return Err(EdtXdtoPackageError::EmptyDescriptorUuid);
    }
    let name = name.ok_or(EdtXdtoPackageError::MissingDescriptorName)?;
    if name.is_empty() {
        return Err(EdtXdtoPackageError::EmptyDescriptorName);
    }
    let namespace = namespace.ok_or(EdtXdtoPackageError::MissingDescriptorNamespace)?;
    if namespace.is_empty() {
        return Err(EdtXdtoPackageError::EmptyDescriptorNamespace);
    }

    if uuid != metadata.id().as_str() || name != metadata.name().as_str() {
        return Err(EdtXdtoPackageError::DescriptorIdentityMismatch {
            expected_uuid: metadata.id().as_str().to_owned(),
            actual_uuid: uuid,
            expected_name: metadata.name().as_str().to_owned(),
            actual_name: name,
        });
    }

    Ok(namespace)
}

fn validate_descriptor_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>, EdtXdtoPackageError> {
    let root =
        xml_name(event.name().as_ref()).map_err(EdtXdtoPackageError::MalformedDescriptorXml)?;
    if root != XDTO_DESCRIPTOR_ROOT {
        return Err(EdtXdtoPackageError::UnexpectedDescriptorRoot(root));
    }
    let namespace = attribute_value(reader, event, b"xmlns:mdclass")
        .map_err(EdtXdtoPackageError::MalformedDescriptorXml)?;
    if namespace.as_deref() != Some(METADATA_NAMESPACE) {
        return Err(EdtXdtoPackageError::UnsupportedDescriptorNamespace(
            namespace,
        ));
    }
    attribute_value(reader, event, b"uuid").map_err(EdtXdtoPackageError::MalformedDescriptorXml)
}

fn set_descriptor_field(
    field: DescriptorField,
    value: String,
    name: &mut Option<String>,
    namespace: &mut Option<String>,
) -> Result<(), EdtXdtoPackageError> {
    let slot = match field {
        DescriptorField::Name => name,
        DescriptorField::Namespace => namespace,
    };
    if slot.replace(value).is_some() {
        return Err(EdtXdtoPackageError::DuplicateDescriptorField(
            field.as_str(),
        ));
    }
    Ok(())
}

fn append_descriptor_text(
    event: &BytesText<'_>,
    pending: &mut Option<(DescriptorField, String)>,
) -> Result<(), EdtXdtoPackageError> {
    if let Some((_, value)) = pending {
        let decoded = event
            .decode()
            .map_err(|source| EdtXdtoPackageError::MalformedDescriptorXml(source.to_string()))?;
        let decoded = unescape(&decoded)
            .map_err(|source| EdtXdtoPackageError::MalformedDescriptorXml(source.to_string()))?;
        value.push_str(&decoded);
    }
    Ok(())
}

fn append_descriptor_cdata(
    event: &BytesCData<'_>,
    pending: &mut Option<(DescriptorField, String)>,
) -> Result<(), EdtXdtoPackageError> {
    if let Some((_, value)) = pending {
        let decoded = event
            .decode()
            .map_err(|source| EdtXdtoPackageError::MalformedDescriptorXml(source.to_string()))?;
        value.push_str(&decoded);
    }
    Ok(())
}

fn append_descriptor_reference(
    event: &BytesRef<'_>,
    pending: &mut Option<(DescriptorField, String)>,
) -> Result<(), EdtXdtoPackageError> {
    if let Some((_, value)) = pending {
        let reference = event
            .decode()
            .map_err(|source| EdtXdtoPackageError::MalformedDescriptorXml(source.to_string()))?;
        let encoded = format!("&{reference};");
        let decoded = unescape(&encoded)
            .map_err(|source| EdtXdtoPackageError::MalformedDescriptorXml(source.to_string()))?;
        value.push_str(&decoded);
    }
    Ok(())
}

struct ParsedSchema {
    namespace: String,
    types: Vec<EdtXdtoTypeDeclaration>,
    deferred: Vec<EdtXdtoDeferredObservation>,
}

#[allow(clippy::too_many_lines)]
fn parse_schema(xml: &str) -> Result<ParsedSchema, EdtXdtoPackageError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut depth = 0_usize;
    let mut root_seen = false;
    let mut root_closed = false;
    let mut namespace = None;
    let mut raw_types = Vec::<(EntityName, XdtoTypeKind)>::new();
    let mut deferred = BTreeMap::<(EdtXdtoDeferredKind, Option<String>), usize>::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if depth == 0 {
                    if root_seen {
                        return Err(EdtXdtoPackageError::MalformedArtifactXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    namespace = validate_schema_root(&reader, &event)?;
                    root_seen = true;
                } else {
                    observe_schema_element(&reader, &event, depth, &mut raw_types, &mut deferred)?;
                }
                depth += 1;
            }
            Ok(Event::Empty(event)) => {
                if depth == 0 {
                    if root_seen {
                        return Err(EdtXdtoPackageError::MalformedArtifactXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    namespace = validate_schema_root(&reader, &event)?;
                    root_seen = true;
                    root_closed = true;
                } else {
                    observe_schema_element(&reader, &event, depth, &mut raw_types, &mut deferred)?;
                }
            }
            Ok(Event::End(_)) => {
                if depth == 1 {
                    root_closed = true;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Ok(
                Event::Decl(_)
                | Event::PI(_)
                | Event::Comment(_)
                | Event::DocType(_)
                | Event::Text(_)
                | Event::CData(_)
                | Event::GeneralRef(_),
            ) => {}
            Err(source) => {
                return Err(EdtXdtoPackageError::MalformedArtifactXml(
                    source.to_string(),
                ));
            }
        }
    }

    if !root_seen {
        return Err(EdtXdtoPackageError::MissingArtifactRoot);
    }
    if !root_closed {
        return Err(EdtXdtoPackageError::MalformedArtifactXml(
            "unexpected end of file before the XDTO schema root was closed".to_owned(),
        ));
    }
    let namespace = namespace.ok_or(EdtXdtoPackageError::MissingTargetNamespace)?;
    if namespace.is_empty() {
        return Err(EdtXdtoPackageError::EmptyTargetNamespace);
    }

    let mut declarations = BTreeMap::<EntityName, Vec<XdtoTypeKind>>::new();
    for (name, kind) in raw_types {
        declarations.entry(name).or_default().push(kind);
    }
    if let Some((name, families)) = declarations.iter_mut().find(|(_, kinds)| kinds.len() > 1) {
        families.sort();
        return Err(EdtXdtoPackageError::DuplicateTypeName {
            name: name.clone(),
            families: families.clone(),
        });
    }
    let types = declarations
        .into_iter()
        .map(|(name, mut families)| EdtXdtoTypeDeclaration {
            name,
            kind: families.remove(0),
        })
        .collect();
    let deferred = deferred
        .into_iter()
        .map(
            |((kind, source_value), occurrence_count)| EdtXdtoDeferredObservation {
                kind,
                source_value,
                occurrence_count,
            },
        )
        .collect();

    Ok(ParsedSchema {
        namespace,
        types,
        deferred,
    })
}

fn validate_schema_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>, EdtXdtoPackageError> {
    let root =
        xml_name(event.name().as_ref()).map_err(EdtXdtoPackageError::MalformedArtifactXml)?;
    if root != XDTO_SCHEMA_ROOT {
        return Err(EdtXdtoPackageError::UnexpectedArtifactRoot(root));
    }
    let namespace = attribute_value(reader, event, b"xmlns")
        .map_err(EdtXdtoPackageError::MalformedArtifactXml)?;
    if namespace.as_deref() != Some(XDTO_SCHEMA_NAMESPACE) {
        return Err(EdtXdtoPackageError::UnsupportedArtifactNamespace(namespace));
    }
    attribute_value(reader, event, b"targetNamespace")
        .map_err(EdtXdtoPackageError::MalformedArtifactXml)
}

fn observe_schema_element(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    depth: usize,
    types: &mut Vec<(EntityName, XdtoTypeKind)>,
    deferred: &mut BTreeMap<(EdtXdtoDeferredKind, Option<String>), usize>,
) -> Result<(), EdtXdtoPackageError> {
    let qualified =
        xml_name(event.name().as_ref()).map_err(EdtXdtoPackageError::MalformedArtifactXml)?;
    let local = qualified.rsplit(':').next().unwrap_or(qualified.as_str());

    if depth == 1 {
        match local {
            "valueType" => read_type(reader, event, XdtoTypeKind::Value, types)?,
            "objectType" => read_type(reader, event, XdtoTypeKind::Object, types)?,
            "import" => {
                let namespace = attribute_value(reader, event, b"namespace")
                    .map_err(EdtXdtoPackageError::MalformedArtifactXml)?;
                increment_deferred(deferred, EdtXdtoDeferredKind::Import, namespace);
            }
            _ => increment_deferred(
                deferred,
                EdtXdtoDeferredKind::UnknownDirectElement,
                Some(qualified),
            ),
        }
    } else if depth >= 2 {
        let kind = match local {
            "enumeration" => Some(EdtXdtoDeferredKind::Enumeration),
            "property" => Some(EdtXdtoDeferredKind::Property),
            "pattern" => Some(EdtXdtoDeferredKind::Pattern),
            "typeDef" => Some(EdtXdtoDeferredKind::TypeDef),
            _ => None,
        };
        if let Some(kind) = kind {
            increment_deferred(deferred, kind, None);
        }
    }

    Ok(())
}

fn read_type(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    kind: XdtoTypeKind,
    types: &mut Vec<(EntityName, XdtoTypeKind)>,
) -> Result<(), EdtXdtoPackageError> {
    let name = attribute_value(reader, event, b"name")
        .map_err(EdtXdtoPackageError::MalformedArtifactXml)?
        .ok_or(EdtXdtoPackageError::MissingTypeName(kind))?;
    if name.is_empty() {
        return Err(EdtXdtoPackageError::EmptyTypeName(kind));
    }
    let name =
        EntityName::new(name.clone()).map_err(|_| EdtXdtoPackageError::InvalidTypeName(name))?;
    types.push((name, kind));
    Ok(())
}

fn increment_deferred(
    deferred: &mut BTreeMap<(EdtXdtoDeferredKind, Option<String>), usize>,
    kind: EdtXdtoDeferredKind,
    source_value: Option<String>,
) {
    *deferred.entry((kind, source_value)).or_default() += 1;
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    key: &[u8],
) -> Result<Option<String>, String> {
    let mut value = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute.map_err(|source| source.to_string())?;
        if attribute.key.as_ref() == key {
            if value.is_some() {
                return Err(format!(
                    "duplicate XML attribute `{}`",
                    String::from_utf8_lossy(key)
                ));
            }
            value = Some(
                attribute
                    .decode_and_unescape_value(reader.decoder())
                    .map_err(|source| source.to_string())?
                    .into_owned(),
            );
        }
    }
    Ok(value)
}

fn xml_name(name: &[u8]) -> Result<String, String> {
    std::str::from_utf8(name)
        .map(str::to_owned)
        .map_err(|source| source.to_string())
}

/// Errors produced while joining and parsing an EDT XDTO Package.
#[derive(Debug)]
pub enum EdtXdtoPackageError {
    /// The supplied metadata descriptor is not an XDTO Package.
    UnexpectedMetadataKind(MetadataKind),
    /// The object directory derived from the descriptor path does not exist.
    ObjectDirectoryNotFound(PathBuf),
    /// The object directory cannot be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An entry in the object directory cannot be inspected.
    ReadDirectoryEntry {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// No `.xdto` artifact exists in the object directory.
    ArtifactNotFound(PathBuf),
    /// The only `.xdto` artifact does not have the exact required name.
    UnexpectedArtifactName(PathBuf),
    /// More than one `.xdto` artifact exists in the object directory.
    MultipleArtifacts {
        /// Object directory.
        directory: PathBuf,
        /// Deterministically ordered candidate paths.
        candidates: Vec<PathBuf>,
    },
    /// The metadata descriptor cannot be read.
    ReadDescriptor {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The schema artifact cannot be read.
    ReadArtifact {
        /// Artifact path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The metadata descriptor XML is malformed.
    MalformedDescriptorXml(String),
    /// The metadata descriptor has no root element.
    MissingDescriptorRoot,
    /// The metadata descriptor root is not `mdclass:XDTOPackage`.
    UnexpectedDescriptorRoot(String),
    /// The metadata descriptor namespace is missing or unsupported.
    UnsupportedDescriptorNamespace(Option<String>),
    /// The metadata descriptor UUID is missing.
    MissingDescriptorUuid,
    /// The metadata descriptor UUID is empty.
    EmptyDescriptorUuid,
    /// The direct metadata descriptor name is missing.
    MissingDescriptorName,
    /// The direct metadata descriptor name is empty.
    EmptyDescriptorName,
    /// The direct metadata descriptor namespace is missing.
    MissingDescriptorNamespace,
    /// The direct metadata descriptor namespace is empty.
    EmptyDescriptorNamespace,
    /// A required direct metadata descriptor field occurs more than once.
    DuplicateDescriptorField(&'static str),
    /// Re-read UUID/name content differs from the supplied generic descriptor.
    DescriptorIdentityMismatch {
        /// UUID from the supplied descriptor.
        expected_uuid: String,
        /// UUID from the re-read source.
        actual_uuid: String,
        /// Name from the supplied descriptor.
        expected_name: String,
        /// Name from the re-read source.
        actual_name: String,
    },
    /// The schema artifact XML is malformed.
    MalformedArtifactXml(String),
    /// The schema artifact has no root element.
    MissingArtifactRoot,
    /// The schema artifact root is not `package`.
    UnexpectedArtifactRoot(String),
    /// The schema artifact namespace is missing or unsupported.
    UnsupportedArtifactNamespace(Option<String>),
    /// The schema target namespace is missing.
    MissingTargetNamespace,
    /// The schema target namespace is empty.
    EmptyTargetNamespace,
    /// Descriptor and schema target namespaces differ.
    NamespaceMismatch {
        /// Exact descriptor namespace.
        descriptor: String,
        /// Exact schema target namespace.
        artifact: String,
    },
    /// A direct type has no `name` attribute.
    MissingTypeName(XdtoTypeKind),
    /// A direct type has an empty `name` attribute.
    EmptyTypeName(XdtoTypeKind),
    /// A direct type name cannot be represented by the common name primitive.
    InvalidTypeName(String),
    /// One exact direct name occurs more than once across either type family.
    DuplicateTypeName {
        /// Duplicated exact local name.
        name: EntityName,
        /// Canonically ordered family occurrences.
        families: Vec<XdtoTypeKind>,
    },
}

impl Display for EdtXdtoPackageError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedMetadataKind(kind) => {
                write!(
                    formatter,
                    "expected EDT XDTO Package metadata, found `{kind}`"
                )
            }
            Self::ObjectDirectoryNotFound(path) => write!(
                formatter,
                "EDT XDTO Package object directory does not exist: {}",
                path.display()
            ),
            Self::ReadDirectory { path, source } | Self::ReadDirectoryEntry { path, source } => {
                write!(
                    formatter,
                    "failed to inspect EDT XDTO Package directory {}: {source}",
                    path.display()
                )
            }
            Self::ArtifactNotFound(path) => write!(
                formatter,
                "EDT XDTO Package artifact is missing in {}",
                path.display()
            ),
            Self::UnexpectedArtifactName(path) => write!(
                formatter,
                "unexpected EDT XDTO Package artifact name: {}",
                path.display()
            ),
            Self::MultipleArtifacts {
                directory,
                candidates,
            } => write!(
                formatter,
                "multiple EDT XDTO Package artifacts found in {}: {candidates:?}",
                directory.display()
            ),
            Self::ReadDescriptor { path, source } => write!(
                formatter,
                "failed to read EDT XDTO Package descriptor {}: {source}",
                path.display()
            ),
            Self::ReadArtifact { path, source } => write!(
                formatter,
                "failed to read EDT XDTO Package artifact {}: {source}",
                path.display()
            ),
            Self::MalformedDescriptorXml(message) => {
                write!(
                    formatter,
                    "malformed EDT XDTO Package descriptor XML: {message}"
                )
            }
            Self::MissingDescriptorRoot => {
                formatter.write_str("EDT XDTO Package descriptor root is missing")
            }
            Self::UnexpectedDescriptorRoot(root) => write!(
                formatter,
                "unexpected EDT XDTO Package descriptor root `{root}`"
            ),
            Self::UnsupportedDescriptorNamespace(Some(namespace)) => write!(
                formatter,
                "unsupported EDT XDTO Package descriptor namespace `{namespace}`"
            ),
            Self::UnsupportedDescriptorNamespace(None) | Self::MissingDescriptorNamespace => {
                formatter.write_str("EDT XDTO Package descriptor namespace is missing")
            }
            Self::MissingDescriptorUuid => {
                formatter.write_str("EDT XDTO Package descriptor UUID is missing")
            }
            Self::EmptyDescriptorUuid => {
                formatter.write_str("EDT XDTO Package descriptor UUID is empty")
            }
            Self::MissingDescriptorName => {
                formatter.write_str("EDT XDTO Package descriptor name is missing")
            }
            Self::EmptyDescriptorName => {
                formatter.write_str("EDT XDTO Package descriptor name is empty")
            }
            Self::EmptyDescriptorNamespace => {
                formatter.write_str("EDT XDTO Package descriptor namespace is empty")
            }
            Self::DuplicateDescriptorField(field) => write!(
                formatter,
                "EDT XDTO Package descriptor field `{field}` occurs more than once"
            ),
            Self::DescriptorIdentityMismatch {
                expected_uuid,
                actual_uuid,
                expected_name,
                actual_name,
            } => write!(
                formatter,
                "EDT XDTO Package descriptor identity mismatch: expected {expected_uuid}/{expected_name}, found {actual_uuid}/{actual_name}"
            ),
            Self::MalformedArtifactXml(message) => {
                write!(
                    formatter,
                    "malformed EDT XDTO Package artifact XML: {message}"
                )
            }
            Self::MissingArtifactRoot => {
                formatter.write_str("EDT XDTO Package artifact root is missing")
            }
            Self::UnexpectedArtifactRoot(root) => write!(
                formatter,
                "unexpected EDT XDTO Package artifact root `{root}`"
            ),
            Self::UnsupportedArtifactNamespace(Some(namespace)) => write!(
                formatter,
                "unsupported EDT XDTO Package artifact namespace `{namespace}`"
            ),
            Self::UnsupportedArtifactNamespace(None) => {
                formatter.write_str("EDT XDTO Package artifact namespace is missing")
            }
            Self::MissingTargetNamespace => {
                formatter.write_str("EDT XDTO Package target namespace is missing")
            }
            Self::EmptyTargetNamespace => {
                formatter.write_str("EDT XDTO Package target namespace is empty")
            }
            Self::NamespaceMismatch {
                descriptor,
                artifact,
            } => write!(
                formatter,
                "EDT XDTO Package namespace mismatch: descriptor `{descriptor}`, artifact `{artifact}`"
            ),
            Self::MissingTypeName(kind) => {
                write!(
                    formatter,
                    "direct XDTO {} type name is missing",
                    kind.as_str()
                )
            }
            Self::EmptyTypeName(kind) => {
                write!(
                    formatter,
                    "direct XDTO {} type name is empty",
                    kind.as_str()
                )
            }
            Self::InvalidTypeName(name) => {
                write!(formatter, "invalid direct XDTO type name `{name}`")
            }
            Self::DuplicateTypeName { name, families } => write!(
                formatter,
                "duplicate direct XDTO type name `{}` in families {families:?}",
                name.as_str()
            ),
        }
    }
}

impl std::error::Error for EdtXdtoPackageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadDescriptor { source, .. }
            | Self::ReadArtifact { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::XdtoTypeKind;
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{TempDir, tempdir};

    use super::{
        EdtXdtoDeferredKind, EdtXdtoPackageError, EdtXdtoPackageReader,
        FileSystemEdtXdtoPackageReader,
    };
    use crate::{
        EdtMetadataObjectDescriptor, EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader,
    };

    const UUID: &str = "2bd80198-4739-4e8f-9af1-40addc91a05b";
    const NAME: &str = "TestPackage";
    const NAMESPACE: &str = "urn:test:package";

    fn descriptor_xml(uuid: &str, name: &str, namespace: Option<&str>) -> String {
        let namespace = namespace
            .map(|value| format!("<namespace>{value}</namespace>"))
            .unwrap_or_default();
        format!(
            r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}"><name>{name}</name>{namespace}</mdclass:XDTOPackage>"#
        )
    }

    fn schema_xml(namespace: Option<&str>, content: &str) -> String {
        let target = namespace
            .map(|value| format!(r#" targetNamespace="{value}""#))
            .unwrap_or_default();
        format!(r#"<package xmlns="http://v8.1c.ru/8.1/xdto"{target}>{content}</package>"#)
    }

    fn package_files(descriptor: &str, artifact: Option<&str>) -> TempDir {
        let directory = tempdir().expect("temporary XDTO Package directory must be created");
        fs::write(directory.path().join(format!("{NAME}.mdo")), descriptor)
            .expect("descriptor fixture must be written");
        if let Some(artifact) = artifact {
            fs::write(directory.path().join("Package.xdto"), artifact)
                .expect("artifact fixture must be written");
        }
        directory
    }

    fn valid_package(content: &str) -> TempDir {
        package_files(
            &descriptor_xml(UUID, NAME, Some(NAMESPACE)),
            Some(&schema_xml(Some(NAMESPACE), content)),
        )
    }

    fn metadata(directory: &Path) -> EdtMetadataObjectDescriptor {
        FileSystemEdtMetadataObjectReader
            .read(directory, MetadataKind::XdtoPackage)
            .expect("generated metadata descriptor must parse")
    }

    fn read(directory: &Path) -> Result<super::EdtXdtoPackageDescriptor, EdtXdtoPackageError> {
        FileSystemEdtXdtoPackageReader.read(&metadata(directory))
    }

    fn overwrite_descriptor(directory: &Path, xml: &str) {
        fs::write(directory.join(format!("{NAME}.mdo")), xml)
            .expect("descriptor fixture must be replaced");
    }

    fn overwrite_artifact(directory: &Path, xml: &str) {
        fs::write(directory.join("Package.xdto"), xml).expect("artifact fixture must be replaced");
    }

    fn manual_metadata(
        kind: MetadataKind,
        descriptor_path: PathBuf,
    ) -> EdtMetadataObjectDescriptor {
        EdtMetadataObjectDescriptor::new(
            EntityId::new(UUID).expect("fixture UUID must be valid"),
            EntityName::new(NAME).expect("fixture name must be valid"),
            None,
            kind,
            None,
            descriptor_path,
        )
    }

    #[test]
    fn parses_canonical_direct_types_and_typed_deferred_observations() {
        let directory = valid_package(
            r#"
                <objectType name="Zulu">
                    <property name="First"/>
                    <typeDef><property name="Nested"/></typeDef>
                    <valueType name="NestedMustNotBecomeDirect"/>
                </objectType>
                <futureElement/>
                <valueType name="Alpha">
                    <enumeration>One</enumeration>
                    <pattern>.+</pattern>
                    <enumeration>Two</enumeration>
                </valueType>
                <import namespace="urn:external"/>
                <import namespace="urn:external"/>
            "#,
        );

        let package = read(directory.path()).expect("generated XDTO Package must parse");
        assert_eq!(package.metadata().id().as_str(), UUID);
        assert_eq!(package.metadata().name().as_str(), NAME);
        assert_eq!(package.namespace(), NAMESPACE);
        assert_eq!(
            package
                .artifact_path()
                .file_name()
                .and_then(|name| name.to_str()),
            Some("Package.xdto")
        );
        assert_eq!(package.types().len(), 2);
        assert_eq!(package.types()[0].name().as_str(), "Alpha");
        assert_eq!(package.types()[0].kind(), XdtoTypeKind::Value);
        assert_eq!(package.types()[1].name().as_str(), "Zulu");
        assert_eq!(package.types()[1].kind(), XdtoTypeKind::Object);
        assert_eq!(
            package.deferred_occurrence_count(EdtXdtoDeferredKind::Import),
            2
        );
        assert_eq!(
            package.deferred_occurrence_count(EdtXdtoDeferredKind::Enumeration),
            2
        );
        assert_eq!(
            package.deferred_occurrence_count(EdtXdtoDeferredKind::Property),
            2
        );
        assert_eq!(
            package.deferred_occurrence_count(EdtXdtoDeferredKind::Pattern),
            1
        );
        assert_eq!(
            package.deferred_occurrence_count(EdtXdtoDeferredKind::TypeDef),
            1
        );
        let import = package
            .deferred()
            .iter()
            .find(|observation| observation.kind() == EdtXdtoDeferredKind::Import)
            .expect("import observation must exist");
        assert_eq!(import.source_value(), Some("urn:external"));
        assert_eq!(import.occurrence_count(), 2);
        let unknown = package
            .deferred()
            .iter()
            .find(|observation| observation.kind() == EdtXdtoDeferredKind::UnknownDirectElement)
            .expect("unknown direct observation must exist");
        assert_eq!(unknown.source_value(), Some("futureElement"));
        assert_eq!(unknown.occurrence_count(), 1);
    }

    #[test]
    fn source_reordering_and_repeated_reads_are_equal() {
        let first_xml = schema_xml(
            Some(NAMESPACE),
            r#"<objectType name="Zulu"><property name="P"/></objectType><import namespace="urn:b"/><valueType name="Alpha"><enumeration>A</enumeration></valueType><import namespace="urn:a"/>"#,
        );
        let reordered_xml = schema_xml(
            Some(NAMESPACE),
            r#"<import namespace="urn:a"/><valueType name="Alpha"><enumeration>A</enumeration></valueType><import namespace="urn:b"/><objectType name="Zulu"><property name="P"/></objectType>"#,
        );
        let directory = package_files(
            &descriptor_xml(UUID, NAME, Some(NAMESPACE)),
            Some(&first_xml),
        );
        let descriptor = metadata(directory.path());
        let first = FileSystemEdtXdtoPackageReader
            .read(&descriptor)
            .expect("first ordering must parse");
        let repeated = FileSystemEdtXdtoPackageReader
            .read(&descriptor)
            .expect("repeated read must parse");
        overwrite_artifact(directory.path(), &reordered_xml);
        let reordered = FileSystemEdtXdtoPackageReader
            .read(&descriptor)
            .expect("reordered source must parse");

        assert_eq!(first, repeated);
        assert_eq!(first, reordered);
    }

    #[test]
    fn filesystem_and_artifact_cardinality_failures_are_typed_and_ordered() {
        let missing_directory = tempdir().expect("temporary parent must be created");
        let descriptor = manual_metadata(
            MetadataKind::XdtoPackage,
            missing_directory.path().join("absent/TestPackage.mdo"),
        );
        assert!(matches!(
            FileSystemEdtXdtoPackageReader.read(&descriptor),
            Err(EdtXdtoPackageError::ObjectDirectoryNotFound(_))
        ));

        let missing_artifact = package_files(&descriptor_xml(UUID, NAME, Some(NAMESPACE)), None);
        assert!(matches!(
            read(missing_artifact.path()),
            Err(EdtXdtoPackageError::ArtifactNotFound(_))
        ));

        let unexpected = package_files(&descriptor_xml(UUID, NAME, Some(NAMESPACE)), None);
        fs::write(
            unexpected.path().join("Other.XDTO"),
            schema_xml(Some(NAMESPACE), ""),
        )
        .expect("unexpected artifact must be written");
        assert!(matches!(
            read(unexpected.path()),
            Err(EdtXdtoPackageError::UnexpectedArtifactName(path))
                if path.file_name().and_then(|name| name.to_str()) == Some("Other.XDTO")
        ));

        let ambiguous = valid_package("");
        fs::write(
            ambiguous.path().join("Additional.xdto"),
            schema_xml(Some(NAMESPACE), ""),
        )
        .expect("additional artifact must be written");
        let error = read(ambiguous.path()).expect_err("multiple artifacts must fail");
        assert!(matches!(
            error,
            EdtXdtoPackageError::MultipleArtifacts { candidates, .. }
                if candidates.len() == 2 && candidates[0] < candidates[1]
        ));
    }

    #[test]
    fn unreadable_descriptor_and_artifact_inputs_are_typed() {
        let descriptor_input = valid_package("");
        let descriptor = metadata(descriptor_input.path());
        fs::write(descriptor.descriptor_path(), [0xff_u8])
            .expect("invalid UTF-8 descriptor must be written");
        assert!(matches!(
            FileSystemEdtXdtoPackageReader.read(&descriptor),
            Err(EdtXdtoPackageError::ReadDescriptor { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidData
        ));

        let artifact_input = valid_package("");
        fs::write(artifact_input.path().join("Package.xdto"), [0xff_u8])
            .expect("invalid UTF-8 artifact must be written");
        assert!(matches!(
            read(artifact_input.path()),
            Err(EdtXdtoPackageError::ReadArtifact { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidData
        ));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn descriptor_root_namespace_and_required_fields_are_typed() {
        let wrong_kind = valid_package("");
        let descriptor = manual_metadata(
            MetadataKind::Catalog,
            wrong_kind.path().join(format!("{NAME}.mdo")),
        );
        assert!(matches!(
            FileSystemEdtXdtoPackageReader.read(&descriptor),
            Err(EdtXdtoPackageError::UnexpectedMetadataKind(
                MetadataKind::Catalog
            ))
        ));

        let cases = [
            ("malformed", "<mdclass:XDTOPackage", 0_u8),
            ("missing root", "<?xml version=\"1.0\"?>", 1),
            (
                "wrong root",
                r#"<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"/>"#,
                2,
            ),
            (
                "wrong namespace",
                r#"<mdclass:XDTOPackage xmlns:mdclass="urn:wrong"/>"#,
                3,
            ),
            (
                "missing UUID",
                r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"><name>TestPackage</name><namespace>urn:test:package</namespace></mdclass:XDTOPackage>"#,
                4,
            ),
            ("empty UUID", &descriptor_xml("", NAME, Some(NAMESPACE)), 5),
            (
                "missing name",
                r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="2bd80198-4739-4e8f-9af1-40addc91a05b"><namespace>urn:test:package</namespace></mdclass:XDTOPackage>"#,
                6,
            ),
            ("empty name", &descriptor_xml(UUID, "", Some(NAMESPACE)), 7),
            ("missing namespace", &descriptor_xml(UUID, NAME, None), 8),
            ("empty namespace", &descriptor_xml(UUID, NAME, Some("")), 9),
        ];

        for (label, xml, expected) in cases {
            let directory = valid_package("");
            let descriptor = metadata(directory.path());
            overwrite_descriptor(directory.path(), xml);
            let error = FileSystemEdtXdtoPackageReader
                .read(&descriptor)
                .unwrap_err();
            let actual = match error {
                EdtXdtoPackageError::MalformedDescriptorXml(_) => 0,
                EdtXdtoPackageError::MissingDescriptorRoot => 1,
                EdtXdtoPackageError::UnexpectedDescriptorRoot(_) => 2,
                EdtXdtoPackageError::UnsupportedDescriptorNamespace(_) => 3,
                EdtXdtoPackageError::MissingDescriptorUuid => 4,
                EdtXdtoPackageError::EmptyDescriptorUuid => 5,
                EdtXdtoPackageError::MissingDescriptorName => 6,
                EdtXdtoPackageError::EmptyDescriptorName => 7,
                EdtXdtoPackageError::MissingDescriptorNamespace => 8,
                EdtXdtoPackageError::EmptyDescriptorNamespace => 9,
                other => panic!("unexpected descriptor error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }

        let duplicate = valid_package("");
        let descriptor = metadata(duplicate.path());
        overwrite_descriptor(
            duplicate.path(),
            &format!(
                r#"<mdclass:XDTOPackage xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{UUID}"><name>{NAME}</name><name>{NAME}</name><namespace>{NAMESPACE}</namespace></mdclass:XDTOPackage>"#
            ),
        );
        assert!(matches!(
            FileSystemEdtXdtoPackageReader.read(&descriptor),
            Err(EdtXdtoPackageError::DuplicateDescriptorField("name"))
        ));

        let mismatch = valid_package("");
        let descriptor = metadata(mismatch.path());
        overwrite_descriptor(
            mismatch.path(),
            &descriptor_xml("different-uuid", "DifferentName", Some(NAMESPACE)),
        );
        assert!(matches!(
            FileSystemEdtXdtoPackageReader.read(&descriptor),
            Err(EdtXdtoPackageError::DescriptorIdentityMismatch { .. })
        ));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn artifact_root_namespace_target_and_type_name_failures_are_typed() {
        let cases = [
            ("malformed", "<package", 0_u8),
            ("missing root", "<?xml version=\"1.0\"?>", 1),
            (
                "wrong root",
                r#"<schema xmlns="http://v8.1c.ru/8.1/xdto"/>"#,
                2,
            ),
            (
                "missing namespace",
                r#"<package targetNamespace="urn:test:package"/>"#,
                3,
            ),
            (
                "wrong namespace",
                r#"<package xmlns="urn:wrong" targetNamespace="urn:test:package"/>"#,
                3,
            ),
            (
                "missing target namespace",
                r#"<package xmlns="http://v8.1c.ru/8.1/xdto"/>"#,
                4,
            ),
            (
                "empty target namespace",
                r#"<package xmlns="http://v8.1c.ru/8.1/xdto" targetNamespace=""/>"#,
                5,
            ),
            (
                "namespace mismatch",
                r#"<package xmlns="http://v8.1c.ru/8.1/xdto" targetNamespace="urn:different"/>"#,
                6,
            ),
            (
                "missing value name",
                &schema_xml(Some(NAMESPACE), "<valueType/>"),
                7,
            ),
            (
                "empty object name",
                &schema_xml(Some(NAMESPACE), r#"<objectType name=""/>"#),
                8,
            ),
        ];

        for (label, xml, expected) in cases {
            let directory = valid_package("");
            overwrite_artifact(directory.path(), xml);
            let error = read(directory.path()).unwrap_err();
            let actual = match error {
                EdtXdtoPackageError::MalformedArtifactXml(_) => 0,
                EdtXdtoPackageError::MissingArtifactRoot => 1,
                EdtXdtoPackageError::UnexpectedArtifactRoot(_) => 2,
                EdtXdtoPackageError::UnsupportedArtifactNamespace(_) => 3,
                EdtXdtoPackageError::MissingTargetNamespace => 4,
                EdtXdtoPackageError::EmptyTargetNamespace => 5,
                EdtXdtoPackageError::NamespaceMismatch { .. } => 6,
                EdtXdtoPackageError::MissingTypeName(XdtoTypeKind::Value) => 7,
                EdtXdtoPackageError::EmptyTypeName(XdtoTypeKind::Object) => 8,
                other => panic!("unexpected artifact error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }
    }

    #[test]
    fn duplicate_type_names_are_family_complete_and_source_order_independent() {
        let same_family =
            valid_package(r#"<valueType name="Duplicate"/><valueType name="Duplicate"/>"#);
        assert!(matches!(
            read(same_family.path()),
            Err(EdtXdtoPackageError::DuplicateTypeName { name, families })
                if name.as_str() == "Duplicate"
                    && families == vec![XdtoTypeKind::Value, XdtoTypeKind::Value]
        ));

        let different_families =
            valid_package(r#"<objectType name="Duplicate"/><valueType name="Duplicate"/>"#);
        let descriptor = metadata(different_families.path());
        let first = FileSystemEdtXdtoPackageReader
            .read(&descriptor)
            .unwrap_err();
        overwrite_artifact(
            different_families.path(),
            &schema_xml(
                Some(NAMESPACE),
                r#"<valueType name="Duplicate"/><objectType name="Duplicate"/>"#,
            ),
        );
        let reordered = FileSystemEdtXdtoPackageReader
            .read(&descriptor)
            .unwrap_err();

        let expected = vec![XdtoTypeKind::Value, XdtoTypeKind::Object];
        assert!(matches!(
            &first,
            EdtXdtoPackageError::DuplicateTypeName { name, families }
                if name.as_str() == "Duplicate" && families == &expected
        ));
        assert_eq!(format!("{first:?}"), format!("{reordered:?}"));
    }

    #[test]
    #[cfg(feature = "external-edt-corpus-tests")]
    #[allow(clippy::too_many_lines)]
    fn all_live_xdto_packages_match_the_accepted_direct_slice() {
        let packages_directory = crate::live_test_support::project_root().join("src/XDTOPackages");
        let mut package_directories = fs::read_dir(&packages_directory)
            .expect("live XDTOPackages directory must be readable")
            .map(|entry| {
                entry
                    .expect("live XDTO Package entry must be readable")
                    .path()
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        package_directories.sort();
        assert_eq!(package_directories.len(), 20);

        let mut value_types = 0_usize;
        let mut object_types = 0_usize;
        let mut imports = 0_usize;
        let mut enumerations = 0_usize;
        let mut properties = 0_usize;
        let mut patterns = 0_usize;
        let mut type_defs = 0_usize;

        for directory in &package_directories {
            let metadata = FileSystemEdtMetadataObjectReader
                .read(directory, MetadataKind::XdtoPackage)
                .expect("live XDTO metadata descriptor must parse");
            let first = FileSystemEdtXdtoPackageReader
                .read(&metadata)
                .expect("live XDTO Package must parse");
            let repeated = FileSystemEdtXdtoPackageReader
                .read(&metadata)
                .expect("repeated live XDTO Package read must parse");
            assert_eq!(first, repeated);
            assert!(
                first
                    .types()
                    .windows(2)
                    .all(|pair| pair[0].name() < pair[1].name())
            );
            assert_eq!(
                first.deferred_occurrence_count(EdtXdtoDeferredKind::UnknownDirectElement),
                0
            );

            let package_value_types = first
                .types()
                .iter()
                .filter(|declaration| declaration.kind() == XdtoTypeKind::Value)
                .count();
            let package_object_types = first.types().len() - package_value_types;
            value_types += package_value_types;
            object_types += package_object_types;
            imports += first.deferred_occurrence_count(EdtXdtoDeferredKind::Import);
            enumerations += first.deferred_occurrence_count(EdtXdtoDeferredKind::Enumeration);
            properties += first.deferred_occurrence_count(EdtXdtoDeferredKind::Property);
            patterns += first.deferred_occurrence_count(EdtXdtoDeferredKind::Pattern);
            type_defs += first.deferred_occurrence_count(EdtXdtoDeferredKind::TypeDef);

            match directory.file_name().and_then(|name| name.to_str()) {
                Some("CurrencyRates") => {
                    assert_eq!(package_value_types, 0);
                    assert_eq!(package_object_types, 1);
                    assert_eq!(first.types()[0].name().as_str(), "Rate");
                }
                Some("ApplicationExtensionsManifest_1_0_0_1") => {
                    assert!(package_value_types > 0);
                    assert!(package_object_types > 0);
                }
                Some("EnterpriseData_1_17_3") => {
                    assert_eq!(package_value_types, 328);
                    assert_eq!(package_object_types, 867);
                }
                _ => {}
            }
        }

        assert_eq!(value_types, 3_421);
        assert_eq!(object_types, 9_245);
        assert_eq!(value_types + object_types, 12_666);
        assert_eq!(imports, 17);
        assert_eq!(enumerations, 5_493);
        assert_eq!(properties, 61_435);
        assert_eq!(patterns, 16);
        assert_eq!(type_defs, 1_667);
    }
}
