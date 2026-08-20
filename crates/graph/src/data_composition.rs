//! Source-independent Report Data Composition graph content and identities.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

/// Source-independent content of a Report-owned Data Composition Schema.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataCompositionSchemaPayload {
    main: bool,
}

impl DataCompositionSchemaPayload {
    /// Creates Schema content with the owning Report's main-schema role.
    #[must_use]
    pub const fn new(main: bool) -> Self {
        Self { main }
    }

    /// Returns whether the owning Report selects this Schema as its main one.
    #[must_use]
    pub const fn is_main(self) -> bool {
        self.main
    }
}

/// Kind of one direct Data Set declared by a Data Composition Schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataSetKind {
    /// Data Set backed by one complete query declaration.
    Query,
    /// Data Set populated from a runtime object.
    Object,
    /// Data Set combining subordinate source sets.
    Union,
}

impl DataSetKind {
    /// Returns a stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Object => "object",
            Self::Union => "union",
        }
    }
}

/// Source-independent content of one direct Data Set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataSetPayload {
    kind: DataSetKind,
    data_source: Option<EntityName>,
}

impl DataSetPayload {
    /// Creates Data Set content and enforces its direct data-source contract.
    ///
    /// # Errors
    ///
    /// Returns [`DataSetPayloadError`] when a Query or Object Data Set omits
    /// its local data source, or when a Union supplies one.
    pub fn new(
        kind: DataSetKind,
        data_source: Option<EntityName>,
    ) -> Result<Self, DataSetPayloadError> {
        match (kind, data_source.as_ref()) {
            (DataSetKind::Query | DataSetKind::Object, None) => {
                Err(DataSetPayloadError::MissingDataSource)
            }
            (DataSetKind::Union, Some(_)) => Err(DataSetPayloadError::UnexpectedDataSource),
            _ => Ok(Self { kind, data_source }),
        }
    }

    /// Returns the declared Data Set kind.
    #[must_use]
    pub const fn kind(&self) -> DataSetKind {
        self.kind
    }

    /// Returns the accepted direct local data-source name, when applicable.
    #[must_use]
    pub const fn data_source(&self) -> Option<&EntityName> {
        self.data_source.as_ref()
    }
}

/// Invalid Data Set payload contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSetPayloadError {
    /// Query or Object Data Set omitted its required local data source.
    MissingDataSource,
    /// Union Data Set supplied a direct data source.
    UnexpectedDataSource,
}

impl Display for DataSetPayloadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDataSource => {
                formatter.write_str("query or object data set requires a data source")
            }
            Self::UnexpectedDataSource => {
                formatter.write_str("union data set must not declare a direct data source")
            }
        }
    }
}

impl std::error::Error for DataSetPayloadError {}

/// Source-independent content of one direct named Data Composition Field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataCompositionFieldPayload {
    data_path: EntityName,
}

impl DataCompositionFieldPayload {
    /// Creates Field content from its non-empty declared data path.
    #[must_use]
    pub const fn new(data_path: EntityName) -> Self {
        Self { data_path }
    }

    /// Returns the declared data path.
    #[must_use]
    pub const fn data_path(&self) -> &EntityName {
        &self.data_path
    }
}

/// Builds the stable owner-scoped identity of one direct Data Set.
///
/// # Errors
///
/// Returns [`DataCompositionIdentityError`] if the derived identifier cannot
/// be represented by the common domain primitive.
pub fn data_set_id(
    schema_id: &EntityId,
    data_set_name: &EntityName,
) -> Result<EntityId, DataCompositionIdentityError> {
    scoped_identity("data_set", schema_id, "name", data_set_name.as_str())
}

/// Builds the stable owner-scoped identity of one direct Data Composition Field.
///
/// # Errors
///
/// Returns [`DataCompositionIdentityError`] if the derived identifier cannot
/// be represented by the common domain primitive.
pub fn data_composition_field_id(
    data_set_id: &EntityId,
    field_name: &EntityName,
) -> Result<EntityId, DataCompositionIdentityError> {
    scoped_identity(
        "data_composition_field",
        data_set_id,
        "name",
        field_name.as_str(),
    )
}

