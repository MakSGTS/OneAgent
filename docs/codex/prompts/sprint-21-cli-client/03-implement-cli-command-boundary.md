# Implement Sprint 21 CLI Command Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 21 execution plan
- `docs/architecture/cli-client-investigation.md`
- `docs/adr/0043-cli-client.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed accepted ADR-0043, clean task-owned state, and any explicit
approval ADR-0043 requires before a production manifest change. Stop rather than
selecting a new CLI or transport contract during implementation.

## Task

Implement only the accepted reusable CLI command boundary: deterministic
argument parsing and validation, typed request selection, stable help/version
and local diagnostics, stream routing, and exit classification. Keep network
execution behind an injected boundary for Task 4.

## Runtime and client ownership

`oneagent-cli` owns command parsing and presentation. It does not own Runtime,
Workspace, graph semantics, a listener, global state, or background tasks.

## Lifecycle, failure, and observability

One invocation parses once, invokes at most one injected request executor, emits
only accepted output, and returns one accepted exit classification. Local usage
errors must not open a connection. Help/version must be deterministic and
side-effect free.

## Scope

### Included

- Minimum reusable library/main split accepted by ADR-0043.
- Complete command and option model, parsing, duplicate/unknown/missing/value
  validation, precedence, query model construction, help/version, diagnostics,
  stdout/stderr/newline behavior, and exit status.
- Focused tests for every grammar branch, default/boundary value, invalid form,
  output stream, executor call/no-call, deterministic repetition, and no panic.

### Excluded

TCP/HTTP I/O, response parsing, production Runtime integration tests, Runtime or
protocol changes, semantic interpretation, new commands, dependencies without
approval, current-state docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- The public executable no longer prints placeholder output, but network work
  remains supplied only through the accepted injected executor seam.
- Parsing and validation exactly implement ADR-0043 with stable precedence and
  no connection attempt for local help/version/error outcomes.
- Typed requests preserve exact server values for Task 4 encoding and cannot
  represent an unsupported command combination.
- stdout, stderr, newline, diagnostics, and exit classification match the ADR.
- Focused tests are non-zero, deterministic, cross-platform, and cover every
  accepted command plus invalid/duplicate/unknown/missing/boundary cases.
- No Runtime route/schema/lifecycle, graph/adapter behavior, protocol authority,
  or production dependency changes.

## Repository Safety

Modify only `apps/cli/src/main.rs`, create only the minimum accepted files under
`apps/cli/src/`, and modify `apps/cli/Cargo.toml`/`Cargo.lock` only if ADR-0043
requires and separate explicit dependency approval is present. Preserve all
other paths and stage only task-owned files.

## Task-specific Validation

- Run the exact non-zero focused CLI command/parser/output tests.
- `cargo test -p oneagent-cli`
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 21 CLI command boundary`

## Final report additions

Report implemented command model, ownership, parsing/validation, outputs/exits,
executor seam, focused/full validation, dependency impact, changed paths,
commit, and final Git state.
