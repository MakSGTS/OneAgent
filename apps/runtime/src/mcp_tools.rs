//! Read-only semantic MCP tools over immutable workspace snapshots.

use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;

use oneagent_analysis::context::{
    ContextEngine, ContextError, ContextInclusionReason, ContextIntent, ContextPolicy,
    ContextRelationDirection, ContextRequest, ContextSeed, ContextTraversalDirection,
};
use oneagent_common::EntityId;
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, ImpactNodeStatus, ImpactPropagationDirection, ImpactReasonKind,
    ImpactSeedKind, ImpactSnapshot, NodeId, SemanticDiagnosticKind, SemanticDiagnosticSeverity,
    SemanticGraphQuery, SemanticGraphValidationIssueKind, SemanticGraphValidationSeverity,
    SemanticImpactAnalyzer, SemanticImpactOptions,
};
use oneagent_protocol::{
    McpServer, McpToolAnnotations, McpToolCallHandler, McpToolCallOutcome, McpToolDefinition,
    McpToolFuture,
};
use oneagent_tool_policy::{
    ActorId, ActorScope, NeverCancelled, PolicyRevision, RuleAction, ToolArguments, ToolEffect,
    ToolExecutor, ToolExecutorOutcome, ToolFuture, ToolId, ToolOutput, ToolPolicy, ToolRequest,
    ToolRequestId, ToolRule, ToolScope, ToolTerminalOutcome, execute_tool,
};
use oneagent_workspace::WorkspaceFormat;
use serde_json::{Map, Value, json};

use crate::{
    GraphQueryEdgeKind, GraphQueryNodeKind, WorkspaceConfigurationSnapshot, WorkspaceSnapshot,
};

const GRAPH: &str = "oneagent.graph";
const QUERY: &str = "oneagent.query";
const VALIDATION: &str = "oneagent.validation";
const DIAGNOSTICS: &str = "oneagent.diagnostics";
const IMPACT: &str = "oneagent.impact";
const CONTEXT: &str = "oneagent.context";
const ACTOR: &str = "oneagent.mcp";
const REQUEST: &str = "oneagent.mcp.request";
const REVISION: &str = "oneagent.mcp.read-only.v1";
const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;
const MAX_DEPTH: usize = 4;
const EDGE_KIND_NAMES: [&str; 11] = [
    "contains",
    "calls",
    "references",
    "reads",
    "writes",
    "grants",
    "includes",
    "extends",
    "depends_on",
    "opens",
    "triggers",
];

/// Closed construction failure for a semantic MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpSemanticServerError;

/// Builds the complete immutable Sprint 29 semantic MCP server.
///
/// # Errors
///
/// Returns a closed error when a fixed catalog or policy invariant fails.
pub fn semantic_server(snapshot: WorkspaceSnapshot) -> Result<McpServer, McpSemanticServerError> {
    let names = [CONTEXT, DIAGNOSTICS, GRAPH, IMPACT, QUERY, VALIDATION];
    let mut catalog = definitions()?;
    catalog.extend(impact_context_definitions()?);
    build_server(snapshot, catalog, &names)
}

fn build_server(
    snapshot: WorkspaceSnapshot,
    catalog: Vec<McpToolDefinition>,
    names: &[&str],
) -> Result<McpServer, McpSemanticServerError> {
    McpServer::with_tools(
        catalog,
        Handler {
            snapshot: Arc::new(snapshot),
            policy: policy(names)?,
        },
    )
    .map_err(|_| McpSemanticServerError)
}

fn impact_context_definitions() -> Result<Vec<McpToolDefinition>, McpSemanticServerError> {
    [
        (
            IMPACT,
            "Analyze bounded semantic impact between two configurations.",
            json!({
                "type": "object",
                "properties": {
                    "previousConfigurationId": {"type": "string"},
                    "currentConfigurationId": {"type": "string"},
                    "maxDepth": {"type": "integer", "minimum": 0, "maximum": MAX_DEPTH},
                    "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
                },
                "required": ["previousConfigurationId", "currentConfigurationId"],
                "additionalProperties": false
            }),
        ),
        (
            CONTEXT,
            "Build bounded semantic context around one exact node.",
            json!({
                "type": "object",
                "properties": {
                    "configurationId": {"type": "string"},
                    "nodeId": {"type": "string"},
                    "direction": {"enum": ["incoming", "outgoing", "both"]},
                    "maxDepth": {"type": "integer", "minimum": 0, "maximum": MAX_DEPTH},
                    "maxCandidates": {"type": "integer", "minimum": 1, "maximum": 128},
                    "budgetBytes": {"type": "integer", "minimum": 1, "maximum": 32768}
                },
                "required": ["configurationId", "nodeId"],
                "additionalProperties": false
            }),
        ),
    ]
    .into_iter()
    .map(|(name, description, schema)| {
        McpToolDefinition::new(
            name,
            description,
            schema.as_object().cloned().ok_or(McpSemanticServerError)?,
            McpToolAnnotations::read_only(),
        )
        .map_err(|_| McpSemanticServerError)
    })
    .collect()
}

