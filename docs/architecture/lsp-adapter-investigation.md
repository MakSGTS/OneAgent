# LSP Adapter Investigation

This investigation records the committed repository and pinned upstream
evidence available before ADR-0054. It does not accept an LSP architecture or
implement production behavior.

## Confirmed repository baseline

- Planning HEAD is `49b7d02b`, `Plan Sprint 32 LSP Adapter`. Sprint 31 is
  completed by `daab0ecf`; Sprint 32 is the unique active target after this
  investigation starts.
- `oneagent-protocol` owns bounded JSON-RPC/MCP parsing, duplicate-key and depth
  validation, deterministic dispatch, compact encoding, and no I/O or semantic
  dependency. It has no LSP module, lifecycle state, LSP capability model, or
  Content-Length framing.
- `oneagent-runtime` owns Tokio stream/process composition and one immutable
  `WorkspaceSnapshot` built from the process working directory. The public
  `oneagent-mcp` process uses LF framing and a stateless MCP lifecycle that must
  not be reused as LSP framing or initialization behavior.
- `WorkspaceSnapshot` retains the normalized startup root. Each Configuration
  snapshot retains its root, canonical identity/name, graph, ordered recoverable
  diagnostics, requests, statistics, and report.
- ADR-0053 and current production graphs provide typed locations for Module,
  Procedure, Function, and EDT Query nodes. Runtime already performs lexical
  confinement under both Configuration and Workspace roots and projects only
  workspace-relative UTF-8 paths through `oneagent.symbols`.
- `SemanticDiagnostic` owns stable code, severity, kind, message, optional
  source node, candidates, and provenance. It has no direct canonical
  `SourceLocation`. The tracked mixed Runtime fixture has one EDT missing-query-
  source diagnostic whose source node is a located Query; the Designer fixture
  has no diagnostics.
- No `oneagent-lsp` binary, LSP test, editor-neutral client, mutable-document
  store, document synchronization owner, or supported external LSP client exists.

## Pinned protocol authority

The selected candidate authority is the official Language Server Protocol 3.17
specification at Microsoft repository commit
`8be2e191506ced923953b94b985c4a1831757b39`, retrieved on 2026-08-27 from the
official `gh-pages` branch. The field-level meta-model and source documents are
immutable at that full commit:

- [LSP 3.17 specification source](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/specification.md);
- [LSP 3.17 meta-model](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/metaModel/metaModel.json);
- [initialize](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/general/initialize.md);
- [shutdown](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_includes/messages/3.17/shutdown.md),
  [exit](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_includes/messages/3.17/exit.md);
