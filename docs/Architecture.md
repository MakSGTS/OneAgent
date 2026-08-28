# Architecture

OneAgent uses a modular Rust workspace centered on a source-independent semantic
graph. The architecture distinguishes the current implementation from planned
product adapters so that roadmap intent is not mistaken for available behavior.

## Current implementation

1. **Shared and domain crates**
   - `oneagent-common` owns shared typed primitives.
   - `oneagent-metadata` owns the typed 1C metadata model.
   - `oneagent-workspace` owns workspace and project abstractions.
   - `oneagent-bsl` owns BSL lexical and syntax analysis.
   - `oneagent-llm` owns the std-only provider-neutral identity, model,
     capability, request, response, policy, cancellation, error, and
     asynchronous provider contracts.
   - `oneagent-tool-policy` owns the independent std-only bounded tool request,
     declared-effect, fail-closed authorization, exact confirmation,
     cancellation-aware execution-gate, terminal-result, and redacted-audit
     contracts.
2. **Semantic core**
   - `oneagent-graph` owns canonical semantic nodes, edges, provenance,
     validation, query, diff, impact, coverage, and resolution APIs.
   - `oneagent-analysis` contributes source-independent declaration and call
     analysis over the BSL and graph contracts. It also owns the additive
     source-independent Context Engine that derives deterministic budgeted
     semantic bundles from one borrowed immutable graph.
3. **Source adapters**
   - `oneagent-edt` reads supported EDT artifacts and contributes facts to the
     canonical semantic graph.
   - `oneagent-designer-xml` reads accepted hierarchical Designer XML artifacts
     and contributes the same source-independent graph kinds without replacing
     EDT semantics.
   - `oneagent-workspace-fs` discovers supported workspaces through the
     filesystem boundary.
   - `oneagent-openai-compatible` implements the bounded concrete ADR-0046
     provider adapter without changing provider-neutral or Runtime ownership.
   - `oneagent-lm-studio` implements the bounded concrete ADR-0047 leaf through
     native type-aware discovery and private composition over the unchanged
     OpenAI-compatible generation operation.
   - `oneagent-ollama` implements the bounded concrete ADR-0048 local-only leaf
     through native capability-safe Tags/Show discovery and raw generation.
4. **Applications and protocol foundation**
   - `oneagent-runtime` exposes the long-running composition root as a reusable
     library. It owns ordered service startup, rollback, task handles,
     per-service cancellation, reverse shutdown, lifecycle, and terminal error
     propagation. Its Runtime-owned Workspace service performs configured
     production discovery/build, observes subsequent file changes, serializes
     complete rebuilds, publishes separate immutable per-configuration semantic
     snapshots, retains the last valid publication across failed rebuilds, and
     clears the snapshot during owned shutdown. It also owns one fixed
     Workspace-local complete-snapshot cache, exact validity checks, safe
     replacement, and typed cache observation without making persisted bytes a
     semantic authority. Public status observers report rebuild phase, attempts,
     publications, failures, and the latest cache load/write outcomes. Its sole
     Axum service exposes HTTP liveness and
     lifecycle-derived readiness probes plus the versioned read-only Graph
     Query route set. A transport-neutral observer-backed query component owns
     exact configuration, node, direct-relation, and bounded-traversal
     operations without becoming a background service. A separate bounded MCP
     stdio adapter and `oneagent-mcp` binary build one immutable startup
     snapshot and serve the seven read-only semantic tools without constructing
     `App`, watching files, or changing Runtime service ownership. A separate
     bounded LSP stdio adapter and `oneagent-lsp` binary expose immutable
     workspace symbols and pull diagnostics through Content-Length framing and
     the accepted LSP lifecycle without changing Graph authority.
   - `oneagent-cli` is the supported dependency-free client for the accepted
     Runtime health and Graph Query HTTP/1.1 surface. It owns a closed command
     grammar, local validation, bounded one-request socket lifecycle, exact
     query encoding, opaque Runtime JSON presentation, and stable failure/exit
     classification without becoming protocol or semantic authority.
   - `oneagent-protocol` owns bounded stateless MCP 2026-07-28,
     connection-local legacy MCP 2025-06-18 and 2025-11-25, and LSP 3.17
     request, notification, response, error, codec, validation, lifecycle,
     capability, and dispatch contracts. It does not own Runtime I/O, semantic
     projection, HTTP, or editor UI.
   - `extensions/vscode` is the desktop workspace client for the accepted MCP
     process boundary. It exposes four contributed commands and one non-default
     chat participant,
     validates one trusted file-backed workspace and one bounded executable,
     owns one directly spawned `oneagent-mcp` child, and derives one status bar
     item from its closed connection state. Its explicit symbol-search command
     presents Runtime order and opens only revalidated Workspace-relative source
     locations. Its explicit Context command projects a canonical symbol through
     the existing Runtime tool into one read-only panel and a bounded
     request-selected model exchange. Its editor adapter does not own MCP
     semantics, Runtime installation, workspace discovery, provider secrets, or
     graph behavior.
   - `extensions/edt` is the bounded native Eclipse/1C:EDT compatibility client.
     It recognizes one selected local configuration project through public
     Eclipse APIs and an exact nature string, validates one executable setting,
     and owns one background `server/discover` probe against a fresh
     `oneagent-mcp` child. It packages one JavaSE-17 bundle in one feature and p2
     repository without proprietary EDT imports, semantic behavior, a bundled
     Runtime/JRE, credentials, or a second protocol owner.

`SemanticGraph` is the canonical semantic authority. Adapters may observe source
formats and contribute provenance-backed facts, but source-specific identities
and parser state must not become competing graph truth. Derived facilities such
as query, resolution, reports, diffs, impact analysis, and the Sprint 4 Semantic
Index remain read-only views over graph snapshots. Context selection and
assembly are another read-only derived view and do not become graph authority.

## Planned boundaries

The roadmap assigns future boundaries explicitly:

