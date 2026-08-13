//! Reader for direct content declarations in top-level EDT Subsystem descriptors.

use oneagent_common::EntityId;
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesCData, BytesRef, BytesStart, BytesText, Event};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::EdtMetadataObjectDescriptor;

const SUBSYSTEM_ROOT: &str = "mdclass:Subsystem";
const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";

/// Direct raw content declarations read from one EDT Subsystem descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtSubsystemContentDescriptor {
    subsystem_id: EntityId,
    descriptor_path: PathBuf,
    raw_content: Vec<String>,
}

impl EdtSubsystemContentDescriptor {
    /// Returns the identifier of the declaring Subsystem metadata object.
    #[must_use]
    pub const fn subsystem_id(&self) -> &EntityId {
        &self.subsystem_id
    }

    /// Returns the source Subsystem descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    /// Returns deterministic unique raw direct `<content>` observations.
    #[must_use]
    pub fn raw_content(&self) -> &[String] {
        &self.raw_content
    }
}

/// Reads direct content declarations from an EDT Subsystem metadata descriptor.
pub trait EdtSubsystemContentReader {
    /// Reads direct raw `<content>` observations from `descriptor`.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied metadata object is not a Subsystem or
    /// when its descriptor cannot be read or does not satisfy the EDT Subsystem
    /// XML contract.
    fn read(
        &self,
        descriptor: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtSubsystemContentDescriptor, EdtSubsystemContentError>;
}

/// Filesystem implementation of [`EdtSubsystemContentReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtSubsystemContentReader;

impl EdtSubsystemContentReader for FileSystemEdtSubsystemContentReader {
    fn read(
        &self,
        descriptor: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtSubsystemContentDescriptor, EdtSubsystemContentError> {
        if descriptor.kind() != MetadataKind::Subsystem {
            return Err(EdtSubsystemContentError::UnexpectedMetadataKind(
                descriptor.kind(),
            ));
        }

        let descriptor_path = descriptor.descriptor_path();
        let xml = fs::read_to_string(descriptor_path).map_err(|source| {
            EdtSubsystemContentError::ReadFile {
                path: descriptor_path.to_path_buf(),
                source,
            }
        })?;

        parse_subsystem_content(&xml, descriptor.id().clone(), descriptor_path.to_path_buf())
    }
}

fn parse_subsystem_content(
    xml: &str,
    subsystem_id: EntityId,
    descriptor_path: PathBuf,
) -> Result<EdtSubsystemContentDescriptor, EdtSubsystemContentError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut path = Vec::<String>::new();
    let mut raw_content = BTreeSet::new();
    let mut pending_content = None::<String>;
    let mut root_seen = false;
    let mut root_closed = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(EdtSubsystemContentError::MalformedXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    validate_root(&reader, &event)?;
                    root_seen = true;
                }

                path.push(qualified_name(event.name().as_ref()));
                if is_direct_content(&path) {
                    pending_content = Some(String::new());
                }
            }
            Ok(Event::Empty(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(EdtSubsystemContentError::MalformedXml(
                            "multiple root elements".to_owned(),
                        ));
                    }
                    validate_root(&reader, &event)?;
                    root_seen = true;
                    root_closed = true;
                } else if path.len() == 1 && event.name().as_ref() == b"content" {
                    raw_content.insert(String::new());
                }
            }
            Ok(Event::Text(event)) => {
                append_text(&event, is_direct_content(&path), &mut pending_content)?;
            }
            Ok(Event::CData(event)) => {
                append_cdata(&event, is_direct_content(&path), &mut pending_content)?;
            }
            Ok(Event::GeneralRef(event)) => {
                append_reference(&event, is_direct_content(&path), &mut pending_content)?;
            }
            Ok(Event::End(_)) => {
                if is_direct_content(&path)
                    && let Some(content) = pending_content.take()
                {
                    raw_content.insert(content);
                }

                if path.len() == 1 {
                    root_closed = true;
                }
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Err(source) => {
                return Err(EdtSubsystemContentError::MalformedXml(source.to_string()));
            }
        }
    }

    if !root_seen {
        return Err(EdtSubsystemContentError::MissingRoot);
    }
    if !root_closed {
        return Err(EdtSubsystemContentError::MalformedXml(
            "unexpected end of file before the Subsystem root was closed".to_owned(),
        ));
    }

    Ok(EdtSubsystemContentDescriptor {
        subsystem_id,
        descriptor_path,
        raw_content: raw_content.into_iter().collect(),
    })
}

