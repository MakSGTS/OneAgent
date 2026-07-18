# OneAgent

AI Development Platform for 1C:Enterprise.

## Workspace

- `apps/runtime` — OneAgent Runtime
- `apps/cli` — command-line client
- `crates/common` — shared primitives
- `crates/workspace` — project and workspace model
- `crates/metadata` — semantic 1C metadata model
- `crates/protocol` — protocol contracts
- `adapters` — integrations with EDT, Designer, Git and filesystem
- `extensions` — IDE extensions
- `docs/adr` — architecture decision records

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
