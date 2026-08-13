//! Reader for serialized EDT role-right declarations.

use oneagent_common::{EntityId, EntityName};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const RIGHTS_FILE_NAME: &str = "Rights.rights";
const RIGHTS_NAMESPACE: &str = "http://v8.1c.ru/8.2/roles";

/// Parsed EDT rights artifact associated with one role metadata object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtRoleRightsDescriptor {
    role_id: EntityId,
    set_for_new_objects: bool,
    set_for_attributes_by_default: bool,
    independent_rights_of_child_objects: bool,
    objects: Vec<EdtRoleObjectRights>,
    source_path: PathBuf,
}

impl EdtRoleRightsDescriptor {
    /// Returns the role metadata object identifier supplied by the caller.
    #[must_use]
    pub const fn role_id(&self) -> &EntityId {
        &self.role_id
    }

    /// Returns whether EDT enables rights for newly created metadata objects.
    #[must_use]
    pub const fn set_for_new_objects(&self) -> bool {
        self.set_for_new_objects
    }

    /// Returns whether EDT enables rights for attributes by default.
    #[must_use]
    pub const fn set_for_attributes_by_default(&self) -> bool {
        self.set_for_attributes_by_default
    }

    /// Returns whether child objects have independent rights.
    #[must_use]
    pub const fn independent_rights_of_child_objects(&self) -> bool {
        self.independent_rights_of_child_objects
    }

    /// Returns protected-resource declarations in source order.
    ///
    /// Duplicate declarations are preserved as separate source observations.
    #[must_use]
    pub fn objects(&self) -> &[EdtRoleObjectRights] {
        &self.objects
    }

    /// Returns the serialized EDT rights artifact path.
    #[must_use]
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

/// Rights declared for one protected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtRoleObjectRights {
    resource_name: EntityName,
    rights: Vec<EdtRoleRightDeclaration>,
}

impl EdtRoleObjectRights {
    /// Returns the EDT-qualified protected-resource name.
    #[must_use]
    pub const fn resource_name(&self) -> &EntityName {
        &self.resource_name
    }

    /// Returns right declarations in source order.
    ///
    /// Duplicate declarations are preserved as separate source observations.
    #[must_use]
    pub fn rights(&self) -> &[EdtRoleRightDeclaration] {
        &self.rights
    }
}

/// One direct right value declared for a protected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtRoleRightDeclaration {
    name: EntityName,
    value: bool,
    row_restriction: Option<EdtRoleRowRestriction>,
}

impl EdtRoleRightDeclaration {
    /// Returns the stable EDT right name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the direct boolean value serialized by EDT.
    #[must_use]
    pub const fn value(&self) -> bool {
        self.value
    }

    /// Returns the optional row-level restriction attached to this right.
    #[must_use]
    pub const fn row_restriction(&self) -> Option<&EdtRoleRowRestriction> {
        self.row_restriction.as_ref()
    }
}

/// Row-level restriction serialized inside an EDT right declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtRoleRowRestriction {
    condition: String,
}

impl EdtRoleRowRestriction {
    /// Returns the source condition without interpreting its expression language.
    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }
}

/// Reads `Rights.rights` from an EDT role object directory.
pub trait EdtRoleRightsReader {
    /// Reads the serialized rights artifact for `role_id`.
    ///
    /// # Errors
    ///
    /// Returns an error when the role directory or rights artifact is missing,
    /// cannot be read, or does not satisfy the confirmed EDT XML contract.
    fn read(
        &self,
        role_directory: &Path,
        role_id: &EntityId,
    ) -> Result<EdtRoleRightsDescriptor, EdtRoleRightsError>;
}

/// Filesystem implementation of [`EdtRoleRightsReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtRoleRightsReader;

