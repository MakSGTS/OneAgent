# Investigate Sprint 16 HTTP API and Health Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 16 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-15-runtime-service-container.md`
- `docs/architecture/runtime-service-container-investigation.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/codex/workflows/runtime-service.md`

## Prerequisites / Required gate

Require the committed Sprint 16 planning baseline and clean task-owned state.
The locked Axum and Tokio sources and current Runtime public path must be locally
readable; otherwise stop because endpoint, ownership, and executable test
contracts cannot be inferred.

## Investigation objective

Create `docs/architecture/http-api-health-investigation.md` with the exact
repository evidence needed to decide ADR-0038 and implement the smallest safe,
testable HTTP liveness/readiness slice.

## Questions to answer

- Which current configuration and composition APIs can own a bind address and
  register one HTTP Runtime service without hidden dependency construction?
- Which locked Axum/Tokio APIs acquire a listener before startup acknowledgement,
  expose its actual address, serve a router, and complete graceful shutdown?
- Which lifecycle facts can derive liveness and readiness without a mutable
  duplicate authority, and what can be observed before `Running` and after
  `Stopping` begins?
- What exact route, method, status, content type, body schema, and unknown-route
  behavior require ADR decisions before implementation?
- How must invalid configuration, bind failure, serve failure, cancellation,
  listener release, in-flight connections, and concurrent terminal outcomes map
  to the accepted Runtime service error boundary?
- Which public consumers exist, what compatibility surface is new, and which
  later Sprint 17-21 APIs must remain absent?
- How can real loopback client/server tests prove the wire contract, readiness
  transitions, negative cases, shutdown, resource release, and repeated fresh
  runs on macOS and Windows without sleeps or external services?

## Evidence scope

Inspect the live Runtime implementation and tests, Runtime Cargo surface and
lockfile, local Axum 0.8.9 and Tokio 1.53.0 sources, CI targets, Roadmap,
architecture, ADRs, current documentation, consumers, history, and existing
HTTP-related code. Record exact APIs, paths, state transitions, failure points,
wire decision matrix, ownership inventory, compatibility risks, and a bounded
public test oracle. Separate confirmed evidence, accepted constraints, candidate
decisions, and unknowns.

## Scope

### Included

- The investigation document only.
- Configuration, listener, lifecycle, endpoint, wire, ownership, failure,
  compatibility, negative-case, and public-test evidence.
- The smallest coherent first slice and explicit ADR decision questions.

### Excluded

- Architecture acceptance, Rust/Cargo changes, HTTP implementation, routes,
  fixtures, support claims, current-state synchronization, sprint completion,
  and prompt retirement.

## Acceptance Criteria

- Every proposed API, lifecycle fact, failure case, wire field, negative case,
  and test step is backed by an exact repository or locked-source location.
- The document identifies the smallest coherent first slice and all deliberately
  deferred endpoint, security, workspace, graph, and client behavior.
- Evidence is sufficient for ADR-0038 or records an exact blocker; no external
  service or speculative dependency is hidden as future implementation work.
- The oracle uses actual loopback HTTP I/O, explicit synchronization, and
  bounded hang guards rather than handler-only success or arbitrary sleeps.

## Repository Safety

Preserve `.codex/`, production code, Cargo files, existing suites, and unrelated
files. Stage only the investigation document when commit mode is authorized.

## Task-specific Validation

- Verify every cited Runtime and locked-dependency API and repository path.
- Verify the current public test inventory and CI platform matrix.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Investigate Sprint 16 HTTP API and health`

## Final report additions

Report confirmed ownership, lifecycle, dependency, wire, failure, consumer, and
test-oracle evidence; remaining decision questions; first-slice readiness;
exact commands; changed paths; commit; and final Git state.
