# ADR-0060: Git Change Adapter

## Status

Accepted

## Context

Sprint 38 must convert local Git repository change evidence into deterministic
Workspace change inputs without making Git a semantic authority. The
[investigation](../architecture/git-change-adapter-investigation.md) confirms
that the repository already has:

- a private deterministic complete-byte Workspace filesystem state;
- a Runtime-owned latest-revision change source and serialized complete rebuild
  coordinator;
- immutable atomic Workspace publication, last-valid failure retention,
  recovery, cancellation, shutdown, and cache validation;
- tracked mixed EDT/Designer production fixtures and deterministic temporary
  mutation seams;
- complete Graph Diff and Impact behavior after canonical graph construction;
  and
- macOS and Windows Rust CI plus a locally available Git 2.50.1 executable.

No Git domain, repository endpoint, reader, Workspace Git input, production
process adapter, or Git implementation package exists. ADR-0027 explicitly
rejects filesystem and Git events as canonical Graph/index inputs. ADR-0041
intentionally reduces raw filesystem events to a complete-state `Changed`
signal and keeps the complete resulting filesystem state authoritative.

This ADR selects one bounded explicit-demand local first slice. It does not
replace the portable filesystem watcher and does not implement Sprint 39
Change Impact Analysis.

## Decision

### Canonical authority and dependency direction

`oneagent-runtime` owns the first-slice repository-change domain, local Git
reader, mapping into Workspace change input, and integration with the existing
Workspace rebuild coordinator. Keeping the boundary inside Runtime:

- reuses the existing Tokio process, cancellation, lifecycle, cache, snapshot,
  and update-status ownership;
- introduces no Cargo member, manifest dependency, lockfile change, external
  Rust library, native library, or unsafe code; and
- avoids exposing Git-local structures through `oneagent-workspace`, Graph,
  Analysis, protocols, or source adapters before a reusable second consumer
  exists.

The domain is source-independent with respect to Git command framing: public
or crate-visible normalized values contain validated repository endpoints,
paths, statuses, and completeness, not porcelain records, modes, command
arguments, stdout/stderr, or process errors. The production reader is the only
Git-specific adapter.

Git is repository evidence only. It does not own or infer:

- Workspace configuration or metadata identity;
- source parsing, Graph nodes or edges, validation, provenance, locations,
  diagnostics, rules, Graph Diff, or Impact seeds;
- cache identity or semantic compatibility;
- refactoring plans, source edits, repository mutation, or transactions; or
- protocol, IDE, or external-client capabilities.

Production filesystem discovery and EDT/Designer adapters remain source
authorities. Complete validated graphs remain semantic authority.
`SemanticGraphDiff::between(previous, current)` remains canonical only after
two complete graph snapshots exist.

### Explicit-demand first slice

The Git reader is a public Runtime library boundary invoked explicitly by a
caller. Runtime `App`, `RuntimeConfig`, default `WorkspaceService`, CLI, HTTP,
MCP, LSP, VS Code, and EDT do not automatically discover or start Git.

`WorkspaceService` adds one optional bounded source-neutral input handle that a
caller must obtain before registering the service. A caller may:

1. invoke the production Git reader for the configured Workspace root;
2. receive one complete normalized result or typed failure; and
3. submit a non-empty accepted result to the input handle.

The existing filesystem watcher remains active and authoritative for change
detection and complete source-state observation. Git input is supplementary:
it may request a complete rebuild sooner or on explicit demand, but an empty
Git result never proves that the Workspace filesystem is unchanged and never
suppresses filesystem observation.

This composition gives Sprint 39 a deterministic repository-evidence boundary
without changing public protocols or weakening ADR-0041 correctness.

### Repository and Workspace boundary

One reader invocation accepts exactly one caller-supplied Workspace root.
Before invoking Git, Runtime:

- requires an existing directory;
- obtains its canonical filesystem path without following any path below the
  root;
- invokes Git against that root; and
- requires Git's canonical top-level worktree path to equal the canonical
  supplied root exactly.

