# Git Change Adapter Investigation

## Purpose

This document records the repository, architecture, implementation,
dependency, platform, consumer, and executable-test evidence required to decide
ADR-0060. It does not select Git Change Adapter architecture, change production
behavior, add a dependency, or treat Git as semantic authority.

## Live baseline

- Investigation head: `89c79c694021c2735d09318de2c5c6d9b8c5d32b`.
- Branch: `codex/v0.7-sprint-38`.
- Sprint 37 review head: `b029544f4e4cd0c7561908f67fccbd0ca79b3b71`.
- Version integration head: `a1434fa03529bc09874001264eb23cc651203bfe`.
- Git Change Adapter framework prerequisite: `7eac8515`.
- Roadmap state before this task: Sprint 37 is `completed`; Sprint 38 is the
  unique `next` target.
- The task started from an empty `git status --short`.
- Local executable evidence: `/usr/bin/git`, version `2.50.1 (Apple Git-155)`.

The current repository is a non-bare worktree whose top level is the OneAgent
repository root. `git rev-parse --git-common-dir` returns `.git`. The live
repository also has other local worktrees, so the main checkout path and the
common Git directory are not interchangeable identities. This fact is
evidence for an explicit repository/worktree contract, not permission to read
another worktree.

## Accepted authority constraints

The following decisions are already authoritative:

- [ADR-0027](../adr/0027-incremental-semantic-index-maintenance.md) accepts only
  complete previous/current `SemanticGraph` snapshots plus their freshly
  derived `SemanticGraphDiff` as canonical incremental-index input. Filesystem,
  Git, Workspace, EDT, and producer events are explicitly not graph/index
  boundary inputs.
- [ADR-0039](../adr/0039-workspace-service.md) assigns production discovery,
  format dispatch, complete build validation, and immutable multi-
  configuration snapshot publication to `oneagent-runtime`. Configuration
  identity is canonical graph identity, not a path or repository object.
- [ADR-0041](../adr/0041-file-watching.md) accepts a Runtime-owned complete-byte
  filesystem observation, one latest-revision signal, serialized complete
  rebuilds, atomic publication, last-valid failure retention, recovery, and
  structured cleanup. Raw create/modify/remove/rename events are deliberately
  not semantic primitives.
- [ADR-0042](../adr/0042-persistent-cache.md) keys the private cache by exact
  complete Workspace source state and publishes only validated complete
  snapshots. Git version or repository metadata does not replace its explicit
  schema and semantic compatibility boundary.
- `SemanticGraphDiff`, `SemanticGraphBuildDiff`, and `SemanticImpactAnalyzer`
  operate only after complete graph construction. Sprint 38 must not seed or
  reinterpret them from repository paths or Git statuses. Sprint 39 owns the
  product-facing Change Impact Analysis expansion.

Git may therefore identify bounded source evidence that can request accepted
Workspace work. It cannot identify a semantic entity, graph operation,
diagnostic, rule result, impact seed, cache entry, refactoring, or edit by
itself.

## Existing Git inventory

### Production code and dependencies

No production Rust module defines a Git repository, endpoint, object ID,
status, change set, reader, process adapter, or Workspace Git input. Repository
search found only these production-adjacent Git references:

- `WorkspaceFileState` and `FileSystemWorkspaceDetector` both exclude
  descendants of `.git` together with `.idea`, `.vscode`, `target`, and
  `node_modules`;
- a Runtime test mutates `.git/transient` to prove ignored-directory behavior;
- the cache source comment states that package and Git versions do not replace
  the manual semantic-compatibility version;
- the VS Code lockfile contains transitive `hosted-git-info` packages used by
  Node tooling, not a OneAgent Git adapter.

`Cargo.toml`, `apps/runtime/Cargo.toml`, and `Cargo.lock` contain no `git2`,
`gix`, `libgit2`, or other Git implementation package. Runtime already depends
on Tokio with `full` features and uses `tempfile` only as a dev dependency.
Adding an external Rust Git dependency would be a new production dependency
and requires an explicit version/features/license/unsafe/platform audit and
approval before a manifest change.

Production code currently starts no child process. Repository process seams
are test-only: CLI tests use `std::process::Command`, while MCP and LSP public-
process tests use `tokio::process::Command` with owned stdio, bounded timeouts,
exit checks, and temporary roots. Those seams prove feasibility but do not
select a production Git process contract.

### Governance Git use

`AGENTS.md`, sprint prompts, reviews, and workflows use Git for repository
governance, branch safety, validation, commits, pushes, and immutable review
ranges. That usage runs under Codex/user control and is not a production API,
fixture format, or Runtime dependency. Historical commit subjects and branch
names are not an application-facing change vocabulary.

