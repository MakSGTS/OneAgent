# ADR-0051: MCP Semantic Tools

## Status

Accepted

## Context

ADR-0050 implements a bounded discovery-only MCP `2026-07-28` server. Sprint
29 must expose graph, query, validation, diagnostics, impact, and context while
preserving existing semantic owners and routing every executable request through
ADR-0049 Tool Policy. The supporting evidence is recorded in
`docs/architecture/mcp-semantic-tools-investigation.md`.

## Decision

### Authority and ownership

MCP revision `2026-07-28` remains the sole protocol authority. Its versioned
tools specification, schema reference, and TypeScript schema cited by the
investigation govern `tools`, `tools/list`, `tools/call`, definitions,
annotations, and results. ADR-0050 continues to govern JSON-RPC, metadata,
errors, bounds, discovery, framing, and process lifecycle.

`oneagent-protocol` owns tool wire values, catalog validation, list/call parsing,
unknown-tool handling, result envelopes, and transport-independent dispatch.
Runtime owns semantic composition, immutable workspace snapshot, Tool Policy
request/evaluation/execution, result projection, stdio adapter, and process.
Graph remains the only validation/diff/impact owner; Analysis remains the only
Context Engine owner. No reverse dependency is permitted.

Runtime adds local workspace dependencies on `oneagent-analysis` and
`oneagent-tool-policy`. Existing protocol and graph edges remain. No new
third-party package or version is accepted.

### Catalog

The complete immutable first-slice catalog, in this canonical order, is:

1. `oneagent.context`
2. `oneagent.diagnostics`
3. `oneagent.graph`
4. `oneagent.impact`
5. `oneagent.query`
6. `oneagent.validation`

Lexicographic order is deliberately both storage and wire order. Each tool has
an English description, a closed object-root input schema, and annotations
`readOnlyHint=true`, `destructiveHint=false`, `idempotentHint=true`, and
`openWorldHint=false`. Titles, icons, and output schemas are omitted. The
server advertises `capabilities.tools={}`; `listChanged` is omitted because the
catalog never changes during a process.

`tools/list` accepts no pagination cursor for this single-page catalog. An
absent cursor returns all six tools plus `ttlMs=0` and `cacheScope="public"`.
Any supplied cursor or unknown parameter is `InvalidParams`.

### Public protocol integration boundary

Protocol exports validated `McpToolDefinition`, `McpToolAnnotations`,
`McpToolCallHandler`, `McpToolCallOutcome`, and a fallible
`McpServer::with_tools` constructor. Runtime supplies definitions and one
handler; protocol never imports Runtime or semantic crates.

`McpToolCallHandler` returns a boxed borrowed `Send` future. Internal method
handlers use the same future contract. `McpServer::dispatch` becomes `async`
and remains sequential, transport-independent, task-free, and notification-
suppressing. Runtime transport awaits it before reading the next frame. All
current protocol tests and Runtime transport/process consumers migrate in Task
3; no synchronous compatibility shim or manual future polling is retained.

The discovery handler must return the server's actual capability object rather
than a separately constructed empty map. Method registration remains closed
except for the accepted tool constructor.

### Call and result behavior

`tools/call` requires exactly `name`, optional `arguments`, and inherited
request `_meta`. `name` must be a catalog member and `arguments` defaults to an
empty object. Non-object arguments, missing/extra fields, and an unknown name
are protocol `InvalidParams`; the semantic handler is not invoked.

A successful known call returns:

- `content`: one `text` block containing deterministic compact JSON;
- `structuredContent`: the same parsed JSON value;
- no `isError` member.

A known call with invalid semantic arguments, policy denial/failure, missing
configuration/node, domain failure, or oversized output returns the same two
representations of `{"code": <stable string>, "message": <stable string>}`
and `isError=true`. It is not a JSON-RPC error. Handler infrastructure failure,
invalid handler output, or impossible response encoding is `InternalError`.
No implicit diagnostic contains inputs, filesystem/provenance paths, raw source
chains, policy internals, or Rust type prose.

### Tool Policy gate

Every known call constructs a validated ADR-0049 `ToolRequest` with exact
catalog `ToolId`, actor `oneagent.mcp`, request id `oneagent.mcp.request`, policy
revision `oneagent.mcp.read-only.v1`, canonical compact JSON arguments, and
only `ToolEffect::ReadOnly`. Runtime constructs one immutable exact-allow policy
for the six tool ids and actor, then invokes only `execute_tool` with no
confirmation and `NeverCancelled`.

The semantic projector implements `ToolExecutor`. Its completed `ToolOutput` is
parsed back to JSON and converted to the MCP result. Denied, partial, failed,
timed-out, cancelled, malformed-output, or output-bound outcomes fail closed as
stable tool errors. The content-free audit result is retained only for the
duration of the call and used to verify one attempted completion; no persistent
audit sink is added. MCP annotations never authorize execution.

Sequential dispatch makes the fixed internal request id unambiguous because at
most one call is in flight. Mid-computation cancellation, per-client actors,
confirmation UX, policy administration, and concurrent audit correlation are
deferred.

### Immutable workspace lifecycle

At process startup Runtime builds exactly one `WorkspaceSnapshot` from the
current working directory before reading protocol frames. The snapshot and
semantic handler are immutable for the process lifetime. An empty supported
workspace is valid and still exposes the catalog. Build failure exits non-zero
before protocol output and writes one bounded stable category to stderr without
a path or source chain.

