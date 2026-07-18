//! Reader for top-level EDT metadata object descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::events::Event;
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
    descriptor_path: PathBuf,
}

impl EdtMetadataObjectDescriptor {
    /// Creates a parsed EDT metadata object descriptor.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        synonym: Option<String>,
        kind: MetadataKind,
        descriptor_path: PathBuf,
    ) -> Self {
        Self {
            id,
            name,
            synonym,
            kind,
            descriptor_path,
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

    /// Returns the source descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
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
    let mut path = Vec::<String>::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                path.push(local_name(event.name().as_ref()));

                if uuid.is_none() {
                    uuid = read_uuid(&reader, &event)?;
                }
            }
            Ok(Event::Empty(event)) => {
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
                    Some("name") if name.is_none() => name = Some(text),
                    Some("content") if is_synonym_content_path(&path) => {
                        synonym.get_or_insert(text);
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

    Ok(EdtMetadataObjectDescriptor::new(
        id,
        name,
        synonym,
        kind,
        descriptor_path,
    ))
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

fn is_synonym_content_path(path: &[String]) -> bool {
    path.len() >= 2 && path[path.len() - 2] == "synonym" && path[path.len() - 1] == "content"
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
    use tempfile::tempdir;

    use super::{EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader};

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
