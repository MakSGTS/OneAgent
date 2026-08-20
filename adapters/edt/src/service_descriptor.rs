//! Typed parsers for EDT HTTP and Web Service descriptors.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{WebServiceParameterDirection, XdtoTypeReference};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesCData, BytesRef, BytesStart, BytesText, Event};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use crate::EdtMetadataObjectDescriptor;

const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";
const HTTP_ROOT: &str = "mdclass:HTTPService";
const WEB_ROOT: &str = "mdclass:WebService";

/// Parsed HTTP Service declaration without graph emission or handler resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtHttpServiceDescriptor {
    metadata: EdtMetadataObjectDescriptor,
    root_url: String,
    url_templates: Vec<EdtHttpUrlTemplate>,
}

impl EdtHttpServiceDescriptor {
    /// Returns the already discovered top-level metadata descriptor.
    #[must_use]
    pub const fn metadata(&self) -> &EdtMetadataObjectDescriptor {
        &self.metadata
    }

    /// Returns the exact declared service root URL.
    #[must_use]
    pub fn root_url(&self) -> &str {
        &self.root_url
    }

    /// Returns URL Templates ordered by stable UUID.
    #[must_use]
    pub fn url_templates(&self) -> &[EdtHttpUrlTemplate] {
        &self.url_templates
    }
}

/// Parsed direct HTTP Service URL Template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtHttpUrlTemplate {
    id: EntityId,
    name: EntityName,
    template: String,
    methods: Vec<EdtHttpMethod>,
}

impl EdtHttpUrlTemplate {
    /// Returns the stable source UUID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact declared name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the exact decoded URL template text.
    #[must_use]
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Returns nested Methods ordered by stable UUID.
    #[must_use]
    pub fn methods(&self) -> &[EdtHttpMethod] {
        &self.methods
    }
}

/// Parsed nested HTTP Service Method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtHttpMethod {
    id: EntityId,
    name: EntityName,
    http_method: Option<EntityName>,
    handler: EntityName,
}

impl EdtHttpMethod {
    /// Returns the stable source UUID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact owner-local name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the explicit HTTP method token, preserving absence.
    #[must_use]
    pub const fn http_method(&self) -> Option<&EntityName> {
        self.http_method.as_ref()
    }

    /// Returns the unresolved exact handler procedure name.
    #[must_use]
    pub const fn handler(&self) -> &EntityName {
        &self.handler
    }
}

/// Parsed Web Service declaration without graph emission or resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtWebServiceDescriptor {
    metadata: EdtMetadataObjectDescriptor,
    namespace: String,
    xdto_packages: Vec<EdtWebServiceXdtoPackage>,
    operations: Vec<EdtWebServiceOperation>,
}

impl EdtWebServiceDescriptor {
    /// Returns the already discovered top-level metadata descriptor.
    #[must_use]
    pub const fn metadata(&self) -> &EdtMetadataObjectDescriptor {
        &self.metadata
    }

    /// Returns the exact declared Web Service namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns unresolved XDTO package declarations in canonical order.
    #[must_use]
    pub fn xdto_packages(&self) -> &[EdtWebServiceXdtoPackage] {
        &self.xdto_packages
    }

    /// Returns Operations ordered by stable UUID.
    #[must_use]
    pub fn operations(&self) -> &[EdtWebServiceOperation] {
        &self.operations
    }
}

/// Typed unresolved XDTO package declaration used by a Web Service.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdtWebServiceXdtoPackage {
    /// Exact `XDTOPackage.<name>` repository reference.
    Repository(EntityName),
    /// Exact external namespace URI.
    ExternalNamespace(String),
}

/// Parsed direct Web Service Operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtWebServiceOperation {
    id: EntityId,
    name: EntityName,
    returning_type: XdtoTypeReference,
    nillable: Option<bool>,
    procedure_name: EntityName,
    parameters: Vec<EdtWebServiceParameter>,
}

impl EdtWebServiceOperation {
    /// Returns the stable source UUID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact owner-local name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the exact unresolved return type declaration.
    #[must_use]
    pub const fn returning_type(&self) -> &XdtoTypeReference {
        &self.returning_type
    }

    /// Returns explicit nillability, preserving absence.
    #[must_use]
    pub const fn nillable(&self) -> Option<bool> {
        self.nillable
    }

    /// Returns the unresolved exact procedure name.
    #[must_use]
    pub const fn procedure_name(&self) -> &EntityName {
        &self.procedure_name
    }

    /// Returns Parameters ordered by stable UUID.
    #[must_use]
    pub fn parameters(&self) -> &[EdtWebServiceParameter] {
        &self.parameters
    }
}

/// Parsed nested Web Service Parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtWebServiceParameter {
    id: EntityId,
    name: EntityName,
    value_type: XdtoTypeReference,
    nillable: Option<bool>,
    direction: Option<WebServiceParameterDirection>,
}

impl EdtWebServiceParameter {
    /// Returns the stable source UUID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact owner-local name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the exact unresolved value type declaration.
    #[must_use]
    pub const fn value_type(&self) -> &XdtoTypeReference {
        &self.value_type
    }

    /// Returns explicit nillability, preserving absence.
    #[must_use]
    pub const fn nillable(&self) -> Option<bool> {
        self.nillable
    }

    /// Returns explicit transfer direction, preserving absence.
    #[must_use]
    pub const fn direction(&self) -> Option<WebServiceParameterDirection> {
        self.direction
    }
}