No Runtime `App`, watcher, cache, background task, port, reload, or filesystem
operation occurs after startup. Existing protocol-only process tests run with
an empty temporary working directory; semantic process evidence uses the
tracked mixed Runtime fixture as its working directory.

### Tool inputs and projections

All objects reject unknown fields. `configurationId` is required except for
`oneagent.graph`, where omission lists all configurations and presence selects
one. Limits default to 50 and accept 1-100. Depth accepts 0-4.

`oneagent.graph` accepts optional `configurationId` and `limit`. It returns
ordered configuration summaries with id, name, source format, node/edge counts,
validation counts, diagnostic count, and reference statistics. It never returns
root or provenance paths.

`oneagent.query` accepts `configurationId`, operation `node`, `relations`, or
`traverse`, and the operation-specific ADR-0040 fields. Node requires `nodeId`.
Relations requires `nodeId`, optional direction `incoming|outgoing|both`,
optional exact edge-kind list, and limit. Traverse additionally accepts depth.
It returns canonical node and edge projections using existing stable string
vocabularies. Unsupported operation/field combinations fail as tool errors.

`oneagent.validation` accepts `configurationId` and limit. It runs the canonical
Graph validator and returns validity, error/warning totals, ordered bounded
issues with stable code/severity/message and semantic identifiers, plus
`truncated`.

`oneagent.diagnostics` accepts `configurationId` and limit. It returns ordered
bounded recoverable diagnostics with stable code/severity/kind/message and
semantic reference identifiers where available, plus total and `truncated`.
Source/provenance paths are excluded.

`oneagent.impact` accepts distinct `previousConfigurationId` and
`currentConfigurationId`, optional depth, and limit. It computes
`previous.graph().diff(current.graph())`, then canonical Graph Impact. Comparing
two configurations in one immutable snapshot is explicitly the accepted first
slice; same identifiers are invalid. The result contains deterministic summary,
bounded affected nodes/reasons, and `truncated`.

`oneagent.context` accepts `configurationId`, exact `nodeId`, optional direction
`incoming|outgoing|both`, depth, `maxCandidates`, and `budgetBytes`. It uses
Context intent `Explain`, one exact node seed, all canonical dependency/context
edge kinds accepted by the Context Engine, no node-kind filter, defaults
`both`, depth 2, candidates 50, and budget 16,384. Bounds are depth 0-4,
candidates 1-128, and budget 1-32,768. It returns rendered context and ordered
selected item/relation summaries. Exact-name seeds and custom kind filters are
deferred at the MCP layer, not removed from Analysis.

### Bounds and ordering

ADR-0050's 1 MiB frame, 128 JSON-depth, 256-byte request-id, and 256-byte method
bounds remain. Tool Policy enforces 128-byte ids and 65,536-byte arguments and
output. Runtime enforces the tool-specific limits before domain invocation.
Catalog, configurations, graph projections, validation issues, diagnostics,
impact, context selections, and result members are deterministic. A result that
still exceeds Tool Policy output fails closed; individual strings are never
silently truncated.

### Errors

Stable tool-error codes are `invalid_arguments`, `not_found`, `policy_denied`,
`execution_failed`, and `result_too_large`. Messages are bounded English
sentences selected by code and contain no user value. Protocol parse,
validation, version, method, and response errors retain ADR-0050 precedence.

### Evidence and compatibility

Required non-zero evidence includes protocol catalog/list/call unit and public
tests; Runtime catalog tests over empty and tracked mixed snapshots; Tool Policy
completed/denied/bypass tests; each tool's positive, negative, exact-bound,
one-over, reordered, and repeated cases; stdio tests; real-process discovery,
list, six-family calls, errors, channel purity, EOF and repetition; existing
protocol, Runtime, CLI, Graph, Analysis, Tool Policy, Workspace and full
workspace regressions.

Public compatibility is additive except for the accepted async
`McpServer::dispatch` migration of all existing consumers. Existing HTTP health,
Graph Query wire, Workspace service, CLI, provider, graph, Analysis, Tool Policy,
and adapter behavior must remain byte/semantically compatible within their
existing contracts.

## Consequences

OneAgent exposes useful deterministic semantics to MCP clients while preserving
one owner per domain and enforcing Tool Policy even for read-only work. Startup
cost now includes one complete snapshot build. The catalog is static and cannot
reflect file changes until a new process starts. Async dispatch is a deliberate
public migration but enables policy execution without protocol-owned runtime.

## Rejected alternatives

- Protocol-owned graph or Tool Policy dependencies violate layering.
- Calling semantic projectors directly bypasses ADR-0049.
- Blocking or manually polling Tool Policy futures is unsound and hides
  lifecycle ownership.
- Advertising tools before a complete handler is installed is untruthful.
- Returning source paths or full provenance expands the security/data contract.
- Loading on every call, watching, or caching changes workspace lifecycle.
- Using output schemas before a complete stable output vocabulary overclaims
  compatibility.
- Requiring a live external client makes repository completion non-deterministic.

## Deferred scope

Mutation/write tools, confirmations, authenticated client actors, policy
configuration/persistence, audit sink, cancellation notifications, concurrent
or task-based calls, watcher/reload/cache/history, cross-workspace impact,
name-seed/custom-kind Context inputs, source/provenance paths, output schemas,
other MCP capabilities, remote transport/authentication, external-client
compatibility, IDE/LSP behavior, graph/source semantics, and broad performance,
security, or interoperability claims remain deferred.
