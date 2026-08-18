//! Stable identifiers used by the `OneAgent` Knowledge Graph.

//! Stable identifiers used by the `OneAgent` Knowledge Graph.

pub mod access_right;
pub mod build_diff;
pub mod coverage;
pub mod diagnostic;
pub mod diff;
pub mod edge;
pub mod identity;
pub mod impact;
pub mod kind;
pub mod measure;
pub mod node;
pub mod provenance;
pub mod query;
pub mod reference_request;
pub mod report;
pub mod resolution;
pub mod standard_attribute;
pub mod validation;

pub use access_right::{AccessRight, AccessRightError};
pub use build_diff::{
    BuildDiffSummary, CountChange, CountChangeDirection, CountDelta, DiagnosticChange,
    DiagnosticChangeKind, DiagnosticDiff, DiagnosticDiffSummary, DiagnosticIdentity,
    DiagnosticModifiedAspect, GraphReportDiff, GraphReportMetric, ProvenanceCoverageDiff,
    ProvenanceCoverageMetric, ResolutionStatisticsDiff, ResolutionStatisticsMetric,
    SemanticGraphBuildDiff,
};
pub use coverage::{
    SemanticCoverageCapability, SemanticCoverageCapabilityId, SemanticCoverageCategory,
    SemanticCoverageEvidence, SemanticCoverageGap, SemanticCoverageGapPriority,
    SemanticCoverageRegistry, SemanticCoverageReport, SemanticCoverageStatus,
    SemanticCoverageSummary, SemanticObservedCoverage, SemanticObservedKindCoverage,
    SemanticProvenanceCapability, SemanticQueryCapability, SemanticReferenceCapability,
    semantic_coverage_edge_kinds, semantic_coverage_node_kinds,
};
pub use diagnostic::{
    SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticDiagnosticSeverity,
};
pub use diff::{
    EdgeChange, EdgeModifiedAspect, EdgeSnapshot, GraphChangeKind, GraphDiffSummary, NodeChange,
    NodeModifiedAspect, NodeSnapshot, SemanticGraphDiff,
};
pub use edge::GraphEdge;
pub use identity::{EdgeId, NodeId};
pub use impact::{
    AffectedNode, ImpactAnalysisError, ImpactCompleteness, ImpactNodeAvailability,
    ImpactNodeStatus, ImpactPropagationDirection, ImpactReason, ImpactReasonKind, ImpactSeed,
    ImpactSeedKind, ImpactSnapshot, OwnershipImpactMode, ProvenanceImpactMode,
    SemanticImpactAnalyzer, SemanticImpactOptions, SemanticImpactResult, SemanticImpactSummary,
};
pub use kind::{EdgeKind, NodeKind};
pub use measure::{Measure, MeasureError};
pub use node::{GraphNode, GraphNodePayload, GraphNodePayloadError};
pub use provenance::{Confidence, FactOrigin, ProducerId, Provenance, ResolutionState};
pub use query::{
    SemanticGraphEdgeFilter, SemanticGraphQuery, SemanticGraphRelation,
    SemanticGraphTraversalDirection, SemanticGraphTraversalNode, SemanticGraphTraversalOptions,
};
pub use reference_request::{
    SemanticReferenceCategory, SemanticReferenceRequest, SemanticReferenceRequestError,
    SemanticReferenceRequestId, SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome,
    SemanticReferenceRequestQuery,
};
pub use report::{
    DiagnosticSummary, EdgeSummary, GraphSummary, NodeSummary, ProvenanceCoverageSummary,
    ResolutionRate, SemanticGraphReport, SemanticReferenceOutcome, SemanticReferenceStatistics,
};
pub use resolution::{ResolutionError, SemanticReference, SemanticResolutionIndex};
pub use standard_attribute::{StandardAttribute, StandardAttributeError, StandardAttributeKind};
pub use validation::{
    SemanticGraphSchema, SemanticGraphValidationCode, SemanticGraphValidationIssue,
    SemanticGraphValidationIssueKind, SemanticGraphValidationResult,
    SemanticGraphValidationSeverity, SemanticGraphValidationSummary, SemanticGraphValidator,
};

