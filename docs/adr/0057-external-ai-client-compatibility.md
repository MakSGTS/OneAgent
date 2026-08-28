# ADR-0057: External AI Client Compatibility

## Status

Accepted

## Context

ADR-0050 and ADR-0051 intentionally implement the stateless MCP revision
`2026-07-28`. Every request carries protocol/client metadata, discovery uses
`server/discover`, complete results contain `resultType`, and tool-list results
contain cache fields. This remains the accepted OneAgent protocol and semantic
authority.

The [external AI client compatibility
investigation](../architecture/external-ai-client-compatibility-investigation.md)
proves that the exact available Codex CLI `0.150.0-alpha.8` and Cursor Agent
`2026.08.25-3e8eec8` cannot initialize the production `oneagent-mcp`. Codex
sends the session-based MCP revision `2025-06-18`; Cursor sends `2025-11-25`.
Both first requests omit the 2026 per-request metadata and receive JSON-RPC
`-32602` before dispatch.

The official dated MCP specifications require initialization, negotiated
connection capabilities, `notifications/initialized`, legacy result shapes,
and EOF-driven stdio shutdown for those revisions. Replacing the accepted
modern protocol would break the existing VS Code and EDT clients. Adding
client-name branches, duplicated catalogs, or an SDK would create a competing
protocol authority. Compatibility therefore needs one bounded connection
adapter over the existing immutable server and semantic handlers.

## Decision

### Canonical statement

OneAgent supports exactly MCP `2025-06-18`, `2025-11-25`, and `2026-07-28`
over the existing `oneagent-mcp` newline-framed stdio process. A new
connection-owned protocol session selects one era from the first valid
response-producing message, owns only negotiated protocol/lifecycle facts, and
uses the existing immutable `McpServer` for the single seven-tool catalog and
semantic handlers. Runtime creates one fresh session for every stream run.

Legacy sessions implement initialize, initialized, ping, tools/list,
tools/call, notifications, and EOF with version-correct result projection.
Modern stateless dispatch remains byte-for-byte and API compatible. No client
name changes protocol behavior. No global mutable state, second binary, second
catalog, new transport, or new production dependency is accepted.

### Protocol authorities and supported set

The authoritative revisions and exact official repository tag commits are:

| Revision | Exact tag commit | OneAgent mode |
| --- | --- | --- |
| `2025-06-18` | `f5ccad944fdf2b7d9cc70cf817f66ca5a8aa03a4` | Legacy negotiated session |
| `2025-11-25` | `38c84e9f93ad191d9eb26d92b945d17bd0efcaf3` | Legacy negotiated session |
| `2026-07-28` | `5f5440bb26a62e2cf3440b92da5a667efa03b267` | Existing stateless per-request metadata |

The investigation's versioned lifecycle, transport, tools, and schema links
are normative. ADR-0050 remains authoritative wherever this decision does not
explicitly add a legacy connection rule. ADR-0051 and ADR-0053 remain
authoritative for catalog, schemas, Tool Policy, semantic projections, path
confinement, and tool failures.

`PROTOCOL_VERSION` remains the modern constant `2026-07-28`. Add a separate
closed public protocol-revision value for all three supported revisions. Its
canonical newest-to-oldest supported order is `2026-07-28`, `2025-11-25`,
`2025-06-18`; this order is for inspection and tests, not initialize fallback.

### Version negotiation

A syntactically and structurally valid legacy `initialize` request selects
legacy negotiation. When its requested version is exactly `2025-06-18` or
`2025-11-25`, the server echoes that exact version. For every other requested
string, including `2026-07-28`, the server responds with legacy
`2025-11-25`, the newest revision supported by the initialization lifecycle.
The client then either accepts that version and sends
`notifications/initialized`, or closes the connection as required by the
legacy specification. A non-string or otherwise invalid initialize parameter
is `InvalidParams` and does not select a mode.

Returning the newest legacy revision for an unsupported initialize request is
protocol negotiation, not a compatibility claim for the requested revision.
The accepted product claims remain limited to the three exact revisions and to
client versions with executed evidence.

A valid modern response-producing request with exact 2026 required metadata
selects modern mode and is dispatched through the existing stateless path.
Parse, envelope, metadata, or version errors do not select modern mode. A
notification received while the mode is undetermined is validated and ignored
without selecting a mode or producing output.

After a mode is selected it never changes. A legacy initialize in modern mode
continues to receive the existing modern invalid-params behavior. A modern
metadata-bearing request in legacy mode is interpreted only by the selected
legacy method contract and cannot create or switch a modern session.

### Connection-owned state machine

`oneagent-protocol` adds a public connection/session type created only from an
immutable `McpServer`. Its states are:

```text
Undetermined
    -- valid legacy initialize --> LegacyAwaitingInitialized(version, facts)
    -- valid modern request ----> Modern

LegacyAwaitingInitialized
    -- notifications/initialized --> LegacyActive(version, facts)
    -- EOF ------------------------> terminal success owned by Runtime

LegacyActive
    -- list/call/ping/notification --> LegacyActive(version, facts)
    -- EOF --------------------------> terminal success owned by Runtime

Modern
    -- any later frame --> existing stateless modern dispatch
    -- EOF -------------> terminal success owned by Runtime
```

The legacy facts are the negotiated revision, validated bounded client
capability object, and validated client implementation identity. They are
connection-local diagnostic/protocol facts only. They are not authentication,
authorization, semantic input, Tool Policy actor selection, logging content,
or a client-name switch. The server retains no request ID, result, arguments,
workspace data, or tool output in session state.

Each `McpStdioTransport::run` creates exactly one fresh session inside the run.
Two sequential or concurrent runs over the same immutable server therefore
have independent mode, version, capability, client-info, lifecycle, and IDs.
The transport retains its public constructor and run signature. The process
still has one stream run, one immutable startup Workspace snapshot, sequential
dispatch, and no detached task.

### Lifecycle and method availability

Legacy behavior is closed as follows:

| State and message | Result | Next state |
| --- | --- | --- |
| `Undetermined` + valid `initialize` request | Version-negotiated initialize result | `LegacyAwaitingInitialized` |
| `Undetermined` + `tools/list`, `tools/call`, or `ping` without valid modern metadata | `ServerNotInitialized` `-32002`, message `Server not initialized` | unchanged |
| `LegacyAwaitingInitialized` + `notifications/initialized` with absent or object params | no response | `LegacyActive` |
| `LegacyAwaitingInitialized` + operational request | `ServerNotInitialized` `-32002` | unchanged |
| legacy state + a second `initialize` request | `InvalidRequest` `-32600` with echoed valid ID | unchanged |
| wrong-state or malformed `notifications/initialized` | no response | unchanged |
| `LegacyActive` + `ping` with absent/empty-object params | empty legacy result | unchanged |
| `LegacyActive` + `tools/list` or `tools/call` | version-correct existing handler result | unchanged |
| legacy state + unknown request method | `MethodNotFound` `-32601` | unchanged |
| any notification other than the one valid initialized transition | validate bounded envelope, emit no response, retain state | unchanged |
| clean EOF between frames | no protocol response; process success | terminal |
| EOF with partial frame or transport/encoding/write/flush failure | existing bounded Runtime failure | terminal failure |

MCP stdio defines no `shutdown` request or `exit` notification. OneAgent does
not borrow the LSP shutdown lifecycle. `shutdown` and `exit` are therefore
unknown legacy methods/notifications with the normal request/notification
behavior. EOF remains the sole client-driven graceful shutdown signal; Ctrl-C
cancellation remains the process-owned alternate success path.

Legacy `notifications/cancelled` is accepted as a silent notification. The
sequential first slice has no concurrent read while a tool handler is active,
so it does not claim in-flight call cancellation. Request timeouts and process
termination remain client-owned. Adding concurrent dispatch or cancellable
handlers requires a later decision.

### Legacy initialize contract

`initialize.params` must be an object containing:

- `protocolVersion`: a string;
- `capabilities`: an object conforming to the selected legacy revision's open
  client-capability shape;
- `clientInfo`: an object with string `name` and `version` and only
  schema-valid known optional fields for that revision.

Unknown schema-permitted fields remain bounded by the existing 1 MiB message
and 128-level nesting limits. Duplicate keys, invalid JSON-RPC, invalid IDs,
wrong known field types, or over-limit values follow existing parse/envelope/
params precedence. Client information is self-reported and never emitted in an
implicit diagnostic.

The exact successful result is:

```json
{
  "protocolVersion": "<negotiated legacy revision>",
  "capabilities": { "tools": {} },
  "serverInfo": { "name": "oneagent", "version": "<crate version>" }
}
```

Titles, descriptions, icons, website URLs, instructions, logging, prompts,
resources, completions, roots, sampling, elicitation, tasks, subscriptions,
and `listChanged` are omitted. The tool catalog is immutable, so
`capabilities.tools={}` is truthful for both legacy revisions.

### Version-specific request and response projection

The existing immutable method registration and semantic handlers remain the
single execution path. The connection supplies a closed response profile:
`Legacy2025` or `Modern2026`. Handlers return validated method fields before a
profile constructs the final result envelope.

