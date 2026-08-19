//! Reader for child metadata elements embedded in EDT `.mdo` descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::NodeKind;
use oneagent_metadata::{MetadataKind, MetadataMemberPayload};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, BytesText, Event};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::EdtMetadataObjectDescriptor;

/// Supported child metadata element kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtMetadataChildKind {
    /// Metadata object attribute.
    Attribute,

    /// Metadata object tabular section.
    TabularSection,

    /// Register dimension.
    Dimension,

    /// Register resource.
    Resource,

    /// Metadata object form.
    Form,

    /// Metadata object command.
    Command,
}

impl EdtMetadataChildKind {
    /// Returns a stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Attribute => "attribute",
            Self::TabularSection => "tabular_section",
            Self::Dimension => "dimension",
            Self::Resource => "resource",
            Self::Form => "form",
            Self::Command => "command",
        }
    }

    /// Returns the corresponding semantic graph node kind.
    #[must_use]
    pub const fn node_kind(self) -> NodeKind {
        match self {
            Self::Attribute => NodeKind::Attribute,
            Self::TabularSection => NodeKind::TabularSection,
            Self::Dimension => NodeKind::Dimension,
            Self::Resource => NodeKind::Resource,
            Self::Form => NodeKind::Form,
            Self::Command => NodeKind::Command,
        }
    }
}

/// Child element declared inside an EDT metadata object descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtMetadataChildDescriptor {
    id: EntityId,
    name: EntityName,
    kind: EdtMetadataChildKind,
    parent_id: EntityId,
    member_payload: MetadataMemberPayload,
    references: Vec<EdtMetadataReferenceDescriptor>,
}

impl EdtMetadataChildDescriptor {
    /// Creates a child metadata descriptor.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        kind: EdtMetadataChildKind,
        parent_id: EntityId,
        references: Vec<EdtMetadataReferenceDescriptor>,
    ) -> Self {
        Self::new_with_member_payload(
            id,
            name,
            kind,
            parent_id,
            MetadataMemberPayload::empty(),
            references,
        )
    }

    /// Creates a child metadata descriptor with explicit member content.
    #[must_use]
    pub const fn new_with_member_payload(
        id: EntityId,
        name: EntityName,
        kind: EdtMetadataChildKind,
        parent_id: EntityId,
        member_payload: MetadataMemberPayload,
        references: Vec<EdtMetadataReferenceDescriptor>,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            parent_id,
            member_payload,
            references,
        }
    }

    /// Returns the stable child identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the child name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the child element kind.
    #[must_use]
    pub const fn kind(&self) -> EdtMetadataChildKind {
        self.kind
    }

    /// Returns the immediate semantic owner identifier.
    #[must_use]
    pub const fn parent_id(&self) -> &EntityId {
        &self.parent_id
    }

    /// Returns accepted source-independent member content.
    #[must_use]
    pub const fn member_payload(&self) -> &MetadataMemberPayload {
        &self.member_payload
    }

    /// Returns explicit metadata references declared by this child element.
    #[must_use]
    pub fn references(&self) -> &[EdtMetadataReferenceDescriptor] {
        &self.references
    }
}

/// Semantic role of an explicit EDT metadata reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtMetadataReferenceRole {
    /// Metadata object reference declared as an EDT type.
    Type,
}

impl EdtMetadataReferenceRole {
    /// Returns a stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
        }
    }
}

/// Explicit metadata object reference declared by a child metadata element.
///
/// EDT metadata type values are normalized from supported `*Ref.Name` forms
/// into a target metadata kind and canonical target name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdtMetadataReferenceDescriptor {
    role: EdtMetadataReferenceRole,
    target_kind: MetadataKind,
    target_name: EntityName,
}

impl EdtMetadataReferenceDescriptor {
    /// Creates a metadata type reference descriptor.
    #[must_use]
    pub const fn new(target_kind: MetadataKind, target_name: EntityName) -> Self {
        Self::new_with_role(EdtMetadataReferenceRole::Type, target_kind, target_name)
    }

    /// Creates a metadata reference descriptor with an explicit semantic role.
    #[must_use]
    pub const fn new_with_role(
        role: EdtMetadataReferenceRole,
        target_kind: MetadataKind,
        target_name: EntityName,
    ) -> Self {
        Self {
            role,
            target_kind,
            target_name,
        }
    }

