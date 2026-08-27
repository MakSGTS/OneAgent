# ADR-0054: LSP Adapter

## Status

Accepted

## Context

Sprint 32 must expose the supported navigation, symbol, and recoverable
diagnostic evidence through an editor-neutral Language Server Protocol boundary.
The repository already owns typed source locations, immutable Workspace
snapshots, deterministic symbol search, and recoverable semantic diagnostics,
but it has no LSP wire model, Content-Length transport, initialization
lifecycle, public process, or method evidence. The supporting repository and
pinned upstream evidence is recorded in
`docs/architecture/lsp-adapter-investigation.md`.

The adapter must preserve Graph and Workspace as semantic authorities, Runtime
as process and projection owner, protocol as wire owner, and the existing MCP,
HTTP, CLI, and VS Code behavior. It must not read or parse source after startup,
claim mutable editor-document analysis, or invent use-site/definition ranges
from declaration points.

## Decision

### Canonical statement and authority

OneAgent implements one static LSP 3.17 server over a dedicated
`oneagent-lsp` Content-Length-framed stdio process. The selected authority is
the official Microsoft LSP 3.17 specification and meta-model at immutable
commit `8be2e191506ced923953b94b985c4a1831757b39`, as pinned by the investigation.

The completed first slice advertises exactly:

- `positionEncoding: "utf-16"`;
- `textDocumentSync: 0` (`None`);
- `workspaceSymbolProvider: true`; and
- `diagnosticProvider` with identifier `oneagent`,
  `interFileDependencies: true`, and `workspaceDiagnostics: false`.

It implements `initialize`, `initialized`, `workspace/symbol`,
`textDocument/diagnostic`, `shutdown`, and `exit`. It accepts and ignores the
standard `$/cancelRequest` notification because dispatch is sequential and no
accepted request starts a detached operation. It advertises and implements no
other LSP capability or method.

Workspace-symbol locations provide source navigation. Sprint 32 does not claim
`textDocument/definition`, document symbols, references, or another cursor-
position semantic operation.

### Ownership and dependency direction

`oneagent-protocol` owns:

- bounded LSP JSON-RPC request IDs, messages, validation, errors, lifecycle
  state, capabilities derived from installed handlers, method dispatch, and
  compact response encoding;
- method-specific wire parameter/result validation; and
- a transport-independent handler boundary for initialize compatibility,
  symbols, and diagnostics.

`oneagent-runtime` owns:

- Content-Length framing over injected Tokio readers/writers;
- immutable Workspace construction before serving input;
- canonical file-URI generation and Workspace/Configuration confinement;
- graph symbol and recoverable diagnostic projection;
- the public `oneagent-lsp` binary, stdio channels, exit status, and bounded
  stderr category; and
- public in-memory and real-process evidence.

Graph, Common, BSL, EDT, and Designer XML retain source identity, location,
diagnostic, and semantic ownership. Protocol never imports those crates.
Runtime does not move graph or diagnostic meaning into the transport.

Existing direct dependencies are sufficient. No Cargo member, third-party
package, package version, Node dependency, or lockfile change is accepted.
The Runtime binary target is additive inside the existing package.

### Public protocol values and bounds

LSP messages use JSON-RPC string or LSP `integer` IDs. String IDs are at most
256 UTF-8 bytes; integers are exactly in the signed 32-bit LSP range
`-2,147,483,648..=2,147,483,647`. `null`, fractions, exponent-form values,
booleans, arrays, objects, and integers outside that range are invalid
requests. Method names are non-empty and at most 256 UTF-8 bytes.

One decoded JSON body is at most 1,048,576 bytes and at most 128 object/array
levels, aligned with the existing public protocol bound. Duplicate object keys
at any level are invalid requests. Fully encoded responses must satisfy the
same body/depth bounds before any transport write.

Protocol errors are the standard bounded fixed messages:

| Name | Code | Message |
|---|---:|---|
| Parse error | `-32700` | `Parse error` |
| Invalid request | `-32600` | `Invalid Request` |
| Method not found | `-32601` | `Method not found` |
| Invalid params | `-32602` | `Invalid params` |
| Internal error | `-32603` | `Internal error` |
| Server not initialized | `-32002` | `Server not initialized` |
| Request failed | `-32803` | `Request failed` |

Errors include the established request ID when valid and omit `data`. They
never include input, URI, filesystem path, source content, provenance, stderr,
source chain, Rust type name, or semantic payload.

### JSON validation and lifecycle precedence

For one decoded body, protocol validation is deterministic:

