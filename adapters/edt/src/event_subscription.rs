//! Typed parser for top-level EDT Event Subscription descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const EVENT_SUBSCRIPTION_ROOT: &str = "mdclass:EventSubscription";
const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";

/// Parsed EDT Event Subscription descriptor without resolved graph targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtEventSubscriptionDescriptor {
    id: EntityId,
    name: EntityName,
    synonym: Option<String>,
    event: EntityName,
    handler: EdtEventSubscriptionHandler,
    sources: Vec<EdtEventSubscriptionSourceObservation>,
    descriptor_path: PathBuf,
}

impl EdtEventSubscriptionDescriptor {
    /// Returns the stable EDT UUID identity.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the canonical Event Subscription name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the first localized synonym value when present.
    #[must_use]
    pub fn synonym(&self) -> Option<&str> {
        self.synonym.as_deref()
    }

    /// Returns the exact decoded event name.
    #[must_use]
    pub const fn event(&self) -> &EntityName {
        &self.event
    }

    /// Returns the parsed Common Module handler path.
    #[must_use]
    pub const fn handler(&self) -> &EdtEventSubscriptionHandler {
        &self.handler
    }

    /// Returns canonical source observations ordered by exact raw selector.
    #[must_use]
    pub fn sources(&self) -> &[EdtEventSubscriptionSourceObservation] {
        &self.sources
    }

    /// Returns the total number of source occurrences before aggregation.
    #[must_use]
    pub fn source_occurrence_count(&self) -> usize {
        self.sources
            .iter()
            .map(|source| source.contexts.len())
            .sum()
    }

    /// Returns the source descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }
}

/// Parsed three-component Common Module handler path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtEventSubscriptionHandler {
    raw_path: String,
    module_name: EntityName,
    procedure_name: EntityName,
}

impl EdtEventSubscriptionHandler {
    /// Returns the exact decoded serialized path.
    #[must_use]
    pub fn raw_path(&self) -> &str {
        &self.raw_path
    }

    /// Returns the declared Common Module name.
    #[must_use]
    pub const fn module_name(&self) -> &EntityName {
        &self.module_name
    }

    /// Returns the declared Procedure name without resolving it.
    #[must_use]
    pub const fn procedure_name(&self) -> &EntityName {
        &self.procedure_name
    }
}

/// Terminal parser classification for one source selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtEventSubscriptionSourceOutcomeKind {
    /// The selector prefix maps to a supported metadata family.
    Supported,
    /// The selector is well-formed but its family is not modeled by this slice.
    Unsupported,
    /// The serialized selector grammar is invalid.
    Malformed,
}

/// Typed reason for a non-supported source selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtEventSubscriptionSourceReason {
    /// The selector prefix is outside the accepted first-slice allowlist.
    UnsupportedPrefix,
    /// The serialized selector is empty.
    EmptyValue,
    /// One of the dot-separated components is empty.
    EmptyComponent,
    /// The selector contains more than two components.
    AdditionalComponents,
}

/// Canonical occurrence context retained for one equal source selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdtEventSubscriptionSourceContext {
    occurrence_ordinal: usize,
}

impl EdtEventSubscriptionSourceContext {
    /// Returns the canonical zero-based ordinal among equal occurrences.
    #[must_use]
    pub const fn occurrence_ordinal(self) -> usize {
        self.occurrence_ordinal
    }
}

/// One deterministic source-selector observation before semantic resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtEventSubscriptionSourceObservation {
    raw_selector: String,
    family: Option<EntityName>,
    target_name: Option<EntityName>,
    target_kind: Option<MetadataKind>,
    outcome: EdtEventSubscriptionSourceOutcomeKind,
    reason: Option<EdtEventSubscriptionSourceReason>,
    contexts: Vec<EdtEventSubscriptionSourceContext>,
}

impl EdtEventSubscriptionSourceObservation {
    /// Returns the exact decoded serialized selector.
    #[must_use]
    pub fn raw_selector(&self) -> &str {
        &self.raw_selector
    }

    /// Returns the parsed selector family when its component grammar is valid.
    #[must_use]
    pub const fn family(&self) -> Option<&EntityName> {
        self.family.as_ref()
    }

    /// Returns the qualified metadata name, or `None` for a bare family selector.
    #[must_use]
    pub const fn target_name(&self) -> Option<&EntityName> {
        self.target_name.as_ref()
    }

    /// Returns the mapped metadata kind for a supported selector prefix.
    #[must_use]
    pub const fn target_kind(&self) -> Option<MetadataKind> {
        self.target_kind
    }

    /// Returns the terminal parser classification.
    #[must_use]
    pub const fn outcome(&self) -> EdtEventSubscriptionSourceOutcomeKind {
        self.outcome
    }

    /// Returns the typed reason for an unsupported or malformed selector.
    #[must_use]
    pub const fn reason(&self) -> Option<EdtEventSubscriptionSourceReason> {
        self.reason
    }

    /// Returns canonical duplicate occurrence evidence.
    #[must_use]
    pub fn contexts(&self) -> &[EdtEventSubscriptionSourceContext] {
        &self.contexts
    }
}

/// Reads one top-level EDT Event Subscription descriptor.
pub trait EdtEventSubscriptionReader {
    /// Reads the single `.mdo` descriptor in `object_directory`.
    ///
    /// # Errors
    ///
    /// Returns a typed error for filesystem, XML, root, or required-field failures.
    fn read(
        &self,
        object_directory: &Path,
    ) -> Result<EdtEventSubscriptionDescriptor, EdtEventSubscriptionError>;
}

/// Filesystem implementation of [`EdtEventSubscriptionReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtEventSubscriptionReader;

