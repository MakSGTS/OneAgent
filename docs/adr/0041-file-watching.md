# ADR-0041: File Watching and Workspace Rebuilds

## Status

Accepted

## Context

Sprint 19 must detect changes below the configured Workspace root and connect
them to Runtime update orchestration. The repository already has the semantic,
build, publication, lifecycle, and consumer authorities:

- `FileSystemWorkspaceDetector` discovers bounded EDT and Designer XML roots;
- `WorkspaceSnapshotBuilder` performs one deterministic complete production
  discovery/build/validation and returns no partial result on failure;
- `WorkspaceService` publishes one immutable `Arc<WorkspaceSnapshot>` through a
  Tokio watch channel and clears it during owned shutdown;
- ADR-0037 owns ordered service lifecycle, cancellation, task joining, and
  terminal failures;
- ADR-0038 makes Runtime lifecycle the sole health/readiness authority;
- ADR-0040 makes Graph Query requests observe one immutable snapshot without
  publication authority or mutable graph access.

The [Sprint 19 investigation](../architecture/file-watching-investigation.md)
confirms production EDT and Designer XML inputs, tracked public fixtures,
temporary mutation oracles, macOS and Windows CI, and deterministic
channel/watch test seams. It also confirms that no filesystem notification
crate is locked, Tokio has no native filesystem watcher, raw platform event
semantics are unaccepted, and the current service performs only one initial
build.

This ADR must fix watcher ownership, watched scope, change normalization,
coalescing, rebuild scheduling, publication, failure, recovery, lifecycle,
dependency, compatibility, and test contracts before production changes.

## Decision

### Semantic authority and dependency direction

Filesystem observation is an invalidation hint, not semantic evidence. Only a
successful complete `WorkspaceSnapshotBuilder` result may replace the published
snapshot. The watcher never creates, removes, repairs, identifies, or interprets
graph facts and never copies adapter source rules into Runtime.

`WorkspaceService` remains the single Runtime owner of the configured root,
complete builder, snapshot sender, change source, rebuild coordinator, blocking
build handles, update-status sender, and shutdown cleanup. Sprint 19 extends
that existing service rather than registering a separately ordered watcher
service.

The dependency direction is:

```text
configured Workspace root
    -> Runtime-owned filesystem state source
        -> bounded change signal
            -> WorkspaceService rebuild coordinator
                -> WorkspaceSnapshotBuilder
                    -> production detector and adapters
                -> atomic snapshot publication
                    -> WorkspaceSnapshotObserver
                        -> GraphQueryService
```

Lower-level graph, metadata, Workspace-domain, detector, and source-adapter
crates do not depend on Runtime or watcher types. Graph Query and HTTP remain
read-only consumers.

### Dependency choice

The first slice uses a portable polling/rescan source implemented with existing
`std` filesystem APIs and the existing direct Tokio dependency. It adds no
production dependency and no Cargo manifest or lockfile change.

The production polling period is exactly 250 milliseconds. It is an internal
first-slice scheduling constant, not a public API, persisted value, health
threshold, performance guarantee, or test oracle. Focused tests inject explicit
scan ticks and coordination gates; they do not wait for the production period.
Public real-filesystem tests may use bounded timeouts only as hang guards and
must await accepted change, update-status, or snapshot observations.

Native notification crates and hand-written macOS/Windows backends are
deferred. A later accepted ADR may replace the source behind the same normalized
boundary without changing rebuild, publication, health, or Graph Query
contracts.

### Watched boundary and filesystem state

The source observes the configured Workspace root recursively. It does not
canonicalize, create, rewrite, or escape that root. Relative paths in the
filesystem state are relative to the configured root and are observation keys,
not semantic identities.

At every directory depth, entries below a directory named exactly `.git`,
`.idea`, `.vscode`, `target`, or `node_modules` are excluded. This matches the
only repository-confirmed discovery exclusions and provides the accepted
irrelevant-change oracle. No extension allow-list is accepted: every other
regular file can be an input to current or future production readers and is
therefore conservatively observed.

One scan produces a sorted owned state containing, for every non-excluded entry:

- relative path bytes through the platform `OsStr` representation retained in
  `PathBuf` ordering;
- entry kind: directory, regular file, or symlink/other;
- complete bytes for each regular file.

The source does not follow symlinks and does not read a symlink target. A
symlink or other entry is represented by its relative path and kind so
creation, removal, or replacement changes the state. Directory entries remain
in the state, so empty-directory add/remove is conservatively relevant.

The state is an internal equality value. It has no stable hash, serialized
schema, generation identity, path case-folding, Unicode normalization, modified
time dependency, or public enumeration API. Reading complete bytes avoids
coarse timestamp and equal-length-content misses. Scan cost is intentionally
unoptimized in the bounded correctness slice and makes no performance claim.

