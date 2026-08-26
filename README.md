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
review. Sprint 27 Tool Execution Policy is the unique next planning target.
See
[`docs/Roadmap.md`](docs/Roadmap.md) for canonical execution order.

## Workspace

- `apps/runtime` — long-running Runtime composition, owned service lifecycle, cancellation, shutdown, EDT/Designer Workspace discovery and file-change rebuilds, validated persistent snapshot caching, immutable semantic snapshots, public update/cache observation, HTTP liveness/readiness, and the versioned read-only Graph Query API
- `apps/cli` — supported dependency-free CLI client for Runtime health, Workspace configuration listing, exact node lookup, direct relations, and bounded traversal
- `crates/common` — shared primitives
- `crates/workspace` — project and workspace model
- `crates/metadata` — typed 1C metadata model
- `crates/graph` — canonical semantic graph, query, validation, diff, impact, coverage, and resolution APIs
- `crates/bsl` — BSL lexical and syntax analysis
- `crates/analysis` — source-independent declaration/call analysis and deterministic semantic Context Engine
- `crates/llm` — provider-neutral bounded identity, model discovery, text request/response, policy, cancellation, error, and asynchronous provider contracts
- `crates/protocol` — protocol package foundation; transport contracts are not implemented yet
- `adapters/edt` — implemented EDT configuration-to-semantic-graph adapter
- `adapters/designer-xml` — implemented hierarchical Designer XML configuration-to-semantic-graph adapter
- `adapters/openai-compatible` — implemented explicit bounded OpenAI-compatible `/v1/models` and non-streaming `/v1/completions` provider adapter
- `adapters/lm-studio` — implemented explicit bounded LM Studio native model-discovery and composed non-streaming text-generation provider adapter
- `adapters/ollama` — implemented local-only bounded Ollama native Tags/Show discovery and non-streaming raw generation provider adapter
- `adapters/filesystem` — implemented filesystem workspace discovery adapter
- `extensions` — reserved for future IDE extensions; currently empty
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
configuration files, richer output, alternate transports, packaging, Git, MCP,
LSP, VS Code, and AI-provider integration remain planned capabilities with
explicit ownership. Health remains available through exact `GET /health/live`
and `GET /health/ready` probes.

The additive `oneagent-analysis` Context Engine borrows one immutable
`SemanticGraph`, resolves exact node-ID or canonical-name seeds, applies bounded
deterministic graph selection with retained provenance paths, and returns an
owned semantic-only bundle under an explicit UTF-8 byte budget. Candidate and
budget omissions are reported separately, and exact two-line item fragments
make the result reproducible. Source text/ranges, tokenizers, providers/models,
Runtime or protocol routes, persistence, MCP, and IDE integration remain
deferred.

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
acceptance, remote/cloud access, model/server lifecycle, prompt/tool policy,
chat/Responses APIs, streaming, later providers, MCP, and IDE integration
remain deferred. Sprints 24–26 are complete; Sprint 27 Tool Execution Policy
is the unique next planning target.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