### CI and platform evidence

The Rust CI matrix runs on `macos-14` and `windows-latest`, begins with
`actions/checkout@v7`, and runs format, check, public Runtime builds, all
workspace tests, focused Context/MCP tests, Clippy, and Rustdoc. The checkout
step makes Git available to the job, but the workflow does not pin a Git
version or prove identical command behavior across the two runners. Any
process-backed first slice must add executable conformance evidence on both
operating systems rather than extrapolating from local Apple Git 2.50.1.

## Existing Workspace change pipeline

### Complete source state

`apps/runtime/src/workspace/change.rs` owns a private
`WorkspaceFileState { BTreeMap<PathBuf, WorkspaceFileEntry> }`.
`WorkspaceFileEntry` is `Directory`, `RegularFile(Vec<u8>)`, or `Other`.
`scan`:

1. validates that the configured root exists and is a directory;
2. recursively enumerates children in sorted path order;
3. retains relative paths, entry kinds, and complete regular-file bytes;
4. does not follow non-directory entries;
5. excludes the complete `.oneagent` cache subtree and descendants of the five
   accepted ignored directory names; and
6. returns typed root, directory, entry, file-type, and file-read failures.

The state is private to Runtime. Cache encoding converts relative components to
UTF-8 and rejects non-relative or non-UTF-8 components. This does not yet define
the Git adapter path contract; ADR-0060 must either align with that accepted
boundary or define an explicit pre-cache rejection/migration.

### Observation and normalized signal

`WorkspaceChangeSource` retains the last successful `WorkspaceFileState` and
polls every 250 milliseconds in production. A capacity-one Tokio watch channel
publishes a monotonically increasing revision with only:

- `Changed`; or
- `ObservationFailed(WorkspaceObservationErrorKind)`.

Equal scans publish nothing. The first failure publishes one typed failure,
equivalent repeated failures are suppressed, and the first later successful
scan conservatively publishes `Changed`. Revision overflow is terminal.
Controlled test ticks make equal, changed, failure, recovery, cancellation, and
fresh repetition deterministic.

There is no path-level public change value and no port that accepts a supplied
change set. The source, observation, and file state are all `pub(super)`.

### Build, publication, failure, and cleanup

`WorkspaceService` performs initial scan/build/scan race closure, then owns the
change source, update coordinator, cache, snapshot sender, status sender,
blocking complete builds, and cancellation. For each newer `Changed` revision
it:

1. increments `attempt`, publishes `Rebuilding`, and starts one complete build;
2. observes source changes concurrently through the latest-value channel;
3. scans before and after the build and writes cache only for a stable state;
4. atomically replaces the snapshot only after complete successful build and
   validation; and
5. publishes `Watching` or a typed recoverable `Failed` state.

A failed observation or build retains the last valid snapshot. A later change
may recover. Cancellation joins source/build work, clears publication,
publishes `Stopped`, closes observers, and preserves Runtime structured
cleanup. Any Git integration must preserve this ownership or explicitly
migrate it in ADR-0060.

### Public consumers

The public source-neutral surface exports only `WorkspaceSnapshotObserver` and
`WorkspaceUpdateObserver` plus phase, counters, and closed failure kinds.
Graph Query, HTTP, CLI, MCP, LSP, VS Code, and EDT consumers acquire immutable
published snapshots or existing protocol projections. They do not observe
filesystem paths, watcher revisions, Git endpoints, statuses, branches, or
object IDs. No existing public schema or capability needs Git data to preserve
current behavior.

## Repository and Workspace boundary questions

ADR-0060 must choose every applicable row rather than combine them implicitly.

| Question | Evidence-supported alternatives | Required decision |
| --- | --- | --- |
| Repository root | Exact configured Workspace root is the worktree root; discover one containing worktree; or receive an explicit repository root separate from Workspace | Ownership, validation, relationship, and failure for no repository or nested repository |
| Worktree identity | Canonical worktree path plus common directory; repository object plus selected worktree; or a validated caller-owned opaque handle | Equality, aliases, linked worktrees, and path disclosure |
| Baseline endpoint | One committed tree/object ID; resolved symbolic ref with pinned object ID; index snapshot; or another investigation-backed immutable endpoint | Resolution time, identity, missing/ambiguous/moving behavior |
| Current endpoint | One committed tree; index; working tree; or an explicit composite of index, worktree, and untracked state | Completeness and stability across reads |
| Workspace relationship | Repository equals Workspace; Workspace is confined below repository; or repository is confined below Workspace | Prefix mapping, changes outside Workspace, and nested boundaries |
| Bare repository | Reject; or support tree-to-tree evidence without Workspace files | First-slice applicability and typed result |
| Linked worktree | Reject; or accept with explicit worktree/common-dir identity | Metadata location, endpoint resolution, and tests |
| Submodule/nested repository | Treat entry as an opaque path change; reject; or support only metadata pointer changes | Traversal and authority boundary |
| Symlink/path alias | Preserve lexical confined path; canonicalize under an accepted rule; or reject ambiguous input | Escape prevention and cross-platform behavior |
| Concurrent mutation | Fail one stability check; retry a bounded number; or accept one explicitly identified snapshot | Complete-result oracle and failure classification |