impl EdtRoleRightsReader for FileSystemEdtRoleRightsReader {
    fn read(
        &self,
        role_directory: &Path,
        role_id: &EntityId,
    ) -> Result<EdtRoleRightsDescriptor, EdtRoleRightsError> {
        if !role_directory.is_dir() {
            return Err(EdtRoleRightsError::RoleDirectoryNotFound(
                role_directory.to_path_buf(),
            ));
        }

        let source_path = role_directory.join(RIGHTS_FILE_NAME);
        if !source_path.is_file() {
            return Err(EdtRoleRightsError::RightsFileNotFound(source_path));
        }

        let xml =
            fs::read_to_string(&source_path).map_err(|source| EdtRoleRightsError::ReadFile {
                path: source_path.clone(),
                source,
            })?;

        parse_role_rights(&xml, role_id.clone(), source_path)
    }
}

#[derive(Debug, Default)]
struct ParsedRoleRights {
    set_for_new_objects: Option<bool>,
    set_for_attributes_by_default: Option<bool>,
    independent_rights_of_child_objects: Option<bool>,
    objects: Vec<EdtRoleObjectRights>,
}

#[derive(Debug, Default)]
struct PendingObject {
    name: Option<String>,
    rights: Vec<EdtRoleRightDeclaration>,
}

#[derive(Debug, Default)]
struct PendingRight {
    name: Option<String>,
    value: Option<bool>,
    restriction_present: bool,
    condition: Option<String>,
}

fn parse_role_rights(
    xml: &str,
    role_id: EntityId,
    source_path: PathBuf,
) -> Result<EdtRoleRightsDescriptor, EdtRoleRightsError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut parsed = ParsedRoleRights::default();
    let mut path = Vec::<String>::new();
    let mut pending_object = None::<PendingObject>;
    let mut pending_right = None::<PendingRight>;
    let mut root_seen = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                handle_start(
                    &reader,
                    &event,
                    &mut path,
                    &mut pending_object,
                    &mut pending_right,
                    &mut root_seen,
                )?;
            }
            Ok(Event::Empty(event)) => {
                handle_empty(&reader, &event, &path, &mut root_seen)?;
            }
            Ok(Event::Text(event)) => {
                let text = event
                    .decode()
                    .map_err(|source| EdtRoleRightsError::MalformedXml(source.to_string()))?
                    .into_owned();
                parse_text(
                    &path,
                    text,
                    &mut parsed,
                    pending_object.as_mut(),
                    pending_right.as_mut(),
                )?;
            }
            Ok(Event::End(_)) => {
                handle_end(&path, &mut parsed, &mut pending_object, &mut pending_right)?;
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
            Err(source) => return Err(EdtRoleRightsError::MalformedXml(source.to_string())),
        }
    }

    if !root_seen {
        return Err(EdtRoleRightsError::MissingRoot);
    }

    Ok(EdtRoleRightsDescriptor {
        role_id,
        set_for_new_objects: parsed
            .set_for_new_objects
            .ok_or(EdtRoleRightsError::MissingField("setForNewObjects"))?,
        set_for_attributes_by_default: parsed.set_for_attributes_by_default.ok_or(
            EdtRoleRightsError::MissingField("setForAttributesByDefault"),
        )?,
        independent_rights_of_child_objects: parsed.independent_rights_of_child_objects.ok_or(
            EdtRoleRightsError::MissingField("independentRightsOfChildObjects"),
        )?,
        objects: parsed.objects,
        source_path,
    })
}

fn handle_start(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    path: &mut Vec<String>,
    pending_object: &mut Option<PendingObject>,
    pending_right: &mut Option<PendingRight>,
    root_seen: &mut bool,
) -> Result<(), EdtRoleRightsError> {
    let element = local_name(event.name().as_ref());
    if path.is_empty() {
        validate_root(reader, event, &element, *root_seen)?;
        *root_seen = true;
    }
    path.push(element);

    match path_as_str(path).as_slice() {
        ["Rights", "object"] => *pending_object = Some(PendingObject::default()),
        ["Rights", "object", "right"] => *pending_right = Some(PendingRight::default()),
        ["Rights", "object", "right", "restrictionByCondition"] => {
            let right = pending_right
                .as_mut()
                .ok_or(EdtRoleRightsError::RightOutsideObject)?;
            if right.restriction_present {
                return Err(EdtRoleRightsError::DuplicateField("restrictionByCondition"));
            }
            right.restriction_present = true;
        }
        _ => {}
    }
    Ok(())
}

