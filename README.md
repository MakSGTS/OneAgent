# OneAgent

AI Development Platform for 1C:Enterprise.

OneAgent has completed the v0.3 source-independent 1C Knowledge Model and the
v0.4 Runtime API boundary with `pass` release integration reviews. v0.4
includes the long-running Runtime, HTTP health, Workspace lifecycle, Graph Query
API, File Watching, Persistent Cache, and supported CLI Client delivered by
Sprints 15–21. The source-independent Context Engine delivered by Sprint 22 is
also complete with a `pass` integration review. The provider-neutral Sprint 23
LLM Provider Abstraction is complete with a `pass` integration review. The
bounded Sprint 24 OpenAI-Compatible Provider is complete with a
`pass with non-blocking follow-ups` integration review. The bounded Sprint 25
LM Studio Integration is complete with a `pass` integration review. The
bounded Sprint 26 Ollama Integration is complete with a `pass` integration
review. The bounded Sprint 27 Tool Execution Policy is complete with a `pass`
integration review, and the
[v0.5 AI Integration release review](docs/reviews/v0.5-release-review.md)
records `pass`. The bounded Sprint 28 discovery-only MCP Server is complete
with a `pass with non-blocking follow-ups` integration review. Sprint 29 MCP
Semantic Tools, Sprint 30 VS Code Extension Foundation, and Sprint 31 Navigation
and Symbol Search are complete. Sprint 32 LSP Adapter is also complete. The
Sprint 33 AI Chat and Context Panel and Sprint 34 EDT Integration Prototype are
complete. Sprint 35 External AI Client Compatibility is active with
implementation and client evidence complete, pending integration review.
See
[`docs/Roadmap.md`](docs/Roadmap.md) for canonical execution order.

## Workspace

- `apps/runtime` — long-running Runtime composition, owned service lifecycle, cancellation, shutdown, EDT/Designer Workspace discovery and file-change rebuilds, validated persistent snapshot caching, immutable semantic snapshots, public update/cache observation, HTTP liveness/readiness, the versioned read-only Graph Query API, and the separate bounded `oneagent-mcp` and `oneagent-lsp` stdio processes
- `apps/cli` — supported dependency-free CLI client for Runtime health, Workspace configuration listing, exact node lookup, direct relations, and bounded traversal
- `crates/common` — shared primitives
- `crates/workspace` — project and workspace model
- `crates/metadata` — typed 1C metadata model
- `crates/graph` — canonical semantic graph, query, validation, diff, impact, coverage, and resolution APIs
- `crates/bsl` — BSL lexical and syntax analysis
- `crates/analysis` — source-independent declaration/call analysis and deterministic semantic Context Engine
- `crates/llm` — provider-neutral bounded identity, model discovery, text request/response, policy, cancellation, error, and asynchronous provider contracts
- `crates/tool-policy` — std-only bounded tool request, fail-closed authorization, exact one-use confirmation, cancellation-aware one-attempt execution gate, terminal result, and redacted audit contracts
- `crates/protocol` — bounded MCP 2025-06-18, 2025-11-25, and 2026-07-28 plus LSP 3.17 domain values, validation, encoding, lifecycle, capabilities, and dispatch contracts
- `adapters/edt` — implemented EDT configuration-to-semantic-graph adapter
- `adapters/designer-xml` — implemented hierarchical Designer XML configuration-to-semantic-graph adapter
- `adapters/openai-compatible` — implemented explicit bounded OpenAI-compatible `/v1/models` and non-streaming `/v1/completions` provider adapter
- `adapters/lm-studio` — implemented explicit bounded LM Studio native model-discovery and composed non-streaming text-generation provider adapter
- `adapters/ollama` — implemented local-only bounded Ollama native Tags/Show discovery and non-streaming raw generation provider adapter
- `adapters/filesystem` — implemented filesystem workspace discovery adapter
- `extensions/vscode` — desktop VS Code workspace extension with explicit Runtime connection, bounded symbol search, safe source navigation, inspectable semantic Context, and request-selected AI chat
- `docs/adr` — architecture decision records