fn definitions() -> Result<Vec<McpToolDefinition>, McpSemanticServerError> {
    let limited = || {
        json!({
            "type": "object",
            "properties": {
                "configurationId": {"type": "string"},
                "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
            },
            "required": ["configurationId"],
            "additionalProperties": false
        })
    };
    [
        (
            GRAPH,
            "Summarize immutable OneAgent semantic graphs.",
            json!({
                "type": "object",
                "properties": {
                    "configurationId": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
                },
                "additionalProperties": false
            }),
        ),
        (
            QUERY,
            "Query immutable OneAgent semantic graph nodes and relations.",
            json!({
                "type": "object",
                "properties": {
                    "configurationId": {"type": "string"},
                    "operation": {"enum": ["node", "relations", "traverse"]},
                    "nodeId": {"type": "string"},
                    "direction": {"enum": ["incoming", "outgoing", "both"]},
                    "edgeKinds": {"type": "array", "items": {"enum": EDGE_KIND_NAMES}, "uniqueItems": true},
                    "maxDepth": {"type": "integer", "minimum": 0, "maximum": MAX_DEPTH},
                    "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
                },
                "required": ["configurationId", "operation", "nodeId"],
                "additionalProperties": false
            }),
        ),
        (VALIDATION, "Validate one immutable OneAgent semantic graph.", limited()),
        (
            DIAGNOSTICS,
            "List bounded source-independent OneAgent semantic diagnostics.",
            limited(),
        ),
    ]
    .into_iter()
    .map(|(name, description, schema)| {
        McpToolDefinition::new(
            name,
            description,
            schema.as_object().cloned().ok_or(McpSemanticServerError)?,
            McpToolAnnotations::read_only(),
        )
        .map_err(|_| McpSemanticServerError)
    })
    .collect()
}

fn policy(names: &[&str]) -> Result<ToolPolicy, McpSemanticServerError> {
    let actor = ActorId::new(ACTOR).map_err(|_| McpSemanticServerError)?;
    let rules = names
        .iter()
        .map(|name| {
            Ok(ToolRule::new(
                ActorScope::Exact(actor.clone()),
                ToolScope::Exact(ToolId::new(*name).map_err(|_| McpSemanticServerError)?),
                ToolEffect::ReadOnly,
                RuleAction::Allow,
            ))
        })
        .collect::<Result<Vec<_>, McpSemanticServerError>>()?;
    ToolPolicy::new(
        PolicyRevision::new(REVISION).map_err(|_| McpSemanticServerError)?,
        rules,
    )
    .map_err(|_| McpSemanticServerError)
}

struct Handler {
    snapshot: Arc<WorkspaceSnapshot>,
    policy: ToolPolicy,
}

impl McpToolCallHandler for Handler {
    fn call<'a>(&'a self, name: &'a str, arguments: &'a Map<String, Value>) -> McpToolFuture<'a> {
        Box::pin(async move {
            let Ok(encoded) = serde_json::to_string(arguments) else {
                return McpToolCallOutcome::Internal;
            };
            let Ok(request) = request(name, encoded) else {
                return tool_error("invalid_arguments");
            };
            let executor = Executor {
                snapshot: &self.snapshot,
            };
            let result = execute_tool(
                self.policy.evaluate(request),
                None,
                &executor,
                &NeverCancelled,
            )
            .await;
            match result.audit().terminal_outcome() {
                ToolTerminalOutcome::Completed => result
                    .output()
                    .and_then(|output| serde_json::from_str(output.expose()).ok())
                    .map_or_else(|| McpToolCallOutcome::Internal, |value| decode(&value)),
                ToolTerminalOutcome::Denied(_) => tool_error("policy_denied"),
                ToolTerminalOutcome::Partial
                | ToolTerminalOutcome::Failed
                | ToolTerminalOutcome::TimedOut
                | ToolTerminalOutcome::Cancelled => tool_error("execution_failed"),
            }
        })
    }
}

fn request(name: &str, arguments: String) -> Result<ToolRequest, ()> {
    ToolRequest::new(
        ToolRequestId::new(REQUEST).map_err(|_| ())?,
        ActorId::new(ACTOR).map_err(|_| ())?,
        ToolId::new(name).map_err(|_| ())?,
        ToolArguments::new(arguments).map_err(|_| ())?,
        [ToolEffect::ReadOnly],
    )
    .map_err(|_| ())
}

