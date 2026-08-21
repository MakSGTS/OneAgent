# Investigate Sprint 15 Runtime Service Container

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 15 execution plan
- `docs/reviews/v0.3-release-review.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `docs/codex/workflows/runtime-service.md`

## Prerequisites / Required gate

Require the committed Sprint 15 planning baseline, committed Runtime Service
framework contracts, and clean task-owned state.

## Investigation objective

Create `docs/architecture/runtime-service-container-investigation.md` with the
exact repository evidence and decision questions needed for ADR-0037 and the
smallest safe, testable long-running service-container slice.

## Questions to answer

- What lifecycle, configuration, state, error, builder, main-entry, and current
  test behavior already exists, and which behavior is compatibility-sensitive?
- Which Runtime dependencies and local APIs can support asynchronous execution,
  owned tasks, cancellation, shutdown, and deterministic probes without a new
  production dependency?
- Who must own registered services, task handles, cancellation state, mutable
  lifecycle state, and terminal failures?
- Which startup, duplicate-registration, partial-start failure, service exit,
  service error, cancellation, join failure, repeated construction, and shutdown
  cases require explicit ADR decisions?
- Which state is safely observable internally in Sprint 15 without defining the
  Sprint 16 HTTP health/readiness contract?
- Which public library boundary is required so later Runtime adapters and
  integration tests can reuse the container without moving composition into an
  adapter?
- Which focused and public integration tests can prove ordering, cleanup, no
  detached tasks, and a genuinely long-running App without arbitrary sleeps?

## Evidence scope

Inspect `apps/runtime/`, its manifest and locked dependencies, all consumers,
ADR-0002, architecture ownership text, CI platforms, current tests, repository
history, and locally available Tokio APIs as needed. Record confirmed behavior,
accepted constraints, unresolved decisions, migration surfaces, deterministic
test oracles, and the smallest candidate implementation boundary.

## Scope

### Included

- The investigation document only.
- Definition/consumer/test/dependency inventories, failure matrix, compatibility
  constraints, candidate public boundary, risks, and ADR readiness.

### Excluded

- Architecture acceptance, production Rust/Cargo changes, new dependencies,
  HTTP/health endpoints, workspace/graph services, file watching, persistence,
  CLI behavior, support claims, and prompt retirement.

## Acceptance Criteria

- Every claimed current behavior and implementation surface cites a live path,
  symbol, test, manifest entry, or committed authority.
- Confirmed evidence, accepted constraints, candidate decisions, and unknowns
  are separated.
- The document defines a deterministic positive/negative test oracle and proves
  that no external fixture or service is required.
- The evidence is sufficient for ADR-0037 or records an exact blocker without
  hiding it as implementation work.

## Repository Safety

Preserve `.codex/`, production code, Cargo files, prompt suites, and unrelated
paths. Stage only the investigation document when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime`
- Verify every cited path, symbol, dependency, and test target.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Investigate Sprint 15 Runtime service container`

## Final report additions

Report confirmed Runtime behavior, ownership and failure questions, public and
consumer surfaces, deterministic testability, unknowns, ADR readiness, exact
validation, commit, and final Git state.