fn handle_empty(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    path: &[String],
    root_seen: &mut bool,
) -> Result<(), EdtRoleRightsError> {
    let element = local_name(event.name().as_ref());
    if path.is_empty() {
        validate_root(reader, event, &element, *root_seen)?;
        *root_seen = true;
    }
    let mut empty_path = path.to_vec();
    empty_path.push(element);
    match path_as_str(&empty_path).as_slice() {
        ["Rights", "object"] => Err(EdtRoleRightsError::MissingObjectName),
        ["Rights", "object", "right"] => Err(EdtRoleRightsError::MissingRightName),
        ["Rights", "object", "right", "restrictionByCondition"] => {
            Err(EdtRoleRightsError::MissingRestrictionCondition)
        }
        _ => Ok(()),
    }
}

fn handle_end(
    path: &[String],
    parsed: &mut ParsedRoleRights,
    pending_object: &mut Option<PendingObject>,
    pending_right: &mut Option<PendingRight>,
) -> Result<(), EdtRoleRightsError> {
    match path_as_str(path).as_slice() {
        ["Rights", "object", "right"] => {
            let right = finish_right(
                pending_right
                    .take()
                    .ok_or(EdtRoleRightsError::RightOutsideObject)?,
            )?;
            pending_object
                .as_mut()
                .ok_or(EdtRoleRightsError::RightOutsideObject)?
                .rights
                .push(right);
        }
        ["Rights", "object"] => {
            parsed.objects.push(finish_object(
                pending_object
                    .take()
                    .ok_or(EdtRoleRightsError::MissingObjectName)?,
            )?);
        }
        _ => {}
    }
    Ok(())
}

fn validate_root(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    element: &str,
    root_seen: bool,
) -> Result<(), EdtRoleRightsError> {
    if root_seen || element != "Rights" {
        return Err(EdtRoleRightsError::UnexpectedRoot(element.to_owned()));
    }

    let mut namespace = None;
    for attribute in event.attributes().with_checks(false) {
        let attribute =
            attribute.map_err(|source| EdtRoleRightsError::MalformedXml(source.to_string()))?;
        if attribute.key.as_ref() == b"xmlns" {
            namespace = Some(
                attribute
                    .decode_and_unescape_value(reader.decoder())
                    .map_err(|source| EdtRoleRightsError::MalformedXml(source.to_string()))?
                    .into_owned(),
            );
        }
    }

    match namespace.as_deref() {
        Some(RIGHTS_NAMESPACE) => Ok(()),
        value => Err(EdtRoleRightsError::UnsupportedNamespace(
            value.map(str::to_owned),
        )),
    }
}

fn parse_text(
    path: &[String],
    text: String,
    parsed: &mut ParsedRoleRights,
    pending_object: Option<&mut PendingObject>,
    pending_right: Option<&mut PendingRight>,
) -> Result<(), EdtRoleRightsError> {
    match path_as_str(path).as_slice() {
        ["Rights", "setForNewObjects"] => {
            set_boolean(&mut parsed.set_for_new_objects, "setForNewObjects", &text)
        }
        ["Rights", "setForAttributesByDefault"] => set_boolean(
            &mut parsed.set_for_attributes_by_default,
            "setForAttributesByDefault",
            &text,
        ),
        ["Rights", "independentRightsOfChildObjects"] => set_boolean(
            &mut parsed.independent_rights_of_child_objects,
            "independentRightsOfChildObjects",
            &text,
        ),
        ["Rights", "object", "name"] => set_text(
            &mut pending_object
                .ok_or(EdtRoleRightsError::MissingObjectName)?
                .name,
            "object.name",
            text,
        ),
        ["Rights", "object", "right", "name"] => set_text(
            &mut pending_right
                .ok_or(EdtRoleRightsError::RightOutsideObject)?
                .name,
            "right.name",
            text,
        ),
        ["Rights", "object", "right", "value"] => {
            let right = pending_right.ok_or(EdtRoleRightsError::RightOutsideObject)?;
            set_boolean(&mut right.value, "right.value", &text)
        }
        [
            "Rights",
            "object",
            "right",
            "restrictionByCondition",
            "condition",
        ] => set_text(
            &mut pending_right
                .ok_or(EdtRoleRightsError::RightOutsideObject)?
                .condition,
            "restrictionByCondition.condition",
            text,
        ),
        _ => Ok(()),
    }
}

