//! Accounting register measures.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

use crate::Provenance;

/// Accounting register resource semantic entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    id: EntityId,
    name: EntityName,
    parent_id: EntityId,
    provenance: Vec<Provenance>,
}

impl Measure {
    /// Creates a measure for an accounting register metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`MeasureError`] when the derived identifier violates semantic
    /// identifier constraints.
    pub fn new(
        parent_id: EntityId,
        name: EntityName,
        provenance: Vec<Provenance>,
    ) -> Result<Self, MeasureError> {
        let raw_id = format!("{}:measure:{}", parent_id.as_str(), name.as_str());
        let id = EntityId::new(raw_id).map_err(|_| MeasureError::InvalidIdentifier)?;

        Ok(Self {
            id,
            name,
            parent_id,
            provenance,
        })
    }

    /// Returns the stable measure identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the measure name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the owning metadata object identifier.
    #[must_use]
    pub const fn parent_id(&self) -> &EntityId {
        &self.parent_id
    }

    /// Returns provenance records attached to the measure.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }
}

/// Error produced while constructing a measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureError {
    /// The derived identifier is invalid.
    InvalidIdentifier,
}

impl Display for MeasureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier => formatter.write_str("invalid measure identifier"),
        }
    }
}

impl std::error::Error for MeasureError {}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use crate::{Confidence, FactOrigin, Measure, ProducerId, Provenance, ResolutionState};

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
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
    fn creates_measure() {
        let parent_id = id("metadata.accounting_register.sales");
        let measure = Measure::new(
            parent_id.clone(),
            name("Amount"),
            vec![provenance(&parent_id)],
        )
        .expect("measure must be valid");

        assert_eq!(
            measure.id().as_str(),
            "metadata.accounting_register.sales:measure:Amount"
        );
        assert_eq!(measure.name().as_str(), "Amount");
        assert_eq!(measure.parent_id(), &parent_id);
        assert_eq!(measure.provenance().len(), 1);
    }

    #[test]
    fn measure_id_is_deterministic() {
        let parent_id = id("metadata.accounting_register.sales");
        let first = Measure::new(parent_id.clone(), name("Amount"), Vec::new())
            .expect("measure must be valid");
        let second =
            Measure::new(parent_id, name("Amount"), Vec::new()).expect("measure must be valid");

        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn different_measure_names_have_different_ids() {
        let parent_id = id("metadata.accounting_register.sales");
        let amount = Measure::new(parent_id.clone(), name("Amount"), Vec::new())
            .expect("measure must be valid");
        let quantity =
            Measure::new(parent_id, name("Quantity"), Vec::new()).expect("measure must be valid");

        assert_ne!(amount.id(), quantity.id());
    }

    #[test]
    fn same_measure_name_has_different_ids_for_different_objects() {
        let sales_amount = Measure::new(
            id("metadata.accounting_register.sales"),
            name("Amount"),
            Vec::new(),
        )
        .expect("measure must be valid");
        let tax_amount = Measure::new(
            id("metadata.accounting_register.tax"),
            name("Amount"),
            Vec::new(),
        )
        .expect("measure must be valid");

        assert_ne!(sales_amount.id(), tax_amount.id());
    }

    #[test]
    fn preserves_provenance() {
        let parent_id = id("metadata.accounting_register.sales");
        let measure = Measure::new(
            parent_id.clone(),
            name("Amount"),
            vec![provenance(&parent_id)],
        )
        .expect("measure must be valid");

        assert_eq!(measure.provenance().len(), 1);
        assert_eq!(measure.provenance()[0].source(), Some(&parent_id));
        assert_eq!(measure.provenance()[0].origin(), FactOrigin::Declared);
    }
}
