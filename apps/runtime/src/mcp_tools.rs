//! Read-only semantic MCP tools over immutable workspace snapshots.

use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;

use oneagent_common::EntityId;
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, SemanticDiagnosticKind, SemanticDiagnosticSeverity,
    SemanticGraphValidationIssueKind, SemanticGraphValidationSeverity,
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
const ACTOR: &str = "oneagent.mcp";
const REQUEST: &str = "oneagent.mcp.request";
const REVISION: &str = "oneagent.mcp.read-only.v1";
const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;
const MAX_DEPTH: usize = 4;

/// Closed construction failure for a semantic MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpSemanticServerError;

/// Builds an immutable MCP server for the graph tool subset.
///
/// # Errors
///
/// Returns a closed error when a fixed catalog or policy invariant fails.
pub fn graph_semantic_server(
    snapshot: WorkspaceSnapshot,
) -> Result<McpServer, McpSemanticServerError> {
    let names = [GRAPH, QUERY, VALIDATION, DIAGNOSTICS];
    McpServer::with_tools(
        definitions()?,
        Handler {
            snapshot: Arc::new(snapshot),
            policy: policy(&names)?,
        },
    )
    .map_err(|_| McpSemanticServerError)
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
                    "edgeKinds": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
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
            serde_json::to_string(&envelope)
                .ok()
                .and_then(|value| ToolOutput::new(value).ok())
                .map_or(
                    ToolExecutorOutcome::Failed(None),
                    ToolExecutorOutcome::Completed,
                )
        })
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
    Ok(json!({
        "configurationId": configuration.configuration_id().as_str(),
        "nodeId": node_id.as_str(),
        "direction": direction,
        "relations": edges.into_iter().map(edge_value).collect::<Vec<_>>(),
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
    let mut queue = VecDeque::from([(start.clone(), 0_usize)]);
    let mut visited = BTreeSet::from([start.clone()]);
    let mut nodes = Vec::new();
    while let Some((current, current_depth)) = queue.pop_front() {
        let node = configuration.graph().node(&current).ok_or(NOT_FOUND)?;
        nodes.push(json!({"node": node_value(node), "depth": current_depth}));
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
                queue.push_back((next.clone(), current_depth + 1));
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

fn edge_value(edge: &GraphEdge) -> Value {
    json!({
        "sourceNodeId": edge.source().as_str(),
        "targetNodeId": edge.target().as_str(),
        "kind": GraphQueryEdgeKind::from(edge.kind()).as_str()
    })
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
    value
        .as_array()
        .ok_or(INVALID)?
        .iter()
        .map(|value| value.as_str().and_then(edge_kind).ok_or(INVALID))
        .collect()
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
