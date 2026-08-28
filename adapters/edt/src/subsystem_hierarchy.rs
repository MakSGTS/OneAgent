//! Deterministic source model for recursive EDT Subsystem hierarchy declarations.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    EdtMetadataObjectDescriptor, EdtMetadataObjectError, EdtMetadataObjectReader,
    FileSystemEdtMetadataObjectReader,
};

const SUBSYSTEM_ROOT: &str = "mdclass:Subsystem";
const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";

/// One recursively discovered EDT Subsystem descriptor and its hierarchy source fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtSubsystemHierarchyDescriptor {
    metadata: EdtMetadataObjectDescriptor,
    hierarchy_path: Vec<EntityName>,
    parent_id: Option<EntityId>,
    raw_child_declarations: Vec<String>,
    raw_parent_declaration: Option<String>,
}

impl EdtSubsystemHierarchyDescriptor {
    /// Returns the existing metadata-object descriptor used by downstream readers.
    #[must_use]
    pub const fn metadata(&self) -> &EdtMetadataObjectDescriptor {
        &self.metadata
    }

    /// Returns the complete top-to-local Subsystem name path.
    #[must_use]
    pub fn hierarchy_path(&self) -> &[EntityName] {
        &self.hierarchy_path
    }

    /// Returns the immediate physical parent's metadata UUID, if this is nested.
    #[must_use]
    pub const fn parent_id(&self) -> Option<&EntityId> {
        self.parent_id.as_ref()
    }

    /// Returns direct raw `<subsystems>` declarations in deterministic order.
    #[must_use]
    pub fn raw_child_declarations(&self) -> &[String] {
        &self.raw_child_declarations
    }

    /// Returns the direct raw `<parentSubsystem>` declaration, if this is nested.
    #[must_use]
    pub fn raw_parent_declaration(&self) -> Option<&str> {
        self.raw_parent_declaration.as_deref()
    }

    /// Returns the descriptor hierarchy depth, where top-level Subsystems have depth one.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.hierarchy_path.len()
    }
}

/// One direct hierarchy observation corroborated by both XML projections and the path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdtSubsystemHierarchyRelation {
    parent_id: EntityId,
    child_id: EntityId,
    raw_child_declaration: String,
    raw_parent_declaration: String,
}

impl EdtSubsystemHierarchyRelation {
    /// Returns the declaring parent's metadata UUID.
    #[must_use]
    pub const fn parent_id(&self) -> &EntityId {
        &self.parent_id
    }

    /// Returns the nested child's metadata UUID.
    #[must_use]
    pub const fn child_id(&self) -> &EntityId {
        &self.child_id
    }

    /// Returns the exact parent-side `<subsystems>` value.
    #[must_use]
    pub fn raw_child_declaration(&self) -> &str {
        &self.raw_child_declaration
    }

    /// Returns the exact child-side `<parentSubsystem>` value.
    #[must_use]
    pub fn raw_parent_declaration(&self) -> &str {
        &self.raw_parent_declaration
    }
}

/// Complete deterministic Subsystem hierarchy source model for one EDT project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtSubsystemHierarchy {
    descriptors: Vec<EdtSubsystemHierarchyDescriptor>,
    relations: Vec<EdtSubsystemHierarchyRelation>,
}

impl EdtSubsystemHierarchy {
    /// Returns descriptors ordered by their complete case-sensitive hierarchy path.
    #[must_use]
    pub fn descriptors(&self) -> &[EdtSubsystemHierarchyDescriptor] {
        &self.descriptors
    }

    /// Returns direct relations ordered by stable parent and child UUIDs.
    #[must_use]
    pub fn relations(&self) -> &[EdtSubsystemHierarchyRelation] {
        &self.relations
    }
}

/// Reads recursive EDT Subsystem hierarchy source facts without graph emission.
pub trait EdtSubsystemHierarchyReader {
    /// Reads the complete hierarchy rooted at `project_root/src/Subsystems`.
    ///
    /// # Errors
    ///
    /// Returns a fatal typed error when filesystem, descriptor, declaration,
    /// path, identity, or hierarchy agreement is invalid. No partial model is
    /// returned.
    fn read(
        &self,
        project_root: &Path,
    ) -> Result<EdtSubsystemHierarchy, EdtSubsystemHierarchyError>;
}

/// Filesystem implementation of [`EdtSubsystemHierarchyReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtSubsystemHierarchyReader;

impl EdtSubsystemHierarchyReader for FileSystemEdtSubsystemHierarchyReader {
    fn read(
        &self,
        project_root: &Path,
    ) -> Result<EdtSubsystemHierarchy, EdtSubsystemHierarchyError> {
        let canonical_project_root = canonical_project_root(project_root)?;
        let subsystems_root = project_root.join("src/Subsystems");
        if !subsystems_root.is_dir() {
            return Err(EdtSubsystemHierarchyError::SubsystemsDirectoryNotFound(
                subsystems_root,
            ));
        }
        let subsystems_root = canonical_within_project(&subsystems_root, &canonical_project_root)?;

        let mut collector = HierarchyCollector {
            project_root: canonical_project_root,
            visited_directories: BTreeMap::new(),
            pending: Vec::new(),
        };
        for directory in direct_child_directories(&subsystems_root)? {
            collector.collect(&directory, Vec::new(), None, None)?;
        }
        collector.finish()
    }
}

#[derive(Debug)]
struct PendingDescriptor {
    metadata: EdtMetadataObjectDescriptor,
    hierarchy_path: Vec<EntityName>,
    physical_parent_id: Option<EntityId>,
    raw_child_from_parent: Option<String>,
    raw_child_declarations: Vec<String>,
    raw_parent_declarations: Vec<String>,
    parsed_parent_path: Option<Vec<EntityName>>,
}

struct HierarchyCollector {
    project_root: PathBuf,
    visited_directories: BTreeMap<PathBuf, Vec<EntityName>>,
    pending: Vec<PendingDescriptor>,
}

