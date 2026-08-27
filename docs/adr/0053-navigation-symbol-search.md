# ADR-0053: Navigation and Symbol Search

## Status

Accepted

## Context

Sprint 31 must add one bounded symbol-search and source-navigation experience
to the accepted desktop VS Code extension. The current graph retains opaque
source identifiers but has no typed source location, the Runtime MCP catalog
cannot enumerate symbols, and the extension owns only connection lifecycle.
The repository and pinned editor evidence is recorded in
`docs/architecture/navigation-symbol-search-investigation.md`.

The implementation must preserve Graph as semantic authority, Runtime as the
workspace and projection authority, Tool Policy as the execution gate, protocol
as the wire owner, and TypeScript as a thin editor adapter. It must not obtain
locations by parsing opaque provenance identifiers or by searching source files
after the immutable Runtime snapshot is built.

## Decision

### Canonical statement and ownership

OneAgent adds typed source locations to Common and optional structured location
evidence to Graph provenance. EDT and Designer XML producers attach truthful
locations for the accepted symbol families while they still own the source
path and declaration line. Runtime searches canonical immutable graph nodes,
confines locations to the startup Workspace root, and exposes one additive
read-only `oneagent.symbols` MCP tool. The VS Code extension invokes that tool
only on explicit command demand, presents the server order in one Quick Pick,
and opens only a revalidated workspace-relative result.

Common owns location value validation. Graph owns association with semantic
facts. Adapters own production. Workspace owns the startup root and immutable
configuration graphs. Runtime owns matching, ordering, deduplication, path
projection, result bounds, and Tool Policy execution. Protocol owns the tool
catalog and closed schema. TypeScript owns wire-result validation, invocation
lifecycle, Quick Pick presentation, URI construction, coordinate conversion,
document opening, selection, reveal, and fixed user-facing failures. No layer
decodes an opaque provenance source to recover a location.

### Common source-location model

`oneagent-common` adds these public values:

- `SourcePath`, an owned UTF-8 path of 1 through 4,096 bytes;
- `SourcePosition`, a one-based `u32` line and one-based `u32` column, both
  non-zero;
- `SourceSpan`, a half-open ordered start/end pair; and
- `SourceLocation`, one `SourcePath` plus an optional `SourceSpan`.

The values are cloneable, comparable, orderable, hashable, and expose only
validated accessors. Construction is fallible. Error kinds and messages are
closed, stable, and contain no rejected value.

