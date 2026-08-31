//! Transport-neutral graph queries over immutable Workspace snapshots.

use std::error::Error;
use std::fmt::{Display, Formatter};

use oneagent_common::EntityId;
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, NodeId, NodeKind, SemanticGraphQuery,
    SemanticGraphTraversalDirection, SemanticGraphTraversalOptions,
};
use oneagent_metadata::MetadataKind;
use oneagent_workspace::WorkspaceFormat;

use super::{WorkspaceConfigurationSnapshot, WorkspaceSnapshot, WorkspaceSnapshotObserver};

const DEFAULT_RESULT_LIMIT: usize = 50;
const MAX_RESULT_LIMIT: usize = 100;
const MAX_TRAVERSAL_DEPTH: usize = 4;

/// Direction used by transport-neutral relation and traversal requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryDirection {
    /// Follow graph edges from source to target.
    Outgoing,
    /// Follow graph edges from target to source.
    Incoming,
}

impl GraphQueryDirection {
    /// Returns the stable public value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outgoing => "outgoing",
            Self::Incoming => "incoming",
        }
    }

    /// Parses one stable public value.
    #[must_use]
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "outgoing" => Some(Self::Outgoing),
            "incoming" => Some(Self::Incoming),
            _ => None,
        }
    }

    const fn traversal(self) -> SemanticGraphTraversalDirection {
        match self {
            Self::Outgoing => SemanticGraphTraversalDirection::Downstream,
            Self::Incoming => SemanticGraphTraversalDirection::Upstream,
        }
    }
}

/// Stable source format exposed by configuration query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryWorkspaceFormat {
    /// `1C:EDT` source project.
    Edt,
    /// Designer hierarchical XML source project.
    DesignerXml,
}

impl GraphQueryWorkspaceFormat {
    /// Returns the stable public value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Edt => "edt",
            Self::DesignerXml => "designer_xml",
        }
    }

    fn from_workspace_format(format: WorkspaceFormat) -> Self {
        match format {
            WorkspaceFormat::Edt => Self::Edt,
            WorkspaceFormat::DesignerXml => Self::DesignerXml,
            WorkspaceFormat::Extension | WorkspaceFormat::Unknown => {
                unreachable!("published Workspace snapshots contain only supported formats")
            }
        }
    }
}

/// Stable metadata kind exposed by node query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryMetadataKind {
    Configuration,
    Subsystem,
    Catalog,
    Document,
    Enumeration,
    CommonModule,
    Report,
    DataProcessor,
    InformationRegister,
    AccumulationRegister,
    AccountingRegister,
    CalculationRegister,
    BusinessProcess,
    Task,
    Role,
    CommonForm,
    Form,
    Command,
    Template,
    HttpService,
    WebService,
    XdtoPackage,
    EventSubscription,
    Unknown,
}

impl GraphQueryMetadataKind {
    /// Returns the stable public value.
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

impl From<MetadataKind> for GraphQueryMetadataKind {
    fn from(value: MetadataKind) -> Self {
        match value {
            MetadataKind::Configuration => Self::Configuration,
            MetadataKind::Subsystem => Self::Subsystem,
            MetadataKind::Catalog => Self::Catalog,
            MetadataKind::Document => Self::Document,
            MetadataKind::Enumeration => Self::Enumeration,
            MetadataKind::CommonModule => Self::CommonModule,
            MetadataKind::Report => Self::Report,
            MetadataKind::DataProcessor => Self::DataProcessor,
            MetadataKind::InformationRegister => Self::InformationRegister,
            MetadataKind::AccumulationRegister => Self::AccumulationRegister,
            MetadataKind::AccountingRegister => Self::AccountingRegister,
            MetadataKind::CalculationRegister => Self::CalculationRegister,
            MetadataKind::BusinessProcess => Self::BusinessProcess,
            MetadataKind::Task => Self::Task,
            MetadataKind::Role => Self::Role,
            MetadataKind::CommonForm => Self::CommonForm,
            MetadataKind::Form => Self::Form,
            MetadataKind::Command => Self::Command,
            MetadataKind::Template => Self::Template,
            MetadataKind::HttpService => Self::HttpService,
            MetadataKind::WebService => Self::WebService,
            MetadataKind::XdtoPackage => Self::XdtoPackage,
            MetadataKind::EventSubscription => Self::EventSubscription,
            MetadataKind::Unknown => Self::Unknown,
        }
    }
}

/// Stable node kind exposed by node query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryNodeKind {
    Metadata(GraphQueryMetadataKind),
    Module,
    Procedure,
    Function,
    Query,
    DataCompositionSchema,
    DataSet,
    DataCompositionField,
    XdtoType,
    HttpServiceUrlTemplate,
    HttpServiceMethod,
    WebServiceOperation,
    WebServiceParameter,
    Form,
    Command,
    Attribute,
    StandardAttribute,
    TabularSection,
    Dimension,
    Resource,
    Measure,
    Role,
    AccessRight,
    Subsystem,
    Unknown,
}