    /// Returns the semantic reference role.
    #[must_use]
    pub const fn role(&self) -> EdtMetadataReferenceRole {
        self.role
    }

    /// Returns the expected target metadata object kind.
    #[must_use]
    pub const fn target_kind(&self) -> MetadataKind {
        self.target_kind
    }

    /// Returns the target metadata object semantic name.
    #[must_use]
    pub const fn target_name(&self) -> &EntityName {
        &self.target_name
    }
}

/// Reads child elements from an EDT metadata object descriptor.
pub trait EdtMetadataStructureReader {
    /// Reads child metadata elements from `descriptor`.
    ///
    /// # Errors
    ///
    /// Returns an error when the descriptor cannot be read or parsed.
    fn read_children(
        &self,
        descriptor: &EdtMetadataObjectDescriptor,
    ) -> Result<Vec<EdtMetadataChildDescriptor>, EdtMetadataStructureError>;
}

/// Filesystem implementation of [`EdtMetadataStructureReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtMetadataStructureReader;

impl EdtMetadataStructureReader for FileSystemEdtMetadataStructureReader {
    fn read_children(
        &self,
        descriptor: &EdtMetadataObjectDescriptor,
    ) -> Result<Vec<EdtMetadataChildDescriptor>, EdtMetadataStructureError> {
        let path = descriptor.descriptor_path();

        let xml =
            fs::read_to_string(path).map_err(|source| EdtMetadataStructureError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;

        parse_children(&xml, descriptor.id(), path)
    }
}

#[derive(Debug)]
struct PendingChild {
    kind: EdtMetadataChildKind,
    uuid: Option<String>,
    name: Option<String>,
    synonym_container_count: usize,
    synonym_value_count: usize,
    synonym: Option<String>,
    unsupported_synonym_encoding: bool,
    references: Vec<EdtMetadataReferenceDescriptor>,
    nested_attributes: Vec<PendingChild>,
    depth: usize,
}

fn parse_children(
    xml: &str,
    parent_id: &EntityId,
    descriptor_path: &Path,
) -> Result<Vec<EdtMetadataChildDescriptor>, EdtMetadataStructureError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut path = Vec::<String>::new();
    let mut pending = Vec::<PendingChild>::new();
    let mut children = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let element_name = local_name(event.name().as_ref());
                path.push(element_name.clone());

                if let Some(kind) = child_kind(&element_name) {
                    pending.push(PendingChild {
                        kind,
                        uuid: read_uuid(&reader, &event)?,
                        name: None,
                        synonym_container_count: 0,
                        synonym_value_count: 0,
                        synonym: None,
                        unsupported_synonym_encoding: false,
                        references: Vec::new(),
                        nested_attributes: Vec::new(),
                        depth: path.len(),
                    });
                }

                if let Some(child) = pending.last_mut() {
                    observe_member_synonym_start(child, &path);
                }
            }

            Ok(Event::Empty(event)) => {
                let element_name = local_name(event.name().as_ref());

                if let Some(kind) = child_kind(&element_name) {
                    let uuid = read_uuid(&reader, &event)?;

                    return Err(match uuid {
                        Some(_) => EdtMetadataStructureError::MissingName {
                            path: descriptor_path.to_path_buf(),
                            kind,
                        },
                        None => EdtMetadataStructureError::MissingIdentifierAndName {
                            path: descriptor_path.to_path_buf(),
                            kind,
                        },
                    });
                }

                if let Some(child) = pending.last_mut() {
                    observe_member_synonym_empty(child, &path, &element_name);
                }
            }

            Ok(Event::Text(event)) => {
                observe_child_text(&event, &path, &mut pending, descriptor_path)?;
            }

            Ok(Event::End(_)) => {
                if let Some(child) = pending.pop_if(|child| child.depth == path.len()) {
                    if child.kind == EdtMetadataChildKind::Attribute
                        && let Some(owner) = pending.last_mut()
                        && owner.kind == EdtMetadataChildKind::TabularSection
                    {
                        owner.nested_attributes.push(child);
                    } else {
                        children.extend(finish_child(child, parent_id, descriptor_path)?);
                    }
                }

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
                return Err(EdtMetadataStructureError::MalformedXml(source.to_string()));
            }
        }
    }

    Ok(children)
}