impl HierarchyCollector {
    fn collect(
        &mut self,
        directory: &Path,
        ancestors: Vec<EntityName>,
        physical_parent_id: Option<EntityId>,
        raw_child_from_parent: Option<String>,
    ) -> Result<(), EdtSubsystemHierarchyError> {
        let directory = canonical_within_project(directory, &self.project_root)?;
        if let Some(first_path) = self.visited_directories.get(&directory) {
            return Err(EdtSubsystemHierarchyError::DirectoryCycleOrAlias {
                directory,
                first_path: qualified_path(first_path),
            });
        }

        let directory_name = directory_name(&directory)?;
        let metadata = FileSystemEdtMetadataObjectReader
            .read(&directory, MetadataKind::Subsystem)
            .map_err(|source| EdtSubsystemHierarchyError::Descriptor {
                directory: directory.clone(),
                source,
            })?;
        if metadata.name().as_str() != directory_name.as_str() {
            return Err(EdtSubsystemHierarchyError::DescriptorPathNameMismatch {
                directory,
                directory_name: directory_name.as_str().to_owned(),
                descriptor_name: metadata.name().as_str().to_owned(),
            });
        }

        let mut hierarchy_path = ancestors;
        hierarchy_path.push(metadata.name().clone());
        self.visited_directories
            .insert(directory.clone(), hierarchy_path.clone());

        let fields = read_hierarchy_fields(metadata.descriptor_path())?;
        let raw_child_declarations =
            validate_child_declarations(metadata.descriptor_path(), fields.raw_child_declarations)?;
        let parsed_parent_path = validate_parent_declarations(
            metadata.descriptor_path(),
            &hierarchy_path,
            &fields.raw_parent_declarations,
        )?;
        let children = reconcile_children(
            &directory,
            metadata.descriptor_path(),
            &raw_child_declarations,
        )?;

        let metadata_id = metadata.id().clone();
        self.pending.push(PendingDescriptor {
            metadata,
            hierarchy_path: hierarchy_path.clone(),
            physical_parent_id,
            raw_child_from_parent,
            raw_child_declarations,
            raw_parent_declarations: fields.raw_parent_declarations,
            parsed_parent_path,
        });

        for (raw_name, child_directory) in children {
            self.collect(
                &child_directory,
                hierarchy_path.clone(),
                Some(metadata_id.clone()),
                Some(raw_name),
            )?;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<EdtSubsystemHierarchy, EdtSubsystemHierarchyError> {
        self.validate_unique_ids()?;
        self.validate_self_parents()?;
        self.validate_parent_cycles()?;
        self.validate_parent_agreement()?;

        self.pending
            .sort_by(|left, right| left.hierarchy_path.cmp(&right.hierarchy_path));
        let mut descriptors = Vec::with_capacity(self.pending.len());
        let mut relations = Vec::new();

        for pending in self.pending {
            let raw_parent_declaration = pending.raw_parent_declarations.into_iter().next();
            if let (Some(parent_id), Some(raw_child), Some(raw_parent)) = (
                pending.physical_parent_id.clone(),
                pending.raw_child_from_parent,
                raw_parent_declaration.clone(),
            ) {
                relations.push(EdtSubsystemHierarchyRelation {
                    parent_id,
                    child_id: pending.metadata.id().clone(),
                    raw_child_declaration: raw_child,
                    raw_parent_declaration: raw_parent,
                });
            }
            descriptors.push(EdtSubsystemHierarchyDescriptor {
                metadata: pending.metadata,
                hierarchy_path: pending.hierarchy_path,
                parent_id: pending.physical_parent_id,
                raw_child_declarations: pending.raw_child_declarations,
                raw_parent_declaration,
            });
        }
        relations.sort();
        Ok(EdtSubsystemHierarchy {
            descriptors,
            relations,
        })
    }

    fn validate_unique_ids(&self) -> Result<(), EdtSubsystemHierarchyError> {
        let mut identifiers = BTreeMap::<EntityId, &Path>::new();
        for descriptor in &self.pending {
            if let Some(first_path) = identifiers.insert(
                descriptor.metadata.id().clone(),
                descriptor.metadata.descriptor_path(),
            ) {
                return Err(EdtSubsystemHierarchyError::DuplicateIdentifier {
                    identifier: descriptor.metadata.id().clone(),
                    first_path: first_path.to_path_buf(),
                    duplicate_path: descriptor.metadata.descriptor_path().to_path_buf(),
                });
            }
        }
        Ok(())
    }

    fn validate_parent_cycles(&self) -> Result<(), EdtSubsystemHierarchyError> {
        let path_to_id = self
            .pending
            .iter()
            .map(|descriptor| {
                (
                    path_key(&descriptor.hierarchy_path),
                    descriptor.metadata.id().clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let parents = self
            .pending
            .iter()
            .filter_map(|descriptor| {
                descriptor
                    .parsed_parent_path
                    .as_ref()
                    .and_then(|parent_path| {
                        path_to_id
                            .get(&path_key(parent_path))
                            .map(|parent_id| (descriptor.metadata.id().clone(), parent_id.clone()))
                    })
            })
            .collect::<BTreeMap<_, _>>();

        for start in parents.keys() {
            let mut current = start;
            let mut chain = Vec::new();
            let mut positions = BTreeMap::new();
            while let Some(parent) = parents.get(current) {
                if let Some(position) = positions.insert(current.clone(), chain.len()) {
                    let mut cycle = chain[position..].to_vec();
                    cycle.sort();
                    cycle.dedup();
                    return Err(EdtSubsystemHierarchyError::ParentCycle(cycle));
                }
                chain.push(current.clone());
                current = parent;
            }
        }
        Ok(())
    }

    fn validate_self_parents(&self) -> Result<(), EdtSubsystemHierarchyError> {
        for descriptor in &self.pending {
            if descriptor.parsed_parent_path.as_deref()
                == Some(descriptor.hierarchy_path.as_slice())
            {
                return Err(EdtSubsystemHierarchyError::SelfParent {
                    descriptor: descriptor.metadata.descriptor_path().to_path_buf(),
                });
            }
        }
        Ok(())
    }

    fn validate_parent_agreement(&self) -> Result<(), EdtSubsystemHierarchyError> {
        for descriptor in &self.pending {
            let Some(parsed_parent) = &descriptor.parsed_parent_path else {
                continue;
            };
            let expected = &descriptor.hierarchy_path[..descriptor.hierarchy_path.len() - 1];
            if parsed_parent.as_slice() != expected {
                return Err(EdtSubsystemHierarchyError::ParentPathMismatch {
                    descriptor: descriptor.metadata.descriptor_path().to_path_buf(),
                    expected: qualified_path(expected),
                    actual: qualified_path(parsed_parent),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct RawHierarchyFields {
    raw_child_declarations: Vec<String>,
    raw_parent_declarations: Vec<String>,
}

fn read_hierarchy_fields(path: &Path) -> Result<RawHierarchyFields, EdtSubsystemHierarchyError> {
    let xml = fs::read_to_string(path).map_err(|source| EdtSubsystemHierarchyError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    parse_hierarchy_fields(&xml, path)
}

fn parse_hierarchy_fields(
    xml: &str,
    descriptor_path: &Path,
) -> Result<RawHierarchyFields, EdtSubsystemHierarchyError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut fields = RawHierarchyFields::default();
    let mut path = Vec::<String>::new();
    let mut root_seen = false;
    let mut root_closed = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(malformed(descriptor_path, "multiple root elements"));
                    }
                    validate_root(&reader, &event, descriptor_path)?;
                    root_seen = true;
                }
                let element = qualified_name(event.name().as_ref());
                if path.len() == 1 && matches!(element.as_str(), "subsystems" | "parentSubsystem") {
                    let value = reader
                        .read_text(event.to_end().name())
                        .map_err(|source| malformed(descriptor_path, source.to_string()))?;
                    let value = unescape(&value)
                        .map_err(|source| malformed(descriptor_path, source.to_string()))?
                        .into_owned();
                    match element.as_str() {
                        "subsystems" => fields.raw_child_declarations.push(value),
                        "parentSubsystem" => fields.raw_parent_declarations.push(value),
                        _ => unreachable!(),
                    }
                    continue;
                }
                path.push(element);
            }
            Ok(Event::Empty(event)) => {
                if path.is_empty() {
                    if root_seen {
                        return Err(malformed(descriptor_path, "multiple root elements"));
                    }
                    validate_root(&reader, &event, descriptor_path)?;
                    root_seen = true;
                    root_closed = true;
                } else if path.len() == 1 {
                    match event.name().as_ref() {
                        b"subsystems" => fields.raw_child_declarations.push(String::new()),
                        b"parentSubsystem" => fields.raw_parent_declarations.push(String::new()),
                        _ => {}
                    }
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
            Err(source) => return Err(malformed(descriptor_path, source.to_string())),
        }
    }

    if !root_seen {
        return Err(EdtSubsystemHierarchyError::MissingRoot(
            descriptor_path.to_path_buf(),
        ));
    }
    if !root_closed {
        return Err(malformed(
            descriptor_path,
            "unexpected end of file before the Subsystem root was closed",
        ));
    }
    Ok(fields)
}

fn validate_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    descriptor_path: &Path,
) -> Result<(), EdtSubsystemHierarchyError> {
    let root = qualified_name(event.name().as_ref());
    if root != SUBSYSTEM_ROOT {
        return Err(EdtSubsystemHierarchyError::UnexpectedRoot {
            path: descriptor_path.to_path_buf(),
            root,
        });
    }
    let mut namespace = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute =
            attribute.map_err(|source| malformed(descriptor_path, source.to_string()))?;
        if attribute.key.as_ref() == b"xmlns:mdclass" {
            namespace = Some(
                attribute
                    .decode_and_unescape_value(reader.decoder())
                    .map_err(|source| malformed(descriptor_path, source.to_string()))?
                    .into_owned(),
            );
        }
    }
    if namespace.as_deref() != Some(METADATA_NAMESPACE) {
        return Err(EdtSubsystemHierarchyError::UnsupportedNamespace {
            path: descriptor_path.to_path_buf(),
            namespace,
        });
    }
    Ok(())
}

fn validate_child_declarations(
    descriptor: &Path,
    declarations: Vec<String>,
) -> Result<Vec<String>, EdtSubsystemHierarchyError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for declaration in declarations {
        validate_exact_name(&declaration).map_err(|()| {
            EdtSubsystemHierarchyError::InvalidChildDeclaration {
                descriptor: descriptor.to_path_buf(),
                value: declaration.clone(),
            }
        })?;
        *counts.entry(declaration).or_default() += 1;
    }
    if let Some((name, count)) = counts.iter().find(|(_, count)| **count > 1) {
        return Err(EdtSubsystemHierarchyError::DuplicateChildDeclaration {
            descriptor: descriptor.to_path_buf(),
            name: name.clone(),
            count: *count,
        });
    }
    Ok(counts.into_keys().collect())
}

fn validate_parent_declarations(
    descriptor: &Path,
    hierarchy_path: &[EntityName],
    declarations: &[String],
) -> Result<Option<Vec<EntityName>>, EdtSubsystemHierarchyError> {
    if hierarchy_path.len() == 1 {
        return match declarations {
            [] => Ok(None),
            _ => Err(EdtSubsystemHierarchyError::UnexpectedTopLevelParent {
                descriptor: descriptor.to_path_buf(),
                declarations: declarations.to_vec(),
            }),
        };
    }
    match declarations {
        [] => Err(EdtSubsystemHierarchyError::MissingParentDeclaration(
            descriptor.to_path_buf(),
        )),
        [declaration] => parse_qualified_parent(descriptor, declaration).map(Some),
        _ => Err(EdtSubsystemHierarchyError::MultipleParentDeclarations {
            descriptor: descriptor.to_path_buf(),
            declarations: declarations.to_vec(),
        }),
    }
}

fn parse_qualified_parent(
    descriptor: &Path,
    value: &str,
) -> Result<Vec<EntityName>, EdtSubsystemHierarchyError> {
    let components = value.split('.').collect::<Vec<_>>();
    if value.trim() != value || components.len() < 2 || components.len() % 2 != 0 {
        return Err(malformed_parent(descriptor, value));
    }
    let mut names = Vec::new();
    for pair in components.chunks_exact(2) {
        if pair[0] != "Subsystem" || validate_exact_name(pair[1]).is_err() {
            return Err(malformed_parent(descriptor, value));
        }
        names.push(EntityName::new(pair[1]).map_err(|_| malformed_parent(descriptor, value))?);
    }
    Ok(names)
}

fn reconcile_children(
    directory: &Path,
    descriptor: &Path,
    declarations: &[String],
) -> Result<Vec<(String, PathBuf)>, EdtSubsystemHierarchyError> {
    let children_root = directory.join("Subsystems");
    let directories = if children_root.exists() {
        if !children_root.is_dir() {
            return Err(EdtSubsystemHierarchyError::ChildContainerNotDirectory(
                children_root,
            ));
        }
        direct_child_directories(&children_root)?
    } else {
        Vec::new()
    };
    let physical = directories
        .into_iter()
        .map(|path| Ok((directory_name(&path)?.as_str().to_owned(), path)))
        .collect::<Result<BTreeMap<_, _>, EdtSubsystemHierarchyError>>()?;
    let declared = declarations.iter().cloned().collect::<BTreeSet<_>>();
    let physical_names = physical.keys().cloned().collect::<BTreeSet<_>>();

    if let Some(name) = declared.difference(&physical_names).next() {
        return Err(EdtSubsystemHierarchyError::MissingChildDirectory {
            descriptor: descriptor.to_path_buf(),
            name: name.clone(),
        });
    }
    if let Some(name) = physical_names.difference(&declared).next() {
        return Err(EdtSubsystemHierarchyError::UndeclaredChildDirectory {
            descriptor: descriptor.to_path_buf(),
            name: name.clone(),
        });
    }
    Ok(declarations
        .iter()
        .map(|name| (name.clone(), physical[name].clone()))
        .collect())
}

fn canonical_project_root(project_root: &Path) -> Result<PathBuf, EdtSubsystemHierarchyError> {
    fs::canonicalize(project_root).map_err(|source| {
        EdtSubsystemHierarchyError::CanonicalizeProjectRoot {
            path: project_root.to_path_buf(),
            source,
        }
    })
}

fn canonical_within_project(
    path: &Path,
    project_root: &Path,
) -> Result<PathBuf, EdtSubsystemHierarchyError> {
    let canonical =
        fs::canonicalize(path).map_err(|source| EdtSubsystemHierarchyError::CanonicalizePath {
            path: path.to_path_buf(),
            source,
        })?;
    if !canonical.starts_with(project_root) {
        return Err(EdtSubsystemHierarchyError::ProjectRootEscape {
            path: path.to_path_buf(),
            canonical,
            project_root: project_root.to_path_buf(),
        });
    }
    Ok(canonical)
}

fn direct_child_directories(path: &Path) -> Result<Vec<PathBuf>, EdtSubsystemHierarchyError> {
    let entries =
        fs::read_dir(path).map_err(|source| EdtSubsystemHierarchyError::ReadDirectory {
            path: path.to_path_buf(),
            source,
        })?;
    let mut directories = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| EdtSubsystemHierarchyError::ReadDirectoryEntry {
            path: path.to_path_buf(),
            source,
        })?;
        let file_type =
            entry
                .file_type()
                .map_err(|source| EdtSubsystemHierarchyError::ReadDirectoryEntry {
                    path: entry.path(),
                    source,
                })?;
        if file_type.is_dir() || file_type.is_symlink() {
            directories.push(entry.path());
        }
    }
    directories.sort();
    Ok(directories)
}

fn directory_name(path: &Path) -> Result<EntityName, EdtSubsystemHierarchyError> {
    let value = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| EdtSubsystemHierarchyError::InvalidDirectoryName(path.to_path_buf()))?;
    validate_exact_name(value)
        .and_then(|()| EntityName::new(value).map_err(|_| ()))
        .map_err(|()| EdtSubsystemHierarchyError::InvalidDirectoryName(path.to_path_buf()))
}

fn validate_exact_name(value: &str) -> Result<(), ()> {
    if value.trim() != value || value.is_empty() || value.contains('.') {
        Err(())
    } else {
        EntityName::new(value).map(|_| ()).map_err(|_| ())
    }
}

fn path_key(path: &[EntityName]) -> Vec<String> {
    path.iter().map(|name| name.as_str().to_owned()).collect()
}

fn qualified_path(path: &[EntityName]) -> String {
    path.iter()
        .map(|name| format!("Subsystem.{}", name.as_str()))
        .collect::<Vec<_>>()
        .join(".")
}

fn qualified_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

fn malformed(path: &Path, message: impl Into<String>) -> EdtSubsystemHierarchyError {
    EdtSubsystemHierarchyError::MalformedXml {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

fn malformed_parent(path: &Path, value: &str) -> EdtSubsystemHierarchyError {
    EdtSubsystemHierarchyError::MalformedParentDeclaration {
        descriptor: path.to_path_buf(),
        value: value.to_owned(),
    }
}

/// Fatal errors produced while reading an EDT Subsystem hierarchy.
#[derive(Debug)]
pub enum EdtSubsystemHierarchyError {
    /// The project root could not be canonicalized.
    CanonicalizeProjectRoot {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The expected `src/Subsystems` directory is absent.
    SubsystemsDirectoryNotFound(PathBuf),
    /// A discovered path could not be canonicalized.
    CanonicalizePath {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A discovered path resolves outside the canonical project root.
    ProjectRootEscape {
        path: PathBuf,
        canonical: PathBuf,
        project_root: PathBuf,
    },
    /// A directory could not be enumerated.
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    /// One directory entry could not be read.
    ReadDirectoryEntry {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A directory name is missing, non-Unicode, empty, padded, or qualified.
    InvalidDirectoryName(PathBuf),
    /// A canonical directory was encountered through a cycle or alias.
    DirectoryCycleOrAlias {
        directory: PathBuf,
        first_path: String,
    },
    /// The existing metadata-object reader rejected a descriptor.
    Descriptor {
        directory: PathBuf,
        source: EdtMetadataObjectError,
    },
    /// A descriptor name differs from its immediate directory name.
    DescriptorPathNameMismatch {
        directory: PathBuf,
        directory_name: String,
        descriptor_name: String,
    },
    /// The hierarchy descriptor could not be read.
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The hierarchy XML is malformed.
    MalformedXml { path: PathBuf, message: String },
    /// The hierarchy XML root is absent.
    MissingRoot(PathBuf),
    /// The hierarchy XML root is not `mdclass:Subsystem`.
    UnexpectedRoot { path: PathBuf, root: String },
    /// The exact EDT metadata namespace is missing or unsupported.
    UnsupportedNamespace {
        path: PathBuf,
        namespace: Option<String>,
    },
    /// A direct child declaration is not an exact local name.
    InvalidChildDeclaration { descriptor: PathBuf, value: String },
    /// A direct child name is declared more than once.
    DuplicateChildDeclaration {
        descriptor: PathBuf,
        name: String,
        count: usize,
    },
    /// A top-level descriptor unexpectedly declares a parent.
    UnexpectedTopLevelParent {
        descriptor: PathBuf,
        declarations: Vec<String>,
    },
    /// A nested descriptor has no direct parent declaration.
    MissingParentDeclaration(PathBuf),
    /// A nested descriptor has multiple direct parent declarations.
    MultipleParentDeclarations {
        descriptor: PathBuf,
        declarations: Vec<String>,
    },
    /// A qualified parent declaration violates the accepted grammar.
    MalformedParentDeclaration { descriptor: PathBuf, value: String },
    /// The immediate `Subsystems` child container is not a directory.
    ChildContainerNotDirectory(PathBuf),
    /// A declared child has no matching immediate directory.
    MissingChildDirectory { descriptor: PathBuf, name: String },
    /// An immediate child directory has no matching declaration.
    UndeclaredChildDirectory { descriptor: PathBuf, name: String },
    /// Two descriptors use the same UUID.
    DuplicateIdentifier {
        identifier: EntityId,
        first_path: PathBuf,
        duplicate_path: PathBuf,
    },
    /// A descriptor declares itself as its own parent.
    SelfParent { descriptor: PathBuf },
    /// Parent declarations form a directed cycle.
    ParentCycle(Vec<EntityId>),
    /// The qualified parent differs from the immediate physical ancestor path.
    ParentPathMismatch {
        descriptor: PathBuf,
        expected: String,
        actual: String,
    },
}

impl Display for EdtSubsystemHierarchyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CanonicalizeProjectRoot { .. }
            | Self::SubsystemsDirectoryNotFound(_)
            | Self::CanonicalizePath { .. }
            | Self::ProjectRootEscape { .. }
            | Self::ReadDirectory { .. }
            | Self::ReadDirectoryEntry { .. }
            | Self::InvalidDirectoryName(_)
            | Self::DirectoryCycleOrAlias { .. }
            | Self::Descriptor { .. }
            | Self::DescriptorPathNameMismatch { .. } => format_filesystem_error(self, formatter),
            Self::ReadFile { .. }
            | Self::MalformedXml { .. }
            | Self::MissingRoot(_)
            | Self::UnexpectedRoot { .. }
            | Self::UnsupportedNamespace { .. } => format_xml_error(self, formatter),
            _ => format_hierarchy_error(self, formatter),
        }
    }
}

fn format_filesystem_error(
    error: &EdtSubsystemHierarchyError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtSubsystemHierarchyError::CanonicalizeProjectRoot { path, source } => write!(
            formatter,
            "failed to canonicalize EDT project root {}: {source}",
            path.display()
        ),
        EdtSubsystemHierarchyError::SubsystemsDirectoryNotFound(path) => write!(
            formatter,
            "EDT Subsystems directory was not found: {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::CanonicalizePath { path, source } => write!(
            formatter,
            "failed to canonicalize EDT Subsystem path {}: {source}",
            path.display()
        ),
        EdtSubsystemHierarchyError::ProjectRootEscape {
            path,
            canonical,
            project_root,
        } => write!(
            formatter,
            "EDT Subsystem path {} resolves to {} outside project root {}",
            path.display(),
            canonical.display(),
            project_root.display()
        ),
        EdtSubsystemHierarchyError::ReadDirectory { path, source } => write!(
            formatter,
            "failed to read EDT Subsystem directory {}: {source}",
            path.display()
        ),
        EdtSubsystemHierarchyError::ReadDirectoryEntry { path, source } => write!(
            formatter,
            "failed to read an entry in EDT Subsystem directory {}: {source}",
            path.display()
        ),
        EdtSubsystemHierarchyError::InvalidDirectoryName(path) => write!(
            formatter,
            "invalid EDT Subsystem directory name: {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::DirectoryCycleOrAlias {
            directory,
            first_path,
        } => write!(
            formatter,
            "EDT Subsystem directory {} repeats hierarchy path `{first_path}`",
            directory.display()
        ),
        EdtSubsystemHierarchyError::Descriptor { directory, source } => write!(
            formatter,
            "invalid EDT Subsystem descriptor in {}: {source}",
            directory.display()
        ),
        EdtSubsystemHierarchyError::DescriptorPathNameMismatch {
            directory,
            directory_name,
            descriptor_name,
        } => write!(
            formatter,
            "EDT Subsystem descriptor name `{descriptor_name}` does not match directory `{directory_name}` in {}",
            directory.display()
        ),
        _ => unreachable!("filesystem error formatter received another error category"),
    }
}

fn format_xml_error(
    error: &EdtSubsystemHierarchyError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtSubsystemHierarchyError::ReadFile { path, source } => write!(
            formatter,
            "failed to read EDT Subsystem hierarchy descriptor {}: {source}",
            path.display()
        ),
        EdtSubsystemHierarchyError::MalformedXml { path, message } => write!(
            formatter,
            "malformed EDT Subsystem hierarchy XML {}: {message}",
            path.display()
        ),
        EdtSubsystemHierarchyError::MissingRoot(path) => write!(
            formatter,
            "EDT Subsystem hierarchy XML root is missing in {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::UnexpectedRoot { path, root } => write!(
            formatter,
            "unexpected EDT Subsystem hierarchy XML root `{root}` in {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::UnsupportedNamespace {
            path,
            namespace: Some(namespace),
        } => write!(
            formatter,
            "unsupported EDT Subsystem hierarchy namespace `{namespace}` in {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::UnsupportedNamespace {
            path,
            namespace: None,
        } => write!(
            formatter,
            "EDT Subsystem hierarchy namespace is missing in {}",
            path.display()
        ),
        _ => unreachable!("XML error formatter received another error category"),
    }
}

fn format_hierarchy_error(
    error: &EdtSubsystemHierarchyError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtSubsystemHierarchyError::InvalidChildDeclaration { descriptor, value } => write!(
            formatter,
            "invalid direct Subsystem child declaration `{value}` in {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::DuplicateChildDeclaration {
            descriptor,
            name,
            count,
        } => write!(
            formatter,
            "duplicate direct Subsystem child declaration `{name}` ({count} occurrences) in {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::UnexpectedTopLevelParent {
            descriptor,
            declarations,
        } => write!(
            formatter,
            "top-level EDT Subsystem {} declares {} parent values",
            descriptor.display(),
            declarations.len()
        ),
        EdtSubsystemHierarchyError::MissingParentDeclaration(path) => write!(
            formatter,
            "nested EDT Subsystem parent declaration is missing in {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::MultipleParentDeclarations {
            descriptor,
            declarations,
        } => write!(
            formatter,
            "nested EDT Subsystem {} declares {} parent values",
            descriptor.display(),
            declarations.len()
        ),
        EdtSubsystemHierarchyError::MalformedParentDeclaration { descriptor, value } => write!(
            formatter,
            "malformed EDT Subsystem parent declaration `{value}` in {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::ChildContainerNotDirectory(path) => write!(
            formatter,
            "EDT Subsystem child container is not a directory: {}",
            path.display()
        ),
        EdtSubsystemHierarchyError::MissingChildDirectory { descriptor, name } => write!(
            formatter,
            "declared EDT Subsystem child `{name}` has no directory for {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::UndeclaredChildDirectory { descriptor, name } => write!(
            formatter,
            "EDT Subsystem child directory `{name}` is not declared by {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::DuplicateIdentifier {
            identifier,
            first_path,
            duplicate_path,
        } => write!(
            formatter,
            "duplicate EDT Subsystem UUID `{identifier}` in {} and {}",
            first_path.display(),
            duplicate_path.display()
        ),
        EdtSubsystemHierarchyError::SelfParent { descriptor } => write!(
            formatter,
            "EDT Subsystem declares itself as parent in {}",
            descriptor.display()
        ),
        EdtSubsystemHierarchyError::ParentCycle(ids) => write!(
            formatter,
            "EDT Subsystem parent declarations contain a cycle across {} identifiers",
            ids.len()
        ),
        EdtSubsystemHierarchyError::ParentPathMismatch {
            descriptor,
            expected,
            actual,
        } => write!(
            formatter,
            "EDT Subsystem parent `{actual}` does not match physical parent `{expected}` in {}",
            descriptor.display()
        ),
        _ => unreachable!("hierarchy error formatter received another error category"),
    }
}

impl std::error::Error for EdtSubsystemHierarchyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CanonicalizeProjectRoot { source, .. }
            | Self::CanonicalizePath { source, .. }
            | Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFile { source, .. } => Some(source),
            Self::Descriptor { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EdtSubsystemHierarchyError, EdtSubsystemHierarchyReader,
        FileSystemEdtSubsystemHierarchyReader,
    };
    use crate::{EdtSubsystemContentReader, FileSystemEdtSubsystemContentReader};
    use std::fmt::Write as _;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    const PREFIX: &str =
        r#"<mdclass:Subsystem xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass""#;

