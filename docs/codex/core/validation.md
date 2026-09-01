# Validation

Use validation that matches the risk and scope of the task. Never claim that a
check passed unless the command completed successfully.

## Focused validation

Run tests and checks directly related to changed components. Start as narrow as
possible when code changes are localized.

## Package validation

Run relevant crate or package checks when a task changes a crate, public API,
parser behavior, graph model, or tests.

## Workspace validation

The canonical full workspace validation commands are:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Run full workspace validation when:

- production Rust code changes;
- Cargo manifests change;
- public APIs change;
- graph model changes;
- parser behavior changes;
- graph emission changes;
- the task explicitly requires it.

## Documentation-only tasks

Do not blindly run Rust workspace validation for documentation-only tasks. At
minimum run:

```bash
git diff --check
```

Also run any existing Markdown linter, link checker, documentation test, or docs
build discovered in the repository. If no such tool exists, inspect changed
Markdown links and required sections manually.

When Prompt Contract v2 files or their generator, base Template, context rules,
or execution workflow change, also run:

```bash
bash -n scripts/validate-codex-prompts.sh
scripts/validate-codex-prompts.sh
```

Generated child prompts must be passed explicitly to the validator before their
planning baseline is accepted.

## Zero matched tests

Report zero matched test filters separately. A filter that runs zero tests is
not evidence that a capability is tested.

## Output retention

Keep successful command output concise. When a materially large complete log is
needed for failure diagnosis or review, retain it under
`local-artifacts/codex-runs/<run-id>/` and report the command, exit status,
meaningful count, concise failure excerpt, and artifact path. Do not copy the
complete log into a prompt, task ledger, or final report.