impl GraphQueryNodeKind {
    /// Returns the stable outer node kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metadata(_) => "metadata",
            Self::Module => "module",
            Self::Procedure => "procedure",
            Self::Function => "function",
            Self::Query => "query",
            Self::DataCompositionSchema => "data_composition_schema",
            Self::DataSet => "data_set",
            Self::DataCompositionField => "data_composition_field",
            Self::XdtoType => "xdto_type",
            Self::HttpServiceUrlTemplate => "http_service_url_template",
            Self::HttpServiceMethod => "http_service_method",
            Self::WebServiceOperation => "web_service_operation",
            Self::WebServiceParameter => "web_service_parameter",
            Self::Form => "form",
            Self::Command => "command",
            Self::Attribute => "attribute",
            Self::StandardAttribute => "standard_attribute",
            Self::TabularSection => "tabular_section",
            Self::Dimension => "dimension",
            Self::Resource => "resource",
            Self::Measure => "measure",
            Self::Role => "role",
            Self::AccessRight => "access_right",
            Self::Subsystem => "subsystem",
            Self::Unknown => "unknown",
        }
    }

    /// Returns the nested metadata kind when this is a metadata node.
    #[must_use]
    pub const fn metadata_kind(self) -> Option<GraphQueryMetadataKind> {
        match self {
            Self::Metadata(kind) => Some(kind),
            Self::Module
            | Self::Procedure
            | Self::Function
            | Self::Query
            | Self::DataCompositionSchema
            | Self::DataSet
            | Self::DataCompositionField
            | Self::XdtoType
            | Self::HttpServiceUrlTemplate
            | Self::HttpServiceMethod
            | Self::WebServiceOperation
            | Self::WebServiceParameter
            | Self::Form
            | Self::Command
            | Self::Attribute
            | Self::StandardAttribute
            | Self::TabularSection
            | Self::Dimension
            | Self::Resource
            | Self::Measure
            | Self::Role
            | Self::AccessRight
            | Self::Subsystem
            | Self::Unknown => None,
        }
    }
}

impl From<NodeKind> for GraphQueryNodeKind {
    fn from(value: NodeKind) -> Self {
        match value {
            NodeKind::Metadata(kind) => Self::Metadata(kind.into()),
            NodeKind::Module => Self::Module,
            NodeKind::Procedure => Self::Procedure,
            NodeKind::Function => Self::Function,
            NodeKind::Query => Self::Query,
            NodeKind::DataCompositionSchema => Self::DataCompositionSchema,
            NodeKind::DataSet => Self::DataSet,
            NodeKind::DataCompositionField => Self::DataCompositionField,
            NodeKind::XdtoType => Self::XdtoType,
            NodeKind::HttpServiceUrlTemplate => Self::HttpServiceUrlTemplate,
            NodeKind::HttpServiceMethod => Self::HttpServiceMethod,
            NodeKind::WebServiceOperation => Self::WebServiceOperation,
            NodeKind::WebServiceParameter => Self::WebServiceParameter,
            NodeKind::Form => Self::Form,
            NodeKind::Command => Self::Command,
            NodeKind::Attribute => Self::Attribute,
            NodeKind::StandardAttribute => Self::StandardAttribute,
            NodeKind::TabularSection => Self::TabularSection,
            NodeKind::Dimension => Self::Dimension,
            NodeKind::Resource => Self::Resource,
            NodeKind::Measure => Self::Measure,
            NodeKind::Role => Self::Role,
            NodeKind::AccessRight => Self::AccessRight,
            NodeKind::Subsystem => Self::Subsystem,
            NodeKind::Unknown => Self::Unknown,
        }
    }
}

/// Stable edge kind exposed by relation query results and filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryEdgeKind {
    Contains,
    Calls,
    References,
    Reads,
    Writes,
    Grants,
    Includes,
    Extends,
    DependsOn,
    Opens,
    Triggers,
}

impl GraphQueryEdgeKind {
    /// Returns the stable public value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Calls => "calls",
            Self::References => "references",
            Self::Reads => "reads",
            Self::Writes => "writes",
            Self::Grants => "grants",
            Self::Includes => "includes",
            Self::Extends => "extends",
            Self::DependsOn => "depends_on",
            Self::Opens => "opens",
            Self::Triggers => "triggers",
        }
    }

    /// Parses one stable public value.
    #[must_use]
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "contains" => Some(Self::Contains),
            "calls" => Some(Self::Calls),
            "references" => Some(Self::References),
            "reads" => Some(Self::Reads),
            "writes" => Some(Self::Writes),
            "grants" => Some(Self::Grants),
            "includes" => Some(Self::Includes),
            "extends" => Some(Self::Extends),
            "depends_on" => Some(Self::DependsOn),
            "opens" => Some(Self::Opens),
            "triggers" => Some(Self::Triggers),
            _ => None,
        }
    }

    const fn graph_kind(self) -> EdgeKind {
        match self {
            Self::Contains => EdgeKind::Contains,
            Self::Calls => EdgeKind::Calls,
            Self::References => EdgeKind::References,
            Self::Reads => EdgeKind::Reads,
            Self::Writes => EdgeKind::Writes,
            Self::Grants => EdgeKind::Grants,
            Self::Includes => EdgeKind::Includes,
            Self::Extends => EdgeKind::Extends,
            Self::DependsOn => EdgeKind::DependsOn,
            Self::Opens => EdgeKind::Opens,
            Self::Triggers => EdgeKind::Triggers,
        }
    }
}

impl From<EdgeKind> for GraphQueryEdgeKind {
    fn from(value: EdgeKind) -> Self {
        match value {
            EdgeKind::Contains => Self::Contains,
            EdgeKind::Calls => Self::Calls,
            EdgeKind::References => Self::References,
            EdgeKind::Reads => Self::Reads,
            EdgeKind::Writes => Self::Writes,
            EdgeKind::Grants => Self::Grants,
            EdgeKind::Includes => Self::Includes,
            EdgeKind::Extends => Self::Extends,
            EdgeKind::DependsOn => Self::DependsOn,
            EdgeKind::Opens => Self::Opens,
            EdgeKind::Triggers => Self::Triggers,
        }
    }
}

