//! Nodes stored in the semantic graph.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataPayload;
use std::fmt::{Display, Formatter};

use crate::{NodeKind, Provenance};

/// Closed typed content stored by a semantic graph node.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum GraphNodePayload {
    /// The node has no accepted typed content.
    #[default]
    None,
    /// Source-independent content of a metadata node.
    Metadata(MetadataPayload),
}

impl GraphNodePayload {
    /// Returns whether this payload is compatible with a node kind.
    #[must_use]
    pub const fn is_compatible_with(&self, kind: NodeKind) -> bool {
        match (self, kind) {
            (Self::None, _) => true,
            (Self::Metadata(payload), NodeKind::Metadata(metadata_kind)) => {
                payload.is_compatible_with(metadata_kind)
            }
            (Self::Metadata(_), _) => false,
        }
    }

    /// Returns metadata content when this is a metadata payload.
    #[must_use]
    pub const fn metadata(&self) -> Option<&MetadataPayload> {
        match self {
            Self::None => None,
            Self::Metadata(payload) => Some(payload),
        }
    }
}

/// Error returned when typed node content conflicts with the node kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphNodePayloadError {
    node_kind: NodeKind,
}

impl GraphNodePayloadError {
    /// Returns the node kind rejected by payload validation.
    #[must_use]
    pub const fn node_kind(self) -> NodeKind {
        self.node_kind
    }
}

impl Display for GraphNodePayloadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("graph node payload is incompatible with node kind")
    }
}

impl std::error::Error for GraphNodePayloadError {}

/// Semantic graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    payload: GraphNodePayload,
    provenance: Vec<Provenance>,
}

impl GraphNode {
    /// Creates a semantic graph node without provenance.
    #[must_use]
    pub const fn new(id: EntityId, name: EntityName, kind: NodeKind) -> Self {
        Self::new_with_provenance(id, name, kind, Vec::new())
    }

    /// Creates a semantic graph node with provenance records.
    #[must_use]
    pub const fn new_with_provenance(
        id: EntityId,
        name: EntityName,
        kind: NodeKind,
        provenance: Vec<Provenance>,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            payload: GraphNodePayload::None,
            provenance,
        }
    }

    /// Creates a semantic graph node with explicit typed content.
    ///
    /// # Errors
    ///
    /// Returns [`GraphNodePayloadError`] when `payload` conflicts with `kind`.
    pub fn new_with_payload(
        id: EntityId,
        name: EntityName,
        kind: NodeKind,
        payload: GraphNodePayload,
    ) -> Result<Self, GraphNodePayloadError> {
        Self::new_with_payload_and_provenance(id, name, kind, payload, Vec::new())
    }

    /// Creates a semantic graph node with typed content and provenance.
    ///
    /// # Errors
    ///
    /// Returns [`GraphNodePayloadError`] when `payload` conflicts with `kind`.
    pub fn new_with_payload_and_provenance(
        id: EntityId,
        name: EntityName,
        kind: NodeKind,
        payload: GraphNodePayload,
        provenance: Vec<Provenance>,
    ) -> Result<Self, GraphNodePayloadError> {
        if !payload.is_compatible_with(kind) {
            return Err(GraphNodePayloadError { node_kind: kind });
        }

        Ok(Self {
            id,
            name,
            kind,
            payload,
            provenance,
        })
    }

    /// Adds provenance and returns the node.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance.push(provenance);
        self
    }

    /// Returns the node identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the node name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns typed semantic content stored by the node.
    #[must_use]
    pub const fn payload(&self) -> &GraphNodePayload {
        &self.payload
    }

    /// Returns source-independent metadata content when present.
    #[must_use]
    pub const fn metadata_payload(&self) -> Option<&MetadataPayload> {
        self.payload.metadata()
    }

    /// Returns provenance records attached to the node.
    #[must_use]
    pub fn provenance(&self) -> &[Provenance] {
        &self.provenance
    }

    /// Adds provenance to the node.
    pub fn add_provenance(&mut self, provenance: Provenance) {
        self.provenance.push(provenance);
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::{
        CommonMetadataPayload, DocumentMetadataPayload, MetadataKind, MetadataPayload,
        MetadataSpecificPayload,
    };

    use super::{GraphNode, GraphNodePayload};
    use crate::NodeKind;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn document_payload() -> MetadataPayload {
        MetadataPayload::new(
            CommonMetadataPayload::new(Some("Sales".to_owned())),
            Some(MetadataSpecificPayload::Document(
                DocumentMetadataPayload::default(),
            )),
        )
    }

    #[test]
    fn compatibility_constructor_creates_node_without_payload() {
        let node = GraphNode::new(id("module"), name("Module"), NodeKind::Module);

        assert_eq!(node.payload(), &GraphNodePayload::None);
        assert_eq!(node.metadata_payload(), None);
    }

    #[test]
    fn rejects_metadata_payload_on_non_metadata_node() {
        let error = GraphNode::new_with_payload(
            id("module"),
            name("Module"),
            NodeKind::Module,
            GraphNodePayload::Metadata(MetadataPayload::empty()),
        )
        .expect_err("metadata payload on Module must be rejected");

        assert_eq!(error.node_kind(), NodeKind::Module);
    }

    #[test]
    fn rejects_kind_specific_payload_mismatch() {
        let error = GraphNode::new_with_payload(
            id("metadata.catalog.products"),
            name("Products"),
            NodeKind::Metadata(MetadataKind::Catalog),
            GraphNodePayload::Metadata(document_payload()),
        )
        .expect_err("Document payload on Catalog node must be rejected");

        assert_eq!(error.node_kind(), NodeKind::Metadata(MetadataKind::Catalog));
    }

    #[test]
    fn accepts_matching_metadata_payload() {
        let node = GraphNode::new_with_payload(
            id("metadata.document.sales"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
            GraphNodePayload::Metadata(document_payload()),
        )
        .expect("Document payload on Document node must be valid");

        assert_eq!(
            node.metadata_payload()
                .expect("metadata payload must exist")
                .common()
                .synonym(),
            Some("Sales")
        );
    }
}