- Graph-query Runtime APIs are implemented in Sprint 18, and Sprint 19 File
  Watching and Sprint 20 Persistent Cache are completed with `pass` integration
  reviews. Sprint 21 CLI Client is also completed with a `pass` integration
  review. The [v0.4 release review](reviews/v0.4-release-review.md) records
  `pass`. The [Sprint 22 Context Engine review](reviews/sprint-22-context-engine.md)
  also records `pass`. The later
  [Sprint 26 Ollama Integration review](reviews/sprint-26-ollama-integration.md)
  records `pass`; the later
  [Sprint 27 Tool Execution Policy review](reviews/sprint-27-tool-execution-policy.md)
  also records `pass`. The
  [v0.5 AI Integration release review](reviews/v0.5-release-review.md) records
  `pass`. The
  [Sprint 28 MCP Server review](reviews/sprint-28-mcp-server.md) records
  `pass with non-blocking follow-ups`. The
  [Sprint 29 MCP Semantic Tools review](reviews/sprint-29-mcp-semantic-tools.md)
  records `pass with non-blocking follow-up`. The
  [Sprint 30 VS Code Extension Foundation review](reviews/sprint-30-vscode-extension-foundation.md)
  records `pass with non-blocking follow-ups`. Sprint 31 Navigation and Symbol
  Search, Sprint 32 LSP Adapter, and Sprint 33 AI Chat and Context Panel are
  completed. The Sprint 34 EDT Integration Prototype review records `pass`,
  Sprint 35 External AI Client Compatibility is completed, and the
  [v0.6 MCP and IDE release review](reviews/v0.6-release-review.md) records
  `pass with non-blocking follow-ups`.
- Semantic MCP tools are implemented in Sprint 29, the bounded desktop VS Code
  connection foundation is implemented in Sprint 30, and typed source locations
  plus bounded symbol search and navigation are implemented in Sprint 31. The
  editor-neutral LSP workspace-symbol and pull-diagnostic slice is implemented
  in Sprint 32. Explicit semantic Context inspection and bounded request-selected
  chat are implemented in Sprint 33. The bounded native EDT compatibility probe
  is implemented in Sprint 34. External Codex and Cursor compatibility is
  implemented in Sprint 35. Definition/reference providers, diagnostics UI,
  model tools or edits, and semantic EDT IDE workflows remain later work.
- Git change ingestion arrives in Sprint 38 as an input adapter, not a semantic
  authority.

Detailed accepted decisions live in `docs/adr`. The dependency-ordered delivery
sequence and status live only in `docs/Roadmap.md`.

## Accepted Context Engine boundary

[ADR-0044](adr/0044-context-engine.md) governs the implemented Sprint 22 first
slice. One additive stateless `ContextEngine` call accepts a validated
`ContextRequest`, borrows exactly one immutable `SemanticGraph`, and returns an
owned `ContextBundle` or one closed typed error. `oneagent-analysis` owns this
derived semantic view; `oneagent-graph` remains the sole authority for facts,
identity, kinds, provenance, indexes, and query behavior.

The request supports only `Explain`, exact node-ID and exact canonical-name
seeds, a closed direction/edge/node filter policy, depth `0..=4`, candidate
limit `1..=128`, and a rendered UTF-8 byte budget `1..=65_536`. Selection is
cycle-safe and deterministic across graph insertion and seed order. It retains
one best provenance-backed path per node using path depth, explicit edge
priority, outgoing-before-incoming direction, stable edge identity, and seed
identity. Final candidate order uses stable candidate identity as its last
tie-breaker.

Assembly first requires every seed fragment to fit, then admits related items
as a whole-fragment prefix. Every item has an exact two-line length-prefixed
semantic rendering and checked byte cost. Candidate-limit and budget omissions
remain distinct and exact; no partial fragment or uncounted overhead exists.
The first slice reads no source text, invokes no tokenizer/provider/model,
mutates or persists no graph state, and exposes no Runtime, HTTP, CLI, protocol,
MCP, or IDE surface.

### Sprint 22 public evidence matrix

The public `crates/analysis/tests/context_engine.rs` target imports only the
exported `oneagent-analysis`, common, and graph library surfaces. Its checked-in
Rust graphs and production `SemanticAnalysisPipeline` inputs require no
filesystem corpus, network, service, clock, or arbitrary ordering oracle.

| Contract | Public evidence |
| --- | --- |
| Request and resolution | Exact validation precedence, defaults and limits, both seed variants, deduplication, missing, ambiguous, incompatible, and unique-seed limit outcomes are asserted through public types. |
| Selection | All eleven edge kinds, three directions, depth zero and maximum, node/edge filters, cycles, alternative seeds, stable ties, candidate omission, and reordered/repeated equality are covered. |
| Provenance and explanations | Reordered duplicate node/edge provenance canonicalizes without graph mutation; selected seed, depth, direction, edge kind/ID, path provenance, and typed reason remain observable. |
| Budget and rendering | ASCII/non-ASCII byte lengths, exact and one-byte-short seed budgets, related prefix admission, separate omission counts, exact fragments, and bundle accounting use fixed string oracles. |
| Production compatibility | The existing production analysis pipeline supplies declaration, containment, call, and provenance facts directly to equal fresh Context evaluations; rendered output contains no fabricated BSL source text. |

The [Sprint 22 integration review](reviews/sprint-22-context-engine.md) records
`pass` after the focused and complete workspace gates. Sprint 22 is completed;
the [Sprint 26 integration review](reviews/sprint-26-ollama-integration.md)
records `pass`; the later
[Sprint 27 Tool Execution Policy review](reviews/sprint-27-tool-execution-policy.md)
also records `pass`. Sprint 28 is completed with `pass with non-blocking
follow-ups`; Sprints 29–35 and the v0.6 MCP and IDE boundary are completed with
a non-blocking [release decision](reviews/v0.6-release-review.md).

## Accepted LLM Provider abstraction boundary

[ADR-0045](adr/0045-llm-provider-abstraction.md) governs the implemented Sprint
23 provider-neutral first slice. The additive std-only `oneagent-llm` crate owns
bounded provider/model identity, canonical provider-scoped model catalogs, the
single closed `TextGeneration` capability, owned bounded text requests and
terminal responses, local UTF-8 byte usage, closed finish and error kinds,
redacted secrets/diagnostics, represented timeout with `RetryPolicy::Never`,
receiver-only cooperative cancellation, and an object-safe standard-library
future/provider seam.

Request construction validates exact text and output-byte bounds before model
compatibility and retains neither provider configuration nor the full model
descriptor. Response construction is bound to the originating request and
cannot supply arbitrary identity or usage. Provider implementations return one
owned catalog or terminal response, perform at most one attempt, and observe
cancellation cooperatively; the shared crate owns no executor, clock, retry
loop, task, transport, cache, or global registry.

### Sprint 23 public evidence matrix

The public `crates/llm/tests/provider_contract.rs` target imports only the
exported `oneagent-llm` surface. It uses standard-library deterministic fakes,
explicit state and wakers, synthetic sentinels, and exact values without
network, filesystem, environment, credentials, sleeps, or developer-local
services.