struct Executor<'a> {
    snapshot: &'a WorkspaceSnapshot,
}

impl ToolExecutor for Executor<'_> {
    fn execute<'a>(
        &'a self,
        request: &'a ToolRequest,
        _cancellation: &'a dyn oneagent_tool_policy::ToolCancellationSignal,
    ) -> ToolFuture<'a, ToolExecutorOutcome> {
        Box::pin(async move {
            let Ok(arguments) =
                serde_json::from_str::<Map<String, Value>>(request.arguments().expose())
            else {
                return ToolExecutorOutcome::Failed(None);
            };
            let envelope = match project(self.snapshot, request.tool().as_str(), &arguments) {
                Ok(value) => json!({"ok": true, "value": value}),
                Err(error) => json!({
                    "ok": false,
                    "error": {"code": error.code, "message": error.message}
                }),
            };
            output(&envelope)
        })
    }
}

fn output(envelope: &Value) -> ToolExecutorOutcome {
    let Ok(encoded) = serde_json::to_string(&envelope) else {
        return ToolExecutorOutcome::Failed(None);
    };
    if let Ok(output) = ToolOutput::new(encoded) {
        ToolExecutorOutcome::Completed(output)
    } else {
        let fallback = json!({
            "ok": false,
            "error": {
                "code": "result_too_large",
                "message": "The semantic tool result exceeds the output limit."
            }
        });
        serde_json::to_string(&fallback)
            .ok()
            .and_then(|value| ToolOutput::new(value).ok())
            .map_or(
                ToolExecutorOutcome::Failed(None),
                ToolExecutorOutcome::Completed,
            )
    }
}

fn decode(envelope: &Value) -> McpToolCallOutcome {
    if envelope.get("ok") == Some(&Value::Bool(true)) {
        return envelope
            .get("value")
            .cloned()
            .map_or(McpToolCallOutcome::Internal, McpToolCallOutcome::Success);
    }
    let error = envelope.get("error").and_then(Value::as_object);
    match error.map(|error| {
        (
            error.get("code").and_then(Value::as_str),
            error.get("message").and_then(Value::as_str),
        )
    }) {
        Some((Some(code), Some(message))) => McpToolCallOutcome::error(code, message),
        _ => McpToolCallOutcome::Internal,
    }
}

fn tool_error(code: &str) -> McpToolCallOutcome {
    let message = match code {
        "invalid_arguments" => "The semantic tool arguments are invalid.",
        "not_found" => "The requested semantic entity was not found.",
        "policy_denied" => "The semantic tool request was denied.",
        "result_too_large" => "The semantic tool result exceeds the output limit.",
        _ => "The semantic tool request failed.",
    };
    McpToolCallOutcome::error(code, message)
}

#[derive(Clone, Copy)]
struct SemanticError {
    code: &'static str,
    message: &'static str,
}

const INVALID: SemanticError = SemanticError {
    code: "invalid_arguments",
    message: "The semantic tool arguments are invalid.",
};
const NOT_FOUND: SemanticError = SemanticError {
    code: "not_found",
    message: "The requested semantic entity was not found.",
};
const EXECUTION_FAILED: SemanticError = SemanticError {
    code: "execution_failed",
    message: "The semantic tool request failed.",
};

fn project(
    snapshot: &WorkspaceSnapshot,
    name: &str,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    match name {
        GRAPH => graph(snapshot, arguments),
        QUERY => query(snapshot, arguments),
        VALIDATION => validation(snapshot, arguments),
        DIAGNOSTICS => diagnostics(snapshot, arguments),
        IMPACT => impact(snapshot, arguments),
        CONTEXT => context(snapshot, arguments),
        _ => Err(INVALID),
    }
}

fn graph(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(arguments, &["configurationId", "limit"])?;
    let limit = limit(arguments)?;
    let selected = optional_string(arguments, "configurationId")?;
    let mut configurations = snapshot
        .configurations()
        .iter()
        .filter(|item| selected.is_none_or(|id| id == item.configuration_id().as_str()))
        .map(summary)
        .collect::<Vec<_>>();
    if selected.is_some() && configurations.is_empty() {
        return Err(NOT_FOUND);
    }
    let total = configurations.len();
    configurations.truncate(limit);
    Ok(json!({"configurations": configurations, "total": total, "truncated": total > limit}))
}