/// Validated maximum number of records returned by one list query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphQueryLimit(usize);

impl GraphQueryLimit {
    /// Creates a result limit in the accepted `1..=100` range.
    ///
    /// # Errors
    ///
    /// Returns [`GraphQueryErrorKind::LimitOutOfRange`] outside that range.
    pub fn new(value: usize) -> Result<Self, GraphQueryError> {
        if (1..=MAX_RESULT_LIMIT).contains(&value) {
            Ok(Self(value))
        } else {
            Err(GraphQueryError::new(GraphQueryErrorKind::LimitOutOfRange))
        }
    }

    /// Returns the validated value.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl Default for GraphQueryLimit {
    fn default() -> Self {
        Self(DEFAULT_RESULT_LIMIT)
    }
}

/// Validated maximum breadth-first traversal depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphQueryMaxDepth(usize);

impl GraphQueryMaxDepth {
    /// Creates a maximum depth in the accepted `0..=4` range.
    ///
    /// # Errors
    ///
    /// Returns [`GraphQueryErrorKind::MaxDepthOutOfRange`] outside that range.
    pub fn new(value: usize) -> Result<Self, GraphQueryError> {
        if value <= MAX_TRAVERSAL_DEPTH {
            Ok(Self(value))
        } else {
            Err(GraphQueryError::new(
                GraphQueryErrorKind::MaxDepthOutOfRange,
            ))
        }
    }

    /// Returns the validated value.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Stable transport-neutral Graph Query failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQueryErrorKind {
    WorkspaceUnavailable,
    InvalidIdentifier,
    ConfigurationNotFound,
    NodeNotFound,
    LimitOutOfRange,
    MaxDepthOutOfRange,
}

/// Typed Graph Query failure with bounded identity context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryError {
    kind: GraphQueryErrorKind,
    configuration_id: Option<String>,
    node_id: Option<String>,
}

impl GraphQueryError {
    const fn new(kind: GraphQueryErrorKind) -> Self {
        Self {
            kind,
            configuration_id: None,
            node_id: None,
        }
    }

    fn with_configuration(mut self, configuration_id: impl Into<String>) -> Self {
        self.configuration_id = Some(configuration_id.into());
        self
    }

    fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> GraphQueryErrorKind {
        self.kind
    }

    /// Returns the selected Configuration ID, when applicable.
    #[must_use]
    pub fn configuration_id(&self) -> Option<&str> {
        self.configuration_id.as_deref()
    }

    /// Returns the selected node ID, when applicable.
    #[must_use]
    pub fn node_id(&self) -> Option<&str> {
        self.node_id.as_deref()
    }
}

impl Display for GraphQueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            GraphQueryErrorKind::WorkspaceUnavailable => {
                formatter.write_str("Workspace snapshot is unavailable")
            }
            GraphQueryErrorKind::InvalidIdentifier => {
                formatter.write_str("Graph Query identifier must not be empty")
            }
            GraphQueryErrorKind::ConfigurationNotFound => {
                formatter.write_str("Graph Query configuration was not found")
            }
            GraphQueryErrorKind::NodeNotFound => {
                formatter.write_str("Graph Query node was not found")
            }
            GraphQueryErrorKind::LimitOutOfRange => {
                formatter.write_str("Graph Query limit must be between 1 and 100")
            }
            GraphQueryErrorKind::MaxDepthOutOfRange => {
                formatter.write_str("Graph Query max depth must be between 0 and 4")
            }
        }
    }
}

impl Error for GraphQueryError {}

/// Owned published Configuration projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryConfiguration {
    id: String,
    name: String,
    format: GraphQueryWorkspaceFormat,
    node_count: usize,
    edge_count: usize,
}

impl GraphQueryConfiguration {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn format(&self) -> GraphQueryWorkspaceFormat {
        self.format
    }

    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edge_count
    }
}

/// Owned bounded Configuration list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryConfigurationList {
    configurations: Vec<GraphQueryConfiguration>,
    truncated: bool,
}

impl GraphQueryConfigurationList {
    #[must_use]
    pub fn configurations(&self) -> &[GraphQueryConfiguration] {
        &self.configurations
    }

    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Owned node projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryNode {
    id: String,
    name: String,
    kind: GraphQueryNodeKind,
}

impl GraphQueryNode {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> GraphQueryNodeKind {
        self.kind
    }
}

/// Owned exact node lookup result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryNodeResult {
    configuration_id: String,
    node: GraphQueryNode,
}

impl GraphQueryNodeResult {
    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    #[must_use]
    pub const fn node(&self) -> &GraphQueryNode {
        &self.node
    }
}

/// Owned direct relation projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryRelation {
    edge_id: String,
    edge_kind: GraphQueryEdgeKind,
    source_node_id: String,
    target_node_id: String,
    related_node: GraphQueryNode,
}

impl GraphQueryRelation {
    #[must_use]
    pub fn edge_id(&self) -> &str {
        &self.edge_id
    }

    #[must_use]
    pub const fn edge_kind(&self) -> GraphQueryEdgeKind {
        self.edge_kind
    }

    #[must_use]
    pub fn source_node_id(&self) -> &str {
        &self.source_node_id
    }

    #[must_use]
    pub fn target_node_id(&self) -> &str {
        &self.target_node_id
    }

    #[must_use]
    pub const fn related_node(&self) -> &GraphQueryNode {
        &self.related_node
    }
}

