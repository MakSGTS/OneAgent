# Investigate Sprint 38 Git Change Adapter

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/investigation.md`
- `docs/codex/templates/investigation-task.md`

## Authoritative documents and evidence

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/file-watching-investigation.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/reviews/sprint-37-rules-engine.md`
- repository Git history and metadata; Workspace observation, build, cache,
  lifecycle, public-process, fixture, platform, dependency, and consumer code

## Prerequisites / required gate

- The committed Sprint 38 planning baseline and Git Change Adapter framework
  prerequisite are ancestors of HEAD.
- Sprint 37 is completed and Sprint 38 is the unique eligible target.
- The working tree has no uncommitted task-created changes.

## Task

Create `docs/architecture/git-change-adapter-investigation.md` and update only
the Sprint 38 Roadmap state needed to record Task 1 start. Produce
decision-ready repository evidence for ADR-0060 without production
implementation.

## Questions and required evidence

- Inventory every existing Git-related type, command, dependency, test helper,
  fixture, CI assumption, path convention, repository operation, and consumer.
  Separate governance Git usage from candidate production adapter behavior.
- Trace the complete ADR-0041 `WorkspaceFileState`, change-source revision,
  rebuild/coalescing, publication, failure/recovery, cache, cancellation,
  shutdown, and public-observer paths plus every internal and public consumer.
- Confirm that no typed Git repository endpoint, normalized change set,
  repository reader port, Git Workspace input, or competing semantic change
  authority already exists.
- Establish the exact local repository forms and real Git capabilities that can
  be tested safely with repository-owned temporary repositories on supported
  macOS and Windows CI. Record executable version and dependency evidence
  without selecting an implementation.
- Define decision-ready repository-root and Workspace-root alternatives,
  baseline/current endpoint identity, committed/index/worktree/untracked/
  ignored/conflict layer alternatives, completeness, concurrent mutation,
  nested repository, worktree, submodule, symlink, bare, missing, and
  incompatible behavior.
- Define change identity, old/new path optionality, status vocabulary,
  additions, modifications, deletions, type changes, rename/copy detection,
  ambiguity/ties, conflicts, duplicates, canonical order, path encoding and
  confinement, exact/over bounds, and redacted failure alternatives.
- Compare feasible injected library, process, and existing-dependency
  boundaries. Record manifests, lockfile, licenses, unsafe/platform surface,
  executable discovery, environment, stdout/stderr, cancellation, timeout,
  cleanup, and explicit approval requirements for each viable family.
- Define a source-independent Workspace change-input and non-empty equivalence
  oracle against complete filesystem end states without bypassing production
  discovery, parsing, validation, complete rebuild, cache, or atomic
  publication.
- Build deterministic positive, negative, reordered, repeated, concurrent,
  cancellation, recovery, sensitive-data, public-consumer, and cross-platform
  test matrices. Record exact likely production/test areas and every ADR-0060
  decision.
- Keep remote repositories, credentials, fetch/pull/push, repository mutation,
  semantic impact, selective Graph mutation, diagnostics, Rules, refactoring,
  edits, protocol/IDE UI, telemetry, and performance/security claims deferred.

## Excluded scope

Architecture acceptance, Rust or Cargo changes, dependency additions, Git
adapter implementation, repository mutation, production configuration,
Workspace API migration, protocol or IDE capability, semantic impact,
refactoring/edit behavior, Coverage transition, and Sprint completion.

## Validation

Run focused non-mutating source/API/consumer/history/Git/dependency/platform/
test/fixture inventories and existing Workspace/cache/watching/Runtime/public
tests needed to confirm the baseline. Validate Markdown links,
`git diff --check`, Roadmap state, and unrelated-change absence. Record exact
commands, non-zero matched counts, zero-test targets, failed probes, and
inconclusive evidence separately.

## Suggested commit message

`Investigate Sprint 38 Git Change Adapter`

## Final report additions

Report existing authorities and consumers, repository/endpoint/state-layer
alternatives, change/path/order/bound/failure questions, implementation-family
and approval evidence, Workspace equivalence oracles, deterministic matrices,
unresolved ADR decisions, exact affected areas, and unchanged production
behavior.
