//! Reader for top-level EDT metadata object descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// Parsed descriptor of a top-level EDT metadata object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtMetadataObjectDescriptor {
    id: EntityId,
    name: EntityName,
    synonym: Option<String>,
    kind: MetadataKind,
    extension: Option<EdtMetadataObjectExtensionDescriptor>,
    descriptor_path: PathBuf,
    document_register_declarations: Vec<EdtDocumentRegisterDeclarationOutcome>,
}

impl EdtMetadataObjectDescriptor {
    /// Creates a parsed EDT metadata object descriptor.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        synonym: Option<String>,
        kind: MetadataKind,
        extension: Option<EdtMetadataObjectExtensionDescriptor>,
        descriptor_path: PathBuf,
    ) -> Self {
        Self {
            id,
            name,
            synonym,
            kind,
            extension,
            descriptor_path,
            document_register_declarations: Vec::new(),
        }
    }

    /// Returns the stable EDT identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the metadata object name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the localized synonym when present.
    #[must_use]
    pub fn synonym(&self) -> Option<&str> {
        self.synonym.as_deref()
    }

    /// Returns the metadata kind.
    #[must_use]
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }

    /// Returns the explicit metadata extension fact, when the descriptor is adopted.
    #[must_use]
    pub const fn extension(&self) -> Option<&EdtMetadataObjectExtensionDescriptor> {
        self.extension.as_ref()
    }

    /// Returns the source descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    #[allow(dead_code)]
    pub(crate) fn document_register_declarations(
        &self,
    ) -> &[EdtDocumentRegisterDeclarationOutcome] {
        &self.document_register_declarations
    }

    fn with_document_register_declarations(
        mut self,
        declarations: Vec<EdtDocumentRegisterDeclarationOutcome>,
    ) -> Self {
        self.document_register_declarations = declarations;
        self
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtDocumentRegisterDeclarationOutcome {
    Supported(EdtDocumentRegisterDeclaration),
    UnsupportedKind(EdtDocumentRegisterDeclaration),
    UnsupportedNamespace(EdtDocumentRegisterDeclaration),
    Malformed(EdtMalformedDocumentRegisterDeclaration),
    Ambiguous(EdtAmbiguousDocumentRegisterDeclaration),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtDocumentRegisterDeclaration {
    pub(crate) owner_id: EntityId,
    pub(crate) owner_name: EntityName,
    pub(crate) descriptor_path: PathBuf,
    pub(crate) raw_value: String,
    pub(crate) namespace: String,
    pub(crate) local_name: String,
    pub(crate) lookup_key: String,
    pub(crate) kind: Option<MetadataKind>,
    pub(crate) provenance: EdtDocumentRegisterDeclarationProvenance,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtMalformedDocumentRegisterDeclaration {
    pub(crate) owner_id: EntityId,
    pub(crate) owner_name: EntityName,
    pub(crate) descriptor_path: PathBuf,
    pub(crate) raw_value: String,
    pub(crate) reason: EdtMalformedDocumentRegisterDeclarationReason,
    pub(crate) provenance: EdtDocumentRegisterDeclarationProvenance,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdtMalformedDocumentRegisterDeclarationReason {
    EmptyValue,
    MissingComponent,
    EmptyComponent,
    AdditionalComponents,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtAmbiguousDocumentRegisterDeclaration {
    pub(crate) kind: MetadataKind,
    pub(crate) lookup_key: String,
    pub(crate) declarations: Vec<EdtDocumentRegisterDeclaration>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtDocumentRegisterDeclarationProvenance {
    Single(EdtDocumentRegisterDeclarationContext),
    Duplicate(Vec<EdtDocumentRegisterDeclarationContext>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EdtDocumentRegisterDeclarationContext {
    pub(crate) ordinal: usize,
}

#[derive(Debug)]
struct RawDocumentRegisterDeclaration {
    raw_value: String,
    contexts: Vec<EdtDocumentRegisterDeclarationContext>,
}

/// Explicit extension fact declared by an adopted EDT metadata object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtMetadataObjectExtensionDescriptor {
    extended_configuration_object_id: EntityId,
}

impl EdtMetadataObjectExtensionDescriptor {
    /// Creates an extension descriptor for a resolved-later base metadata object id.
    #[must_use]
    pub const fn new(extended_configuration_object_id: EntityId) -> Self {
        Self {
            extended_configuration_object_id,
        }
    }

    /// Returns the base/original metadata object identifier declared by EDT.
    #[must_use]
    pub const fn extended_configuration_object_id(&self) -> &EntityId {
        &self.extended_configuration_object_id
    }
}

/// Reads top-level EDT metadata object descriptors.
pub trait EdtMetadataObjectReader {
    /// Reads a metadata object descriptor from its object directory.
    ///
    /// # Errors
    ///
    /// Returns an error when an `.mdo` file cannot be found, read or parsed.
    fn read(
        &self,
        object_directory: &Path,
        kind: MetadataKind,
    ) -> Result<EdtMetadataObjectDescriptor, EdtMetadataObjectError>;
}

/// Filesystem implementation of [`EdtMetadataObjectReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtMetadataObjectReader;

impl EdtMetadataObjectReader for FileSystemEdtMetadataObjectReader {
    fn read(
        &self,
        object_directory: &Path,
        kind: MetadataKind,
    ) -> Result<EdtMetadataObjectDescriptor, EdtMetadataObjectError> {
        let descriptor_path = find_descriptor_file(object_directory)?;
        let xml = fs::read_to_string(&descriptor_path).map_err(|source| {
            EdtMetadataObjectError::ReadFile {
                path: descriptor_path.clone(),
                source,
            }
        })?;

        parse_descriptor(&xml, kind, descriptor_path)
    }
}

fn find_descriptor_file(object_directory: &Path) -> Result<PathBuf, EdtMetadataObjectError> {
    if !object_directory.is_dir() {
        return Err(EdtMetadataObjectError::ObjectDirectoryNotFound(
            object_directory.to_path_buf(),
        ));
    }

    let mut candidates = fs::read_dir(object_directory)
        .map_err(|source| EdtMetadataObjectError::ReadDirectory {
            path: object_directory.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("mdo"))
        })
        .collect::<Vec<_>>();

    candidates.sort();

    match candidates.len() {
        0 => Err(EdtMetadataObjectError::DescriptorNotFound(
            object_directory.to_path_buf(),
        )),
        1 => Ok(candidates.remove(0)),
        _ => Err(EdtMetadataObjectError::MultipleDescriptors {
            directory: object_directory.to_path_buf(),
            candidates,
        }),
    }
}

fn parse_descriptor(
    xml: &str,
    kind: MetadataKind,
    descriptor_path: PathBuf,
) -> Result<EdtMetadataObjectDescriptor, EdtMetadataObjectError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut uuid = None;
    let mut name = None;
    let mut synonym = None;
    let mut object_belonging = None;
    let mut extended_configuration_object = None;
    let mut path = Vec::<String>::new();
    let mut raw_document_register_declarations = Vec::new();
    let mut register_declaration_ordinal = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let element = local_name(event.name().as_ref());

                if is_direct_document_register_declaration(kind, &path, &element) {
                    let raw_value = read_document_register_value(&mut reader, &event)?;
                    let context =
                        next_register_declaration_context(&mut register_declaration_ordinal);
                    raw_document_register_declarations.push((raw_value, context));
                    continue;
                }

                path.push(element);

                if uuid.is_none() {
                    uuid = read_uuid(&reader, &event)?;
                }
            }
            Ok(Event::Empty(event)) => {
                let element = local_name(event.name().as_ref());

                if is_direct_document_register_declaration(kind, &path, &element) {
                    let context =
                        next_register_declaration_context(&mut register_declaration_ordinal);
                    raw_document_register_declarations.push((String::new(), context));
                    continue;
                }

                if uuid.is_none() {
                    uuid = read_uuid(&reader, &event)?;
                }
            }
            Ok(Event::Text(event)) => {
                let text = event
                    .decode()
                    .map_err(|source| EdtMetadataObjectError::MalformedXml(source.to_string()))?
                    .into_owned();

                match path.last().map(String::as_str) {
                    Some(element) if element.eq_ignore_ascii_case("name") && name.is_none() => {
                        name = Some(text);
                    }
                    Some(element) if is_synonym_content_path(&path, element) => {
                        synonym.get_or_insert(text);
                    }
                    Some(element) if element.eq_ignore_ascii_case("objectBelonging") => {
                        object_belonging.get_or_insert(text);
                    }
                    Some(element)
                        if element.eq_ignore_ascii_case("extendedConfigurationObject") =>
                    {
                        extended_configuration_object.get_or_insert(text);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(
                Event::Decl(_)
                | Event::PI(_)
                | Event::Comment(_)
                | Event::CData(_)
                | Event::DocType(_)
                | Event::GeneralRef(_),
            ) => {}
            Err(source) => {
                return Err(EdtMetadataObjectError::MalformedXml(source.to_string()));
            }
        }
    }

    let uuid = uuid.ok_or(EdtMetadataObjectError::MissingUuid)?;
    let name = name.ok_or(EdtMetadataObjectError::MissingName)?;

    let id = EntityId::new(uuid).map_err(|_| EdtMetadataObjectError::InvalidIdentifier)?;
    let name = EntityName::new(name).map_err(|_| EdtMetadataObjectError::InvalidName)?;
    let extension =
        extension_descriptor(object_belonging.as_deref(), extended_configuration_object)?;
    let document_register_declarations = if kind == MetadataKind::Document {
        parse_document_register_declarations(
            raw_document_register_declarations,
            &id,
            &name,
            &descriptor_path,
        )
    } else {
        Vec::new()
    };

    Ok(
        EdtMetadataObjectDescriptor::new(id, name, synonym, kind, extension, descriptor_path)
            .with_document_register_declarations(document_register_declarations),
    )
}

fn is_direct_document_register_declaration(
    kind: MetadataKind,
    path: &[String],
    element: &str,
) -> bool {
    kind == MetadataKind::Document && path.len() == 1 && element == "registerRecords"
}

fn read_document_register_value(
    reader: &mut Reader<&[u8]>,
    event: &quick_xml::events::BytesStart<'_>,
) -> Result<String, EdtMetadataObjectError> {
    let raw_value = reader
        .read_text(event.to_end().name())
        .map_err(|source| EdtMetadataObjectError::MalformedXml(source.to_string()))?;

    unescape(&raw_value)
        .map_err(|source| EdtMetadataObjectError::MalformedXml(source.to_string()))
        .map(std::borrow::Cow::into_owned)
}

fn next_register_declaration_context(ordinal: &mut usize) -> EdtDocumentRegisterDeclarationContext {
    *ordinal += 1;
    EdtDocumentRegisterDeclarationContext { ordinal: *ordinal }
}

fn parse_document_register_declarations(
    occurrences: Vec<(String, EdtDocumentRegisterDeclarationContext)>,
    owner_id: &EntityId,
    owner_name: &EntityName,
    descriptor_path: &Path,
) -> Vec<EdtDocumentRegisterDeclarationOutcome> {
    let mut unique = Vec::<RawDocumentRegisterDeclaration>::new();

    for (raw_value, context) in occurrences {
        if let Some(existing) = unique
            .iter_mut()
            .find(|declaration| declaration.raw_value == raw_value)
        {
            existing.contexts.push(context);
        } else {
            unique.push(RawDocumentRegisterDeclaration {
                raw_value,
                contexts: vec![context],
            });
        }
    }

    let parsed = unique
        .into_iter()
        .map(|declaration| {
            parse_document_register_declaration(declaration, owner_id, owner_name, descriptor_path)
        })
        .collect::<Vec<_>>();
    let mut collision_members = BTreeMap::<(MetadataKind, String), Vec<usize>>::new();

    for (index, outcome) in parsed.iter().enumerate() {
        let declaration = match outcome {
            EdtDocumentRegisterDeclarationOutcome::Supported(declaration)
            | EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(declaration) => declaration,
            EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(_)
            | EdtDocumentRegisterDeclarationOutcome::Malformed(_)
            | EdtDocumentRegisterDeclarationOutcome::Ambiguous(_) => continue,
        };
        let kind = declaration
            .kind
            .expect("known register declarations must preserve their metadata kind");
        collision_members
            .entry((kind, declaration.lookup_key.clone()))
            .or_default()
            .push(index);
    }

    let ambiguous_keys = collision_members
        .iter()
        .filter(|(_, members)| members.len() > 1)
        .map(|(key, _)| key.clone())
        .collect::<BTreeSet<_>>();
    let mut emitted_ambiguous_keys = BTreeSet::new();
    let mut outcomes = Vec::new();

    for outcome in &parsed {
        let declaration = match outcome {
            EdtDocumentRegisterDeclarationOutcome::Supported(declaration)
            | EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(declaration) => declaration,
            EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(_)
            | EdtDocumentRegisterDeclarationOutcome::Malformed(_)
            | EdtDocumentRegisterDeclarationOutcome::Ambiguous(_) => {
                outcomes.push(outcome.clone());
                continue;
            }
        };
        let kind = declaration
            .kind
            .expect("known register declarations must preserve their metadata kind");
        let key = (kind, declaration.lookup_key.clone());

        if !ambiguous_keys.contains(&key) {
            outcomes.push(outcome.clone());
            continue;
        }

        if emitted_ambiguous_keys.insert(key.clone()) {
            let declarations = collision_members[&key]
                .iter()
                .map(|index| match &parsed[*index] {
                    EdtDocumentRegisterDeclarationOutcome::Supported(declaration)
                    | EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(declaration) => {
                        declaration.clone()
                    }
                    EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(_)
                    | EdtDocumentRegisterDeclarationOutcome::Malformed(_)
                    | EdtDocumentRegisterDeclarationOutcome::Ambiguous(_) => {
                        unreachable!("only known register declarations are collision candidates")
                    }
                })
                .collect::<Vec<_>>();
            outcomes.push(EdtDocumentRegisterDeclarationOutcome::Ambiguous(
                EdtAmbiguousDocumentRegisterDeclaration {
                    kind,
                    lookup_key: key.1,
                    declarations,
                },
            ));
        }
    }

    outcomes
}

fn parse_document_register_declaration(
    declaration: RawDocumentRegisterDeclaration,
    owner_id: &EntityId,
    owner_name: &EntityName,
    descriptor_path: &Path,
) -> EdtDocumentRegisterDeclarationOutcome {
    let RawDocumentRegisterDeclaration {
        raw_value,
        contexts,
    } = declaration;
    let provenance = declaration_provenance(contexts);
    let components = raw_value.split('.').collect::<Vec<_>>();
    let malformed_reason = if raw_value.is_empty() {
        Some(EdtMalformedDocumentRegisterDeclarationReason::EmptyValue)
    } else if components.len() == 1 {
        Some(EdtMalformedDocumentRegisterDeclarationReason::MissingComponent)
    } else if components.len() > 2 {
        Some(EdtMalformedDocumentRegisterDeclarationReason::AdditionalComponents)
    } else if components.iter().any(|component| component.is_empty()) {
        Some(EdtMalformedDocumentRegisterDeclarationReason::EmptyComponent)
    } else {
        None
    };

    if let Some(reason) = malformed_reason {
        return EdtDocumentRegisterDeclarationOutcome::Malformed(
            EdtMalformedDocumentRegisterDeclaration {
                owner_id: owner_id.clone(),
                owner_name: owner_name.clone(),
                descriptor_path: descriptor_path.to_path_buf(),
                raw_value,
                reason,
                provenance,
            },
        );
    }

    let namespace = components[0].to_owned();
    let local_name = components[1].to_owned();
    let kind = register_namespace_kind(&namespace);
    let parsed = EdtDocumentRegisterDeclaration {
        owner_id: owner_id.clone(),
        owner_name: owner_name.clone(),
        descriptor_path: descriptor_path.to_path_buf(),
        raw_value,
        namespace,
        lookup_key: local_name.to_lowercase(),
        local_name,
        kind,
        provenance,
    };

    match kind {
        Some(MetadataKind::AccumulationRegister) => {
            EdtDocumentRegisterDeclarationOutcome::Supported(parsed)
        }
        Some(_) => EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(parsed),
        None => EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(parsed),
    }
}

fn declaration_provenance(
    contexts: Vec<EdtDocumentRegisterDeclarationContext>,
) -> EdtDocumentRegisterDeclarationProvenance {
    match contexts.as_slice() {
        [context] => EdtDocumentRegisterDeclarationProvenance::Single(*context),
        _ => EdtDocumentRegisterDeclarationProvenance::Duplicate(contexts),
    }
}

const fn register_namespace_kind(namespace: &str) -> Option<MetadataKind> {
    match namespace.as_bytes() {
        b"InformationRegister" => Some(MetadataKind::InformationRegister),
        b"AccumulationRegister" => Some(MetadataKind::AccumulationRegister),
        b"AccountingRegister" => Some(MetadataKind::AccountingRegister),
        b"CalculationRegister" => Some(MetadataKind::CalculationRegister),
        _ => None,
    }
}

fn read_uuid(
    reader: &Reader<&[u8]>,
    event: &quick_xml::events::BytesStart<'_>,
) -> Result<Option<String>, EdtMetadataObjectError> {
    for attribute in event.attributes().with_checks(false) {
        let attribute =
            attribute.map_err(|source| EdtMetadataObjectError::MalformedXml(source.to_string()))?;

        if local_name(attribute.key.as_ref()) == "uuid" {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|source| EdtMetadataObjectError::MalformedXml(source.to_string()))?;

            return Ok(Some(value.into_owned()));
        }
    }

    Ok(None)
}

fn extension_descriptor(
    object_belonging: Option<&str>,
    extended_configuration_object: Option<String>,
) -> Result<Option<EdtMetadataObjectExtensionDescriptor>, EdtMetadataObjectError> {
    if !object_belonging.is_some_and(|value| value.eq_ignore_ascii_case("Adopted")) {
        return Ok(None);
    }

    let Some(extended_configuration_object) = extended_configuration_object else {
        return Ok(None);
    };

    let extended_configuration_object_id = EntityId::new(extended_configuration_object.trim())
        .map_err(|_| EdtMetadataObjectError::InvalidIdentifier)?;

    Ok(Some(EdtMetadataObjectExtensionDescriptor::new(
        extended_configuration_object_id,
    )))
}

fn is_synonym_content_path(path: &[String], element: &str) -> bool {
    path.len() >= 2
        && path[path.len() - 2].eq_ignore_ascii_case("synonym")
        && element.eq_ignore_ascii_case("content")
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

/// Error produced while reading an EDT metadata object.
#[derive(Debug)]
pub enum EdtMetadataObjectError {
    /// The supplied object directory does not exist.
    ObjectDirectoryNotFound(PathBuf),
    /// The object directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// No `.mdo` descriptor was found.
    DescriptorNotFound(PathBuf),
    /// More than one `.mdo` descriptor was found.
    MultipleDescriptors {
        /// Object directory.
        directory: PathBuf,
        /// Candidate files.
        candidates: Vec<PathBuf>,
    },
    /// The descriptor could not be read.
    ReadFile {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The XML is malformed.
    MalformedXml(String),
    /// The descriptor does not contain a UUID.
    MissingUuid,
    /// The descriptor does not contain a metadata name.
    MissingName,
    /// The UUID cannot be represented as an entity identifier.
    InvalidIdentifier,
    /// The metadata name cannot be represented as an entity name.
    InvalidName,
}

impl Display for EdtMetadataObjectError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectDirectoryNotFound(path) => write!(
                formatter,
                "EDT metadata object directory was not found: {}",
                path.display()
            ),
            Self::ReadDirectory { path, source } => write!(
                formatter,
                "failed to read EDT metadata directory {}: {source}",
                path.display()
            ),
            Self::DescriptorNotFound(path) => write!(
                formatter,
                "EDT metadata descriptor was not found in {}",
                path.display()
            ),
            Self::MultipleDescriptors {
                directory,
                candidates,
            } => write!(
                formatter,
                "multiple EDT metadata descriptors found in {}: {}",
                directory.display(),
                candidates.len()
            ),
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT metadata descriptor {}: {source}",
                path.display()
            ),
            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT metadata XML: {message}")
            }
            Self::MissingUuid => formatter.write_str("EDT metadata object UUID is missing"),
            Self::MissingName => formatter.write_str("EDT metadata object name is missing"),
            Self::InvalidIdentifier => {
                formatter.write_str("EDT metadata object identifier is invalid")
            }
            Self::InvalidName => formatter.write_str("EDT metadata object name is invalid"),
        }
    }
}

impl std::error::Error for EdtMetadataObjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. } | Self::ReadFile { source, .. } => Some(source),
            Self::ObjectDirectoryNotFound(_)
            | Self::DescriptorNotFound(_)
            | Self::MultipleDescriptors { .. }
            | Self::MalformedXml(_)
            | Self::MissingUuid
            | Self::MissingName
            | Self::InvalidIdentifier
            | Self::InvalidName => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::{
        EdtDocumentRegisterDeclaration, EdtDocumentRegisterDeclarationOutcome,
        EdtDocumentRegisterDeclarationProvenance, EdtMalformedDocumentRegisterDeclarationReason,
        EdtMetadataObjectError, EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader,
        parse_descriptor,
    };

    const DOCUMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="ef7f0101-8d38-4db9-bdb7-c12f72bcf39a">
    <name>SalesDocument</name>
    <synonym>
        <key>ru</key>
        <content>Документ продажи</content>
    </synonym>
