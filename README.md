# OneAgent

AI Development Platform for 1C:Enterprise.

OneAgent is currently building its source-independent semantic core. Sprints
1–3 are complete, and Sprint 4 (Semantic Index) is the next execution target.
See [`docs/Roadmap.md`](docs/Roadmap.md) for the canonical execution order.

## Workspace

- `apps/runtime` — Runtime composition and lifecycle foundation; long-running services are planned for v0.4
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

Designer XML, Git, HTTP, CLI, MCP, LSP, VS Code, and AI-provider integration
are planned capabilities with explicit sprint ownership in the roadmap; they are
not part of the current implemented adapter surface.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
