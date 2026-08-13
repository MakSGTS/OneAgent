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
select the smallest sufficient Profile and Template instead of duplicating Core
rules, Workflow rules, validation requirements, and final-report requirements.