fn summary(configuration: &WorkspaceConfigurationSnapshot) -> Value {
    let validation = configuration.graph().validate();
    let references = configuration.reference_statistics();
    let format = match configuration.format() {
        WorkspaceFormat::Edt => "edt",
        WorkspaceFormat::DesignerXml => "designer_xml",
        WorkspaceFormat::Extension | WorkspaceFormat::Unknown => "unsupported",
    };
    json!({
        "id": configuration.configuration_id().as_str(),
        "name": configuration.configuration_name().as_str(),
        "format": format,
        "nodeCount": configuration.graph().node_count(),
        "edgeCount": configuration.graph().edges().count(),
        "diagnosticCount": configuration.diagnostics().len(),
        "validation": {
            "valid": validation.is_valid(),
            "errors": validation.error_count(),
            "warnings": validation.warning_count()
        },
        "references": {
            "total": references.total(),
            "resolved": references.resolved(),
            "unresolved": references.unresolved()
        }
    })
}

fn query(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(
        arguments,
        &[
            "configurationId",
            "operation",
            "nodeId",
            "direction",
            "edgeKinds",
            "maxDepth",
            "limit",
        ],
    )?;
    let configuration = configuration(snapshot, string(arguments, "configurationId")?)?;
    let node_id = EntityId::new(string(arguments, "nodeId")?.to_owned()).map_err(|_| INVALID)?;
    let node = configuration.graph().node(&node_id).ok_or(NOT_FOUND)?;
    match string(arguments, "operation")? {
        "node" => {
            absent(arguments, &["direction", "edgeKinds", "maxDepth", "limit"])?;
            Ok(json!({
                "configurationId": configuration.configuration_id().as_str(),
                "node": node_value(node)
            }))
        }
        "relations" => {
            absent(arguments, &["maxDepth"])?;
            relations(configuration, &node_id, arguments)
        }
        "traverse" => traverse(configuration, &node_id, arguments),
        _ => Err(INVALID),
    }
}

fn relations(
    configuration: &WorkspaceConfigurationSnapshot,
    node_id: &EntityId,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    let direction = direction(arguments)?;
    let kinds = kinds(arguments)?;
    let limit = limit(arguments)?;
    let mut edges = edges(configuration, node_id, direction);
    edges.retain(|edge| kinds.is_empty() || kinds.contains(&edge.kind()));
    let total = edges.len();
    edges.truncate(limit);
    let relations = edges
        .into_iter()
        .map(|edge| relation_value(configuration, node_id, edge))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "nodeId": node_id.as_str(),
        "direction": direction,
        "relations": relations,
        "total": total,
        "truncated": total > limit
    }))
}

fn traverse(
    configuration: &WorkspaceConfigurationSnapshot,
    start: &EntityId,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    let direction = direction(arguments)?;
    let kinds = kinds(arguments)?;
    let depth = number(arguments, "maxDepth")?.unwrap_or(2);
    if depth > MAX_DEPTH {
        return Err(INVALID);
    }
    let limit = limit(arguments)?;
    let mut queue = VecDeque::from([(start.clone(), 0_usize, None::<String>)]);
    let mut visited = BTreeSet::from([start.clone()]);
    let mut nodes = Vec::new();
    while let Some((current, current_depth, via_edge_id)) = queue.pop_front() {
        let node = configuration.graph().node(&current).ok_or(NOT_FOUND)?;
        nodes.push(json!({
            "node": node_value(node),
            "depth": current_depth,
            "viaEdgeId": via_edge_id
        }));
        if current_depth == depth {
            continue;
        }
        for edge in edges(configuration, &current, direction) {
            if !kinds.is_empty() && !kinds.contains(&edge.kind()) {
                continue;
            }
            let next = if edge.source() == &current {
                edge.target()
            } else {
                edge.source()
            };
            if visited.insert(next.clone()) {
                queue.push_back((next.clone(), current_depth + 1, Some(graph_edge_id(edge))));
            }
        }
    }
    let total = nodes.len();
    nodes.truncate(limit);
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "startNodeId": start.as_str(),
        "direction": direction,
        "maxDepth": depth,
        "nodes": nodes,
        "total": total,
        "truncated": total > limit
    }))
}

fn edges<'a>(
    configuration: &'a WorkspaceConfigurationSnapshot,
    node_id: &EntityId,
    direction: &str,
) -> Vec<&'a GraphEdge> {
    let mut result = Vec::new();
    if matches!(direction, "outgoing" | "both") {
        result.extend(configuration.graph().outgoing(node_id));
    }
    if matches!(direction, "incoming" | "both") {
        result.extend(configuration.graph().incoming(node_id));
    }
    result.sort();
    result.dedup();
    result
}