| Contract | Public evidence |
| --- | --- |
| Identity, capability, and discovery | Exact identity and catalog bounds, provider scoping, duplicate capability canonicalization, empty discovery, reordered models, duplicates, mismatches, and repeated equality are asserted. |
| Request and response | Exact UTF-8 byte bounds and precedence, preserved whitespace/Unicode, capability rejection, reordered equivalence, request-bound output, local usage, both finish reasons, and empty/over-bound failures are asserted. |
| Secrets and errors | Secret and diagnostic bounds, the complete closed error taxonomy, retry classification, and sentinel absence from implicit formatting prove the accepted redaction boundary. |
| Execution policy | Optional timeout bounds, `RetryPolicy::Never`, and exactly one maximum attempt prove representation without a hidden clock, delay, or replay. |
| Provider substitution and cleanup | Independent providers work through `&dyn LlmProvider`; canonical/empty discovery, repeated generation, typed failures, provider mismatch, cancellation before/during work, and zero surviving active state are exact oracles. |
| Compatibility | `oneagent-analysis` and `oneagent-runtime` remain unchanged and independently validated; neither depends on `oneagent-llm`, and Context text receives no prompt semantics. |

The [Sprint 26 integration review](reviews/sprint-26-ollama-integration.md)
records `pass`. The later
[Sprint 27 Tool Execution Policy review](reviews/sprint-27-tool-execution-policy.md)
also records `pass`. Sprint 28 is completed with `pass with non-blocking
follow-ups`; Sprints 29–35 and the v0.6 MCP and IDE boundary are completed with
a non-blocking [release decision](reviews/v0.6-release-review.md).

## Implemented Tool Execution Policy boundary

[ADR-0049](adr/0049-tool-execution-policy.md) governs the implemented
provider- and Runtime-independent `oneagent-tool-policy` first slice. The
additive std-only crate owns bounded request, actor, tool, policy-revision, and
argument values; a closed conservative effect vocabulary; canonical immutable
rules; and request-wide fail-closed authorization. Any matching deny wins,
unmatched declared effects deny by default, required confirmation wins over
allow, and only complete allow coverage permits unconfirmed execution.

A `RequireConfirmation` authorization can issue one non-cloneable challenge.
Accepted evidence is privately bound to the exact policy revision, request ID,
actor, tool, canonical effects, and argument bytes. The public execution gate
consumes authorization and optional confirmation, rejects denial or missing,
mismatched, stale, and unexpected confirmation before executor construction,
observes pre-existing and in-flight cancellation, and invokes an object-safe
substitutable executor at most once. Cancellation wins a simultaneously ready
executor outcome. Completed, partial, failed, executor-reported timeout, and
cancelled paths become one owned terminal result; its audit record retains only
safe identities, canonical effects, byte counts, authorization and confirmation
states, zero-or-one attempt count, and terminal classification.

### Sprint 27 public evidence matrix

The public `crates/tool-policy/tests/conformance.rs` target imports only the
exported `oneagent-tool-policy` surface. Standard-library fakes use counters,
explicit cancellation, and drop guards without filesystem, shell, Git,
network, environment, credential, clock, privileged, destructive, or external
state.

| Contract | Public evidence |
| --- | --- |
| Construction and policy | UTF-8 byte bounds, contradictory effects, canonical duplicate rules, global deny, default deny, stable reasons, and sensitive argument formatting are asserted. |
| Confirmation and denial | One challenge per required authorization, stale revision and changed-request mismatch, missing or denied paths, zero attempts, and zero executor calls are exact oracles. |
| Accepted execution | Allow and exact confirmation produce one attempt; no retry or fallback exists, and repeated fresh calls have equal safe observations. |
| Terminal outcomes | Completed, partial, failed with bounded diagnostic, executor-reported timeout, pre-cancelled, and in-flight-cancelled results retain the closed outcome and output-presence matrix. |
| Audit and redaction | Every specified correlation field, canonical field/effect order, argument/output byte counts, and sentinel absence from implicit formatting are checked. |
| Cleanup | Drop guards prove no active fake work remains after completion, failure, timeout, cancellation, or repeated calls. |

The first slice owns no concrete tool, side effect, policy persistence or
configuration source, authentication or confirmation UX, clock or timeout
enforcement, retry/fallback, rollback, audit sink/export, Runtime lifecycle or
registration, transport, MCP/provider/IDE mapping, sandbox, or cross-process
replay prevention. The
[Sprint 27 integration review](reviews/sprint-27-tool-execution-policy.md)
records `pass`. Sprint 28 is completed with `pass with non-blocking follow-ups`;
Sprints 29–35 and the v0.6 MCP and IDE boundary are completed with a
non-blocking [release decision](reviews/v0.6-release-review.md).

## Implemented MCP Server boundary

[ADR-0050](adr/0050-mcp-server.md) governs the discovery and transport
foundation, [ADR-0051](adr/0051-mcp-semantic-tools.md) governs the additive
semantic-tool slice, and
[ADR-0057](adr/0057-external-ai-client-compatibility.md) governs negotiated
legacy compatibility. `oneagent-protocol` owns bounded request IDs, method
names, request and notification metadata, closed responses and errors,
newline-payload codec validation, exact discovery data, truthful tool
capability/catalog definitions, `tools/list`, `tools/call`, asynchronous
sequential dispatch, and one connection-local state machine for revisions
`2025-06-18` and `2025-11-25`. Existing stateless revision `2026-07-28` and its
`server/discover` response remain unchanged. Pagination remains absent.

`oneagent-runtime` owns one injected sequential asynchronous stdio adapter. Each
`run` creates exactly one fresh negotiated session borrowing the immutable
server. The adapter accepts LF and CRLF framing, enforces a 1 MiB payload limit
and bounded JSON nesting, emits no response for notifications, flushes every
response, maps controlled read/write/flush/shutdown failures to stable
categories, and treats cancellation and EOF as successful completion. The
separate `oneagent-mcp` binary constructs one immutable Workspace snapshot from
its working directory before reading stdin and creates no Runtime `App`,
watcher, cache, listener, or background task. Stdout contains protocol frames
only, EOF exits with status zero, and startup or terminal failures use stable
redacted stderr categories.

The exact lexicographic catalog is `oneagent.context`,
`oneagent.diagnostics`, `oneagent.graph`, `oneagent.impact`, `oneagent.query`,
`oneagent.symbols`, and `oneagent.validation`. All definitions declare
read-only, non-destructive annotations. Runtime projects canonical Workspace,
Graph, Impact, Validation, Diagnostics, Query, Context, and source-location
owners into bounded JSON and sends every known call through the fail-closed
Tool Policy gate. The original six results remain path-free;
`oneagent.symbols` exposes only confined Workspace-relative forward-slash paths
and one-based locations for Module, Procedure, Function, and EDT Query nodes.
Results contain equivalent compact JSON text and structured content. Known-tool
semantic failures set `isError`; malformed or unknown calls remain protocol
`Invalid params` failures. No tool mutates files, graphs, processes, network
state, or other external state.