No existing accepted document chooses these rows.

## State-layer and completeness questions

A local repository can expose distinct committed tree, index, worktree,
untracked, ignored, and unmerged evidence. A first slice must name included
layers and reject or deliberately omit every other layer.

| Layer or case | Current evidence | ADR-0060 question |
| --- | --- | --- |
| Commit/tree to commit/tree | Git can provide immutable object endpoints | Is this required, optional, or outside the local Workspace first slice? |
| HEAD/tree to index | Staged state differs from working files | Is index-only evidence meaningful to Workspace rebuild input? |
| Index to worktree | Unstaged tracked changes | Does the adapter represent this layer separately or only a composite end state? |
| Baseline tree to worktree | Matches the complete visible Workspace better but excludes untracked files by ordinary diff commands | How are staged and unstaged changes combined without duplicates? |
| Untracked files | Visible to filesystem discovery/build but absent from a tree diff | Include through a separate bounded enumeration or classify the result incomplete |
| Ignored files | ADR-0041 ignores only confirmed directory families, not arbitrary `.gitignore` rules | Do Git-ignored source files count as Workspace input or remain filesystem-authoritative? |
| Unmerged/conflicted index | Multiple stages do not describe one accepted complete file transition | Reject, represent a closed conflict status, or define a complete current-file policy |
| Sparse checkout | Visible worktree may be intentionally incomplete relative to the tree | Reject, accept current visible state, or require explicit completeness metadata |

The existing complete filesystem state is authoritative for cache and build
stability. A Git-derived result cannot claim complete Workspace equivalence if
its chosen layers omit a visible relevant file without recording that
limitation.

## Normalized change questions

ADR-0060 must define a source-independent domain before repository I/O.
Decision points are:

- validated repository/worktree and endpoint identities, and which fields are
  observable rather than internal;
- one closed status vocabulary for accepted additions, modifications,
  deletions, type changes, rename or copy candidates, conflicts, untracked
  paths, and deliberately ignored/unsupported cases;
- exact old/new path optionality: additions have no old path, deletions have no
  new path, while a rename/copy candidate needs both only if the first slice
  accepts similarity evidence;
- whether rename/copy detection is disabled and represented as delete/add, or
  enabled with a fixed algorithm/threshold, ambiguity/tie behavior, and
  reproducibility contract;
- path bytes versus platform strings versus validated UTF-8 forward-slash
  components, including absolute paths, `..`, `.`, empty components, separators,
  case, Unicode, and non-text values;
- identity and total order independent from Git/process/traversal order;
- duplicate identical observations, contradictory statuses, same path in
  multiple included layers, case/normalization collisions, and endpoint drift;
- exact path-count, path-length, endpoint, object/output, retry, timeout, and
  error-detail bounds; and
- closed redacted failures that never echo absolute roots, config, credentials,
  source bytes, raw stdout/stderr, environment, or internal chains.

Path or status equality cannot imply semantic entity identity. Rename
similarity, if accepted, is repository evidence only.

## Feasible implementation families

### Existing-executable process adapter

The installed Git CLI exposes machine-oriented NUL-delimited status/diff
families, and Tokio already provides process, async I/O, cancellation, and
timeout primitives. This family can use an injected command runner for focused
tests and real temporary repositories for production-entry evidence without a
new Cargo package.

ADR-0060 would still need to accept a new production executable dependency and
define discovery, supported version/capability checks, fixed arguments,
working directory, environment isolation, config/color/pager suppression,
stdin closure, stdout/stderr bounds, encoding, exit/signal mapping,
cancellation, timeout, process-tree cleanup, Windows executable behavior, and
concurrent mutation. The local version does not prove the Windows runner.

### Rust Git library