fn node_value(node: &GraphNode) -> Value {
    let kind = GraphQueryNodeKind::from(node.kind());
    json!({
        "id": node.id().as_str(),
        "name": node.name().as_str(),
        "kind": kind.as_str(),
        "metadataKind": kind.metadata_kind().map(crate::GraphQueryMetadataKind::as_str)
    })
}

fn relation_value(
    configuration: &WorkspaceConfigurationSnapshot,
    node_id: &EntityId,
    edge: &GraphEdge,
) -> Result<Value, SemanticError> {
    let related_id = if edge.source() == node_id {
        edge.target()
    } else {
        edge.source()
    };
    let related = configuration
        .graph()
        .node(related_id)
        .ok_or(EXECUTION_FAILED)?;
    Ok(json!({
        "edgeId": graph_edge_id(edge),
        "edgeKind": GraphQueryEdgeKind::from(edge.kind()).as_str(),
        "sourceNodeId": edge.source().as_str(),
        "targetNodeId": edge.target().as_str(),
        "relatedNode": node_value(related)
    }))
}

fn graph_edge_id(edge: &GraphEdge) -> String {
    SemanticGraphQuery::edge_id(
        &NodeId::new(edge.source().as_str()),
        &NodeId::new(edge.target().as_str()),
        edge.kind(),
    )
    .into_inner()
}

