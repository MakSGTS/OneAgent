# External AI Client Compatibility Investigation

## Status and scope

- Sprint: 35 — External AI Client Compatibility
- Evidence captured: 2026-08-28
- Planning baseline: `cd876c83`
- Decision target: ADR-0057
- Scope: investigation only; this document does not accept an architecture or
  claim implemented client compatibility

## Purpose

This investigation identifies why the public `oneagent-mcp` process cannot be
initialized by the exact available Codex and Cursor clients, pins the relevant
client and protocol evidence, maps the current OneAgent ownership boundary, and
defines the decisions and executable oracles required before compatibility is
implemented. Downloaded clients, project-local configurations, wrappers, and
wire logs remain ignored under `local-artifacts/sprint-35/`.

## Pinned client evidence

The current user authorized the exact client reads and executions summarized
below. Personal absolute paths are intentionally replaced by repository and
application categories.

| Client | Exact version | Executable SHA-256 | Provenance and retrieval |
| --- | --- | --- | --- |
| Codex CLI bundled with the installed official ChatGPT desktop application | `codex-cli 0.150.0-alpha.8` | `4ff5e75f028e913cfeb53bd7319f87573cdce6538c1b1ccc44ce62d5ce51ca1d` | Existing signed application resource, inspected 2026-08-28; no client download or installation was performed |
| Cursor Agent for `darwin/arm64` | `2026.08.25-3e8eec8` | `2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233` | Official [installer](https://cursor.com/install) and exact [package](https://downloads.cursor.com/lab/2026.08.25-3e8eec8/darwin/arm64/agent-cli-package.tar.gz), retrieved 2026-08-28 into the ignored evidence directory |

The retained Cursor archive has locally observed SHA-256
`81d4de7349e208d4ce441ca9c2d4e7d019ec2fbeb1137a79099fd8c4b8662f5f`.
The installer publishes the versioned URL but no vendor checksum, so these two
hashes identify the executed local artifact rather than asserting a vendor-
signed digest.

The official [Codex MCP documentation](https://learn.chatgpt.com/docs/extend/mcp)
confirms local stdio support and configuration shared by Codex clients. The
official [Cursor MCP guide](https://prod.cursor.com/docs/cli/mcp) defines
`agent mcp list-tools <identifier>`, project-to-global configuration
precedence, and automatic MCP use. The [Cursor MCP configuration
guide](https://prod.cursor.com/help/customization/mcp) identifies
`.cursor/mcp.json` as the project-local stdio configuration boundary. Those
product pages are mutable operational documentation; the executable versions
and hashes above pin the clients actually tested.

### Isolated invocation boundary

Both clients were pointed only at a repository-built `target/debug/oneagent-mcp`
through ignored trace wrappers. Cursor ran from a disposable nested Git
workspace whose only MCP definition was project-local. Codex ran with
`--ignore-user-config`, `--ephemeral`, `--sandbox read-only`, an explicit
disposable working directory, and command-line MCP configuration. No tracked
file or repository-external server was used. The commands did not request a
write to user-global client configuration; Cursor may still read merged global
configuration according to its documented precedence.

Redacted reproductions are:

```text
<ONEAGENT_CURSOR_CLI> mcp list-tools oneagent

<ONEAGENT_CODEX_CLI> exec --ignore-user-config --ephemeral \
  --skip-git-repo-check --sandbox read-only \
  -C <DISPOSABLE_WORKSPACE> \
  -c 'mcp_servers.oneagent.command="<REPOSITORY_TRACE_WRAPPER>"' \
  -c 'mcp_servers.oneagent.required=true' \
  -c 'mcp_servers.oneagent.startup_timeout_sec=5' \
  'Reply exactly OK without calling tools.'
```

## Observed wire incompatibility

The exact first requests captured at the server boundary are:

```json
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{"elicitation":{"form":{},"url":{}}},"clientInfo":{"name":"codex-mcp-client","title":"Codex","version":"0.150.0-alpha.8"}}}
```

```json
{"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{"elicitation":{"form":{}}},"clientInfo":{"name":"Cursor","version":"1.0.0"}},"jsonrpc":"2.0","id":0}
```

Each public command exited `1`. Cursor reported
`MCP error -32602: Invalid params`; Codex reported that the required MCP server
failed initialization with the same JSON-RPC code. The production server
rejected the request before method dispatch because its decoder requires
`params._meta.io.modelcontextprotocol/protocolVersion` and
`params._meta.io.modelcontextprotocol/clientCapabilities` on every
response-producing request. No `initialize` response, `notifications/initialized`,
`tools/list`, or `tools/call` was observed, so current compatibility is
conclusively absent but later client behavior is not inferred from an
unexecuted exchange.

The client-specific public success oracles are:

- Cursor: `mcp list-tools oneagent` exits zero and prints all seven tool names
  and their accepted schemas after a successful initialize/list exchange.
- Codex: a required MCP server reaches ready state, its seven tools are visible
  to the run, a forced representative call completes, and the command exits
  zero. A no-call prompt proves startup but is insufficient for call support.
- Both: repeat the public workflow, close client input, and observe prompt clean
  server exit with protocol-only stdout and bounded/empty stderr.

Task 5 must distinguish network/model authentication needed by a product
command from MCP server behavior. A product login prompt or unavailable model
blocks that public-client row rather than being converted into server success.

## Authoritative MCP revisions

The official MCP repository tags resolved on 2026-08-28 to:

| Revision | Exact tag commit | Normative lifecycle/tools sources |
| --- | --- | --- |
| `2025-06-18` | `f5ccad944fdf2b7d9cc70cf817f66ca5a8aa03a4` | [Lifecycle](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle), [stdio transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports), [tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools), [schema](https://modelcontextprotocol.io/specification/2025-06-18/schema) |
| `2025-11-25` | `38c84e9f93ad191d9eb26d92b945d17bd0efcaf3` | [Lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle), [stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [schema](https://modelcontextprotocol.io/specification/2025-11-25/schema) |
| `2026-07-28` | `5f5440bb26a62e2cf3440b92da5a667efa03b267` | [Basic protocol](https://modelcontextprotocol.io/specification/2026-07-28/basic), [stdio transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio), [tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools), [schema](https://modelcontextprotocol.io/specification/2026-07-28/schema), [pinned TypeScript schema](https://raw.githubusercontent.com/modelcontextprotocol/modelcontextprotocol/5f5440bb26a62e2cf3440b92da5a667efa03b267/schema/2026-07-28/schema.ts) |

The dated specification directories and exact tag commits pin the normative
inputs. The `2026-07-28` release is already the accepted ADR-0050/ADR-0051
authority; this investigation does not supersede it.

### Confirmed 2025 lifecycle facts

- `initialize` is the first interaction. It carries `protocolVersion`, client
  capabilities, and client information outside request `_meta`.
- If the requested version is supported, the server returns that exact version;
  otherwise it may return another supported version and the client disconnects
  if it cannot use it.
- The initialize result carries `protocolVersion`, server capabilities,
  `serverInfo`, and optional instructions. A tools server declares
  `capabilities.tools`; immutable tools need not advertise `listChanged`.
- After the initialize response, the client sends
  `notifications/initialized`. Operational messages use the negotiated
  connection state rather than repeating version/capability metadata.
- `tools/list` returns `tools` and optional `nextCursor`. It does not use the
  2026 `resultType`, `ttlMs`, or `cacheScope` fields.
- `tools/call` returns content, optional structured content, and optional
  `isError`; it does not use the 2026 `resultType` field. Known domain failures
  remain tool results, while malformed or unknown calls remain JSON-RPC errors.
- No protocol shutdown request is defined for stdio. The client closes child
  stdin, waits, then may escalate termination. EOF therefore remains the
  primary graceful server shutdown signal.
- Requests should not precede a successful initialize response except pings;
  server requests should not precede `notifications/initialized` except the
  limited specified cases. Exact server behavior for invalid ordering and a
  duplicate initialize remains an ADR-0057 decision.

The `2025-11-25` revision adds richer optional implementation and task
capabilities, but Cursor's observed request declares only elicitation form
support. OneAgent has no evidence or need to advertise tasks, roots, sampling,
elicitation, prompts, resources, logging, or completion support.

### Confirmed 2026 preservation facts

ADR-0050 intentionally selected stateless `2026-07-28`: each request includes
protocol/client metadata, `server/discover` replaces initialization, successful
results include `resultType`, and EOF closes stdio. ADR-0051/ADR-0053 add the
immutable tool catalog, `ttlMs=0`, `cacheScope="public"`, structured tool
results, and semantic execution through Tool Policy. Compatibility must retain
this exact mode rather than silently translating modern consumers into a legacy
session.

## Current repository ownership and behavior

### Protocol domain

- `crates/protocol/src/mcp.rs` owns the single public
  `PROTOCOL_VERSION="2026-07-28"`, bounded JSON-RPC IDs/messages, mandatory
  per-request metadata, errors, encoding, 1 MiB message bound, and 128-level
  nesting bound.
- `crates/protocol/src/server.rs` owns immutable `McpServer` registration and
  async stateless `dispatch(&self, input)`. It registers `server/discover`,
  `tools/list`, and `tools/call`; no state survives a request.
- `ResultResponse::complete` always inserts `resultType="complete"`.
  `ToolsListHandler` always inserts `ttlMs=0` and `cacheScope="public"`.
- The public types are re-exported by `crates/protocol/src/lib.rs`. Existing
  protocol tests are `mcp_domain` and `mcp_dispatch` plus unit tests.

### Runtime and process

- `apps/runtime/src/mcp.rs` owns the sequential newline-framed Tokio adapter.
  `McpStdioTransport::run(&self, ...)` borrows one immutable `McpServer`,
  dispatches each line independently, suppresses notification output, flushes
  each response, and exits successfully on clean EOF or cancellation.
- `apps/runtime/src/mcp_tools.rs` owns the semantic composition and Tool Policy
  execution over one startup `WorkspaceSnapshot`.
- `apps/runtime/src/bin/oneagent-mcp.rs` owns cwd snapshot construction, real
  stdin/stdout, Ctrl-C, exit status, and bounded redacted stderr diagnostics.
- `apps/runtime/tests/mcp_process.rs` proves public modern process behavior,
  exact tool results, framing, malformed input, EOF, and error diagnostics.

The exact current lexicographic catalog is:

1. `oneagent.context`
2. `oneagent.diagnostics`
3. `oneagent.graph`
4. `oneagent.impact`
5. `oneagent.query`
6. `oneagent.symbols`
7. `oneagent.validation`

Graph, Analysis, Workspace, and Tool Policy remain the semantic owners. The
protocol crate must not interpret their results or acquire a Runtime dependency.

### Existing consumers

- The VS Code extension owns a separate TypeScript modern client. It sends
  `server/discover` and tools requests with 2026 metadata and validates the
  exact seven-tool catalog. Its unit, process, Extension Host, and package tests
  are mandatory compatibility evidence.
- The EDT Java probe sends one modern `server/discover`, validates the exact
  compatible response, closes stdin, and requires clean process completion.
  Its Runtime/process compatibility tests must remain green.
- Runtime protocol/transport/process tests and documentation embed modern
  request/response shapes. LSP uses a separate protocol state machine and must
  not be coupled to MCP negotiation.
- No production code depends on an external MCP SDK. Current workspace
  dependencies are sufficient for a state machine and version-specific JSON
  projection; no new production dependency is evidenced.

## Bounded compatibility candidates

These are candidates for ADR-0057, not accepted decisions.

### Candidate A — connection-owned protocol session

Add a transport-independent session value whose initial mode is unresolved.
The first valid message selects either a 2025 negotiated lifecycle or the
existing 2026 stateless mode. The session calls the immutable server for shared
method/catalog/semantic execution and owns only version, capabilities, client
information, and lifecycle state. Runtime creates one session per stream.

This directly models legacy ordering and prevents global state, but it changes
the dispatch mutability/API and requires exact isolation and migration tests.

### Candidate B — explicit context passed to pure dispatch

Keep `McpServer` immutable and let a separate connection driver decode lifecycle
messages, then pass an immutable negotiated context into version-aware dispatch.
This preserves a pure semantic dispatcher and makes state ownership explicit,
but can duplicate validation or allow inconsistent state/context unless the
public construction boundary is closed.

### Candidate C — legacy compatibility adapter

Wrap the current server with a legacy adapter that implements initialize and
projects legacy request/results to/from the modern handler boundary. This may
minimize changes to modern code, but a blind JSON transformation risks leaking
`resultType`/cache fields, misclassifying errors, or inventing per-request
metadata. It is viable only with typed projection and one shared catalog.

### Evidence-rejected directions

- Replacing `2026-07-28` with a 2025 revision breaks the accepted VS Code/EDT
  clients and ADR-0050 rather than adding compatibility.
- A process-global session permits cross-connection leakage and contradicts
  deterministic independent process/stream tests.
- Separate binaries or duplicated catalogs create two public MCP products and
  semantic drift without evidence.
- An MCP SDK adds a new production dependency and competing protocol authority;
  the existing bounded codec/transport already owns the required primitives.
- Client-name branching is not protocol conformance. Behavior must depend on
  negotiated revision and capabilities, with exact clients used only as public
  acceptance evidence.

## Required protocol and lifecycle matrix

ADR-0057 must resolve every `Decision` cell; Tasks 3-5 must turn the accepted
rows into non-zero executable evidence.

| Case | 2025-06-18 / 2025-11-25 requirement | Existing 2026-07-28 preservation |
| --- | --- | --- |
| First valid interaction | `initialize`; exact accepted response version and tools capability | Direct metadata-bearing `server/discover` or tool request remains valid |
| Unsupported initialize version | Decision: deterministic negotiated fallback or closed error and exact code/data | Existing `-32022` metadata-version behavior unchanged |
| `notifications/initialized` | Accepted once after initialize response; no response | Remains an ordinary notification with no session effect |
| Pre-initialize operational request | Decision: exact error and state retention | Not applicable; every request is independent |
| Duplicate initialize | Decision: exact error and state retention | `initialize` remains unknown/invalid for modern mode |
| `tools/list` | No repeated metadata; legacy result has tools and optional cursor only | Exact seven tools plus `resultType`, `ttlMs=0`, `cacheScope="public"` |
| `tools/call` success/domain error | Legacy content/structured content/`isError`; no `resultType` | Existing complete result and Tool Policy semantics unchanged |
| Unknown method/tool or malformed params | Exact JSON-RPC precedence selected by ADR | Existing precedence and redaction unchanged |
| Notification/unknown notification | Never emit a response | Never emit a response |
| Request IDs | Echo each valid string/integer ID; no outstanding reuse | Existing bounded exact echo |
| EOF/shutdown | EOF is graceful; no protocol shutdown request | Existing EOF/cancellation completion |
| Repetition | Repeated list/call in one initialized connection and repeated processes | Repeated independent requests/processes remain deterministic |
| Isolation | Two sessions cannot share version, capabilities, client info, IDs, or lifecycle | No legacy state can affect a modern stream |
| Framing and diagnostics | One JSON message per line, bounded stdout/stderr, prompt cleanup | Existing 1 MiB/LF/flush/error behavior unchanged |

Synthetic fixtures may represent any ADR-supported revision and malformed
peer. They prove protocol conformance, not named product compatibility. Only an
exact executed client/version may be named as a supported product.

## Deterministic evidence seams

### Protocol tests

- exact initialize decode/result for both observed requests;
- supported, alternate, and unsupported negotiation;
- every state/order transition and error precedence;
- version-specific absence/presence of `resultType`, `ttlMs`, `cacheScope`,
  initialization fields, and request `_meta`;
- exact seven definitions and success/domain-error projections for every
  accepted revision;
- malformed, duplicate-key, depth, size, ID, notification, and unknown-method
  cases;
- repeated calls and two independent sessions, including interleaved use;
- byte/semantic equality for every existing modern regression fixture.

### Runtime and public process tests

- partial/multiple LF frames, notification silence, flush, EOF, cancellation,
  reader/writer failure, stderr, and cleanup in each accepted mode;
- production `oneagent-mcp` initialize → initialized → list → call → EOF for
  both observed legacy request shapes;
- direct modern discover/list/call regression, existing semantic tool matrix,
  repeated processes, and simultaneous independent streams;
- existing VS Code and EDT real-process compatibility with no source change
  required unless an accepted public API migration demands it.

### Exact public clients

- macOS host evidence uses only the pinned executables and ignored project-local
  configuration against the repository-built production binary;
- Cursor's public `mcp list-tools` is the minimum list oracle; a supported
  non-interactive command is required before claiming an actual tool call;
- Codex requires MCP startup plus an explicitly requested deterministic tool
  call; startup alone is recorded separately;
- CI remains platform-neutral and uses repository-owned protocol/process
  fixtures because the proprietary client artifacts are not release inputs.

## ADR-0057 decision checklist

ADR-0057 must decide:

1. the exact supported revision set and ordering;
2. requested-version equality, fallback, and unsupported-version behavior;
3. how the first message selects legacy versus modern mode;
4. connection/session state, ownership, mutability, isolation, and public API;
5. initialize parameter validation and exact result fields;
6. initialized notification, pre-initialize, duplicate initialize, and invalid
   transition behavior;
7. accepted client capability storage and use, including open extension data;
8. version-specific request parsing and response projection without field
   leakage;
9. error precedence, codes, bounded data, request-ID behavior, and redaction;
10. notification, ping, cancellation, EOF, and any shutdown behavior;
11. Runtime transport construction, connection scope, and cleanup;
12. modern `server/discover` and per-request metadata preservation;
13. exact catalog, Tool Policy, semantic result, and domain-error equivalence;
14. consumer migration for protocol tests, Runtime, VS Code, and EDT;
15. message, nesting, timeout, stderr, concurrency, and resource limits;
16. dependency policy and rejection of an MCP SDK absent new approval/evidence;
17. the synthetic conformance and exact Codex/Cursor acceptance matrices;
18. public-client authentication/configuration limitations and claim language;
19. the exact first implementation slice and task ownership; and
20. deferred revisions, clients, transports, authentication, publication, and
    semantic/tool expansion.

## First-slice and deferred boundary

The evidence supports investigating one hybrid-compatible `oneagent-mcp`
process for the two observed legacy revisions plus the accepted modern
revision, but ADR-0057 must make the final supported-set decision. The first
slice may change only protocol/session projection and Runtime stdio composition,
then add repository-owned conformance/public-client evidence. It must preserve
the single catalog and semantic executor and add no production dependency.

Deferred until separately evidenced and authorized: protocol revisions not
accepted by ADR-0057; named clients other than the exact Codex and Cursor
versions; HTTP, SSE, Streamable HTTP, remote or multi-tenant transport;
authentication and credential exchange; client installation or distribution;
global configuration management; dynamic tools/subscriptions/tasks; new tool
semantics; concurrent dispatch; performance claims; and release publication.

## Conclusion

The incompatibility is reproducible and localized: both exact clients begin
with legacy initialization while OneAgent intentionally implements only the
stateless 2026 request shape. The repository has sufficient protocol,
transport, client, fixture, and consumer evidence to decide ADR-0057 without a
new dependency. Production work remains blocked until that ADR fixes the
supported revisions, state machine, projections, migrations, and executable
completion matrix.