A Workspace below or above the worktree root is rejected. The caller must
select the exact worktree root. This prevents prefix rewriting, changes outside
the Workspace, and implicit parent or nested repository discovery.

The first slice accepts a normal or linked local worktree when its reported top
level equals the supplied root. The common Git directory is internal process
state and is neither inspected directly nor exposed. Bare repositories,
missing repositories, an unborn `HEAD`, mismatched roots, and repositories
without one readable worktree are typed failures.

Nested repositories are accepted only when the caller supplies that nested
worktree's exact top level as the Workspace root. Parent repositories are not
searched or combined. Worktree creation/removal/management and submodule
content traversal are deferred.

Runtime canonicalizes only the supplied root and Git-reported top level for the
equality check. Normalized change paths remain lexical relative paths; Runtime
does not canonicalize, open, or follow each changed path in the Git reader.
Complete Workspace observation/build retains its existing filesystem behavior.

### Repository endpoints and included state

Every accepted `GitChangeSet` identifies:

- one baseline `GitCommitId`, resolved from `HEAD` at read time; and
- one current endpoint `GitWorktree`, meaning the visible tracked working tree
  plus non-ignored untracked files under the exact accepted worktree root.

`GitCommitId` accepts exactly 40 or 64 lowercase ASCII hexadecimal characters
so SHA-1 and SHA-256 repositories are representable. Ref names, branch names,
tags, reflogs, index paths, absolute roots, and common-directory paths are not
endpoint identity and are not exposed.

The reader compares the resolved `HEAD` tree with the final visible worktree
state. Staged and unstaged tracked changes are folded into that one directional
baseline-to-worktree result; the index is not a separately observable endpoint.
Changes that cancel between index and worktree and leave bytes/mode equal to
`HEAD` produce no normalized record.

Non-ignored untracked files are included as additions. Git-ignored paths are
excluded and the result carries the closed completeness value
`TrackedAndUntrackedNonIgnored`. Because the portable filesystem watcher
remains active, this partial repository vocabulary does not claim complete
filesystem absence. Empty untracked directories are likewise outside Git
change evidence and remain visible only to filesystem observation.

Unmerged index entries or any conflict make the entire read fail with
`ConflictedRepository`; no partial set is returned. Sparse checkout, submodule
content, and ignored-source completeness are not claimed. A changed gitlink or
unsupported entry mode is `UnsupportedEntryKind` rather than an ordinary file
change.

### Stable observation and concurrent mutation

One read performs two complete Git observation passes under one 30-second
overall timeout. Each pass resolves:

1. the canonical worktree top level;
2. the complete `HEAD` object identity;
3. unmerged entries;
4. tracked baseline-to-worktree raw changes with rename/copy detection
   disabled; and
5. non-ignored untracked files.

The reader accepts a result only when both independently parsed normalized
passes are exactly equal, including baseline identity, completeness, ordered
changes, modes, and paths. Any difference is `UnstableRepository`. There is no
third pass or unbounded retry.

This is a bounded point-in-time evidence contract, not a lock on subsequent
repository mutation. The filesystem watcher and complete build stability scan
remain authoritative after submission.

### Normalized path contract

`RepositoryChangePath` is one non-empty UTF-8 path relative to the accepted
worktree root. It:

