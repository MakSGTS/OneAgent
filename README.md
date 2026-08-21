# OneAgent

AI Development Platform for 1C:Enterprise.

OneAgent has completed the v0.3 source-independent 1C Knowledge Model and the
Sprint 15 long-running Runtime service container. Sprint 16 HTTP API and Health
is the next execution target. See [`docs/Roadmap.md`](docs/Roadmap.md) for the
canonical execution order.

## Workspace

- `apps/runtime` — long-running Runtime composition, owned service lifecycle, cancellation, shutdown, and public in-memory integration evidence
- `apps/cli` — CLI package placeholder; supported client behavior is planned for Sprint 21
- `crates/common` — shared primitives
- `crates/workspace` — project and workspace model
- `crates/metadata` — typed 1C metadata model
- `crates/graph` — canonical semantic graph, query, validation, diff, impact, coverage, and resolution APIs
- `crates/bsl` — BSL lexical and syntax analysis
- `crates/analysis` — source-independent declaration and call-graph analysis
- `crates/protocol` — protocol package foundation; transport contracts are not implemented yet
- `adapters/edt` — implemented EDT configuration-to-semantic-graph adapter
- `adapters/filesystem` — implemented filesystem workspace discovery adapter
- `extensions` — reserved for future IDE extensions; currently empty
- `docs/adr` — architecture decision records

HTTP/health, workspace and graph Runtime services, persistence, supported CLI,
Git, MCP, LSP, VS Code, and AI-provider integration are planned capabilities
with explicit sprint ownership in the roadmap; they are not part of the current
implemented Runtime product surface.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
