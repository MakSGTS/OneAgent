# OneAgent

AI Development Platform for 1C:Enterprise.

OneAgent has completed the v0.3 source-independent 1C Knowledge Model, the
Sprint 15 long-running Runtime service container, and the Sprint 16 HTTP API
and Health boundary. The Sprint 17 Workspace Service implementation and public
production evidence are completed with a `pass` integration review. Sprint 18
Graph Query API is also completed with a `pass` integration review. Sprint 19
File Watching, Sprint 20 Persistent Cache, and Sprint 21 CLI Client are
completed with `pass` integration reviews. The v0.4 release integration review
is the unique next gate. See
[`docs/Roadmap.md`](docs/Roadmap.md) for canonical execution order.

## Workspace

- `apps/runtime` — long-running Runtime composition, owned service lifecycle, cancellation, shutdown, EDT/Designer Workspace discovery and file-change rebuilds, validated persistent snapshot caching, immutable semantic snapshots, public update/cache observation, HTTP liveness/readiness, and the versioned read-only Graph Query API
- `apps/cli` — supported dependency-free CLI client for Runtime health, Workspace configuration listing, exact node lookup, direct relations, and bounded traversal
- `crates/common` — shared primitives
- `crates/workspace` — project and workspace model
- `crates/metadata` — typed 1C metadata model
- `crates/graph` — canonical semantic graph, query, validation, diff, impact, coverage, and resolution APIs
- `crates/bsl` — BSL lexical and syntax analysis
- `crates/analysis` — source-independent declaration and call-graph analysis
- `crates/protocol` — protocol package foundation; transport contracts are not implemented yet
- `adapters/edt` — implemented EDT configuration-to-semantic-graph adapter
- `adapters/designer-xml` — implemented hierarchical Designer XML configuration-to-semantic-graph adapter
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

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