fn append_text(
    event: &BytesText<'_>,
    capture: bool,
    pending_content: &mut Option<String>,
) -> Result<(), EdtSubsystemContentError> {
    if capture && let Some(content) = pending_content {
        let decoded = event
            .decode()
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        let decoded = unescape(&decoded)
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        content.push_str(&decoded);
    }
    Ok(())
}

fn append_cdata(
    event: &BytesCData<'_>,
    capture: bool,
    pending_content: &mut Option<String>,
) -> Result<(), EdtSubsystemContentError> {
    if capture && let Some(content) = pending_content {
        let decoded = event
            .decode()
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        content.push_str(&decoded);
    }
    Ok(())
}

fn append_reference(
    event: &BytesRef<'_>,
    capture: bool,
    pending_content: &mut Option<String>,
) -> Result<(), EdtSubsystemContentError> {
    if capture && let Some(content) = pending_content {
        let reference = event
            .decode()
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        let encoded = format!("&{reference};");
        let decoded = unescape(&encoded)
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        content.push_str(&decoded);
    }
    Ok(())
}

fn validate_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<(), EdtSubsystemContentError> {
    let root = qualified_name(event.name().as_ref());
    if root != SUBSYSTEM_ROOT {
        return Err(EdtSubsystemContentError::UnexpectedRoot(root));
    }

    let mut namespace = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute
            .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?;
        if attribute.key.as_ref() == b"xmlns:mdclass" {
            namespace = Some(
                attribute
                    .decode_and_unescape_value(reader.decoder())
                    .map_err(|source| EdtSubsystemContentError::MalformedXml(source.to_string()))?
                    .into_owned(),
            );
        }
    }

    if namespace.as_deref() != Some(METADATA_NAMESPACE) {
        return Err(EdtSubsystemContentError::UnsupportedNamespace(namespace));
    }

    Ok(())
}

fn is_direct_content(path: &[String]) -> bool {
    path.len() == 2 && path[0] == SUBSYSTEM_ROOT && path[1] == "content"
}

fn qualified_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

/// Errors produced while reading direct EDT Subsystem content declarations.
#[derive(Debug)]
pub enum EdtSubsystemContentError {
    /// The supplied metadata descriptor is not a Subsystem.
    UnexpectedMetadataKind(MetadataKind),
    /// The Subsystem descriptor could not be read.
    ReadFile {
        /// File path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The XML document is malformed.
    MalformedXml(String),
    /// The XML document has no root element.
    MissingRoot,
    /// The XML root is not `mdclass:Subsystem`.
    UnexpectedRoot(String),
    /// The `mdclass` namespace is missing or unsupported.
    UnsupportedNamespace(Option<String>),
}

impl Display for EdtSubsystemContentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedMetadataKind(kind) => {
                write!(formatter, "expected EDT Subsystem metadata, found `{kind}`")
            }
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT Subsystem descriptor {}: {source}",
                path.display()
            ),
            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT Subsystem XML: {message}")
            }
            Self::MissingRoot => formatter.write_str("EDT Subsystem XML root is missing"),
            Self::UnexpectedRoot(root) => {
                write!(formatter, "unexpected EDT Subsystem XML root `{root}`")
            }
            Self::UnsupportedNamespace(Some(namespace)) => write!(
                formatter,
                "unsupported EDT Subsystem XML namespace `{namespace}`"
            ),
            Self::UnsupportedNamespace(None) => {
                formatter.write_str("EDT Subsystem XML namespace is missing")
            }
        }
    }
}

