# Development

Primary development platform: macOS on Apple Silicon.

Required checks:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Codex task workflow rules live in `docs/codex/README.md`. Task prompts should
select the required framework modules explicitly instead of duplicating the
full permanent rule set.