/// Owned bounded direct relation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryRelationResult {
    configuration_id: String,
    node_id: String,
    direction: GraphQueryDirection,
    edge_kind: Option<GraphQueryEdgeKind>,
    relations: Vec<GraphQueryRelation>,
    truncated: bool,
}

impl GraphQueryRelationResult {
    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    #[must_use]
    pub const fn direction(&self) -> GraphQueryDirection {
        self.direction
    }

    #[must_use]
    pub const fn edge_kind(&self) -> Option<GraphQueryEdgeKind> {
        self.edge_kind
    }

    #[must_use]
    pub fn relations(&self) -> &[GraphQueryRelation] {
        &self.relations
    }

    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Owned breadth-first traversal node projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryTraversalNode {
    node: GraphQueryNode,
    depth: usize,
    via_edge_id: Option<String>,
}

impl GraphQueryTraversalNode {
    #[must_use]
    pub const fn node(&self) -> &GraphQueryNode {
        &self.node
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    pub fn via_edge_id(&self) -> Option<&str> {
        self.via_edge_id.as_deref()
    }
}

/// Owned bounded breadth-first traversal result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryTraversalResult {
    configuration_id: String,
    start_node_id: String,
    direction: GraphQueryDirection,
    edge_kind: Option<GraphQueryEdgeKind>,
    max_depth: usize,
    include_start: bool,
    nodes: Vec<GraphQueryTraversalNode>,
    truncated: bool,
}

impl GraphQueryTraversalResult {
    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    #[must_use]
    pub fn start_node_id(&self) -> &str {
        &self.start_node_id
    }

    #[must_use]
    pub const fn direction(&self) -> GraphQueryDirection {
        self.direction
    }

    #[must_use]
    pub const fn edge_kind(&self) -> Option<GraphQueryEdgeKind> {
        self.edge_kind
    }

    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    #[must_use]
    pub const fn include_start(&self) -> bool {
        self.include_start
    }

    #[must_use]
    pub fn nodes(&self) -> &[GraphQueryTraversalNode] {
        &self.nodes
    }

    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Cloneable read-only Graph Query component over published Workspace snapshots.
#[derive(Debug, Clone)]
pub struct GraphQueryService {
    workspace: WorkspaceSnapshotObserver,
}

impl GraphQueryService {
    /// Creates a transport-neutral query component from one Workspace observer.
    #[must_use]
    pub const fn new(workspace: WorkspaceSnapshotObserver) -> Self {
        Self { workspace }
    }

    /// Lists published configurations in canonical identity order.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when no Workspace snapshot is published.
    pub fn configurations(
        &self,
        limit: GraphQueryLimit,
    ) -> Result<GraphQueryConfigurationList, GraphQueryError> {
        let snapshot = self.snapshot()?;
        let configurations = snapshot
            .configurations()
            .iter()
            .map(configuration_projection)
            .collect();
        let (configurations, truncated) = bounded(configurations, limit);
        Ok(GraphQueryConfigurationList {
            configurations,
            truncated,
        })
    }

    /// Looks up one node in one exact published configuration.
    ///
    /// # Errors
    ///
    /// Returns typed identifier, availability, configuration, or node failures.
    pub fn node(
        &self,
        configuration_id: &str,
        node_id: &str,
    ) -> Result<GraphQueryNodeResult, GraphQueryError> {
        let configuration_id = parse_configuration_id(configuration_id)?;
        let node_id = parse_node_id(node_id)?;
        let snapshot = self.snapshot()?;
        let configuration = select_configuration(&snapshot, &configuration_id)?;
        let query = configuration.graph().query();
        let node = select_node(&query, &configuration_id, &node_id)?;

        Ok(GraphQueryNodeResult {
            configuration_id: configuration_id.as_str().to_owned(),
            node: node_projection(node),
        })
    }

    /// Lists direct graph relations for one exact node.
    ///
    /// # Errors
    ///
    /// Returns typed identifier, availability, configuration, or node failures.
    pub fn relations(
        &self,
        configuration_id: &str,
        node_id: &str,
        direction: GraphQueryDirection,
        edge_kind: Option<GraphQueryEdgeKind>,
        limit: GraphQueryLimit,
    ) -> Result<GraphQueryRelationResult, GraphQueryError> {
        let configuration_id = parse_configuration_id(configuration_id)?;
        let node_id = parse_node_id(node_id)?;
        let snapshot = self.snapshot()?;
        let configuration = select_configuration(&snapshot, &configuration_id)?;
        let query = configuration.graph().query();
        select_node(&query, &configuration_id, &node_id)?;

        let edges = match direction {
            GraphQueryDirection::Outgoing => query.outgoing_edges(&node_id),
            GraphQueryDirection::Incoming => query.incoming_edges(&node_id),
        };
        let relations = edges
            .into_iter()
            .filter(|edge| edge_kind.is_none_or(|kind| edge.kind() == kind.graph_kind()))
            .filter_map(|edge| relation_projection(&query, edge, direction))
            .collect();
        let (relations, truncated) = bounded(relations, limit);

        Ok(GraphQueryRelationResult {
            configuration_id: configuration_id.as_str().to_owned(),
            node_id: node_id.as_str().to_owned(),
            direction,
            edge_kind,
            relations,
            truncated,
        })
    }