Runtime builds one all-or-nothing immutable Workspace snapshot from a configured
root through the production filesystem detector and EDT/Designer builders. It
then observes complete file bytes through a Runtime-owned polling source,
serializes rebuilds, atomically publishes valid replacements, and retains the
last valid snapshot across failed rebuilds until a later change recovers. The
transport-neutral snapshot contains separate ordered per-configuration graphs
plus preserved diagnostics, reference evidence, and reports; a public status
observer reports rebuild attempts, publications, phases, and failures. Runtime
stores complete validated snapshots in the fixed Workspace-local
`.oneagent/cache/workspace-v1.json` entry. Startup restores only an exact
source/schema/semantic-version hit; otherwise it clean-builds and safely replaces
the entry after stable initial or watched builds. Incompatible, stale, corrupt,
unavailable, or failed cache work remains recoverable and observable through a
closed typed in-process cache status without changing readiness or query wires.
Runtime exposes exact read-only configuration listing, node lookup, direct
relation, and bounded traversal operations through `GET /api/v1/...`, with
lifecycle/snapshot gating, bounded deterministic results, and closed JSON
success/error schemas. The supported CLI maps its exact commands to those health
and Graph Query GET routes through one bounded HTTP/1.1 connection, preserves
Runtime JSON, and distinguishes usage, transport, server, protocol, and output
failures with stable exit codes. Runtime process management, endpoint discovery,
configuration files, richer output, alternate transports, packaging, Git,
additional MCP/LSP-client compatibility and AI-provider integration remain
planned capabilities with explicit ownership. The bounded desktop VS Code client uses
the separate MCP process for explicit connection and symbol-search/navigation
only. Health remains available through exact `GET /health/live` and
`GET /health/ready` probes.

The additive `oneagent-analysis` Context Engine borrows one immutable
`SemanticGraph`, resolves exact node-ID or canonical-name seeds, applies bounded
deterministic graph selection with retained provenance paths, and returns an
owned semantic-only bundle under an explicit UTF-8 byte budget. Candidate and
budget omissions are reported separately, and exact two-line item fragments
make the result reproducible. Source text/ranges, tokenizers, providers/models,
persistence, direct Context-owned transports, automatic Context collection,
and source-derived IDE integration remain deferred. Runtime adapts this engine
only through the bounded read-only MCP context tool described below.

The additive std-only `oneagent-llm` crate owns provider-scoped identities,
canonical model catalogs, the closed `TextGeneration` capability, bounded
owned text requests and responses, redacted secret/error values, represented
timeout with no automatic retry, cooperative cancellation, and an object-safe
asynchronous provider seam. Public repository-owned fakes prove discovery,
generation, error, cancellation, cleanup, and provider substitution without
network, credentials, Runtime, or Context Engine coupling. The additive
`oneagent-openai-compatible` adapter implements ADR-0046 through
one explicit HTTP/HTTPS server-root URL and optional bearer credential. It
performs fresh bounded `/v1/models` discovery and one strict non-streaming
`/v1/completions` attempt with exact model identity, local UTF-8 byte usage,
no redirect/proxy/retry/fallback, total timeout, cooperative cancellation, and
redacted typed failures. The additive `oneagent-lm-studio` leaf implements
ADR-0047 with stable `lm-studio` identity, explicit or numeric-loopback
construction, type-aware native `/api/v1/models` discovery, and a private
one-attempt generation bridge through the unchanged generic
`/v1/completions` operation. The additive `oneagent-ollama` leaf implements
ADR-0048 with stable `ollama` identity, credential-free numeric-loopback-only
construction, capability-safe native `/api/tags` plus sequential `/api/show`
discovery that excludes remote-backed entries, and one strict non-streaming raw
`/api/generate` attempt. Repository acceptance for all three concrete adapters
uses controlled loopback only. Runtime configuration/exposure, live-provider
acceptance, remote/cloud access, model/server lifecycle, prompt construction,
chat/Responses APIs, streaming, later providers, MCP, and IDE integration
remain deferred.

The additive std-only `oneagent-tool-policy` crate implements the bounded
ADR-0049 library boundary independently of providers and Runtime. It owns
validated request identity, opaque bounded arguments, conservative declared
effects, canonical fail-closed rules, request-wide authorization precedence,
exact non-cloneable one-use confirmation, an object-safe executor seam,
cooperative cancellation precedence, and one typed terminal result with
content-free audit correlation. Repository-owned deterministic fakes prove
zero executor calls for denied, unconfirmed, mismatched, and pre-cancelled
requests; one attempt for allowed or exactly confirmed requests; mapped
completed, partial, failed, executor-reported timeout, and cancelled outcomes;
redaction, repetition, and cleanup. The crate owns no concrete tools, real side
effects, policy storage, clock or timeout enforcement, retry/fallback,
rollback, audit sink, Runtime registration, transport, provider, or IDE
integration. Runtime composes its execution gate for the MCP semantic tools
without moving policy authority. Sprint 27 is complete with a `pass`
integration review.

