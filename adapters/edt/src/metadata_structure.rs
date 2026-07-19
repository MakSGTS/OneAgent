//! Reader for child metadata elements embedded in EDT `.mdo` descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::NodeKind;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
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
}

impl EdtMetadataChildDescriptor {
    /// Creates a child metadata descriptor.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        kind: EdtMetadataChildKind,
        parent_id: EntityId,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            parent_id,
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

    /// Returns the containing metadata object identifier.
    #[must_use]
    pub const fn parent_id(&self) -> &EntityId {
        &self.parent_id
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
                        depth: path.len(),
                    });
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
            }

            Ok(Event::Text(event)) => {
                if path.last().is_some_and(|element| element == "name")
                    && let Some(child) = pending.last_mut()
                    && child.name.is_none()
                {
                    child.name = Some(
                        event
                            .decode()
                            .map_err(|source| {
                                EdtMetadataStructureError::MalformedXml(source.to_string())
                            })?
                            .into_owned(),
                    );
                }
            }

            Ok(Event::End(_)) => {
                if let Some(child) = pending.pop_if(|child| child.depth == path.len()) {
                    children.push(finish_child(child, parent_id, descriptor_path)?);
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

fn finish_child(
    child: PendingChild,
    parent_id: &EntityId,
    descriptor_path: &Path,
) -> Result<EdtMetadataChildDescriptor, EdtMetadataStructureError> {
    let raw_name = child
        .name
        .ok_or_else(|| EdtMetadataStructureError::MissingName {
            path: descriptor_path.to_path_buf(),
            kind: child.kind,
        })?;

    let name = EntityName::new(&raw_name).map_err(|_| EdtMetadataStructureError::InvalidName {
        path: descriptor_path.to_path_buf(),
        name: raw_name.clone(),
    })?;

    let raw_id = child.uuid.unwrap_or_else(|| {
        format!(
            "{}:{}:{}",
            parent_id.as_str(),
            child.kind.as_str(),
            raw_name
        )
    });

    let id = EntityId::new(&raw_id).map_err(|_| EdtMetadataStructureError::InvalidIdentifier {
        path: descriptor_path.to_path_buf(),
        identifier: raw_id,
    })?;

    Ok(EdtMetadataChildDescriptor::new(
        id,
        name,
        child.kind,
        parent_id.clone(),
    ))
}

fn child_kind(element_name: &str) -> Option<EdtMetadataChildKind> {
    match element_name {
        // EDT uses plural property names for collection elements.
        "attributes" | "attribute" => Some(EdtMetadataChildKind::Attribute),

        "tabularSections" | "tabularSection" => Some(EdtMetadataChildKind::TabularSection),

        "dimensions" | "dimension" => Some(EdtMetadataChildKind::Dimension),

        "resources" | "resource" => Some(EdtMetadataChildKind::Resource),

        "forms" | "form" => Some(EdtMetadataChildKind::Form),

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
            | Self::InvalidName { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use tempfile::tempdir;

    use crate::EdtMetadataObjectDescriptor;

    use super::{
        EdtMetadataChildKind, EdtMetadataStructureReader, FileSystemEdtMetadataStructureReader,
    };

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
}