impl EdtEventSubscriptionReader for FileSystemEdtEventSubscriptionReader {
    fn read(
        &self,
        object_directory: &Path,
    ) -> Result<EdtEventSubscriptionDescriptor, EdtEventSubscriptionError> {
        let descriptor_path = find_descriptor_file(object_directory)?;
        let xml = fs::read_to_string(&descriptor_path).map_err(|source| {
            EdtEventSubscriptionError::ReadFile {
                path: descriptor_path.clone(),
                source,
            }
        })?;
        parse_descriptor(&xml, descriptor_path)
    }
}

fn find_descriptor_file(object_directory: &Path) -> Result<PathBuf, EdtEventSubscriptionError> {
    if !object_directory.is_dir() {
        return Err(EdtEventSubscriptionError::ObjectDirectoryNotFound(
            object_directory.to_path_buf(),
        ));
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(object_directory).map_err(|source| {
        EdtEventSubscriptionError::ReadDirectory {
            path: object_directory.to_path_buf(),
            source,
        }
    })? {
        let entry = entry.map_err(|source| EdtEventSubscriptionError::ReadDirectoryEntry {
            path: object_directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type =
            entry
                .file_type()
                .map_err(|source| EdtEventSubscriptionError::ReadDirectoryEntry {
                    path: object_directory.to_path_buf(),
                    source,
                })?;
        if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("mdo"))
        {
            candidates.push(path);
        }
    }
    candidates.sort();

    match candidates.len() {
        0 => Err(EdtEventSubscriptionError::DescriptorNotFound(
            object_directory.to_path_buf(),
        )),
        1 => Ok(candidates.remove(0)),
        _ => Err(EdtEventSubscriptionError::MultipleDescriptors {
            directory: object_directory.to_path_buf(),
            candidates,
        }),
    }
}

#[derive(Debug, Default)]
struct RawDescriptor {
    uuid: Option<String>,
    names: Vec<String>,
    synonyms: Vec<String>,
    source_count: usize,
    source_values: Vec<String>,
    events: Vec<String>,
    handlers: Vec<String>,
}

#[allow(clippy::too_many_lines)]
fn parse_descriptor(
    xml: &str,
    descriptor_path: PathBuf,
) -> Result<EdtEventSubscriptionDescriptor, EdtEventSubscriptionError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut raw = RawDescriptor::default();
    let mut path = Vec::<String>::new();
    let mut root_seen = false;
    let mut root_closed = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(EdtEventSubscriptionError::MultipleRoots(
                            descriptor_path.clone(),
                        ));
                    }
                    raw.uuid = validate_root(&reader, &event, &descriptor_path)?;
                    root_seen = true;
                    path.push(local_name(event.name().as_ref()));
                    continue;
                }

                let element = local_name(event.name().as_ref());
                if path.len() == 1 {
                    match element.as_str() {
                        "name" => {
                            raw.names
                                .push(read_text(&mut reader, &event, &descriptor_path)?);
                            continue;
                        }
                        "source" => raw.source_count += 1,
                        "event" => {
                            raw.events
                                .push(read_text(&mut reader, &event, &descriptor_path)?);
                            continue;
                        }
                        "handler" => {
                            raw.handlers
                                .push(read_text(&mut reader, &event, &descriptor_path)?);
                            continue;
                        }
                        _ => {}
                    }
                } else if path.len() == 2 && path[1] == "source" && element == "types" {
                    raw.source_values
                        .push(read_text(&mut reader, &event, &descriptor_path)?);
                    continue;
                } else if path.len() == 2
                    && path[1] == "synonym"
                    && matches!(element.as_str(), "value" | "content")
                {
                    raw.synonyms
                        .push(read_text(&mut reader, &event, &descriptor_path)?);
                    continue;
                }
                path.push(element);
            }
            Ok(Event::Empty(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(EdtEventSubscriptionError::MultipleRoots(
                            descriptor_path.clone(),
                        ));
                    }
                    raw.uuid = validate_root(&reader, &event, &descriptor_path)?;
                    root_seen = true;
                    root_closed = true;
                    continue;
                }

                let element = local_name(event.name().as_ref());
                if path.len() == 1 {
                    match element.as_str() {
                        "name" => raw.names.push(String::new()),
                        "source" => raw.source_count += 1,
                        "event" => raw.events.push(String::new()),
                        "handler" => raw.handlers.push(String::new()),
                        _ => {}
                    }
                } else if path.len() == 2 && path[1] == "source" && element == "types" {
                    raw.source_values.push(String::new());
                }
            }
            Ok(Event::End(_)) => {
                if path.len() == 1 {
                    root_closed = true;
                }
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(
                Event::Decl(_)
                | Event::PI(_)
                | Event::Comment(_)
                | Event::Text(_)
                | Event::CData(_)
                | Event::DocType(_)
                | Event::GeneralRef(_),
            ) => {}
            Err(source) => return Err(malformed(&descriptor_path, source.to_string())),
        }
    }

    if !root_seen {
        return Err(EdtEventSubscriptionError::MissingRoot(descriptor_path));
    }
    if !root_closed {
        return Err(malformed(
            &descriptor_path,
            "unexpected end of file before the EventSubscription root was closed",
        ));
    }

    finish_descriptor(raw, descriptor_path)
}