The MCP foundation preserves stateless revision `2026-07-28` and adds
connection-owned initialize/initialized compatibility for revisions
`2025-06-18` and `2025-11-25`. Sprint 29 adds the semantic-tool boundary, and
Sprint 31 extends its truthful `tools` capability with this lexicographically
ordered read-only catalog: `oneagent.context`, `oneagent.diagnostics`,
`oneagent.graph`, `oneagent.impact`, `oneagent.query`, `oneagent.symbols`, and
`oneagent.validation`.
`oneagent-protocol` owns bounded discovery, `tools/list`, `tools/call`, schemas,
wire errors/results, and asynchronous sequential dispatch. `oneagent-runtime`
builds one immutable Workspace snapshot from the process working directory,
composes every known call through the fail-closed Tool Policy execution gate,
and serves it through the separate newline-framed 1 MiB `oneagent-mcp` stdio
process. Successful tool results contain both compact JSON text and identical
structured content; known semantic failures are tool errors, while malformed
or unknown calls are protocol `Invalid params` errors. Outputs are bounded and
deterministic. The six original tools remain path-free; `oneagent.symbols`
returns only confined Workspace-relative forward-slash source paths and
one-based locations for the accepted Module, Procedure, Function, and EDT Query
slice.

The process constructs no long-running Runtime `App`, watcher, cache, HTTP
listener, background task, remote client, or real side effect. Each stdio run
owns one fresh negotiated session over the immutable server. It keeps stdout
protocol-pure, treats EOF as successful completion, and reports only stable
startup or transport categories on stderr. Exact Codex CLI
`0.150.0-alpha.8` and Cursor Agent `2026.08.25-3e8eec8` evidence is recorded in
the [Sprint 35 compatibility evidence](docs/architecture/external-ai-client-compatibility-evidence.md).
Cursor's public `mcp list-tools` command proves discovery but exposes no direct
tool-call command; Codex directly exercises all seven tools, including semantic
success and domain failure. Additional revisions and clients, remote
transports, authentication, snapshot refresh, Runtime packaging, references,
diagnostics UI, and broader IDE integration remain deferred. The desktop VS
Code extension also consumes the accepted Context and symbol tools through the
bounded Sprint 33 UI described below.

Sprint 32 adds an independent bounded LSP 3.17 process without migrating the
VS Code extension. `oneagent-lsp` builds one immutable startup snapshot, accepts
only Content-Length-framed stdio, and enforces initialize, initialized,
shutdown, and exit sequencing. Its static capabilities are UTF-16 positions,
no document synchronization, `workspaceSymbolProvider`, and pull-only
`diagnosticProvider`. Workspace symbols cover located Procedure, Function, and
EDT Query nodes; full document diagnostic reports project only existing
recoverable Graph diagnostics with located source nodes. Runtime owns canonical
confined file URIs and zero-based ranges. The process reads no source after
startup, emits protocol frames only on stdout, treats EOF before `exit` as a
failure, adds no dependency, and does not claim definition, references,
completion, edits, mutable documents, workspace diagnostics, remote transport,
or external-client compatibility.

Sprint 33 adds an extension-only semantic Context and AI chat slice without
changing Rust, MCP, or provider authority. `OneAgent: Inspect Semantic Context` starts
from an explicitly selected canonical symbol and calls `oneagent.context` with
fixed `both`, depth 2, 32-candidate, and 16,384-byte bounds. The resulting
semantic-only bundle is shown in one escaped, script-free, strict-CSP read-only
panel; closing the panel, disconnecting, replacing the Runtime connection, or
deactivating the extension invalidates model-eligible Context.

The non-default `@oneagent` participant sends exactly the visible rendered
Context and the current 1–8,192-byte user prompt as two messages to the model
selected by the VS Code chat request. Admission enforces a 32,768-byte message
bound and the selected model token limit, accepts text fragments only, and caps
raw output at 65,536 bytes. The extension does not read source for Context,
retain conversation history, expose model tools or edits, infer hidden Context,
or own provider discovery, credentials, or Runtime provider wiring.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

The default test suite uses repository-owned fixtures only. Optional validation
against a separately maintained EDT corpus requires an explicitly authorized
absolute path:

```bash
ONEAGENT_EDT_CORPUS=/absolute/path/to/edt-project \
  cargo test -p oneagent-edt --features external-edt-corpus-tests
```