/// Reads already discovered EDT HTTP and Web Service descriptors.
pub trait EdtServiceDescriptorReader {
    /// Reads one HTTP Service descriptor.
    ///
    /// # Errors
    ///
    /// Returns a typed filesystem, XML, hierarchy, cardinality, or field error.
    fn read_http(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtHttpServiceDescriptor, EdtServiceDescriptorError>;

    /// Reads one Web Service descriptor.
    ///
    /// # Errors
    ///
    /// Returns a typed filesystem, XML, hierarchy, wrapper, or field error.
    fn read_web(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtWebServiceDescriptor, EdtServiceDescriptorError>;
}

/// Filesystem implementation of [`EdtServiceDescriptorReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtServiceDescriptorReader;

impl EdtServiceDescriptorReader for FileSystemEdtServiceDescriptorReader {
    fn read_http(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtHttpServiceDescriptor, EdtServiceDescriptorError> {
        require_kind(metadata, MetadataKind::HttpService)?;
        let xml = read_descriptor(metadata)?;
        parse_http(&xml, metadata)
    }

    fn read_web(
        &self,
        metadata: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtWebServiceDescriptor, EdtServiceDescriptorError> {
        require_kind(metadata, MetadataKind::WebService)?;
        let xml = read_descriptor(metadata)?;
        parse_web(&xml, metadata)
    }
}

fn require_kind(
    metadata: &EdtMetadataObjectDescriptor,
    expected: MetadataKind,
) -> Result<(), EdtServiceDescriptorError> {
    if metadata.kind() != expected {
        return Err(EdtServiceDescriptorError::UnexpectedMetadataKind {
            expected,
            actual: metadata.kind(),
        });
    }
    Ok(())
}

fn read_descriptor(
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<String, EdtServiceDescriptorError> {
    fs::read_to_string(metadata.descriptor_path()).map_err(|source| {
        EdtServiceDescriptorError::ReadFile {
            path: metadata.descriptor_path().to_path_buf(),
            source,
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawElement {
    name: String,
    attributes: BTreeMap<String, String>,
    text: String,
    children: Vec<RawElement>,
}

fn parse_xml(xml: &str) -> Result<RawElement, EdtServiceDescriptorError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut stack = Vec::<RawElement>::new();
    let mut root = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => stack.push(raw_element(&reader, &event)?),
            Ok(Event::Empty(event)) => {
                attach_element(raw_element(&reader, &event)?, &mut stack, &mut root)?;
            }
            Ok(Event::Text(event)) => append_text(&event, stack.last_mut())?,
            Ok(Event::CData(event)) => append_cdata(&event, stack.last_mut())?,
            Ok(Event::GeneralRef(event)) => append_reference(&event, stack.last_mut())?,
            Ok(Event::End(_)) => {
                let element = stack.pop().ok_or_else(|| {
                    EdtServiceDescriptorError::MalformedXml("unexpected closing tag".to_owned())
                })?;
                attach_element(element, &mut stack, &mut root)?;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Err(source) => {
                return Err(EdtServiceDescriptorError::MalformedXml(source.to_string()));
            }
        }
    }
    if !stack.is_empty() {
        return Err(EdtServiceDescriptorError::MalformedXml(
            "unexpected end of file before the root was closed".to_owned(),
        ));
    }
    root.ok_or(EdtServiceDescriptorError::MissingRoot)
}

fn raw_element(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<RawElement, EdtServiceDescriptorError> {
    let name = std::str::from_utf8(event.name().as_ref())
        .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?
        .to_owned();
    let mut attributes = BTreeMap::new();
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        let key = std::str::from_utf8(attribute.key.as_ref())
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?
            .to_owned();
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?
            .into_owned();
        if attributes.insert(key.clone(), value).is_some() {
            return Err(EdtServiceDescriptorError::MalformedXml(format!(
                "duplicate XML attribute `{key}`"
            )));
        }
    }
    Ok(RawElement {
        name,
        attributes,
        text: String::new(),
        children: Vec::new(),
    })
}

fn attach_element(
    element: RawElement,
    stack: &mut [RawElement],
    root: &mut Option<RawElement>,
) -> Result<(), EdtServiceDescriptorError> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(element);
    } else if root.replace(element).is_some() {
        return Err(EdtServiceDescriptorError::MalformedXml(
            "multiple root elements".to_owned(),
        ));
    }
    Ok(())
}

fn append_text(
    event: &BytesText<'_>,
    element: Option<&mut RawElement>,
) -> Result<(), EdtServiceDescriptorError> {
    if let Some(element) = element {
        let decoded = event
            .decode()
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        let decoded = unescape(&decoded)
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        element.text.push_str(&decoded);
    }
    Ok(())
}

fn append_cdata(
    event: &BytesCData<'_>,
    element: Option<&mut RawElement>,
) -> Result<(), EdtServiceDescriptorError> {
    if let Some(element) = element {
        let decoded = event
            .decode()
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        element.text.push_str(&decoded);
    }
    Ok(())
}

fn append_reference(
    event: &BytesRef<'_>,
    element: Option<&mut RawElement>,
) -> Result<(), EdtServiceDescriptorError> {
    if let Some(element) = element {
        let reference = event
            .decode()
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        let encoded = format!("&{reference};");
        let decoded = unescape(&encoded)
            .map_err(|source| EdtServiceDescriptorError::MalformedXml(source.to_string()))?;
        element.text.push_str(&decoded);
    }
    Ok(())
}

fn validate_root(
    root: &RawElement,
    expected_root: &'static str,
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<(), EdtServiceDescriptorError> {
    if root.name != expected_root {
        return Err(EdtServiceDescriptorError::UnexpectedRoot(root.name.clone()));
    }
    let namespace = root.attributes.get("xmlns:mdclass").cloned();
    if namespace.as_deref() != Some(METADATA_NAMESPACE) {
        return Err(EdtServiceDescriptorError::UnsupportedNamespace(namespace));
    }
    let uuid = required_attribute_named(root, "uuid", "service root")?;
    let name = required_text(root, "name", expected_root)?;
    if uuid != metadata.id().as_str() || name != metadata.name().as_str() {
        return Err(EdtServiceDescriptorError::DescriptorIdentityMismatch {
            expected_uuid: metadata.id().as_str().to_owned(),
            actual_uuid: uuid.to_owned(),
            expected_name: metadata.name().as_str().to_owned(),
            actual_name: name,
        });
    }
    Ok(())
}

fn parse_http(
    xml: &str,
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<EdtHttpServiceDescriptor, EdtServiceDescriptorError> {
    let root = parse_xml(xml)?;
    validate_root(&root, HTTP_ROOT, metadata)?;
    validate_http_hierarchy(&root, None)?;
    let root_url = required_text(&root, "rootURL", HTTP_ROOT)?;

    let mut url_templates = root
        .children
        .iter()
        .filter(|child| child.name == "urlTemplates")
        .map(parse_http_url_template)
        .collect::<Result<Vec<_>, _>>()?;
    ensure_http_uniqueness(&url_templates)?;
    url_templates.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(EdtHttpServiceDescriptor {
        metadata: metadata.clone(),
        root_url,
        url_templates,
    })
}

fn parse_http_url_template(
    element: &RawElement,
) -> Result<EdtHttpUrlTemplate, EdtServiceDescriptorError> {
    let id = entity_id(element)?;
    let name = entity_name(required_text(element, "name", "urlTemplates")?, "name")?;
    let template = required_text(element, "template", "urlTemplates")?;
    let mut methods = element
        .children
        .iter()
        .filter(|child| child.name == "methods")
        .map(parse_http_method)
        .collect::<Result<Vec<_>, _>>()?;
    ensure_owner_local_names(
        id.as_str(),
        methods.iter().map(|method| method.name.as_str()),
        "methods",
    )?;
    methods.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(EdtHttpUrlTemplate {
        id,
        name,
        template,
        methods,
    })
}

fn parse_http_method(element: &RawElement) -> Result<EdtHttpMethod, EdtServiceDescriptorError> {
    Ok(EdtHttpMethod {
        id: entity_id(element)?,
        name: entity_name(required_text(element, "name", "methods")?, "name")?,
        http_method: optional_text(element, "httpMethod", "methods")?
            .map(|value| entity_name(value, "httpMethod"))
            .transpose()?,
        handler: entity_name(required_text(element, "handler", "methods")?, "handler")?,
    })
}

fn ensure_http_uniqueness(
    url_templates: &[EdtHttpUrlTemplate],
) -> Result<(), EdtServiceDescriptorError> {
    ensure_owner_local_names(
        HTTP_ROOT,
        url_templates.iter().map(|template| template.name.as_str()),
        "urlTemplates",
    )?;
    let mut identities = Vec::new();
    for template in url_templates {
        identities.push((template.id.as_str(), "urlTemplates"));
        identities.extend(
            template
                .methods
                .iter()
                .map(|method| (method.id.as_str(), "methods")),
        );
    }
    ensure_unique_uuids(identities)
}

fn parse_web(
    xml: &str,
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<EdtWebServiceDescriptor, EdtServiceDescriptorError> {
    let root = parse_xml(xml)?;
    validate_root(&root, WEB_ROOT, metadata)?;
    validate_web_hierarchy(&root, None)?;
    let namespace = required_text(&root, "namespace", WEB_ROOT)?;
    let mut xdto_packages = direct_children(&root, "xdtoPackages")
        .into_iter()
        .map(parse_xdto_package)
        .collect::<Result<Vec<_>, _>>()?;
    xdto_packages.sort();
    xdto_packages.dedup();
    let mut operations = root
        .children
        .iter()
        .filter(|child| child.name == "operations")
        .map(parse_web_operation)
        .collect::<Result<Vec<_>, _>>()?;
    ensure_web_uniqueness(&operations)?;
    operations.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(EdtWebServiceDescriptor {
        metadata: metadata.clone(),
        namespace,
        xdto_packages,
        operations,
    })
}

fn parse_xdto_package(
    element: &RawElement,
) -> Result<EdtWebServiceXdtoPackage, EdtServiceDescriptorError> {
    let wrapper = required_attribute_named(element, "xsi:type", "xdtoPackages")?;
    let value = required_text(element, "value", "xdtoPackages")?;
    match wrapper {
        "core:ReferenceValue" => {
            let package = value
                .strip_prefix("XDTOPackage.")
                .ok_or_else(|| EdtServiceDescriptorError::InvalidPackageReference(value.clone()))?;
            if package.is_empty() || package.contains('.') {
                return Err(EdtServiceDescriptorError::InvalidPackageReference(value));
            }
            Ok(EdtWebServiceXdtoPackage::Repository(entity_name(
                package.to_owned(),
                "xdtoPackages.value",
            )?))
        }
        "core:StringValue" => Ok(EdtWebServiceXdtoPackage::ExternalNamespace(value)),
        other => Err(EdtServiceDescriptorError::UnsupportedPackageWrapper(
            other.to_owned(),
        )),
    }
}

fn parse_web_operation(
    element: &RawElement,
) -> Result<EdtWebServiceOperation, EdtServiceDescriptorError> {
    let id = entity_id(element)?;
    let name = entity_name(required_text(element, "name", "operations")?, "name")?;
    let returning_type = parse_type_declaration(
        required_element(element, "xdtoReturningValueType", "operations")?,
        "xdtoReturningValueType",
    )?;
    let nillable = optional_boolean(element, "nillable", "operations")?;
    let procedure_name = entity_name(
        required_text(element, "procedureName", "operations")?,
        "procedureName",
    )?;
    let mut parameters = element
        .children
        .iter()
        .filter(|child| child.name == "parameters")
        .map(parse_web_parameter)
        .collect::<Result<Vec<_>, _>>()?;
    ensure_owner_local_names(
        id.as_str(),
        parameters.iter().map(|parameter| parameter.name.as_str()),
        "parameters",
    )?;
    parameters.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(EdtWebServiceOperation {
        id,
        name,
        returning_type,
        nillable,
        procedure_name,
        parameters,
    })
}

fn parse_web_parameter(
    element: &RawElement,
) -> Result<EdtWebServiceParameter, EdtServiceDescriptorError> {
    let direction = optional_text(element, "transferDirection", "parameters")?
        .map(|value| match value.as_str() {
            "Out" => Ok(WebServiceParameterDirection::Out),
            "InOut" => Ok(WebServiceParameterDirection::InOut),
            _ => Err(EdtServiceDescriptorError::UnsupportedDirection(value)),
        })
        .transpose()?;
    Ok(EdtWebServiceParameter {
        id: entity_id(element)?,
        name: entity_name(required_text(element, "name", "parameters")?, "name")?,
        value_type: parse_type_declaration(
            required_element(element, "xdtoValueType", "parameters")?,
            "xdtoValueType",
        )?,
        nillable: optional_boolean(element, "nillable", "parameters")?,
        direction,
    })
}

fn parse_type_declaration(
    element: &RawElement,
    context: &'static str,
) -> Result<XdtoTypeReference, EdtServiceDescriptorError> {
    let name = entity_name(required_text(element, "name", context)?, "type.name")?;
    let namespace = required_text(element, "nsUri", context)?;
    Ok(XdtoTypeReference::new(namespace, name))
}

fn ensure_web_uniqueness(
    operations: &[EdtWebServiceOperation],
) -> Result<(), EdtServiceDescriptorError> {
    ensure_owner_local_names(
        WEB_ROOT,
        operations.iter().map(|operation| operation.name.as_str()),
        "operations",
    )?;
    let mut identities = Vec::new();
    for operation in operations {
        identities.push((operation.id.as_str(), "operations"));
        identities.extend(
            operation
                .parameters
                .iter()
                .map(|parameter| (parameter.id.as_str(), "parameters")),
        );
    }
    ensure_unique_uuids(identities)
}

fn validate_http_hierarchy(
    element: &RawElement,
    parent: Option<&str>,
) -> Result<(), EdtServiceDescriptorError> {
    let accepted = match element.name.as_str() {
        "urlTemplates" => parent == Some(HTTP_ROOT),
        "methods" => parent == Some("urlTemplates"),
        _ => true,
    };
    if !accepted {
        return Err(EdtServiceDescriptorError::UnexpectedHierarchy {
            element: element.name.clone(),
            parent: parent.unwrap_or("<root>").to_owned(),
        });
    }
    for child in &element.children {
        validate_http_hierarchy(child, Some(&element.name))?;
    }
    Ok(())
}

fn validate_web_hierarchy(
    element: &RawElement,
    parent: Option<&str>,
) -> Result<(), EdtServiceDescriptorError> {
    match element.name.as_str() {
        "xdtoReturningValueType" if parent == Some("parameters") => {
            return Err(EdtServiceDescriptorError::ConflictingTypeWrapper {
                context: "parameters",
                expected: "xdtoValueType",
                actual: "xdtoReturningValueType",
            });
        }
        "xdtoValueType" if parent == Some("operations") => {
            return Err(EdtServiceDescriptorError::ConflictingTypeWrapper {
                context: "operations",
                expected: "xdtoReturningValueType",
                actual: "xdtoValueType",
            });
        }
        "operations" | "xdtoPackages" if parent != Some(WEB_ROOT) => {
            return hierarchy_error(element, parent);
        }
        "parameters" | "xdtoReturningValueType" if parent != Some("operations") => {
            return hierarchy_error(element, parent);
        }
        "xdtoValueType" if parent != Some("parameters") => {
            return hierarchy_error(element, parent);
        }
        _ => {}
    }
    for child in &element.children {
        validate_web_hierarchy(child, Some(&element.name))?;
    }
    Ok(())
}

fn hierarchy_error<T>(
    element: &RawElement,
    parent: Option<&str>,
) -> Result<T, EdtServiceDescriptorError> {
    Err(EdtServiceDescriptorError::UnexpectedHierarchy {
        element: element.name.clone(),
        parent: parent.unwrap_or("<root>").to_owned(),
    })
}

fn required_attribute_named<'a>(
    element: &'a RawElement,
    attribute: &'static str,
    context: &'static str,
) -> Result<&'a str, EdtServiceDescriptorError> {
    let value = element
        .attributes
        .get(attribute)
        .ok_or(EdtServiceDescriptorError::MissingAttribute { context, attribute })?;
    if value.is_empty() {
        return Err(EdtServiceDescriptorError::EmptyAttribute { context, attribute });
    }
    Ok(value)
}

fn entity_id(element: &RawElement) -> Result<EntityId, EdtServiceDescriptorError> {
    let raw =
        element
            .attributes
            .get("uuid")
            .ok_or(EdtServiceDescriptorError::MissingAttribute {
                context: "service child",
                attribute: "uuid",
            })?;
    if raw.is_empty() {
        return Err(EdtServiceDescriptorError::EmptyAttribute {
            context: "service child",
            attribute: "uuid",
        });
    }
    EntityId::new(raw.clone()).map_err(|_| EdtServiceDescriptorError::InvalidUuid(raw.clone()))
}

fn entity_name(raw: String, field: &'static str) -> Result<EntityName, EdtServiceDescriptorError> {
    EntityName::new(raw.clone())
        .map_err(|_| EdtServiceDescriptorError::InvalidName { field, value: raw })
}

fn direct_children<'a>(element: &'a RawElement, name: &str) -> Vec<&'a RawElement> {
    element
        .children
        .iter()
        .filter(|child| child.name == name)
        .collect()
}

fn required_element<'a>(
    element: &'a RawElement,
    name: &'static str,
    context: &'static str,
) -> Result<&'a RawElement, EdtServiceDescriptorError> {
    let children = direct_children(element, name);
    match children.as_slice() {
        [] => Err(EdtServiceDescriptorError::MissingField {
            context,
            field: name,
        }),
        [child] => Ok(child),
        _ => Err(EdtServiceDescriptorError::DuplicateField {
            context,
            field: name,
        }),
    }
}

fn optional_element<'a>(
    element: &'a RawElement,
    name: &'static str,
    context: &'static str,
) -> Result<Option<&'a RawElement>, EdtServiceDescriptorError> {
    let children = direct_children(element, name);
    match children.as_slice() {
        [] => Ok(None),
        [child] => Ok(Some(child)),
        _ => Err(EdtServiceDescriptorError::DuplicateField {
            context,
            field: name,
        }),
    }
}