- uses `/` as its only separator on every platform;
- contains only non-empty normal components;
- rejects absolute roots, drive or UNC prefixes, leading or trailing `/`,
  repeated `/`, `.`, `..`, NUL, and `\`;
- preserves case and Unicode code points without folding or normalization; and
- contains at most 4,096 UTF-8 bytes.

Non-UTF-8 Git output is `UnsupportedPathEncoding`. Two distinct raw paths that
normalize to one accepted string are a conflict, not a first-wins choice.
Errors never expose the rejected path or absolute root.

The reader never opens a normalized path to decide identity or semantics.
Path validation is confinement for evidence publication; complete Workspace
observation retains responsibility for actual file kinds and bytes.

### Change status, identity, and order

The closed `RepositoryChangeKind` vocabulary is:

- `Added`;
- `Modified`;
- `Deleted`;
- `TypeChanged`; and
- `Untracked`.

The first slice disables Git rename and copy similarity detection. A move is
therefore one `Deleted` old path plus one `Added` or `Untracked` current path.
No similarity threshold, score, or inferred object identity enters the domain.
This is deterministic and preserves ADR-0041's resulting-state authority.

`RepositoryChange` carries:

- `kind`;
- `previous_path: Option<RepositoryChangePath>`; and
- `current_path: Option<RepositoryChangePath>`.

The valid matrix is:

| Kind | Previous path | Current path |
| --- | --- | --- |
| `Added` | absent | present |
| `Untracked` | absent | present |
| `Deleted` | present | absent |
| `Modified` | present | present and exactly equal |
| `TypeChanged` | present | present and exactly equal |

Any other combination is invalid. The stable change identity is the complete
`(effective_path, kind, previous_path, current_path)` tuple. `effective_path`
is current when present and previous otherwise.

Normalization sorts by:

1. effective path bytewise;
2. kind in the declared vocabulary order; and
3. previous then current path bytewise.

An exact duplicate collapses. Two non-identical records involving the same
effective path, including tracked/untracked or multiple status disagreement,
fail the complete set with `ConflictingChange`. Encounter, process, hash,
filesystem, and insertion order never select a winner.

The complete `GitChangeSet` identity and equality include baseline commit,
current endpoint kind, completeness, and the ordered normalized changes.
Repository root and process details are excluded from observable equality.

### Bounds

The first slice accepts these fixed non-configurable limits:

- at most 10,000 normalized changes;
- at most 4,096 UTF-8 bytes per normalized path;
- at most 16 MiB stdout per Git command;
- at most 64 KiB stderr per Git command, retained only for bounded internal
  draining and never copied into public errors;
- at most one child process at a time;
- exactly two observation passes; and
- one 30-second timeout for the complete read, including both passes and child
  cleanup.

The operation deadline reserves its final second for child cleanup. A shorter
test-only timeout reserves at most half of its duration. Cancellation can begin
cleanup earlier, and caller drop performs synchronous kill and reap before the
drop completes. One-over values fail before publication. The reader drains
stdout and stderr incrementally under their byte limits and then parses the
bounded NUL-delimited buffers; `wait_with_output` or an equivalent unbounded
collection is not accepted.

These are safety and deterministic-test bounds, not performance guarantees.
Configuration, benchmarks, and tuning are deferred.

### Git process contract

The production adapter uses the caller's `git` executable discovered through
the process environment. Sprint 38 adds no installer and no Cargo dependency.
The current sprint request explicitly targets a local Git adapter; accepting
the already required local Git executable as this adapter's external runtime
precondition does not authorize installing or modifying Git.

The adapter uses only fixed non-shell argument vectors and machine-oriented,
NUL-delimited commands equivalent to:

- `git --no-pager -C <root> rev-parse --show-toplevel`;
- `git --no-pager -C <root> rev-parse --verify HEAD`;
- `git --no-pager -C <root> ls-files --unmerged -z`;
- `git --no-pager -C <root> diff --raw -z --no-renames HEAD --`; and
- `git --no-pager -C <root> ls-files --others --exclude-standard -z`.

Implementation may combine probes only when tests prove byte-for-byte equal
semantics and preserve the same failure categories. Shell execution, command
strings, path interpolation, aliases, hooks, remote operations, and repository
mutation are forbidden.

Every child:

- receives closed stdin;
- has piped stdout and stderr drained concurrently under the byte limits;
- uses `GIT_OPTIONAL_LOCKS=0`;
- disables pager, color, quote escaping, rename detection, filesystem monitor,
  and untracked cache through fixed command options/config overrides when
  applicable;
- inherits no credential input and invokes no command that contacts a remote;
- is owned by the read future with kill-on-drop behavior; and
- is terminated and joined on cancellation, timeout, read-limit failure, or
  caller drop.

Stdout and stderr read futures are owned directly by the same read future and
are never detached tasks. Cancellation, timeout, or a read failure uses the
reserved cleanup interval to kill and asynchronously reap the child. If that
interval is exhausted, the guard synchronously reaps before control can leave
the read future. Dropping the caller future invokes that same synchronous
kill-and-reap fallback, so no process or pipe-reader task outlives its owner.

Locale-dependent human messages are never parsed. Exit code and typed parser
state determine the closed error. The adapter validates behavior by capability,
not a hard-coded Git version, so unsupported porcelain or object formats fail
`IncompatibleGit`.

Tests inject a bounded command runner for spawn/read/exit/cancellation/error
cases and use the real production runner against disposable temporary
repositories for conformance. Ambient global Git user name, email, signing,
hooks, pager, color, editor, credential helpers, and network are not test
oracles. Temporary commits set identity locally and disable signing.

### Errors and sensitive data

The closed public error kinds are:

- `RootUnavailable`;
- `RootNotDirectory`;
- `NotRepository`;
- `WorktreeRootMismatch`;
- `BareRepository`;
- `MissingBaseline`;
- `ConflictedRepository`;
- `UnsupportedEntryKind`;
- `UnsupportedPathEncoding`;
- `InvalidPath`;
- `TooManyChanges`;
- `OutputLimitExceeded`;
- `IncompatibleGit`;
- `UnstableRepository`;
- `SpawnFailed`;
- `ProcessFailed`;
- `TimedOut`; and
- `Cancelled`.

Domain construction has its own closed invalid-identity, invalid-change,
duplicate/conflict, and bound errors where needed. Public `Display` text names
only the kind and accepted bounded counts. It never includes an absolute or
rejected path, repository configuration, ref, branch name, raw object beyond
the accepted baseline ID, source content, command line, environment, stdout,
stderr, OS error, or internal chain. Internal errors may retain a source for
debugging only when doing so does not enter public snapshots, protocols, cache,
or test output assertions.

### Source-independent Workspace input

`WorkspaceService` gains a cloneable `WorkspaceChangeInputHandle` before
registration. The handle owns the sender half of one capacity-one channel;
the running Workspace coordinator owns the receiver. Submission is explicit
and non-blocking.

The handle accepts one already validated `GitChangeSet` and maps only its
ordered non-empty path/status evidence into a private source-neutral rebuild
request. Baseline ID, Git completeness, worktree identity, and Git error data
do not enter `WorkspaceSnapshot`, cache bytes, Graph, diagnostics, update
status, or protocols.

Submission outcomes are closed and deterministic:

- an empty set is `IgnoredEmpty` and queues nothing;
- a non-empty set accepted into the one-slot channel is `Accepted`;
- a full channel is `Backpressure` and queues nothing;
- a closed service is `Closed` and queues nothing.

The caller owns retry after `Backpressure`; Runtime does not spin or block.
Two separate accepted submissions are separate rebuild requests even when
their normalized sets are equal. Exact record duplicates have already been
collapsed within each set.

The coordinator selects both filesystem observations and explicit inputs. A
Git input received while idle starts one complete rebuild through the existing
path. At most one input remains pending while a build runs; after that build,
the pending input causes one follow-up attempt. Filesystem latest-revision
coalescing remains unchanged. If both sources request work, extra complete
rebuilds are permitted but partial publication, source-priority selection, and
lost accepted input are not.

No change paths or source identifier are published in Sprint 38. The public
update observer retains its exact phase, attempt, published, and failure
vocabulary. A Git reader failure occurs before submission and does not mutate
Workspace status. A submitted change followed by build failure uses the
existing source-neutral Workspace failure category and recovery behavior.

### Rebuild, cache, lifecycle, and compatibility

Every accepted non-empty Git input triggers the existing complete production
pipeline:

1. complete filesystem state scan;
2. production Workspace discovery;
3. complete EDT/Designer builds and validation;
4. stable post-build filesystem scan;
5. cache write only for stable state; and
6. atomic immutable snapshot replacement.

Git paths do not select configurations, parsers, nodes, or invalidation. A Git
change that yields an equal complete result may still publish according to the
existing ADR-0041 behavior. Build and observation failures retain the last
valid snapshot and recover only through a later accepted filesystem or explicit
input.

Cache schema remains `1`; semantic compatibility remains `4`. Git baseline,
changes, statuses, paths, process results, and input queue are not serialized.
Warm load remains keyed by complete filesystem source state and recomputes all
derived semantic evidence as already accepted.

The Workspace service owns the explicit-input receiver and closes it during
normal or failed shutdown. Cancellation stops accepting new input, joins any
active blocking build and filesystem source, clears the snapshot, publishes
`Stopped`, and closes observers. The Git reader separately owns and joins its
process before it can submit; no process is attached to the service after
reader completion.

Existing Runtime default construction, health/readiness, Graph Query, HTTP,
CLI, seven-tool MCP catalog and Tool Policy, LSP capabilities, VS Code, EDT,
source adapters, Graph/Analysis APIs, and Coverage Registry remain unchanged.
No public wire schema advertises Git support in Sprint 38.

### Deterministic evidence contract

Task 3 must prove domain behavior without repository I/O:

- valid and invalid commit IDs, paths, status/path matrices, ordering,
  duplicate collapse, conflicting same-path records, exact/over counts and path
  bounds, redaction, reordered construction, and repeated equality.

Task 4 must prove the real reader and injected process boundary:

- normal and linked exact-root worktrees;
- empty, modified, staged, unstaged, combined, added, deleted, type-changed,
  untracked, ignored, moved-as-delete/add, detached-HEAD, and repeated states;
- conflict, unborn HEAD, bare, missing, root mismatch, nested exact-root,
  submodule/gitlink, non-UTF-8 where constructible, unsupported output,
  spawn/exit/read/output-limit/timeout/cancellation, first/second-pass drift,
  production-boundary future-drop and deadline cleanup, local config isolation,
  no network, and sensitive-data redaction;
- equivalent operation order and both SHA-1 plus SHA-256 when the installed Git
  capability supports repository-format selection; unsupported SHA-256 is
  recorded as an environment limitation, not silently skipped.

Task 5 must prove source-neutral Workspace integration over disposable Git
repositories containing copies of the tracked mixed EDT/Designer fixture:

- empty ignored input;
- accepted modification, addition, removal, and move-as-delete/add;
- equal end states created in different orders;
- one-slot backpressure and one accepted follow-up during a gated build;
- complete build, atomic old/new publication, invalid build retention, repair
  recovery, cache cold/warm/change behavior, filesystem/Git source coexistence,
  cancellation, shutdown, closed input, fresh repetition, and receiver/resource
  cleanup;
- unchanged Graph Query, HTTP/CLI, diagnostics/rules, MCP/LSP, adapter, and
  current Coverage behavior.

Tasks 3-5 run non-zero focused targets and the canonical full Rust workspace
gate. Task 6 records the complete matrix, macOS/Windows CI for the exact code
head, API/dependency/license/unsafe/executable/configuration/path/sensitive-
data/scope audits, and current-state documentation. A zero-match filter,
unbounded output path, ambient user repository/config requirement, network
access, unresolved platform row, or incomplete cleanup is not passing evidence.

## Rejected alternatives

### Make Git the canonical Workspace or semantic change source

Rejected. Ignored paths and empty directories are not complete Git change
evidence, and repository statuses cannot identify semantic entities. The
portable complete filesystem watcher and complete production builds remain
authoritative.

### Replace filesystem watching with Git polling

Rejected. It would miss visible Git-ignored or untracked-directory changes and
would make Git/executable availability a prerequisite for existing Runtime
behavior. Explicit supplementary input preserves compatibility.

### Add a Rust Git library

Rejected for the first slice. No package is locked or approved, and selecting
one would add manifest, transitive, license, unsafe/native, and platform surface
without evidence that the bounded process contract is insufficient.

### Add a new Git adapter crate now

Rejected. One Runtime consumer owns process, cancellation, and rebuild
composition. A new crate and local dependency add layering and public API
before repeated reuse exists. Extraction remains possible after evidence of a
second consumer.

### Parse `.git` directly

Rejected. Refs, packed refs, worktrees, indexes, object formats, ignore rules,
and conflicts are not a bounded reimplementation target.

### Enable rename or copy similarity

Rejected. Thresholds and ambiguity add order- and implementation-sensitive
identity. Delete/add is deterministic and compatible with complete resulting-
state authority.

### Expose Git changes through Workspace snapshots or protocols

Rejected. Sprint 38 needs an input boundary, not a persisted or wire Git model.
Sprint 39 may consume the accepted library result through a separately decided
product workflow.

### Infer changed graph nodes directly from paths

Rejected. It bypasses parsing, provenance, validation, complete graph
construction, and ADR-0027 canonical diff derivation.

### Block submission until queue space is available

Rejected. It creates hidden backpressure and lifecycle coupling. One explicit
slot plus `Backpressure` keeps ownership bounded and caller-visible.

## Implementation prerequisites

1. Task 3 adds the normalized domain, invariants, bounds, closed errors, public
   exports, Rustdoc, and focused tests inside `oneagent-runtime` without process
   or Workspace changes.
2. Task 4 adds the injected bounded command runner, production Git reader,
   two-pass stability, parsing, temporary-repository conformance, cancellation,
   timeout, and cleanup without Workspace integration.
3. Task 5 adds the one-slot input handle and coordinator selection, composes
   the production reader explicitly in public tests, and preserves complete
   rebuild/cache/lifecycle/consumer behavior.
4. Task 6 completes exact platform, dependency, API, sensitive-data, scope,
   documentation, and full validation evidence without new behavior.

No Cargo manifest or lockfile change is accepted by this architecture. If the
selected fixed Git commands cannot be implemented portably and safely with the
existing Tokio/std surface, stop and return to architecture; do not add a
library or shell workaround inside implementation.

## Coverage Registry impact

No Coverage Registry capability changes. Git input is repository evidence and
does not add or complete a semantic node, edge, metadata, resolution, query,
validation, or impact capability.

## Consequences

- One deterministic local Git evidence boundary becomes available without
  changing default Runtime behavior or semantic authority.
- The first slice includes tracked final-worktree and non-ignored untracked
  changes relative to pinned `HEAD`, rejects conflicts, disables rename/copy,
  and reports explicit partial repository completeness.
- The filesystem watcher remains the correctness source; Git input can request
  the same complete rebuild through one bounded explicit channel.
- No Git path, status, object, process, or error enters snapshots, cache,
  protocols, diagnostics, Graph, Analysis, or Coverage.
- System Git becomes an explicit precondition only when a caller invokes the
  reader; ordinary Runtime operation remains independent from it.

## Deferred scope

- arbitrary commit-to-commit, tree-to-tree, index-only, and separately exposed
  staged/unstaged endpoints;
- automatic Git polling, repository discovery, multi-repository aggregation,
  configurable limits, persistence, and public Git subscriptions;
- rename/copy similarity, ignored-source completeness, sparse-checkout claims,
  submodule-content traversal, worktree management, and conflict resolution;
- remote repositories, fetch/pull/push, credentials, authentication, hosting
  APIs, network workspaces, and repository mutation;
- incremental Graph/index mutation, path-to-entity inference, semantic impact,
  diagnostics/rules from Git, selective parsing/building, and partial
  snapshots;
- protocol/IDE Git UI, refactoring plans, source edits, safe transactions,
  rollback, code actions, telemetry, benchmarks, and broad performance or
  security claims.