    fn object_directory(root: &Path, path: &[&str]) -> PathBuf {
        let mut directory = root.join("src/Subsystems");
        for (index, name) in path.iter().enumerate() {
            if index > 0 {
                directory.push("Subsystems");
            }
            directory.push(name);
        }
        directory
    }

    fn write_descriptor(
        root: &Path,
        path: &[&str],
        uuid: &str,
        children: &[&str],
        parents: &[&str],
    ) {
        let directory = object_directory(root, path);
        fs::create_dir_all(&directory).expect("fixture directory must be created");
        let name = path.last().expect("fixture path must have a name");
        let mut children_xml = String::new();
        for child in children {
            write!(children_xml, "<subsystems>{child}</subsystems>")
                .expect("writing to a String must succeed");
        }
        let mut parents_xml = String::new();
        for parent in parents {
            write!(parents_xml, "<parentSubsystem>{parent}</parentSubsystem>")
                .expect("writing to a String must succeed");
        }
        let xml = format!(
            r#"{PREFIX} uuid="{uuid}"><name>{name}</name><content>Document.Sample</content>{children_xml}{parents_xml}</mdclass:Subsystem>"#
        );
        fs::write(directory.join(format!("{name}.mdo")), xml)
            .expect("fixture descriptor must be written");
    }