fn validation(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(arguments, &["configurationId", "limit"])?;
    let configuration = configuration(snapshot, string(arguments, "configurationId")?)?;
    let limit = limit(arguments)?;
    let result = configuration.graph().validate();
    let total = result.issues().len();
    let issues = result
        .issues()
        .iter()
        .take(limit)
        .map(|issue| {
            json!({
                "code": issue.code().as_str(),
                "severity": validation_severity(issue.severity()),
                "kind": validation_kind(issue.kind()),
                "message": issue.message(),
                "nodeIds": issue.nodes().iter().map(EntityId::as_str).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "valid": result.is_valid(),
        "errorCount": result.error_count(),
        "warningCount": result.warning_count(),
        "issues": issues,
        "total": total,
        "truncated": total > limit
    }))
}

fn diagnostics(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(arguments, &["configurationId", "limit"])?;
    let configuration = configuration(snapshot, string(arguments, "configurationId")?)?;
    let limit = limit(arguments)?;
    let total = configuration.diagnostics().len();
    let values = configuration
        .diagnostics()
        .iter()
        .take(limit)
        .map(|item| {
            json!({
                "code": item.code().as_str(),
                "severity": diagnostic_severity(item.severity()),
                "kind": diagnostic_kind(item.kind()),
                "message": item.message(),
                "sourceNodeId": item.source_node().map(EntityId::as_str),
                "candidateNodeIds": item.candidates().iter().map(EntityId::as_str).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "diagnostics": values,
        "total": total,
        "truncated": total > limit
    }))
}

fn impact(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(
        arguments,
        &[
            "previousConfigurationId",
            "currentConfigurationId",
            "maxDepth",
            "limit",
        ],
    )?;
    let previous_id = string(arguments, "previousConfigurationId")?;
    let current_id = string(arguments, "currentConfigurationId")?;
    if previous_id == current_id {
        return Err(INVALID);
    }
    let previous = configuration(snapshot, previous_id)?;
    let current = configuration(snapshot, current_id)?;
    let depth = number(arguments, "maxDepth")?.unwrap_or(1);
    if depth > MAX_DEPTH {
        return Err(INVALID);
    }
    let limit = limit(arguments)?;
    let diff = previous.graph().diff(current.graph());
    let result = SemanticImpactAnalyzer::analyze(
        previous.graph(),
        current.graph(),
        &diff,
        &SemanticImpactOptions::new(depth),
    )
    .map_err(|_| EXECUTION_FAILED)?;
    let summary = result.summary();
    let total = result.affected_nodes().len();
    let affected = result
        .affected_nodes()
        .iter()
        .take(limit)
        .map(|node| {
            json!({
                "nodeId": node.node_id().as_str(),
                "kind": node.node_kind().map(GraphQueryNodeKind::from).map(GraphQueryNodeKind::as_str),
                "status": impact_status(node.status()),
                "depth": node.depth(),
                "reasons": node.reasons().iter().map(|reason| json!({
                    "kind": impact_reason_kind(reason.kind()),
                    "seed": {
                        "kind": impact_seed_kind(reason.seed().kind()),
                        "nodeId": reason.seed().node_id().map(NodeId::as_str),
                        "edgeId": reason.seed().edge_id().map(oneagent_graph::EdgeId::as_str)
                    },
                    "sourceNodeId": reason.source_node().map(NodeId::as_str),
                    "edgeId": reason.edge_id().map(oneagent_graph::EdgeId::as_str),
                    "edgeKind": reason.edge_kind().map(GraphQueryEdgeKind::from).map(GraphQueryEdgeKind::as_str),
                    "depth": reason.depth(),
                    "snapshot": impact_snapshot(reason.snapshot()),
                    "propagation": reason.propagation().map(impact_propagation)
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "previousConfigurationId": previous.configuration_id().as_str(),
        "currentConfigurationId": current.configuration_id().as_str(),
        "summary": {
            "seedNodeChanges": summary.seed_node_changes(),
            "seedEdgeChanges": summary.seed_edge_changes(),
            "directlyChangedNodes": summary.directly_changed_nodes(),
            "transitivelyAffectedNodes": summary.transitively_affected_nodes(),
            "removedNodes": summary.removed_nodes(),
            "totalAffectedNodes": summary.total_affected_nodes(),
            "maxReachedDepth": summary.max_reached_depth(),
            "requestedMaxDepth": summary.requested_max_depth()
        },
        "affectedNodes": affected,
        "total": total,
        "truncated": total > limit
    }))
}

fn context(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    fields(
        arguments,
        &[
            "configurationId",
            "nodeId",
            "direction",
            "maxDepth",
            "maxCandidates",
            "budgetBytes",
        ],
    )?;
    let configuration = configuration(snapshot, string(arguments, "configurationId")?)?;
    let node_id = EntityId::new(string(arguments, "nodeId")?.to_owned()).map_err(|_| INVALID)?;
    let depth = number(arguments, "maxDepth")?.unwrap_or(2);
    let candidates = number(arguments, "maxCandidates")?.unwrap_or(50);
    let budget = number(arguments, "budgetBytes")?.unwrap_or(16_384);
    if depth > MAX_DEPTH || !(1..=128).contains(&candidates) || !(1..=32_768).contains(&budget) {
        return Err(INVALID);
    }
    let traversal = match direction(arguments)? {
        "incoming" => ContextTraversalDirection::Incoming,
        "outgoing" => ContextTraversalDirection::Outgoing,
        "both" => ContextTraversalDirection::Both,
        _ => return Err(INVALID),
    };
    let policy = ContextPolicy::new(
        traversal,
        BTreeSet::from([
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
        ]),
        None,
        depth,
        candidates,
    );
    let request = ContextRequest::new(
        ContextIntent::Explain,
        vec![ContextSeed::node(node_id.as_str())],
        budget,
        policy,
    )
    .map_err(|_| INVALID)?;
    let bundle = ContextEngine
        .build(configuration.graph(), &request)
        .map_err(|error| context_error(&error))?;
    let items = bundle
        .items()
        .iter()
        .map(|item| {
            json!({
                "nodeId": item.node_id().as_str(),
                "name": item.name().as_str(),
                "kind": GraphQueryNodeKind::from(item.kind()).as_str(),
                "depth": item.depth(),
                "seedId": item.seed_id().as_str(),
                "reason": context_reason(item.reason()),
                "relations": item.path().iter().map(|step| json!({
                    "direction": context_relation_direction(step.direction()),
                    "edgeKind": GraphQueryEdgeKind::from(step.edge_kind()).as_str(),
                    "edgeId": step.edge_id().as_str()
                })).collect::<Vec<_>>(),
                "costBytes": item.cost_bytes()
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "rendered": bundle.rendered(),
        "items": items,
        "budgetBytes": bundle.budget().bytes(),
        "usedBytes": bundle.used_bytes(),
        "remainingBytes": bundle.remaining_bytes(),
        "candidateTruncated": bundle.candidate_truncated(),
        "candidateOmitted": bundle.candidate_omitted(),
        "budgetTruncated": bundle.budget_truncated(),
        "budgetOmitted": bundle.budget_omitted()
    }))
}

fn context_error(error: &ContextError) -> SemanticError {
    match error {
        ContextError::MissingSeed { .. } => NOT_FOUND,
        ContextError::CostOverflow | ContextError::UnsupportedKind => EXECUTION_FAILED,
        ContextError::InvalidBudget { .. }
        | ContextError::InvalidPolicy { .. }
        | ContextError::EmptyEdgeKinds
        | ContextError::EmptyNodeKinds
        | ContextError::InvalidSeedCount { .. }
        | ContextError::InvalidSeedIdentifier { .. }
        | ContextError::AmbiguousSeed { .. }
        | ContextError::IncompatibleSeed { .. }
        | ContextError::TooManyUniqueSeeds { .. }
        | ContextError::InsufficientBudget { .. } => INVALID,
    }
}

const fn context_reason(value: ContextInclusionReason) -> &'static str {
    match value {
        ContextInclusionReason::Seed => "seed",
        ContextInclusionReason::Related => "related",
    }
}

const fn context_relation_direction(value: ContextRelationDirection) -> &'static str {
    match value {
        ContextRelationDirection::Outgoing => "outgoing",
        ContextRelationDirection::Incoming => "incoming",
    }
}

const fn impact_status(value: ImpactNodeStatus) -> &'static str {
    match value {
        ImpactNodeStatus::DirectlyChanged => "directly_changed",
        ImpactNodeStatus::TransitivelyAffected => "transitively_affected",
        ImpactNodeStatus::Removed => "removed",
    }
}

const fn impact_reason_kind(value: ImpactReasonKind) -> &'static str {
    match value {
        ImpactReasonKind::NodeAdded => "node_added",
        ImpactReasonKind::NodeRemoved => "node_removed",
        ImpactReasonKind::NodeModified => "node_modified",
        ImpactReasonKind::EdgeAdded => "edge_added",
        ImpactReasonKind::EdgeRemoved => "edge_removed",
        ImpactReasonKind::EdgeModified => "edge_modified",
        ImpactReasonKind::DependencyPropagation => "dependency_propagation",
        ImpactReasonKind::OwnershipPropagation => "ownership_propagation",
    }
}

const fn impact_seed_kind(value: ImpactSeedKind) -> &'static str {
    match value {
        ImpactSeedKind::NodeAdded => "node_added",
        ImpactSeedKind::NodeRemoved => "node_removed",
        ImpactSeedKind::NodeModified => "node_modified",
        ImpactSeedKind::EdgeAdded => "edge_added",
        ImpactSeedKind::EdgeRemoved => "edge_removed",
        ImpactSeedKind::EdgeModified => "edge_modified",
    }
}

const fn impact_snapshot(value: ImpactSnapshot) -> &'static str {
    match value {
        ImpactSnapshot::Previous => "previous",
        ImpactSnapshot::Current => "current",
    }
}

const fn impact_propagation(value: ImpactPropagationDirection) -> &'static str {
    match value {
        ImpactPropagationDirection::DependencyToUsage => "dependency_to_usage",
        ImpactPropagationDirection::ChildToOwner => "child_to_owner",
        ImpactPropagationDirection::OwnerToChild => "owner_to_child",
    }
}