fn validate_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    descriptor_path: &Path,
) -> Result<Option<String>, EdtEventSubscriptionError> {
    let root = qualified_name(event.name().as_ref());
    if root != EVENT_SUBSCRIPTION_ROOT {
        return Err(EdtEventSubscriptionError::UnexpectedRoot {
            path: descriptor_path.to_path_buf(),
            root,
        });
    }

    let mut namespace = None;
    let mut uuid = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute =
            attribute.map_err(|source| malformed(descriptor_path, source.to_string()))?;
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|source| malformed(descriptor_path, source.to_string()))?
            .into_owned();
        match attribute.key.as_ref() {
            b"xmlns:mdclass" => namespace = Some(value),
            b"uuid" if uuid.is_none() => uuid = Some(value),
            b"uuid" => return Err(EdtEventSubscriptionError::DuplicateUuid),
            _ => {}
        }
    }
    if namespace.as_deref() != Some(METADATA_NAMESPACE) {
        return Err(EdtEventSubscriptionError::UnsupportedNamespace {
            path: descriptor_path.to_path_buf(),
            namespace,
        });
    }
    Ok(uuid)
}

fn read_text(
    reader: &mut Reader<&[u8]>,
    event: &BytesStart<'_>,
    descriptor_path: &Path,
) -> Result<String, EdtEventSubscriptionError> {
    let value = reader
        .read_text(event.to_end().name())
        .map_err(|source| malformed(descriptor_path, source.to_string()))?;
    unescape(&value)
        .map_err(|source| malformed(descriptor_path, source.to_string()))
        .map(std::borrow::Cow::into_owned)
}

fn finish_descriptor(
    raw: RawDescriptor,
    descriptor_path: PathBuf,
) -> Result<EdtEventSubscriptionDescriptor, EdtEventSubscriptionError> {
    let uuid = raw.uuid.ok_or(EdtEventSubscriptionError::MissingUuid)?;
    if !is_uuid(&uuid) {
        return Err(EdtEventSubscriptionError::InvalidUuid(uuid));
    }
    let id =
        EntityId::new(uuid.clone()).map_err(|_| EdtEventSubscriptionError::InvalidUuid(uuid))?;
    let name = exactly_one(raw.names, RequiredField::Name)?;
    let name = EntityName::new(name).map_err(|_| EdtEventSubscriptionError::InvalidName)?;

    match raw.source_count {
        0 => return Err(EdtEventSubscriptionError::MissingSource),
        1 => {}
        count => return Err(EdtEventSubscriptionError::DuplicateSource(count)),
    }
    if raw.source_values.is_empty() {
        return Err(EdtEventSubscriptionError::EmptySource);
    }

    let event = exactly_one(raw.events, RequiredField::Event)?;
    let event = EntityName::new(event).map_err(|_| EdtEventSubscriptionError::InvalidEvent)?;
    let handler = exactly_one(raw.handlers, RequiredField::Handler)?;
    let handler = parse_handler(handler)?;
    let sources = finish_sources(raw.source_values);

    Ok(EdtEventSubscriptionDescriptor {
        id,
        name,
        synonym: raw.synonyms.into_iter().next(),
        event,
        handler,
        sources,
        descriptor_path,
    })
}

#[derive(Debug, Clone, Copy)]
enum RequiredField {
    Name,
    Event,
    Handler,
}

fn exactly_one(
    values: Vec<String>,
    field: RequiredField,
) -> Result<String, EdtEventSubscriptionError> {
    match values.len() {
        0 => Err(match field {
            RequiredField::Name => EdtEventSubscriptionError::MissingName,
            RequiredField::Event => EdtEventSubscriptionError::MissingEvent,
            RequiredField::Handler => EdtEventSubscriptionError::MissingHandler,
        }),
        1 => Ok(values.into_iter().next().expect("one value must exist")),
        count => Err(match field {
            RequiredField::Name => EdtEventSubscriptionError::DuplicateName(count),
            RequiredField::Event => EdtEventSubscriptionError::DuplicateEvent(count),
            RequiredField::Handler => EdtEventSubscriptionError::DuplicateHandler(count),
        }),
    }
}

fn parse_handler(
    raw_path: String,
) -> Result<EdtEventSubscriptionHandler, EdtEventSubscriptionError> {
    if raw_path.is_empty() {
        return Err(EdtEventSubscriptionError::InvalidHandler {
            value: raw_path,
            reason: EdtEventSubscriptionHandlerReason::EmptyValue,
        });
    }
    let components = raw_path.split('.').collect::<Vec<_>>();
    let reason = if components.len() < 3 {
        Some(EdtEventSubscriptionHandlerReason::MissingComponents)
    } else if components.len() > 3 {
        Some(EdtEventSubscriptionHandlerReason::AdditionalComponents)
    } else if components.iter().any(|component| component.is_empty()) {
        Some(EdtEventSubscriptionHandlerReason::EmptyComponent)
    } else if components[0] != "CommonModule" {
        Some(EdtEventSubscriptionHandlerReason::UnsupportedNamespace)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(EdtEventSubscriptionError::InvalidHandler {
            value: raw_path,
            reason,
        });
    }

    let module_name =
        EntityName::new(components[1]).map_err(|_| EdtEventSubscriptionError::InvalidHandler {
            value: raw_path.clone(),
            reason: EdtEventSubscriptionHandlerReason::EmptyComponent,
        })?;
    let procedure_name =
        EntityName::new(components[2]).map_err(|_| EdtEventSubscriptionError::InvalidHandler {
            value: raw_path.clone(),
            reason: EdtEventSubscriptionHandlerReason::EmptyComponent,
        })?;
    Ok(EdtEventSubscriptionHandler {
        raw_path,
        module_name,
        procedure_name,
    })
}

fn finish_sources(values: Vec<String>) -> Vec<EdtEventSubscriptionSourceObservation> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(value, count)| parse_source(value, count))
        .collect()
}