    fn valid_project(root: &Path, reverse: bool) {
        let top_children = if reverse {
            ["Shared", "Branch"]
        } else {
            ["Branch", "Shared"]
        };
        write_descriptor(root, &["Alpha"], "id-alpha", &top_children, &[]);
        write_descriptor(
            root,
            &["Alpha", "Branch"],
            "id-branch",
            &["Leaf"],
            &["Subsystem.Alpha"],
        );
        write_descriptor(
            root,
            &["Alpha", "Branch", "Leaf"],
            "id-leaf",
            &[],
            &["Subsystem.Alpha.Subsystem.Branch"],
        );
        write_descriptor(
            root,
            &["Alpha", "Shared"],
            "id-alpha-shared",
            &[],
            &["Subsystem.Alpha"],
        );
        write_descriptor(root, &["Beta"], "id-beta", &["Shared"], &[]);
        write_descriptor(
            root,
            &["Beta", "Shared"],
            "id-beta-shared",
            &[],
            &["Subsystem.Beta"],
        );
    }

    fn read(root: &Path) -> Result<super::EdtSubsystemHierarchy, EdtSubsystemHierarchyError> {
        FileSystemEdtSubsystemHierarchyReader.read(root)
    }

    #[test]
    fn generated_hierarchy_is_ordered_preserves_duplicate_local_names_and_content_inputs() {
        let root = tempdir().expect("temporary project must be created");
        valid_project(root.path(), true);

        let hierarchy = read(root.path()).expect("valid hierarchy must parse");
        let paths = hierarchy
            .descriptors()
            .iter()
            .map(|descriptor| {
                descriptor
                    .hierarchy_path()
                    .iter()
                    .map(oneagent_common::EntityName::as_str)
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            paths,
            [
                "Alpha",
                "Alpha/Branch",
                "Alpha/Branch/Leaf",
                "Alpha/Shared",
                "Beta",
                "Beta/Shared",
            ]
        );
        assert_eq!(hierarchy.relations().len(), 4);
        let shared = hierarchy
            .descriptors()
            .iter()
            .filter(|descriptor| descriptor.metadata().name().as_str() == "Shared")
            .map(|descriptor| descriptor.metadata().id().as_str())
            .collect::<Vec<_>>();
        assert_eq!(shared, ["id-alpha-shared", "id-beta-shared"]);
        let content = FileSystemEdtSubsystemContentReader
            .read(hierarchy.descriptors()[2].metadata())
            .expect("existing content reader must accept nested metadata input");
        assert_eq!(content.raw_content(), ["Document.Sample"]);
    }

    #[test]
    fn reordered_and_repeated_reads_are_equal() {
        let root = tempdir().expect("temporary project must be created");
        valid_project(root.path(), false);
        let first = read(root.path()).expect("first hierarchy must parse");
        let repeated = read(root.path()).expect("repeated hierarchy must parse");
        valid_project(root.path(), true);
        let reordered = read(root.path()).expect("reordered hierarchy must parse");

        assert_eq!(first, repeated);
        assert_eq!(first, reordered);
    }

    #[test]
    #[cfg(feature = "external-edt-corpus-tests")]
    fn live_source_has_expected_depth_counts_and_relations() {
        let hierarchy = read(&crate::live_test_support::project_root())
            .expect("bootstrap-selected live EDT hierarchy must parse");
        let counts = hierarchy
            .descriptors()
            .iter()
            .fold([0_usize; 5], |mut counts, descriptor| {
                counts[descriptor.depth() - 1] += 1;
                counts
            });

        assert_eq!(hierarchy.descriptors().len(), 127);
        assert_eq!(hierarchy.relations().len(), 114);
        assert_eq!(counts, [13, 64, 39, 9, 2]);
    }

    #[test]
    fn missing_duplicate_and_extra_child_projections_are_typed() {
        let missing = tempdir().expect("temporary project must be created");
        write_descriptor(missing.path(), &["Root"], "root", &["Missing"], &[]);
        assert!(
            matches!(read(missing.path()), Err(EdtSubsystemHierarchyError::MissingChildDirectory { name, .. }) if name == "Missing")
        );

        let duplicate = tempdir().expect("temporary project must be created");
        write_descriptor(
            duplicate.path(),
            &["Root"],
            "root",
            &["Child", "Child"],
            &[],
        );
        assert!(
            matches!(read(duplicate.path()), Err(EdtSubsystemHierarchyError::DuplicateChildDeclaration { name, count: 2, .. }) if name == "Child")
        );

        let extra = tempdir().expect("temporary project must be created");
        write_descriptor(extra.path(), &["Root"], "root", &[], &[]);
        write_descriptor(
            extra.path(),
            &["Root", "Extra"],
            "extra",
            &[],
            &["Subsystem.Root"],
        );
        assert!(
            matches!(read(extra.path()), Err(EdtSubsystemHierarchyError::UndeclaredChildDirectory { name, .. }) if name == "Extra")
        );
    }

    #[test]
    fn nested_parent_cardinality_and_grammar_are_typed() {
        for (parents, expected) in [
            (vec![], "missing"),
            (vec!["Subsystem.Root", "Subsystem.Other"], "multiple"),
            (vec!["Wrong.Root"], "malformed"),
            (vec![" Subsystem.Root"], "malformed"),
            (vec!["Subsystem.Root.Subsystem"], "malformed"),
        ] {
            let root = tempdir().expect("temporary project must be created");
            write_descriptor(root.path(), &["Root"], "root", &["Child"], &[]);
            write_descriptor(root.path(), &["Root", "Child"], "child", &[], &parents);
            let error = read(root.path()).expect_err("invalid parent projection must fail");
            assert!(matches!(
                (expected, error),
                (
                    "missing",
                    EdtSubsystemHierarchyError::MissingParentDeclaration(_)
                ) | (
                    "multiple",
                    EdtSubsystemHierarchyError::MultipleParentDeclarations { .. }
                ) | (
                    "malformed",
                    EdtSubsystemHierarchyError::MalformedParentDeclaration { .. }
                )
            ));
        }
    }

    #[test]
    fn top_level_parent_path_mismatch_and_self_parent_are_typed() {
        let top = tempdir().expect("temporary project must be created");
        write_descriptor(top.path(), &["Root"], "root", &[], &["Subsystem.Other"]);
        assert!(matches!(
            read(top.path()),
            Err(EdtSubsystemHierarchyError::UnexpectedTopLevelParent { .. })
        ));

        let mismatch = tempdir().expect("temporary project must be created");
        write_descriptor(mismatch.path(), &["Root"], "root", &["Child"], &[]);
        write_descriptor(
            mismatch.path(),
            &["Root", "Child"],
            "child",
            &[],
            &["Subsystem.Other"],
        );
        assert!(matches!(
            read(mismatch.path()),
            Err(EdtSubsystemHierarchyError::ParentPathMismatch { .. })
        ));

        let self_parent = tempdir().expect("temporary project must be created");
        write_descriptor(self_parent.path(), &["Root"], "root", &["Child"], &[]);
        write_descriptor(
            self_parent.path(),
            &["Root", "Child"],
            "child",
            &[],
            &["Subsystem.Root.Subsystem.Child"],
        );
        assert!(matches!(
            read(self_parent.path()),
            Err(EdtSubsystemHierarchyError::SelfParent { .. })
        ));
    }

    #[test]
    fn parent_cycles_are_typed_before_path_mismatch() {
        let root = tempdir().expect("temporary project must be created");
        write_descriptor(root.path(), &["Root"], "root", &["A", "B"], &[]);
        write_descriptor(
            root.path(),
            &["Root", "A"],
            "a",
            &[],
            &["Subsystem.Root.Subsystem.B"],
        );
        write_descriptor(
            root.path(),
            &["Root", "B"],
            "b",
            &[],
            &["Subsystem.Root.Subsystem.A"],
        );

        assert!(
            matches!(read(root.path()), Err(EdtSubsystemHierarchyError::ParentCycle(ids)) if ids.len() == 2)
        );
    }

    #[test]
    fn path_name_duplicate_identifier_and_descriptor_ambiguity_are_typed() {
        let name = tempdir().expect("temporary project must be created");
        write_descriptor(name.path(), &["Root"], "root", &[], &[]);
        let descriptor = object_directory(name.path(), &["Root"]).join("Root.mdo");
        let xml = fs::read_to_string(&descriptor)
            .expect("fixture must be readable")
            .replace("<name>Root</name>", "<name>Other</name>");
        fs::write(&descriptor, xml).expect("fixture must be changed");
        assert!(matches!(
            read(name.path()),
            Err(EdtSubsystemHierarchyError::DescriptorPathNameMismatch { .. })
        ));

        let duplicate = tempdir().expect("temporary project must be created");
        write_descriptor(duplicate.path(), &["A"], "same", &[], &[]);
        write_descriptor(duplicate.path(), &["B"], "same", &[], &[]);
        assert!(matches!(
            read(duplicate.path()),
            Err(EdtSubsystemHierarchyError::DuplicateIdentifier { .. })
        ));

        let ambiguity = tempdir().expect("temporary project must be created");
        write_descriptor(ambiguity.path(), &["Root"], "root", &[], &[]);
        fs::write(
            object_directory(ambiguity.path(), &["Root"]).join("Other.mdo"),
            "ignored",
        )
        .expect("ambiguous descriptor must be written");
        assert!(matches!(
            read(ambiguity.path()),
            Err(EdtSubsystemHierarchyError::Descriptor {
                source: crate::EdtMetadataObjectError::MultipleDescriptors { .. },
                ..
            })
        ));
    }

    #[test]
    fn missing_and_unreadable_descriptors_are_typed() {
        let missing = tempdir().expect("temporary project must be created");
        fs::create_dir_all(object_directory(missing.path(), &["Root"]))
            .expect("descriptor directory must be created");
        assert!(matches!(
            read(missing.path()),
            Err(EdtSubsystemHierarchyError::Descriptor {
                source: crate::EdtMetadataObjectError::DescriptorNotFound(_),
                ..
            })
        ));

        let unreadable = tempdir().expect("temporary project must be created");
        write_descriptor(unreadable.path(), &["Root"], "root", &[], &[]);
        let descriptor = object_directory(unreadable.path(), &["Root"]).join("Root.mdo");
        fs::write(&descriptor, [0xff_u8]).expect("invalid UTF-8 descriptor must be written");
        assert!(matches!(
            read(unreadable.path()),
            Err(EdtSubsystemHierarchyError::Descriptor {
                source: crate::EdtMetadataObjectError::ReadFile { source, .. },
                ..
            }) if source.kind() == std::io::ErrorKind::InvalidData
        ));
    }

    #[test]
    fn malformed_root_namespace_names_and_absent_project_paths_are_typed() {
        let malformed = tempdir().expect("temporary project must be created");
        write_descriptor(malformed.path(), &["Root"], "root", &[], &[]);
        let descriptor = object_directory(malformed.path(), &["Root"]).join("Root.mdo");
        fs::write(&descriptor, "<mdclass:Subsystem").expect("fixture must be changed");
        assert!(matches!(
            read(malformed.path()),
            Err(EdtSubsystemHierarchyError::Descriptor {
                source: crate::EdtMetadataObjectError::MalformedXml(_),
                ..
            })
        ));

        let absent = tempdir().expect("temporary project must be created");
        assert!(matches!(
            read(absent.path()),
            Err(EdtSubsystemHierarchyError::SubsystemsDirectoryNotFound(_))
        ));

        let invalid = tempdir().expect("temporary project must be created");
        write_descriptor(invalid.path(), &["Root"], "root", &[" Bad"], &[]);
        assert!(matches!(
            read(invalid.path()),
            Err(EdtSubsystemHierarchyError::InvalidChildDeclaration { .. })
        ));
    }

    #[test]
    fn hierarchy_field_parser_requires_exact_root_namespace_and_direct_fields() {
        let descriptor = Path::new("Root.mdo");
        let wrong_root = r#"<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"><name>Root</name></mdclass:Document>"#;
        assert!(matches!(
            super::parse_hierarchy_fields(wrong_root, descriptor),
            Err(EdtSubsystemHierarchyError::UnexpectedRoot { root, .. })
                if root == "mdclass:Document"
        ));

        let wrong_namespace =
            r#"<mdclass:Subsystem xmlns:mdclass="urn:wrong"><name>Root</name></mdclass:Subsystem>"#;
        assert!(matches!(
            super::parse_hierarchy_fields(wrong_namespace, descriptor),
            Err(EdtSubsystemHierarchyError::UnsupportedNamespace {
                namespace: Some(namespace),
                ..
            }) if namespace == "urn:wrong"
        ));

        let nested = format!(
            r#"{PREFIX} uuid="root"><name>Root</name><properties><subsystems>Ignored</subsystems><parentSubsystem>Subsystem.Ignored</parentSubsystem></properties></mdclass:Subsystem>"#
        );
        let fields = super::parse_hierarchy_fields(&nested, descriptor)
            .expect("nested hierarchy-like fields must be ignored");
        assert!(fields.raw_child_declarations.is_empty());
        assert!(fields.raw_parent_declarations.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn duplicate_physical_alias_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary project must be created");
        write_descriptor(root.path(), &["Alpha"], "alpha", &[], &[]);
        let subsystems = root.path().join("src/Subsystems");
        symlink(subsystems.join("Alpha"), subsystems.join("ZAlias"))
            .expect("in-project alias must be created");

        assert!(matches!(
            read(root.path()),
            Err(EdtSubsystemHierarchyError::DirectoryCycleOrAlias { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary project must be created");
        let outside = tempdir().expect("outside directory must be created");
        write_descriptor(root.path(), &["Root"], "root", &["Outside"], &[]);
        let child = object_directory(root.path(), &["Root", "Outside"]);
        fs::create_dir_all(child.parent().expect("child must have a parent"))
            .expect("child container must be created");
        symlink(outside.path(), &child).expect("escape symlink must be created");

        assert!(matches!(
            read(root.path()),
            Err(EdtSubsystemHierarchyError::ProjectRootEscape { .. })
        ));
    }
}