Public protocol, semantic-library, fixture, adapter, and real-process tests
cover exact discovery, catalog order, annotations and schemas, validation
bounds, all seven tool families, Tool Policy execution, path redaction and
symbol-path confinement, malformed and oversized input, unknown methods/tools,
notifications, LF/CRLF
framing, cancellation, transport failures, EOF, stdout purity, exit status,
repetition, and cleanup. Exact Codex CLI `0.150.0-alpha.8` and Cursor Agent
`2026.08.25-3e8eec8` public-client results plus the complete synthetic matrix
are recorded in the
[Sprint 35 evidence](architecture/external-ai-client-compatibility-evidence.md).
Codex directly calls all seven tools and observes semantic success and domain
failure. Cursor's public `mcp list-tools` command proves all seven definitions;
that client version exposes no non-interactive direct-call command, so no
Cursor call result is claimed. Additional revisions and clients, remote
transports, authentication, snapshot refresh, Runtime packaging, references,
diagnostics UI, and broader IDE integration remain deferred. Sprint 31 is
completed; the additive LSP boundary is described below.

## Implemented LSP adapter boundary

[ADR-0054](adr/0054-lsp-adapter.md) governs the additive editor-neutral LSP
3.17 slice. `oneagent-protocol` owns bounded duplicate-aware JSON-RPC decoding,
request IDs, lifecycle state, method validation, closed errors, truthful static
capabilities, and transport-independent dispatch. `oneagent-runtime` owns the
8,192-byte Content-Length header bound, 1 MiB UTF-8 body bound, injected
sequential stdio adapter, canonical startup root URI, confined document URIs,
semantic handlers, process channels, terminal classification, and the public
`oneagent-lsp` binary. The process owns one immutable startup snapshot and
creates no Runtime `App`, watcher, cache, listener, queue, or background task.

Initialize advertises UTF-16 positions, `textDocumentSync: 0`,
`workspaceSymbolProvider`, and a pull-only `diagnosticProvider` with no
workspace diagnostics. `workspace/symbol` projects only Procedure, Function,
and EDT Query nodes with one distinct confined typed span. It applies the
accepted Unicode-lowercase substring match, deterministic identity order, LSP
kinds 12/19, zero-based ranges, and a complete-result limit of 100 without
silent truncation. `textDocument/diagnostic` returns full reports without a
result ID. It projects only existing recoverable Graph diagnostics whose source
node has one matching confined span, preserves Graph code, severity, message,
and stable order, and returns an empty full report for a valid document without
projected evidence.

Content-Length stdout is protocol-only and flushed per response. Notifications
are silent; successful process completion requires `shutdown` followed by
`exit`; EOF before `exit`, malformed framing, I/O/encoding failure,
cancellation, or early exit is a bounded redacted failure. Existing MCP
revision/framing, seven tools, Tool Policy, HTTP, CLI, Workspace, cache, Graph,
adapters, and the VS Code MCP experience remain unchanged. Mutable documents,
source reads after startup, definition/references/completion/edits,
push/workspace diagnostics, dynamic registration, remote transport, multi-root,
external-client claims, and IDE migration remain deferred.

## Implemented VS Code AI chat and Context panel boundary

[ADR-0055](adr/0055-ai-chat-context-panel.md) governs the extension-only Sprint
33 slice. `OneAgent: Inspect Semantic Context` requires the existing explicit
connection, presents canonical Runtime symbols, and calls `oneagent.context`
with exact symbol identity plus fixed `both`, depth 2, 32-candidate, and
16,384-byte inputs. One generic FIFO serializes symbol and Context calls over
the single-pending-request transport. The controller retains one immutable
generation only while its matching panel remains live.

`oneagent.contextPanel` renders that generation as fully escaped, static,
script-free, form-free, command-free, resource-free HTML under a strict content
security policy. Closing or replacing the panel, disconnecting or replacing
the Runtime, or deactivating the extension invalidates the generation and makes
it unavailable to chat.

The non-default `oneagent.chat` participant sends exactly two user messages to
the model selected by the current VS Code request: the visible rendered Context
and the current 1–8,192-byte prompt. Admission enforces a 32,768-byte assembled
message bound and the selected model token budget. Response handling consumes
text fragments only, renders them as escaped untrusted Markdown text, caps raw
output at 65,536 bytes, and owns cancellation plus one concurrent request. The
extension retains no conversation history, hidden Context, provider identity,
credential, or source bytes. Model tools and edits, source inference, automatic
Context collection, Runtime provider wiring, persistence, webview scripts,
remote/web/multi-root operation and diagnostics UI remain deferred.

## Implemented EDT integration prototype boundary

[ADR-0056](adr/0056-edt-integration-prototype.md) governs the additive Sprint 34
prototype. The `extensions/edt` Tycho 5.0.2 reactor builds against the frozen
public Eclipse 2023-12 target with Maven 3.9.16 on JDK 25 and compiles the
production bundle for Java 17. The bundle imports only public Eclipse and OSGi
packages, exports no package, and treats the exact EDT configuration nature as
data. Authenticated 1C p2 access and the installed p2 pool are not production or
CI inputs.

The public handler accepts exactly one selected open, accessible, non-linked,
non-virtual local project with that nature. The instance preference supplies one
bounded bare executable token or absolute executable path. One generation-owned
Eclipse Job launches that executable directly with the project directory as its
working directory, sends the exact newline-framed MCP 2026-07-28
`server/discover` request, accepts only the closed OneAgent 0.1.0 compatibility
response, and publishes one fixed success or redacted failure on the UI thread.
Every path owns and bounds the child, streams, readers, stderr, deadline,
cancellation, job, and stale UI callback; preference replacement, cancellation,
bundle stop, and host shutdown join or suppress all owned work.

The distributable p2 repository contains exactly one feature and one production
bundle under the `OneAgent` category. It contains no test fragment, fixture,
Runtime, JRE, JavaFX, native executable, credential, private-p2 metadata, or
personal path. Repository-owned macOS and Windows CI builds the real Runtime,
runs the complete 41-test Tycho/PDE/real-process matrix, and audits the exact p2
inventory without an ITS secret. Authorized local x86_64 EDT 2026.1 evidence on
JDK 17 and matching OpenJFX proves positive, repeated, invalid-configuration,
timeout, cancellation, stop, install, uninstall, and clean-host outcomes while
leaving application bundles, signatures, and the read-only p2 pool unchanged.

