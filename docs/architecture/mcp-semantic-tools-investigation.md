# MCP Semantic Tools Investigation

## Status and scope

This document records the repository and normative protocol evidence required
to decide ADR-0051. Evidence was collected on 2026-08-26 from committed head
`0becccad`. It does not accept architecture or claim implemented semantic MCP
tools.

## Normative MCP evidence

The selected authority remains MCP revision `2026-07-28`, matching ADR-0050:

- [Server tools specification](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
  defines capability declaration, discovery, invocation, error separation,
  security expectations, and result behavior.
- [Schema reference](https://modelcontextprotocol.io/specification/2026-07-28/schema)
  is the human-readable type authority.
- [Versioned TypeScript schema source](https://raw.githubusercontent.com/modelcontextprotocol/modelcontextprotocol/5f5440bb26a62e2cf3440b92da5a667efa03b267/schema/2026-07-28/schema.ts)
  is the field-level source used for the candidate wire matrix.

Confirmed normative facts are:

- a server that supports tools declares `capabilities.tools`; immutable tools
  may omit `listChanged`;
- `tools/list` accepts optional cursor pagination and returns an ordered tool
  array with optional `nextCursor`; the OneAgent catalog can be one complete,
  zero-TTL, public-cache page;
- each tool requires a name and an object-root `inputSchema`; title,
  description, icons, `outputSchema`, and annotations are optional;
- recommended tool names are 1-128 ASCII letters, digits, underscore, hyphen,
  and period, matching the existing Tool Policy identifier bound;
- `tools/call` requires a tool name and accepts an optional object argument;
- a successful or known-tool execution failure is a call result with `content`,
  optional `structuredContent`, and optional `isError`; an unknown tool or
  malformed protocol request is a JSON-RPC protocol error;
- when `structuredContent` is returned with an output schema it must conform;
  omitting an output schema avoids claiming a complete stable output schema;
- annotations include read-only, destructive, idempotent, and open-world hints
  and are advisory rather than authorization;
- request `_meta` and ADR-0050's protocol validation remain unchanged.

The TypeScript schema source is pinned to full commit
`5f5440bb26a62e2cf3440b92da5a667efa03b267`, identified by the
[official stable `2026-07-28` release](https://github.com/modelcontextprotocol/modelcontextprotocol/releases/tag/2026-07-28).
The human-readable specification URLs remain revision-directory-specific.
ADR-0051 cites the selected revision and immutable field-level source without
depending on mutable upstream branch content.

## Current repository ownership

### Protocol and transport

`oneagent-protocol` owns bounded MCP values, codec, discovery, method registry,
and synchronous sequential dispatch. `McpServer::new()` registers only
`server/discover`; the public capability map is empty and `tools/list` returns
`MethodNotFound`. Handler registration and `MethodHandler` are private. Runtime
owns `McpStdioTransport`, which calls synchronous dispatch while preserving
newline framing, protocol-only stdout, terminal failures, EOF, and cancellation.
The `oneagent-mcp` binary owns process streams and Ctrl-C composition.

Tool Policy execution is asynchronous. A blocking poll or a Runtime-specific
handler inside the protocol crate would violate existing ownership. The
smallest compatible candidate is an additive public boxed-future handler
contract and asynchronous sequential `McpServer::dispatch`, with Runtime
awaiting it. Current consumers are limited to protocol tests, Runtime transport,
and Runtime process tests, so the migration is discoverable and bounded.

### Semantic owners

- Runtime `WorkspaceSnapshotBuilder` synchronously creates one deterministic
  ordered immutable `WorkspaceSnapshot` from a root. Each configuration exposes
  identity, name, format, graph, validation, diagnostics, reference evidence,
  and statistics. No watcher or background service is required.
- ADR-0040 and Runtime Graph Query define exact node, relation, and bounded
  traversal vocabularies. HTTP routes must remain unchanged.
- Graph owns canonical validation, diff, and impact. Impact accepts previous and
  current graphs, their diff, and bounded options.
- Analysis owns synchronous Context Engine selection and rendering over one
  graph with exact seed, policy, and byte budget contracts.
- Tool Policy owns authorization and execution for every executable request,
  including `ReadOnly`. It supplies bounded identifiers/arguments/output,
  fail-closed exact policy evaluation, cancellation, terminal results, and
  redacted audit records.

Runtime is the only valid composition owner. The dependency direction is
`oneagent-runtime -> oneagent-protocol/graph/analysis/tool-policy`; no semantic
owner may depend on MCP or Runtime. Runtime already depends on protocol and
graph. The planned additions are local path dependencies on existing
`oneagent-analysis` and `oneagent-tool-policy`; no new third-party package or
version is necessary.

## Candidate catalog and contracts

The Roadmap requires exactly six capabilities. The following names satisfy
both MCP recommendations and Tool Policy bounds:

| Tool | Existing authority | Candidate bounded input | Candidate projection |
|---|---|---|---|
| `oneagent.graph` | Workspace snapshot and Graph report | optional configuration identifier and limit | ordered configuration/graph summaries |
| `oneagent.query` | ADR-0040 | configuration identifier; `node`, `relations`, or `traverse`; accepted direction/kind/depth/limit | canonical bounded node/relation/traversal projection |
| `oneagent.validation` | Graph validator | configuration identifier and issue limit | validity, counts, ordered bounded issues |
| `oneagent.diagnostics` | Workspace configuration diagnostics | configuration identifier and diagnostic limit | ordered bounded diagnostics without source paths |
| `oneagent.impact` | Graph diff and Impact | previous/current configuration identifiers, depth, limit | canonical summary and bounded affected nodes/reasons |
| `oneagent.context` | ADR-0044 Context Engine | configuration identifier, exact node-id seed, direction/depth/candidate/budget bounds | bounded rendered context and ordered selected items |

All six are `readOnlyHint: true`, `destructiveHint: false`,
`idempotentHint: true`, and `openWorldHint: false`. Each has a closed object-root
input schema. The first slice can omit output schemas while returning both one
compact JSON text content block and the same JSON object as structured content.
This preserves machine and text clients without asserting a broader schema
stability contract.

Unknown tool names are protocol `InvalidParams`. A known tool with invalid
arguments, missing configuration/node, domain failure, denied policy, or
oversized result returns `isError: true` with a stable code/message object and
matching text. No error contains filesystem paths, provenance paths, raw source
chains, policy internals, or argument contents.

## Tool Policy composition candidate

For every known call Runtime creates one bounded `ToolRequest` with:

- the exact catalog name as `ToolId`;
- a fixed authenticated local MCP actor identity;
- a bounded request identity independent from arbitrary MCP request-id length;
- canonical compact JSON arguments as `ToolArguments`;
- `ToolEffect::ReadOnly`;
- an exact catalog allow policy and fixed policy revision.

Runtime calls only `execute_tool`; the executor performs the semantic
projection after authorization and returns bounded compact JSON as
`ToolOutput`. Annotations never replace policy evaluation. With sequential
dispatch, one fixed internal request identity is sufficient because there is at
most one in-flight invocation; ADR-0051 must decide and document this explicitly.
The existing `NeverCancelled` adapter is acceptable only because semantic
owners are synchronous and ADR-0050 transport cancellation remains between
requests; concurrent execution and mid-computation cancellation remain deferred.

## Workspace and lifecycle candidate

The process working directory is the smallest source-independent workspace-root
contract. Before reading MCP frames, `oneagent-mcp` builds exactly one immutable
snapshot and semantic catalog, then starts the existing stdio loop. Empty roots
produce an empty but usable catalog. A typed build failure terminates before
protocol output with one bounded stderr category and non-zero exit. No path is
serialized.

This preserves ADR-0037 structured ownership and avoids creating Runtime `App`,
watchers, caches, ports, or background tasks. Existing protocol-only process
tests can use an empty temporary working directory; semantic process tests can
set the tracked mixed `apps/runtime/tests/fixtures/workspace_service` fixture as
working directory.

## Bounds and deterministic behavior

- ADR-0050 retains 1 MiB messages and aggregate JSON depth 128.
- Tool names and Tool Policy identifiers remain at most 128 bytes.
- Tool arguments and output remain at most 65,536 bytes through Tool Policy.
- List/query/validation/diagnostic/impact item limits use default 50 and maximum
  100 unless ADR-0040 or an owner has a smaller accepted maximum.
- Graph/context depth remains 0-4; context candidates remain 1-128 and rendered
  budget remains within the Context Engine maximum, with a smaller MCP default.
- Catalog order, configuration order, node/edge order, issue/diagnostic order,
  impact order, context order, JSON member construction, and repeated responses
  must be deterministic.
- If a bounded projection still cannot serialize within Tool Policy output,
  the known call fails closed instead of truncating an individual string or
  leaking an internal constructor error.

## Compatibility and public-surface impact

The async dispatch migration changes a public protocol method and therefore
requires updating every current consumer in the same protocol task. It does not
remove request/response values, framing, discovery, or error semantics. The
semantic catalog is additive. Existing Runtime HTTP health, Graph Query,
Workspace service, CLI, providers, graph semantics, Analysis APIs, and Tool
Policy APIs remain unchanged.

## Deterministic evidence matrix

| Layer | Required positive evidence | Required negative/boundary evidence |
|---|---|---|
| Protocol | truthful discovery, stable six-item list, reordered/repeated call values | unknown tool, non-object/missing/extra arguments, handler failure, bounds |
| Runtime catalog | every tool over immutable fixture snapshot, repeated equality, Tool Policy completed audit | denied policy, missing configuration/node, invalid vocabulary/range, oversized output, no path leakage |
| Graph/query | summary, exact node, relations, traversal, validation, diagnostics | empty graph, wrong identifier/kind, limit/depth exact and one-over |
| Impact/context | two selected configurations, diff/impact, exact context seed and budget | same/missing configuration, missing seed, budget/depth/candidate bounds |
| Transport/process | discover, list, six families, error result, sequential repetition, EOF | malformed/unknown calls, invalid UTF-8/oversized frame, clean stderr/stdout, startup failure |
| Compatibility | complete protocol, Runtime, CLI, Graph, Analysis, Tool Policy and workspace tests | dependency/public API/catalog/schema/bypass/ignored-test/no-real-effect audits |

The tracked mixed workspace fixture contains Designer and EDT configurations,
including a deliberate diagnostic, and is sufficient for positive graph,
diagnostic, impact, and context evidence. No completion oracle needs a live MCP
client, credential, external network after specification retrieval, fixed port,
real signal, wall-clock timing, mutation, or concrete tool side effect.

## Decisions required from ADR-0051

ADR-0051 must accept or reject the candidate catalog and exact per-tool input
subset; handler future type and migration; capability/list metadata; manual
schema validation; error/result encoding; Tool Policy identities/policy/audit;
snapshot root/startup failure; bounds; dependency edges; public evidence; and
compatibility matrix. It must also decide whether impact between two
configuration graphs in one snapshot is a truthful first slice.

## Explicit deferrals

Deferred scope is mutation tools, write confirmation, user/session identity,
policy administration and persistent audit, watcher/reload/cache behavior,
historical workspace snapshots, cross-workspace impact, broader Context policy
or name-seed surfaces, provenance/source paths, prompts/resources/other MCP
capabilities, concurrency/tasks/progress/cancellation notifications, remote
transport/authentication, external-client compatibility, IDE/LSP work, graph
or source-adapter semantics, and performance/security claims beyond executable
bounds.