</mdclass:Document>
"#;

    const ADOPTED_DOCUMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="11111111-2222-3333-4444-555555555555">
    <Properties>
        <Name>SalesExtension</Name>
        <ObjectBelonging>Adopted</ObjectBelonging>
        <ExtendedConfigurationObject>aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</ExtendedConfigurationObject>
    </Properties>
</mdclass:Document>
"#;

    fn writes_document_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/writes_project/src/Documents/RefundOfPaymentByOrder")
    }

    fn generated_descriptor(xml: &str, kind: MetadataKind) -> super::EdtMetadataObjectDescriptor {
        parse_descriptor(xml, kind, PathBuf::from("GeneratedObject.mdo"))
            .expect("generated descriptor must parse")
    }

    fn generated_document(register_records: &str) -> super::EdtMetadataObjectDescriptor {
        let xml = format!(
            r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="document-generated">
    <name>GeneratedDocument</name>
{register_records}
</mdclass:Document>"#
        );
        generated_descriptor(&xml, MetadataKind::Document)
    }

    fn supported_declaration(
        outcome: &EdtDocumentRegisterDeclarationOutcome,
    ) -> &EdtDocumentRegisterDeclaration {
        match outcome {
            EdtDocumentRegisterDeclarationOutcome::Supported(declaration) => declaration,
            other => panic!("expected supported declaration, got {other:?}"),
        }
    }

    fn provenance_ordinals(provenance: &EdtDocumentRegisterDeclarationProvenance) -> Vec<usize> {
        match provenance {
            EdtDocumentRegisterDeclarationProvenance::Single(context) => vec![context.ordinal],
            EdtDocumentRegisterDeclarationProvenance::Duplicate(contexts) => {
                contexts.iter().map(|context| context.ordinal).collect()
            }
        }
    }

    #[test]
    fn reads_top_level_metadata_descriptor() {
        let root = tempdir().expect("temporary directory must be created");
        let object_directory = root.path().join("SalesDocument");

        fs::create_dir_all(&object_directory).expect("object directory must be created");
        fs::write(object_directory.join("SalesDocument.mdo"), DOCUMENT_XML)
            .expect("descriptor must be created");

        let descriptor = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Document)
            .expect("descriptor must load");

        assert_eq!(
            descriptor.id().as_str(),
            "ef7f0101-8d38-4db9-bdb7-c12f72bcf39a"
        );
        assert_eq!(descriptor.name().as_str(), "SalesDocument");
        assert_eq!(descriptor.synonym(), Some("Документ продажи"));
        assert_eq!(descriptor.kind(), MetadataKind::Document);
        assert!(descriptor.extension().is_none());
        assert!(descriptor.document_register_declarations().is_empty());
    }

    #[test]
    fn repository_fixture_preserves_document_register_declarations() {
        let object_directory = writes_document_directory();
        let descriptor = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Document)
            .expect("repository-backed Document descriptor must load");
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 2);
        assert_eq!(
            descriptor.id().as_str(),
            "ed647f67-f8fe-476b-8823-8d52b365ab20"
        );
        assert_eq!(descriptor.name().as_str(), "RefundOfPaymentByOrder");

        let first = supported_declaration(&declarations[0]);
        let second = supported_declaration(&declarations[1]);
        assert_eq!(
            [first.raw_value.as_str(), second.raw_value.as_str()],
            [
                "AccumulationRegister.CashAccountBalance",
                "AccumulationRegister.RefundBankPayment",
            ]
        );
        assert_eq!(
            [first.namespace.as_str(), second.namespace.as_str()],
            ["AccumulationRegister", "AccumulationRegister"]
        );
        assert_eq!(
            [first.local_name.as_str(), second.local_name.as_str()],
            ["CashAccountBalance", "RefundBankPayment"]
        );
        assert_eq!(
            [first.lookup_key.as_str(), second.lookup_key.as_str()],
            ["cashaccountbalance", "refundbankpayment"]
        );
        assert_eq!(
            [first.kind, second.kind],
            [
                Some(MetadataKind::AccumulationRegister),
                Some(MetadataKind::AccumulationRegister),
            ]
        );
        assert_eq!(
            [first.owner_id.as_str(), second.owner_id.as_str()],
            [
                "ed647f67-f8fe-476b-8823-8d52b365ab20",
                "ed647f67-f8fe-476b-8823-8d52b365ab20",
            ]
        );
        assert_eq!(
            [first.owner_name.as_str(), second.owner_name.as_str()],
            ["RefundOfPaymentByOrder", "RefundOfPaymentByOrder"]
        );
        assert_eq!(first.descriptor_path, descriptor.descriptor_path());
        assert_eq!(second.descriptor_path, descriptor.descriptor_path());
        assert_eq!(provenance_ordinals(&first.provenance), vec![1]);
        assert_eq!(provenance_ordinals(&second.provenance), vec![2]);
    }

    #[test]
    fn generated_document_without_register_declarations_is_valid() {
        let descriptor = generated_document("");

        assert!(descriptor.document_register_declarations().is_empty());
        assert_eq!(descriptor.id().as_str(), "document-generated");
        assert_eq!(descriptor.name().as_str(), "GeneratedDocument");
    }

    #[test]
    fn generated_non_document_does_not_acquire_register_declarations() {
        let descriptor = generated_descriptor(
            r#"<mdclass:Catalog xmlns:mdclass="urn:test" uuid="catalog-generated">
    <name>GeneratedCatalog</name>
    <registerRecords>AccumulationRegister.ShouldBeIgnored</registerRecords>
</mdclass:Catalog>"#,
            MetadataKind::Catalog,
        );

        assert!(descriptor.document_register_declarations().is_empty());
    }

    #[test]
    fn generated_document_ignores_nested_register_declarations() {
        let descriptor = generated_document(
            r"    <attributes>
        <registerRecords>AccumulationRegister.Nested</registerRecords>
    </attributes>
    <registerRecords>AccumulationRegister.Direct</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 1);
        assert_eq!(supported_declaration(&declarations[0]).local_name, "Direct");
    }

    #[test]
    fn generated_known_non_allowlisted_register_kinds_are_typed_as_unsupported() {
        let descriptor = generated_document(
            r"    <registerRecords>InformationRegister.Info</registerRecords>
    <registerRecords>AccountingRegister.Accounting</registerRecords>
    <registerRecords>CalculationRegister.Calculation</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 3);
        assert_eq!(
            declarations
                .iter()
                .map(|outcome| match outcome {
                    EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(declaration) => {
                        declaration.kind
                    }
                    other => panic!("expected unsupported kind, got {other:?}"),
                })
                .collect::<Vec<_>>(),
            vec![
                Some(MetadataKind::InformationRegister),
                Some(MetadataKind::AccountingRegister),
                Some(MetadataKind::CalculationRegister),
            ]
        );
    }

    #[test]
    fn generated_unknown_namespace_is_typed_without_unknown_metadata_kind() {
        let descriptor = generated_document(
            "    <registerRecords>LocalizedRegister.Unsupported</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        let [EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(declaration)] =
            declarations
        else {
            panic!("unknown namespace must remain a typed unsupported declaration");
        };
        assert_eq!(declaration.raw_value, "LocalizedRegister.Unsupported");
        assert_eq!(declaration.namespace, "LocalizedRegister");
        assert_eq!(declaration.local_name, "Unsupported");
        assert_eq!(declaration.lookup_key, "unsupported");
        assert_eq!(declaration.kind, None);
    }

    #[test]
    fn generated_malformed_values_preserve_typed_outcomes_and_order() {
        let descriptor = generated_document(
            r"    <registerRecords/>
    <registerRecords>NameOnly</registerRecords>
    <registerRecords>.MissingNamespace</registerRecords>
    <registerRecords>AccumulationRegister.</registerRecords>
    <registerRecords>AccumulationRegister.Name.Additional</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 5);
        assert_eq!(
            declarations
                .iter()
                .map(|outcome| match outcome {
                    EdtDocumentRegisterDeclarationOutcome::Malformed(declaration) => {
                        (declaration.raw_value.as_str(), declaration.reason)
                    }
                    other => panic!("expected malformed declaration, got {other:?}"),
                })
                .collect::<Vec<_>>(),
            vec![
                (
                    "",
                    EdtMalformedDocumentRegisterDeclarationReason::EmptyValue,
                ),
                (
                    "NameOnly",
                    EdtMalformedDocumentRegisterDeclarationReason::MissingComponent,
                ),
                (
                    ".MissingNamespace",
                    EdtMalformedDocumentRegisterDeclarationReason::EmptyComponent,
                ),
                (
                    "AccumulationRegister.",
                    EdtMalformedDocumentRegisterDeclarationReason::EmptyComponent,
                ),
                (
                    "AccumulationRegister.Name.Additional",
                    EdtMalformedDocumentRegisterDeclarationReason::AdditionalComponents,
                ),
            ]
        );
        assert_eq!(descriptor.id().as_str(), "document-generated");
        assert_eq!(descriptor.name().as_str(), "GeneratedDocument");
    }

    #[test]
    fn generated_exact_duplicates_are_deduplicated_with_ordered_provenance() {
        let descriptor = generated_document(
            r"    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>AccumulationRegister.Stock</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 1);
        let declaration = supported_declaration(&declarations[0]);
        assert_eq!(declaration.raw_value, "AccumulationRegister.Stock");
        assert_eq!(provenance_ordinals(&declaration.provenance), vec![1, 2, 3]);
        assert!(matches!(
            declaration.provenance,
            EdtDocumentRegisterDeclarationProvenance::Duplicate(_)
        ));
    }

    #[test]
    fn generated_normalized_collisions_are_typed_as_ambiguous() {
        let descriptor = generated_document(
            r"    <registerRecords>AccumulationRegister.StockBalance</registerRecords>
    <registerRecords>AccumulationRegister.STOCKBALANCE</registerRecords>
    <registerRecords>AccumulationRegister.StockBalance</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        let [EdtDocumentRegisterDeclarationOutcome::Ambiguous(ambiguous)] = declarations else {
            panic!("normalized collision must produce one typed ambiguous outcome");
        };
        assert_eq!(ambiguous.kind, MetadataKind::AccumulationRegister);
        assert_eq!(ambiguous.lookup_key, "stockbalance");
        assert_eq!(ambiguous.declarations.len(), 2);
        assert_eq!(
            ambiguous
                .declarations
                .iter()
                .map(|declaration| declaration.raw_value.as_str())
                .collect::<Vec<_>>(),
            vec![
                "AccumulationRegister.StockBalance",
                "AccumulationRegister.STOCKBALANCE",
            ]
        );
        assert_eq!(
            provenance_ordinals(&ambiguous.declarations[0].provenance),
            vec![1, 3]
        );
        assert_eq!(
            provenance_ordinals(&ambiguous.declarations[1].provenance),
            vec![2]
        );
    }

    #[test]
    fn generated_lookup_keys_use_unicode_lowercase_without_normalization() {
        let descriptor = generated_document(
            "    <registerRecords>AccumulationRegister.É</registerRecords>\n    <registerRecords>AccumulationRegister.E\u{301}</registerRecords>\n    <registerRecords>AccumulationRegister.Cash&amp;Account</registerRecords>",
        );
        let declarations = descriptor.document_register_declarations();

        assert_eq!(declarations.len(), 3);
        let parsed = declarations
            .iter()
            .map(supported_declaration)
            .collect::<Vec<_>>();
        assert_eq!(parsed[0].lookup_key, "é");
        assert_eq!(parsed[1].lookup_key, "e\u{301}");
        assert_ne!(parsed[0].lookup_key, parsed[1].lookup_key);
        assert_eq!(parsed[2].raw_value, "AccumulationRegister.Cash&Account");
        assert_eq!(parsed[2].local_name, "Cash&Account");
        assert_eq!(parsed[2].lookup_key, "cash&account");
    }

    #[test]
    fn generated_malformed_xml_remains_a_descriptor_error() {
        let error = parse_descriptor(
            r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="document-generated">
    <name>GeneratedDocument</name>
    <registerRecords>AccumulationRegister.Stock
</mdclass:Document>"#,
            MetadataKind::Document,
            PathBuf::from("GeneratedObject.mdo"),
        )
        .expect_err("malformed XML must fail descriptor parsing");

        assert!(matches!(error, EdtMetadataObjectError::MalformedXml(_)));
    }

    #[test]
    fn reads_adopted_metadata_extension_descriptor() {
        let root = tempdir().expect("temporary directory must be created");
        let object_directory = root.path().join("SalesExtension");

        fs::create_dir_all(&object_directory).expect("object directory must be created");
        fs::write(
            object_directory.join("SalesExtension.mdo"),
            ADOPTED_DOCUMENT_XML,
        )
        .expect("descriptor must be created");

        let descriptor = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Document)
            .expect("descriptor must load");

        let extension = descriptor
            .extension()
            .expect("adopted metadata object must expose extension fact");
        assert_eq!(
            extension.extended_configuration_object_id().as_str(),
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn rejects_directory_without_descriptor() {
        let root = tempdir().expect("temporary directory must be created");

        let error = FileSystemEdtMetadataObjectReader
            .read(root.path(), MetadataKind::Document)
            .expect_err("missing descriptor must be rejected");

        assert!(
            error
                .to_string()
                .contains("EDT metadata descriptor was not found")
        );
    }

    #[test]
    fn rejects_multiple_descriptors() {
        let root = tempdir().expect("temporary directory must be created");

        fs::write(root.path().join("First.mdo"), DOCUMENT_XML)
            .expect("first descriptor must be created");
        fs::write(root.path().join("Second.mdo"), DOCUMENT_XML)
            .expect("second descriptor must be created");

        let error = FileSystemEdtMetadataObjectReader
            .read(root.path(), MetadataKind::Document)
            .expect_err("multiple descriptors must be rejected");

        assert!(
            error
                .to_string()
                .contains("multiple EDT metadata descriptors")
        );
    }
}
