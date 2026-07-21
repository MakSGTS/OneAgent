//! Standard attributes provided by the `1C:Enterprise` platform.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

use crate::Provenance;

/// Kind of a platform-provided standard metadata attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StandardAttributeKind {
    /// Object reference.
    Ref,
    /// Logical deletion marker.
    DeletionMark,
    /// Object code.
    Code,
    /// Object description.
    Description,
    /// Document date.
    Date,
    /// Document number.
    Number,
    /// Document posted flag.
    Posted,
    /// Owner reference.
    Owner,
    /// Hierarchical parent reference.
    Parent,
}

impl StandardAttributeKind {
    /// Returns a stable machine-readable representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ref => "ref",
            Self::DeletionMark => "deletion_mark",
            Self::Code => "code",
            Self::Description => "description",
            Self::Date => "date",
            Self::Number => "number",
            Self::Posted => "posted",
            Self::Owner => "owner",
            Self::Parent => "parent",
        }
    }

    /// Returns the canonical semantic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ref => "Ref",
            Self::DeletionMark => "DeletionMark",
            Self::Code => "Code",
            Self::Description => "Description",
            Self::Date => "Date",
            Self::Number => "Number",
            Self::Posted => "Posted",
            Self::Owner => "Owner",
            Self::Parent => "Parent",
        }
    }
}

/// Standard metadata attribute semantic entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardAttribute {
    id: EntityId,
    name: EntityName,
    parent_id: EntityId,
    kind: StandardAttributeKind,
    provenance: Vec<Provenance>,
}

impl StandardAttribute {
    /// Creates a standard attribute for a metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`StandardAttributeError`] when the derived identifier or name
    /// violates semantic identifier constraints.
    pub fn new(
        parent_id: EntityId,
        kind: StandardAttributeKind,
        provenance: Vec<Provenance>,
    ) -> Result<Self, StandardAttributeError> {
        let raw_id = format!(
            "{}:standard_attribute:{}",
            parent_id.as_str(),
            kind.as_str()
        );
        let id = EntityId::new(raw_id).map_err(|_| StandardAttributeError::InvalidIdentifier)?;
        let name = EntityName::new(kind.name()).map_err(|_| StandardAttributeError::InvalidName)?;

        Ok(Self {
            id,
            name,
            parent_id,
            kind,
            provenance,
        })
    }

    /// Returns the stable standard attribute identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the standard attribute name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the owning metadata object identifier.
    #[must_use]
    pub const fn parent_id(&self) -> &EntityId {
        &self.parent_id
    }

    /// Returns the standard attribute kind.
    #[must_use]
    pub const fn kind(&self) -> StandardAttributeKind {
        self.kind
    }

    /// Returns provenance records attached to the standard attribute.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }
}

/// Error produced while constructing a standard attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardAttributeError {
    /// The derived identifier is invalid.
    InvalidIdentifier,
    /// The derived name is invalid.
    InvalidName,
}

impl Display for StandardAttributeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier => formatter.write_str("invalid standard attribute identifier"),
            Self::InvalidName => formatter.write_str("invalid standard attribute name"),
        }
    }
}

impl std::error::Error for StandardAttributeError {}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use crate::{
        Confidence, FactOrigin, ProducerId, Provenance, ResolutionState, StandardAttribute,
        StandardAttributeKind,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn provenance(source: &EntityId) -> Provenance {
        Provenance::new(
            Some(source.clone()),
            ProducerId::new("oneagent.graph.tests"),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    #[test]
    fn creates_standard_attribute() {
        let parent_id = id("metadata.document.sales");
        let attribute = StandardAttribute::new(
            parent_id.clone(),
            StandardAttributeKind::Ref,
            vec![provenance(&parent_id)],
        )
        .expect("standard attribute must be valid");

        assert_eq!(
            attribute.id().as_str(),
            "metadata.document.sales:standard_attribute:ref"
        );
        assert_eq!(attribute.name().as_str(), "Ref");
        assert_eq!(attribute.parent_id(), &parent_id);
        assert_eq!(attribute.kind(), StandardAttributeKind::Ref);
        assert_eq!(attribute.provenance().len(), 1);
    }

    #[test]
    fn standard_attribute_id_is_deterministic() {
        let parent_id = id("metadata.catalog.products");
        let first =
            StandardAttribute::new(parent_id.clone(), StandardAttributeKind::Code, Vec::new())
                .expect("standard attribute must be valid");
        let second = StandardAttribute::new(parent_id, StandardAttributeKind::Code, Vec::new())
            .expect("standard attribute must be valid");

        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn different_standard_attribute_kinds_have_different_ids() {
        let parent_id = id("metadata.catalog.products");
        let code =
            StandardAttribute::new(parent_id.clone(), StandardAttributeKind::Code, Vec::new())
                .expect("standard attribute must be valid");
        let description =
            StandardAttribute::new(parent_id, StandardAttributeKind::Description, Vec::new())
                .expect("standard attribute must be valid");

        assert_ne!(code.id(), description.id());
    }

    #[test]
    fn same_standard_attribute_kind_has_different_ids_for_different_objects() {
        let catalog_code = StandardAttribute::new(
            id("metadata.catalog.products"),
            StandardAttributeKind::Code,
            Vec::new(),
        )
        .expect("standard attribute must be valid");
        let document_code = StandardAttribute::new(
            id("metadata.document.sales"),
            StandardAttributeKind::Code,
            Vec::new(),
        )
        .expect("standard attribute must be valid");

        assert_ne!(catalog_code.id(), document_code.id());
    }

    #[test]
    fn preserves_provenance() {
        let parent_id = id("metadata.document.sales");
        let attribute = StandardAttribute::new(
            parent_id.clone(),
            StandardAttributeKind::Number,
            vec![provenance(&parent_id)],
        )
        .expect("standard attribute must be valid");

        assert_eq!(attribute.provenance().len(), 1);
        assert_eq!(attribute.provenance()[0].source(), Some(&parent_id));
        assert_eq!(attribute.provenance()[0].origin(), FactOrigin::Declared);
    }
}