    /// Performs one deterministic bounded breadth-first traversal.
    ///
    /// # Errors
    ///
    /// Returns typed identifier, availability, configuration, or node failures.
    #[allow(clippy::too_many_arguments)]
    pub fn traverse(
        &self,
        configuration_id: &str,
        node_id: &str,
        direction: GraphQueryDirection,
        edge_kind: Option<GraphQueryEdgeKind>,
        max_depth: GraphQueryMaxDepth,
        include_start: bool,
        limit: GraphQueryLimit,
    ) -> Result<GraphQueryTraversalResult, GraphQueryError> {
        let configuration_id = parse_configuration_id(configuration_id)?;
        let node_id = parse_node_id(node_id)?;
        let snapshot = self.snapshot()?;
        let configuration = select_configuration(&snapshot, &configuration_id)?;
        let query = configuration.graph().query();
        select_node(&query, &configuration_id, &node_id)?;

        let mut options =
            SemanticGraphTraversalOptions::new(direction.traversal(), max_depth.get())
                .with_include_start(include_start);
        if let Some(edge_kind) = edge_kind {
            options = options.with_edge_kind(edge_kind.graph_kind());
        }

        let nodes = query
            .traverse(&node_id, &options)
            .into_iter()
            .filter_map(|record| {
                query
                    .node(record.node_id())
                    .map(|node| GraphQueryTraversalNode {
                        node: node_projection(node),
                        depth: record.depth(),
                        via_edge_id: record.via_edge().map(ToString::to_string),
                    })
            })
            .collect();
        let (nodes, truncated) = bounded(nodes, limit);

        Ok(GraphQueryTraversalResult {
            configuration_id: configuration_id.as_str().to_owned(),
            start_node_id: node_id.as_str().to_owned(),
            direction,
            edge_kind,
            max_depth: max_depth.get(),
            include_start,
            nodes,
            truncated,
        })
    }

    fn snapshot(&self) -> Result<std::sync::Arc<WorkspaceSnapshot>, GraphQueryError> {
        self.workspace
            .snapshot()
            .ok_or_else(|| GraphQueryError::new(GraphQueryErrorKind::WorkspaceUnavailable))
    }
}

fn parse_configuration_id(value: &str) -> Result<EntityId, GraphQueryError> {
    EntityId::new(value.to_owned()).map_err(|_| {
        GraphQueryError::new(GraphQueryErrorKind::InvalidIdentifier).with_configuration(value)
    })
}

fn parse_node_id(value: &str) -> Result<NodeId, GraphQueryError> {
    if value.trim().is_empty() {
        Err(GraphQueryError::new(GraphQueryErrorKind::InvalidIdentifier).with_node(value))
    } else {
        Ok(NodeId::new(value))
    }
}

fn select_configuration<'snapshot>(
    snapshot: &'snapshot WorkspaceSnapshot,
    configuration_id: &EntityId,
) -> Result<&'snapshot WorkspaceConfigurationSnapshot, GraphQueryError> {
    snapshot.configuration(configuration_id).ok_or_else(|| {
        GraphQueryError::new(GraphQueryErrorKind::ConfigurationNotFound)
            .with_configuration(configuration_id.as_str())
    })
}

fn select_node<'graph>(
    query: &SemanticGraphQuery<'graph>,
    configuration_id: &EntityId,
    node_id: &NodeId,
) -> Result<&'graph GraphNode, GraphQueryError> {
    query.node(node_id).ok_or_else(|| {
        GraphQueryError::new(GraphQueryErrorKind::NodeNotFound)
            .with_configuration(configuration_id.as_str())
            .with_node(node_id.as_str())
    })
}

fn configuration_projection(
    configuration: &WorkspaceConfigurationSnapshot,
) -> GraphQueryConfiguration {
    GraphQueryConfiguration {
        id: configuration.configuration_id().as_str().to_owned(),
        name: configuration.configuration_name().as_str().to_owned(),
        format: GraphQueryWorkspaceFormat::from_workspace_format(configuration.format()),
        node_count: configuration.graph().node_count(),
        edge_count: configuration.graph().edge_count(),
    }
}

fn node_projection(node: &GraphNode) -> GraphQueryNode {
    GraphQueryNode {
        id: node.id().as_str().to_owned(),
        name: node.name().as_str().to_owned(),
        kind: node.kind().into(),
    }
}

fn relation_projection(
    query: &SemanticGraphQuery<'_>,
    edge: &GraphEdge,
    direction: GraphQueryDirection,
) -> Option<GraphQueryRelation> {
    let related_id = match direction {
        GraphQueryDirection::Outgoing => edge.target(),
        GraphQueryDirection::Incoming => edge.source(),
    };
    let related_node = query.node_by_entity_id(related_id)?;
    let source = NodeId::new(edge.source().as_str());
    let target = NodeId::new(edge.target().as_str());

    Some(GraphQueryRelation {
        edge_id: SemanticGraphQuery::edge_id(&source, &target, edge.kind()).into_inner(),
        edge_kind: edge.kind().into(),
        source_node_id: edge.source().as_str().to_owned(),
        target_node_id: edge.target().as_str().to_owned(),
        related_node: node_projection(related_node),
    })
}

