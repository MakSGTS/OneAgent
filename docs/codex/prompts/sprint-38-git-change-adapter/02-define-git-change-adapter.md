# Define Sprint 38 Git Change Adapter

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/architecture.md`
- `docs/codex/templates/architecture-task.md`

## Required workflow

`docs/codex/workflows/architecture.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/git-change-adapter-investigation.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/reviews/sprint-37-rules-engine.md`

## Prerequisite

Task 1 is committed, the Git Change Adapter framework prerequisite remains
committed, and the investigation contains no blocking evidence gap or
unapproved production dependency.

## Task

Create `docs/adr/0060-git-change-adapter.md` and synchronize only planning-level
architecture text required by the accepted decision. Implement no production
behavior.

## Required decisions

- Fix the first-slice local repository and Workspace boundaries, dependency
  direction, owner, and explicit rule that Git evidence is not semantic,
  validation, diagnostic, impact, rule, cache, or edit authority.
- Define typed baseline and current endpoints, resolution and stability,
  included repository state layers, completeness, concurrent mutation,
  nested/worktree/submodule/bare/missing/incompatible behavior, and exact
  first-slice deferrals.
- Fix normalized change identity, old/new path representation, closed statuses,
  addition/modification/deletion/type-change/rename/copy/conflict/untracked/
  ignored behavior, duplicates, ambiguity, canonical total order, bounds,
  path encoding and confinement, and closed redacted failures.
- Select the implementation family and exact dependency/process boundary from
  investigation evidence. Define executable or library lifecycle, injection,
  environment, I/O, cancellation, timeout, cleanup, platform compatibility,
  and approval status without hiding an unresolved prerequisite.
- Define the source-independent Workspace change-input contract and mapping,
  relevance or empty behavior, equivalence to complete filesystem end states,
  coalescing, serialization, complete rebuild, publication, failure/recovery,
  cache, lifecycle, and supported-consumer behavior.
- Preserve ADR-0027 canonical Graph/index input, production source adapter and
  Graph authority, full build validation, immutable snapshots, and existing
  public protocols unless an exact additive migration is required.
- Fix exact repository-owned acceptance evidence for domain, reader, process
  or dependency, repository states, paths, ordering, bounds, failures,
  Workspace equivalence, cache/rebuild, lifecycle, consumers, platform,
  sensitive data, scope, and full validation.
- Record rejected alternatives and defer remote access, credentials,
  repository mutation, semantic impact, selective semantic updates,
  refactoring, edits, protocol/IDE Git UI, telemetry, benchmarks, and broad
  performance/security claims.

## Acceptance evidence

ADR-0060 is `Accepted`, maps every investigation question to one explicit
decision or deferral, assigns Tasks 3–6, identifies public consumers and any
migration, preserves accepted authority, introduces no dependency without
approval, and agrees with the Roadmap and Sprint 39 boundary.

## Excluded scope

Rust implementation, behavior-encoding fixtures or tests, Cargo changes,
unapproved dependency use, repository mutation, review artifacts, prompt-suite
retirement, Sprint completion, Change Impact Analysis, refactoring, source
edits, and product UI.

## Validation

Run investigation-question coverage; authority/endpoint/state/status/path/
order/bound/failure/process/dependency/Workspace/cache/lifecycle/consumer
consistency; sensitive-data and deferred-scope audits; Markdown link checks;
`git diff --check`; and unrelated-change inspection.

## Suggested commit message

`Define Sprint 38 Git Change Adapter`

## Final report additions

Report accepted authority, repository and endpoints, included state layers,
change identity/status/path/order/bounds/failures, implementation family,
Workspace mapping and equivalence, lifecycle/cache/consumer compatibility,
evidence, rejected alternatives, deferred scope, and unchanged production
behavior.