Missing/non-directory roots, unreadable directories or entries, file-type
failures, and regular-file read failures are typed observation errors. A file
that changes between enumeration and read may produce a changed state or a scan
failure; it never produces semantic evidence directly.

### Normalized change and failure vocabulary

The private source-to-coordinator boundary publishes a monotonically increasing
observation revision and only these outcomes:

- `Changed`: the latest successful state differs from the last successful
  observed state, or a successful scan follows an observation failure and a
  conservative rescan is required;
- `ObservationFailed`: one scan could not produce a complete state;
- cancellation/closure through the owned channel and task lifecycle.

Raw create, modify, remove, rename, duplicate, reorder, overflow, platform
cookie, and backend event kinds are not public or internal semantic contracts.
Rename is the removal/addition effect in two complete states. A later native
source must normalize its raw events to `Changed` or `ObservationFailed` and
must request a full rescan after overflow.

The source records a new successful state before publishing its `Changed`
revision. Duplicate scans with equal state produce no revision. The first scan
failure publishes one `ObservationFailed` revision; repeated equivalent
failures do not create an update loop. The source retains the last successful
state, marks rescan required, and keeps polling. The first later successful scan
publishes `Changed` even when equal to the retained state, because changes may
have occurred while observation was incomplete.

### Startup race closure

The service must not lose a persistent change made during its initial build:

1. perform a complete baseline scan;
2. perform the existing complete initial build in one owned blocking task;
3. perform a complete post-build scan;
4. publish the successful initial snapshot;
5. initialize the long-running source with the post-build state;
6. publish one immediate `Changed` revision when baseline and post-build states
   differ.

Baseline or post-build scan failure is a service-start failure and publishes no
snapshot. The existing initial build failures remain service-start failures.
This first slice does not claim that a build reads an operating-system atomic
filesystem transaction; the before/after comparison guarantees a follow-up
complete build when a persistent observable state changed during startup.

### Bounded coalescing and scheduling

The long-running source and coordinator communicate through one private Tokio
watch channel. Its single retained value contains only the latest observation
revision and outcome; it does not queue raw events or filesystem states. The
coordinator records the last processed revision. Channel closure terminates the
source normally only during owned shutdown.

At most one complete rebuild runs for the configured Workspace. The coordinator
receives one signal, marks an attempt, and starts one `spawn_blocking` build.
The change source continues scanning while that build runs. After the attempt,
the coordinator compares the latest source revision with the processed
revision. One or more newer `Changed` revisions cause exactly one immediate
follow-up build against the latest filesystem state. A latest
`ObservationFailed` revision records failure and defers building until a later
successful `Changed` revision. Intermediate revisions coalesce into the one
latest retained value.

There is no debounce timer in addition to the polling period, unbounded queue,
parallel build, per-file build, retry loop, or dropped in-flight handle.

### Atomic publication and reader consistency

Every successful rebuild returns one complete validated `WorkspaceSnapshot`.
The coordinator wraps it in a new `Arc` and replaces the watch value with one
`send_replace`. Every successful rebuild is published, including a semantically
equal result, because Sprint 19 defines no stable complete-snapshot equality or
persistent generation scheme.

The old publication remains unchanged while a rebuild is pending or running.
A retained old `Arc` remains valid after replacement. A new observer read sees
the complete old or complete new value; no reader can observe an in-progress or
partially built snapshot. A Graph Query request continues to hold exactly the
one `Arc` it obtained at request start and its Sprint 18 schema and errors do not
change.

### Post-start failure and recovery

Post-start failures are recoverable update failures, not Runtime service
termination:

- `ObservationFailed` retains the last valid snapshot, publishes no replacement,
  records typed failed update status, and the source continues polling;
- a `WorkspaceBuildError` or blocking build join failure retains the last valid
  snapshot, publishes no replacement, records typed failed update status, and
  waits for a later `Changed` signal;
- no automatic retry occurs against an unchanged successfully scanned invalid
  state; a later filesystem change is required;
- recovery begins with the next accepted signal and publishes only a later
  successful complete build.

This policy preserves query availability during a recoverable source edit while
making the failed attempt observable. Persistence, restart recovery, and cache
invalidation remain Sprint 20.

### Update-status observation

`WorkspaceService` exposes a cloneable transport-neutral
`WorkspaceUpdateObserver` before registration, parallel to snapshot
observation. It provides the current owned `WorkspaceUpdateStatus` and a Tokio
watch subscription.

The closed public status contains:

- `attempt`: monotonically increasing `u64`, starting at zero before work and
  incremented before the initial build and each rebuild;
- `published`: monotonically increasing `u64`, starting at zero and incremented
  only after a complete snapshot publication;