fn observe_child_text(
    event: &BytesText<'_>,
    path: &[String],
    pending: &mut [PendingChild],
    descriptor_path: &Path,
) -> Result<(), EdtMetadataStructureError> {
    let Some(child) = pending.last_mut() else {
        return Ok(());
    };

    if path.last().is_some_and(|element| element == "name") && child.name.is_none() {
        child.name = Some(decode_text(event)?);
    }

    if path.last().is_some_and(|element| element == "types") {
        let value = decode_text(event)?;
        if let Some(reference) = parse_metadata_reference_type(&value).map_err(|()| {
            EdtMetadataStructureError::InvalidReferenceName {
                path: descriptor_path.to_path_buf(),
                type_name: value,
            }
        })? {
            child.references.push(reference);
        }
    }

    if is_direct_member_synonym_value(path, child.depth) {
        let value = decode_text(event)?;
        if !value.is_empty() {
            match &mut child.synonym {
                Some(current) => current.push_str(&value),
                None => child.synonym = Some(value),
            }
        }
    }

    Ok(())
}

fn decode_text(event: &BytesText<'_>) -> Result<String, EdtMetadataStructureError> {
    event
        .decode()
        .map(std::borrow::Cow::into_owned)
        .map_err(|source| EdtMetadataStructureError::MalformedXml(source.to_string()))
}

fn observe_member_synonym_start(child: &mut PendingChild, path: &[String]) {
    if !is_member_kind(child.kind) {
        return;
    }

    if path.len() == child.depth + 1 && path.last().is_some_and(|element| element == "synonym") {
        child.synonym_container_count += 1;
    } else if is_direct_member_synonym_element(path, child.depth, "value") {
        child.synonym_value_count += 1;
    } else if is_direct_member_synonym_element(path, child.depth, "content") {
        child.unsupported_synonym_encoding = true;
    }
}

fn observe_member_synonym_empty(child: &mut PendingChild, path: &[String], element_name: &str) {
    if !is_member_kind(child.kind) {
        return;
    }

    if path.len() == child.depth && element_name == "synonym" {
        child.synonym_container_count += 1;
    } else if path.len() == child.depth + 1
        && path.last().is_some_and(|element| element == "synonym")
    {
        match element_name {
            "value" => child.synonym_value_count += 1,
            "content" => child.unsupported_synonym_encoding = true,
            _ => {}
        }
    }
}

fn is_direct_member_synonym_value(path: &[String], child_depth: usize) -> bool {
    is_direct_member_synonym_element(path, child_depth, "value")
}

fn is_direct_member_synonym_element(
    path: &[String],
    child_depth: usize,
    element_name: &str,
) -> bool {
    path.len() == child_depth + 2
        && path
            .get(child_depth)
            .is_some_and(|element| element == "synonym")
        && path.last().is_some_and(|element| element == element_name)
}

const fn is_member_kind(kind: EdtMetadataChildKind) -> bool {
    matches!(
        kind,
        EdtMetadataChildKind::Attribute | EdtMetadataChildKind::TabularSection
    )
}

fn finish_member_payload(
    child: &PendingChild,
    descriptor_path: &Path,
) -> Result<MetadataMemberPayload, EdtMetadataStructureError> {
    if !is_member_kind(child.kind) || child.synonym_container_count == 0 {
        return Ok(MetadataMemberPayload::empty());
    }

    if child.synonym_container_count > 1 || child.synonym_value_count > 1 {
        return Err(EdtMetadataStructureError::DuplicateMemberSynonym {
            path: descriptor_path.to_path_buf(),
            kind: child.kind,
        });
    }

    if child.unsupported_synonym_encoding
        || child.synonym_value_count != 1
        || child.synonym.as_deref().is_none_or(str::is_empty)
    {
        return Err(EdtMetadataStructureError::InvalidMemberSynonym {
            path: descriptor_path.to_path_buf(),
            kind: child.kind,
        });
    }

    Ok(MetadataMemberPayload::new(child.synonym.clone()))
}