fn configuration<'a>(
    snapshot: &'a WorkspaceSnapshot,
    id: &str,
) -> Result<&'a WorkspaceConfigurationSnapshot, SemanticError> {
    let id = EntityId::new(id.to_owned()).map_err(|_| INVALID)?;
    snapshot.configuration(&id).ok_or(NOT_FOUND)
}

fn fields(arguments: &Map<String, Value>, allowed: &[&str]) -> Result<(), SemanticError> {
    arguments
        .keys()
        .all(|key| allowed.contains(&key.as_str()))
        .then_some(())
        .ok_or(INVALID)
}

fn absent(arguments: &Map<String, Value>, rejected: &[&str]) -> Result<(), SemanticError> {
    rejected
        .iter()
        .all(|field| !arguments.contains_key(*field))
        .then_some(())
        .ok_or(INVALID)
}

fn string<'a>(arguments: &'a Map<String, Value>, field: &str) -> Result<&'a str, SemanticError> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(INVALID)
}

fn optional_string<'a>(
    arguments: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, SemanticError> {
    arguments
        .get(field)
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or(INVALID)
        })
        .transpose()
}

fn number(arguments: &Map<String, Value>, field: &str) -> Result<Option<usize>, SemanticError> {
    arguments
        .get(field)
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(INVALID)
        })
        .transpose()
}

fn limit(arguments: &Map<String, Value>) -> Result<usize, SemanticError> {
    let value = number(arguments, "limit")?.unwrap_or(DEFAULT_LIMIT);
    (1..=MAX_LIMIT)
        .contains(&value)
        .then_some(value)
        .ok_or(INVALID)
}

fn direction(arguments: &Map<String, Value>) -> Result<&str, SemanticError> {
    let value = arguments
        .get("direction")
        .map_or(Some("both"), Value::as_str)
        .ok_or(INVALID)?;
    matches!(value, "incoming" | "outgoing" | "both")
        .then_some(value)
        .ok_or(INVALID)
}

fn kinds(arguments: &Map<String, Value>) -> Result<BTreeSet<EdgeKind>, SemanticError> {
    let Some(value) = arguments.get("edgeKinds") else {
        return Ok(BTreeSet::new());
    };
    let mut kinds = BTreeSet::new();
    for value in value.as_array().ok_or(INVALID)? {
        let kind = value.as_str().and_then(edge_kind).ok_or(INVALID)?;
        if !kinds.insert(kind) {
            return Err(INVALID);
        }
    }
    Ok(kinds)
}