fn set_boolean(
    target: &mut Option<bool>,
    field: &'static str,
    value: &str,
) -> Result<(), EdtRoleRightsError> {
    if target.is_some() {
        return Err(EdtRoleRightsError::DuplicateField(field));
    }
    *target = Some(match value.trim() {
        "true" => true,
        "false" => false,
        value => {
            return Err(EdtRoleRightsError::InvalidBoolean {
                field,
                value: value.to_owned(),
            });
        }
    });
    Ok(())
}

fn set_text(
    target: &mut Option<String>,
    field: &'static str,
    value: String,
) -> Result<(), EdtRoleRightsError> {
    if target.is_some() {
        return Err(EdtRoleRightsError::DuplicateField(field));
    }
    *target = Some(value);
    Ok(())
}

fn finish_object(object: PendingObject) -> Result<EdtRoleObjectRights, EdtRoleRightsError> {
    let name = object.name.ok_or(EdtRoleRightsError::MissingObjectName)?;
    let resource_name = EntityName::new(name).map_err(|_| EdtRoleRightsError::InvalidObjectName)?;
    Ok(EdtRoleObjectRights {
        resource_name,
        rights: object.rights,
    })
}

fn finish_right(right: PendingRight) -> Result<EdtRoleRightDeclaration, EdtRoleRightsError> {
    let name = right.name.ok_or(EdtRoleRightsError::MissingRightName)?;
    let name = EntityName::new(name).map_err(|_| EdtRoleRightsError::InvalidRightName)?;
    let value = right.value.ok_or(EdtRoleRightsError::MissingRightValue)?;
    let row_restriction = if right.restriction_present {
        Some(EdtRoleRowRestriction {
            condition: right
                .condition
                .ok_or(EdtRoleRightsError::MissingRestrictionCondition)?,
        })
    } else {
        None
    };

    Ok(EdtRoleRightDeclaration {
        name,
        value,
        row_restriction,
    })
}

fn path_as_str(path: &[String]) -> Vec<&str> {
    path.iter().map(String::as_str).collect()
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

/// Error produced while reading an EDT role-right artifact.
#[derive(Debug)]
pub enum EdtRoleRightsError {
    /// The supplied role directory does not exist.
    RoleDirectoryNotFound(PathBuf),
    /// `Rights.rights` is absent from the supplied role directory.
    RightsFileNotFound(PathBuf),
    /// The rights artifact could not be read.
    ReadFile {
        /// File path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The XML document is malformed.
    MalformedXml(String),
    /// The document has no root element.
    MissingRoot,
    /// The document root is not `Rights`.
    UnexpectedRoot(String),
    /// The rights XML namespace is missing or unsupported.
    UnsupportedNamespace(Option<String>),
    /// A required singleton field is missing.
    MissingField(&'static str),
    /// A singleton field occurs more than once.
    DuplicateField(&'static str),
    /// A boolean field contains an unsupported value.
    InvalidBoolean {
        /// Field name.
        field: &'static str,
        /// Serialized value.
        value: String,
    },
    /// An object declaration has no protected-resource name.
    MissingObjectName,
    /// A protected-resource name is empty.
    InvalidObjectName,
    /// A right declaration occurs outside an object declaration.
    RightOutsideObject,
    /// A right declaration has no name.
    MissingRightName,
    /// A right name is empty.
    InvalidRightName,
    /// A right declaration has no boolean value.
    MissingRightValue,
    /// An RLS container has no condition.
    MissingRestrictionCondition,
}

impl Display for EdtRoleRightsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoleDirectoryNotFound(path) => write!(
                formatter,
                "EDT role directory was not found: {}",
                path.display()
            ),
            Self::RightsFileNotFound(path) => write!(
                formatter,
                "EDT role rights file was not found: {}",
                path.display()
            ),
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT role rights {}: {source}",
                path.display()
            ),
            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT role rights XML: {message}")
            }
            Self::MissingRoot => formatter.write_str("EDT role rights XML root is missing"),
            Self::UnexpectedRoot(root) => {
                write!(formatter, "unexpected EDT role rights XML root `{root}`")
            }
            Self::UnsupportedNamespace(Some(namespace)) => write!(
                formatter,
                "unsupported EDT role rights XML namespace `{namespace}`"
            ),
            Self::UnsupportedNamespace(None) => {
                formatter.write_str("EDT role rights XML namespace is missing")
            }
            Self::MissingField(field) => {
                write!(formatter, "EDT role rights field `{field}` is missing")
            }
            Self::DuplicateField(field) => {
                write!(formatter, "duplicate EDT role rights field `{field}`")
            }
            Self::InvalidBoolean { field, value } => write!(
                formatter,
                "invalid EDT role rights boolean `{value}` in `{field}`"
            ),
            Self::MissingObjectName => {
                formatter.write_str("EDT role rights object name is missing")
            }
            Self::InvalidObjectName => formatter.write_str("EDT role rights object name is empty"),
            Self::RightOutsideObject => {
                formatter.write_str("EDT role right is declared outside an object")
            }
            Self::MissingRightName => formatter.write_str("EDT role right name is missing"),
            Self::InvalidRightName => formatter.write_str("EDT role right name is empty"),
            Self::MissingRightValue => formatter.write_str("EDT role right value is missing"),
            Self::MissingRestrictionCondition => {
                formatter.write_str("EDT role right restriction condition is missing")
            }
        }
    }
}