fn finish_child(
    mut child: PendingChild,
    parent_id: &EntityId,
    descriptor_path: &Path,
) -> Result<Vec<EdtMetadataChildDescriptor>, EdtMetadataStructureError> {
    let nested_attributes = std::mem::take(&mut child.nested_attributes);
    let raw_name = child
        .name
        .as_deref()
        .ok_or_else(|| EdtMetadataStructureError::MissingName {
            path: descriptor_path.to_path_buf(),
            kind: child.kind,
        })?;

    let name = EntityName::new(raw_name).map_err(|_| EdtMetadataStructureError::InvalidName {
        path: descriptor_path.to_path_buf(),
        name: raw_name.to_owned(),
    })?;

    let raw_id = child.uuid.take().unwrap_or_else(|| {
        format!(
            "{}:{}:{}",
            parent_id.as_str(),
            child.kind.as_str(),
            name.as_str()
        )
    });

    let id = EntityId::new(&raw_id).map_err(|_| EdtMetadataStructureError::InvalidIdentifier {
        path: descriptor_path.to_path_buf(),
        identifier: raw_id,
    })?;
    let member_payload = finish_member_payload(&child, descriptor_path)?;

    let descriptor = EdtMetadataChildDescriptor::new_with_member_payload(
        id,
        name,
        child.kind,
        parent_id.clone(),
        member_payload,
        child.references,
    );
    let immediate_owner_id = descriptor.id().clone();
    let mut descriptors = vec![descriptor];

    for nested_attribute in nested_attributes {
        descriptors.extend(finish_child(
            nested_attribute,
            &immediate_owner_id,
            descriptor_path,
        )?);
    }

    Ok(descriptors)
}

fn parse_metadata_reference_type(
    value: &str,
) -> Result<Option<EdtMetadataReferenceDescriptor>, ()> {
    let value = value.trim();
    let value = value.rsplit(':').next().unwrap_or(value);
    let Some((prefix, target_name)) = value.split_once('.') else {
        return Ok(None);
    };

    let Some(target_kind) = metadata_reference_kind(prefix) else {
        return Ok(None);
    };

    let target_name = EntityName::new(target_name).map_err(|_| ())?;

    Ok(Some(EdtMetadataReferenceDescriptor::new(
        target_kind,
        target_name,
    )))
}

fn metadata_reference_kind(prefix: &str) -> Option<MetadataKind> {
    match prefix {
        "CatalogRef" => Some(MetadataKind::Catalog),
        "DocumentRef" => Some(MetadataKind::Document),
        "EnumRef" => Some(MetadataKind::Enumeration),
        "InformationRegisterRecordSet" | "InformationRegisterRecordKey" => {
            Some(MetadataKind::InformationRegister)
        }
        "AccumulationRegisterRecordSet" | "AccumulationRegisterRecordKey" => {
            Some(MetadataKind::AccumulationRegister)
        }
        "AccountingRegisterRecordSet" | "AccountingRegisterRecordKey" => {
            Some(MetadataKind::AccountingRegister)
        }
        "CalculationRegisterRecordSet" | "CalculationRegisterRecordKey" => {
            Some(MetadataKind::CalculationRegister)
        }
        "BusinessProcessRef" => Some(MetadataKind::BusinessProcess),
        "TaskRef" => Some(MetadataKind::Task),
        _ => None,
    }
}

fn child_kind(element_name: &str) -> Option<EdtMetadataChildKind> {
    match element_name {
        // EDT uses plural property names for collection elements.
        "attributes" | "attribute" => Some(EdtMetadataChildKind::Attribute),

        "tabularSections" | "tabularSection" => Some(EdtMetadataChildKind::TabularSection),

        "dimensions" | "dimension" => Some(EdtMetadataChildKind::Dimension),

        "resources" | "resource" => Some(EdtMetadataChildKind::Resource),

        "forms" | "form" => Some(EdtMetadataChildKind::Form),

        "commands" | "command" => Some(EdtMetadataChildKind::Command),

        _ => None,
    }
}