1. parse exactly one JSON value and reject duplicate keys or over-depth values;
2. require a top-level object and `jsonrpc: "2.0"`;
3. classify a present `id` as a request and otherwise as a notification;
4. validate request ID, method, and absent/object params;
5. enforce lifecycle state before method-specific validation;
6. validate method-specific fields; and
7. invoke one handler only after every prior check succeeds.

The server state machine is:

```text
Uninitialized -> AwaitingInitialized -> Running -> Shutdown -> Exited
       |                   |                |          |
       +-------------------+----------------+----------+-> ExitedFailure
```

- `initialize` is the only request accepted in `Uninitialized`, may occur once,
  and transitions only after a successful response is constructed.
- A pre-initialize request other than `initialize` receives
  `ServerNotInitialized`; notifications are dropped except `exit`.
- `initialized` is a notification with absent/object params and moves
  `AwaitingInitialized` to `Running`. Requests before it receive
  `ServerNotInitialized`; unrelated notifications are dropped.
- Semantic requests and `shutdown` are accepted only in `Running`.
- `shutdown` has absent params, returns `null`, and moves to `Shutdown`.
- In `Shutdown`, requests receive `InvalidRequest`; notifications are dropped
  except `exit`.
- `exit` never receives a response. It selects successful terminal status only
  from `Shutdown`; every other state selects failure.
- EOF before `exit`, framing/I/O failure, encoding failure, or an impossible
  handler result is terminal failure. No implicit graceful EOF exists.
- A second initialize, initialized in the wrong state, malformed exit/shutdown,
  or request after shutdown follows the lifecycle error above and never invokes
  semantic code.

Unknown requests in `Running` use `MethodNotFound`. Unknown notifications and
`$/cancelRequest` produce no response and no state change.

### Initialize, roots, and position encoding

`initialize` requires an object with `processId` as LSP `integer` or null,
`rootUri` as a non-empty string, and `capabilities` as an object. Known optional
`clientInfo`, `locale`, `rootPath`, `initializationOptions`, `trace`,
`workspaceFolders`, and `workDoneToken` must have specification-compatible
shapes when present; unknown extension fields are ignored after depth/body
validation.

Runtime constructs the canonical file URI for the normalized startup Workspace
root and validates initialize input through the handler boundary:

- `rootUri` must equal that exact canonical URI;
- non-null `rootPath` is rejected;
- `workspaceFolders`, when present and non-null, must contain exactly one object
  whose `uri` equals the same root URI and whose `name` is a string;
- absent or null workspace folders are accepted;
- multiple, non-file, non-canonical, malformed, escaping, or conflicting roots
  are `InvalidParams`; and
- no rejected root or path appears in an error.