use oneagent_common::{EntityId, EntityName};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// In-memory `OneAgent Semantic Graph`.
#[derive(Debug, Default, Clone)]
pub struct SemanticGraph {
    nodes: BTreeMap<EntityId, GraphNode>,
    edges: BTreeSet<GraphEdge>,
}

impl SemanticGraph {
    /// Creates an empty semantic graph.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeSet::new(),
        }
    }

    /// Inserts or replaces a node.
    pub fn insert_node(&mut self, node: GraphNode) -> Option<GraphNode> {
        self.nodes.insert(node.id().clone(), node)
    }

    /// Inserts or replaces a node with provenance attached.
    pub fn insert_node_with_provenance(
        &mut self,
        id: EntityId,
        name: EntityName,
        kind: NodeKind,
        provenance: Provenance,
    ) -> Option<GraphNode> {
        self.insert_node(GraphNode::new_with_provenance(
            id,
            name,
            kind,
            vec![provenance],
        ))
    }

    /// Inserts an edge after validating both endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::MissingNode`] when either endpoint is absent.
    pub fn insert_edge(&mut self, edge: GraphEdge) -> Result<bool, GraphError> {
        if !self.nodes.contains_key(edge.source()) {
            return Err(GraphError::MissingNode(edge.source().clone()));
        }

        if !self.nodes.contains_key(edge.target()) {
            return Err(GraphError::MissingNode(edge.target().clone()));
        }

        Ok(self.edges.insert(edge))
    }

    /// Inserts an edge with provenance attached after validating both endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::MissingNode`] when either endpoint is absent.
    pub fn insert_edge_with_provenance(
        &mut self,
        source: EntityId,
        target: EntityId,
        kind: EdgeKind,
        provenance: Provenance,
    ) -> Result<bool, GraphError> {
        self.insert_edge(GraphEdge::new_with_provenance(
            source,
            target,
            kind,
            vec![provenance],
        ))
    }

    /// Inserts a standard attribute node and connects it to its metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::MissingNode`] when the owning metadata object is
    /// absent from the graph.
    pub fn insert_standard_attribute(
        &mut self,
        attribute: &StandardAttribute,
    ) -> Result<Option<GraphNode>, GraphError> {
        if !self.nodes.contains_key(attribute.parent_id()) {
            return Err(GraphError::MissingNode(attribute.parent_id().clone()));
        }

        let node = GraphNode::new_with_provenance(
            attribute.id().clone(),
            attribute.name().clone(),
            NodeKind::StandardAttribute,
            attribute.provenance().to_vec(),
        );
        let edge = GraphEdge::new_with_provenance(
            attribute.parent_id().clone(),
            attribute.id().clone(),
            EdgeKind::Contains,
            attribute.provenance().to_vec(),
        );
        let previous = self.insert_node(node);

        self.insert_edge(edge)?;

        Ok(previous)
    }

    /// Inserts a measure node and connects it to its metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::MissingNode`] when the owning metadata object is
    /// absent from the graph.
    pub fn insert_measure(&mut self, measure: &Measure) -> Result<Option<GraphNode>, GraphError> {
        if !self.nodes.contains_key(measure.parent_id()) {
            return Err(GraphError::MissingNode(measure.parent_id().clone()));
        }

        let node = GraphNode::new_with_provenance(
            measure.id().clone(),
            measure.name().clone(),
            NodeKind::Measure,
            measure.provenance().to_vec(),
        );
        let edge = GraphEdge::new_with_provenance(
            measure.parent_id().clone(),
            measure.id().clone(),
            EdgeKind::Contains,
            measure.provenance().to_vec(),
        );
        let previous = self.insert_node(node);

        self.insert_edge(edge)?;

        Ok(previous)
    }

    /// Inserts an access-right node.
    pub fn insert_access_right(&mut self, access_right: &AccessRight) -> Option<GraphNode> {
        self.insert_node(GraphNode::new_with_provenance(
            access_right.id().clone(),
            access_right.name().clone(),
            NodeKind::AccessRight,
            access_right.provenance().to_vec(),
        ))
    }

    /// Returns a node by identifier.
    #[must_use]
    pub fn node(&self, id: &EntityId) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    /// Returns all graph nodes in deterministic order.
    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    /// Returns all graph edges in deterministic order.
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter()
    }

    /// Builds a semantic resolution index for the current graph snapshot.
    #[must_use]
    pub fn resolution_index(&self) -> SemanticResolutionIndex<'_> {
        SemanticResolutionIndex::new(self)
    }

    /// Builds a deterministic graph-only quality report.
    #[must_use]
    pub fn report(&self) -> SemanticGraphReport {
        SemanticGraphReport::from_graph(self)
    }

    /// Compares this graph snapshot with a newer graph snapshot.
    #[must_use]
    pub fn diff(&self, new: &Self) -> SemanticGraphDiff {
        SemanticGraphDiff::between(self, new)
    }

    /// Validates graph-level structural, semantic and provenance invariants.
    #[must_use]
    pub fn validate(&self) -> SemanticGraphValidationResult {
        SemanticGraphValidator::new().validate(self)
    }

    /// Creates a read-only Semantic Query API for this graph snapshot.
    #[must_use]
    pub const fn query(&self) -> SemanticGraphQuery<'_> {
        SemanticGraphQuery::new(self)
    }

    /// Returns all nodes of a specified kind.
    #[must_use]
    pub fn nodes_by_kind(&self, kind: NodeKind) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|node| node.kind() == kind)
            .collect()
    }

    /// Returns outgoing edges from a node.
    #[must_use]
    pub fn outgoing(&self, source: &EntityId) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source() == source)
            .collect()
    }

    /// Returns outgoing edges of a specified kind.
    #[must_use]
    pub fn outgoing_by_kind(&self, source: &EntityId, kind: EdgeKind) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source() == source && edge.kind() == kind)
            .collect()
    }

    /// Returns incoming edges to a node.
    #[must_use]
    pub fn incoming(&self, target: &EntityId) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.target() == target)
            .collect()
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns `true` when the graph has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Semantic graph operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// An edge references a node absent from the graph.
    MissingNode(EntityId),
}