fn required_text(
    element: &RawElement,
    name: &'static str,
    context: &'static str,
) -> Result<String, EdtServiceDescriptorError> {
    let child = required_element(element, name, context)?;
    scalar_text(child, context, name)
}

fn optional_text(
    element: &RawElement,
    name: &'static str,
    context: &'static str,
) -> Result<Option<String>, EdtServiceDescriptorError> {
    optional_element(element, name, context)?
        .map(|child| scalar_text(child, context, name))
        .transpose()
}

fn scalar_text(
    element: &RawElement,
    context: &'static str,
    field: &'static str,
) -> Result<String, EdtServiceDescriptorError> {
    if !element.children.is_empty() {
        return Err(EdtServiceDescriptorError::NonScalarField { context, field });
    }
    if element.text.is_empty() {
        return Err(EdtServiceDescriptorError::EmptyField { context, field });
    }
    Ok(element.text.clone())
}

fn optional_boolean(
    element: &RawElement,
    name: &'static str,
    context: &'static str,
) -> Result<Option<bool>, EdtServiceDescriptorError> {
    optional_text(element, name, context)?
        .map(|value| match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(EdtServiceDescriptorError::InvalidBoolean { field: name, value }),
        })
        .transpose()
}

fn ensure_owner_local_names<'a>(
    owner: &str,
    names: impl IntoIterator<Item = &'a str>,
    element: &'static str,
) -> Result<(), EdtServiceDescriptorError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for name in names {
        *counts.entry(name.to_owned()).or_default() += 1;
    }
    if let Some((name, _)) = counts.into_iter().find(|(_, count)| *count > 1) {
        return Err(EdtServiceDescriptorError::DuplicateName {
            owner: owner.to_owned(),
            element,
            name,
        });
    }
    Ok(())
}

