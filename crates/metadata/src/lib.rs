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
    /// Event subscription.
    EventSubscription,
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
            Self::EventSubscription => "event_subscription",
            Self::Unknown => "unknown",
        }
    }
}

impl Display for MetadataKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Source-independent semantic content shared by all metadata kinds.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MetadataPayload {
    common: CommonMetadataPayload,
    specific: Option<MetadataSpecificPayload>,
}

impl MetadataPayload {
    /// Creates metadata payload from common and optional kind-specific content.
    #[must_use]
    pub const fn new(
        common: CommonMetadataPayload,
        specific: Option<MetadataSpecificPayload>,
    ) -> Self {
        Self { common, specific }
    }

    /// Creates metadata payload with no accepted semantic content.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(CommonMetadataPayload::empty(), None)
    }

    /// Returns content shared by all metadata kinds.
    #[must_use]
    pub const fn common(&self) -> &CommonMetadataPayload {
        &self.common
    }

    /// Returns kind-specific metadata content when present.
    #[must_use]
    pub const fn specific(&self) -> Option<&MetadataSpecificPayload> {
        self.specific.as_ref()
    }

    /// Returns whether this payload is compatible with a metadata kind.
    #[must_use]
    pub const fn is_compatible_with(&self, kind: MetadataKind) -> bool {
        match self.specific {
            None => true,
            Some(MetadataSpecificPayload::Document(_)) => matches!(kind, MetadataKind::Document),
            Some(MetadataSpecificPayload::EventSubscription(_)) => {
                matches!(kind, MetadataKind::EventSubscription)
            }
        }
    }
}

/// Semantic content shared by all metadata kinds.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommonMetadataPayload {
    synonym: Option<String>,
}

impl CommonMetadataPayload {
    /// Creates common metadata content with an optional localized synonym.
    #[must_use]
    pub const fn new(synonym: Option<String>) -> Self {
        Self { synonym }
    }

    /// Creates common metadata content with no synonym.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(None)
    }

    /// Returns the explicitly declared localized synonym when present.
    #[must_use]
    pub fn synonym(&self) -> Option<&str> {
        self.synonym.as_deref()
    }
}

/// Source-independent semantic content of a subordinate metadata member.
///
/// The first accepted member-content slice is shared by `Attribute` and
/// `TabularSection` graph nodes. Compatibility with those node kinds is
/// enforced by the graph domain rather than this source-independent value.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MetadataMemberPayload {
    synonym: Option<String>,
}

impl MetadataMemberPayload {
    /// Creates member content with an optional localized display synonym.
    #[must_use]
    pub const fn new(synonym: Option<String>) -> Self {
        Self { synonym }
    }

    /// Creates member content with no accepted semantic value.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(None)
    }

    /// Returns the explicitly declared localized synonym when present.
    #[must_use]
    pub fn synonym(&self) -> Option<&str> {
        self.synonym.as_deref()
    }
}

/// Closed kind-specific metadata content supported by the domain model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataSpecificPayload {
    /// Content intrinsic to a Document metadata object.
    Document(DocumentMetadataPayload),
    /// Content intrinsic to an Event Subscription metadata object.
    EventSubscription(EventSubscriptionMetadataPayload),
}

/// Source-independent Event Subscription content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSubscriptionMetadataPayload {
    event: EntityName,
}

impl EventSubscriptionMetadataPayload {
    /// Creates Event Subscription content from its declared event name.
    #[must_use]
    pub const fn new(event: EntityName) -> Self {
        Self { event }
    }

    /// Returns the declared event name.
    #[must_use]
    pub const fn event(&self) -> &EntityName {
        &self.event
    }
}

/// Source-independent Document content.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DocumentMetadataPayload {
    register_records: Vec<MetadataRegisterRecord>,
}

impl DocumentMetadataPayload {
    /// Creates canonical Document content from declared register-record targets.
    #[must_use]
    pub fn new(register_records: impl IntoIterator<Item = MetadataRegisterRecord>) -> Self {
        let mut register_records = register_records.into_iter().collect::<Vec<_>>();
        register_records.sort();
        register_records.dedup();
        Self { register_records }
    }

    /// Returns declared register-record targets in canonical order.
    #[must_use]
    pub fn register_records(&self) -> &[MetadataRegisterRecord] {
        &self.register_records
    }
}

/// Canonical target declared by a Document register-record entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetadataRegisterRecord {
    target_kind: MetadataKind,
    target_name: EntityName,
}

impl MetadataRegisterRecord {
    /// Creates a declared register-record target.
    #[must_use]
    pub const fn new(target_kind: MetadataKind, target_name: EntityName) -> Self {
        Self {
            target_kind,
            target_name,
        }
    }

    /// Returns the declared target metadata kind.
    #[must_use]
    pub const fn target_kind(&self) -> MetadataKind {
        self.target_kind
    }

    /// Returns the declared canonical target name.
    #[must_use]
    pub const fn target_name(&self) -> &EntityName {
        &self.target_name
    }
}

/// Error returned when metadata object content conflicts with its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataObjectPayloadError {
    kind: MetadataKind,
}

impl MetadataObjectPayloadError {
    /// Returns the metadata kind rejected by payload validation.
    #[must_use]
    pub const fn kind(self) -> MetadataKind {
        self.kind
    }
}

impl Display for MetadataObjectPayloadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "metadata payload is incompatible with {} metadata kind",
            self.kind
        )
    }
}

impl std::error::Error for MetadataObjectPayloadError {}