This boundary proves host compatibility only. Java does not read EDT sources,
call semantic tools, duplicate Workspace or protocol authority, or alter the
existing seven-tool MCP catalog, LSP process, VS Code client, Runtime services,
Graph, adapters, or Coverage. Semantic EDT UI, navigation, Context/chat,
diagnostics, edits, proprietary EDT services, remote/multi-project support,
automatic Runtime lifecycle, publication, signing, telemetry, and bundled
toolchains remain deferred.

## Accepted OpenAI-compatible provider boundary

[ADR-0046](adr/0046-openai-compatible-provider.md) governs the implemented
concrete `oneagent-openai-compatible` leaf adapter. Construction consumes one
exact `openai-compatible` provider configuration, one explicit HTTP/HTTPS
server-root URL, and an optional bearer secret. Reqwest defaults are disabled;
redirects and implicit proxies are disabled; Rust TLS uses platform roots.

Each discovery call performs one bounded `GET /v1/models`, strictly maps the
list IDs to a canonical text-capable `ModelCatalog`, and retains no provider
metadata. Each generation call performs one non-streaming
`POST /v1/completions` with only `model`, `prompt`, `max_tokens`, and
`stream=false`, requires exact response model identity and one index-zero
choice, maps only `stop` and `length`, and constructs local request-bound byte
usage. Successful bodies are incrementally bounded to 1 MiB for discovery and
512 KiB for completion. Status, transport, protocol, response, timeout, and
cancellation outcomes are typed and redacted; no operation retries, redirects,
falls back, caches, or leaves background work.

The public `adapters/openai-compatible/tests/conformance.rs` target uses only
exported adapter and `oneagent-llm` APIs plus deterministic loopback servers. It
proves explicit construction/redaction, exact authenticated wires, canonical
discovery, both terminal mappings, local usage, fallback/malformed/bound/status/
redirect rejection, timeout/cancellation precedence, one attempt, cleanup, and
fresh repetition without live services or credentials. Runtime composition,
configuration sources, Context prompt semantics, chat/Responses APIs,
streaming, tools, provider usage authority, and additional providers remain
deferred except for the implemented LM Studio and Ollama specializations
described below.
The Sprint 24 integration review records `pass with non-blocking follow-ups`;
the Sprint 25 and Sprint 26 integration reviews record `pass`. Sprint 26 is
completed. Sprint 27 Tool Execution Policy is completed with a `pass` review.
The [v0.5 release review](reviews/v0.5-release-review.md) records `pass` for the
complete AI Integration boundary.
Sprint 28 is completed with `pass with non-blocking follow-ups`; the Sprint 29
implementation is present and remains the unique `next` target pending
integration review.

## Accepted LM Studio provider boundary

[ADR-0047](adr/0047-lm-studio-integration.md) governs the implemented
`oneagent-lm-studio` leaf adapter. Construction consumes one exact `lm-studio`
provider configuration, one explicit server-root URL or the numeric-loopback
`http://127.0.0.1:1234` default, and an optional bearer secret. Its private
native client performs one fresh bounded `GET /api/v1/models`, projects only
loaded `llm` instance IDs into the provider-neutral catalog, ignores embedding
and unloaded entries, and rejects unknown types or invalid, duplicate, and
over-count catalogs atomically.

Generation privately translates one validated LM Studio request into an exact
temporary `openai-compatible` request and delegates once to the unchanged
ADR-0046 `/v1/completions` operation. The successful response is rebound to the
original `lm-studio` request identity with local UTF-8 byte usage and only the
accepted `Completed` or `OutputLimit` finish. Both clients disable redirects
and implicit proxies; operations do not retry, fall back, cache, manage models,
or retain background work.

The public `adapters/lm-studio/tests/conformance.rs` target uses only exported
adapter and `oneagent-llm` APIs through `&dyn LlmProvider` plus deterministic
controlled-loopback servers. It proves explicit construction and redaction,
mixed LLM/embedding projection, exact authenticated native/generic wires,
canonical and repeated discovery, both finish mappings, identity and local
usage, malformed/fallback/bound/status/redirect rejection, transport, timeout,
cancellation, one-attempt behavior, and cleanup without installed LM Studio,
downloaded models, credentials, or external network. Runtime registration and
configuration sources, live-provider acceptance, server/model lifecycle,
chat/template quality, streaming, tools, MCP, and IDE integration remain
deferred. The Sprint 25 and
[Sprint 26](reviews/sprint-26-ollama-integration.md) integration reviews record
`pass`; Sprint 27 Tool Execution Policy is completed with a `pass` review.
Sprint 28 is completed with `pass with non-blocking follow-ups`; the Sprint 29
implementation is present and remains the unique `next` target pending
integration review.

## Accepted Ollama provider boundary

[ADR-0048](adr/0048-ollama-integration.md) governs the implemented
`oneagent-ollama` leaf adapter. Construction consumes one exact `ollama`
provider configuration without a credential and either an explicit
numeric-loopback HTTP root or the fixed `http://127.0.0.1:11434` default.
Remote roots, DNS names, HTTPS, authentication, redirects, and implicit proxies
are rejected or disabled.

Each fresh discovery performs one bounded native `GET /api/tags`, validates
exact equal model identities and remote markers, excludes well-formed
remote-backed entries before provider-specific inspection, and sends sequential
canonical `POST /api/show` requests only for local candidates. Exact lowercase
`completion` is the sole evidence projected as `TextGeneration`; malformed,
duplicate, invalid, over-count, ambiguous, or later-failing catalogs are
rejected atomically.

Generation performs one bounded non-streaming raw `POST /api/generate` with the
exact validated model and prompt, `stream=false`, `raw=true`, `think=false`, and
`options.num_predict` equal to the request output-byte bound. It accepts only an
exact response model, `done=true`, absent or empty thinking, and `stop` or
`length`, then constructs local request-bound UTF-8 usage and the corresponding
`Completed` or `OutputLimit` finish. Operations share one total timeout and
cooperative cancellation race and never retry, redirect, fall back, cache,
manage models, or retain background work.

