//! Read-only semantic MCP tools over immutable workspace snapshots.

use std::collections::{BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use oneagent_analysis::context::{
    ContextEngine, ContextError, ContextInclusionReason, ContextIntent, ContextPolicy,
    ContextRelationDirection, ContextRequest, ContextSeed, ContextTraversalDirection,
};
use oneagent_analysis::diagnostics::{
    DiagnosticCategory, DiagnosticDisposition, DiagnosticFamily, DiagnosticFilter,
    DiagnosticFinding, DiagnosticReport, DiagnosticSeverity, DiagnosticSummary,
};
use oneagent_common::{EntityId, SourceLocation};
use oneagent_graph::{
    EdgeKind, GraphEdge, GraphNode, ImpactNodeStatus, ImpactPropagationDirection, ImpactReasonKind,
    ImpactSeedKind, ImpactSnapshot, NodeId, NodeKind, SemanticGraphQuery,
    SemanticGraphValidationIssueKind, SemanticGraphValidationSeverity, SemanticImpactAnalyzer,
    SemanticImpactOptions,
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
const SYMBOLS: &str = "oneagent.symbols";
const ACTOR: &str = "oneagent.mcp";
const REQUEST: &str = "oneagent.mcp.request";
const REVISION: &str = "oneagent.mcp.read-only.v1";
const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;
const MAX_DEPTH: usize = 4;
const MAX_SYMBOL_QUERY_BYTES: usize = 256;
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

/// Builds the complete immutable Sprint 31 semantic MCP server.
///
/// # Errors
///
/// Returns a closed error when a fixed catalog or policy invariant fails.
pub fn semantic_server(snapshot: WorkspaceSnapshot) -> Result<McpServer, McpSemanticServerError> {
    let names = [
        CONTEXT,
        DIAGNOSTICS,
        GRAPH,
        IMPACT,
        QUERY,
        SYMBOLS,
        VALIDATION,
    ];
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

fn diagnostic_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "configurationId": {"type": "string"},
            "families": {
                "type": "array",
                "items": {"enum": ["semantic", "validation"]},
                "minItems": 1,
                "uniqueItems": true
            },
            "severities": {
                "type": "array",
                "items": {"enum": ["error", "warning"]},
                "minItems": 1,
                "uniqueItems": true
            },
            "categories": {
                "type": "array",
                "items": {"enum": [
                    "source", "semantic", "structural", "provenance", "build_consistency"
                ]},
                "minItems": 1,
                "uniqueItems": true
            },
            "includeSuppressed": {"type": "boolean"},
            "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
        },
        "required": ["configurationId"],
        "additionalProperties": false
    })
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
        (
            SYMBOLS,
            "Search bounded navigable symbols in immutable OneAgent semantic graphs.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "minLength": 1},
                    "configurationId": {"type": "string"},
                    "kinds": {
                        "type": "array",
                        "items": {"enum": ["module", "procedure", "function", "query"]},
                        "minItems": 1,
                        "uniqueItems": true
                    },
                    "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LIMIT}
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        ),
        (VALIDATION, "Validate one immutable OneAgent semantic graph.", limited()),
        (
            DIAGNOSTICS,
            "List bounded source-independent OneAgent diagnostic findings.",
            diagnostic_schema(),
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
            if name == SYMBOLS && symbol_arguments(arguments).is_err() {
                return tool_error("invalid_arguments");
            }
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
        SYMBOLS => symbols(snapshot, arguments),
        _ => Err(INVALID),
    }
}

struct SymbolResult<'a> {
    configuration: &'a WorkspaceConfigurationSnapshot,
    node: &'a GraphNode,
    folded_name: String,
    kind: &'static str,
    location: Value,
}

struct SymbolArguments<'a> {
    query: &'a str,
    selected: Option<&'a str>,
    accepted_kinds: BTreeSet<&'static str>,
    limit: usize,
}

fn symbol_arguments(arguments: &Map<String, Value>) -> Result<SymbolArguments<'_>, SemanticError> {
    fields(arguments, &["query", "configurationId", "kinds", "limit"])?;
    let query = string(arguments, "query")?;
    if query.len() > MAX_SYMBOL_QUERY_BYTES {
        return Err(INVALID);
    }
    Ok(SymbolArguments {
        query,
        selected: optional_string(arguments, "configurationId")?,
        accepted_kinds: symbol_kinds(arguments)?,
        limit: limit(arguments)?,
    })
}