impl std::error::Error for EdtSubsystemContentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            Self::UnexpectedMetadataKind(_)
            | Self::MalformedXml(_)
            | Self::MissingRoot
            | Self::UnexpectedRoot(_)
            | Self::UnsupportedNamespace(_) => None,
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

    use super::{
        EdtSubsystemContentError, EdtSubsystemContentReader, FileSystemEdtSubsystemContentReader,
        parse_subsystem_content,
    };
    use crate::{
        EdtMetadataObjectDescriptor, EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader,
    };

    const SUBSYSTEM_PREFIX: &str = r#"<mdclass:Subsystem xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="b72ed007-5756-4a1d-b27d-e74aef13083f">"#;

    fn fixture_subsystem(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/subsystem_content")
            .join(name)
    }

    fn id() -> EntityId {
        EntityId::new("b72ed007-5756-4a1d-b27d-e74aef13083f")
            .expect("fixture identifier must be valid")
    }

    fn parse(xml: &str) -> Result<super::EdtSubsystemContentDescriptor, EdtSubsystemContentError> {
        parse_subsystem_content(xml, id(), PathBuf::from("TestObject.mdo"))
    }

    fn descriptor(kind: MetadataKind, path: PathBuf) -> EdtMetadataObjectDescriptor {
        EdtMetadataObjectDescriptor::new(
            id(),
            EntityName::new("TestObject").expect("fixture name must be valid"),
            None,
            kind,
            None,
            path,
        )
    }

    #[test]
    fn subsystem_content_reader_reads_direct_content_from_edt_fixture() {
        let object_directory = fixture_subsystem("TestObject");
        let metadata = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Subsystem)
            .expect("fixture metadata descriptor must parse");

        let content = FileSystemEdtSubsystemContentReader
            .read(&metadata)
            .expect("real-format Subsystem content fixture must parse");

        assert_eq!(
            content.subsystem_id().as_str(),
            "b72ed007-5756-4a1d-b27d-e74aef13083f"
        );
        assert!(
            content
                .descriptor_path()
                .ends_with("TestObject/TestObject.mdo")
        );
        assert_eq!(
            content.raw_content(),
            [
                "Document.EmptyDocument",
                "Document.TestDocument",
                "Role.Document_EmptyDocument_Posting",
            ]
        );
    }

    #[test]
    fn subsystem_content_parser_ignores_nested_and_missing_content() {
        let xml = format!(
            r"{SUBSYSTEM_PREFIX}
                <name>TestObject</name>
                <synonym><content>Nested synonym</content></synonym>
                <properties><content>Document.Descendant</content></properties>
            </mdclass:Subsystem>"
        );

        let content = parse(&xml).expect("missing direct content must be valid");

        assert!(content.raw_content().is_empty());
    }

    #[test]
    fn subsystem_content_parser_sorts_and_deduplicates_without_normalizing() {
        let first = format!(
            r"{SUBSYSTEM_PREFIX}
                <content>Role.Second</content>
                <content> Document.First </content>
                <content>Role.Second</content>
                <content>Document.A&amp;B</content>
            </mdclass:Subsystem>"
        );
        let second = format!(
            r"{SUBSYSTEM_PREFIX}
                <content>Document.A&amp;B</content>
                <content>Role.Second</content>
                <content> Document.First </content>
            </mdclass:Subsystem>"
        );

        let first = parse(&first).expect("first source order must parse");
        let second = parse(&second).expect("second source order must parse");

        assert_eq!(first.raw_content(), second.raw_content());
        assert_eq!(
            first.raw_content(),
            [" Document.First ", "Document.A&B", "Role.Second"]
        );
    }

    #[test]
    fn subsystem_content_parser_preserves_empty_direct_content() {
        let xml = format!(
            r"{SUBSYSTEM_PREFIX}
                <content></content>
                <content/>
            </mdclass:Subsystem>"
        );

        let content = parse(&xml).expect("empty direct content must be preserved");

        assert_eq!(content.raw_content(), [""]);
    }

    #[test]
    fn subsystem_content_parser_reports_malformed_xml() {
        let xml = format!("{SUBSYSTEM_PREFIX}<content>Document.Broken</content>");

        assert!(matches!(
            parse(&xml),
            Err(EdtSubsystemContentError::MalformedXml(_))
        ));
    }

    #[test]
    fn subsystem_content_reader_rejects_non_subsystem_descriptor() {
        let metadata = descriptor(MetadataKind::Document, PathBuf::from("Document.mdo"));

        assert!(matches!(
            FileSystemEdtSubsystemContentReader.read(&metadata),
            Err(EdtSubsystemContentError::UnexpectedMetadataKind(
                MetadataKind::Document
            ))
        ));
    }

    #[test]
    fn subsystem_content_reader_rejects_non_subsystem_xml_root() {
        let root = tempdir().expect("temporary directory must be created");
        let path = root.path().join("TestObject.mdo");
        fs::write(
            &path,
            r#"<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"/>"#,
        )
        .expect("descriptor must be written");
        let metadata = descriptor(MetadataKind::Subsystem, path);

        assert!(matches!(
            FileSystemEdtSubsystemContentReader.read(&metadata),
            Err(EdtSubsystemContentError::UnexpectedRoot(root))
                if root == "mdclass:Document"
        ));
    }
}