The public `adapters/ollama/tests/conformance.rs` target uses only exported
adapter and `oneagent-llm` APIs through `&dyn LlmProvider` plus deterministic
numeric-loopback servers. It proves construction/redaction, exact Tags, Show,
and Generate wires, remote exclusion, capability projection, identity and both
finish mappings, malformed/ambiguous/bound/status/redirect failures, transport,
timeout, cancellation, one-attempt repetition, and cleanup without installed
Ollama, local or cloud models, credentials, or external network. Runtime
registration/configuration, live-provider compatibility, daemon/model
lifecycle, cloud/authentication, chat, templates, streaming, tools, MCP, and IDE
integration remain deferred. The
[Sprint 26 integration review](reviews/sprint-26-ollama-integration.md) records
`pass`; Sprint 27 Tool Execution Policy is completed with a `pass` review.
Sprint 28 is completed with `pass with non-blocking follow-ups`; the Sprint 29
implementation is present and remains the unique `next` target pending
integration review.

## Accepted Runtime service-container boundary

[ADR-0037](adr/0037-runtime-service-container.md) governs the implemented Sprint
15 boundary. `oneagent-runtime` remains the composition root and exposes a
transport-independent library boundary.
`AppBuilder` owns ordered, uniquely named service registration; `App` owns the
built container and lifecycle; the running container owns every service task
handle and per-service cancellation source until all handles terminate.

Services start sequentially in registration order and acknowledge startup by
returning their owned task. Partial startup rolls acknowledged services back in
reverse order. A requested shutdown, unexpected exit, service error, or task
join failure triggers reverse cooperative cancellation and complete joining;
the application reaches `Stopped` before returning its terminal result. The
first slice has no detached tasks, global registry, new dependency, or bounded
shutdown timeout.

The public Runtime lifecycle and deterministic in-memory service probes remain
the ownership foundation for the HTTP adapter; workspace, graph, watcher,
persistence, and CLI services remain Sprints 17-21.

## Accepted HTTP and health boundary

[ADR-0038](adr/0038-http-api-health.md) governs the implemented Sprint 16 HTTP
slice. One Runtime-owned Axum service binds during service startup, exposes only
`GET /health/live` and `GET /health/ready`, derives readiness exclusively from
the canonical Runtime lifecycle, and completes through ADR-0037 cancellation
and task ownership. The default address is `127.0.0.1:3000`; callers can supply
a typed override, including port zero, and observe the actual bound address
without controlling the listener.

Liveness returns `200` with `{"status":"alive"}` while the handler is
reachable. Readiness returns `200` with `{"status":"ready"}` only during
`Running`, and `503` with `{"status":"not_ready"}` during observable
`Initializing` and `Stopping` states. Only GET is supported; registered wrong
methods return `405` with `Allow: GET`, and unknown exact paths return `404`.
The listener binds before startup acknowledgement, bind errors remain named
service-start failures, and graceful shutdown releases the listener only after
the Runtime-owned HTTP task completes.

## Accepted Graph Query API boundary

[ADR-0040](adr/0040-graph-query-api.md) governs the implemented Sprint 18
boundary. Production composition constructs one Workspace observer, injects it
into one transport-neutral `GraphQueryService`, and gives that component to the
existing `HttpService`; `http` still starts before `workspace`, and
`HttpService::new()` remains a health-only compatible construction path.

The query-enabled listener registers exactly four GET routes:

- `/api/v1/configurations` lists separate published configurations;
- `/api/v1/graph/node` selects one exact node in one exact configuration;
- `/api/v1/graph/relations` returns direct incoming or outgoing edges with an
  optional one-kind filter;
- `/api/v1/graph/traverse` performs deterministic breadth-first traversal with
  mandatory depth and result bounds.

Each request observes one immutable Workspace snapshot. The HTTP adapter first
validates the closed query syntax and values, then requires canonical Runtime
readiness, and finally delegates to the transport-neutral component. Results
use owned payload-free projections, limits default to 50 and cannot exceed 100,
traversal depth cannot exceed 4, and truncation is explicit. Exact stable JSON
errors distinguish lifecycle, snapshot, selection, identifier, syntax,
vocabulary, boolean, and bound failures without exposing internal diagnostics.
Health routes remain the sole liveness/readiness authority and retain their
Sprint 16 wire contract.

### Sprint 18 public evidence matrix

The public `apps/runtime/tests/graph_query_api.rs` target uses raw Tokio
loopback HTTP and the tracked Sprint 17 provenance fixture through production
filesystem discovery and both production builders.

| Contract | Public evidence |
| --- | --- |
| Separate production graphs | Configuration listing preserves exact Designer XML and EDT identities, formats, counts, and canonical order; node queries select facts from each graph without merging. |
| Four accepted operations | Exact node, outgoing/incoming direct relation, filtered relation, empty relation, bounded traversal, included start, and empty depth-zero results are asserted through public HTTP. |
| Bounds and closed errors | Defaults, truncation, unknown configuration/node, invalid identifier/query/encoding, unsupported direction/edge kind, limit/depth bounds, invalid boolean, and unavailable snapshot map to exact status/code/message rows. |
| Route compatibility | Every registered route is GET-only; HEAD/POST return `405` with `Allow: GET`; unknown and trailing-slash paths retain empty `404`; JSON is returned independently of `Accept`. |
| Lifecycle authority | Published snapshots remain query-inaccessible during gated `Initializing` and `Stopping`, become available only in `Running`, and absent snapshots are distinct from lifecycle readiness. |
| Ownership and determinism | Two fresh production runs return equal wire observations, clear snapshot/address watches, join all owned work, release the listener, and permit immediate rebind. |

## Accepted Workspace service boundary

[ADR-0039](adr/0039-workspace-service.md) governs the implemented Sprint 17
initial-build slice. `RuntimeConfig` owns one Workspace root, and production
composition starts HTTP before one uniquely named `workspace` service. That
service moves the configured path and complete snapshot builder into exactly one
owned blocking task, runs filesystem discovery once, dispatches EDT and
Designer XML builds sequentially, validates every graph, rejects unsupported or
colliding configurations, and publishes only one complete immutable snapshot.

The snapshot keeps configurations as separate graphs ordered by canonical
Configuration identity. Each record preserves its detected root and format,
exact Configuration name and ID, canonical graph, diagnostics, reference
ledger/statistics, and report. A valid empty root publishes an empty snapshot;
any discovery, adapter, validation, cardinality, duplicate-identity, or blocking
task failure publishes nothing and becomes a named Workspace startup failure.
Cancellation clears the snapshot before the owned service task returns. Runtime
readiness remains derived only from lifecycle, so snapshot presence is not an
independently mutable health label.

### Sprint 17 public evidence matrix

The public `apps/runtime/tests/workspace_service.rs` target uses only production
discovery/build paths and public Runtime observation. Its bounded tracked EDT
and Designer inputs have an explicit provenance and SHA-256 inventory.