fn symbols(
    snapshot: &WorkspaceSnapshot,
    arguments: &Map<String, Value>,
) -> Result<Value, SemanticError> {
    let arguments = symbol_arguments(arguments)?;
    let folded_query = arguments.query.to_lowercase();
    let selected = arguments.selected;
    if let Some(id) = selected {
        configuration(snapshot, id)?;
    }
    let mut results = Vec::new();

    for configuration in snapshot.configurations().iter().filter(|configuration| {
        selected.is_none_or(|id| id == configuration.configuration_id().as_str())
    }) {
        for node in configuration.graph().nodes() {
            let Some(kind) = symbol_kind(node.kind()) else {
                continue;
            };
            if !arguments.accepted_kinds.contains(kind) {
                continue;
            }
            let folded_name = node.name().as_str().to_lowercase();
            if !folded_name.contains(&folded_query) {
                continue;
            }
            let Some(location) = unique_symbol_location(snapshot, configuration, node) else {
                continue;
            };
            results.push(SymbolResult {
                configuration,
                node,
                folded_name,
                kind,
                location,
            });
        }
    }

    results.sort_by(|left, right| {
        (
            left.folded_name.as_str(),
            left.node.name().as_str(),
            left.kind,
            left.node.id().as_str(),
            left.configuration.configuration_id().as_str(),
        )
            .cmp(&(
                right.folded_name.as_str(),
                right.node.name().as_str(),
                right.kind,
                right.node.id().as_str(),
                right.configuration.configuration_id().as_str(),
            ))
    });
    let total = u64::try_from(results.len()).map_err(|_| EXECUTION_FAILED)?;
    results.truncate(arguments.limit);
    let values = results
        .into_iter()
        .map(|result| {
            json!({
                "configurationId": result.configuration.configuration_id().as_str(),
                "configurationName": result.configuration.configuration_name().as_str(),
                "nodeId": result.node.id().as_str(),
                "name": result.node.name().as_str(),
                "kind": result.kind,
                "location": result.location
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "results": values,
        "total": total,
        "truncated": total > u64::try_from(arguments.limit).map_err(|_| EXECUTION_FAILED)?
    }))
}

fn symbol_kinds(arguments: &Map<String, Value>) -> Result<BTreeSet<&'static str>, SemanticError> {
    let Some(value) = arguments.get("kinds") else {
        return Ok(BTreeSet::from(["module", "procedure", "function", "query"]));
    };
    let values = value
        .as_array()
        .filter(|values| !values.is_empty())
        .ok_or(INVALID)?;
    let mut kinds = BTreeSet::new();
    for value in values {
        let kind = value.as_str().and_then(symbol_kind_name).ok_or(INVALID)?;
        if !kinds.insert(kind) {
            return Err(INVALID);
        }
    }
    Ok(kinds)
}

const fn symbol_kind(kind: NodeKind) -> Option<&'static str> {
    match kind {
        NodeKind::Module => Some("module"),
        NodeKind::Procedure => Some("procedure"),
        NodeKind::Function => Some("function"),
        NodeKind::Query => Some("query"),
        _ => None,
    }
}

const fn symbol_kind_name(value: &str) -> Option<&'static str> {
    match value.as_bytes() {
        b"module" => Some("module"),
        b"procedure" => Some("procedure"),
        b"function" => Some("function"),
        b"query" => Some("query"),
        _ => None,
    }
}

pub(crate) fn unique_symbol_location(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    node: &GraphNode,
) -> Option<Value> {
    let locations = node
        .provenance()
        .iter()
        .filter_map(oneagent_graph::Provenance::location)
        .collect::<BTreeSet<_>>();
    if locations.len() != 1 {
        return None;
    }
    project_symbol_location(snapshot, configuration, locations.into_iter().next()?)
}