fn read_uuid(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>, EdtMetadataStructureError> {
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute
            .map_err(|source| EdtMetadataStructureError::MalformedXml(source.to_string()))?;

        if local_name(attribute.key.as_ref()) == "uuid" {
            return attribute
                .decode_and_unescape_value(reader.decoder())
                .map(|value| Some(value.into_owned()))
                .map_err(|source| EdtMetadataStructureError::MalformedXml(source.to_string()));
        }
    }

    Ok(None)
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

/// Error produced while reading EDT metadata object internals.
#[derive(Debug)]
pub enum EdtMetadataStructureError {
    /// The descriptor file could not be read.
    ReadFile {
        /// Descriptor path.
        path: PathBuf,

        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The descriptor contains malformed XML.
    MalformedXml(String),

    /// A child element does not contain a name.
    MissingName {
        /// Descriptor path.
        path: PathBuf,

        /// Child element kind.
        kind: EdtMetadataChildKind,
    },

    /// An empty child contains neither an identifier nor a name.
    MissingIdentifierAndName {
        /// Descriptor path.
        path: PathBuf,

        /// Child element kind.
        kind: EdtMetadataChildKind,
    },

    /// A child identifier is invalid.
    InvalidIdentifier {
        /// Descriptor path.
        path: PathBuf,

        /// Invalid identifier.
        identifier: String,
    },

    /// A child name is invalid.
    InvalidName {
        /// Descriptor path.
        path: PathBuf,

        /// Invalid name.
        name: String,
    },

    /// A member synonym container has no accepted direct non-empty value.
    InvalidMemberSynonym {
        /// Descriptor path.
        path: PathBuf,

        /// Child element kind.
        kind: EdtMetadataChildKind,
    },

    /// A member declares more than one direct synonym container or value.
    DuplicateMemberSynonym {
        /// Descriptor path.
        path: PathBuf,

        /// Child element kind.
        kind: EdtMetadataChildKind,
    },

    /// A metadata reference target name is invalid.
    InvalidReferenceName {
        /// Descriptor path.
        path: PathBuf,

        /// Invalid EDT type name.
        type_name: String,
    },
}

impl Display for EdtMetadataStructureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT metadata structure {}: {source}",
                path.display()
            ),

            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT metadata structure XML: {message}")
            }

            Self::MissingName { path, kind } => write!(
                formatter,
                "EDT {} in {} does not contain a name",
                kind.as_str(),
                path.display()
            ),

            Self::MissingIdentifierAndName { path, kind } => write!(
                formatter,
                "empty EDT {} in {} has no identifier or name",
                kind.as_str(),
                path.display()
            ),

            Self::InvalidIdentifier { path, identifier } => write!(
                formatter,
                "invalid EDT metadata child identifier `{identifier}` in {}",
                path.display()
            ),

            Self::InvalidName { path, name } => write!(
                formatter,
                "invalid EDT metadata child name `{name}` in {}",
                path.display()
            ),

            Self::InvalidMemberSynonym { path, kind } => write!(
                formatter,
                "EDT {} in {} has no accepted direct member synonym value",
                kind.as_str(),
                path.display()
            ),

            Self::DuplicateMemberSynonym { path, kind } => write!(
                formatter,
                "EDT {} in {} declares duplicate direct member synonym content",
                kind.as_str(),
                path.display()
            ),

            Self::InvalidReferenceName { path, type_name } => write!(
                formatter,
                "invalid EDT metadata reference type `{type_name}` in {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for EdtMetadataStructureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            Self::MalformedXml(_)
            | Self::MissingName { .. }
            | Self::MissingIdentifierAndName { .. }
            | Self::InvalidIdentifier { .. }
            | Self::InvalidName { .. }
            | Self::InvalidMemberSynonym { .. }
            | Self::DuplicateMemberSynonym { .. }
            | Self::InvalidReferenceName { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use crate::EdtMetadataObjectDescriptor;

    use super::{
        EdtMetadataChildDescriptor, EdtMetadataChildKind, EdtMetadataReferenceRole,
        EdtMetadataStructureError, EdtMetadataStructureReader,
        FileSystemEdtMetadataStructureReader, parse_children,
    };

    fn document_descriptor(path: PathBuf, id: &str, name: &str) -> EdtMetadataObjectDescriptor {
        EdtMetadataObjectDescriptor::new(
            EntityId::new(id).expect("identifier must be valid"),
            EntityName::new(name).expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            path,
        )
    }

    fn generated_children(
        xml: &str,
    ) -> Result<Vec<EdtMetadataChildDescriptor>, EdtMetadataStructureError> {
        parse_children(
            xml,
            &EntityId::new("document-generated").expect("identifier must be valid"),
            Path::new("generated/Document.mdo"),
        )
    }

    #[test]
    fn reads_attributes_and_tabular_sections() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="11111111-1111-1111-1111-111111111111">
        <name>Company</name>
    </attributes>

    <attributes uuid="22222222-2222-2222-2222-222222222222">
        <name>Warehouse</name>
    </attributes>

    <tabularSections uuid="33333333-3333-3333-3333-333333333333">
        <name>Goods</name>
    </tabularSections>
</mdclass:Document>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .expect("identifier must be valid"),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("metadata children must be read");

        assert_eq!(children.len(), 3);

        assert_eq!(
            children
                .iter()
                .filter(|child| child.kind() == EdtMetadataChildKind::Attribute)
                .count(),
            2
        );

        assert_eq!(
            children
                .iter()
                .filter(|child| { child.kind() == EdtMetadataChildKind::TabularSection })
                .count(),
            1
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "Company")
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "Goods")
        );
    }

    #[test]
    fn preserves_immediate_parent_for_nested_attributes() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="document-sales">
    <name>Sales</name>
    <attributes uuid="top-level-company">
        <name>Company</name>
    </attributes>
    <tabularSections>
        <name>Goods</name>
        <attributes>
            <name>Item</name>
            <type><types>CatalogRef.Products</types></type>
        </attributes>
        <attributes uuid="nested-quantity">
            <name>Quantity</name>
        </attributes>
    </tabularSections>
    <tabularSections>
        <name>Services</name>
        <attributes>
            <name>Item</name>
        </attributes>
    </tabularSections>