impl std::error::Error for EdtRoleRightsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::{
        EdtRoleRightsError, EdtRoleRightsReader, FileSystemEdtRoleRightsReader, parse_role_rights,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn fixture_role(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/role_rights")
            .join(name)
    }

    fn parse(xml: &str) -> Result<super::EdtRoleRightsDescriptor, EdtRoleRightsError> {
        parse_role_rights(xml, id("role-id"), PathBuf::from("Rights.rights"))
    }

    #[test]
    fn role_rights_reader_reads_explicit_allows_and_rls_from_edt_fixture() {
        let role_directory = fixture_role("BaseUser");
        let descriptor = FileSystemEdtRoleRightsReader
            .read(&role_directory, &id("872b31fd-6bc2-44fa-8fbc-1995d6237ed7"))
            .expect("real EDT role rights fixture must parse");

        assert_eq!(
            descriptor.role_id().as_str(),
            "872b31fd-6bc2-44fa-8fbc-1995d6237ed7"
        );
        assert!(!descriptor.set_for_new_objects());
        assert!(descriptor.set_for_attributes_by_default());
        assert!(!descriptor.independent_rights_of_child_objects());
        assert_eq!(descriptor.objects().len(), 5);
        assert!(descriptor.source_path().ends_with("BaseUser/Rights.rights"));

        let catalog = &descriptor.objects()[2];
        assert_eq!(catalog.resource_name().as_str(), "Catalog.Product");
        assert_eq!(catalog.rights().len(), 5);
        assert_eq!(catalog.rights()[0].name().as_str(), "Read");
        assert!(catalog.rights()[0].value());
        assert_eq!(
            catalog.rights()[0]
                .row_restriction()
                .expect("read right must preserve RLS")
                .condition(),
            "WHERE NOT DeletionMark"
        );
        assert_eq!(catalog.rights()[1].name().as_str(), "Insert");
        assert!(catalog.rights()[1].row_restriction().is_none());
        assert_eq!(catalog.rights()[2].name().as_str(), "Update");
        assert!(catalog.rights()[2].row_restriction().is_some());
    }

    #[test]
    fn role_rights_reader_reads_default_only_edt_fixture() {
        let descriptor = FileSystemEdtRoleRightsReader
            .read(&fixture_role("FullAccess"), &id("full-access-role"))
            .expect("default-only EDT rights fixture must parse");

        assert!(descriptor.set_for_new_objects());
        assert!(descriptor.set_for_attributes_by_default());
        assert!(!descriptor.independent_rights_of_child_objects());
        assert!(descriptor.objects().is_empty());
    }

