# Implement Sprint 21 Runtime HTTP Client

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
- committed Task 3 command boundary

## Prerequisites / Required gate

Require committed Task 3 with exact command/request/output/exit behavior and a
clean task-owned state. Require any explicit production dependency approval
mandated by ADR-0043 before changing a manifest.

## Task

Implement only the accepted bounded Runtime HTTP/1.1 client and connect it to
the Task 3 command executor seam.

## Ownership and lifecycle

One CLI invocation owns at most one blocking client connection. It creates,
writes, reads, validates, reports, and drops that connection before returning.
No background task, listener, pool, cache, global state, retry, or detached work
is permitted.

## Transport and compatibility

Map every accepted command to its exact Sprint 16/18 GET route. Preserve exact
query values through accepted encoding, accepted parameter order, response
status/media/body, and Runtime JSON without semantic reserialization.

## Scope

### Included

- Accepted endpoint connection, exact request target and headers, percent
  encoding, HTTP/1.1 write/read/framing, status/header validation, body and size
  bounds, JSON media/body boundary, success/domain-error passthrough, output
  routing, transport/protocol/server exit classification, and cleanup.
- Focused controlled-server tests for every framing mode and representative
  success, domain error, unreachable, malformed, truncated, oversized,
  unsupported, cleanup, exact-request, and repeated-call outcome.
- Connection of `main` to the production executor.

### Excluded

Runtime server changes, new routes, schema interpretation/reformatting, DNS,
URLs, proxies, redirects, authentication, TLS, HTTP/2, retries, connection
pooling, configurable timeout policy not accepted by ADR-0043, process
supervision, new commands, protocol activation, current-state docs, sprint
transition, and prompt retirement.

## Acceptance Criteria

- Every command emits the exact accepted GET target and no unsupported request.
- Request encoding preserves arbitrary accepted identifier bytes according to
  ADR-0040 and never double-encodes or emits ambiguous query delimiters.
- Response handling implements every accepted framing/limit/media/status/body
  rule, rejects malformed or incomplete protocol deterministically, and never
  prints partial or untrusted output as success.
- Successful and Runtime-domain JSON remains byte-preserving except the exact
  accepted terminal newline policy.
- Local, transport, protocol, and server outcomes retain distinct accepted
  stream and exit behavior.
- Connections close on every terminal path; repeated calls remain independent.
- Non-zero focused tests cover exact requests, responses, failures, cleanup, and
  repetition on cross-platform standard-library seams.
- No Runtime/semantic/protocol contract or production dependency changes beyond
  separately approved ADR scope.

## Repository Safety

Modify only Task 3 files under `apps/cli/src/` and the minimum new focused tests
under `apps/cli/`. Modify `apps/cli/Cargo.toml`/`Cargo.lock` only with the exact
accepted need and separate approval. Preserve Runtime, graph, adapters, docs,
fixtures, prompts, `.codex/`, and unrelated paths. Stage task-owned paths only.

## Task-specific Validation

- Run non-zero focused exact-request/framing/failure/cleanup/repetition tests.
- `cargo test -p oneagent-cli`
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 21 Runtime HTTP client`

## Final report additions

Report request/response contracts, resource ownership, failure/exit behavior,
body limits, cleanup/repetition evidence, focused/full validation, dependency
impact, changed paths, commit, and final Git state.