</mdclass:Document>"#,
        )
        .expect("descriptor must be written");

        let document_id = EntityId::new("document-sales").expect("identifier must be valid");
        let descriptor = EdtMetadataObjectDescriptor::new(
            document_id.clone(),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let first = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("metadata children must be read");
        let repeated = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("repeated metadata read must succeed");

        assert_eq!(first, repeated);
        assert_eq!(first.len(), 6);
        assert_eq!(
            first
                .iter()
                .map(|child| child.id().as_str())
                .collect::<Vec<_>>(),
            vec![
                "top-level-company",
                "document-sales:tabular_section:Goods",
                "document-sales:tabular_section:Goods:attribute:Item",
                "nested-quantity",
                "document-sales:tabular_section:Services",
                "document-sales:tabular_section:Services:attribute:Item",
            ]
        );
        assert_eq!(first[0].parent_id(), &document_id);
        assert_eq!(first[1].parent_id(), &document_id);
        assert_eq!(first[2].parent_id(), first[1].id());
        assert_eq!(first[3].parent_id(), first[1].id());
        assert_eq!(first[4].parent_id(), &document_id);
        assert_eq!(first[5].parent_id(), first[4].id());
        assert_ne!(first[2].id(), first[5].id());
        assert_eq!(first[2].references().len(), 1);
        assert_eq!(
            first[2].references()[0].target_kind(),
            MetadataKind::Catalog
        );
        assert_eq!(first[2].references()[0].target_name().as_str(), "Products");
    }

    #[test]
    fn creates_deterministic_identifier_when_uuid_is_absent() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Products.mdo");

        fs::write(
            &descriptor_path,
            r#"<mdclass:Catalog xmlns:mdclass="urn:test">
    <name>Products</name>
    <attributes>
        <name>Article</name>
    </attributes>
</mdclass:Catalog>"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("catalog-products").expect("identifier must be valid"),
            EntityName::new("Products").expect("name must be valid"),
            None,
            MetadataKind::Catalog,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("metadata children must be read");

        assert_eq!(children.len(), 1);
        assert_eq!(
            children[0].id().as_str(),
            "catalog-products:attribute:Article"
        );
    }
    #[test]
    fn reads_register_dimensions_and_resources() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("StockBalance.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:AccumulationRegister
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="44444444-4444-4444-4444-444444444444">
    <name>StockBalance</name>

    <dimensions uuid="55555555-5555-5555-5555-555555555555">
        <name>Product</name>
    </dimensions>

    <dimensions uuid="66666666-6666-6666-6666-666666666666">
        <name>Warehouse</name>
    </dimensions>

    <resources uuid="77777777-7777-7777-7777-777777777777">
        <name>Quantity</name>
    </resources>
</mdclass:AccumulationRegister>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("44444444-4444-4444-4444-444444444444")
                .expect("identifier must be valid"),
            EntityName::new("StockBalance").expect("name must be valid"),
            None,
            MetadataKind::AccumulationRegister,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("register structure must be read");

        assert_eq!(children.len(), 3);

        assert_eq!(
            children
                .iter()
                .filter(|child| child.kind() == EdtMetadataChildKind::Dimension)
                .count(),
            2
        );

        assert_eq!(
            children
                .iter()
                .filter(|child| child.kind() == EdtMetadataChildKind::Resource)
                .count(),
            1
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "Product")
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "Quantity")
        );
    }

    #[test]
    fn reads_explicit_metadata_references() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="11111111-1111-1111-1111-111111111111">
        <name>Product</name>
        <type>
            <types>CatalogRef.Products</types>
        </type>
    </attributes>

    <attributes uuid="22222222-2222-2222-2222-222222222222">
        <name>Comment</name>
        <type>
            <types>String</types>
        </type>
    </attributes>