fn parse_source(raw_selector: String, count: usize) -> EdtEventSubscriptionSourceObservation {
    let contexts = (0..count)
        .map(|occurrence_ordinal| EdtEventSubscriptionSourceContext { occurrence_ordinal })
        .collect::<Vec<_>>();
    let malformed = |reason| EdtEventSubscriptionSourceObservation {
        raw_selector: raw_selector.clone(),
        family: None,
        target_name: None,
        target_kind: None,
        outcome: EdtEventSubscriptionSourceOutcomeKind::Malformed,
        reason: Some(reason),
        contexts: contexts.clone(),
    };
    if raw_selector.trim().is_empty() {
        return malformed(EdtEventSubscriptionSourceReason::EmptyValue);
    }
    let components = raw_selector.split('.').collect::<Vec<_>>();
    if components.len() > 2 {
        return malformed(EdtEventSubscriptionSourceReason::AdditionalComponents);
    }
    if components
        .iter()
        .any(|component| component.trim().is_empty())
    {
        return malformed(EdtEventSubscriptionSourceReason::EmptyComponent);
    }

    let Ok(family) = EntityName::new(components[0]) else {
        return malformed(EdtEventSubscriptionSourceReason::EmptyComponent);
    };
    let target_name = if let Some(component) = components.get(1) {
        let Ok(target_name) = EntityName::new(*component) else {
            return malformed(EdtEventSubscriptionSourceReason::EmptyComponent);
        };
        Some(target_name)
    } else {
        None
    };
    let target_kind = source_metadata_kind(components[0]);
    let (outcome, reason) = if target_kind.is_some() {
        (EdtEventSubscriptionSourceOutcomeKind::Supported, None)
    } else {
        (
            EdtEventSubscriptionSourceOutcomeKind::Unsupported,
            Some(EdtEventSubscriptionSourceReason::UnsupportedPrefix),
        )
    };
    EdtEventSubscriptionSourceObservation {
        raw_selector,
        family: Some(family),
        target_name,
        target_kind,
        outcome,
        reason,
        contexts,
    }
}

const fn source_metadata_kind(prefix: &str) -> Option<MetadataKind> {
    match prefix.as_bytes() {
        b"CatalogObject" | b"CatalogManager" => Some(MetadataKind::Catalog),
        b"DocumentObject" | b"DocumentManager" => Some(MetadataKind::Document),
        b"InformationRegisterRecordSet" => Some(MetadataKind::InformationRegister),
        b"AccumulationRegisterRecordSet" => Some(MetadataKind::AccumulationRegister),
        b"AccountingRegisterRecordSet" => Some(MetadataKind::AccountingRegister),
        b"CalculationRegisterRecordSet" => Some(MetadataKind::CalculationRegister),
        b"BusinessProcessObject" | b"BusinessProcessManager" => Some(MetadataKind::BusinessProcess),
        b"TaskObject" => Some(MetadataKind::Task),
        _ => None,
    }
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn qualified_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

fn malformed(path: &Path, message: impl Into<String>) -> EdtEventSubscriptionError {
    EdtEventSubscriptionError::MalformedXml {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

/// Typed reason for rejecting a serialized handler path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdtEventSubscriptionHandlerReason {
    /// The handler value is empty.
    EmptyValue,
    /// The handler has fewer than three components.
    MissingComponents,
    /// The handler has more than three components.
    AdditionalComponents,
    /// A handler component is empty.
    EmptyComponent,
    /// The first component is not `CommonModule`.
    UnsupportedNamespace,
}

/// Error produced while reading an EDT Event Subscription descriptor.
#[derive(Debug)]
pub enum EdtEventSubscriptionError {
    /// The supplied object directory does not exist.
    ObjectDirectoryNotFound(PathBuf),
    /// The object directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A directory entry or its type could not be read.
    ReadDirectoryEntry {
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
        /// Sorted candidate paths.
        candidates: Vec<PathBuf>,
    },
    /// The descriptor could not be read.
    ReadFile {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The XML stream is malformed.
    MalformedXml {
        /// Descriptor path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// No XML root was present.
    MissingRoot(PathBuf),
    /// More than one XML root was present.
    MultipleRoots(PathBuf),
    /// The root is not `mdclass:EventSubscription`.
    UnexpectedRoot {
        /// Descriptor path.
        path: PathBuf,
        /// Observed qualified root name.
        root: String,
    },
    /// The required EDT metadata namespace is absent or different.
    UnsupportedNamespace {
        /// Descriptor path.
        path: PathBuf,
        /// Observed namespace value.
        namespace: Option<String>,
    },
    /// The root UUID is missing.
    MissingUuid,
    /// More than one root UUID attribute was present.
    DuplicateUuid,
    /// The root UUID does not use canonical UUID grammar.
    InvalidUuid(String),
    /// The direct name field is missing.
    MissingName,
    /// More than one direct name field was present.
    DuplicateName(usize),
    /// The direct name is empty or invalid.
    InvalidName,
    /// The direct source container is missing.
    MissingSource,
    /// More than one direct source container was present.
    DuplicateSource(usize),
    /// The direct source container has no `types` declarations.
    EmptySource,
    /// The direct event field is missing.
    MissingEvent,
    /// More than one direct event field was present.
    DuplicateEvent(usize),
    /// The direct event is empty or invalid.
    InvalidEvent,
    /// The direct handler field is missing.
    MissingHandler,
    /// More than one direct handler field was present.
    DuplicateHandler(usize),
    /// The handler path violates the accepted parser grammar.
    InvalidHandler {
        /// Exact decoded handler value.
        value: String,
        /// Typed grammar failure.
        reason: EdtEventSubscriptionHandlerReason,
    },
}

impl Display for EdtEventSubscriptionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectDirectoryNotFound(path) => write!(
                formatter,
                "EDT Event Subscription directory was not found: {}",
                path.display()
            ),
            Self::ReadDirectory { path, source } | Self::ReadDirectoryEntry { path, source } => {
                write!(
                    formatter,
                    "failed to read EDT Event Subscription directory {}: {source}",
                    path.display()
                )
            }
            Self::DescriptorNotFound(path) => write!(
                formatter,
                "EDT Event Subscription descriptor was not found in {}",
                path.display()
            ),
            Self::MultipleDescriptors {
                directory,
                candidates,
            } => write!(
                formatter,
                "multiple EDT Event Subscription descriptors found in {}: {}",
                directory.display(),
                candidates.len()
            ),
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT Event Subscription descriptor {}: {source}",
                path.display()
            ),
            Self::MalformedXml { path, message } => write!(
                formatter,
                "malformed EDT Event Subscription XML in {}: {message}",
                path.display()
            ),
            Self::MissingRoot(path) => write!(
                formatter,
                "EDT Event Subscription XML root is missing in {}",
                path.display()
            ),
            Self::MultipleRoots(path) => write!(
                formatter,
                "multiple EDT Event Subscription XML roots found in {}",
                path.display()
            ),
            Self::UnexpectedRoot { path, root } => write!(
                formatter,
                "unexpected EDT Event Subscription root {root} in {}",
                path.display()
            ),
            Self::UnsupportedNamespace { path, namespace } => write!(
                formatter,
                "unsupported EDT Event Subscription namespace {:?} in {}",
                namespace,
                path.display()
            ),
            Self::MissingUuid => formatter.write_str("EDT Event Subscription UUID is missing"),
            Self::DuplicateUuid => formatter.write_str("EDT Event Subscription UUID is duplicated"),
            Self::InvalidUuid(value) => {
                write!(formatter, "EDT Event Subscription UUID is invalid: {value}")
            }
            Self::MissingName => formatter.write_str("EDT Event Subscription name is missing"),
            Self::DuplicateName(count) => write!(
                formatter,
                "EDT Event Subscription name is duplicated: {count} declarations"
            ),
            Self::InvalidName => formatter.write_str("EDT Event Subscription name is invalid"),
            Self::MissingSource => formatter.write_str("EDT Event Subscription source is missing"),
            Self::DuplicateSource(count) => write!(
                formatter,
                "EDT Event Subscription source is duplicated: {count} declarations"
            ),
            Self::EmptySource => formatter.write_str("EDT Event Subscription source is empty"),
            Self::MissingEvent => formatter.write_str("EDT Event Subscription event is missing"),
            Self::DuplicateEvent(count) => write!(
                formatter,
                "EDT Event Subscription event is duplicated: {count} declarations"
            ),
            Self::InvalidEvent => formatter.write_str("EDT Event Subscription event is invalid"),
            Self::MissingHandler => {
                formatter.write_str("EDT Event Subscription handler is missing")
            }
            Self::DuplicateHandler(count) => write!(
                formatter,
                "EDT Event Subscription handler is duplicated: {count} declarations"
            ),
            Self::InvalidHandler { value, reason } => write!(
                formatter,
                "EDT Event Subscription handler is invalid ({reason:?}): {value}"
            ),
        }
    }
}