fn bounded<T>(mut values: Vec<T>, limit: GraphQueryLimit) -> (Vec<T>, bool) {
    let truncated = values.len() > limit.get();
    values.truncate(limit.get());
    (values, truncated)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use oneagent_analysis::diagnostics::{DiagnosticEngine, DiagnosticPolicy};
    use oneagent_analysis::rules::RuleExecutionReport;
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticDiagnostic, SemanticGraph,
        SemanticGraphReport, SemanticReferenceRequestLedger, SemanticReferenceStatistics,
    };
    use oneagent_metadata::MetadataKind;
    use oneagent_workspace::WorkspaceFormat;
    use tokio::sync::watch;

    use super::{
        GraphQueryDirection, GraphQueryEdgeKind, GraphQueryErrorKind, GraphQueryLimit,
        GraphQueryMaxDepth, GraphQueryMetadataKind, GraphQueryNodeKind, GraphQueryService,
        GraphQueryWorkspaceFormat,
    };
    use crate::workspace::{
        WorkspaceConfigurationSnapshot, WorkspaceSnapshot, WorkspaceSnapshotObserver,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("test identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("test name must be valid")
    }

    fn node(value: &str, kind: NodeKind) -> GraphNode {
        GraphNode::new(id(value), name(value), kind)
    }

    fn configuration(
        configuration_id: &str,
        format: WorkspaceFormat,
        graph: SemanticGraph,
    ) -> WorkspaceConfigurationSnapshot {
        let report = SemanticGraphReport::from_graph(&graph);
        let validation = graph.validate();
        let diagnostic_report = DiagnosticEngine
            .build(&[], &validation, &DiagnosticPolicy::default())
            .expect("test graph diagnostics must build");
        WorkspaceConfigurationSnapshot {
            root_path: PathBuf::from(configuration_id),
            format,
            configuration_id: id(configuration_id),
            configuration_name: name(configuration_id),
            graph: Arc::new(graph),
            diagnostics: Arc::<[SemanticDiagnostic]>::from([]),
            reference_requests: Arc::new(SemanticReferenceRequestLedger::new()),
            reference_statistics: SemanticReferenceStatistics::new(),
            report,
            validation: Arc::new(validation),
            rule_execution_report: Arc::new(RuleExecutionReport::default()),
            diagnostic_report: Arc::new(diagnostic_report),
        }
    }

    fn snapshot(mut configurations: Vec<WorkspaceConfigurationSnapshot>) -> WorkspaceSnapshot {
        configurations.sort_by(|left, right| left.configuration_id.cmp(&right.configuration_id));
        WorkspaceSnapshot::initial(std::path::PathBuf::new(), configurations)
    }

    fn service(
        snapshot: Option<WorkspaceSnapshot>,
    ) -> (
        GraphQueryService,
        watch::Sender<Option<Arc<WorkspaceSnapshot>>>,
    ) {
        let (sender, receiver) = watch::channel(snapshot.map(Arc::new));
        (
            GraphQueryService::new(WorkspaceSnapshotObserver { snapshot: receiver }),
            sender,
        )
    }

    fn relation_graph() -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        for (value, kind) in [
            ("a", NodeKind::Metadata(MetadataKind::Configuration)),
            ("b", NodeKind::Module),
            ("c", NodeKind::Procedure),
        ] {
            graph.insert_node(node(value, kind));
        }
        graph
            .insert_edge(GraphEdge::new(id("a"), id("b"), EdgeKind::Contains))
            .expect("contains edge must insert");
        graph
            .insert_edge(GraphEdge::new(id("a"), id("c"), EdgeKind::Calls))
            .expect("calls edge must insert");
        graph
            .insert_edge(GraphEdge::new(id("c"), id("a"), EdgeKind::References))
            .expect("cycle edge must insert");
        graph
    }

    #[test]
    fn configuration_listing_distinguishes_unavailable_empty_ordered_and_truncated() {
        let (queries, sender) = service(None);
        assert_eq!(
            queries
                .configurations(GraphQueryLimit::default())
                .expect_err("missing snapshot must fail")
                .kind(),
            GraphQueryErrorKind::WorkspaceUnavailable
        );

        sender.send_replace(Some(Arc::new(WorkspaceSnapshot::default())));
        let empty = queries
            .configurations(GraphQueryLimit::default())
            .expect("empty published snapshot must succeed");
        assert!(empty.configurations().is_empty());
        assert!(!empty.truncated());

        let configurations = (0..=100)
            .rev()
            .map(|index| {
                let identifier = format!("configuration-{index:03}");
                configuration(&identifier, WorkspaceFormat::Edt, SemanticGraph::new())
            })
            .collect();
        sender.send_replace(Some(Arc::new(snapshot(configurations))));
        let listed = queries
            .configurations(GraphQueryLimit::new(100).expect("maximum limit must be valid"))
            .expect("published configurations must list");
        assert_eq!(listed.configurations().len(), 100);
        assert!(listed.truncated());
        assert_eq!(listed.configurations()[0].id(), "configuration-000");
        assert_eq!(listed.configurations()[99].id(), "configuration-099");
    }

    #[test]
    fn node_lookup_is_exact_owned_and_keeps_error_categories_distinct() {
        let graph = relation_graph();
        let (queries, _sender) = service(Some(snapshot(vec![configuration(
            "configuration",
            WorkspaceFormat::DesignerXml,
            graph,
        )])));

        let result = queries
            .node("configuration", "a")
            .expect("known node must resolve");
        assert_eq!(result.configuration_id(), "configuration");
        assert_eq!(result.node().id(), "a");
        assert_eq!(result.node().name(), "a");
        assert_eq!(result.node().kind().as_str(), "metadata");
        assert_eq!(
            result.node().kind().metadata_kind(),
            Some(GraphQueryMetadataKind::Configuration)
        );

        let unknown_configuration = queries
            .node("missing", "a")
            .expect_err("unknown configuration must fail");
        assert_eq!(
            unknown_configuration.kind(),
            GraphQueryErrorKind::ConfigurationNotFound
        );
        assert_eq!(unknown_configuration.configuration_id(), Some("missing"));

        let unknown_node = queries
            .node("configuration", "missing")
            .expect_err("unknown node must fail");
        assert_eq!(unknown_node.kind(), GraphQueryErrorKind::NodeNotFound);
        assert_eq!(unknown_node.node_id(), Some("missing"));

        for invalid in ["", "   "] {
            assert_eq!(
                queries
                    .node(invalid, "a")
                    .expect_err("empty configuration ID must fail")
                    .kind(),
                GraphQueryErrorKind::InvalidIdentifier
            );
            assert_eq!(
                queries
                    .node("configuration", invalid)
                    .expect_err("empty node ID must fail")
                    .kind(),
                GraphQueryErrorKind::InvalidIdentifier
            );
        }
    }

    #[test]
    fn direct_relations_preserve_direction_filter_order_and_truncation() {
        let (queries, _sender) = service(Some(snapshot(vec![configuration(
            "configuration",
            WorkspaceFormat::Edt,
            relation_graph(),
        )])));

        let outgoing = queries
            .relations(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                None,
                GraphQueryLimit::default(),
            )
            .expect("outgoing relations must resolve");
        assert_eq!(outgoing.relations().len(), 2);
        assert_eq!(outgoing.relations()[0].related_node().id(), "b");
        assert_eq!(outgoing.relations()[1].related_node().id(), "c");
        assert_eq!(outgoing.direction(), GraphQueryDirection::Outgoing);
        assert_eq!(outgoing.edge_kind(), None);

        let contains = queries
            .relations(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                Some(GraphQueryEdgeKind::Contains),
                GraphQueryLimit::default(),
            )
            .expect("filtered relation must resolve");
        assert_eq!(contains.relations().len(), 1);
        assert_eq!(contains.relations()[0].edge_kind().as_str(), "contains");
        assert_eq!(contains.relations()[0].source_node_id(), "a");
        assert_eq!(contains.relations()[0].target_node_id(), "b");

        let incoming = queries
            .relations(
                "configuration",
                "a",
                GraphQueryDirection::Incoming,
                None,
                GraphQueryLimit::new(1).expect("one must be a valid limit"),
            )
            .expect("incoming relations must resolve");
        assert_eq!(incoming.relations().len(), 1);
        assert!(!incoming.truncated());
        assert_eq!(incoming.relations()[0].related_node().id(), "c");

        let truncated = queries
            .relations(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                None,
                GraphQueryLimit::new(1).expect("one must be a valid limit"),
            )
            .expect("bounded relations must resolve");
        assert_eq!(truncated.relations().len(), 1);
        assert!(truncated.truncated());

        let known_empty = queries
            .relations(
                "configuration",
                "b",
                GraphQueryDirection::Outgoing,
                Some(GraphQueryEdgeKind::Writes),
                GraphQueryLimit::default(),
            )
            .expect("known node with no relation must succeed");
        assert!(known_empty.relations().is_empty());
        assert!(!known_empty.truncated());
    }

    #[test]
    fn traversal_is_depth_bounded_cycle_safe_owned_and_truncated() {
        let (queries, _sender) = service(Some(snapshot(vec![configuration(
            "configuration",
            WorkspaceFormat::Edt,
            relation_graph(),
        )])));

        let depth_zero = queries
            .traverse(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                None,
                GraphQueryMaxDepth::new(0).expect("zero depth must be valid"),
                false,
                GraphQueryLimit::default(),
            )
            .expect("zero depth traversal must succeed");
        assert!(depth_zero.nodes().is_empty());

        let included = queries
            .traverse(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                None,
                GraphQueryMaxDepth::new(0).expect("zero depth must be valid"),
                true,
                GraphQueryLimit::default(),
            )
            .expect("included start must succeed");
        assert_eq!(included.nodes().len(), 1);
        assert_eq!(included.nodes()[0].depth(), 0);
        assert_eq!(included.nodes()[0].via_edge_id(), None);

        let traversal = queries
            .traverse(
                "configuration",
                "a",
                GraphQueryDirection::Outgoing,
                None,
                GraphQueryMaxDepth::new(4).expect("maximum depth must be valid"),
                true,
                GraphQueryLimit::new(2).expect("two must be a valid limit"),
            )
            .expect("cycle traversal must succeed");
        assert_eq!(traversal.nodes().len(), 2);
        assert!(traversal.truncated());
        assert_eq!(traversal.nodes()[0].node().id(), "a");
        assert_eq!(traversal.nodes()[1].depth(), 1);
        assert!(traversal.nodes()[1].via_edge_id().is_some());
        assert_eq!(traversal.max_depth(), 4);
        assert!(traversal.include_start());
    }

    #[test]
    fn bounds_and_closed_value_parsers_are_total() {
        assert_eq!(GraphQueryLimit::default().get(), 50);
        assert_eq!(GraphQueryLimit::new(1).expect("minimum must work").get(), 1);
        assert_eq!(
            GraphQueryLimit::new(100).expect("maximum must work").get(),
            100
        );
        for invalid in [0, 101, usize::MAX] {
            assert_eq!(
                GraphQueryLimit::new(invalid)
                    .expect_err("invalid limit must fail")
                    .kind(),
                GraphQueryErrorKind::LimitOutOfRange
            );
        }
        assert_eq!(
            GraphQueryMaxDepth::new(0)
                .expect("minimum depth must work")
                .get(),
            0
        );
        assert_eq!(
            GraphQueryMaxDepth::new(4)
                .expect("maximum depth must work")
                .get(),
            4
        );
        assert_eq!(
            GraphQueryMaxDepth::new(5)
                .expect_err("over-limit depth must fail")
                .kind(),
            GraphQueryErrorKind::MaxDepthOutOfRange
        );
        assert_eq!(
            GraphQueryDirection::from_name("outgoing"),
            Some(GraphQueryDirection::Outgoing)
        );
        assert_eq!(GraphQueryDirection::from_name("OUTGOING"), None);
        assert_eq!(
            GraphQueryEdgeKind::from_name("depends_on"),
            Some(GraphQueryEdgeKind::DependsOn)
        );
        assert_eq!(GraphQueryEdgeKind::from_name("depends-on"), None);
    }

    #[test]
    fn domain_kind_mappings_are_exhaustive_and_stable() {
        let node_kinds = [
            (NodeKind::Module, "module"),
            (NodeKind::Procedure, "procedure"),
            (NodeKind::Function, "function"),
            (NodeKind::Query, "query"),
            (NodeKind::DataCompositionSchema, "data_composition_schema"),
            (NodeKind::DataSet, "data_set"),
            (NodeKind::DataCompositionField, "data_composition_field"),
            (NodeKind::XdtoType, "xdto_type"),
            (
                NodeKind::HttpServiceUrlTemplate,
                "http_service_url_template",
            ),
            (NodeKind::HttpServiceMethod, "http_service_method"),
            (NodeKind::WebServiceOperation, "web_service_operation"),
            (NodeKind::WebServiceParameter, "web_service_parameter"),
            (NodeKind::Form, "form"),
            (NodeKind::Command, "command"),
            (NodeKind::Attribute, "attribute"),
            (NodeKind::StandardAttribute, "standard_attribute"),
            (NodeKind::TabularSection, "tabular_section"),
            (NodeKind::Dimension, "dimension"),
            (NodeKind::Resource, "resource"),
            (NodeKind::Measure, "measure"),
            (NodeKind::Role, "role"),
            (NodeKind::AccessRight, "access_right"),
            (NodeKind::Subsystem, "subsystem"),
            (NodeKind::Unknown, "unknown"),
        ];
        for (domain, stable) in node_kinds {
            let mapped = GraphQueryNodeKind::from(domain);
            assert_eq!(mapped.as_str(), stable);
            assert_eq!(mapped.metadata_kind(), None);
        }

        let metadata_kinds = [
            MetadataKind::Configuration,
            MetadataKind::Subsystem,
            MetadataKind::Catalog,
            MetadataKind::Document,
            MetadataKind::Enumeration,
            MetadataKind::CommonModule,
            MetadataKind::Report,
            MetadataKind::DataProcessor,
            MetadataKind::InformationRegister,
            MetadataKind::AccumulationRegister,
            MetadataKind::AccountingRegister,
            MetadataKind::CalculationRegister,
            MetadataKind::BusinessProcess,
            MetadataKind::Task,
            MetadataKind::Role,
            MetadataKind::CommonForm,
            MetadataKind::Form,
            MetadataKind::Command,
            MetadataKind::Template,
            MetadataKind::HttpService,
            MetadataKind::WebService,
            MetadataKind::XdtoPackage,
            MetadataKind::EventSubscription,
            MetadataKind::Unknown,
        ];
        for domain in metadata_kinds {
            let mapped = GraphQueryNodeKind::from(NodeKind::Metadata(domain));
            assert_eq!(mapped.as_str(), "metadata");
            assert_eq!(
                mapped
                    .metadata_kind()
                    .expect("metadata node must retain nested kind")
                    .as_str(),
                domain.as_str()
            );
        }

        let edge_kinds = [
            EdgeKind::Contains,
            EdgeKind::Calls,
            EdgeKind::References,
            EdgeKind::Reads,
            EdgeKind::Writes,
            EdgeKind::Grants,
            EdgeKind::Includes,
            EdgeKind::Extends,
            EdgeKind::DependsOn,
            EdgeKind::Opens,
            EdgeKind::Triggers,
        ];
        for domain in edge_kinds {
            let mapped = GraphQueryEdgeKind::from(domain);
            assert_eq!(mapped.graph_kind(), domain);
            assert_eq!(GraphQueryEdgeKind::from_name(mapped.as_str()), Some(mapped));
        }
        assert_eq!(GraphQueryWorkspaceFormat::Edt.as_str(), "edt");
        assert_eq!(
            GraphQueryWorkspaceFormat::DesignerXml.as_str(),
            "designer_xml"
        );
    }

    #[test]
    fn each_call_owns_one_snapshot_and_repeated_results_are_equal() {
        let first_graph = relation_graph();
        let (queries, sender) = service(Some(snapshot(vec![configuration(
            "first",
            WorkspaceFormat::Edt,
            first_graph,
        )])));

        let first = queries
            .configurations(GraphQueryLimit::default())
            .expect("first snapshot must list");
        let repeated = queries
            .configurations(GraphQueryLimit::default())
            .expect("repeated query must list");
        assert_eq!(first, repeated);

        sender.send_replace(Some(Arc::new(snapshot(vec![configuration(
            "second",
            WorkspaceFormat::DesignerXml,
            SemanticGraph::new(),
        )]))));
        assert_eq!(first.configurations()[0].id(), "first");
        let second = queries
            .configurations(GraphQueryLimit::default())
            .expect("replacement snapshot must list");
        assert_eq!(second.configurations()[0].id(), "second");
    }
}
