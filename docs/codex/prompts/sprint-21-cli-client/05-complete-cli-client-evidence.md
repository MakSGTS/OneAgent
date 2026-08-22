# Complete Sprint 21 CLI Client Evidence

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
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- committed Tasks 3-4 implementation

## Prerequisites / Required gate

Require committed Task 4, all focused implementation validation successful,
and a clean task-owned state. Stop rather than repairing architecture or
implementation defects inside an evidence-only task.

## Task

Add the complete public CLI-to-production-Runtime integration evidence required
by ADR-0043 and synchronize truthful current-state documentation. Do not mark
Sprint 21 completed.

## Public client/server boundary

Exercise the supported CLI entry point through its public invocation boundary
against a real query-enabled `oneagent-runtime` instance using tracked temporary
EDT and Designer XML inputs and loopback port zero. Handler-only or fake-server-
only tests are insufficient for production compatibility evidence.

## Scope

### Included

- Public tests for help/version and every health/configuration/node/relation/
  traversal command, exact request mapping, defaults/bounds, both configuration
  formats, deterministic output, stdout/stderr, and process exit behavior.
- Invalid CLI invocations, Runtime domain errors, readiness/workspace
  unavailable cases where deterministically observable, unreachable endpoint,
  shutdown, listener/connection cleanup, and equal fresh/repeated runs.
- Preservation of Graph Query ordering and one-snapshot responses, lifecycle
  readiness, File Watching/cache compatibility, and Runtime cleanup.
- Synchronize `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` with implemented supported behavior and
  explicit deferred scope.

### Excluded

Production fixes, architecture reselection, new commands or dependencies,
Runtime wire/configuration changes, protocol activation, new fixtures without
proven necessity, semantic/adapter/Coverage changes, v0.4 review, Sprint 22,
packaging, benchmarks, performance/security claims, sprint completion, and
prompt retirement.

## Acceptance Criteria

- A non-zero public integration target proves every supported command through
  the real CLI and real Runtime public boundaries over both tracked formats.
- Success output exactly matches accepted Runtime JSON and deterministic order;
  local, server, transport, and protocol failures use accepted streams/exits.
- Required defaults, minima/maxima, filters, include-start, missing selections,
  invalid invocations, readiness/unavailable behavior, and unreachable endpoint
  have observable evidence without zero-match filters or arbitrary sleeps.
- Runtime shutdown releases its listener, CLI connections terminate, observers
  and tasks close, and fresh/repeated runs remain independent and equal.
- Existing health/Graph Query schemas, Workspace publication, File Watching,
  Persistent Cache, graph semantics, source adapters, and Coverage stay intact.
- Current-state docs agree on the supported first slice and all deferred
  process-management, discovery, protocol, security, packaging, and later work.
- Sprint 21 remains incomplete pending Task 6.

## Repository Safety

Create only the minimum public test files under `apps/cli/tests/`; modify only
`apps/cli/Cargo.toml` for repository-local/dev dependencies proven necessary,
`Cargo.lock` only if mechanically required, and the three current-state docs
listed above. Preserve production code, Runtime fixtures, prompts, Roadmap,
`.codex/`, and unrelated paths. Stage task-owned paths only.

## Task-specific Validation

- List and run the exact non-zero public CLI/client/server target.
- `cargo test -p oneagent-cli`
- Run relevant Runtime health and Graph Query public targets.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Validate doc links/claims, diff scope, and `git status --short`.

## Suggested commit message

`Complete Sprint 21 CLI Client evidence`

## Final report additions

Report the public matrix, both fixture formats, outputs/exits, lifecycle and
cleanup evidence, preserved contracts, current-state docs, deferred scope,
focused/full validation, changed paths, commit, and final Git state.