A library can avoid raw process framing and make object/status APIs typed, but
no candidate is locked. Adding one changes Cargo manifests and lockfile and
requires explicit approval plus version, features, transitive dependency,
license, unsafe/native surface, toolchain, and macOS/Windows evidence. The
investigation contains no basis to select `git2`, `gix`, or another package.

### Direct `.git` parsing

Implementing refs, packed refs, indexes, object storage, worktrees, ignore
rules, conflicts, and platform path behavior with `std` alone would create a
second partial Git implementation and lacks repository evidence. It is not a
bounded credible first-slice candidate.

### Injected domain-only reader

An injected reader can prove domain and Workspace mapping without selecting a
real source, but it cannot satisfy the Sprint 38 production-reader objective by
itself. It remains a deterministic test seam for either accepted production
family.

## Workspace change-input alternatives

The current coordinator accepts only its private latest-revision observation.
ADR-0060 must choose one source-independent boundary, for example:

1. replace the private observation with a trait/enum whose filesystem and Git
   implementations both emit a bounded source-neutral `Changed`/failure
   contract;
2. retain filesystem observation and add an explicit one-shot Git change input
   that is normalized before entering the same rebuild coordinator; or
3. compose multiple sources under one Runtime owner with a single latest-value
   revision and explicit precedence/coalescing.

The first slice need not publish paths publicly. It must define whether an
empty Git change requests no build, how a non-empty or conflict result maps to
`Changed` or failure, how duplicate filesystem/Git evidence coalesces, and how
startup, cancellation, source termination, and recovery work.

The minimum equivalence oracle is end-state based:

- create two disposable copies of the tracked mixed EDT/Designer fixture;
- initialize equivalent local repositories where required;
- produce the same relevant complete file end state through different Git and
  filesystem operation orders;
- run production discovery, both adapters, complete validation, cache policy,
  and immutable publication; and
- compare canonical Workspace snapshots, update outcomes, Graph Query or
  another supported consumer result, failure/recovery, and fresh repetition.

Parser-only equality, path-list equality, or an injected event without a real
production reader is insufficient. Equivalent adapter provenance need not be
identical when accepted source-independent results are equal.

## Repository-owned deterministic evidence matrix

The tracked `apps/runtime/tests/fixtures/workspace_service/` fixture has a
documented SHA-256 inventory and separately discoverable EDT and Designer XML
projects. Existing tests already copy it into `tempfile::TempDir` before every
mutation, so temporary Git initialization can preserve tracked source
provenance and never mutate the repository checkout.

| Case | Constructible evidence | Observable oracle |
| --- | --- | --- |
| Empty/equal | Equal accepted endpoints and unchanged worktree | Empty complete change set and no unclaimed semantic work |
| Added/modified/deleted/type change | Temporary tracked and untracked source mutations | Exact normalized changes plus complete Workspace end-state result |
| Rename/copy | Equivalent content/path operations with detection enabled or disabled as accepted | Stable accepted candidate or deterministic delete/add policy |
| Conflict/unmerged | Temporary branches and deterministic merge conflict | Closed accepted conflict result; never a guessed complete semantic transition |
| Ignored/untracked | Repository ignore rules and files inside/outside ADR-0041 ignored directories | Accepted layer classification plus truthful Workspace completeness |
| Reordered | Build equivalent repository states through different command/file orders | Equal normalized set and Workspace result |
| Duplicate/contradictory | Injected raw-reader observations | Exact collapse or typed failure independent from encounter order |
| Path confinement | Nested, absolute, traversal-like, separator, non-text, symlink, and outside-Workspace candidates | Confined normalized path or typed rejection before publication |
| Endpoint drift | Move a symbolic ref or mutate index/worktree between controlled reader phases | Accepted stable snapshot, bounded retry, or typed unstable failure |
| Exact/over bounds | Generated temporary path/change/output counts | Exact succeeds; one-over fails before unbounded allocation/publication |
| Process/library failure | Missing executable or injected spawn/read/exit/signal/parse failure, or library error | Closed redacted error and complete cleanup |
| Cancellation | Gate reader/process and cancel at every accepted phase | No detached task/process or partial result |
| Invalid build/recovery | Git-visible corruption followed by repair | Last valid snapshot retained, later complete publication succeeds |
| Cache | Cold/warm and changed-source runs | Existing validation/invalidation/recovery and equal complete snapshots |
| Public consumers | Mixed-fixture Workspace, Graph Query, HTTP/CLI/MCP/LSP regressions | Existing schemas/capabilities/results remain truthful and atomic |
| Fresh repetition | Recreate repositories and processes | Equal normalized and consumer results; resources released |