</mdclass:Document>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .expect("identifier must be valid"),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("metadata children must be read");
        let product = children
            .iter()
            .find(|child| child.name().as_str() == "Product")
            .expect("Product attribute must exist");
        let comment = children
            .iter()
            .find(|child| child.name().as_str() == "Comment")
            .expect("Comment attribute must exist");

        assert_eq!(product.references().len(), 1);
        assert_eq!(
            product.references()[0].role(),
            EdtMetadataReferenceRole::Type
        );
        assert_eq!(product.references()[0].target_kind(), MetadataKind::Catalog);
        assert_eq!(product.references()[0].target_name().as_str(), "Products");
        assert!(comment.references().is_empty());
    }

    #[test]
    fn reads_composite_metadata_type_references() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="11111111-1111-1111-1111-111111111111">
        <name>Target</name>
        <type>
            <types>CatalogRef.Products</types>
            <types>DocumentRef.Sales</types>
        </type>
    </attributes>
</mdclass:Document>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .expect("identifier must be valid"),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("metadata children must be read");
        let target = children
            .iter()
            .find(|child| child.name().as_str() == "Target")
            .expect("Target attribute must exist");

        assert_eq!(target.references().len(), 2);
        assert_eq!(target.references()[0].target_kind(), MetadataKind::Catalog);
        assert_eq!(target.references()[0].target_name().as_str(), "Products");
        assert_eq!(target.references()[1].target_kind(), MetadataKind::Document);
        assert_eq!(target.references()[1].target_name().as_str(), "Sales");
        assert!(
            target
                .references()
                .iter()
                .all(|reference| reference.role() == EdtMetadataReferenceRole::Type)
        );
    }

    #[test]
    fn reads_metadata_object_forms() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <forms uuid="88888888-8888-8888-8888-888888888888">
        <name>DocumentForm</name>
    </forms>

    <forms uuid="99999999-9999-9999-9999-999999999999">
        <name>ListForm</name>
    </forms>