| Contract | Public evidence |
| --- | --- |
| Both production formats | One mixed root builds exact EDT and complete Designer graphs, preserves their distinct evidence, and orders them by Configuration ID. |
| Determinism and fresh ownership | Repeated fresh applications publish equal observations and close every snapshot sender after `App::run`. |
| Empty and invalid roots | Empty readable roots publish an empty snapshot; missing and non-directory roots return named startup failures without publication. |
| Discovery and atomic failure | Conflicting markers, duplicate Configuration identity, and a later fatal adapter input reject the entire snapshot. |
| Readiness authority | With a deterministic later startup/cleanup gate, real health requests remain not-ready in `Initializing` and `Stopping`, become ready only in `Running`, and retain the Sprint 16 wire vocabulary. |
| Shutdown cleanup | Reverse cancellation keeps the complete snapshot available until the Workspace service is reached, then clears it and closes observation before terminal `Stopped`. |

## Accepted File Watching boundary

[ADR-0041](adr/0041-file-watching.md) governs the implemented Sprint 19
boundary. After the startup build, one Runtime-owned source recursively scans
the configured Workspace root every 250 milliseconds using normalized relative
paths, entry kinds, and complete regular-file bytes. Descendants of `.git`,
`.idea`, `.vscode`, `target`, and `node_modules` are excluded; source extensions
are not filtered. The source emits only the latest opaque revision through a
private single-value channel.

The Workspace service remains the sole rebuild owner. It closes the startup
scan/build race with before/build/after scans, serializes complete rebuilds,
coalesces changes that arrive during a build, and atomically replaces the
published `Arc` only after a valid all-or-nothing build. A post-start observation
or semantic-build failure retains the last valid snapshot and becomes public
update status instead of terminating Runtime; a later change can recover.
Health/readiness and Graph Query wire contracts remain unchanged. Shutdown
cancels and joins the change source and any in-flight build, prevents a
post-cancellation publication, clears the snapshot, and publishes terminal
`Stopped` update status.

### Sprint 19 production and deterministic evidence matrix

The public `apps/runtime/tests/file_watching.rs` target imports only the
`oneagent_runtime` library surface, copies the tracked Sprint 17 fixture into
fresh temporary roots, uses production polling/discovery/build paths, and
queries the existing Graph Query API over raw Tokio loopback HTTP. Event watches
are the asserted synchronization mechanism; five-second timeouts are hang
guards rather than timing evidence. Negative ignored-change and exact active-
build concurrency assertions use the focused controlled-tick/gated-builder
tests because ADR-0041 explicitly forbids treating the production polling period
as a test oracle; those tests retain the real scanner or complete builder as the
authority.

| Contract | Evidence |
| --- | --- |
| Both production formats | Exact EDT and Designer XML name changes trigger complete production rebuilds and become visible in separate snapshots and Graph Query responses. |
| Atomic immutable replacement | A held pre-change `Arc` remains unchanged while later observations receive one valid replacement; Graph Query requests observe only complete published snapshots. |
| Add/remove/rename-equivalent changes | Moving a Designer root outside the watched Workspace and back under a different root name proves removal and addition detection without a native rename event contract. |
| Relevance and ignored state | Focused real-filesystem scans prove complete bytes, paths, entry kinds, and all five ignored-directory exclusions; a controlled production-service scan proves an ignored mutation leaves public update status unchanged. |
| Burst and in-flight coalescing | Public status proves a mutation accepted after `Rebuilding` causes exactly one follow-up publication and a multi-entry project-tree addition causes one attempt; the focused gated builder proves one active build and one bounded latest-state follow-up. |
| Failure retention and recovery | Corrupt EDT input reports a semantic-build failure while the last valid snapshot and query result remain available; a later repair publishes a recovered snapshot. |
| Observation failure and readiness | Removing the watched root reports `Observation`, retains the queryable snapshot and exact ready health response, and publishes one recovered rebuild when the root returns. |
| Status and ownership | Public update status proves attempts, publications, phases, failure classification, recovery, terminal `Stopped`, closed snapshot/update receivers, listener release, and equal fresh-run observations. |

## Accepted Persistent Cache boundary

[ADR-0042](adr/0042-persistent-cache.md) governs the implemented Sprint 20
baseline without changing `SemanticGraph`, source adapters, Runtime lifecycle,
health, or Graph Query authority. `WorkspaceService` owns source observation,
cache orchestration, complete clean builds, immutable publication, cancellation,
and cleanup. The cache is a private versioned representation of one complete
validated `WorkspaceSnapshot`; decoded content is reconstructed through checked
domain APIs and passes complete build validation before it can be published.

The fixed entry is `.oneagent/cache/workspace-v1.json` under the configured
Workspace root, with one bounded temporary replacement file. Exact complete
source state plus explicit schema and semantic-build versions determine validity.
Startup performs scan/load/scan before accepting a hit, otherwise runs one clean
build, closes the build race with a final scan, and writes only stable state.
File Watching rebuilds use the same pre/build/post stability rule and finish
cache work before atomically publishing a valid replacement. Cache-owned paths
are excluded from source observation, so replacement cannot create a watcher
feedback loop.

Missing, changed, incompatible, corrupt, or unavailable entries clean-build
instead of becoming semantic authority. Failed writes and unstable-source skips
do not reject a valid snapshot. Public cloneable cache observation exposes only
the closed latest load and write outcomes; it does not add HTTP, CLI, readiness,
or protocol state. Shutdown joins current blocking cache/build work, closes cache
observation with the other Workspace observers, and preserves the complete cache
entry for a fresh process.

### Sprint 20 public evidence matrix

The public `apps/runtime/tests/persistent_cache.rs` target imports only the
`oneagent_runtime` library surface, copies the tracked Sprint 17 mixed EDT and
Designer XML provenance fixture into disposable roots, and exercises production
source scans, cache storage, both clean builders, Workspace/File Watching,
Graph Query, health, cancellation, and shutdown. Watches are synchronization;
five-second timeouts are hang guards, not polling-duration evidence.