fn ensure_unique_uuids<'a>(
    identities: impl IntoIterator<Item = (&'a str, &'static str)>,
) -> Result<(), EdtServiceDescriptorError> {
    let mut occurrences = BTreeMap::<String, Vec<&'static str>>::new();
    for (uuid, element) in identities {
        occurrences
            .entry(uuid.to_owned())
            .or_default()
            .push(element);
    }
    if let Some((uuid, elements)) = occurrences
        .into_iter()
        .find(|(_, elements)| elements.len() > 1)
    {
        let mut elements = elements;
        elements.sort_unstable();
        return Err(EdtServiceDescriptorError::DuplicateUuid { uuid, elements });
    }
    Ok(())
}

/// Errors produced while parsing EDT HTTP and Web Service descriptors.
#[derive(Debug)]
pub enum EdtServiceDescriptorError {
    /// The supplied metadata descriptor has the wrong top-level kind.
    UnexpectedMetadataKind {
        /// Required service kind.
        expected: MetadataKind,
        /// Supplied kind.
        actual: MetadataKind,
    },
    /// The descriptor cannot be read.
    ReadFile {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The descriptor XML is malformed.
    MalformedXml(String),
    /// The descriptor has no root element.
    MissingRoot,
    /// The descriptor root does not match its service family.
    UnexpectedRoot(String),
    /// The metadata namespace is missing or unsupported.
    UnsupportedNamespace(Option<String>),
    /// Re-read UUID/name content differs from the supplied descriptor.
    DescriptorIdentityMismatch {
        /// UUID from the supplied descriptor.
        expected_uuid: String,
        /// UUID from the source.
        actual_uuid: String,
        /// Name from the supplied descriptor.
        expected_name: String,
        /// Name from the source.
        actual_name: String,
    },
    /// A required XML attribute is missing.
    MissingAttribute {
        /// Element context.
        context: &'static str,
        /// Attribute name.
        attribute: &'static str,
    },
    /// A required XML attribute is empty.
    EmptyAttribute {
        /// Element context.
        context: &'static str,
        /// Attribute name.
        attribute: &'static str,
    },
    /// A UUID cannot be represented by the common identifier primitive.
    InvalidUuid(String),
    /// A required direct field is missing.
    MissingField {
        /// Semantic owner context.
        context: &'static str,
        /// Field name.
        field: &'static str,
    },
    /// A direct field is empty.
    EmptyField {
        /// Semantic owner context.
        context: &'static str,
        /// Field name.
        field: &'static str,
    },
    /// A singleton direct field occurs more than once.
    DuplicateField {
        /// Semantic owner context.
        context: &'static str,
        /// Field name.
        field: &'static str,
    },
    /// A scalar field contains nested XML elements.
    NonScalarField {
        /// Semantic owner context.
        context: &'static str,
        /// Field name.
        field: &'static str,
    },
    /// A name cannot be represented by the common name primitive.
    InvalidName {
        /// Field role.
        field: &'static str,
        /// Exact decoded value.
        value: String,
    },
    /// A recognized declaration appears under the wrong immediate parent.
    UnexpectedHierarchy {
        /// Recognized element.
        element: String,
        /// Actual immediate parent.
        parent: String,
    },
    /// One UUID is reused by multiple service children.
    DuplicateUuid {
        /// Duplicated UUID.
        uuid: String,
        /// Canonically ordered element families.
        elements: Vec<&'static str>,
    },
    /// One owner-local name is reused.
    DuplicateName {
        /// Exact owner identity.
        owner: String,
        /// Child element family.
        element: &'static str,
        /// Duplicated exact name.
        name: String,
    },
    /// The `xdtoPackages` value wrapper is unsupported.
    UnsupportedPackageWrapper(String),
    /// A repository package reference does not match `XDTOPackage.<name>`.
    InvalidPackageReference(String),
    /// A type wrapper belongs to the opposite operation/parameter role.
    ConflictingTypeWrapper {
        /// Semantic owner context.
        context: &'static str,
        /// Required wrapper.
        expected: &'static str,
        /// Found wrapper.
        actual: &'static str,
    },
    /// An optional Boolean contains an unsupported token.
    InvalidBoolean {
        /// Boolean field.
        field: &'static str,
        /// Exact unsupported value.
        value: String,
    },
    /// A transfer direction is outside the accepted `Out`/`InOut` vocabulary.
    UnsupportedDirection(String),
}

impl Display for EdtServiceDescriptorError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedMetadataKind { expected, actual } => {
                write!(
                    formatter,
                    "expected EDT `{expected}` metadata, found `{actual}`"
                )
            }
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT service descriptor {}: {source}",
                path.display()
            ),
            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT service descriptor XML: {message}")
            }
            Self::MissingRoot => formatter.write_str("EDT service descriptor root is missing"),
            Self::UnexpectedRoot(root) => {
                write!(formatter, "unexpected EDT service descriptor root `{root}`")
            }
            Self::UnsupportedNamespace(Some(namespace)) => {
                write!(formatter, "unsupported EDT service namespace `{namespace}`")
            }
            Self::UnsupportedNamespace(None) => {
                formatter.write_str("EDT service namespace is missing")
            }
            Self::DescriptorIdentityMismatch {
                expected_uuid,
                actual_uuid,
                expected_name,
                actual_name,
            } => write!(
                formatter,
                "EDT service identity mismatch: expected {expected_uuid}/{expected_name}, found {actual_uuid}/{actual_name}"
            ),
            Self::MissingAttribute { context, attribute } => {
                write!(formatter, "`{context}` attribute `{attribute}` is missing")
            }
            Self::EmptyAttribute { context, attribute } => {
                write!(formatter, "`{context}` attribute `{attribute}` is empty")
            }
            Self::InvalidUuid(uuid) => write!(formatter, "invalid service child UUID `{uuid}`"),
            Self::MissingField { context, field } => {
                write!(formatter, "`{context}` field `{field}` is missing")
            }
            Self::EmptyField { context, field } => {
                write!(formatter, "`{context}` field `{field}` is empty")
            }
            Self::DuplicateField { context, field } => {
                write!(
                    formatter,
                    "`{context}` field `{field}` occurs more than once"
                )
            }
            Self::NonScalarField { context, field } => {
                write!(formatter, "`{context}` field `{field}` is not scalar")
            }
            Self::InvalidName { field, value } => {
                write!(formatter, "invalid `{field}` name `{value}`")
            }
            Self::UnexpectedHierarchy { element, parent } => {
                write!(
                    formatter,
                    "service element `{element}` has wrong parent `{parent}`"
                )
            }
            Self::DuplicateUuid { uuid, elements } => {
                write!(formatter, "duplicate service UUID `{uuid}` in {elements:?}")
            }
            Self::DuplicateName {
                owner,
                element,
                name,
            } => write!(
                formatter,
                "duplicate owner-local `{element}` name `{name}` under `{owner}`"
            ),
            Self::UnsupportedPackageWrapper(wrapper) => {
                write!(formatter, "unsupported XDTO package wrapper `{wrapper}`")
            }
            Self::InvalidPackageReference(value) => {
                write!(formatter, "invalid XDTO package reference `{value}`")
            }
            Self::ConflictingTypeWrapper {
                context,
                expected,
                actual,
            } => write!(
                formatter,
                "`{context}` requires `{expected}`, found `{actual}`"
            ),
            Self::InvalidBoolean { field, value } => {
                write!(formatter, "invalid Boolean `{value}` in `{field}`")
            }
            Self::UnsupportedDirection(direction) => {
                write!(formatter, "unsupported transfer direction `{direction}`")
            }
        }
    }
}

