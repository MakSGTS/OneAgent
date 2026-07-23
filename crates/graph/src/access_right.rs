//! Scoped access-right semantic entities.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

use crate::Provenance;

/// Scoped access-right semantic entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRight {
    id: EntityId,
    name: EntityName,
    protected_resource_id: EntityId,
    right_id: EntityId,
    provenance: Vec<Provenance>,
}

impl AccessRight {
    /// Creates an access right scoped to a protected resource.
    ///
    /// # Errors
    ///
    /// Returns [`AccessRightError`] when the derived identifier or name
    /// violates semantic identifier constraints.
    pub fn new(
        protected_resource_id: EntityId,
        right_id: EntityId,
        provenance: Vec<Provenance>,
    ) -> Result<Self, AccessRightError> {
        let raw_id = format!(
            "access_right:resource#{}:{};right#{}:{}",
            protected_resource_id.as_str().len(),
            protected_resource_id.as_str(),
            right_id.as_str().len(),
            right_id.as_str()
        );
        let id = EntityId::new(raw_id).map_err(|_| AccessRightError::InvalidIdentifier)?;
        let name = EntityName::new(format!(
            "{} on {}",
            right_id.as_str(),
            protected_resource_id.as_str()
        ))
        .map_err(|_| AccessRightError::InvalidName)?;

        Ok(Self {
            id,
            name,
            protected_resource_id,
            right_id,
            provenance,
        })
    }

    /// Returns the stable access-right identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the access-right display name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the protected resource identifier.
    #[must_use]
    pub const fn protected_resource_id(&self) -> &EntityId {
        &self.protected_resource_id
    }

    /// Returns the right or operation identifier.
    #[must_use]
    pub const fn right_id(&self) -> &EntityId {
        &self.right_id
    }

    /// Returns provenance records attached to the access right.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }
}

/// Error produced while constructing an access right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRightError {
    /// The derived identifier is invalid.
    InvalidIdentifier,
    /// The derived name is invalid.
    InvalidName,
}

impl Display for AccessRightError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier => formatter.write_str("invalid access right identifier"),
            Self::InvalidName => formatter.write_str("invalid access right name"),
        }
    }
}

impl std::error::Error for AccessRightError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oneagent_common::EntityId;

    use crate::{AccessRight, Confidence, FactOrigin, ProducerId, Provenance, ResolutionState};

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
    fn creates_access_right() {
        let resource_id = id("metadata.document.sales");
        let right_id = id("right.read");
        let access_right = AccessRight::new(
            resource_id.clone(),
            right_id.clone(),
            vec![provenance(&resource_id)],
        )
        .expect("access right must be valid");

        assert_eq!(
            access_right.id().as_str(),
            "access_right:resource#23:metadata.document.sales;right#10:right.read"
        );
        assert_eq!(
            access_right.name().as_str(),
            "right.read on metadata.document.sales"
        );
        assert_eq!(access_right.protected_resource_id(), &resource_id);
        assert_eq!(access_right.right_id(), &right_id);
        assert_eq!(access_right.provenance().len(), 1);
        assert_eq!(access_right.provenance()[0].source(), Some(&resource_id));
    }

    #[test]
    fn access_right_id_is_deterministic() {
        let resource_id = id("metadata.document.sales");
        let right_id = id("right.read");
        let first = AccessRight::new(resource_id.clone(), right_id.clone(), Vec::new())
            .expect("access right must be valid");
        let second = AccessRight::new(resource_id, right_id, Vec::new())
            .expect("access right must be valid");

        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn different_rights_for_same_resource_have_different_ids() {
        let resource_id = id("metadata.document.sales");
        let read = AccessRight::new(resource_id.clone(), id("right.read"), Vec::new())
            .expect("access right must be valid");
        let update = AccessRight::new(resource_id, id("right.update"), Vec::new())
            .expect("access right must be valid");

        assert_ne!(read.id(), update.id());
    }

    #[test]
    fn same_right_for_different_resources_has_different_ids() {
        let sales_read =
            AccessRight::new(id("metadata.document.sales"), id("right.read"), Vec::new())
                .expect("access right must be valid");
        let products_read = AccessRight::new(
            id("metadata.catalog.products"),
            id("right.read"),
            Vec::new(),
        )
        .expect("access right must be valid");

        assert_ne!(sales_read.id(), products_read.id());
    }

    #[test]
    fn access_right_identity_ordering_is_deterministic() {
        let mut ids = BTreeSet::new();

        ids.insert(
            AccessRight::new(
                id("metadata.document.sales"),
                id("right.update"),
                Vec::new(),
            )
            .expect("access right must be valid")
            .id()
            .clone(),
        );
        ids.insert(
            AccessRight::new(id("metadata.document.sales"), id("right.read"), Vec::new())
                .expect("access right must be valid")
                .id()
                .clone(),
        );
        ids.insert(
            AccessRight::new(
                id("metadata.catalog.products"),
                id("right.read"),
                Vec::new(),
            )
            .expect("access right must be valid")
            .id()
            .clone(),
        );

        let ordered = ids
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                "access_right:resource#23:metadata.document.sales;right#10:right.read",
                "access_right:resource#23:metadata.document.sales;right#12:right.update",
                "access_right:resource#25:metadata.catalog.products;right#10:right.read",
            ]
        );
    }
}