| Method/result | Legacy `2025-06-18` and `2025-11-25` | Modern `2026-07-28` |
| --- | --- | --- |
| Request metadata | optional generic `_meta`; no version/capability repetition | existing required protocol/client `_meta` |
| `tools/list` params | absent or empty object; a non-null cursor or unknown field is `InvalidParams` | existing object with required modern metadata; existing cursor rejection |
| Tool definitions | exact seven names/order/descriptions/input schemas/annotations | byte/semantic existing definitions |
| `tools/list` result | `tools` only; omit `resultType`, `ttlMs`, `cacheScope`, and `nextCursor` | existing `resultType="complete"`, `tools`, `ttlMs=0`, `cacheScope="public"` |
| `tools/call` params | exact `name`, optional object `arguments`, optional generic `_meta` | existing params and required modern metadata |
| successful known call | exact existing `content` and `structuredContent`; omit `resultType` and `isError` | existing complete result unchanged |
| known domain failure | exact existing `content`, `structuredContent`, `isError=true`; omit `resultType` | existing complete error result unchanged |
| malformed/unknown call | existing JSON-RPC error ownership and Tool Policy non-invocation | existing behavior unchanged |
| `ping` result | empty object without `resultType` | not added to modern registered methods |

Legacy request `_meta`, when present, may contain only schema-valid generic
legacy metadata such as a progress token; it must not be synthesized into
modern required metadata. Version projection is typed construction, not
serialize-remove-reparse mutation. No response field is selected from client
name or capability guesswork.

The response identifier echoes every accepted bounded string or integer ID.
Legacy errors retain standard JSON-RPC envelopes with no result projection.
Add `ServerNotInitialized=-32002` as the sole new protocol error. Its response
has no data. Existing modern `-32020`, `-32021`, `-32022`, standard error
codes, data, precedence, and encoding remain unchanged.

### Semantic, Tool Policy, and security preservation

The exact catalog remains:

1. `oneagent.context`
2. `oneagent.diagnostics`
3. `oneagent.graph`
4. `oneagent.impact`
5. `oneagent.query`
6. `oneagent.symbols`
7. `oneagent.validation`

Legacy and modern calls reach the same Runtime handler, exact Tool Policy
actor/request/revision, immutable Workspace snapshot, Graph/Analysis owners,
argument validation, result bounds, ordering, redaction, path confinement, and
known-domain failure mapping. Capabilities and client information cannot bypass
Tool Policy or change tool visibility/result semantics. The public clients are
untrusted peers; their names, versions, arguments, and metadata are never
logged implicitly.

All existing 1 MiB frame/message, 128-level JSON, 256-byte ID/method, tool
argument/output, source-path, flush, stdout purity, stderr, cancellation, and
EOF limits remain. This decision adds no credential, filesystem, network,
process-spawn, write, dynamic tool, or remote-user authority.

### Public API and consumer migration

`McpServer::dispatch(&self, input)` remains the public modern stateless API and
retains its current return type and behavior. Add a public connection/session
constructor and `dispatch(&mut self, input)` for transport owners. The exact
type names are implementation details, but construction must require a borrowed
immutable server and must not permit callers to forge an active state or
negotiated facts.

`McpStdioTransport` changes internally from direct server dispatch to one
fresh connection per `run`; its public constructor and run signature stay
unchanged. The `oneagent-mcp` binary and Runtime semantic-server construction
need no public migration.

Existing protocol tests that call `McpServer::dispatch` remain modern
regression authorities. New session tests use the additive connection API.
Runtime process tests gain legacy rows. VS Code and EDT production sources,
configuration, modern request bytes, expected response bytes, and public APIs
must not change; their existing unit and real-process tests prove preservation.
LSP types, lifecycle, transport, binary, and tests are unrelated and unchanged.

### Dependency and packaging policy

The current `serde`, `serde_json`, Tokio, protocol-to-Runtime, semantic-owner,
and Tool Policy dependency graph is sufficient. No Cargo manifest, lockfile,
third-party package, MCP SDK, feature, binary, install package, or client bundle
is accepted. If implementation evidence contradicts this, stop for a separate
dependency decision and user approval rather than expanding this ADR silently.

Downloaded clients, wrappers, project-local config, traces, and logs remain
ignored under `local-artifacts/sprint-35/`. Production and CI do not download
Codex or Cursor. No test or documentation may contain a personal absolute path,
credential, token, client cache, or global configuration mutation.

### Deterministic evidence contract

Task 3 owns protocol-domain implementation and non-zero focused evidence for:

- exact and fallback initialize negotiation for both legacy revisions;
- all state transitions, wrong-order/duplicate behavior, ping, notification
  silence, request IDs, errors, and malformed/boundary cases;
- exact version-specific list/call success/domain-error shapes and forbidden
  field absence;
- repeated calls, independent and interleaved sessions, and immutable-server
  sharing;
- exact existing modern decode/dispatch/result regression.