Supported-platform acceptance requires the production reader and this matrix's
applicable subset on both macOS and Windows CI. No network, credentials, user
repository, global Git configuration, or path outside disposable test roots is
required.

## Executed baseline

The following existing focused commands passed at the investigation head:

```text
cargo test -p oneagent-runtime --lib workspace::change::tests
  5 passed; 94 unrelated filtered
cargo test -p oneagent-runtime --test workspace_service
  6 passed
cargo test -p oneagent-runtime --test file_watching
  2 passed
cargo test -p oneagent-runtime --test persistent_cache
  4 passed
```

These 17 tests prove complete-state scanning, ignored/cache directories,
failure/recovery, cancellation, deterministic fresh runs, production discovery
and adapter builds, immutable publication, public watching, and cache behavior.
They do not prove any Git adapter behavior.

An initial broad command,
`cargo test -p oneagent-runtime workspace::change::tests -- --list`, was
interrupted after it had listed the five matching library tests and several
zero-test binary/unrelated targets. It is not acceptance evidence. The corrected
`--lib` command above executed all five matching tests successfully. The
zero-test Runtime binary entry points are not treated as test evidence.

## Likely affected areas

Exact ownership is an ADR decision, but current evidence confines likely work
to:

- a source-independent change domain in an existing or new accepted Workspace
  layer;
- one local Git reader module or adapter and its focused temporary-repository
  tests;
- `apps/runtime/src/workspace/change.rs` and
  `apps/runtime/src/workspace/mod.rs` only if the accepted source-neutral input
  enters the existing coordinator there;
- `apps/runtime/src/workspace/cache.rs` only when accepted source-state or
  compatibility behavior changes;
- `apps/runtime/tests/workspace_service.rs`, `file_watching.rs`,
  `persistent_cache.rs`, and affected public-consumer tests for integration;
- Cargo manifests/lockfile only if ADR-0060 selects a new crate or external
  library; and
- README, Architecture, Semantic Model, Roadmap, evidence, and review documents
  for truthful current state.

No graph model, parser, source-adapter semantic mapping, diagnostics/rules,
Coverage registry, MCP tool catalog, LSP capability, VS Code/EDT UI, or edit
module is inherently required.

## ADR-0060 decision checklist

ADR-0060 must decide or explicitly defer:

1. canonical owner and dependency direction for domain, reader, Workspace
   input, Runtime orchestration, and consumers;
2. repository root, worktree identity, Workspace relationship, nested,
   linked-worktree, bare, submodule, and symlink behavior;
3. baseline/current endpoint types, resolution, equality, included state
   layers, completeness, and concurrent mutation;
4. normalized change identity, statuses, old/new paths, rename/copy policy,
   conflicts, untracked/ignored behavior, duplicates, and total order;
5. path encoding, separators, confinement, case/Unicode collisions, bounds,
   and redacted failures;
6. process, library, or other production family with exact dependency,
   configuration, I/O, cancellation, timeout, cleanup, and platform contract;
7. source-independent Workspace input, empty/failure mapping, source
   composition, coalescing, complete rebuild, publication, and recovery;
8. cache key/compatibility/recompute behavior and unchanged public consumers;
9. repository-owned focused, production-entry, public-consumer, macOS/Windows,
   dependency, API, sensitive-data, and complete-workspace evidence; and
10. first production slice, rejected alternatives, and explicit Sprint 39–41,
    remote, credential, repository-mutation, protocol/UI, telemetry, benchmark,
    performance, and security deferrals.

## Decision readiness

The repository contains enough accepted authority, real local Git history,
available executable evidence, complete production Workspace fixtures,
deterministic temporary mutation seams, failure/recovery/lifecycle ownership,
cache and public-consumer oracles, and macOS/Windows CI to decide and test one
bounded local first slice. No external data, live remote, credential, user
repository, or speculative source format is required.

Task 2 may proceed. It must stop if it cannot select an implementation family
without an unresolved dependency approval, cannot define complete included
state layers, cannot confine every observable path, or cannot preserve complete
Workspace and canonical Graph authority.

## Deferred scope

- remote repositories, fetch/pull/push, credentials, authentication, hosting
  providers, and network workspaces;
- repository mutation, staging, committing, branch/worktree management,
  checkout, merge, rebase, reset, cleanup, and conflict resolution;
- incremental Graph/index mutation, changed-entity inference, semantic impact,
  diagnostics or rules derived from Git, selective parsing/building, and
  partial publication;
- refactoring plans, source edits, safe transactions, rollback, code actions,
  mutable documents, and Git UI/protocol surfaces;
- telemetry, benchmarks, and broad performance or security claims.