- `phase`: `Starting`, `Watching`, `Rebuilding`, `Failed`, or `Stopped`;
- `failure`: absent except in `Failed`, where it is one stable source-neutral
  kind: `Observation`, `Discovery`, `UnsupportedFormat`, `SemanticBuild`,
  `GraphValidation`, `InvalidConfigurationCardinality`,
  `DuplicateConfigurationIdentity`, or `BuildTask`.

Initial construction exposes `Starting` with zero counters and no failure.
Immediately before the initial build it publishes `Starting` with attempt one.
Successful initial publication sets published one and phase `Watching`.
Before each rebuild it increments attempt and sets `Rebuilding`; success
increments published and sets `Watching`; recoverable failure retains published
and sets `Failed`. A later rebuild moves `Failed -> Rebuilding -> Watching` on
success. Owned shutdown sets `Stopped` after clearing the snapshot and before
the service task returns.

Status is diagnostic/test evidence only. It is not HTTP, health, readiness,
persistence, cache generation, snapshot identity, or a stable wire schema.
Counter overflow is practically unreachable in this slice and uses checked
increment; overflow is a terminal internal service failure rather than wrap.

### Lifecycle, cancellation, and shutdown

The service acknowledges startup only after baseline scan, successful initial
build, post-build scan, initial publication, and change-source construction.
Runtime becomes `Running` only after all registered services acknowledge as in
ADR-0037. A recoverable post-start update failure does not change Runtime
lifecycle or readiness.

The extended Workspace service task is the structured owner of the source task,
single-value change channel, coordinator loop, and every blocking build handle. On
cancellation it:

1. stops scheduling new rebuilds and closes the coordinator receiver;
2. requests source termination and joins the source task;
3. if a blocking rebuild has already started, awaits its join but never
   publishes its result after cancellation;
4. ignores any newer unprocessed change revision;
5. clears the snapshot with one `send_replace(None)`;
6. publishes terminal `Stopped` status;
7. returns only after all owned work and channels terminate.

Tokio blocking work is cooperative only at its owner boundary and is not
aborted or detached. No shutdown timeout, forced termination, restart, or
post-cancellation publication is accepted. Existing reverse service cleanup
keeps HTTP ahead of Workspace during startup and behind it during shutdown, so
the Sprint 16/18 lifecycle gates remain unchanged.

Unexpected source-task panic or coordinator/channel invariant failure is a
named Runtime service failure followed by complete ADR-0037 reverse cleanup.
Normal source closure is accepted only after Workspace cancellation.

### Compatibility contract

- `RuntimeConfig` constructors, root handling, HTTP address, and public health
  and Graph Query wire APIs do not change.
- `WorkspaceService::new`, `Default`, `snapshot_observer`,
  `WorkspaceSnapshotBuilder`, snapshot shapes, build errors, and graph semantics
  remain source-compatible.
- Sprint 19 may add public update-observation types and one
  `WorkspaceService::update_observer` method without changing existing callers.
- Production composition retains `http` before `workspace`; no new registered
  service, listener, route, protocol crate, global registry, or mutable
  `AppState` flag is added.
- The polling implementation and state inventory are private and replaceable;
  only complete publication and update-status behavior are public Rust
  contracts.

## Deterministic evidence contract

Focused tests use an injected state source or explicit scan-tick port and
channels to prove:

- sorted recursive state and ignored-directory behavior;
- regular-file content, path, directory, and symlink/other changes;
- equal scans, changed scans, duplicate/burst coalescing, observation failure,
  rescan recovery, and closed-channel termination;
- startup before/after race closure;
- one active build, one bounded follow-up, no concurrent build, and latest-state
  execution;
- successful publication, equal-result republication, invalid-build retention,
  later recovery, checked counters, cancellation during idle and in-flight
  build, terminal status, channel closure, and repeated fresh ownership.

Public integration tests use disposable copies of the tracked Runtime EDT and
Designer XML fixture and the public `oneagent_runtime` surface. They must prove:

- production observation and complete rebuild for exact accepted EDT and
  Designer XML mutations;
- project add/remove or marker transition and rename-equivalent resulting-state
  behavior;
- a change inside a confirmed ignored directory does not trigger an attempt;
- burst coalescing and a change during a gated build;
- corrupt source failure retains the prior queryable snapshot and exposes
  typed `Failed`, followed by repair and successful publication;
- Graph Query observes complete old or new values with unchanged wire schema;
- lifecycle-derived health remains exact; shutdown closes snapshot and update
  observations, joins every task, releases HTTP, and permits equal fresh runs.

Timeouts are hang guards. Fixed sleeps, modification-time assumptions, raw
platform event ordering, ignored local corpora, network filesystems, and
external services are not acceptance evidence. The complete matrix must pass
on both repository CI operating systems.