/// Builds the fixed-role identity of the Query owned by a direct Query Data Set.
///
/// # Errors
///
/// Returns [`DataCompositionIdentityError`] if the derived identifier cannot
/// be represented by the common domain primitive.
pub fn data_set_query_id(data_set_id: &EntityId) -> Result<EntityId, DataCompositionIdentityError> {
    scoped_identity("data_set_query", data_set_id, "role", "query")
}

fn scoped_identity(
    kind: &str,
    owner_id: &EntityId,
    local_role: &str,
    local_value: &str,
) -> Result<EntityId, DataCompositionIdentityError> {
    EntityId::new(format!(
        "data_composition:{kind};owner#{}:{};{local_role}#{}:{}",
        owner_id.as_str().len(),
        owner_id.as_str(),
        local_value.len(),
        local_value,
    ))
    .map_err(|_| DataCompositionIdentityError)
}

/// Error produced when a Data Composition identity cannot be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataCompositionIdentityError;

impl Display for DataCompositionIdentityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid data composition identifier")
    }
}

impl std::error::Error for DataCompositionIdentityError {}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{
        DataCompositionFieldPayload, DataCompositionSchemaPayload, DataSetKind, DataSetPayload,
        DataSetPayloadError, data_composition_field_id, data_set_id, data_set_query_id,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn typed_payloads_preserve_accepted_semantic_content() {
        let schema = DataCompositionSchemaPayload::new(true);
        let query = DataSetPayload::new(DataSetKind::Query, Some(name("DataSource1")))
            .expect("Query Data Set must accept a local data source");
        let object = DataSetPayload::new(DataSetKind::Object, Some(name("DataSource1")))
            .expect("Object Data Set must accept a local data source");
        let union = DataSetPayload::new(DataSetKind::Union, None)
            .expect("Union Data Set must accept absent direct data source");
        let field = DataCompositionFieldPayload::new(name("Products.Ref"));

        assert!(schema.is_main());
        assert_eq!(query.kind(), DataSetKind::Query);
        assert_eq!(query.data_source(), Some(&name("DataSource1")));
        assert_eq!(object.kind().as_str(), "object");
        assert_eq!(union.data_source(), None);
        assert_eq!(field.data_path().as_str(), "Products.Ref");
    }

    #[test]
    fn data_set_payload_rejects_invalid_data_source_cardinality() {
        assert_eq!(
            DataSetPayload::new(DataSetKind::Query, None),
            Err(DataSetPayloadError::MissingDataSource)
        );
        assert_eq!(
            DataSetPayload::new(DataSetKind::Object, None),
            Err(DataSetPayloadError::MissingDataSource)
        );
        assert_eq!(
            DataSetPayload::new(DataSetKind::Union, Some(name("DataSource1"))),
            Err(DataSetPayloadError::UnexpectedDataSource)
        );
    }

    #[test]
    fn identities_are_stable_collision_safe_and_content_independent() {
        let schema = id("schema:main");
        let data_set =
            data_set_id(&schema, &name("Sales:Current")).expect("Data Set identity must be valid");
        let repeated = data_set_id(&schema, &name("Sales:Current"))
            .expect("repeated Data Set identity must be valid");
        let concatenated = data_set_id(&id("schema:main:Sales"), &name("Current"))
            .expect("second Data Set identity must be valid");
        let field = data_composition_field_id(&data_set, &name("Product:Ref"))
            .expect("Field identity must be valid");
        let query = data_set_query_id(&data_set).expect("Query identity must be valid");

        assert_eq!(data_set, repeated);
        assert_ne!(data_set, concatenated);
        assert_ne!(field, query);
        assert!(data_set.as_str().contains("owner#11:schema:main"));
        assert!(data_set.as_str().contains("name#13:Sales:Current"));
    }
}