impl Display for GraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingNode(id) => write!(formatter, "semantic graph node is missing: {id}"),
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use super::{
        AccessRight, Confidence, EdgeKind, FactOrigin, GraphEdge, GraphError, GraphNode, Measure,
        NodeKind, ProducerId, Provenance, ResolutionState, SemanticGraph, StandardAttribute,
        StandardAttributeKind,
    };
    use oneagent_metadata::MetadataKind;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn provenance(source: &EntityId, origin: FactOrigin) -> Provenance {
        Provenance::new(
            Some(source.clone()),
            ProducerId::new("oneagent.graph.tests"),
            origin,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    #[test]
    fn inserts_valid_edge() {
        let document_id = id("document.sales");
        let form_id = id("form.sales.main");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            document_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));
        graph.insert_node(GraphNode::new(
            form_id.clone(),
            name("MainForm"),
            NodeKind::Form,
        ));

        let inserted = graph
            .insert_edge(GraphEdge::new(
                document_id.clone(),
                form_id,
                EdgeKind::Contains,
            ))
            .expect("edge endpoints must exist");

        assert!(inserted);
        assert_eq!(graph.outgoing(&document_id).len(), 1);
    }

    #[test]
    fn rejects_edge_with_missing_target() {
        let source_id = id("module.sales");
        let target_id = id("procedure.missing");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            source_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));

        let error = graph
            .insert_edge(GraphEdge::new(
                source_id,
                target_id.clone(),
                EdgeKind::Calls,
            ))
            .expect_err("missing target must be rejected");

        assert_eq!(
            error.to_string(),
            format!("semantic graph node is missing: {target_id}")
        );
    }

    #[test]
    fn filters_nodes_and_edges_by_kind() {
        let module_id = id("module.sales");
        let procedure_id = id("procedure.sales.post");
        let query_id = id("query.sales.balance");
        let access_right_id =
            id("access_right:resource#23:metadata.document.sales;right#10:right.read");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            module_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            procedure_id.clone(),
            name("Post"),
            NodeKind::Procedure,
        ));
        graph.insert_node(GraphNode::new(
            query_id.clone(),
            name("BalanceQuery"),
            NodeKind::Query,
        ));
        graph.insert_node(GraphNode::new(
            access_right_id,
            name("right.read on metadata.document.sales"),
            NodeKind::AccessRight,
        ));

        graph
            .insert_edge(GraphEdge::new(
                module_id.clone(),
                procedure_id,
                EdgeKind::Contains,
            ))
            .expect("contains edge must be valid");
        graph
            .insert_edge(GraphEdge::new(module_id.clone(), query_id, EdgeKind::Reads))
            .expect("reads edge must be valid");

        assert_eq!(graph.nodes_by_kind(NodeKind::Query).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::AccessRight).len(), 1);
        assert_eq!(graph.outgoing_by_kind(&module_id, EdgeKind::Reads).len(), 1);
    }

    #[test]
    fn inserts_node_with_provenance() {
        let module_id = id("module.sales");
        let mut graph = SemanticGraph::new();

        graph.insert_node_with_provenance(
            module_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
            provenance(&module_id, FactOrigin::Parsed),
        );

        let module = graph.node(&module_id).expect("module node must exist");

        assert_eq!(module.id(), &module_id);
        assert_eq!(module.kind(), NodeKind::Module);
        assert_eq!(module.provenance().len(), 1);
        assert_eq!(module.provenance()[0].source(), Some(&module_id));
        assert_eq!(module.provenance()[0].origin(), FactOrigin::Parsed);
    }

    #[test]
    fn node_constructor_accepts_explicit_provenance() {
        let module_id = id("module.sales");
        let node = GraphNode::new_with_provenance(
            module_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
            vec![provenance(&module_id, FactOrigin::Parsed)],
        );

        assert_eq!(node.id(), &module_id);
        assert_eq!(node.kind(), NodeKind::Module);
        assert_eq!(node.provenance().len(), 1);
        assert_eq!(node.provenance()[0].source(), Some(&module_id));
    }

    #[test]
    fn inserts_edge_with_provenance() {
        let module_id = id("module.sales");
        let procedure_id = id("procedure.sales.post");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            module_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            procedure_id.clone(),
            name("Post"),
            NodeKind::Procedure,
        ));

        graph
            .insert_edge_with_provenance(
                module_id.clone(),
                procedure_id.clone(),
                EdgeKind::Contains,
                provenance(&module_id, FactOrigin::Declared),
            )
            .expect("edge endpoints must exist");

        let edges = graph.outgoing_by_kind(&module_id, EdgeKind::Contains);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target(), &procedure_id);
        assert_eq!(edges[0].kind(), EdgeKind::Contains);
        assert_eq!(edges[0].provenance().len(), 1);
        assert_eq!(edges[0].provenance()[0].source(), Some(&module_id));
    }

    #[test]
    fn edge_constructor_accepts_explicit_provenance() {
        let module_id = id("module.sales");
        let procedure_id = id("procedure.sales.post");
        let edge = GraphEdge::new_with_provenance(
            module_id.clone(),
            procedure_id.clone(),
            EdgeKind::Contains,
            vec![provenance(&module_id, FactOrigin::Declared)],
        );

        assert_eq!(edge.source(), &module_id);
        assert_eq!(edge.target(), &procedure_id);
        assert_eq!(edge.kind(), EdgeKind::Contains);
        assert_eq!(edge.provenance().len(), 1);
        assert_eq!(edge.provenance()[0].source(), Some(&module_id));
    }

    #[test]
    fn edge_identity_is_independent_from_provenance() {
        let source_id = id("module.sales");
        let target_id = id("procedure.sales.post");
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            source_id.clone(),
            name("SalesModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            target_id.clone(),
            name("Post"),
            NodeKind::Procedure,
        ));

        let first = graph
            .insert_edge_with_provenance(
                source_id.clone(),
                target_id.clone(),
                EdgeKind::Contains,
                provenance(&source_id, FactOrigin::Declared),
            )
            .expect("edge endpoints must exist");
        let second = graph
            .insert_edge_with_provenance(
                source_id.clone(),
                target_id,
                EdgeKind::Contains,
                provenance(&source_id, FactOrigin::Derived),
            )
            .expect("edge endpoints must exist");

        assert!(first);
        assert!(!second);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn inserts_standard_attribute_node_and_parent_edge() {
        let document_id = id("metadata.document.sales");
        let attribute = StandardAttribute::new(
            document_id.clone(),
            StandardAttributeKind::Number,
            vec![provenance(&document_id, FactOrigin::Declared)],
        )
        .expect("standard attribute must be valid");
        let attribute_id = attribute.id().clone();
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            document_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));

        let previous = graph
            .insert_standard_attribute(&attribute)
            .expect("standard attribute parent must exist");

        let node = graph
            .node(&attribute_id)
            .expect("standard attribute node must exist");
        let edges = graph.outgoing_by_kind(&document_id, EdgeKind::Contains);

        assert!(previous.is_none());
        assert_eq!(node.kind(), NodeKind::StandardAttribute);
        assert_eq!(node.name().as_str(), "Number");
        assert_eq!(node.provenance().len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::StandardAttribute).len(), 1);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target(), &attribute_id);
        assert_eq!(edges[0].provenance().len(), 1);
    }

    #[test]
    fn rejects_standard_attribute_when_parent_is_missing() {
        let document_id = id("metadata.document.sales");
        let attribute =
            StandardAttribute::new(document_id.clone(), StandardAttributeKind::Ref, Vec::new())
                .expect("standard attribute must be valid");
        let attribute_id = attribute.id().clone();
        let mut graph = SemanticGraph::new();

        let error = graph
            .insert_standard_attribute(&attribute)
            .expect_err("missing parent must be rejected");

        assert_eq!(error, GraphError::MissingNode(document_id));
        assert!(graph.node(&attribute_id).is_none());
    }

    #[test]
    fn standard_attribute_does_not_regress_user_attribute() {
        let document_id = id("metadata.document.sales");
        let user_attribute_id = id("metadata.document.sales:attribute:Code");
        let standard_attribute = StandardAttribute::new(
            document_id.clone(),
            StandardAttributeKind::Code,
            vec![provenance(&document_id, FactOrigin::Declared)],
        )
        .expect("standard attribute must be valid");
        let standard_attribute_id = standard_attribute.id().clone();
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            document_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::Document),
        ));
        graph.insert_node(GraphNode::new(
            user_attribute_id.clone(),
            name("Code"),
            NodeKind::Attribute,
        ));
        graph
            .insert_edge(GraphEdge::new(
                document_id.clone(),
                user_attribute_id.clone(),
                EdgeKind::Contains,
            ))
            .expect("attribute edge must be valid");
        graph
            .insert_standard_attribute(&standard_attribute)
            .expect("standard attribute parent must exist");

        assert_ne!(user_attribute_id, standard_attribute_id);
        assert_eq!(graph.nodes_by_kind(NodeKind::Attribute).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::StandardAttribute).len(), 1);
        assert_eq!(
            graph
                .outgoing_by_kind(&document_id, EdgeKind::Contains)
                .len(),
            2
        );
        assert_eq!(
            graph.node(&user_attribute_id).map(GraphNode::kind),
            Some(NodeKind::Attribute)
        );
        assert_eq!(
            graph.node(&standard_attribute_id).map(GraphNode::kind),
            Some(NodeKind::StandardAttribute)
        );
    }

    #[test]
    fn inserts_measure_node_and_parent_edge() {
        let register_id = id("metadata.accounting_register.sales");
        let measure = Measure::new(
            register_id.clone(),
            name("Amount"),
            vec![provenance(&register_id, FactOrigin::Declared)],
        )
        .expect("measure must be valid");
        let measure_id = measure.id().clone();
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            register_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::AccountingRegister),
        ));

        let previous = graph
            .insert_measure(&measure)
            .expect("measure parent must exist");

        let node = graph.node(&measure_id).expect("measure node must exist");
        let edges = graph.outgoing_by_kind(&register_id, EdgeKind::Contains);

        assert!(previous.is_none());
        assert_eq!(node.kind(), NodeKind::Measure);
        assert_eq!(node.name().as_str(), "Amount");
        assert_eq!(node.provenance().len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Measure).len(), 1);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target(), &measure_id);
        assert_eq!(edges[0].provenance().len(), 1);
    }

    #[test]
    fn inserts_access_right_node_without_owner_edge() {
        let resource_id = id("metadata.document.sales");
        let right_id = id("right.read");
        let access_right = AccessRight::new(
            resource_id.clone(),
            right_id,
            vec![provenance(&resource_id, FactOrigin::Declared)],
        )
        .expect("access right must be valid");
        let access_right_id = access_right.id().clone();
        let mut graph = SemanticGraph::new();

        let previous = graph.insert_access_right(&access_right);

        let node = graph
            .node(&access_right_id)
            .expect("access right node must exist");

        assert!(previous.is_none());
        assert_eq!(node.kind(), NodeKind::AccessRight);
        assert_eq!(
            node.name().as_str(),
            "right.read on metadata.document.sales"
        );
        assert_eq!(node.provenance().len(), 1);
        assert_eq!(node.provenance()[0].source(), Some(&resource_id));
        assert_eq!(graph.nodes_by_kind(NodeKind::AccessRight).len(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn rejects_measure_when_parent_is_missing() {
        let register_id = id("metadata.accounting_register.sales");
        let measure = Measure::new(register_id.clone(), name("Amount"), Vec::new())
            .expect("measure must be valid");
        let measure_id = measure.id().clone();
        let mut graph = SemanticGraph::new();

        let error = graph
            .insert_measure(&measure)
            .expect_err("missing parent must be rejected");

        assert_eq!(error, GraphError::MissingNode(register_id));
        assert!(graph.node(&measure_id).is_none());
    }

    #[test]
    fn measure_does_not_collide_with_resource_or_dimension() {
        let register_id = id("metadata.accounting_register.sales");
        let dimension_id = id("metadata.accounting_register.sales:dimension:Amount");
        let resource_id = id("metadata.accounting_register.sales:resource:Amount");
        let measure = Measure::new(
            register_id.clone(),
            name("Amount"),
            vec![provenance(&register_id, FactOrigin::Declared)],
        )
        .expect("measure must be valid");
        let measure_id = measure.id().clone();
        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            register_id.clone(),
            name("Sales"),
            NodeKind::Metadata(MetadataKind::AccountingRegister),
        ));
        graph.insert_node(GraphNode::new(
            dimension_id.clone(),
            name("Amount"),
            NodeKind::Dimension,
        ));
        graph.insert_node(GraphNode::new(
            resource_id.clone(),
            name("Amount"),
            NodeKind::Resource,
        ));
        graph
            .insert_edge(GraphEdge::new(
                register_id.clone(),
                dimension_id.clone(),
                EdgeKind::Contains,
            ))
            .expect("dimension edge must be valid");
        graph
            .insert_edge(GraphEdge::new(
                register_id.clone(),
                resource_id.clone(),
                EdgeKind::Contains,
            ))
            .expect("resource edge must be valid");
        graph
            .insert_measure(&measure)
            .expect("measure parent must exist");

        assert_ne!(measure_id, dimension_id);
        assert_ne!(measure_id, resource_id);
        assert_eq!(graph.nodes_by_kind(NodeKind::Dimension).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Resource).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Measure).len(), 1);
        assert_eq!(
            graph
                .outgoing_by_kind(&register_id, EdgeKind::Contains)
                .len(),
            3
        );
        assert_eq!(
            graph.node(&dimension_id).map(GraphNode::kind),
            Some(NodeKind::Dimension)
        );
        assert_eq!(
            graph.node(&resource_id).map(GraphNode::kind),
            Some(NodeKind::Resource)
        );
        assert_eq!(
            graph.node(&measure_id).map(GraphNode::kind),
            Some(NodeKind::Measure)
        );
    }
}