</mdclass:Document>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .expect("identifier must be valid"),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("forms must be read");

        assert_eq!(children.len(), 2);

        assert!(
            children
                .iter()
                .all(|child| child.kind() == EdtMetadataChildKind::Form)
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "DocumentForm")
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "ListForm")
        );
    }
    #[test]
    fn reads_metadata_object_commands() {
        let root = tempdir().expect("temporary directory must be created");
        let descriptor_path = root.path().join("Sales.mdo");

        fs::write(
            &descriptor_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <commands uuid="12121212-1212-1212-1212-121212121212">
        <name>PostAndClose</name>
    </commands>

    <commands uuid="34343434-3434-3434-3434-343434343434">
        <name>Print</name>
    </commands>
</mdclass:Document>
"#,
        )
        .expect("descriptor must be written");

        let descriptor = EdtMetadataObjectDescriptor::new(
            EntityId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .expect("identifier must be valid"),
            EntityName::new("Sales").expect("name must be valid"),
            None,
            MetadataKind::Document,
            None,
            descriptor_path,
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("commands must be read");

        assert_eq!(children.len(), 2);

        assert!(
            children
                .iter()
                .all(|child| child.kind() == EdtMetadataChildKind::Command)
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "PostAndClose")
        );

        assert!(
            children
                .iter()
                .any(|child| child.name().as_str() == "Print")
        );
    }

    #[test]
    fn reads_real_present_member_synonyms_repeatedly() {
        let descriptor_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/grants_project/src/Documents/Sale/Sale.mdo");
        let descriptor = document_descriptor(
            descriptor_path,
            "d9ad89c3-91b4-4d3e-bbfb-1b6e930a38f8",
            "Sale",
        );

        let first = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("real grants structure must be read");
        let repeated = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("repeated grants structure read must succeed");

        assert_eq!(first, repeated);
        assert_eq!(first.len(), 5);
        for (name, synonym) in [
            ("Proucts", Some("Proucts")),
            ("Реквизит1", None),
            ("Price", Some("Price")),
            ("Quantity", Some("Quantity")),
            ("Ammount", Some("Ammount")),
        ] {
            let child = first
                .iter()
                .find(|child| child.name().as_str() == name)
                .expect("real fixture child must exist");
            assert_eq!(child.member_payload().synonym(), synonym);
        }
    }

    #[test]
    fn reads_real_absent_member_synonyms_without_synthesis() {
        let descriptor_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/ownership_project/src/Documents/Sales/Sales.mdo");
        let descriptor = document_descriptor(
            descriptor_path,
            "20000000-0000-0000-0000-000000000000",
            "Sales",
        );

        let children = FileSystemEdtMetadataStructureReader
            .read_children(&descriptor)
            .expect("real ownership structure must be read");

        assert_eq!(children.len(), 4);
        assert!(
            children
                .iter()
                .all(|child| child.member_payload().synonym().is_none())
        );
    }

    #[test]
    fn direct_member_synonym_is_non_ascii_order_independent_and_owner_scoped() {
        let xml = |synonym_body: &str| {
            format!(
                r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="document-generated">
  <name>Generated</name>
  <synonym><value>Document display</value></synonym>
  <attributes uuid="top-attribute">
    <name>TopAttribute</name>
    <synonym>{synonym_body}</synonym>
  </attributes>
  <tabularSections uuid="section">
    <name>Lines</name>
    <attributes uuid="nested-attribute">
      <name>NestedAttribute</name>
      <synonym><key>ru</key><value>Вложенный реквизит</value></synonym>
    </attributes>
  </tabularSections>
</mdclass:Document>"#
            )
        };

        let key_first = generated_children(&xml("<key>ru</key><value>Верхний реквизит</value>"))
            .expect("key-first synonym must be parsed");
        let value_first = generated_children(&xml("<value>Верхний реквизит</value><key>ru</key>"))
            .expect("value-first synonym must be parsed");

        assert_eq!(key_first, value_first);
        assert_eq!(key_first[0].id().as_str(), "top-attribute");
        assert_eq!(
            key_first[0].member_payload().synonym(),
            Some("Верхний реквизит")
        );
        assert_eq!(key_first[1].id().as_str(), "section");
        assert_eq!(key_first[1].member_payload().synonym(), None);
        assert_eq!(key_first[2].parent_id(), key_first[1].id());
        assert_eq!(
            key_first[2].member_payload().synonym(),
            Some("Вложенный реквизит")
        );
    }

    #[test]
    fn rejects_invalid_and_duplicate_direct_member_synonyms_deterministically() {
        let xml = |synonym: &str| {
            format!(
                r#"<mdclass:Document xmlns:mdclass="urn:test">
  <name>Generated</name>
  <attributes uuid="attribute">
    <name>Attribute</name>
    {synonym}
  </attributes>
</mdclass:Document>"#
            )
        };
        let cases = [
            ("<synonym><key>ru</key></synonym>", false),
            ("<synonym><value/></synonym>", false),
            ("<synonym><content>Display</content></synonym>", false),
            (
                "<synonym><value>One</value></synonym><synonym><value>Two</value></synonym>",
                true,
            ),
            (
                "<synonym><value>One</value><value>Two</value></synonym>",
                true,
            ),
        ];

        for (synonym, duplicate) in cases {
            let source = xml(synonym);
            let first = generated_children(&source).expect_err("member synonym must be rejected");
            let repeated =
                generated_children(&source).expect_err("repeated invalid read must fail");

            assert_eq!(first.to_string(), repeated.to_string());
            match first {
                EdtMetadataStructureError::DuplicateMemberSynonym { path, kind } if duplicate => {
                    assert_eq!(path, Path::new("generated/Document.mdo"));
                    assert_eq!(kind, EdtMetadataChildKind::Attribute);
                }
                EdtMetadataStructureError::InvalidMemberSynonym { path, kind } if !duplicate => {
                    assert_eq!(path, Path::new("generated/Document.mdo"));
                    assert_eq!(kind, EdtMetadataChildKind::Attribute);
                }
                error => panic!("unexpected member synonym error: {error}"),
            }
        }
    }

    #[test]
    fn malformed_member_synonym_remains_a_malformed_xml_error() {
        let error = generated_children(
            r#"<mdclass:Document xmlns:mdclass="urn:test">
  <name>Generated</name>
  <attributes><name>Attribute</name><synonym><value>Broken</synonym></attributes>
</mdclass:Document>"#,
        )
        .expect_err("malformed member synonym XML must be rejected");

        assert!(matches!(error, EdtMetadataStructureError::MalformedXml(_)));
    }
}