`SourcePath` is an evidence path, not a public URI or proof of containment. It
accepts either an absolute producer path or a producer-relative path. It
normalizes `\` separators to `/`, collapses no component, rejects NUL, empty
components, `.` and `..` components, a trailing separator, and values over the
byte bound. A leading POSIX root, Windows drive prefix, or UNC prefix may be
retained because current producers observe native absolute paths. Runtime, not
Common, decides whether that evidence belongs to the configured Workspace.
Non-UTF-8 filesystem paths cannot become first-slice locations.

`SourceSpan` requires `start <= end` in `(line, column)` order. Equality is
accepted and represents a navigation point rather than an exact text range.
Current one-based BSL declaration lines become point spans at column 1.
Module file locations have no span. No current producer claims an identifier
end column. Query Language byte ranges are not converted into these positions.

The model has no serialization dependency. Runtime maps it explicitly to JSON.
No third-party dependency is accepted.

### Graph provenance and compatibility

`Provenance` adds `location: Option<SourceLocation>`. Its existing `new`
constructor and `source()` accessor retain their signatures and behavior and
produce no structured location. A new explicit constructor/builder and
`location()` accessor add typed evidence. Existing graph node and edge
identities, names, kinds, payloads, source identifiers, ordering, and insertion
rules are unchanged.

Location participates in provenance equality and hashing because it is part of
fact evidence. Graph diff/report/semantic-index behavior continues to use the
complete provenance value and must be migrated without dropping or separately
inventing location data. Graph query adds no Sprint 31 fuzzy-search public API;
Runtime may iterate the existing deterministic node view and project a bounded
result.

Multiple identical locations across a node's provenance collapse to one
candidate. A supported node with no location, or with more than one distinct
location, is non-navigable and is omitted from `oneagent.symbols`. Runtime does
not guess which source is primary. Tests must cover missing, repeated-identical,
and conflicting locations.

### Accepted producer slice

The first slice contains these exact graph kinds:

| Graph kind | EDT | Designer XML | Location |
|---|---|---|---|
| `Module` | included | included | source file, no span |
| `Procedure` | included | included | module source, point at declaration line/column 1 |
| `Function` | included | included | module source, point at declaration line/column 1 |
| `Query` | included | not produced | EDT module source, point at declaration line/column 1 |

EDT and Designer XML builders attach location while the typed module descriptor,
configuration root, and parsed one-based line are available. They may retain
their existing opaque source identifiers for identity and other evidence.
Current public adapter entry points remain source compatible; any helper added
for root-relative production is internal or additive.

All other node kinds remain non-searchable in this tool even when one producer
happens to have a source path. Calls, references, edges, metadata members,
diagnostics, and exact identifier-column ranges remain outside this producer
slice. Designer XML Query is absent rather than synthesized.

### Workspace root and path confinement

`WorkspaceSnapshot` retains the exact normalized startup root supplied to
`WorkspaceSnapshotBuilder::build` and exposes it through a read-only accessor.
The root is snapshot identity needed for source projection, not MCP output.
Cache encode/decode, watcher rebuilds, manual test fixtures, and graph-query
consumers migrate so the root is preserved or explicitly supplied. Existing
configuration-root access remains unchanged.

For each candidate location Runtime performs lexical confinement without a
post-snapshot filesystem read:

1. interpret an absolute evidence path directly, or join a relative evidence
   path to that symbol's configuration root;
2. normalize native separators and components without resolving symlinks;
3. require the candidate to be under both its configuration root and the
   snapshot Workspace root;
4. strip the Workspace root;
5. require a non-empty relative path with no root, prefix, `.` or `..` component;
   and
6. emit forward-slash-separated UTF-8 text at most 4,096 bytes.

An invalid, ambiguous, non-UTF-8, absolute-output, cross-configuration, or
escaping candidate makes that node non-navigable and therefore omitted. It is
not returned as a path-bearing error and no absolute/source value appears in a
diagnostic. Symlink-target containment is not claimed because no filesystem
read occurs during a call; the extension's second lexical gate and the accepted
trusted local workspace are the first-slice boundary.

### Symbol query semantics

`oneagent.symbols` accepts a closed object with:

| Field | Contract |
|---|---|
| `query` | required string, 1-256 UTF-8 bytes after no trimming or other mutation |
| `configurationId` | optional exact canonical Configuration ID |
| `kinds` | optional non-empty unique array of `module`, `procedure`, `function`, `query` |
| `limit` | optional integer, default 50, range 1-100 |

Whitespace is data. An empty query is invalid. Matching applies Rust Unicode
lowercasing to the query and canonical `EntityName`, performs one substring
test, and does no normalization, locale mapping, trimming, tokenization, regex,
glob, fuzzy score, transliteration, synonym expansion, qualified-name
invention, source-content match, or TypeScript-side semantic filtering.

When `configurationId` is absent, all configurations participate. A supplied
unknown Configuration is `not_found`. Omitted `kinds` means all four accepted
kinds. Unknown, duplicate, empty, or wrong-type fields are `invalid_arguments`.

One graph node contributes at most one result after identical location
deduplication. Equal names on different nodes/configurations remain separate.
The complete matching set is sorted by this exact tuple:

1. Unicode-lowercased canonical name;
2. exact canonical name;
3. wire kind in lexicographic order;
4. canonical node ID; and
5. canonical Configuration ID.

Runtime computes `total` before applying `limit`, returns the first `limit`
items, and sets `truncated = total > results.length`. `total` is a non-negative
JSON-safe integer or execution fails closed. Discovery, insertion, provenance,
and request field order cannot change the result.

### MCP catalog, schema, and result

ADR-0053 extends the ADR-0051 catalog to these seven exact names in canonical
lexicographic order:

1. `oneagent.context`
2. `oneagent.diagnostics`
3. `oneagent.graph`
4. `oneagent.impact`
5. `oneagent.query`
6. `oneagent.symbols`
7. `oneagent.validation`

The new definition uses the same description, closed object-root schema, and
annotations contract as the existing read-only tools. `capabilities.tools`,
single-page `tools/list`, zero TTL, public cache scope, request metadata,
sequential dispatch, framing, and MCP revision remain unchanged. Existing six
tool definitions, arguments, results, and errors are unchanged. The additive
catalog member intentionally changes exact-catalog clients; the VS Code client
and every repository-owned catalog assertion migrate atomically with Runtime.
No external-client compatibility claim exists.

A successful structured result is:

```json
{
  "results": [
    {
      "configurationId": "...",
      "configurationName": "...",
      "nodeId": "...",
      "name": "...",
      "kind": "procedure",
      "location": {
        "path": "configuration/src/CommonModules/Sales/Module.bsl",
        "span": {
          "start": { "line": 12, "column": 1 },
          "end": { "line": 12, "column": 1 }
        }
      }
    }
  ],
  "total": 1,
  "truncated": false
}
```

`span` is omitted for file-only Module locations. Member order follows the
example. Compact `content` and `structuredContent`, Tool Policy's 65,536-byte
argument/output bounds, the 1 MiB MCP frame bound, JSON depth, and stable error
envelope retain ADR-0050/0051 behavior. A semantically limited result that
still exceeds the Tool Policy output bound is `result_too_large`; paths or
names are never shortened. No source identifier, provenance record, absolute
path, source content, hash, Runtime root, or configuration root is returned.

### Tool Policy and failure contract

The immutable policy revision remains `oneagent.mcp.read-only.v1` and adds an
exact allow rule for `oneagent.symbols`, actor `oneagent.mcp`, effect
`ReadOnly`. The existing request ID, canonical compact arguments, execution,
audit, confirmation, and `NeverCancelled` behavior remain unchanged. An MCP
annotation does not authorize execution.

Protocol validation precedes semantic argument validation, which precedes
Configuration lookup, candidate projection, Tool Policy execution outcome, and
output-size validation. Stable tool-error codes remain `invalid_arguments`,
`not_found`, `policy_denied`, `execution_failed`, and `result_too_large` with
fixed bounded English messages containing no input or path. No result is a
successful empty result (`results=[]`, `total=0`, `truncated=false`). A missing
or ambiguous location is not an error because the node is outside the
navigable result set.

### VS Code command and client behavior

The package contributes one additional command:

| ID | Title |
|---|---|
| `oneagent.searchSymbols` | `OneAgent: Search Symbols` |

The existing empty `activationEvents` remains valid because the contributed
command auto-activates on VS Code 1.134.0. The command does not connect, spawn,
restart, reload, index, or search the filesystem. It requires the existing
Runtime lifecycle to be `connected`; otherwise it shows the fixed information
message `OneAgent must be connected before searching symbols.` and returns.

Each accepted invocation owns exactly one `createQuickPick` instance and its
subscriptions. The Quick Pick is single-select, has a fixed English title and
placeholder, starts with no result, and sends no request until the input is
non-empty and within the 256-byte bound. Items preserve server order. `label`
is the exact symbol name; `description` contains the closed kind plus exact
Configuration name; `detail` contains only the returned relative path and
one-based line when present. Equal labels remain distinct.

The current sequential client is extended with one closed `symbols` call and a
strict result validator. At most one request remains in flight. Input changes
coalesce to the latest value: the current local generation becomes stale, its
late response is ignored, and the latest valid query starts only after the
outstanding response completes. No MCP cancellation notification, second
concurrent request, process termination, or reconnect occurs. Hiding the Quick
Pick or deactivation invalidates the invocation; a late response cannot update
or open UI. Repeated command invocation closes and disposes the previous
invocation before creating the next.

Accepting one exact item closes the picker, revalidates its returned path as a
relative forward-slash path without empty, `.`, `..`, root, drive, UNC, NUL, or
over-bound components, and joins it under the sole trusted file Workspace URI.
The extension verifies the joined lexical path remains under that root before
calling `workspace.openTextDocument` and `window.showTextDocument`.

For a span, one-based line/column values are checked and converted by
subtracting one to VS Code's zero-based UTF-16 coordinates. A point span creates
a caret selection. A non-empty span creates the exact selection. The editor
reveals the selection in the center when practical. A Module without a span
opens without changing selection. The extension does not inspect source text,
adjust UTF-8 offsets, search for the symbol name, or claim an exact identifier
range.

Malformed results are `protocol_failure` and close the incompatible process
through existing client ownership. A server tool error, missing file, document
open failure, stale response, or rejected path never falls back to filesystem
search or another result. Active user-visible failures use fixed English
messages and expose no executable, absolute path, payload, provenance, stderr,
or source chain. Cancellation/hide and empty results are not errors.

Every command registration, Quick Pick, event subscription, generation,
pending action, and cancellation marker is owned by the extension context or
one invocation and is disposed on replacement, hide, completion, disconnect,
configuration replacement, failure, or deactivation. Connection lifecycle and
child-process cleanup remain owned by ADR-0052.

### Evidence and implementation sequence

Task 3 implements Common values, Provenance migration, Workspace-root retention,
and the exact EDT/Designer producer slice. Required non-zero evidence covers
value bounds/order/errors, point/half-open spans, existing provenance
constructors, missing/identical/conflicting locations, both real adapter
fixtures, exact lines, non-UTF-8/escape evidence where portable, repeated and
reordered builds, cache/snapshot compatibility, and existing graph identities.

Task 4 implements the seventh protocol definition, Tool Policy rule, Runtime
projection, schemas, errors, and public process behavior. Evidence covers every
argument field and bound, four kinds, both source formats, multi-configuration
aggregation/filtering, Unicode/whitespace, duplicate names, deterministic
ordering, missing/conflicting locations, confinement, exact/over result bounds,
policy denial/bypass, six-tool regression, malformed frames, channel purity,
EOF, and repeated fresh processes.

Task 5 implements the TypeScript call, result validator, command, Quick Pick,
path gate, document navigation, cancellation/coalescing, repetition, and cleanup.
Pure tests cover wire bounds, result/member validation, stale and late responses,
sequential coalescing, path/coordinate cases, failures, and disposal. Pinned
VS Code 1.134.0 Extension Host and real-process tests cover explicit activation,
connected/disconnected gates, real result presentation and selection, file-only
opening, missing file, repetition, disconnect, and deactivation with no orphan.

Task 6 runs the complete clean extension package matrix, public process matrix,
canonical Rust workspace gate, macOS/Windows CI definition audit, exact catalog/
schema/handler/policy/manifest inventory, dependency/license audit, and deferred-
scope absence checks. Test filters must match non-zero cases.

### Compatibility and dependency impact

Common and Provenance gain additive public types/accessors while existing
constructors remain source compatible. Provenance equality/hash intentionally
includes optional location. WorkspaceSnapshot gains root identity and its
internal/manual/cache constructors migrate; existing configuration access and
semantic behavior remain. MCP's catalog changes from six to seven exact names,
so repository clients that intentionally assert the complete catalog migrate.
The six existing tool schemas and results remain byte/semantically compatible.

The VS Code manifest gains one explicit-demand command and the client gains one
semantic call; connection, configuration, status, spawn, framing, failure,
shutdown, and package identity remain unchanged. HTTP, CLI, Graph Query wire,
Analysis, LLM providers, source contents, and external clients do not change.
No Cargo or production Node dependency is added.

## Consequences

A connected supported desktop workspace can search and open canonical Module,
Procedure, Function, and EDT Query graph facts using immutable Runtime evidence.
Results are bounded, deterministic, ambiguity-preserving, and contain only
workspace-relative paths. Nodes without one unambiguous confined location are
not shown. Current declaration navigation is line/column-1 precision, not exact
identifier-range precision.

The additive MCP catalog is intentionally incompatible with exact six-member
catalog assertions until they migrate. Snapshot and provenance values carry
more evidence, and persistent cache codecs must preserve it. No background
index, watcher behavior, or post-start filesystem search is introduced.

## Rejected alternatives

- Parsing opaque source `EntityId` values is producer-specific and leaks
  identity evidence into consumers.
- Returning absolute paths, configuration roots, full provenance, or source
  contents expands the sensitive-data contract.
- Putting locations only in MCP or TypeScript creates a second semantic owner.
- Attaching one location directly to `GraphNode` loses fact-level provenance
  and cannot represent conflicting evidence.
- Reusing `oneagent.query` requires incompatible conditional `nodeId` semantics.
- Searching a truncated graph result in TypeScript is incomplete and duplicates
  semantic matching.
- Empty-query enumeration, fuzzy scoring, locale collation, and normalization
  lack a first-slice deterministic oracle.
- Choosing the first of multiple locations silently hides ambiguity.
- Resolving symlinks or reading file contents during a call breaks the immutable
  snapshot boundary.
- Standard language providers belong to the editor-neutral LSP boundary.
- Concurrent MCP requests or cancellation notifications expand ADR-0050/0052
  lifecycle contracts without being required for one Quick Pick.
- A new MCP SDK, path library, fuzzy-search package, UI framework, or bundler is
  unnecessary for the accepted behavior.

## Deferred scope

Exact identifier-column ranges; reference, declaration-from-cursor, definition,
document-symbol, workspace-symbol, or reference providers; LSP; metadata/member
and arbitrary node navigation; Designer XML Query; source contents/fragments;
fuzzy/relevance ranking; Unicode normalization and locale collation; aliases;
multi-location choice UI; symlink-target guarantees; mutable workspace reload,
watch, cache refresh, or pagination; concurrent MCP and protocol cancellation;
automatic Runtime connection/install/update; diagnostics UI; chat/context UI;
EDT plugin integration; remote/web/multi-root support; external-client
compatibility; Marketplace publication/signing; telemetry; edits/refactoring;
and broad performance or security claims remain deferred.