fn edge_kind(value: &str) -> Option<EdgeKind> {
    match value {
        "contains" => Some(EdgeKind::Contains),
        "calls" => Some(EdgeKind::Calls),
        "references" => Some(EdgeKind::References),
        "reads" => Some(EdgeKind::Reads),
        "writes" => Some(EdgeKind::Writes),
        "grants" => Some(EdgeKind::Grants),
        "includes" => Some(EdgeKind::Includes),
        "extends" => Some(EdgeKind::Extends),
        "depends_on" => Some(EdgeKind::DependsOn),
        "opens" => Some(EdgeKind::Opens),
        "triggers" => Some(EdgeKind::Triggers),
        _ => None,
    }
}

const fn validation_severity(value: SemanticGraphValidationSeverity) -> &'static str {
    match value {
        SemanticGraphValidationSeverity::Error => "error",
        SemanticGraphValidationSeverity::Warning => "warning",
    }
}

const fn validation_kind(value: SemanticGraphValidationIssueKind) -> &'static str {
    match value {
        SemanticGraphValidationIssueKind::Structural => "structural",
        SemanticGraphValidationIssueKind::Semantic => "semantic",
        SemanticGraphValidationIssueKind::Provenance => "provenance",
        SemanticGraphValidationIssueKind::BuildConsistency => "build_consistency",
    }
}

const fn diagnostic_severity(value: SemanticDiagnosticSeverity) -> &'static str {
    match value {
        SemanticDiagnosticSeverity::Warning => "warning",
        SemanticDiagnosticSeverity::Error => "error",
    }
}

const fn diagnostic_kind(value: SemanticDiagnosticKind) -> &'static str {
    match value {
        SemanticDiagnosticKind::QueryLanguageMalformedSyntax => "query_language_malformed_syntax",
        SemanticDiagnosticKind::QueryLanguageUnsupportedStructure => {
            "query_language_unsupported_structure"
        }
        SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace => {
            "query_language_unsupported_persistent_namespace"
        }
        SemanticDiagnosticKind::QueryLanguageVirtualTableSource => {
            "query_language_virtual_table_source"
        }
        SemanticDiagnosticKind::QueryLanguageTemporaryTableSource => {
            "query_language_temporary_table_source"
        }
        SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource => {
            "query_language_external_or_parameter_data_source"
        }
        SemanticDiagnosticKind::DataCompositionNestedDataSetDeferred => {
            "data_composition_nested_data_set_deferred"
        }
        SemanticDiagnosticKind::DataCompositionFieldFolderDeferred => {
            "data_composition_field_folder_deferred"
        }
        SemanticDiagnosticKind::DataCompositionUnsupportedDataSetType => {
            "data_composition_unsupported_data_set_type"
        }
        SemanticDiagnosticKind::DataCompositionUnsupportedFieldType => {
            "data_composition_unsupported_field_type"
        }
        SemanticDiagnosticKind::MalformedReferenceFormat => "malformed_reference_format",
        SemanticDiagnosticKind::UnsupportedReferencePrefix => "unsupported_reference_prefix",
        SemanticDiagnosticKind::UnresolvedTarget => "unresolved_target",
        SemanticDiagnosticKind::AmbiguousTarget => "ambiguous_target",
        SemanticDiagnosticKind::IncompatibleTargetKind => "incompatible_target_kind",
        SemanticDiagnosticKind::InvalidOwnerReference => "invalid_owner_reference",
        SemanticDiagnosticKind::DuplicateSemanticEdgeRequest => "duplicate_semantic_edge_request",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oneagent_protocol::{McpToolCallHandler, McpToolCallOutcome};
    use oneagent_tool_policy::{PolicyRevision, ToolPolicy};
    use serde_json::Map;
    use tempfile::tempdir;

    use super::{GRAPH, Handler};
    use crate::WorkspaceSnapshotBuilder;

    #[tokio::test]
    async fn denied_policy_cannot_bypass_the_semantic_executor_gate() {
        let root = tempdir().expect("empty workspace root");
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect("empty workspace must build");
        let handler = Handler {
            snapshot: Arc::new(snapshot),
            policy: ToolPolicy::new(
                PolicyRevision::new("oneagent.mcp.denied-test").expect("test revision"),
                Vec::new(),
            )
            .expect("empty policy must deny by default"),
        };

        let outcome = handler.call(GRAPH, &Map::new()).await;
        assert!(matches!(
            outcome,
            McpToolCallOutcome::Error { ref code, ref message }
                if code == "policy_denied" && message == "The semantic tool request was denied."
        ));
    }
}
