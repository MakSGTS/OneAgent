# Complete Sprint 35 External Client Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/templates/mcp-protocol-task.md`

## Required workflows

- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/runtime-service.md`
- `docs/codex/workflows/implementation.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/external-ai-client-compatibility-investigation.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0057-external-ai-client-compatibility.md`

## Prerequisite

Task 4 is committed, its full gate passes, and the exact user-authorized Codex
and Cursor executables remain available without broadening external access.

## Task

Complete the repository-owned compatibility fixtures, public-client evidence,
cross-platform conformance coverage, audits, and current-state documentation.
Do not introduce new production behavior.

## Required evidence

- Add only ADR-0057-accepted reusable fixtures or harness code needed to drive
  supported-version initialize/list/call/failure/shutdown workflows through the
  repository-built production `oneagent-mcp`; keep client binaries and logs
  untracked.
- Run the exact pinned Codex CLI against a disposable repository-local config
  and prove successful initialization, exact seven-tool discovery, at least one
  deterministic representative tool call, domain failure visibility, repeated
  invocation, clean termination, and no global configuration mutation.
- Run the exact pinned Cursor Agent through its supported project-local public
  MCP command and prove the corresponding initialize, seven-tool discovery,
  representative call if the public command supports it, failure, repetition,
  and cleanup rows. Clearly distinguish client-command limitations from server
  failures and never claim an unexecuted row.
- Execute platform-neutral synthetic conformance for every ADR-0057-supported
  version, lifecycle order, response projection, malformed/unsupported case,
  connection isolation, EOF/shutdown, framing, repetition, and modern regression.
- Reassert exact tool names/order/schemas, Tool Policy, semantic success/domain
  results, startup workspace behavior, and existing VS Code/LSP/EDT compatibility.
- Synchronize only current-state README, Architecture, Semantic Model, and
  Roadmap text required to describe verified behavior, supported versions,
  client setup, limitations, and deferred scope.
- Audit dependencies/licenses, secrets, credentials, personal paths, global
  config changes, ignored/generated/downloaded artifacts, repository status,
  protocol source pinning, scope, documentation links, and zero-match/zero-skip
  claims. Preserve `local-artifacts/sprint-35/` as ignored evidence only.

## Excluded scope

New protocol behavior, new clients beyond Codex/Cursor, client publication or
installation, global config, credentials, HTTP/SSE/remote transport,
authentication, tool/catalog/semantic changes, release review, Sprint 36, and
prompt-suite retirement.

## Validation

Run the complete focused protocol/Runtime/public-process matrix, exact public
Codex and Cursor commands, synthetic supported-version conformance, existing
MCP semantic, VS Code, LSP, and EDT compatibility gates affected by the change,
documentation/audit scripts, then:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
git diff --check
```

Record exact commands, versions, counts, exits, skips, limitations, and
artifact locations. A missing executable, authentication prompt, zero-match
filter, skipped required row, global-config requirement, or client/server
failure blocks completion.

## Suggested commit message

`Complete Sprint 35 external client evidence`

## Final report additions

Report exact Codex/Cursor and synthetic rows, commands and outcomes, repository
fixtures/docs, supported versions, catalog/semantic preservation, audits,
external/global-config compliance, limitations, and canonical gate results.
