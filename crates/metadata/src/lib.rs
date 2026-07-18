//! Semantic model of `1C:Enterprise` metadata.

use oneagent_common::{EntityId, EntityName};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

/// Supported kinds of `1C:Enterprise` metadata objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetadataKind {
    /// Configuration root.
    Configuration,
    /// Subsystem.
    Subsystem,
    /// Catalog.
    Catalog,
    /// Document.
    Document,
    /// Enumeration.
    Enumeration,
    /// Common module.
    CommonModule,
    /// Report.
    Report,
    /// Data processor.
    DataProcessor,
    /// Information register.
    InformationRegister,
    /// Accumulation register.
    AccumulationRegister,
    /// Accounting register.
    AccountingRegister,
    /// Calculation register.
    CalculationRegister,
    /// Business process.
    BusinessProcess,
    /// Task.
    Task,
    /// Role.
    Role,
    /// Common form.
    CommonForm,
    /// Managed form.
    Form,
    /// Command.
    Command,
    /// Template.
    Template,
    /// HTTP service.
    HttpService,
    /// Web service.
    WebService,
    /// `XDTO` package.
    XdtoPackage,
    /// Unknown or not-yet-supported metadata kind.
    Unknown,
}

impl MetadataKind {
    /// Returns a stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Configuration => "configuration",
            Self::Subsystem => "subsystem",
            Self::Catalog => "catalog",
            Self::Document => "document",
            Self::Enumeration => "enumeration",
            Self::CommonModule => "common_module",
            Self::Report => "report",
            Self::DataProcessor => "data_processor",
            Self::InformationRegister => "information_register",
            Self::AccumulationRegister => "accumulation_register",
            Self::AccountingRegister => "accounting_register",
            Self::CalculationRegister => "calculation_register",
            Self::BusinessProcess => "business_process",
            Self::Task => "task",
            Self::Role => "role",
            Self::CommonForm => "common_form",
            Self::Form => "form",
            Self::Command => "command",
            Self::Template => "template",
            Self::HttpService => "http_service",
            Self::WebService => "web_service",
            Self::XdtoPackage => "xdto_package",
            Self::Unknown => "unknown",
        }
    }
}

impl Display for MetadataKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A semantic metadata object independent from its source file format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataObject {
    id: EntityId,
    name: EntityName,
    kind: MetadataKind,
    parent_id: Option<EntityId>,
}

impl MetadataObject {
    /// Creates a metadata object.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        kind: MetadataKind,
        parent_id: Option<EntityId>,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            parent_id,
        }
    }

    /// Returns the stable object identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the object name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the metadata kind.
    #[must_use]
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }

    /// Returns the parent object identifier.
    #[must_use]
    pub const fn parent_id(&self) -> Option<&EntityId> {
        self.parent_id.as_ref()
    }
}

/// In-memory semantic tree of configuration metadata.
#[derive(Debug, Default, Clone)]
pub struct MetadataTree {
    objects: BTreeMap<EntityId, MetadataObject>,
}

impl MetadataTree {
    /// Creates an empty tree.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            objects: BTreeMap::new(),
        }
    }

    /// Inserts an object into the tree.
    ///
    /// Returns the previous object when the identifier already existed.
    pub fn insert(&mut self, object: MetadataObject) -> Option<MetadataObject> {
        self.objects.insert(object.id().clone(), object)
    }

    /// Returns an object by identifier.
    #[must_use]
    pub fn get(&self, id: &EntityId) -> Option<&MetadataObject> {
        self.objects.get(id)
    }

    /// Returns all direct children of a parent.
    #[must_use]
    pub fn children_of(&self, parent_id: &EntityId) -> Vec<&MetadataObject> {
        self.objects
            .values()
            .filter(|object| object.parent_id() == Some(parent_id))
            .collect()
    }

    /// Returns all objects of a specified kind.
    #[must_use]
    pub fn objects_by_kind(&self, kind: MetadataKind) -> Vec<&MetadataObject> {
        self.objects
            .values()
            .filter(|object| object.kind() == kind)
            .collect()
    }

    /// Returns the number of indexed objects.
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Returns `true` when the tree contains no objects.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{MetadataKind, MetadataObject, MetadataTree};

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn tree_returns_children_by_parent() {
        let root_id = id("configuration");
        let mut tree = MetadataTree::new();

        tree.insert(MetadataObject::new(
            root_id.clone(),
            name("MainConfiguration"),
            MetadataKind::Configuration,
            None,
        ));
        tree.insert(MetadataObject::new(
            id("document.sales"),
            name("Sales"),
            MetadataKind::Document,
            Some(root_id.clone()),
        ));

        let children = tree.children_of(&root_id);

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name().as_str(), "Sales");
    }

    #[test]
    fn tree_filters_objects_by_kind() {
        let mut tree = MetadataTree::new();

        tree.insert(MetadataObject::new(
            id("catalog.products"),
            name("Products"),
            MetadataKind::Catalog,
            None,
        ));
        tree.insert(MetadataObject::new(
            id("document.sales"),
            name("Sales"),
            MetadataKind::Document,
            None,
        ));

        assert_eq!(tree.objects_by_kind(MetadataKind::Document).len(), 1);
    }
}
