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

## Validation staging

These staging rules apply to Sprint 41 and later sprint suites. Earlier active
suites continue under their committed execution prompts and validation
contracts.

Treat a changing implementation as unstable until its accepted architecture,
production locations, negative oracles, and task-owned path inventory are
reconciled. While the diff is unstable, run only the focused and package checks
needed to develop or diagnose the current change. Do not spend a canonical full
workspace gate on a speculative, partially remediated, or reviewer-visible
floating diff unless a higher-priority instruction explicitly requires it.

Run the canonical full workspace gate once after the pre-review implementation
diff is stable and all focused checks pass. If review requires remediation, do
not rerun the full gate for each individual finding or remediation commit:
return to focused validation until all accepted findings are fixed, then run
one final canonical gate on the stable final remediation range. The independent
reviewer and primary completion-gate runs remain separate required evidence;
they do not justify additional pre-review full runs on the same unchanged diff.

Do not run a reviewer or another Cargo command concurrently with validation
that shares the same build directory. Sequence those operations or give them
explicitly isolated build directories when the task requires safe parallelism.
Record the stable commit or diff range qualified by every full validation run.

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
complete log into a prompt, agent handoff, task ledger, or final report. Pass
reviewers the compact command summary and retained-log path; load the full log
only when a specific failure or evidence discrepancy requires it.
