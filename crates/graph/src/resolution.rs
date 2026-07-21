//! Deterministic semantic resolution over a semantic graph.

use oneagent_common::{EntityId, EntityName};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use crate::{EdgeKind, GraphNode, NodeId, NodeKind, SemanticGraph};

/// Semantic reference resolved against a graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticReference {
    /// Reference by stable node identifier.
    NodeId(String),
    /// Reference by exact semantic name.
    Name(EntityName),
    /// Reference to a child by owner and exact local name.
    Child {
        /// Owner node identifier.
        owner: EntityId,
        /// Child local name.
        name: EntityName,
    },
    /// Reference to an owner of a child node.
    Owner {
        /// Child node identifier.
        child: EntityId,
    },
    /// Reference to a concrete child expected under a concrete owner.
    OwnedChild {
        /// Owner node identifier.
        owner: EntityId,
        /// Child node identifier.
        child: EntityId,
    },
}

/// Error produced by semantic resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionError {
    /// The target reference does not resolve to any node.
    MissingTarget {
        /// Failed reference.
        reference: SemanticReference,
    },
    /// The target reference resolves to multiple nodes.
    AmbiguousTarget {
        /// Ambiguous reference.
        reference: SemanticReference,
        /// Deterministically ordered candidate node identifiers.
        candidates: Vec<EntityId>,
    },
    /// The resolved node has an incompatible kind.
    IncompatibleNodeKind {
        /// Resolved node identifier.
        id: EntityId,
        /// Accepted node kinds.
        expected: Vec<NodeKind>,
        /// Actual node kind.
        actual: NodeKind,
    },
    /// The child is not owned by the supplied owner.
    InvalidOwnerReference {
        /// Owner node identifier.
        owner: EntityId,
        /// Child node identifier.
        child: EntityId,
    },
}

impl Display for ResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTarget { reference } => {
                write!(formatter, "semantic reference is missing: {reference:?}")
            }
            Self::AmbiguousTarget {
                reference,
                candidates,
            } => write!(
                formatter,
                "semantic reference is ambiguous: {reference:?}; candidates: {candidates:?}"
            ),
            Self::IncompatibleNodeKind {
                id,
                expected,
                actual,
            } => write!(
                formatter,
                "semantic node `{id}` has incompatible kind {actual:?}; expected one of {expected:?}"
            ),
            Self::InvalidOwnerReference { owner, child } => {
                write!(
                    formatter,
                    "semantic node `{child}` is not owned by `{owner}`"
                )
            }
        }
    }
}

impl std::error::Error for ResolutionError {}

/// Deterministic semantic resolution index built from a semantic graph.
#[derive(Debug)]
pub struct SemanticResolutionIndex<'graph> {
    nodes_by_id: BTreeMap<EntityId, &'graph GraphNode>,
    nodes_by_name: BTreeMap<EntityName, BTreeSet<EntityId>>,
    children_by_owner_name: BTreeMap<(EntityId, EntityName), BTreeSet<EntityId>>,
    owners_by_child: BTreeMap<EntityId, BTreeSet<EntityId>>,
}

impl<'graph> SemanticResolutionIndex<'graph> {
    /// Builds a deterministic resolution index from a graph snapshot.
    #[must_use]
    pub fn new(graph: &'graph SemanticGraph) -> Self {
        let mut nodes_by_id = BTreeMap::new();
        let mut nodes_by_name = BTreeMap::<EntityName, BTreeSet<EntityId>>::new();

        for node in graph.nodes() {
            nodes_by_id.insert(node.id().clone(), node);
            nodes_by_name
                .entry(node.name().clone())
                .or_default()
                .insert(node.id().clone());
        }

        let mut children_by_owner_name =
            BTreeMap::<(EntityId, EntityName), BTreeSet<EntityId>>::new();
        let mut owners_by_child = BTreeMap::<EntityId, BTreeSet<EntityId>>::new();

        for edge in graph
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::Contains)
        {
            if let Some(child) = nodes_by_id.get(edge.target()) {
                children_by_owner_name
                    .entry((edge.source().clone(), child.name().clone()))
                    .or_default()
                    .insert(edge.target().clone());
                owners_by_child
                    .entry(edge.target().clone())
                    .or_default()
                    .insert(edge.source().clone());
            }
        }

