//! Source-independent XDTO and service graph content and identities.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

/// Direct XDTO type family declared by one package schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum XdtoTypeKind {
    /// Direct XDTO Value type.
    Value,
    /// Direct XDTO Object type.
    Object,
}

impl XdtoTypeKind {
    /// Returns the stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Object => "object",
        }
    }
}

/// Source-independent content of one direct XDTO type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XdtoTypePayload {
    kind: XdtoTypeKind,
}

impl XdtoTypePayload {
    /// Creates direct XDTO type content.
    #[must_use]
    pub const fn new(kind: XdtoTypeKind) -> Self {
        Self { kind }
    }

    /// Returns the direct XDTO type family.
    #[must_use]
    pub const fn kind(self) -> XdtoTypeKind {
        self.kind
    }
}

/// Exact XDTO type declaration used by a Web Service operation or parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XdtoTypeReference {
    namespace: String,
    name: EntityName,
}

impl XdtoTypeReference {
    /// Creates an exact namespace and local-name type declaration.
    #[must_use]
    pub fn new(namespace: impl Into<String>, name: EntityName) -> Self {
        Self {
            namespace: namespace.into(),
            name,
        }
    }

    /// Returns the exact declared namespace URI.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the exact declared local type name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }
}

/// Source-independent content of one HTTP Service URL Template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpServiceUrlTemplatePayload {
    template: String,
}

impl HttpServiceUrlTemplatePayload {
    /// Creates URL Template content from exact decoded template text.
    #[must_use]
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Returns exact decoded template text.
    #[must_use]
    pub fn template(&self) -> &str {
        &self.template
    }
}

/// Source-independent content of one HTTP Service Method.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpServiceMethodPayload {
    http_method: Option<EntityName>,
}

impl HttpServiceMethodPayload {
    /// Creates Method content while preserving absence of an explicit verb.
    #[must_use]
    pub const fn new(http_method: Option<EntityName>) -> Self {
        Self { http_method }
    }

    /// Returns the explicit HTTP method token, when declared.
    #[must_use]
    pub const fn http_method(&self) -> Option<&EntityName> {
        self.http_method.as_ref()
    }
}

/// Source-independent content of one Web Service Operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebServiceOperationPayload {
    returning_type: XdtoTypeReference,
    nillable: Option<bool>,
}

impl WebServiceOperationPayload {
    /// Creates Operation content from its return type and explicit nillability.
    #[must_use]
    pub const fn new(returning_type: XdtoTypeReference, nillable: Option<bool>) -> Self {
        Self {
            returning_type,
            nillable,
        }
    }

    /// Returns the declared return type.
    #[must_use]
    pub const fn returning_type(&self) -> &XdtoTypeReference {
        &self.returning_type
    }

    /// Returns explicitly declared nillability, preserving absence.
    #[must_use]
    pub const fn nillable(&self) -> Option<bool> {
        self.nillable
    }
}

/// Explicit transfer direction accepted for a Web Service Parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WebServiceParameterDirection {
    /// Output-only parameter.
    Out,
    /// Input/output parameter.
    InOut,
}

impl WebServiceParameterDirection {
    /// Returns the stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::InOut => "in_out",
        }
    }
}

/// Source-independent content of one Web Service Parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebServiceParameterPayload {
    value_type: XdtoTypeReference,
    nillable: Option<bool>,
    direction: Option<WebServiceParameterDirection>,
}

impl WebServiceParameterPayload {
    /// Creates Parameter content while preserving optional declarations.
    #[must_use]
    pub const fn new(
        value_type: XdtoTypeReference,
        nillable: Option<bool>,
        direction: Option<WebServiceParameterDirection>,
    ) -> Self {
        Self {
            value_type,
            nillable,
            direction,
        }
    }

    /// Returns the declared value type.
    #[must_use]
    pub const fn value_type(&self) -> &XdtoTypeReference {
        &self.value_type
    }

    /// Returns explicitly declared nillability, preserving absence.
    #[must_use]
    pub const fn nillable(&self) -> Option<bool> {
        self.nillable
    }

    /// Returns explicitly declared transfer direction, preserving absence.
    #[must_use]
    pub const fn direction(&self) -> Option<WebServiceParameterDirection> {
        self.direction
    }
}

/// Builds the stable owner-scoped identity of one direct XDTO type.
///
/// # Errors
///
/// Returns [`XdtoTypeIdentityError`] if the derived identifier cannot be
/// represented by the common domain primitive.
pub fn xdto_type_id(
    package_id: &EntityId,
    type_name: &EntityName,
) -> Result<EntityId, XdtoTypeIdentityError> {
    EntityId::new(format!(
        "xdto_type;owner#{}:{};name#{}:{}",
        package_id.as_str().len(),
        package_id.as_str(),
        type_name.as_str().len(),
        type_name.as_str(),
    ))
    .map_err(|_| XdtoTypeIdentityError)
}

/// Error produced when a direct XDTO type identity cannot be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XdtoTypeIdentityError;

impl Display for XdtoTypeIdentityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid XDTO type identifier")
    }
}

impl std::error::Error for XdtoTypeIdentityError {}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{
        HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, WebServiceOperationPayload,
        WebServiceParameterDirection, WebServiceParameterPayload, XdtoTypeKind, XdtoTypePayload,
        XdtoTypeReference, xdto_type_id,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn typed_payloads_preserve_exact_optional_content() {
        let type_reference = XdtoTypeReference::new("urn:package", name("Result"));
        let operation = WebServiceOperationPayload::new(type_reference.clone(), Some(true));
        let parameter = WebServiceParameterPayload::new(
            type_reference.clone(),
            None,
            Some(WebServiceParameterDirection::InOut),
        );

        assert_eq!(
            XdtoTypePayload::new(XdtoTypeKind::Object).kind(),
            XdtoTypeKind::Object
        );
        assert_eq!(XdtoTypeKind::Value.as_str(), "value");
        assert_eq!(type_reference.namespace(), "urn:package");
        assert_eq!(operation.returning_type(), &type_reference);
        assert_eq!(operation.nillable(), Some(true));
        assert_eq!(parameter.value_type(), &type_reference);
        assert_eq!(parameter.nillable(), None);
        assert_eq!(
            parameter.direction(),
            Some(WebServiceParameterDirection::InOut)
        );
        assert_eq!(WebServiceParameterDirection::Out.as_str(), "out");
        assert_eq!(
            HttpServiceUrlTemplatePayload::new("/{id}").template(),
            "/{id}"
        );
        assert_eq!(HttpServiceMethodPayload::new(None).http_method(), None);
    }

    #[test]
    fn xdto_type_identity_is_stable_collision_safe_and_kind_independent() {
        let package = id("package:a:b");
        let first = xdto_type_id(&package, &name("c:d")).expect("identity must be valid");
        let repeated = xdto_type_id(&package, &name("c:d")).expect("identity must be valid");
        let concatenated =
            xdto_type_id(&id("package:a"), &name("b:c:d")).expect("identity must be valid");

        assert_eq!(first, repeated);
        assert_ne!(first, concatenated);
        assert!(first.as_str().contains("owner#11:package:a:b"));
        assert!(first.as_str().contains("name#3:c:d"));
        assert_eq!(
            XdtoTypePayload::new(XdtoTypeKind::Value),
            XdtoTypePayload::new(XdtoTypeKind::Value),
        );
    }
}
