# OneAgent

AI Development Platform for 1C:Enterprise.

OneAgent has completed the v0.3 source-independent 1C Knowledge Model, the
Sprint 15 long-running Runtime service container, and the Sprint 16 HTTP API
and Health boundary. The Sprint 17 Workspace Service implementation and public
production evidence are completed with a `pass` integration review. Sprint 18
Graph Query API is the unique next target. See
[`docs/Roadmap.md`](docs/Roadmap.md) for canonical execution order.

## Workspace

- `apps/runtime` — long-running Runtime composition, owned service lifecycle, cancellation, shutdown, initial EDT/Designer Workspace discovery and immutable semantic snapshots, and public HTTP liveness/readiness
- `apps/cli` — CLI package placeholder; supported client behavior is planned for Sprint 21
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

Runtime now builds one all-or-nothing immutable Workspace snapshot from a
configured root through the production filesystem detector and EDT/Designer
builders. The initial snapshot remains transport-neutral and contains separate
ordered per-configuration graphs plus preserved diagnostics, reference
evidence, and reports. Graph-query HTTP APIs, rebuild watching, persistence,
supported CLI, Git, MCP, LSP, VS Code, and AI-provider integration remain
planned capabilities with explicit sprint ownership. The current HTTP surface
is still limited to `GET /health/live` and `GET /health/ready`.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