        Self {
            nodes_by_id,
            nodes_by_name,
            children_by_owner_name,
            owners_by_child,
        }
    }

    /// Resolves a node by stable [`NodeId`].
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError::MissingTarget`] when no node has this
    /// identifier.
    pub fn resolve_node_id(&self, id: &NodeId) -> Result<&'graph GraphNode, ResolutionError> {
        let reference = SemanticReference::NodeId(id.as_str().to_owned());
        let entity_id = EntityId::new(id.as_str()).map_err(|_| ResolutionError::MissingTarget {
            reference: reference.clone(),
        })?;

        self.resolve_entity_id_with_reference(&entity_id, reference)
    }

    /// Resolves a node by stable [`EntityId`].
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError::MissingTarget`] when no node has this
    /// identifier.
    pub fn resolve_entity_id(&self, id: &EntityId) -> Result<&'graph GraphNode, ResolutionError> {
        self.resolve_entity_id_with_reference(id, SemanticReference::NodeId(id.as_str().to_owned()))
    }

    /// Resolves a node by stable [`EntityId`] and validates its kind.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the node is missing or has an
    /// incompatible kind.
    pub fn resolve_entity_id_of_kind(
        &self,
        id: &EntityId,
        expected: NodeKind,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        let node = self.resolve_entity_id(id)?;

        Self::ensure_kind(node, &[expected])
    }

    /// Resolves a node by exact semantic name.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError::MissingTarget`] when no node has this name and
    /// [`ResolutionError::AmbiguousTarget`] when multiple nodes have it.
    pub fn resolve_name(&self, name: &EntityName) -> Result<&'graph GraphNode, ResolutionError> {
        let reference = SemanticReference::Name(name.clone());
        let candidates =
            self.nodes_by_name
                .get(name)
                .ok_or_else(|| ResolutionError::MissingTarget {
                    reference: reference.clone(),
                })?;

        self.expect_single_candidate(reference, candidates)
    }

    /// Resolves a node by exact semantic name and validates its kind.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the name is missing, ambiguous or
    /// resolves to an incompatible node kind.
    pub fn resolve_name_of_kind(
        &self,
        name: &EntityName,
        expected: NodeKind,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        let reference = SemanticReference::Name(name.clone());
        let candidates =
            self.nodes_by_name
                .get(name)
                .ok_or_else(|| ResolutionError::MissingTarget {
                    reference: reference.clone(),
                })?;
        let matching = candidates
            .iter()
            .filter_map(|id| {
                let node = self.nodes_by_id.get(id).copied()?;

                (node.kind() == expected).then_some(id.clone())
            })
            .collect::<BTreeSet<_>>();

        if matching.is_empty() {
            let Some(first_id) = candidates.iter().next() else {
                return Err(ResolutionError::MissingTarget { reference });
            };
            let Some(first_node) = self.nodes_by_id.get(first_id).copied() else {
                return Err(ResolutionError::MissingTarget { reference });
            };

            Self::ensure_kind(first_node, &[expected])
        } else {
            self.expect_single_candidate(reference, &matching)
        }
    }

    /// Resolves a child node by owner identifier and exact local name.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the owner is missing, the child is
    /// missing under the owner or the local name is ambiguous under the owner.
    pub fn resolve_child(
        &self,
        owner: &EntityId,
        name: &EntityName,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        self.resolve_entity_id(owner)?;

        let reference = SemanticReference::Child {
            owner: owner.clone(),
            name: name.clone(),
        };
        let candidates = self
            .children_by_owner_name
            .get(&(owner.clone(), name.clone()))
            .ok_or_else(|| ResolutionError::MissingTarget {
                reference: reference.clone(),
            })?;

        self.expect_single_candidate(reference, candidates)
    }

    /// Resolves a child node by owner identifier, exact local name and kind.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the child cannot be resolved or has
    /// an incompatible kind.
    pub fn resolve_child_of_kind(
        &self,
        owner: &EntityId,
        name: &EntityName,
        expected: NodeKind,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        let node = self.resolve_child(owner, name)?;

        Self::ensure_kind(node, &[expected])
    }

    /// Resolves a child node after validating a concrete owner-child relation.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError::InvalidOwnerReference`] when both nodes exist
    /// but no `Contains` relation links the supplied owner to the supplied
    /// child.
    pub fn resolve_owned_child(
        &self,
        owner: &EntityId,
        child: &EntityId,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        self.resolve_entity_id(owner)?;
        let child_node = self.resolve_entity_id(child)?;

        if self
            .owners_by_child
            .get(child)
            .is_some_and(|owners| owners.contains(owner))
        {
            Ok(child_node)
        } else {
            Err(ResolutionError::InvalidOwnerReference {
                owner: owner.clone(),
                child: child.clone(),
            })
        }
    }

    /// Resolves the owner of a node.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the child is missing, has no owner
    /// or has multiple owners.
    pub fn resolve_owner(&self, child: &EntityId) -> Result<&'graph GraphNode, ResolutionError> {
        self.resolve_entity_id(child)?;

        let reference = SemanticReference::Owner {
            child: child.clone(),
        };
        let candidates =
            self.owners_by_child
                .get(child)
                .ok_or_else(|| ResolutionError::MissingTarget {
                    reference: reference.clone(),
                })?;

        self.expect_single_candidate(reference, candidates)
    }

    /// Resolves the owner of a node and validates the owner kind.
    ///
    /// # Errors
    ///
    /// Returns a typed resolution error when the owner cannot be resolved or has
    /// an incompatible kind.
    pub fn resolve_owner_of_kind(
        &self,
        child: &EntityId,
        expected: NodeKind,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        let node = self.resolve_owner(child)?;

        Self::ensure_kind(node, &[expected])
    }

    fn resolve_entity_id_with_reference(
        &self,
        id: &EntityId,
        reference: SemanticReference,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        self.nodes_by_id
            .get(id)
            .copied()
            .ok_or(ResolutionError::MissingTarget { reference })
    }

    fn expect_single_candidate(
        &self,
        reference: SemanticReference,
        candidates: &BTreeSet<EntityId>,
    ) -> Result<&'graph GraphNode, ResolutionError> {
        match candidates.len() {
            0 => Err(ResolutionError::MissingTarget { reference }),
            1 => {
                let id = candidates
                    .iter()
                    .next()
                    .expect("single candidate must exist");
                self.resolve_entity_id(id)
            }
            _ => Err(ResolutionError::AmbiguousTarget {
                reference,
                candidates: candidates.iter().cloned().collect(),
            }),
        }
    }

    fn ensure_kind(
        node: &'graph GraphNode,
        expected: &[NodeKind],
    ) -> Result<&'graph GraphNode, ResolutionError> {
        if expected.contains(&node.kind()) {
            Ok(node)
        } else {
            Err(ResolutionError::IncompatibleNodeKind {
                id: node.id().clone(),
                expected: expected.to_vec(),
                actual: node.kind(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;

    use crate::{
        Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, ProducerId,
        Provenance, ResolutionError, ResolutionState, SemanticGraph, SemanticReference,
    };

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

    fn insert_node(graph: &mut SemanticGraph, id: EntityId, name: EntityName, kind: NodeKind) {
        let source = id.clone();
        graph.insert_node(GraphNode::new_with_provenance(
            id,
            name,
            kind,
            vec![provenance(&source)],
        ));
    }

    fn insert_contains(graph: &mut SemanticGraph, owner: &EntityId, child: EntityId) {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                owner.clone(),
                child,
                EdgeKind::Contains,
                vec![provenance(owner)],
            ))
            .expect("contains edge must be valid");
    }

    fn graph_with_owned_attributes(reverse_order: bool) -> SemanticGraph {
        let sales = id("metadata.document.sales");
        let returns = id("metadata.document.returns");
        let sales_company = id("metadata.document.sales:attribute:Company");
        let returns_company = id("metadata.document.returns:attribute:Company");
        let sales_form = id("metadata.document.sales:form:Main");
        let mut graph = SemanticGraph::new();

        let nodes = [
            (
                sales.clone(),
                name("Sales"),
                NodeKind::Metadata(MetadataKind::Document),
            ),
            (
                returns.clone(),
                name("Returns"),
                NodeKind::Metadata(MetadataKind::Document),
            ),
            (sales_company.clone(), name("Company"), NodeKind::Attribute),
            (
                returns_company.clone(),
                name("Company"),
                NodeKind::Attribute,
            ),
            (sales_form.clone(), name("Main"), NodeKind::Form),
        ];

        if reverse_order {
            for (id, name, kind) in nodes.into_iter().rev() {
                insert_node(&mut graph, id, name, kind);
            }
        } else {
            for (id, name, kind) in nodes {
                insert_node(&mut graph, id, name, kind);
            }
        }

        insert_contains(&mut graph, &sales, sales_company);
        insert_contains(&mut graph, &returns, returns_company);
        insert_contains(&mut graph, &sales, sales_form);

        graph
    }

    #[test]
    fn resolves_node_by_node_id() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let node = index
            .resolve_node_id(&NodeId::new("metadata.document.sales"))
            .expect("node must resolve");

        assert_eq!(node.name().as_str(), "Sales");
        assert_eq!(node.kind(), NodeKind::Metadata(MetadataKind::Document));
    }

    #[test]
    fn resolves_child_inside_owner() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let node = index
            .resolve_child(&id("metadata.document.sales"), &name("Company"))
            .expect("child must resolve");

        assert_eq!(
            node.id().as_str(),
            "metadata.document.sales:attribute:Company"
        );
        assert_eq!(node.kind(), NodeKind::Attribute);
    }

    #[test]
    fn same_local_names_under_different_owners_resolve_with_owner_context() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let sales_company = index
            .resolve_child(&id("metadata.document.sales"), &name("Company"))
            .expect("Sales Company must resolve");
        let returns_company = index
            .resolve_child(&id("metadata.document.returns"), &name("Company"))
            .expect("Returns Company must resolve");

        assert_ne!(sales_company.id(), returns_company.id());
    }

    #[test]
    fn name_without_context_reports_ambiguity() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let error = index
            .resolve_name(&name("Company"))
            .expect_err("unqualified name must be ambiguous");

        assert_eq!(
            error,
            ResolutionError::AmbiguousTarget {
                reference: SemanticReference::Name(name("Company")),
                candidates: vec![
                    id("metadata.document.returns:attribute:Company"),
                    id("metadata.document.sales:attribute:Company"),
                ],
            }
        );
    }

    #[test]
    fn missing_target_is_typed() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let error = index
            .resolve_entity_id(&id("metadata.document.missing"))
            .expect_err("missing node must fail");

        assert_eq!(
            error,
            ResolutionError::MissingTarget {
                reference: SemanticReference::NodeId("metadata.document.missing".to_owned()),
            }
        );
    }

    #[test]
    fn incompatible_kind_is_rejected() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let error = index
            .resolve_entity_id_of_kind(
                &id("metadata.document.sales:form:Main"),
                NodeKind::Attribute,
            )
            .expect_err("form must not resolve as attribute");

        assert_eq!(
            error,
            ResolutionError::IncompatibleNodeKind {
                id: id("metadata.document.sales:form:Main"),
                expected: vec![NodeKind::Attribute],
                actual: NodeKind::Form,
            }
        );
    }

    #[test]
    fn name_of_kind_resolves_when_other_kinds_share_name() {
        let mut graph = graph_with_owned_attributes(false);
        insert_node(
            &mut graph,
            id("metadata.catalog.sales"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let index = graph.resolution_index();
        let node = index
            .resolve_name_of_kind(&name("Sales"), NodeKind::Metadata(MetadataKind::Catalog))
            .expect("catalog target must resolve by name and kind");

        assert_eq!(node.id().as_str(), "metadata.catalog.sales");
    }

    #[test]
    fn name_of_kind_still_reports_same_kind_ambiguity() {
        let mut graph = graph_with_owned_attributes(false);
        insert_node(
            &mut graph,
            id("metadata.catalog.sales"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        insert_node(
            &mut graph,
            id("metadata.catalog.sales-copy"),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let index = graph.resolution_index();
        let error = index
            .resolve_name_of_kind(&name("Sales"), NodeKind::Metadata(MetadataKind::Catalog))
            .expect_err("duplicate catalog names must be ambiguous");

        assert_eq!(
            error,
            ResolutionError::AmbiguousTarget {
                reference: SemanticReference::Name(name("Sales")),
                candidates: vec![
                    id("metadata.catalog.sales"),
                    id("metadata.catalog.sales-copy")
                ],
            }
        );
    }

    #[test]
    fn invalid_owner_reference_is_rejected() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let error = index
            .resolve_owned_child(
                &id("metadata.document.returns"),
                &id("metadata.document.sales:attribute:Company"),
            )
            .expect_err("wrong owner must fail");

        assert_eq!(
            error,
            ResolutionError::InvalidOwnerReference {
                owner: id("metadata.document.returns"),
                child: id("metadata.document.sales:attribute:Company"),
            }
        );
    }

    #[test]
    fn resolves_owner_for_metadata_member() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let owner = index
            .resolve_owner(&id("metadata.document.sales:attribute:Company"))
            .expect("owner must resolve");

        assert_eq!(owner.id().as_str(), "metadata.document.sales");
    }

    #[test]
    fn resolution_is_deterministic_across_insertion_order() {
        let normal = graph_with_owned_attributes(false);
        let reversed = graph_with_owned_attributes(true);
        let normal_error = normal
            .resolution_index()
            .resolve_name(&name("Company"))
            .expect_err("name must be ambiguous");
        let reversed_error = reversed
            .resolution_index()
            .resolve_name(&name("Company"))
            .expect_err("name must be ambiguous");

        assert_eq!(normal_error, reversed_error);
    }

    #[test]
    fn resolution_preserves_existing_provenance() {
        let graph = graph_with_owned_attributes(false);
        let index = graph.resolution_index();
        let node = index
            .resolve_child(&id("metadata.document.sales"), &name("Company"))
            .expect("child must resolve");
        let edges = graph.outgoing_by_kind(&id("metadata.document.sales"), EdgeKind::Contains);

        assert_eq!(node.provenance().len(), 1);
        assert_eq!(
            node.provenance()[0].source(),
            Some(&id("metadata.document.sales:attribute:Company"))
        );
        assert!(edges.iter().all(|edge| edge.provenance().len() == 1));
    }
}