impl std::error::Error for EdtServiceDescriptorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_graph::WebServiceParameterDirection;
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{TempDir, tempdir};

    use super::{
        EdtServiceDescriptorError, EdtServiceDescriptorReader, EdtWebServiceXdtoPackage,
        FileSystemEdtServiceDescriptorReader,
    };
    use crate::{EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader};

    const HTTP_UUID: &str = "http-service-id";
    const HTTP_NAME: &str = "HttpService";
    const WEB_UUID: &str = "web-service-id";
    const WEB_NAME: &str = "WebService";
    const XDTO_INTERNAL_NAMESPACE: &str = "http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange";

    fn http_xml(children: &str) -> String {
        format!(
            r#"<mdclass:HTTPService xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{HTTP_UUID}"><name>{HTTP_NAME}</name>{children}</mdclass:HTTPService>"#
        )
    }

    fn web_xml(children: &str) -> String {
        format!(
            r#"<mdclass:WebService xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://g5.1c.ru/v8/dt/mcore" xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{WEB_UUID}"><name>{WEB_NAME}</name>{children}</mdclass:WebService>"#
        )
    }

    fn type_xml(wrapper: &str, name: &str, namespace: &str) -> String {
        format!(r"<{wrapper}><name>{name}</name><nsUri>{namespace}</nsUri></{wrapper}>")
    }

    fn service_directory(kind: MetadataKind, xml: &str) -> TempDir {
        let directory = tempdir().expect("temporary service directory must be created");
        let name = match kind {
            MetadataKind::HttpService => HTTP_NAME,
            MetadataKind::WebService => WEB_NAME,
            other => panic!("unsupported fixture kind: {other:?}"),
        };
        fs::write(directory.path().join(format!("{name}.mdo")), xml)
            .expect("service descriptor fixture must be written");
        directory
    }

    fn metadata(directory: &Path, kind: MetadataKind) -> crate::EdtMetadataObjectDescriptor {
        FileSystemEdtMetadataObjectReader
            .read(directory, kind)
            .expect("generated service metadata must parse")
    }

    fn descriptor_path(directory: &Path, kind: MetadataKind) -> PathBuf {
        directory.join(match kind {
            MetadataKind::HttpService => format!("{HTTP_NAME}.mdo"),
            MetadataKind::WebService => format!("{WEB_NAME}.mdo"),
            other => panic!("unsupported fixture kind: {other:?}"),
        })
    }

    fn generated_http() -> TempDir {
        service_directory(
            MetadataKind::HttpService,
            &http_xml(
                r#"
                <rootURL>api</rootURL>
                <urlTemplates uuid="url-z"><name>Zulu</name><template>/z/{id}</template>
                    <methods uuid="method-z"><name>POST</name><httpMethod>POST</httpMethod><handler>HandlePost</handler></methods>
                </urlTemplates>
                <urlTemplates uuid="url-a"><name>Alpha</name><template>/a</template>
                    <methods uuid="method-a"><name>GET</name><handler>HandleGet</handler></methods>
                </urlTemplates>
                "#,
            ),
        )
    }

    fn generated_web(package: &str, operation_children: &str) -> TempDir {
        service_directory(
            MetadataKind::WebService,
            &web_xml(&format!(
                r#"<namespace>urn:web</namespace>{package}<operations uuid="operation-z"><name>Zulu</name>{}<procedureName>HandleZulu</procedureName>{operation_children}</operations><operations uuid="operation-a"><name>Alpha</name>{}<nillable>false</nillable><procedureName>HandleAlpha</procedureName></operations>"#,
                type_xml(
                    "xdtoReturningValueType",
                    "string",
                    "http://www.w3.org/2001/XMLSchema"
                ),
                type_xml(
                    "xdtoReturningValueType",
                    "boolean",
                    "http://www.w3.org/2001/XMLSchema"
                ),
            )),
        )
    }

    #[test]
    fn parses_http_children_optional_method_and_unresolved_handlers_canonically() {
        let directory = generated_http();
        let descriptor = metadata(directory.path(), MetadataKind::HttpService);
        let service = FileSystemEdtServiceDescriptorReader
            .read_http(&descriptor)
            .expect("generated HTTP Service must parse");

        assert_eq!(service.metadata(), &descriptor);
        assert_eq!(service.root_url(), "api");
        assert_eq!(service.url_templates().len(), 2);
        assert_eq!(service.url_templates()[0].id().as_str(), "url-a");
        assert_eq!(service.url_templates()[0].name().as_str(), "Alpha");
        assert_eq!(service.url_templates()[0].template(), "/a");
        assert_eq!(service.url_templates()[0].methods().len(), 1);
        assert_eq!(service.url_templates()[0].methods()[0].http_method(), None);
        assert_eq!(
            service.url_templates()[0].methods()[0].handler().as_str(),
            "HandleGet"
        );
        assert_eq!(
            service.url_templates()[1].methods()[0]
                .http_method()
                .map(oneagent_common::EntityName::as_str),
            Some("POST")
        );
    }

    #[test]
    fn parses_web_package_types_optionals_directions_and_handlers_canonically() {
        let package = r#"<xdtoPackages xsi:type="core:StringValue"><value>urn:external:z</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.Exchange</value></xdtoPackages><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.Exchange</value></xdtoPackages><xdtoPackages xsi:type="core:StringValue"><value>urn:external:a</value></xdtoPackages>"#;
        let parameters = format!(
            r#"<parameters uuid="parameter-z"><name>Zulu</name>{}<nillable>true</nillable><transferDirection>Out</transferDirection></parameters><parameters uuid="parameter-a"><name>Alpha</name>{}<transferDirection>InOut</transferDirection></parameters>"#,
            type_xml("xdtoValueType", "string", "urn:external"),
            type_xml("xdtoValueType", "Result", XDTO_INTERNAL_NAMESPACE),
        );
        let directory = generated_web(package, &parameters);
        let descriptor = metadata(directory.path(), MetadataKind::WebService);
        let service = FileSystemEdtServiceDescriptorReader
            .read_web(&descriptor)
            .expect("generated Web Service must parse");

        assert_eq!(service.metadata(), &descriptor);
        assert_eq!(service.namespace(), "urn:web");
        assert_eq!(service.xdto_packages().len(), 3);
        assert!(matches!(
            &service.xdto_packages()[0],
            EdtWebServiceXdtoPackage::Repository(name) if name.as_str() == "Exchange"
        ));
        assert_eq!(
            &service.xdto_packages()[1],
            &EdtWebServiceXdtoPackage::ExternalNamespace("urn:external:a".to_owned())
        );
        assert_eq!(
            &service.xdto_packages()[2],
            &EdtWebServiceXdtoPackage::ExternalNamespace("urn:external:z".to_owned())
        );
        assert_eq!(service.operations().len(), 2);
        assert_eq!(service.operations()[0].id().as_str(), "operation-a");
        assert_eq!(service.operations()[0].nillable(), Some(false));
        assert_eq!(
            service.operations()[1].procedure_name().as_str(),
            "HandleZulu"
        );
        assert_eq!(service.operations()[1].parameters().len(), 2);
        assert_eq!(
            service.operations()[1].parameters()[0]
                .value_type()
                .namespace(),
            XDTO_INTERNAL_NAMESPACE
        );
        assert_eq!(
            service.operations()[1].parameters()[0].direction(),
            Some(WebServiceParameterDirection::InOut)
        );
        assert_eq!(
            service.operations()[1].parameters()[1].direction(),
            Some(WebServiceParameterDirection::Out)
        );
        assert_eq!(
            service.operations()[1].parameters()[1].nillable(),
            Some(true)
        );
    }

    #[test]
    fn sibling_reordering_and_repeated_reads_are_equal() {
        let directory = generated_http();
        let descriptor = metadata(directory.path(), MetadataKind::HttpService);
        let first = FileSystemEdtServiceDescriptorReader
            .read_http(&descriptor)
            .expect("first HTTP ordering must parse");
        let repeated = FileSystemEdtServiceDescriptorReader
            .read_http(&descriptor)
            .expect("repeated HTTP read must parse");
        let reordered = http_xml(
            r#"<urlTemplates uuid="url-a"><template>/a</template><methods uuid="method-a"><handler>HandleGet</handler><name>GET</name></methods><name>Alpha</name></urlTemplates><rootURL>api</rootURL><urlTemplates uuid="url-z"><methods uuid="method-z"><handler>HandlePost</handler><httpMethod>POST</httpMethod><name>POST</name></methods><template>/z/{id}</template><name>Zulu</name></urlTemplates>"#,
        );
        fs::write(
            descriptor_path(directory.path(), MetadataKind::HttpService),
            reordered,
        )
        .expect("reordered HTTP descriptor must be written");
        let reordered = FileSystemEdtServiceDescriptorReader
            .read_http(&descriptor)
            .expect("reordered HTTP source must parse");
        assert_eq!(first, repeated);
        assert_eq!(first, reordered);
    }

    #[test]
    fn filesystem_root_namespace_and_identity_failures_are_typed() {
        let directory = generated_http();
        let descriptor = metadata(directory.path(), MetadataKind::HttpService);
        fs::write(descriptor.descriptor_path(), [0xff_u8])
            .expect("invalid UTF-8 input must be written");
        assert!(matches!(
            FileSystemEdtServiceDescriptorReader.read_http(&descriptor),
            Err(EdtServiceDescriptorError::ReadFile { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidData
        ));

        let cases = [
            ("malformed", "<mdclass:HTTPService", 0_u8),
            ("missing root", "<?xml version=\"1.0\"?>", 1),
            (
                "wrong root",
                r#"<mdclass:WebService xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"/>"#,
                2,
            ),
            (
                "wrong namespace",
                r#"<mdclass:HTTPService xmlns:mdclass="urn:wrong"/>"#,
                3,
            ),
        ];
        for (label, xml, expected) in cases {
            let directory = generated_http();
            let descriptor = metadata(directory.path(), MetadataKind::HttpService);
            fs::write(descriptor.descriptor_path(), xml).expect("invalid source must be written");
            let error = FileSystemEdtServiceDescriptorReader
                .read_http(&descriptor)
                .unwrap_err();
            let actual = match error {
                EdtServiceDescriptorError::MalformedXml(_) => 0,
                EdtServiceDescriptorError::MissingRoot => 1,
                EdtServiceDescriptorError::UnexpectedRoot(_) => 2,
                EdtServiceDescriptorError::UnsupportedNamespace(_) => 3,
                other => panic!("unexpected root error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }

        let mismatch = generated_http();
        let descriptor = metadata(mismatch.path(), MetadataKind::HttpService);
        fs::write(
            descriptor.descriptor_path(),
            http_xml("<name>Different</name><rootURL>api</rootURL>"),
        )
        .expect("mismatched source must be written");
        assert!(matches!(
            FileSystemEdtServiceDescriptorReader.read_http(&descriptor),
            Err(
                EdtServiceDescriptorError::DuplicateField { field: "name", .. }
                    | EdtServiceDescriptorError::DescriptorIdentityMismatch { .. }
            )
        ));

        let web = generated_web("", "");
        let web_descriptor = metadata(web.path(), MetadataKind::WebService);
        assert!(matches!(
            FileSystemEdtServiceDescriptorReader.read_http(&web_descriptor),
            Err(EdtServiceDescriptorError::UnexpectedMetadataKind { .. })
        ));
    }

    #[test]
    fn required_duplicate_uuid_name_and_hierarchy_failures_are_typed() {
        let cases = [
            ("missing root URL", http_xml(""), 0_u8),
            (
                "empty handler",
                http_xml(
                    r#"<rootURL>api</rootURL><urlTemplates uuid="u"><name>U</name><template>/u</template><methods uuid="m"><name>GET</name><handler/></methods></urlTemplates>"#,
                ),
                1,
            ),
            (
                "duplicate UUID",
                http_xml(
                    r#"<rootURL>api</rootURL><urlTemplates uuid="same"><name>A</name><template>/a</template><methods uuid="same"><name>GET</name><handler>H</handler></methods></urlTemplates>"#,
                ),
                2,
            ),
            (
                "duplicate name",
                http_xml(
                    r#"<rootURL>api</rootURL><urlTemplates uuid="a"><name>Same</name><template>/a</template></urlTemplates><urlTemplates uuid="b"><name>Same</name><template>/b</template></urlTemplates>"#,
                ),
                3,
            ),
            (
                "wrong hierarchy",
                http_xml(
                    r#"<rootURL>api</rootURL><methods uuid="m"><name>GET</name><handler>H</handler></methods>"#,
                ),
                4,
            ),
        ];
        for (label, xml, expected) in cases {
            let directory = service_directory(MetadataKind::HttpService, &xml);
            let descriptor = metadata(directory.path(), MetadataKind::HttpService);
            let error = FileSystemEdtServiceDescriptorReader
                .read_http(&descriptor)
                .unwrap_err();
            let actual = match error {
                EdtServiceDescriptorError::MissingField { .. } => 0,
                EdtServiceDescriptorError::EmptyField { .. } => 1,
                EdtServiceDescriptorError::DuplicateUuid { .. } => 2,
                EdtServiceDescriptorError::DuplicateName { .. } => 3,
                EdtServiceDescriptorError::UnexpectedHierarchy { .. } => 4,
                other => panic!("unexpected HTTP error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }
    }

    #[test]
    fn package_type_boolean_and_direction_failures_are_typed() {
        let base_return = type_xml(
            "xdtoReturningValueType",
            "string",
            "http://www.w3.org/2001/XMLSchema",
        );
        let cases = [
            (
                "unsupported package wrapper",
                web_xml(&format!(
                    r#"<namespace>urn:web</namespace><xdtoPackages xsi:type="core:Other"><value>value</value></xdtoPackages><operations uuid="o"><name>O</name>{base_return}<procedureName>H</procedureName></operations>"#
                )),
                0_u8,
            ),
            (
                "invalid package grammar",
                web_xml(&format!(
                    r#"<namespace>urn:web</namespace><xdtoPackages xsi:type="core:ReferenceValue"><value>XDTOPackage.A.B</value></xdtoPackages><operations uuid="o"><name>O</name>{base_return}<procedureName>H</procedureName></operations>"#
                )),
                1,
            ),
            (
                "conflicting type wrapper",
                web_xml(&format!(
                    r#"<namespace>urn:web</namespace><operations uuid="o"><name>O</name>{}<procedureName>H</procedureName></operations>"#,
                    type_xml("xdtoValueType", "string", "urn:type")
                )),
                2,
            ),
            (
                "invalid Boolean",
                web_xml(&format!(
                    r#"<namespace>urn:web</namespace><operations uuid="o"><name>O</name>{base_return}<nillable>yes</nillable><procedureName>H</procedureName></operations>"#
                )),
                3,
            ),
            (
                "unsupported direction",
                web_xml(&format!(
                    r#"<namespace>urn:web</namespace><operations uuid="o"><name>O</name>{base_return}<procedureName>H</procedureName><parameters uuid="p"><name>P</name>{}<transferDirection>In</transferDirection></parameters></operations>"#,
                    type_xml("xdtoValueType", "string", "urn:type")
                )),
                4,
            ),
        ];
        for (label, xml, expected) in cases {
            let directory = service_directory(MetadataKind::WebService, &xml);
            let descriptor = metadata(directory.path(), MetadataKind::WebService);
            let error = FileSystemEdtServiceDescriptorReader
                .read_web(&descriptor)
                .unwrap_err();
            let actual = match error {
                EdtServiceDescriptorError::UnsupportedPackageWrapper(_) => 0,
                EdtServiceDescriptorError::InvalidPackageReference(_) => 1,
                EdtServiceDescriptorError::ConflictingTypeWrapper { .. } => 2,
                EdtServiceDescriptorError::InvalidBoolean { .. } => 3,
                EdtServiceDescriptorError::UnsupportedDirection(_) => 4,
                other => panic!("unexpected Web error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn all_live_http_and_web_services_match_the_accepted_contract() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OneAgent_EDTproject/src");
        let mut http_directories = fs::read_dir(source.join("HTTPServices"))
            .expect("live HTTPServices directory must be readable")
            .map(|entry| {
                entry
                    .expect("live HTTP Service entry must be readable")
                    .path()
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        http_directories.sort();
        assert_eq!(http_directories.len(), 2);

        let mut url_templates = 0_usize;
        let mut methods = 0_usize;
        let mut explicit_http_methods = 0_usize;
        let mut handlers = 0_usize;
        for directory in &http_directories {
            let metadata = FileSystemEdtMetadataObjectReader
                .read(directory, MetadataKind::HttpService)
                .expect("live HTTP metadata must parse");
            let first = FileSystemEdtServiceDescriptorReader
                .read_http(&metadata)
                .expect("live HTTP Service must parse");
            let repeated = FileSystemEdtServiceDescriptorReader
                .read_http(&metadata)
                .expect("repeated live HTTP Service read must parse");
            assert_eq!(first, repeated);
            url_templates += first.url_templates().len();
            for method in first
                .url_templates()
                .iter()
                .flat_map(super::EdtHttpUrlTemplate::methods)
            {
                methods += 1;
                handlers += usize::from(!method.handler().as_str().is_empty());
                explicit_http_methods += usize::from(method.http_method().is_some());
            }
        }
        assert_eq!(url_templates, 35);
        assert_eq!(methods, 35);
        assert_eq!(explicit_http_methods, 11);

        let mut web_directories = fs::read_dir(source.join("WebServices"))
            .expect("live WebServices directory must be readable")
            .map(|entry| {
                entry
                    .expect("live Web Service entry must be readable")
                    .path()
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        web_directories.sort();
        assert_eq!(web_directories.len(), 8);

        let mut operations = 0_usize;
        let mut parameters = 0_usize;
        let mut repository_packages = 0_usize;
        let mut external_packages = 0_usize;
        let mut absent_packages = 0_usize;
        let mut type_occurrences = 0_usize;
        let mut internal_types = 0_usize;
        let mut explicit_nillable = 0_usize;
        let mut out_directions = 0_usize;
        let mut in_out_directions = 0_usize;
        for directory in &web_directories {
            let metadata = FileSystemEdtMetadataObjectReader
                .read(directory, MetadataKind::WebService)
                .expect("live Web metadata must parse");
            let first = FileSystemEdtServiceDescriptorReader
                .read_web(&metadata)
                .expect("live Web Service must parse");
            let repeated = FileSystemEdtServiceDescriptorReader
                .read_web(&metadata)
                .expect("repeated live Web Service read must parse");
            assert_eq!(first, repeated);
            if first.xdto_packages().is_empty() {
                absent_packages += 1;
            }
            for package in first.xdto_packages() {
                match package {
                    EdtWebServiceXdtoPackage::Repository(name) => {
                        repository_packages += 1;
                        assert_eq!(name.as_str(), "EnterpriseDataExchange_1_0_1_1");
                    }
                    EdtWebServiceXdtoPackage::ExternalNamespace(namespace) => {
                        external_packages += 1;
                        assert_eq!(namespace, "http://v8.1c.ru/8.1/data/core");
                    }
                }
            }
            operations += first.operations().len();
            for operation in first.operations() {
                handlers += 1;
                type_occurrences += 1;
                internal_types +=
                    usize::from(operation.returning_type().namespace() == XDTO_INTERNAL_NAMESPACE);
                explicit_nillable += usize::from(operation.nillable().is_some());
                parameters += operation.parameters().len();
                for parameter in operation.parameters() {
                    type_occurrences += 1;
                    internal_types +=
                        usize::from(parameter.value_type().namespace() == XDTO_INTERNAL_NAMESPACE);
                    explicit_nillable += usize::from(parameter.nillable().is_some());
                    match parameter.direction() {
                        Some(WebServiceParameterDirection::Out) => out_directions += 1,
                        Some(WebServiceParameterDirection::InOut) => in_out_directions += 1,
                        None => {}
                    }
                }
            }
        }
        assert_eq!(operations, 119);
        assert_eq!(parameters, 360);
        assert_eq!(handlers, 154);
        assert_eq!(repository_packages, 2);
        assert_eq!(external_packages, 5);
        assert_eq!(absent_packages, 1);
        assert_eq!(type_occurrences, 479);
        assert_eq!(internal_types, 1);
        assert_eq!(type_occurrences - internal_types, 478);
        assert_eq!(explicit_nillable, 180);
        assert_eq!(out_directions, 41);
        assert_eq!(in_out_directions, 49);
    }
}