Canonical file URIs use UTF-8 percent encoding with uppercase hex, forward
slashes, no query/fragment/user-info, and platform-correct absolute roots.
On Windows, Runtime converts canonical drive paths to `file:///C:/...`,
canonical UNC paths to `file://server/share/...`, and removes the extended
`\\?\` or `\\?\UNC\` filesystem prefix before URI encoding; other extended
namespaces and incomplete UNC roots are rejected. Runtime owns encoding and
lexical containment. It does not resolve symlinks or read the filesystem during
a request.

The server supports only UTF-16 positions, as LSP requires. If
`capabilities.general.positionEncodings` is absent, UTF-16 is selected by
default. If present, it must be a non-empty unique string array containing
`utf-16`; otherwise initialize is `InvalidParams`. Other client capabilities
do not expand server behavior.

Capabilities are a truthful projection of installed production handlers. The
lifecycle-only Task 4 process advertises only `positionEncoding` and
`textDocumentSync`; Task 5 adds `workspaceSymbolProvider` with its handler; Task
6 adds `diagnosticProvider` with its handler. The completed initialize result is
deterministic:

```json
{
  "capabilities": {
    "positionEncoding": "utf-16",
    "textDocumentSync": 0,
    "workspaceSymbolProvider": true,
    "diagnosticProvider": {
      "identifier": "oneagent",
      "interFileDependencies": true,
      "workspaceDiagnostics": false
    }
  },
  "serverInfo": {"name": "oneagent", "version": "0.1.0"}
}
```

No dynamic registration, experimental field, server request, progress, trace,
workspace-folder change, document synchronization, diagnostic refresh, or
server notification is emitted.

### Content-Length framing and channel ownership

Runtime accepts an ASCII header block ending in `\r\n\r\n`, with at most
8,192 bytes including the delimiter. Every header line is `Name: Value`.
Header names compare ASCII case-insensitively.

- Exactly one `Content-Length` is required. Its value is one or more ASCII
  decimal digits, with no sign or surrounding whitespace, no leading zero
  except the exact value `0`, and a value at most 1,048,576.
- Zero length reaches the protocol decoder and produces one parse error.
- At most one `Content-Type` is accepted. Its case-insensitive value must be
  exactly `application/vscode-jsonrpc; charset=utf-8` after trimming outer ASCII
  spaces, or the frame is a terminal framing failure.
- Unknown, duplicate, malformed, non-ASCII, bare-LF, overlong, or EOF-truncated
  headers are terminal framing failures with no response for that frame.
- The body is exactly `Content-Length` bytes and must be UTF-8. Truncated EOF,
  invalid UTF-8, or I/O failure is terminal. Additional bytes begin the next
  header; coalesced and fragmented input is equivalent.

Each response body is compact UTF-8 JSON preceded by
`Content-Length: <bytes>\r\n\r\n` and flushed before reading the next frame.
Notifications write nothing. stdout contains only framed protocol messages.
The public binary writes at most one fixed bounded English failure category to
stderr and never a path, URI, payload, source, provenance, or source chain.

The adapter processes sequentially and spawns no task. Reader, writer,
snapshot, server state, and terminal outcome have one owner. No request queue,
concurrent handler, timeout, retry, detached cancellation, log stream, or
background watcher is accepted.

### Workspace symbol behavior

`workspace/symbol` params is an object with required `query` string and optional
well-typed `workDoneToken` and `partialResultToken`; unknown fields are
`InvalidParams`. The query is at most 256 UTF-8 bytes. Empty and whitespace
queries are data and are accepted.

Runtime selects only Procedure, Function, and EDT Query nodes with exactly one
distinct typed confined location containing a span. Module nodes are omitted
because they have no range and the server does not advertise
`workspaceSymbol/resolve`. Missing, repeated-identical, conflicting,
non-UTF-8, escaping, or absent-span location evidence is omitted without error.

Matching and ordering reuse ADR-0053:

- Rust Unicode lowercase substring comparison with no trim, normalization,
  locale, fuzzy score, regex, glob, alias, source-content match, or client-side
  semantic filter;
- empty query matches every accepted symbol;
- one result per node after identical-location deduplication; and
- exact sort by lowercased name, exact name, LSP kind, canonical node ID, then
  Configuration ID.

Procedure and Function map to LSP `Function` (`12`); Query maps to LSP `Object`
(`19`). Each complete `WorkspaceSymbol` contains exact `name`, numeric `kind`,
Configuration name as `containerName`, and `location` with canonical file URI
and zero-based range. Tags and data are absent.

One-based half-open source spans convert by subtracting one from line and
column. Every emitted zero-based line and character is an LSP `uinteger` in
`0..=2,147,483,647`; an out-of-range handler projection is an internal error.
Current declaration points therefore become zero-length ranges at character
zero and are independent of source encoding. Runtime never reads a line,
adjusts to an identifier, or claims an exact identifier range.

The complete result is limited to 100 symbols and the protocol body bound.
Runtime collects the complete projected candidate set and checks its length
before constructing the response. More than 100 matches or an otherwise
over-bound encoded result is `RequestFailed`; no silent prefix, partial result,
pagination, or truncation claim is returned. No match returns `[]`, never null.

### Pull document diagnostics

`textDocument/diagnostic` params is an object with exact required
`textDocument: {"uri": <string>}` and optional well-typed `identifier`,
`previousResultId`, `workDoneToken`, and `partialResultToken`. Unknown fields,
an identifier other than `oneagent`, or malformed/non-file/non-canonical/
escaping URI is `InvalidParams`. `previousResultId` is accepted and ignored
because this server returns no result ID and never advertises unchanged reports.
Both progress-token fields accept only a string or the same signed 32-bit LSP
`integer`; fractions and out-of-range numbers are `InvalidParams`.

A canonical Workspace-confined URI with no projected diagnostic returns a full
empty report. Runtime does not require the file to exist during the request.

For the requested URI, Runtime selects recoverable diagnostics whose
`source_node` exists in the same Configuration and whose source node has exactly
one distinct typed confined location with a span for that exact URI. A missing
source node, missing/ambiguous/conflicting/escaping location, or span-less node
is omitted rather than guessed. The diagnostic's own stable ordering remains
primary; URI, zero-based range, code, and message provide deterministic ties.

Each LSP `Diagnostic` contains:

- the source-node span converted to a zero-based range;
- severity `1` for Graph Error and `2` for Graph Warning;
- the stable Graph diagnostic code as a string;
- source `oneagent`; and
- the exact existing bounded semantic message.

Tags, related information, code description, data, candidates, semantic IDs,
provenance, and source values are absent. This is a navigation anchor at the
known declaration point, not a claim that the exact erroneous token is ranged.

At most 100 diagnostics may be returned for one document. Runtime computes the
complete projected candidate set before response construction; more than 100
items or an over-bound encoded result is `RequestFailed`. Success is always
`{"kind":"full","items":[...]}` without `resultId` or related documents.
The server emits no publish, workspace-diagnostic, refresh, or unchanged report.

### Compatibility and sensitive-data policy

The new protocol module and Runtime binary are additive. Existing MCP revision,
LF framing, seven-tool catalog/schemas/Tool Policy behavior, immutable snapshot
semantics, HTTP, CLI, Graph, adapters, cache, and VS Code extension remain
byte/semantically compatible. The LSP process is not invoked by the current VS
Code extension and does not prove any external client compatibility.

No LSP result or implicit error includes an absolute filesystem path,
Configuration root, Workspace root, source content, opaque source identifier,
provenance record, hash, Runtime error, stderr text, or source chain. File URIs
necessarily identify confined source documents; they are returned only for
typed locations beneath both accepted roots.

### Evidence and implementation sequence

Task 3 implements protocol-only messages, validation, lifecycle, capabilities,
dispatch, errors, and tests while preserving MCP.

Task 4 implements Runtime framing, URI/root validation, immutable process
composition, public binary, lifecycle outcomes, and in-memory/raw-process tests
with only the truthful lifecycle capabilities. It does not register a semantic
method before its handler exists.

Task 5 implements complete workspace-symbol projection and public process
evidence across EDT and Designer locations.

Task 6 implements full pull document diagnostic projection and public process
evidence, including the tracked located EDT diagnostic and empty Designer case.

Task 7 runs the complete canonical Rust, protocol, Runtime, process,
cross-platform CI, dependency, compatibility, sensitive-data, generated-
artifact, scope, and documentation gates. Every test filter must match a
non-zero case.

Required evidence includes positive, malformed, missing, duplicate, unknown,
pre-initialize, post-shutdown, reordered, repeated, exact/over-bound, fragmented,
coalesced, EOF, channel-purity, root/URI/escape, Unicode, symbol ambiguity,
missing/conflicting location, diagnostic omission, empty result, exit status,
and resource-cleanup cases as applicable. Numeric conformance evidence covers
exact and one-over LSP integer/uinteger bounds for request IDs, `processId`,
progress tokens, and outbound positions, plus fractional rejection. URI
evidence includes platform-independent drive/UNC oracles and a Windows-only
canonical-path assertion.

## Consequences

Any LSP 3.17 client that can launch the bounded public process and use its exact
static capability set can search navigable immutable symbols and pull located
recoverable diagnostics. The server observes one startup snapshot and does not
track open buffers or file changes. Workspace-symbol and diagnostic requests
may fail when their complete deterministic result exceeds 100 items rather than
silently truncating.

The accepted navigation experience is selection of a workspace-symbol result,
not go-to-definition. Diagnostic ranges identify the known source declaration
point and may be less precise than the source construct that caused the
diagnostic.

## Rejected alternatives

- Reusing MCP LF framing or discovery violates LSP transport and lifecycle.
- Adding an LSP SDK or types crate is unnecessary and adds a production
  dependency for a deliberately small static surface.
- Advertising definition or document symbols would require occurrence or full
  symbol ranges that current production facts do not retain.
- Returning Module workspace symbols without ranges would require resolve
  capability and state that the first slice does not need.
- Silently truncating workspace symbols or diagnostics hides incomplete data
  because these selected result shapes have no accepted truncation marker.
- Push or workspace diagnostics require client/document scheduling and broader
  source-location coverage absent from the immutable first slice.
- Reading files, parsing source, or decoding opaque provenance during a request
  violates accepted semantic ownership and snapshot immutability.
- Migrating the VS Code extension from MCP to LSP combines an editor-neutral
  adapter with unrelated product integration and packaging behavior.
- Accepting arbitrary roots, multiple folders, sockets, pipes, or remote URIs
  expands process, security, and compatibility contracts without evidence.

## Deferred scope

Mutable document synchronization; open/change/close/save handling; watcher or
cache refresh; definition, declaration, document symbols, references,
completion, hover, signature help, document highlights, code actions, code
lens, formatting, rename, folding, semantic tokens, inlay hints, call/type
hierarchy, commands, edits, and every other language feature; Module symbols
and workspace-symbol resolve; partial results and progress; request
cancellation during execution; unchanged diagnostic result IDs; publish,
workspace, related, tagged, configured, suppressed, or refreshable diagnostics;
dynamic registration; multiple Workspace folders; symlink-target guarantees;
socket/pipe/TCP/IPC and remote transport; parent-process monitoring; IDE client
integration; external-client compatibility; telemetry; refactoring; and broad
performance/security claims remain deferred.

No Coverage Registry capability changes from architecture documentation alone.