## Rejected alternatives

### Add a native notification dependency now

Rejected for the first slice. No crate/version/API is locked or approved, while
bounded polling is implementable and testable on both current CI platforms.

### Implement macOS and Windows native backends directly

Rejected. It would add large platform-specific and unsafe surface, conflict
with `unsafe_code = "forbid"`, and exceed the bounded correctness slice.

### Watch only initially discovered project roots

Rejected. It would miss creation of a new supported project or marker change
elsewhere under the configured root.

### Maintain a source-extension allow-list in Runtime

Rejected. It would copy adapter knowledge, miss future accepted artifacts, and
create a second source-format authority.

### Use modified time and file length only

Rejected. Coarse timestamps and equal-length content changes can be missed and
make deterministic tests platform-dependent.

### Run one build for every raw change

Rejected. Raw event multiplicity is platform-specific and would create
unbounded duplicate work. Capacity-one coalescing retains bounded latest-state
correctness.

### Publish `None` while rebuilding

Rejected. It would make readers observe avoidable unavailability and discard a
known valid immutable snapshot before a replacement exists.

### Terminate Runtime on every post-start build failure

Rejected. Invalid intermediate source edits are recoverable and the last valid
snapshot remains safe to query. Startup failures remain fatal.

### Retry a failed unchanged tree automatically

Rejected. It creates an unbounded hot loop without new evidence. A later change
or observation recovery is the retry trigger.

### Derive readiness from update status

Rejected. ADR-0038 makes lifecycle the sole readiness authority. Failed update
status is diagnostic and the retained snapshot remains queryable.

### Mutate graphs or indexes incrementally

Rejected. The current adapters and snapshot builder own complete deterministic
builds. Incremental semantic mutation and persistent invalidation require later
architecture.

## Implementation prerequisites

1. Implement the private sorted recursive state scan, typed observation error,
   normalized source, injected deterministic test trigger, 250 ms production
   schedule, and single-value revision channel without manifest changes.
2. Add public update status/observer values with closed counters, phases, and
   failure kinds while preserving existing Workspace APIs.
3. Extend `WorkspaceService` startup with before/build/after race closure and
   structured ownership of the source and coordinator.
4. Serialize `WorkspaceSnapshotBuilder` rebuilds, publish successful complete
   snapshots atomically, retain on recoverable failures, and implement later
   change recovery and cancellation cleanup exactly as decided.
5. Add non-zero focused observation, coalescing, orchestration, failure,
   recovery, lifecycle, cleanup, and repeated-run tests.
6. Add public real-filesystem EDT/Designer XML and Graph Query evidence using
   only disposable tracked fixture copies and event/status acknowledgements.
7. Synchronize current-state documentation only after public evidence exists;
   do not transition Sprint 19 before integration review.
8. Run focused affected tests and the complete workspace validation gate.

## Deferred scope

- native filesystem notifications, raw backend events, configurable poll
  periods, scan budgets, optimized fingerprints, benchmarks, and performance or
  security guarantees;
- persistent cache, schema versions, invalidation, migration, corruption, and
  clean-rebuild equivalence: Sprint 20;
- supported CLI watcher/update behavior: Sprint 21;
- incremental graph or Semantic Index mutation, per-file semantic repair,
  stable snapshot generation identity, changed-configuration sets, and Diff
  publication;
- watch-control/status HTTP routes, subscriptions, streaming progress,
  `oneagent-protocol`, MCP, LSP, IDE, AI/context, and alternate transports;
- Git change ingestion, remote/network workspaces, symlink traversal, path
  canonicalization/case-folding/Unicode equivalence, edit transactions,
  retries, restart, forced termination, and shutdown timeouts;
- authentication, authorization, metrics/tracing export, packaging, and
  performance/security certification.

## Coverage Registry impact

None. Sprint 19 invalidates and republishes already accepted complete semantic
snapshots. It adds no graph fact, source-parser capability, adapter format, or
Coverage Registry transition.

## Consequences

- Runtime gains deterministic cross-platform change detection without a new
  production dependency or platform event vocabulary.
- Conservative complete-byte scans favor correctness and testability over
  performance; optimization remains explicit deferred work.
- Complete rebuilds remain serialized and atomic, readers keep querying the
  last valid immutable snapshot, and invalid edits can recover on a later
  change.
- Single-value revision signaling bounds queued work while preserving one latest-state
  follow-up for changes observed during a build.
- Update status makes attempts, failures, recovery, and cleanup observable
  without changing lifecycle-derived readiness or HTTP schemas.
- Sprint 20 can consume complete valid publications for persistence without
  inheriting raw filesystem-event semantics.
