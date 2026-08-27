# Navigation and Symbol Search Investigation

This investigation records the committed repository and pinned upstream
evidence available before ADR-0053. It does not accept a navigation architecture
or implement production behavior.

## Confirmed repository baseline

- Planning HEAD is `6ac7a073`, `Plan Sprint 31 Navigation and Symbol Search`.
  Sprint 30 is completed by `4b3198d1`; Sprint 31 is the unique active target
  after this investigation starts.
- The desktop workspace extension targets VS Code `1.134.0`, Node 24, pnpm
  `11.19.0`, and exact development dependencies. It contributes only
  `oneagent.connect` and `oneagent.disconnect`, activates only on command demand,
  supports exactly one trusted file-backed workspace folder, and owns one
  sequential `oneagent-mcp` process.
- `RuntimeClient` validates the exact six-name ADR-0051 tool catalog during
  connection and currently sends only `server/discover` and `tools/list`.
  It has no public semantic call method, request arguments/result validation,
  cancellation input, navigation state, or editor-opening behavior.
- The public `oneagent-mcp` process builds one immutable `WorkspaceSnapshot`
  from its current directory. Each `WorkspaceConfigurationSnapshot` retains its
  discovered configuration root, but `WorkspaceSnapshot` does not retain the
  encompassing process workspace root.
- ADR-0051 exposes graph nodes without payload, provenance, source paths, or
  locations. `oneagent.query` requires one exact node ID and supports only
  `node`, `relations`, and `traverse`; it cannot enumerate or search symbols.
- The Tool Policy catalog consists of the same six exact read-only tool IDs.
  Adding a tool therefore requires catalog, schema, handler, policy, Runtime,
  process, and extension compatibility changes in one bounded accepted slice.
- No LSP adapter, VS Code definition/reference/document/workspace-symbol
  provider, search command, Quick Pick, source-opening adapter, or navigation
  test exists.

## Pinned editor authority

The selected platform remains desktop VS Code `1.134.0`, pinned by ADR-0052 to
the official release tag and source commit
`474a349ad5b745e512ef86b864d1c74f7264dd7a`. The committed lockfile selects
`@types/vscode` `1.134.0`, and the installed locked package was inspected only
as a local validation copy of that accepted API.

Relevant official sources are:

- [immutable VS Code API declaration](https://github.com/microsoft/vscode/blob/474a349ad5b745e512ef86b864d1c74f7264dd7a/src/vscode-dts/vscode.d.ts);
- [VS Code API reference](https://code.visualstudio.com/api/references/vscode-api);
- [Quick Pick UX](https://code.visualstudio.com/api/ux-guidelines/quick-picks);
- [commands](https://code.visualstudio.com/api/extension-guides/command);
- [activation events](https://code.visualstudio.com/api/references/activation-events);
- [extension anatomy](https://code.visualstudio.com/api/get-started/extension-anatomy).

The living documentation explains intended UX. Version-specific API shape is
anchored by the immutable source commit, exact `engines.vscode`, locked types,
and pinned Extension Host.

Confirmed API facts are:

- contributed commands auto-activate the extension on the accepted host without
  explicit `onCommand` activation events;
- `window.showQuickPick` accepts a `CancellationToken` and resolves to one item
  or `undefined`, while `window.createQuickPick` exposes input-change,
  acceptance, hide, and selection events and must be explicitly disposed;
- Quick Pick supports deterministic supplied item order plus label,
  description, and detail fields; client-side filtering is presentation, not a
  complete semantic search oracle for a truncated server result;
- `workspace.openTextDocument(Uri)` rejects a missing/unreadable file and
  editor ownership begins after a successful open;
- `window.showTextDocument` returns the editor, whose selection can be set and
  whose range can be revealed;
- `Position`, `Range`, and `Selection` use zero-based lines and zero-based UTF-16
  code-unit character offsets; and
- a command handler, Quick Pick, cancellation source, request, and every event
  subscription need explicit extension-context or invocation ownership.

No production dependency is required. Node built-ins and the host-provided
`vscode` API cover validation, relative-path joining, cancellation ownership,
Quick Pick, document opening, selection, and reveal.

## Source-location evidence inventory

### Common, BSL, and Graph

- `oneagent-common` currently owns only `EntityId` and `EntityName`; it has no
  `SourcePath`, `SourcePosition`, `SourceSpan`, or `SourceLocation` type.
- Semantic Model 2.0 already assigns source paths, source identifiers, and
  source spans to Common and conceptual provenance contains `source` plus an
  optional span.
- `BslSymbol` retains a one-based declaration line for every accepted top-level
  Procedure or Function. Calls and static Query declarations also retain
  one-based lines; Query Language ranges use zero-based UTF-8 byte offsets and
  are a distinct contract.
- `GraphNode` owns identity, name, kind, payload, and a vector of `Provenance`.
  `Provenance` owns only an optional opaque `EntityId`, producer, origin,
  confidence, and resolution. Its Rustdoc explicitly states that Common lacks
  source-specific identity and span types.
- Adding a structured optional location to `Provenance` matches the conceptual
  authority and preserves the fact-level evidence model. Adding location only
  to an MCP result or TypeScript client would create a second semantic owner.
  Adding it only to `GraphNode` would not cover edge/diagnostic provenance and
  would duplicate provenance ownership.

### EDT production evidence

- EDT module analysis reads an exact `EdtModuleDescriptor::path`, extracts
  `BslSymbol` values with declaration lines, and stores the module path as one
  opaque source `EntityId`.
- EDT `insert_declarations` discards `BslSymbol::line()` and attaches only the
  module-source provenance to Procedure and Function nodes. This is the exact
  loss point for first-slice declaration navigation.
- EDT static Query extraction retains its one-based declaration line, but the
  line is encoded only in a length-prefixed provenance context string used by
  Query Reads evidence. Consumers have no accepted parser for that string.
- Metadata, member, role, subsystem, SKD, XDTO, service, form-command, Writes,
  and other EDT contributors create source identifiers from real paths plus
  contributor-specific fragments. The path is available at production emission
  time, but no typed location is retained separately.

### Designer XML production evidence

- Designer metadata and module descriptors retain exact source artifact paths.
  Their graph producer builds opaque IDs containing the path, SHA-256 evidence,
  and a fact fragment.
- Designer Procedures and Functions are extracted with one-based lines. The
  producer appends `;declaration=<id>;line=<line>` to the opaque provenance ID,
  so the information exists but remains untyped and inseparable from identity
  evidence without an unsupported parser.
- Designer XML does not currently emit static Query nodes. That absence must not
  be interpreted as a navigable Query capability.

### Path and coordinate implications

- Current producer paths may be absolute because production builders receive
  discovered filesystem roots. Returning an opaque provenance source would
  disclose absolute paths, hashes, raw identifiers, and contributor context.
- The Runtime owns each configuration root and is the first layer that can
  combine typed producer locations with Workspace confinement. The extension
  knows only the configured process workspace root.
- A usable public location therefore needs a slash-normalized relative path
  anchored by an explicit accepted root. The Runtime must reject absolute,
  parent, empty, malformed, or non-contained projections before MCP output;
  TypeScript must revalidate the returned relative path before URI creation.
- Existing declarations prove a one-based line but no declaration column or end
  range. A first slice can truthfully navigate to the first character of that
  line. It cannot claim an exact identifier selection until a producer supplies
  an exact UTF-16-compatible range.
- ADR-0053 must choose the Common coordinate convention and the wire conversion
  explicitly. Mixing BSL one-based lines, Query UTF-8 byte offsets, and VS Code
  zero-based UTF-16 characters implicitly is unsafe.

## Searchable and navigable first-slice candidates

Every graph node has a stable ID, exact canonical name, and closed kind. The
existing query index exposes all nodes in deterministic ID order and exact-name
lookup only. It provides a safe source-independent input set for a bounded
search projection, but no accepted substring or user-facing order.

The smallest coherent producer-complete first slice is:

| Node family | EDT location evidence | Designer location evidence | Candidate behavior |
|---|---|---|---|
| Module | Exact file path, no range | Exact file path, no range | Searchable and file-navigable |
| Procedure | Exact module path and one-based declaration line before emission | Exact module path and one-based line before opaque encoding | Searchable and line-navigable |
| Function | Exact module path and one-based declaration line before emission | Exact module path and one-based line before opaque encoding | Searchable and line-navigable |
| Query | Exact EDT module path and one-based declaration line | Not emitted | Searchable/navigable for EDT only if ADR-0053 includes its producer migration |
| Metadata and other semantic nodes | Real source artifact path exists in producer-specific evidence; line generally absent | Real source artifact path exists; line generally absent | Defer unless Task 1 inventory proves one bounded uniform producer migration |

Restricting the first slice to Module, Procedure, Function, and optionally EDT
Query avoids pretending that every semantic node has one canonical source file.
It also provides positive, missing-location, cross-adapter, duplicate-name, and
unsupported-kind oracles from current tracked fixtures.

The following search rules are decision-ready candidates for ADR-0053:

- one optional Configuration ID or deterministic aggregation across all
  configurations, with each result retaining Configuration identity and name;
- one UTF-8 query bounded at 256 bytes, with empty input either rejected or
  explicitly defined as a bounded alphabetic list;
- Unicode lowercase comparison without normalization, matching canonical name
  by substring; no locale, regex, glob, fuzzy score, transliteration, synonym,
  qualified-name invention, or source-content match;
- deterministic order by lowercase canonical name, exact canonical name, node
  kind, node ID, then Configuration ID, independent of insertion and discovery
  order;
- exact duplicate identity retained once, equal display names retained as
  distinct results, and ambiguity presented to the user rather than resolved by
  first match; and
- default 50 and maximum 100 results with total and `truncated`, using checked
  counting and no cursor claim.

ADR-0053 must accept or reject these values; this investigation does not make
them stable behavior.

## MCP and Tool Policy candidates

Extending `oneagent.query` is structurally awkward because its accepted schema
requires `nodeId` and its operations all begin from one exact node. Making
`nodeId` conditional would change the existing schema and validation precedence.
A dedicated read-only `oneagent.symbols` tool is the smallest independently
testable candidate:

- one closed input object for query, optional Configuration selection, optional
  accepted symbol-kind filter, and bounded limit;
- deterministic results containing Configuration identity/name, node ID/name/
  kind, workspace-relative source path, optional one-based source position or
  span, total, and `truncated`;
- one read-only Tool Policy rule under the existing actor and revision;
- existing `invalid_arguments`, `not_found`, `policy_denied`,
  `execution_failed`, and `result_too_large` mapping unless ADR-0053 proves a
  new stable category is necessary; and
- unchanged MCP revision, JSON-RPC validation, frame/depth bounds, sequential
  dispatch, stdio framing, EOF, and channel ownership.

This adds a seventh catalog member and requires the VS Code connection contract
to expect the new exact catalog only after Runtime and public process evidence
is committed. Existing six tool inputs/results remain unchanged. No supported
external client exists yet, so external-client compatibility remains deferred;
the additive catalog still requires an explicit ADR-0051 compatibility update.

The Runtime needs an accepted source of the process workspace root to emit a
workspace-relative path. Viable candidates are:

1. retain the configured root in `WorkspaceSnapshot` and make it part of the
   snapshot's source-location contract;
2. add a source-navigation server constructor that accepts the already observed
   process root while preserving the existing `semantic_server(snapshot)` API;
3. return configuration-root-relative paths plus a separately confined
   workspace-relative configuration root.

Returning an absolute path, asking TypeScript to parse provenance, deriving a
root from a source string, or reading the filesystem during a query is rejected.
ADR-0053 must select one of the three bounded owners after auditing existing
Workspace/cache consumers.

## VS Code experience candidates

The accepted first slice can add one contributed command,
`oneagent.searchSymbols`, that auto-activates on explicit user demand and
requires the existing client to be `connected`. It must not connect, restart,
or rebuild automatically.

One `createQuickPick` flow is preferable to a preloaded `showQuickPick` list
when the server owns semantic filtering:

1. create and own one single-select Quick Pick;
2. on a bounded input change, cancel the prior semantic request and issue at
   most one accepted sequential request;
3. replace items only with the latest matching response, preserving server
   order and exposing Configuration/kind/path as bounded presentation fields;
4. keep equal labels as distinct items and open only the selected exact result;
5. on accept, hide and dispose the Quick Pick, validate the relative path again,
   open the file URI under the selected workspace root, convert accepted source
   coordinates to VS Code coordinates, set selection, and reveal it; and
6. on hide, cancellation, protocol/process failure, deactivation, or command
   repetition, cancel pending work and dispose every invocation-owned resource.

The current MCP client allows one pending request but exposes no general call
or caller cancellation. ADR-0053 must define whether a cancelled Quick Pick
drops only the local result or terminates the connection; MCP cancellation
notifications remain unsupported. A safe first slice can ignore a late response
after local cancellation while retaining sequential request ownership and
without issuing a second request until the first completes.

No provider API is needed. Definition, reference, document-symbol, and
workspace-symbol providers are reserved for Sprint 32's editor-neutral LSP
boundary and must remain absent in Sprint 31.

## Deterministic evidence matrix

| Layer | Required observable cases |
|---|---|
| Common/Graph | Empty/absolute/parent/malformed path rejection; slash normalization; one-based position/span bounds; missing span; equality/order/deduplication; existing provenance and graph regressions |
| BSL/producers | EDT and Designer Module/Procedure/Function; optional EDT Query; exact one-based line; Unicode and LF/CRLF; missing/non-navigable family; duplicate names; repeated-build identity/location equality |
| Workspace | Single/multiple/empty configurations; root retention or constructor ownership; nested roots; path containment and escape; snapshot/cache compatibility; reordered discovery |
| MCP/Policy | Exact seventh catalog if accepted; schema/handler/policy agreement; query/kind/configuration/limit bounds; empty/no-result/duplicate/ambiguous/truncated results; malformed/unknown/extra fields; absolute/escape rejection; redaction; repeated calls |
| Public process | Real tracked EDT and Designer workspaces; list/call results; exact relative paths/lines; existing six-tool regression; malformed/oversized input; channel purity; EOF; repeated fresh process |
| Pure extension | Result validation; stale/late response; cancellation; order preservation; path containment; coordinate conversion; missing file; protocol/process failures; repetition and cleanup |
| Extension Host | Demand activation; command registration; connected/disconnected gates; Quick Pick input/items/accept/hide; actual document open, selection, reveal; no duplicate resources; deactivation cleanup |
| Package/CI | Exact manifest/package inventory; no production dependency; pinned Node/editor; macOS/Windows Rust and extension gates; deferred provider/LSP absence |

Tracked Runtime workspace fixtures already contain both EDT and Designer roots,
duplicate-style semantic names, real module sources, and BSL declarations.
Synthetic tests can cover missing location, path escape, maximum result count,
duplicate canonical names, Unicode comparison, reordered insertion, and
multi-configuration ambiguity. No external source corpus is required.

## Executed baseline evidence

The investigation executed these non-zero baselines from planning HEAD:

- `cargo test -p oneagent-common`: 2 unit tests passed; 0 failed/ignored;
- `cargo test -p oneagent-bsl`: 37 unit tests passed; 0 failed/ignored;
- `cargo test -p oneagent-graph --test query`: 19 integration tests passed;
- `cargo test -p oneagent-runtime --test mcp_semantic_tools`: 4 integration
  tests passed;
- locked Node `24.19.0` extension typecheck and compile: passed;
- extension unit tests: 27 passed; 0 failed/skipped/todo;
- real public `oneagent-mcp` process tests: 2 passed; and
- pinned VS Code `1.134.0` Extension Host: 14 tests passed across package,
  repeat, empty, virtual, multi-root, and untrusted labels.

The ordinary shell initially lacked `node`, so the first pnpm typecheck failed
before executing TypeScript. The configured bundled Node `24.19.0` then ran the
same locked commands successfully without install or dependency change. The
first sandboxed Extension Host attempt ended five labels with `SIGABRT`; the
required unsandboxed rerun passed all 14 tests. Successful package-host runs
still printed the known exit-zero `Unexpected SIGPIPE` diagnostic recorded by
the Sprint 30 review. It remains a non-blocking environment observation, not a
navigation correctness oracle.

## Rejected investigation candidates

- Parsing current source `EntityId` strings is rejected because formats differ
  by producer, delimiters can occur in paths/context, and the identifiers also
  contain hashes and semantic evidence.
- Returning full provenance or absolute paths is rejected because it expands
  the sensitive-data and compatibility surface beyond navigation needs.
- Client-side search over one truncated graph dump is rejected because it is
  incomplete, moves semantic matching to TypeScript, and cannot enforce a
  stable server-side result bound.
- Reusing `oneagent.query` with a fake `nodeId` or conditionally optional fields
  is rejected as an obscure compatibility and validation contract.
- Local filesystem search is rejected because source discovery and semantic
  ownership already belong to Workspace/adapters and the MCP snapshot is
  immutable after startup.
- VS Code language providers are rejected because Sprint 32 owns the
  editor-neutral LSP boundary and provider compatibility.
- An MCP SDK, path library, fuzzy-search library, or UI dependency is rejected;
  current Rust/Node/VS Code APIs are sufficient.
- Automatic Runtime connection, concurrent requests, debouncing timers without
  an accepted clock, and MCP cancellation notifications are rejected from the
  smallest first slice.

## Remaining ADR-0053 decisions

ADR-0053 must select:

- exact Common source path/position/span types, UTF-8/path bounds, coordinate
  basis, half-open or point behavior, and validation/error precedence;
- whether structured location extends `Provenance`, `GraphNode`, or another
  accepted graph fact boundary, plus constructor and compatibility migration;
- exact EDT/Designer producer families, including whether EDT Query belongs to
  the first slice;
- Workspace-root ownership and the exact relative path returned to clients;
- searchable kinds, query bound, empty-query policy, case/Unicode behavior,
  ordering, kind filtering, limits, truncation, duplicate/ambiguity behavior;
- the exact `oneagent.symbols` schema or a better evidence-backed alternative,
  Tool Policy rule, result fields, errors, redaction, and ADR-0051 compatibility;
- Runtime client call/cancellation behavior and Quick Pick request sequencing;
- command ID/title, connected-state gate, presentation fields, selection/reveal
  behavior, missing-file/error UX, ownership, and cleanup; and
- the exact focused/public/cross-platform acceptance matrix and documentation
  transitions.

Every alternative has repository-owned source evidence and a deterministic
oracle. No missing-data or external-dependency blocker remains.

## Deferred scope

Exact identifier-column ranges not supplied by current producers; arbitrary
metadata/member navigation; source contents/fragments; declaration/reference
search from an editor cursor; reference, definition, document-symbol, or
workspace-symbol providers; LSP; fuzzy/relevance scoring; aliases/synonyms;
workspace reload/watch changes; mutable/cursor pagination; concurrent MCP and
protocol cancellation; remote/web/multi-root; automatic Runtime install or
connection; diagnostics; chat/context UI; EDT plugin integration; external
clients; Marketplace work; telemetry; edits/refactoring; and broad performance
or security claims remain deferred.