Task 4 owns Runtime composition and non-zero in-memory/public-process evidence
for initialize → initialized → list → call → EOF, exact Codex/Cursor first
requests, invalid order, fallback, malformed frames, LF/CRLF, stdout/stderr,
flush, cancellation, repeated processes, two-session isolation, cleanup, and
all existing modern public-process behavior.

Task 5 owns repository fixtures, exact public clients, cross-platform synthetic
conformance, existing-client regression, current-state documentation, and the
canonical full workspace gate. Required public evidence is:

- pinned Codex startup, seven-tool visibility, one forced deterministic tool
  success or clearly separated startup-only evidence, one domain failure when
  supported by the public command, repetition, and clean exit;
- pinned Cursor `mcp list-tools` success with all seven definitions, plus a
  public non-interactive tool call/failure only when the supported command can
  execute it without an unrecorded configuration change;
- exact command, executable version/hash, exit, output oracle, client/product
  limitation, and ignored artifact record for every claimed row;
- synthetic conformance for all three revisions on platform-neutral CI;
- existing VS Code, EDT, protocol, Runtime, semantic, catalog, Tool Policy,
  path, dependency, secret, personal-path, generated-artifact, and scope gates.

A required zero-match filter, skip, authentication prompt, unavailable client,
global-config requirement, server/client failure, or missing cleanup blocks the
corresponding task. A named product claim requires the exact executed version;
synthetic fixtures claim only protocol revision conformance.

### First slice and task boundary

The accepted first slice is exactly:

1. add the protocol-owned connection state, legacy parsing/negotiation, response
   profiles, and focused tests while preserving modern stateless dispatch;
2. create one fresh connection inside each existing Runtime stdio run and add
   public production-process lifecycle evidence;
3. add only accepted reusable fixtures/audits/docs and execute exact Codex,
   Cursor, synthetic, existing-client, and full workspace validation;
4. complete independent review before state transition.

No task may combine architecture, protocol, Runtime, evidence, or review
commits. No implementation task may change the catalog or semantic behavior to
make a client pass.

## Compatibility and migration

This decision is additive at the public process boundary. Existing modern
clients continue to send the same bytes and receive the same bytes. Existing
direct `McpServer::dispatch` callers continue to compile and observe stateless
2026 behavior. The Runtime transport internally gains per-run mutable state but
retains its public API and sequential resource ownership.

The only additive public protocol surface is the closed connection/session
construction used by Runtime and tests. If legacy support is later removed, the
connection adapter and its tests can be deleted while retaining the modern
server and transport shape. No persisted data, cache, client configuration, or
semantic migration is introduced.

## Consequences

One `oneagent-mcp` process can serve exact legacy clients and existing modern
OneAgent consumers without duplicating semantics. Session state is explicit,
bounded, connection-local, and testable. Version-correct projection adds code
and matrix surface, and the newest protocol is not a drop-in response to an old
initialize request; the explicit legacy fallback resolves that mismatch.

Strict initialization order may reject permissive peers that skip
`notifications/initialized`, and exact product evidence remains version-bound.
Those are deliberate conformance boundaries rather than reasons to infer
support from client names.

## Rejected alternatives

- Replace 2026 with a 2025 revision: breaks accepted VS Code/EDT behavior and
  discards current MCP features.
- Make all requests session-based: violates the stateless 2026 authority and
  would require modern client migration.
- Accept operational requests before `notifications/initialized`: converts a
  mandatory lifecycle transition into untracked permissive state.
- Put session state in `McpServer` or a process global: leaks facts across
  streams and removes immutable shared-server semantics.
- Maintain separate legacy and modern catalogs/handlers/binaries: creates drift
  and a second public product.
- Transform arbitrary modern JSON into legacy JSON after encoding: can leak or
  delete fields without typed invariants and obscures response bounds.
- Branch on Codex/Cursor names or versions: self-reported identity is neither a
  protocol capability nor a security authority.
- Add an MCP SDK: introduces a second codec/lifecycle owner and an unsupported
  production dependency.
- Add LSP-style shutdown/exit, concurrent calls, background readers, or retry:
  not required by MCP stdio or current client evidence.
- Modify or install client/global configuration from production or CI: exceeds
  repository and security scope.

## Deferred scope

MCP revisions other than the accepted three; product claims beyond exact
executed Codex/Cursor versions; HTTP, SSE, Streamable HTTP, remote, persistent,
multi-tenant, or authenticated transport; OAuth and credentials; global client
configuration; client installation, bundling, publication, or updates; dynamic
tools, subscriptions, tasks, prompts, resources, sampling, elicitation, roots,
logging, completion, and server-initiated requests; concurrent dispatch,
in-flight cancellation, server deadlines, retries, reconnect, snapshot refresh,
watchers, cache changes, new tools or semantics, per-client policy, telemetry,
performance claims, and release publication remain deferred.
