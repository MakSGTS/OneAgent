//! Nodes stored in the semantic graph.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::{MetadataMemberPayload, MetadataPayload};
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
    /// Source-independent content of an `Attribute` or `TabularSection` node.
    MetadataMember(MetadataMemberPayload),
}

impl GraphNodePayload {
    /// Returns whether this payload is compatible with a node kind.
    #[must_use]
    pub const fn is_compatible_with(&self, kind: NodeKind) -> bool {
        match (self, kind) {
            (Self::None, _)
            | (Self::MetadataMember(_), NodeKind::Attribute | NodeKind::TabularSection) => true,
            (Self::Metadata(payload), NodeKind::Metadata(metadata_kind)) => {
                payload.is_compatible_with(metadata_kind)
            }
            (Self::Metadata(_) | Self::MetadataMember(_), _) => false,
        }
    }

    /// Returns metadata content when this is a metadata payload.
    #[must_use]
    pub const fn metadata(&self) -> Option<&MetadataPayload> {
        match self {
            Self::Metadata(payload) => Some(payload),
            Self::None | Self::MetadataMember(_) => None,
        }
    }

    /// Returns subordinate member content when this is a member payload.
    #[must_use]
    pub const fn metadata_member(&self) -> Option<&MetadataMemberPayload> {
        match self {
            Self::None | Self::Metadata(_) => None,
            Self::MetadataMember(payload) => Some(payload),
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

    /// Returns source-independent `Attribute` or `TabularSection` content.
    #[must_use]
    pub const fn metadata_member_payload(&self) -> Option<&MetadataMemberPayload> {
        self.payload.metadata_member()
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
        CommonMetadataPayload, DocumentMetadataPayload, MetadataKind, MetadataMemberPayload,
        MetadataPayload, MetadataSpecificPayload,
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
        assert_eq!(node.metadata_member_payload(), None);
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

    #[test]
    fn accepts_member_payload_for_attribute_and_tabular_section() {
        for kind in [NodeKind::Attribute, NodeKind::TabularSection] {
            let node = GraphNode::new_with_payload(
                id("metadata.member"),
                name("Member"),
                kind,
                GraphNodePayload::MetadataMember(MetadataMemberPayload::new(Some(
                    "Member synonym".to_owned(),
                ))),
            )
            .expect("member payload must be valid for accepted member kinds");

            assert_eq!(node.metadata_payload(), None);
            assert_eq!(
                node.metadata_member_payload()
                    .expect("member payload must exist")
                    .synonym(),
                Some("Member synonym")
            );
        }
    }

    #[test]
    fn rejects_member_payload_for_every_unrelated_node_kind() {
        let unrelated = [
            NodeKind::Metadata(MetadataKind::Catalog),
            NodeKind::Module,
            NodeKind::Procedure,
            NodeKind::Function,
            NodeKind::Query,
            NodeKind::Form,
            NodeKind::Command,
            NodeKind::StandardAttribute,
            NodeKind::Dimension,
            NodeKind::Resource,
            NodeKind::Measure,
            NodeKind::Role,
            NodeKind::AccessRight,
            NodeKind::Subsystem,
            NodeKind::Unknown,
        ];

        for kind in unrelated {
            let error = GraphNode::new_with_payload(
                id("unrelated"),
                name("Unrelated"),
                kind,
                GraphNodePayload::MetadataMember(MetadataMemberPayload::empty()),
            )
            .expect_err("member payload must be rejected for unrelated node kinds");

            assert_eq!(error.node_kind(), kind);
        }
    }
}
