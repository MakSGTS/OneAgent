# Define Sprint 21 CLI Client Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 21 execution plan
- `docs/architecture/cli-client-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned production source and deterministic oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0043-cli-client.md`, defining the smallest complete
cross-platform supported CLI Client contract. Synchronize only planning-level
architecture text required to make the decision unambiguous. Implement no Rust.

## Scope

### Included

- CLI/library/main ownership, dependency direction, process/resource ownership,
  and relationship to Runtime, HTTP, graph, adapters, and `oneagent-protocol`.
- Exact command/subcommand/option grammar, ordering, duplication, required and
  optional values, defaults, validation precedence, help/version behavior, and
  stable local diagnostics.
- Endpoint representation/default/override and exact accepted address scope.
- Mapping of commands to accepted health and `/api/v1` GET routes, deterministic
  query ordering/percent encoding, request headers, HTTP version, and bounds.
- Response status/header/body framing, response-size containment, JSON media
  validation and opaque presentation, stdout/stderr/newline rules, server-domain
  error handling, transport/protocol failures, and exact exit classifications.
- Connection lifecycle, blocking ownership, read/write termination, cleanup,
  repeated calls, compatibility impact, dependency choice, public test strategy,
  first production slice, rejected alternatives, and deferred scope.

### Excluded

Rust/Cargo/fixture changes, Runtime wire/configuration changes, service process
management, Workspace mutations, cache/watch management, JSON interpretation or
reformatting, protocol-crate activation, new production dependencies without
approval, DNS/URL/proxy/redirect/TLS/HTTP2/retry/general-timeout contracts,
packaging/completion/telemetry, benchmarks, security/performance claims,
Coverage transitions, sprint completion, and prompt retirement.

## Acceptance Criteria

- ADR-0043 answers every Task 1 decision question with one canonical contract
  grounded in live evidence and accepted ADRs.
- Every supported command maps to exactly one existing stable Runtime GET route
  and preserves its parameters, defaults, limits, lifecycle, schema, ordering,
  and server error behavior.
- Command parsing, endpoint input, query encoding, framing/body bounds, streams,
  newline behavior, local/server/transport/protocol errors, precedence, and exit
  codes are closed and observable.
- The client treats successful and domain-error JSON as Runtime-owned wire data;
  it does not become semantic authority or silently tolerate malformed protocol.
- Every connection/resource has one owner and deterministic terminal behavior;
  no task, listener, global state, cache, or detached work is introduced.
- Dependency choice is explicit. If a new production dependency is required,
  Task 3 remains gated on separate explicit user approval.
- Public evidence covers every command and representative local, server,
  unavailable, malformed, truncated, oversized, cleanup, ordering, and repeated
  case on supported platforms without external services or arbitrary sleeps.
- Rejected alternatives, compatibility, first slice, implementation order,
  Coverage impact, v0.4 review hand-off, and later deferrals are explicit.
  Sprint 21 remains `next`; current-state docs do not claim implementation.

## Repository Safety

Create only `docs/adr/0043-cli-client.md` and modify only the minimum planning-
level architecture document if required. Preserve `.codex/`, Rust, manifests,
lockfile, fixtures, prompts, Roadmap state, current implementation claims, and
unrelated files. Stage only ADR-owned paths when commit mode is authorized.

## Task-specific Validation

- Verify decision/evidence consistency with Task 1 and cited public contracts.
- Validate internal links, ADR status, closed grammar/endpoint/request/response/
  error/exit/resource/dependency matrices, alternatives, first slice,
  prerequisites, accepted/deferred scope, and `git diff --check`.
- `git status --short`

## Suggested commit message

`Define Sprint 21 CLI Client contract`

## Final report additions

Report the accepted ownership, grammar, endpoint, request/response, streams,
errors/exits, resource, compatibility, dependency, testing, first-slice, and
deferred-scope decisions; changed paths; validation; commit; final Git state;
and whether Task 3 is unblocked.