- [workspace symbols](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/workspace/symbol.md);
- [pull diagnostics](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/language/pullDiagnostics.md);
- [positions and encoding](https://raw.githubusercontent.com/microsoft/language-server-protocol/8be2e191506ced923953b94b985c4a1831757b39/_specifications/lsp/3.17/types/position.md).

Confirmed upstream facts are:

- stdio messages use an ASCII header and required `Content-Length`, separated
  from the UTF-8 JSON body by `\r\n\r\n`; this is incompatible with MCP LF
  framing even though both payloads use JSON-RPC 2.0;
- `initialize` is the first request and occurs once; pre-initialize requests use
  `ServerNotInitialized` `-32002`, other pre-initialize notifications are
  dropped except `exit`;
- `shutdown` returns `null` without exiting; `exit` then selects success only
  after shutdown and failure otherwise;
- LSP positions are zero-based and negotiate character encoding. UTF-16 is the
  default and must be supported; current accepted declaration points are always
  column 1 and therefore convert to character 0 identically in UTF-8/16/32;
- `workspace/symbol` accepts an arbitrary query including empty text and returns
  `SymbolInformation[]`, `WorkspaceSymbol[]`, or `null`; its base result has no
  truncation marker;
- document pull diagnostics are a 3.17 capability, can return a full or
  unchanged report, and declare whether inter-file dependencies and workspace
  diagnostics are supported; and
- document synchronization is a separate capability. Advertising no sync is
  possible but makes an immutable/read-only document contract explicit.

## Ownership and compatibility inventory

| Boundary | Current owner | LSP implication |
|---|---|---|
| JSON values and duplicate/depth checks | `oneagent-protocol` | LSP can add a sibling module without changing MCP types or method catalog. |
| Content-Length stream framing | none | Runtime should own Tokio reads/writes; protocol should own decoded body validation/encoding. |
| Workspace/root identity | `WorkspaceSnapshot` | The process cwd and initialize root URI need one explicit compatibility check. |
| Semantic symbols and locations | Graph/adapters/Runtime | LSP must project existing canonical node/location evidence and cannot parse source or provenance. |
| Recoverable diagnostics | Graph/adapters/Workspace | LSP may project only diagnostics with one unambiguous located source node. |
| MCP symbols/diagnostics | protocol/Runtime/Tool Policy | LSP is an independent adapter and must not change the seven-tool catalog or bypass/borrow Tool Policy semantics. |
| VS Code search UX | TypeScript extension over MCP | Sprint 32 need not migrate or register VS Code language providers to prove an editor-neutral public boundary. |

Existing serde `1.0.228`, serde_json `1.0.150`, Tokio `1.53.0`, path handling,
and Runtime test utilities are sufficient candidates. No new Cargo or Node
production dependency is required.

## Decision-ready lifecycle and wire candidates

ADR-0054 must select exact values, but the smallest coherent candidate is:

- one dedicated `oneagent-lsp` Content-Length-framed stdio process with a
  complete immutable snapshot built from cwd before serving messages;
- one 1 MiB body bound aligned with the existing protocol process boundary and
  a separate small bounded ASCII header block; exactly one required decimal
  `Content-Length`, optional specification-compatible `Content-Type`, no unknown
  or duplicate header ambiguity, and no stdout diagnostic bytes;
- string or integer request IDs within existing deterministic bounds, closed
  JSON-RPC errors, one sequential request at a time, notification suppression,
  and no outgoing server requests;
- lifecycle states before initialize, awaiting `initialized`, running, shutdown,
  and exited, with exact pre/post-shutdown behavior and immediate terminal EOF;
- exactly one file-backed Workspace root whose normalized initialize `rootUri`
  matches the startup root; absent, non-file, malformed, multiple, or conflicting
  roots fail initialization without exposing a local path; and
- static capabilities, no dynamic registration, no document synchronization,
  no post-start filesystem reads, no background task, and no process restart.

The process must not reuse MCP framing, session assumptions, tool catalog,
request metadata, Tool Policy, or the existing TypeScript MCP client.

## Method feasibility matrix

| LSP surface | Repository evidence | Decision readiness |
|---|---|---|
| `workspace/symbol` | Four accepted node kinds, canonical names/IDs, confined locations, deterministic matching/order, mixed EDT/Designer fixture. | Ready. Empty query and result-over-bound behavior require an explicit ADR rule because LSP has no truncation flag. |
| `workspaceSymbol/resolve` | Every accepted result can already carry a complete location. | Defer; no lazy field is needed. |
| `textDocument/definition` | References/calls do not retain one accepted typed cursor occurrence range; declaration points alone cannot resolve arbitrary use-site positions. | Not implementable truthfully in this sprint. |
| `textDocument/documentSymbol` | Module has no range and current declaration spans are points rather than full symbol ranges/hierarchy extents. | Defer rather than invent ranges. |
| `textDocument/diagnostic` | Recoverable diagnostics and one located EDT source node exist; the immutable server can filter by confined document URI. | Ready for a bounded full-report-only slice with no result ID or workspace diagnostic claim. |
| publish/workspace diagnostics | No client document lifecycle, push scheduler, refresh owner, or complete workspace diagnostic location coverage exists. | Defer. |
| completion, hover, references, rename, edits and other language features | No accepted method-specific semantic/source contract. | Defer. |

The bounded first-slice candidate therefore advertises only
`workspaceSymbolProvider` and `diagnosticProvider`, plus `textDocumentSync: 0`
and an explicit position encoding. Workspace-symbol results provide navigation
through their confined locations; Sprint 32 does not claim go-to-definition.

## Symbol projection candidates

- Map Module to LSP `Module`, Procedure to `Function`, Function to `Function`,
  and Query to a conservative accepted kind selected by ADR-0054.
- Reuse ADR-0053 Unicode-lowercase substring matching and deterministic tuple
  ordering. Unlike MCP, LSP admits an empty query, which can mean all accepted
  symbols subject to the same order.
- Use complete `WorkspaceSymbol` values with file URI plus zero-based point
  range. Module locations without spans may use the 3.17 location-without-range
  form only when the client advertises `resolveSupport`; otherwise omitting
  Modules is safer than inventing a range. ADR-0054 must choose a single
  capability-dependent behavior.
- Because the base response cannot signal truncation, silently returning a
  prefix is unsafe. Candidate policies are a conservative stable maximum with
  `RequestFailed` when the complete set exceeds it, or a complete response
  bounded only by the protocol body limit. ADR-0054 must select and test one.
- Convert the already one-based half-open span to zero-based LSP positions. The
  accepted point spans need no source-text read and are encoding-independent at
  column 1. No exact identifier-column claim is added.

## Diagnostic projection candidates

For one requested confined file URI, Runtime can find diagnostics whose
`source_node` exists in the same Configuration graph and whose node provenance
has exactly one distinct confined typed location. It can then emit the stable
code, mapped Error/Warning severity, fixed source `oneagent`, current message,
and the node's zero-based point range. Missing source node, missing/conflicting
location, cross-root location, or unsupported file is omitted rather than
guessed.

The tracked mixed fixture proves a positive EDT `ReferenceUnresolved`
diagnostic on a located Query source node and an empty Designer result. Existing
query/reference fixtures cover malformed, missing, ambiguous, incompatible,
duplicate, repeated, and reordered diagnostics. The first slice can always
return `kind: "full"` without `resultId`, related documents, tags, data, code
descriptions, workspace diagnostics, or refresh notifications. This avoids
claiming unchanged-result cache identity or mutable synced-document analysis.

ADR-0054 must decide whether an unknown but confined file returns an empty full
report or `InvalidParams`, and must fix URI equality, code value, diagnostic
range, stable ordering, maximum count/body behavior, and message redaction.

## Deterministic evidence matrix

| Layer | Required oracle |
|---|---|
| Protocol | Positive initialize/initialized/shutdown/exit; request IDs; duplicate keys; invalid JSON/params; unknown/pre-init/post-shutdown methods; reordered and repeated bodies; exact/over depth/body/header bounds. |
| Framing/process | Fragmented/coalesced headers and bodies; CRLF; EOF at every boundary; invalid UTF-8; channel purity; shutdown/exit codes; repeated fresh processes; no orphan. |
| Roots/URIs | Exact cwd root, percent encoding, separator normalization, non-file URI, mismatch, escape, multi-root/conflict, Unicode path, and Windows drive behavior in CI. |
| Symbols | Four kinds across EDT/Designer, empty/non-empty/case/Unicode/whitespace queries, duplicates, missing/conflicting locations, ordering, complete/over-bound result, and repeated requests. |
| Diagnostics | Positive located EDT diagnostic, empty Designer/file, missing source node/location, ambiguous/conflicting location, stable code/severity/message/order, repeated/reordered build, and MCP regression. |
| Compatibility | Existing MCP protocol/stdio/process/semantic tools, Runtime HTTP/CLI/Workspace/cache, Graph/adapters, VS Code extension build/tests, Cargo dependency/lockfile, and public docs. |

## Executed baseline evidence

Before this document was written:

- `cargo test -p oneagent-protocol` passed 28 non-doc tests; its doc-test target
  contained zero tests;
- `cargo test -p oneagent-graph diagnostic_order_is_deterministic` passed the
  one matching Graph test; unrelated integration binaries matched zero and are
  not counted as evidence;
- `cargo test -p oneagent-runtime --test mcp_process public_mcp_process_serves_every_semantic_tool_family_repeatably`
  passed one matching public process test;
- initial guessed EDT and Designer filters matched zero tests and were rejected
  as evidence;
- `cargo test -p oneagent-edt attaches_provenance_to_edt_graph_facts` then
  passed the one matching production location test; and
- `cargo test -p oneagent-designer-xml public_builder_emits_only_accepted_graph_slice`
  passed the one matching production location test.

These checks prove the prerequisite baseline only. They do not prove LSP
behavior.

## Decision readiness

Repository and pinned upstream evidence are sufficient for ADR-0054. The ADR
must explicitly settle framing/header/body bounds, lifecycle state/error
precedence, root URI and position encoding, complete workspace-symbol behavior,
Module-without-range handling, result-over-bound failure, diagnostic URI/range/
omission behavior, and exact public process matrix before implementation.

No hard external-data blocker or production dependency approval is required.