impl std::error::Error for EdtEventSubscriptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EdtEventSubscriptionError, EdtEventSubscriptionHandlerReason, EdtEventSubscriptionReader,
        EdtEventSubscriptionSourceOutcomeKind, EdtEventSubscriptionSourceReason,
        FileSystemEdtEventSubscriptionReader, parse_descriptor,
    };
    #[cfg(feature = "external-edt-corpus-tests")]
    use oneagent_metadata::MetadataKind;
    #[cfg(feature = "external-edt-corpus-tests")]
    use std::collections::BTreeSet;
    use std::fmt::Write as _;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    const VALID_UUID: &str = "7fa2863c-662c-4893-b42e-01a883bffc54";

    #[cfg(feature = "external-edt-corpus-tests")]
    fn repository_event_subscriptions() -> PathBuf {
        crate::live_test_support::project_root().join("src/EventSubscriptions")
    }

    fn valid_xml(source_values: &[&str], handler: &str) -> String {
        let mut sources = String::new();
        for value in source_values {
            writeln!(&mut sources, "    <types>{value}</types>")
                .expect("writing to String must succeed");
        }
        format!(
            r#"<mdclass:EventSubscription xmlns:mdclass="{METADATA_NAMESPACE}" uuid="{VALID_UUID}">
  <name>GeneratedSubscription</name>
  <synonym><key>ru</key><value>Подписка &amp; проверка</value></synonym>
  <source>
{sources}  </source>
  <event>BeforeWrite</event>
  <handler>{handler}</handler>
</mdclass:EventSubscription>"#,
            METADATA_NAMESPACE = super::METADATA_NAMESPACE,
        )
    }

    fn generated(
        xml: &str,
    ) -> Result<super::EdtEventSubscriptionDescriptor, EdtEventSubscriptionError> {
        parse_descriptor(xml, PathBuf::from("GeneratedSubscription.mdo"))
    }

    #[cfg(feature = "external-edt-corpus-tests")]
    fn read_live(name: &str) -> super::EdtEventSubscriptionDescriptor {
        FileSystemEdtEventSubscriptionReader
            .read(&repository_event_subscriptions().join(name))
            .unwrap_or_else(|error| panic!("live Event Subscription {name} must parse: {error}"))
    }

    #[test]
    #[cfg(feature = "external-edt-corpus-tests")]
    #[allow(clippy::too_many_lines)]
    fn reads_live_cardinality_matrix_and_complete_event_vocabulary() {
        for (name, uuid, synonym, event, handler, source_count) in [
            (
                "Catalogs_BeforeWrite",
                "7fa2863c-662c-4893-b42e-01a883bffc54",
                "Catalogs before write",
                "BeforeWrite",
                "CommonModule.ObjectEvents.Catalogs_BeforeWrite",
                1,
            ),
            (
                "AccountingRules_Posting",
                "83ca0a9d-722f-40c1-8ab8-67c0eace2fe6",
                "Accounting rules posting",
                "Posting",
                "CommonModule.Accounting.RulesHandler",
                30,
            ),
            (
                "CheckSafeModeBeforeWritingRecordSet",
                "667b0f85-b82d-4d8e-9d75-b55a9723dce7",
                "Before you start writing, check if the safe mode is on.",
                "BeforeWrite",
                "CommonModule.StandardSubsystemsServer.CheckSafeModeBeforeWritingRecordSet",
                41,
            ),
            (
                "CheckSafeModeBeforeWrite",
                "82035887-4f22-435b-a43f-b5ef9cb845ce",
                "Before you start writing, check if the safe mode is on.",
                "BeforeWrite",
                "CommonModule.StandardSubsystemsServer.CheckSafeModeBeforeWrite",
                94,
            ),
        ] {
            let descriptor = read_live(name);
            assert_eq!(descriptor.id().as_str(), uuid);
            assert_eq!(descriptor.name().as_str(), name);
            assert_eq!(descriptor.synonym(), Some(synonym));
            assert_eq!(descriptor.event().as_str(), event);
            assert_eq!(descriptor.handler().raw_path(), handler);
            assert_eq!(descriptor.source_occurrence_count(), source_count);
            assert_eq!(descriptor, read_live(name));
        }

        let mut events = BTreeSet::new();
        let mut supported_prefixes = BTreeSet::new();
        let mut unsupported_prefixes = BTreeSet::new();
        let mut directories = fs::read_dir(repository_event_subscriptions())
            .expect("live Event Subscription directory must be readable")
            .map(|entry| entry.expect("directory entry must be readable").path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        directories.sort();
        assert_eq!(directories.len(), 99);
        for directory in directories {
            let descriptor = FileSystemEdtEventSubscriptionReader
                .read(&directory)
                .expect("every live Event Subscription must parse");
            events.insert(descriptor.event().as_str().to_owned());
            for source in descriptor.sources() {
                let family = source
                    .family()
                    .expect("live selector grammar must be valid")
                    .as_str()
                    .to_owned();
                match source.outcome() {
                    EdtEventSubscriptionSourceOutcomeKind::Supported => {
                        supported_prefixes.insert(family);
                    }
                    EdtEventSubscriptionSourceOutcomeKind::Unsupported => {
                        unsupported_prefixes.insert(family);
                    }
                    EdtEventSubscriptionSourceOutcomeKind::Malformed => {
                        panic!("live selector must not be malformed")
                    }
                }
            }
        }
        assert_eq!(
            events,
            BTreeSet::from([
                "AfterWriteDataHistoryVersionsProcessing".to_owned(),
                "BeforeDelete".to_owned(),
                "BeforeWrite".to_owned(),
                "FillCheckProcessing".to_owned(),
                "Filling".to_owned(),
                "OnCopy".to_owned(),
                "OnReceiveDataFromMaster".to_owned(),
                "OnReceiveDataFromSlave".to_owned(),
                "OnSendDataToMaster".to_owned(),
                "OnSendDataToSlave".to_owned(),
                "OnSendNodeDataToSlave".to_owned(),
                "OnSetNewCode".to_owned(),
                "OnSetNewNumber".to_owned(),
                "OnWrite".to_owned(),
                "Posting".to_owned(),
                "PresentationFieldsGetProcessing".to_owned(),
                "PresentationGetProcessing".to_owned(),
                "UndoPosting".to_owned(),
            ])
        );
        assert_eq!(
            supported_prefixes,
            BTreeSet::from([
                "AccountingRegisterRecordSet".to_owned(),
                "AccumulationRegisterRecordSet".to_owned(),
                "BusinessProcessManager".to_owned(),
                "BusinessProcessObject".to_owned(),
                "CalculationRegisterRecordSet".to_owned(),
                "CatalogManager".to_owned(),
                "CatalogObject".to_owned(),
                "DocumentManager".to_owned(),
                "DocumentObject".to_owned(),
                "InformationRegisterRecordSet".to_owned(),
                "TaskObject".to_owned(),
            ])
        );
        assert_eq!(
            unsupported_prefixes,
            BTreeSet::from([
                "ChartOfAccountsObject".to_owned(),
                "ChartOfCalculationTypesObject".to_owned(),
                "ChartOfCharacteristicTypesObject".to_owned(),
                "ConstantValueManager".to_owned(),
                "DefinedType".to_owned(),
                "ExchangePlanObject".to_owned(),
            ])
        );
    }

    #[test]
    #[cfg(feature = "external-edt-corpus-tests")]
    fn preserves_synonym_bare_and_qualified_selector_and_handler_spelling() {
        let bare = read_live("Catalogs_BeforeWrite");
        assert_eq!(bare.synonym(), Some("Catalogs before write"));
        let [source] = bare.sources() else {
            panic!("bare live descriptor must have one source");
        };
        assert_eq!(source.raw_selector(), "CatalogObject");
        assert_eq!(
            source.family().map(oneagent_common::EntityName::as_str),
            Some("CatalogObject")
        );
        assert_eq!(source.target_name(), None);
        assert_eq!(source.target_kind(), Some(MetadataKind::Catalog));

        let qualified = read_live("AccountingRules_Posting");
        assert!(qualified.sources().iter().all(|source| {
            source.family().map(oneagent_common::EntityName::as_str) == Some("DocumentObject")
                && source.target_name().is_some()
                && source.target_kind() == Some(MetadataKind::Document)
        }));

        let multilingual = read_live("OnReceiveDataFromMaster");
        assert_eq!(
            multilingual.synonym(),
            Some("On receiving data from the master node")
        );
        assert_eq!(
            multilingual.handler().module_name().as_str(),
            "StandardSubsystemsServer"
        );
        assert_eq!(
            multilingual.handler().procedure_name().as_str(),
            "OnReceiveDataFromMasterEvent"
        );
    }

    #[test]
    fn canonicalizes_reordered_sources_and_retains_duplicate_evidence() {
        let first = generated(&valid_xml(
            &[
                "DocumentManager.Sales",
                "CatalogObject",
                "ConstantValueManager.Flag",
                "DocumentManager.Sales",
                "",
                ".Broken",
                "Deep.Value.Extra",
            ],
            "CommonModule.Events.BeforeWrite",
        ))
        .expect("generated descriptor must parse");
        let reordered = generated(&valid_xml(
            &[
                "Deep.Value.Extra",
                "DocumentManager.Sales",
                ".Broken",
                "",
                "ConstantValueManager.Flag",
                "CatalogObject",
                "DocumentManager.Sales",
            ],
            "CommonModule.Events.BeforeWrite",
        ))
        .expect("reordered descriptor must parse");

        assert_eq!(first, reordered);
        assert_eq!(first.source_occurrence_count(), 7);
        let duplicate = first
            .sources()
            .iter()
            .find(|source| source.raw_selector() == "DocumentManager.Sales")
            .expect("duplicate selector must exist");
        assert_eq!(duplicate.contexts().len(), 2);
        assert_eq!(duplicate.contexts()[0].occurrence_ordinal(), 0);
        assert_eq!(duplicate.contexts()[1].occurrence_ordinal(), 1);
        assert!(
            first
                .sources()
                .windows(2)
                .all(|pair| { pair[0].raw_selector() < pair[1].raw_selector() })
        );
        assert!(first.sources().iter().any(|source| {
            source.reason() == Some(EdtEventSubscriptionSourceReason::UnsupportedPrefix)
        }));
        assert!(first.sources().iter().any(|source| {
            source.reason() == Some(EdtEventSubscriptionSourceReason::EmptyValue)
        }));
        assert!(first.sources().iter().any(|source| {
            source.reason() == Some(EdtEventSubscriptionSourceReason::EmptyComponent)
        }));
        assert!(first.sources().iter().any(|source| {
            source.reason() == Some(EdtEventSubscriptionSourceReason::AdditionalComponents)
        }));
    }

    #[test]
    fn whitespace_only_source_values_and_components_are_typed() {
        let descriptor = generated(&valid_xml(
            &[" ", "CatalogObject. ", " .Products"],
            "CommonModule.Events.BeforeWrite",
        ))
        .expect("whitespace-only source components must remain recoverable");

        for (raw_selector, expected_reason) in [
            (" ", EdtEventSubscriptionSourceReason::EmptyValue),
            (
                "CatalogObject. ",
                EdtEventSubscriptionSourceReason::EmptyComponent,
            ),
            (
                " .Products",
                EdtEventSubscriptionSourceReason::EmptyComponent,
            ),
        ] {
            let source = descriptor
                .sources()
                .iter()
                .find(|source| source.raw_selector() == raw_selector)
                .expect("source observation must retain the exact raw selector");
            assert_eq!(
                source.outcome(),
                EdtEventSubscriptionSourceOutcomeKind::Malformed
            );
            assert_eq!(source.reason(), Some(expected_reason));
            assert_eq!(source.family(), None);
            assert_eq!(source.target_name(), None);
        }
    }

    #[test]
    fn accepts_absent_non_ascii_and_multilingual_synonym_without_handler_policy() {
        let present = generated(&valid_xml(
            &["CatalogObject.Products"],
            "CommonModule.PrivateHandlers.NotExportedHandler",
        ))
        .expect("non-exported spelling must parse without policy");
        assert_eq!(present.synonym(), Some("Подписка & проверка"));

        let absent_xml = valid_xml(
            &["CatalogObject.Products"],
            "CommonModule.PublicHandlers.ExportedHandler",
        )
        .replace(
            "  <synonym><key>ru</key><value>Подписка &amp; проверка</value></synonym>\n",
            "",
        );
        let absent = generated(&absent_xml).expect("absent synonym must parse");
        assert_eq!(absent.synonym(), None);
        assert_eq!(
            absent.handler().raw_path(),
            "CommonModule.PublicHandlers.ExportedHandler"
        );
    }

    #[test]
    fn required_field_root_and_handler_failures_are_typed() {
        let valid = valid_xml(&["CatalogObject"], "CommonModule.Events.BeforeWrite");
        assert!(matches!(
            generated(&valid.replace(&format!(" uuid=\"{VALID_UUID}\""), "")),
            Err(EdtEventSubscriptionError::MissingUuid)
        ));
        assert!(matches!(
            generated(&valid.replace(VALID_UUID, "not-a-uuid")),
            Err(EdtEventSubscriptionError::InvalidUuid(_))
        ));
        assert!(matches!(
            generated(&valid.replace(
                &format!("uuid=\"{VALID_UUID}\""),
                &format!("uuid=\"{VALID_UUID}\" uuid=\"{VALID_UUID}\"")
            )),
            Err(EdtEventSubscriptionError::DuplicateUuid)
        ));
        assert!(matches!(
            generated(&valid.replace("<name>GeneratedSubscription</name>", "")),
            Err(EdtEventSubscriptionError::MissingName)
        ));
        assert!(matches!(
            generated(&valid.replace(
                "<name>GeneratedSubscription</name>",
                "<name>GeneratedSubscription</name><name>Duplicate</name>"
            )),
            Err(EdtEventSubscriptionError::DuplicateName(2))
        ));
        assert!(matches!(
            generated(&valid.replace("<name>GeneratedSubscription</name>", "<name/>")),
            Err(EdtEventSubscriptionError::InvalidName)
        ));
        assert!(matches!(
            generated(&valid.replace(
                "<source>\n    <types>CatalogObject</types>\n  </source>",
                ""
            )),
            Err(EdtEventSubscriptionError::MissingSource)
        ));
        assert!(matches!(
            generated(&valid.replace("    <types>CatalogObject</types>\n", "")),
            Err(EdtEventSubscriptionError::EmptySource)
        ));
        assert!(matches!(
            generated(&valid.replace(
                "  <source>\n    <types>CatalogObject</types>\n  </source>",
                "  <source><types>CatalogObject</types></source><source><types>DocumentObject</types></source>"
            )),
            Err(EdtEventSubscriptionError::DuplicateSource(2))
        ));
        assert!(matches!(
            generated(&valid.replace("<event>BeforeWrite</event>", "<event/>")),
            Err(EdtEventSubscriptionError::InvalidEvent)
        ));
        assert!(matches!(
            generated(&valid.replace("<event>BeforeWrite</event>", "")),
            Err(EdtEventSubscriptionError::MissingEvent)
        ));
        assert!(matches!(
            generated(&valid.replace(
                "<event>BeforeWrite</event>",
                "<event>BeforeWrite</event><event>OnWrite</event>"
            )),
            Err(EdtEventSubscriptionError::DuplicateEvent(2))
        ));
        assert!(matches!(
            generated(&valid.replace(
                "<handler>CommonModule.Events.BeforeWrite</handler>",
                "<handler>Catalog.Events.BeforeWrite</handler>"
            )),
            Err(EdtEventSubscriptionError::InvalidHandler {
                reason: EdtEventSubscriptionHandlerReason::UnsupportedNamespace,
                ..
            })
        ));
        assert!(matches!(
            generated(&valid.replace("<handler>CommonModule.Events.BeforeWrite</handler>", "")),
            Err(EdtEventSubscriptionError::MissingHandler)
        ));
        assert!(matches!(
            generated(&valid.replace(
                "<handler>CommonModule.Events.BeforeWrite</handler>",
                "<handler>CommonModule.Events.BeforeWrite</handler><handler>CommonModule.Events.OnWrite</handler>"
            )),
            Err(EdtEventSubscriptionError::DuplicateHandler(2))
        ));
        assert!(matches!(
            generated(&valid.replace("mdclass:EventSubscription", "mdclass:Document")),
            Err(EdtEventSubscriptionError::UnexpectedRoot { .. })
        ));
        assert!(matches!(
            generated(&valid.replace(super::METADATA_NAMESPACE, "urn:wrong")),
            Err(EdtEventSubscriptionError::UnsupportedNamespace { .. })
        ));
        assert!(matches!(
            generated("<mdclass:EventSubscription"),
            Err(EdtEventSubscriptionError::MalformedXml { .. })
        ));
    }

    #[test]
    fn handler_depth_and_empty_components_are_typed() {
        for (handler, expected) in [
            ("", EdtEventSubscriptionHandlerReason::EmptyValue),
            (
                "CommonModule.Events",
                EdtEventSubscriptionHandlerReason::MissingComponents,
            ),
            (
                "CommonModule.Events.BeforeWrite.Extra",
                EdtEventSubscriptionHandlerReason::AdditionalComponents,
            ),
            (
                "CommonModule..BeforeWrite",
                EdtEventSubscriptionHandlerReason::EmptyComponent,
            ),
        ] {
            assert!(matches!(
                generated(&valid_xml(&["CatalogObject"], handler)),
                Err(EdtEventSubscriptionError::InvalidHandler { reason, .. }) if reason == expected
            ));
        }
    }

    #[test]
    fn filesystem_missing_ambiguous_and_unreadable_inputs_are_typed() {
        let missing = tempdir().expect("temporary directory must be created");
        assert!(matches!(
            FileSystemEdtEventSubscriptionReader.read(missing.path()),
            Err(EdtEventSubscriptionError::DescriptorNotFound(_))
        ));

        let ambiguous = tempdir().expect("temporary directory must be created");
        let xml = valid_xml(&["CatalogObject"], "CommonModule.Events.BeforeWrite");
        fs::write(ambiguous.path().join("First.mdo"), &xml)
            .expect("first descriptor must be written");
        fs::write(ambiguous.path().join("Second.mdo"), &xml)
            .expect("second descriptor must be written");
        assert!(matches!(
            FileSystemEdtEventSubscriptionReader.read(ambiguous.path()),
            Err(EdtEventSubscriptionError::MultipleDescriptors { candidates, .. })
                if candidates.len() == 2 && candidates[0] < candidates[1]
        ));

        let unreadable = tempdir().expect("temporary directory must be created");
        fs::write(unreadable.path().join("Invalid.mdo"), [0xff_u8])
            .expect("invalid UTF-8 descriptor must be written");
        assert!(matches!(
            FileSystemEdtEventSubscriptionReader.read(unreadable.path()),
            Err(EdtEventSubscriptionError::ReadFile { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidData
        ));
    }
}