/// A semantic metadata object independent from its source file format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataObject {
    id: EntityId,
    name: EntityName,
    kind: MetadataKind,
    parent_id: Option<EntityId>,
    payload: MetadataPayload,
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
            payload: MetadataPayload::empty(),
        }
    }

    /// Creates a metadata object with explicit semantic payload.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataObjectPayloadError`] when kind-specific content does
    /// not match `kind`.
    pub fn new_with_payload(
        id: EntityId,
        name: EntityName,
        kind: MetadataKind,
        parent_id: Option<EntityId>,
        payload: MetadataPayload,
    ) -> Result<Self, MetadataObjectPayloadError> {
        if !payload.is_compatible_with(kind) {
            return Err(MetadataObjectPayloadError { kind });
        }

        Ok(Self {
            id,
            name,
            kind,
            parent_id,
            payload,
        })
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

    /// Returns source-independent semantic metadata content.
    #[must_use]
    pub const fn payload(&self) -> &MetadataPayload {
        &self.payload
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

    use super::{
        CommonMetadataPayload, DocumentMetadataPayload, EventSubscriptionMetadataPayload,
        MetadataKind, MetadataMemberPayload, MetadataObject, MetadataPayload,
        MetadataRegisterRecord, MetadataSpecificPayload, MetadataTree,
    };

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

    #[test]
    fn compatibility_constructor_creates_empty_payload() {
        let object = MetadataObject::new(
            id("catalog.products"),
            name("Products"),
            MetadataKind::Catalog,
            None,
        );

        assert_eq!(object.payload(), &MetadataPayload::empty());
        assert_eq!(object.payload().common().synonym(), None);
        assert_eq!(object.payload().specific(), None);
    }

    #[test]
    fn common_payload_preserves_absent_and_explicit_synonym() {
        let absent = CommonMetadataPayload::new(None);
        let present = CommonMetadataPayload::new(Some("Продажи".to_owned()));

        assert_eq!(absent.synonym(), None);
        assert_eq!(present.synonym(), Some("Продажи"));
    }

    #[test]
    fn member_payload_preserves_absent_and_explicit_synonym() {
        let absent = MetadataMemberPayload::empty();
        let present = MetadataMemberPayload::new(Some("Товары".to_owned()));

        assert_eq!(absent.synonym(), None);
        assert_eq!(present.synonym(), Some("Товары"));
        assert_ne!(absent, present);
    }

    #[test]
    fn document_payload_is_sorted_and_deduplicated() {
        let accumulation =
            MetadataRegisterRecord::new(MetadataKind::AccumulationRegister, name("Stock"));
        let information =
            MetadataRegisterRecord::new(MetadataKind::InformationRegister, name("Prices"));

        let payload =
            DocumentMetadataPayload::new([information.clone(), accumulation.clone(), information]);

        assert_eq!(
            payload.register_records(),
            &[
                MetadataRegisterRecord::new(MetadataKind::InformationRegister, name("Prices")),
                accumulation,
            ]
        );
    }

    #[test]
    fn document_payload_equality_is_independent_of_input_order() {
        let accumulation =
            MetadataRegisterRecord::new(MetadataKind::AccumulationRegister, name("Stock"));
        let accounting =
            MetadataRegisterRecord::new(MetadataKind::AccountingRegister, name("Ledger"));

        assert_eq!(
            DocumentMetadataPayload::new([accumulation.clone(), accounting.clone()]),
            DocumentMetadataPayload::new([accounting, accumulation]),
        );
    }

    #[test]
    fn metadata_object_equality_includes_payload() {
        let make_object = |synonym: &str| {
            MetadataObject::new_with_payload(
                id("catalog.products"),
                name("Products"),
                MetadataKind::Catalog,
                None,
                MetadataPayload::new(CommonMetadataPayload::new(Some(synonym.to_owned())), None),
            )
            .expect("Catalog common payload must be valid")
        };

        assert_ne!(make_object("Products"), make_object("Goods"));
    }

    #[test]
    fn metadata_object_rejects_kind_specific_payload_mismatch() {
        let payload = MetadataPayload::new(
            CommonMetadataPayload::empty(),
            Some(MetadataSpecificPayload::Document(
                DocumentMetadataPayload::default(),
            )),
        );

        let error = MetadataObject::new_with_payload(
            id("catalog.products"),
            name("Products"),
            MetadataKind::Catalog,
            None,
            payload,
        )
        .expect_err("Document payload on Catalog must be rejected");

        assert_eq!(error.kind(), MetadataKind::Catalog);
        assert_eq!(
            error.to_string(),
            "metadata payload is incompatible with catalog metadata kind"
        );
    }

    #[test]
    fn event_subscription_payload_preserves_event_and_exact_compatibility() {
        let event = name("BeforeWrite");
        let payload = MetadataPayload::new(
            CommonMetadataPayload::empty(),
            Some(MetadataSpecificPayload::EventSubscription(
                EventSubscriptionMetadataPayload::new(event.clone()),
            )),
        );

        let object = MetadataObject::new_with_payload(
            id("event_subscription.before_write"),
            name("BeforeWriteSubscription"),
            MetadataKind::EventSubscription,
            None,
            payload.clone(),
        )
        .expect("Event Subscription payload must be accepted for its metadata kind");
        let Some(MetadataSpecificPayload::EventSubscription(specific)) =
            object.payload().specific()
        else {
            panic!("Event Subscription payload must remain typed");
        };

        assert_eq!(specific.event(), &event);
        assert_eq!(object.payload(), &payload);

        let error = MetadataObject::new_with_payload(
            id("document.sales"),
            name("Sales"),
            MetadataKind::Document,
            None,
            payload,
        )
        .expect_err("Event Subscription payload on Document must be rejected");

        assert_eq!(error.kind(), MetadataKind::Document);
        assert_eq!(
            error.to_string(),
            "metadata payload is incompatible with document metadata kind"
        );
    }
}