fn project_symbol_location(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    location: &SourceLocation,
) -> Option<Value> {
    let source = Path::new(location.path().as_str());
    let candidate = if source.is_absolute() {
        source.to_path_buf()
    } else {
        configuration.root_path().join(source)
    };
    let candidate = lexical_normalize(&candidate)?;
    let configuration_root = lexical_normalize(configuration.root_path())?;
    let workspace_root = lexical_normalize(snapshot.root_path())?;
    candidate.strip_prefix(&configuration_root).ok()?;
    let relative = candidate.strip_prefix(&workspace_root).ok()?;
    let path = relative_path(relative)?;
    let span = location.span().map(|span| {
        json!({
            "start": {"line": span.start().line(), "column": span.start().column()},
            "end": {"line": span.end().line(), "column": span.end().column()}
        })
    });
    let mut value = Map::new();
    value.insert("path".to_owned(), Value::String(path));
    if let Some(span) = span {
        value.insert("span".to_owned(), span);
    }
    Some(Value::Object(value))
}

fn lexical_normalize(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(value) => normalized.push(value),
        }
    }
    Some(normalized)
}

fn relative_path(path: &Path) -> Option<String> {
    let components = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    if components.is_empty() {
        return None;
    }
    let path = components.join("/");
    (path.len() <= oneagent_common::MAX_SOURCE_PATH_BYTES).then_some(path)
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
    let validation = configuration.validation();
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
    let result = configuration.validation();
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
    let arguments = diagnostic_arguments(arguments)?;
    let configuration = configuration(snapshot, arguments.configuration_id)?;
    Ok(project_diagnostic_report(
        configuration.configuration_id().as_str(),
        configuration.diagnostic_report(),
        &arguments.filter,
        arguments.limit,
    ))
}

struct DiagnosticArguments<'a> {
    configuration_id: &'a str,
    filter: DiagnosticFilter,
    limit: usize,
}

fn diagnostic_arguments(
    arguments: &Map<String, Value>,
) -> Result<DiagnosticArguments<'_>, SemanticError> {
    fields(
        arguments,
        &[
            "configurationId",
            "families",
            "severities",
            "categories",
            "includeSuppressed",
            "limit",
        ],
    )?;
    let families = diagnostic_values(arguments, "families", diagnostic_family)?;
    let severities = diagnostic_values(arguments, "severities", diagnostic_severity)?;
    let categories = diagnostic_values(arguments, "categories", diagnostic_category)?;
    let include_suppressed = match arguments.get("includeSuppressed") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(INVALID),
    };
    let dispositions = if include_suppressed {
        BTreeSet::new()
    } else {
        BTreeSet::from([DiagnosticDisposition::Active])
    };
    Ok(DiagnosticArguments {
        configuration_id: string(arguments, "configurationId")?,
        filter: DiagnosticFilter::new(families, severities, categories, dispositions),
        limit: limit(arguments)?,
    })
}

fn diagnostic_values<T: Ord>(
    arguments: &Map<String, Value>,
    field: &str,
    parse: fn(&str) -> Option<T>,
) -> Result<BTreeSet<T>, SemanticError> {
    let Some(value) = arguments.get(field) else {
        return Ok(BTreeSet::new());
    };
    let values = value
        .as_array()
        .filter(|values| !values.is_empty())
        .ok_or(INVALID)?;
    let mut accepted = BTreeSet::new();
    for value in values {
        let value = value.as_str().and_then(parse).ok_or(INVALID)?;
        if !accepted.insert(value) {
            return Err(INVALID);
        }
    }
    Ok(accepted)
}

fn project_diagnostic_report(
    configuration_id: &str,
    report: &DiagnosticReport,
    filter: &DiagnosticFilter,
    limit: usize,
) -> Value {
    let matching = report.filtered(filter).collect::<Vec<_>>();
    let total = matching.len();
    let values = matching
        .into_iter()
        .take(limit)
        .map(project_diagnostic_finding)
        .collect::<Vec<_>>();
    let returned = values.len();
    json!({
        "configurationId": configuration_id,
        "diagnostics": values,
        "total": total,
        "truncated": total > returned,
        "summary": project_diagnostic_summary(report.summary())
    })
}