| Contract | Public evidence |
| --- | --- |
| Cold and warm completeness | A cold missing entry clean-builds both production formats and writes once; a fresh exact hit performs no write and restores equal graphs, payloads, provenance, diagnostics, reference evidence, statistics, reports, transport-neutral queries, and HTTP results. |
| Identity and compatibility | Complete source changes produce `SourceChanged`; older and newer schema and semantic-build versions produce `Incompatible`; every case clean-builds and replaces current state. |
| Corruption containment | Malformed, truncated, partial, checksum-invalid, and checksum-valid semantically invalid entries produce `Corrupt`, publish no persisted partial state, and recover through equal complete clean builds. |
| Storage failure and repair | A publicly constructible wrong-kind cache owner produces `Unavailable`/`Failed` while the valid Workspace remains ready and queryable; removing the obstacle permits missing/write recovery followed by a warm hit. |
| Watched replacement and reuse | Production EDT and Designer changes publish complete immutable replacements, preserve held older snapshots, replace cache bytes, ignore cache-owned probe state in the source identity, and restore the latest replacement on a fresh hit. |
| Lifecycle and cleanup | Cache work completes before `Running` or replacement publication; health and Graph Query contracts remain exact; shutdown clears snapshots, publishes terminal update state, closes snapshot/update/cache watches, releases listeners, leaves no temporary file, and preserves only reusable complete cache state. |

## Accepted CLI Client boundary

[ADR-0043](adr/0043-cli-client.md) governs the implemented Sprint 21 client
without changing Runtime or semantic authority. `oneagent-cli` exposes exact
health, configurations, node, relations, and traverse commands over one numeric
`SocketAddr`, validates the closed local grammar and server bounds, emits exact
percent-encoded GET targets, and owns one blocking `TcpStream` per invocation.

The dependency-free HTTP/1.1 executor uses fixed connection/read/write timeouts,
bounded response head/body sizes, connection-close framing, exact JSON media
validation, and no retry, pooling, redirect, DNS, TLS, or background task. It
passes complete Runtime JSON through without interpretation, writes success to
stdout and server errors to stderr, and distinguishes usage, transport, server,
protocol, and output failure through stable exit codes. `oneagent-protocol`
remains inactive.

### Sprint 21 public evidence matrix

The public `apps/cli/tests/runtime_client.rs` target invokes the built CLI
executable and a real query-enabled Runtime over port-zero loopback. It copies
the tracked mixed EDT/Designer fixture into fresh temporary roots, gates Runtime
startup through public lifecycle seams, and uses timeouts only as hang guards.

| Contract | Public evidence |
| --- | --- |
| Executable boundary | Exact help, version, usage, transport, and malformed-protocol paths prove stdout/stderr and exits through the real binary. |
| Lifecycle and health | CLI liveness succeeds during `Initializing`; readiness and Graph Query return exact Runtime `503` JSON before `Running`; readiness succeeds after complete startup. |
| Both production formats | Configuration listing preserves canonical Designer/EDT order, and exact node commands return both Configuration identities and names. |
| Graph operations | Direct outgoing `contains` relations and bounded traversal with edge filter, include-start, depth, and limit preserve Runtime JSON and order. |
| Bounds and domain failures | Limit `1` reports truncation; an unknown Configuration preserves the exact Runtime error envelope and server exit. |
| Cleanup and repetition | Shutdown releases the Runtime listener, every child process and connection terminates, and two fresh mixed-fixture runs return equal observations. |

### Sprint 16 public evidence matrix

The public `apps/runtime/tests/http_health.rs` target imports only the
`oneagent_runtime` library surface and uses raw Tokio loopback TCP. Lifecycle
watches and one-shot channels define asserted events; one-second timeouts are
hang guards rather than timing evidence.

| Contract | Public evidence |
| --- | --- |
| Lifecycle-derived readiness | Real requests return not-ready during gated `Initializing`, ready during `Running`, and not-ready during gated reverse cleanup in `Stopping`. |
| Stable probe wire format | Liveness and readiness assert exact status, JSON media type, and closed single-field bodies. |
| Exact negative matrix | HEAD and POST on both routes return `405`, `Allow: GET`, and empty bodies; unknown and trailing-slash paths return `404` with empty bodies. |
| Startup failure | An occupied loopback address becomes named `ServiceStartFailed` for `http`, with no published address and terminal `Stopped`. |
| Graceful shutdown and ownership | Requested shutdown retains the HTTP service until earlier reverse cleanup completes, then joins it, clears address observation, and permits rebind. |
| Fresh repetition | Two separately built port-zero apps return equal wire responses and independently release every listener. |

### Sprint 15 public evidence matrix

The public `apps/runtime/tests/service_container.rs` target imports only the
`oneagent_runtime` library surface. Its deterministic in-memory probes use
channels as acknowledgements and timeouts only as hang guards.

| Contract | Public evidence |
| --- | --- |
| Genuinely long-running execution | The App remains pending after ordered startup until injected shutdown is released. |
| Requested shutdown | Services observe receiver-only cancellation and terminate in reverse registration order before `Stopped`. |
| Partial startup failure | A later named start error rolls the earlier acknowledged task back and closes every probe sender. |
| Running-service failure | The named error reaches the App caller after reverse sibling cleanup. |
| Unexpected exit and join panic | Early `Ok` and task panic retain distinct `RuntimeErrorKind` classifications. |
| Shutdown-source error | The source failure remains primary while the worker is cancelled and joined. |
| Fresh repetition and no detached work | Two separately built apps produce equal start/stop behavior; event-channel closure proves no probe task survives `App::run`. |

The [Sprint 15 integration review](reviews/sprint-15-runtime-service-container.md)
records `pass` after the focused and complete workspace gates. Sprint 15 is
completed. The [Sprint 16 integration review](reviews/sprint-16-http-api-health.md)
records `pass` for the owned HTTP and public health/readiness boundary; Sprint
17 implementation and public production evidence are completed with a `pass`
decision in the
[Sprint 17 integration review](reviews/sprint-17-workspace-service.md). Sprint
18 Graph Query API is completed with a `pass` decision in the
[Sprint 18 integration review](reviews/sprint-18-graph-query-api.md). Sprint 19
File Watching is completed with a `pass` decision in the
[Sprint 19 integration review](reviews/sprint-19-file-watching.md). Sprint 20
Persistent Cache is completed with a `pass` decision in the
[Sprint 20 integration review](reviews/sprint-20-persistent-cache.md). Sprint 21
CLI Client is completed with a `pass` decision in the
[Sprint 21 integration review](reviews/sprint-21-cli-client.md). The
[v0.4 release integration review](reviews/v0.4-release-review.md) also records
`pass` and completes the Runtime API boundary. The subsequent
[Sprint 22 Context Engine review](reviews/sprint-22-context-engine.md) also
records `pass`. The later
[Sprint 26 Ollama Integration review](reviews/sprint-26-ollama-integration.md)
records `pass`; Sprint 27 Tool Execution Policy is completed with a `pass`
review. Sprint 28 is completed with `pass with non-blocking follow-ups`; the
later Sprints 29–35 and the v0.6 MCP and IDE boundary are completed with a
non-blocking [release decision](reviews/v0.6-release-review.md).