    #[test]
    fn role_rights_parser_preserves_order_and_duplicate_declarations() {
        let xml = r#"<Rights xmlns="http://v8.1c.ru/8.2/roles">
            <setForNewObjects>false</setForNewObjects>
            <setForAttributesByDefault>true</setForAttributesByDefault>
            <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
            <object><name>Catalog.Product</name>
                <right><name>Read</name><value>true</value></right>
                <right><name>Read</name><value>true</value></right>
            </object>
            <object><name>Catalog.Product</name></object>
        </Rights>"#;

        let descriptor = parse(xml).expect("duplicate declarations must be preserved");

        assert_eq!(descriptor.objects().len(), 2);
        assert_eq!(descriptor.objects()[0].rights().len(), 2);
        assert_eq!(descriptor.objects()[0].rights()[0].name().as_str(), "Read");
        assert_eq!(descriptor.objects()[0].rights()[1].name().as_str(), "Read");
        assert_eq!(
            descriptor.objects()[1].resource_name().as_str(),
            "Catalog.Product"
        );
    }

    #[test]
    fn role_rights_parser_preserves_false_without_assigning_deny_semantics() {
        let xml = r#"<Rights xmlns="http://v8.1c.ru/8.2/roles">
            <setForNewObjects>false</setForNewObjects>
            <setForAttributesByDefault>true</setForAttributesByDefault>
            <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
            <object><name>Catalog.Product</name>
                <right><name>Read</name><value>false</value></right>
            </object>
        </Rights>"#;

        let descriptor = parse(xml).expect("boolean false is valid XML vocabulary");

        assert!(!descriptor.objects()[0].rights()[0].value());
    }

    #[test]
    fn role_rights_parser_rejects_malformed_and_incomplete_input() {
        assert!(matches!(
            parse("<Rights>"),
            Err(EdtRoleRightsError::UnsupportedNamespace(None)
                | EdtRoleRightsError::MalformedXml(_))
        ));

        let invalid_boolean = r#"<Rights xmlns="http://v8.1c.ru/8.2/roles">
            <setForNewObjects>yes</setForNewObjects>
            <setForAttributesByDefault>true</setForAttributesByDefault>
            <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
        </Rights>"#;
        assert!(matches!(
            parse(invalid_boolean),
            Err(EdtRoleRightsError::InvalidBoolean {
                field: "setForNewObjects",
                ..
            })
        ));

        let missing_value = r#"<Rights xmlns="http://v8.1c.ru/8.2/roles">
            <setForNewObjects>false</setForNewObjects>
            <setForAttributesByDefault>true</setForAttributesByDefault>
            <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
            <object><name>Catalog.Product</name><right><name>Read</name></right></object>
        </Rights>"#;
        assert!(matches!(
            parse(missing_value),
            Err(EdtRoleRightsError::MissingRightValue)
        ));

        let missing_condition = r#"<Rights xmlns="http://v8.1c.ru/8.2/roles">
            <setForNewObjects>false</setForNewObjects>
            <setForAttributesByDefault>true</setForAttributesByDefault>
            <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
            <object><name>Catalog.Product</name><right><name>Read</name><value>true</value>
                <restrictionByCondition></restrictionByCondition>
            </right></object>
        </Rights>"#;
        assert!(matches!(
            parse(missing_condition),
            Err(EdtRoleRightsError::MissingRestrictionCondition)
        ));
    }

    #[test]
    fn role_rights_reader_reports_missing_directory_and_file() {
        let root = tempdir().expect("temporary directory must be created");
        let missing_directory = root.path().join("MissingRole");
        assert!(matches!(
            FileSystemEdtRoleRightsReader.read(&missing_directory, &id("role-id")),
            Err(EdtRoleRightsError::RoleDirectoryNotFound(_))
        ));

        assert!(matches!(
            FileSystemEdtRoleRightsReader.read(root.path(), &id("role-id")),
            Err(EdtRoleRightsError::RightsFileNotFound(_))
        ));
    }
}