fn project_diagnostic_finding(finding: &DiagnosticFinding) -> Value {
    let semantic = matches!(finding.family(), DiagnosticFamily::Semantic);
    let source_node = semantic
        .then(|| finding.node_anchors().first())
        .flatten()
        .map(EntityId::as_str);
    let candidate_nodes = if semantic {
        finding
            .related_nodes()
            .iter()
            .map(EntityId::as_str)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    json!({
        "family": finding.family().as_str(),
        "code": finding.code().as_str(),
        "severity": finding.severity().as_str(),
        "category": finding.category().as_str(),
        "kind": finding.kind().as_str(),
        "message": finding.message(),
        "disposition": finding.disposition().as_str(),
        "sourceNodeId": source_node,
        "candidateNodeIds": candidate_nodes,
        "nodeIds": finding.node_anchors().iter().map(EntityId::as_str).collect::<Vec<_>>(),
        "edgeId": finding.edge_id().map(oneagent_graph::EdgeId::as_str),
        "referenceRequestId": finding
            .reference_request_id()
            .map(oneagent_graph::SemanticReferenceRequestId::as_str)
    })
}

fn project_diagnostic_summary(summary: &DiagnosticSummary) -> Value {
    let family_count = |value| summary.by_family().get(&value).copied().unwrap_or(0);
    let severity_count = |value| summary.by_severity().get(&value).copied().unwrap_or(0);
    let category_count = |value| summary.by_category().get(&value).copied().unwrap_or(0);
    json!({
        "total": summary.total(),
        "active": summary.active(),
        "suppressed": summary.suppressed(),
        "byFamily": {
            "semantic": family_count(DiagnosticFamily::Semantic),
            "validation": family_count(DiagnosticFamily::Validation)
        },
        "bySeverity": {
            "error": severity_count(DiagnosticSeverity::Error),
            "warning": severity_count(DiagnosticSeverity::Warning)
        },
        "byCategory": {
            "source": category_count(DiagnosticCategory::Source),
            "semantic": category_count(DiagnosticCategory::Semantic),
            "structural": category_count(DiagnosticCategory::Structural),
            "provenance": category_count(DiagnosticCategory::Provenance),
            "build_consistency": category_count(DiagnosticCategory::BuildConsistency)
        },
        "activeByCode": summary.active_by_code(),
        "suppressedByCode": summary.suppressed_by_code()
    })
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

fn diagnostic_family(value: &str) -> Option<DiagnosticFamily> {
    match value {
        "semantic" => Some(DiagnosticFamily::Semantic),
        "validation" => Some(DiagnosticFamily::Validation),
        _ => None,
    }
}

fn diagnostic_severity(value: &str) -> Option<DiagnosticSeverity> {
    match value {
        "error" => Some(DiagnosticSeverity::Error),
        "warning" => Some(DiagnosticSeverity::Warning),
        _ => None,
    }
}

fn diagnostic_category(value: &str) -> Option<DiagnosticCategory> {
    match value {
        "source" => Some(DiagnosticCategory::Source),
        "semantic" => Some(DiagnosticCategory::Semantic),
        "structural" => Some(DiagnosticCategory::Structural),
        "provenance" => Some(DiagnosticCategory::Provenance),
        "build_consistency" => Some(DiagnosticCategory::BuildConsistency),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;
    use std::sync::Arc;

    use oneagent_analysis::diagnostics::{DiagnosticEngine, DiagnosticIdentity, DiagnosticPolicy};
    use oneagent_common::{EntityId, EntityName, SourceLocation, SourcePath};
    use oneagent_graph::{
        Confidence, FactOrigin, GraphNode, NodeKind, ProducerId, Provenance, ResolutionState,
        SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    };
    use oneagent_protocol::{McpToolCallHandler, McpToolCallOutcome};
    use oneagent_tool_policy::{PolicyRevision, ToolPolicy};
    use serde_json::{Map, json};
    use tempfile::tempdir;

    use super::{
        DIAGNOSTICS, GRAPH, Handler, SYMBOLS, diagnostic_arguments, project_diagnostic_report,
        project_symbol_location, unique_symbol_location,
    };
    use crate::WorkspaceSnapshotBuilder;

    fn fixture_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/workspace_service")
            .leak()
    }

    fn provenance(location: SourceLocation, producer: &str) -> Provenance {
        Provenance::new_with_location(
            None,
            Some(location),
            ProducerId::new(producer),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn diagnostic(source: &str, severity: SemanticDiagnosticSeverity) -> SemanticDiagnostic {
        SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            severity,
            SemanticDiagnosticKind::UnresolvedTarget,
            "semantic reference target could not be resolved",
            SemanticReference::NodeId("metadata.target".to_owned()),
        )
        .with_source_node(EntityId::new(source).expect("source node ID"))
    }

    #[test]
    fn diagnostic_projection_filters_complete_report_and_retains_unfiltered_summary() {
        let active = diagnostic("metadata.active", SemanticDiagnosticSeverity::Error);
        let suppressed = diagnostic("metadata.suppressed", SemanticDiagnosticSeverity::Warning);
        let policy = DiagnosticPolicy::new(BTreeSet::from([DiagnosticIdentity::from_semantic(
            &suppressed,
        )]))
        .expect("one suppression");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            EntityId::new("metadata.validation").expect("validation node ID"),
            EntityName::new("Validation").expect("validation node name"),
            NodeKind::Unknown,
        ));
        let report = DiagnosticEngine
            .build(&[suppressed, active], &graph.validate(), &policy)
            .expect("mixed report");

        let arguments = json!({
            "configurationId": "configuration.test",
            "families": ["semantic", "validation"],
            "severities": ["warning"],
            "categories": ["semantic", "provenance"],
            "includeSuppressed": true,
            "limit": 1
        });
        let arguments = diagnostic_arguments(arguments.as_object().expect("arguments object"))
            .unwrap_or_else(|_| panic!("valid diagnostic filters"));
        let projected = project_diagnostic_report(
            arguments.configuration_id,
            &report,
            &arguments.filter,
            arguments.limit,
        );

        assert_eq!(projected["total"], 2);
        assert_eq!(projected["truncated"], true);
        assert_eq!(projected["diagnostics"][0]["family"], "validation");
        assert_eq!(projected["diagnostics"][0]["disposition"], "active");
        assert_eq!(
            projected["diagnostics"][0]["nodeIds"],
            json!(["metadata.validation"])
        );
        assert!(projected["diagnostics"][0]["sourceNodeId"].is_null());
        assert_eq!(projected["summary"]["total"], 3);
        assert_eq!(projected["summary"]["active"], 2);
        assert_eq!(projected["summary"]["suppressed"], 1);
        assert_eq!(projected["summary"]["byFamily"]["semantic"], 2);
        assert_eq!(projected["summary"]["byFamily"]["validation"], 1);
        assert_eq!(projected["summary"]["bySeverity"]["error"], 1);
        assert_eq!(projected["summary"]["bySeverity"]["warning"], 2);
        assert_eq!(projected["summary"]["byCategory"]["provenance"], 1);
        assert_eq!(
            projected["summary"]["suppressedByCode"]["semantic.reference.unresolved"],
            1
        );

        let default_arguments = json!({"configurationId": "configuration.test"});
        let default_arguments = diagnostic_arguments(
            default_arguments
                .as_object()
                .expect("default arguments object"),
        )
        .unwrap_or_else(|_| panic!("default diagnostic arguments"));
        let active_only = project_diagnostic_report(
            default_arguments.configuration_id,
            &report,
            &default_arguments.filter,
            default_arguments.limit,
        );
        assert_eq!(active_only["total"], 2);
        assert!(
            active_only["diagnostics"]
                .as_array()
                .expect("active diagnostics")
                .iter()
                .all(|finding| finding["disposition"] == "active")
        );
        assert_eq!(active_only["summary"], projected["summary"]);
    }

    #[test]
    fn diagnostic_arguments_reject_empty_duplicate_invalid_and_unknown_values() {
        for arguments in [
            json!({"configurationId": "configuration.test", "families": []}),
            json!({"configurationId": "configuration.test", "families": ["semantic", "semantic"]}),
            json!({"configurationId": "configuration.test", "families": ["rules"]}),
            json!({"configurationId": "configuration.test", "severities": []}),
            json!({"configurationId": "configuration.test", "severities": ["info"]}),
            json!({"configurationId": "configuration.test", "categories": []}),
            json!({"configurationId": "configuration.test", "categories": ["source", "source"]}),
            json!({"configurationId": "configuration.test", "categories": ["runtime"]}),
            json!({"configurationId": "configuration.test", "includeSuppressed": "true"}),
            json!({"configurationId": "configuration.test", "extra": true}),
        ] {
            assert!(
                diagnostic_arguments(arguments.as_object().expect("arguments object")).is_err(),
                "{arguments}"
            );
        }
    }

    #[tokio::test]
    async fn symbols_validate_before_policy_and_valid_calls_remain_policy_gated() {
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

        let graph = handler.call(GRAPH, &Map::new()).await;
        assert!(matches!(
            graph,
            McpToolCallOutcome::Error { ref code, ref message }
                if code == "policy_denied" && message == "The semantic tool request was denied."
        ));

        let valid_diagnostics = json!({"configurationId": "configuration.test"})
            .as_object()
            .expect("diagnostic arguments must be an object")
            .clone();
        let valid_diagnostics = handler.call(DIAGNOSTICS, &valid_diagnostics).await;
        assert!(matches!(
            valid_diagnostics,
            McpToolCallOutcome::Error { ref code, ref message }
                if code == "policy_denied" && message == "The semantic tool request was denied."
        ));

        let invalid_symbols = handler.call(SYMBOLS, &Map::new()).await;
        assert!(matches!(
            invalid_symbols,
            McpToolCallOutcome::Error { ref code, ref message }
                if code == "invalid_arguments"
                    && message == "The semantic tool arguments are invalid."
        ));

        let valid_arguments = json!({"query": "x"})
            .as_object()
            .expect("symbol arguments must be an object")
            .clone();
        let valid_symbols = handler.call(SYMBOLS, &valid_arguments).await;
        assert!(matches!(
            valid_symbols,
            McpToolCallOutcome::Error { ref code, ref message }
                if code == "policy_denied" && message == "The semantic tool request was denied."
        ));
    }

    #[test]
    fn symbol_locations_require_one_distinct_confined_location() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let configuration = &snapshot.configurations()[0];
        let path = SourcePath::new(
            configuration
                .root_path()
                .join("Module.bsl")
                .to_string_lossy(),
        )
        .expect("confined source path");
        let location = SourceLocation::new(path, None);

        let missing = GraphNode::new(
            EntityId::new("missing").expect("node id"),
            EntityName::new("Missing").expect("node name"),
            NodeKind::Module,
        );
        assert!(unique_symbol_location(&snapshot, configuration, &missing).is_none());

        let repeated = GraphNode::new_with_provenance(
            EntityId::new("repeated").expect("node id"),
            EntityName::new("Repeated").expect("node name"),
            NodeKind::Module,
            vec![
                provenance(location.clone(), "producer-a"),
                provenance(location.clone(), "producer-b"),
            ],
        );
        assert_eq!(
            unique_symbol_location(&snapshot, configuration, &repeated)
                .expect("identical locations must collapse")["path"],
            format!(
                "{}/Module.bsl",
                configuration
                    .root_path()
                    .file_name()
                    .expect("root name")
                    .to_string_lossy()
            )
        );

        let conflicting = GraphNode::new_with_provenance(
            EntityId::new("conflicting").expect("node id"),
            EntityName::new("Conflicting").expect("node name"),
            NodeKind::Module,
            vec![
                provenance(location, "producer-a"),
                provenance(
                    SourceLocation::new(
                        SourcePath::new(
                            configuration
                                .root_path()
                                .join("Other.bsl")
                                .to_string_lossy(),
                        )
                        .expect("second confined source path"),
                        None,
                    ),
                    "producer-b",
                ),
            ],
        );
        assert!(unique_symbol_location(&snapshot, configuration, &conflicting).is_none());
    }

    #[test]
    fn symbol_location_projection_rejects_workspace_and_configuration_escape() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let configuration = &snapshot.configurations()[0];
        let outside_workspace = snapshot
            .root_path()
            .parent()
            .expect("fixture root parent")
            .join("outside.bsl");
        let outside = SourceLocation::new(
            SourcePath::new(outside_workspace.to_string_lossy()).expect("absolute source path"),
            None,
        );
        assert!(project_symbol_location(&snapshot, configuration, &outside).is_none());

        let other_configuration = &snapshot.configurations()[1];
        let cross_configuration = SourceLocation::new(
            SourcePath::new(
                other_configuration
                    .root_path()
                    .join("Module.bsl")
                    .to_string_lossy(),
            )
            .expect("cross-configuration source path"),
            None,
        );
        assert!(project_symbol_location(&snapshot, configuration, &cross_configuration).is_none());
    }
}
