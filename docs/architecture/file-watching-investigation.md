# File Watching Investigation

## Status and scope

This investigation records the committed repository evidence available before
ADR-0041. It does not choose watcher architecture, add a dependency, change
Runtime behavior, or claim Sprint 19 support.

Investigated baseline:

- HEAD: `6e220073c0b90f36e1997ed49898e6cb32f7b6d0`
- Sprint 18 review: `pass`
- Roadmap state: Sprint 19 File Watching is the unique `next` target
- initial task working tree: clean
- CI operating systems: `macos-14` and `windows-latest`

## Confirmed accepted constraints

- ADR-0037 makes `oneagent-runtime` the composition root. Services start in
  registration order, own every long-running task until termination, cancel in
  reverse order, and join all work before `App::run` returns.
- ADR-0038 makes the canonical Runtime lifecycle the only health/readiness
  authority. A watcher or rebuild status cannot become another mutable
  readiness label or change the health wire contract.
- ADR-0039 makes `WorkspaceSnapshotBuilder` the complete source-neutral build
  path. It discovers supported roots, dispatches EDT and Designer XML builders
  sequentially, validates every graph, rejects the complete result on any fatal
  input, and returns configurations ordered by canonical Configuration ID.
- ADR-0039 publishes only complete immutable snapshots through
  `tokio::sync::watch`. It accepts no partial graph, aggregate graph, source
  repair, incremental mutation, or second semantic authority.
- ADR-0040 makes `GraphQueryService` a read-only consumer of one
  `WorkspaceSnapshotObserver`. Each request clones one `Arc<WorkspaceSnapshot>`
  and therefore can keep observing one immutable old or new snapshot
  independently of later publication.
- Sprint 19 owns file watching and rebuild triggers. Sprint 20 owns persistent
  cache and invalidation; Sprint 21 owns the supported CLI client. Git, remote
  workspaces, IDE integration, diagnostics, and edit transactions remain later
  work.

## Current ownership and data flow

### Configuration and composition

`RuntimeConfig` owns one `PathBuf` Workspace root and exposes it as `&Path`.
`apps/runtime/src/main.rs` constructs one `WorkspaceService`, derives one
`WorkspaceSnapshotObserver` for `GraphQueryService`, registers `http` before
`workspace`, and then runs until `ctrl_c` completes.

The current dependency direction is:

```text
RuntimeConfig.workspace_root
    -> WorkspaceService
        -> WorkspaceSnapshotBuilder
            -> FileSystemWorkspaceDetector
            -> FileSystemEdtSemanticGraphBuilder
            -> FileSystemDesignerXmlSemanticGraphBuilder
        -> watch::Sender<Option<Arc<WorkspaceSnapshot>>>
            -> WorkspaceSnapshotObserver
                -> GraphQueryService
                    -> HttpService
```

No watcher, raw filesystem event type, rebuild coordinator, rebuild status, or
change-observation public API currently exists.

### Initial build and publication

`WorkspaceService::start` currently:

1. clones the configured root;
2. moves its `WorkspaceSnapshotBuilder` into one `spawn_blocking` initial build;
3. converts a join failure to `WorkspaceBuildError::BuildTask`;
4. propagates any discovery, semantic-build, validation, cardinality, or
   duplicate-identity failure as service-start failure;
5. publishes `Some(Arc<WorkspaceSnapshot>)` exactly once;
6. waits only for cancellation in its returned service task;
7. publishes `None` before that task completes.

The builder and production detector are owned values. The current generic
bound requires only `WorkspaceDetector + Send + 'static`; repeated use after
startup is not part of the public service contract even though the concrete
production detector and builder are `Copy`.

### Failure vocabulary

`WorkspaceBuildErrorKind` distinguishes discovery, unsupported format,
semantic build, graph validation, invalid Configuration cardinality, duplicate
Configuration identity, and blocking-task join failure. The error records the
root and applicable source format or validation result. It does not define
watcher startup, watcher runtime, overflow, root-loss, coalescing, stale
snapshot, retained snapshot, or recovery outcomes.

### Consumers and compatibility

- `WorkspaceSnapshotObserver::snapshot` clones the current optional `Arc`.
- `WorkspaceSnapshotObserver::subscribe` returns an owned watch receiver for
  future publication changes.
- `GraphQueryService` consumes only the observer and has no publication or
  rebuild authority.
- The HTTP adapter checks lifecycle readiness before Workspace availability and
  retains exact Sprint 16 and Sprint 18 health/query routes, schemas, error
  codes, and method behavior.
- Public Workspace and Graph Query tests depend on publication during
  `Initializing`, visibility during `Running`, availability until reverse
  Workspace cleanup in `Stopping`, final `None`, sender closure, and released
  HTTP resources.

## Filesystem discovery evidence

`FileSystemWorkspaceDetector` recursively walks the configured root to a
default depth of six and returns a `BTreeMap`-ordered set of project roots. It
does not follow non-directory entries. Before a project root is recognized it
skips directories named exactly `.git`, `.idea`, `.vscode`, `target`, and
`node_modules`.

An EDT project is recognized only when both of these regular files exist:

- `.project`
- `src/Configuration/Configuration.mdo`

A Designer XML project is recognized through the production Designer XML
marker check, including `ConfigDumpInfo.xml` and `Configuration.xml`. A root
with both accepted marker families is a fatal conflicting-format error. Once a
supported project root is recognized, the detector stops descending below that
root; format-specific builders own its contents.

This proves that creation, removal, rename, or replacement of project markers
can change the discovered project set or its format. It also proves that a
watch limited only to initially discovered project roots would miss creation of
a new project elsewhere under the configured root.

The detector exposes deterministic errors for a missing/non-directory root,
unreadable directory or entry metadata, invalid Designer XML candidate, and
conflicting markers. It does not canonicalize the configured path or publish a
symlink traversal policy for watching.

## Confirmed production input families

The tracked Runtime fixture proves these source families are read by the
production build path:

| Format | Confirmed tracked input | Observable oracle |
| --- | --- | --- |
| EDT | `.project` and `src/Configuration/Configuration.mdo` | Project discovery and exact Configuration ID/name |
| EDT | metadata descriptors under `src/<MetadataKind>/<Name>/*.mdo` | Metadata and member nodes, ownership, reports, validation |
| EDT | BSL modules such as `src/Documents/<Name>/ObjectModule.bsl` | Module, procedure, query, Calls/Reads/Writes facts where present |
| Designer XML | `ConfigDumpInfo.xml` and `Configuration.xml` | Project discovery and exact Configuration ID/name |
| Designer XML | top-level metadata descriptors such as `CommonModules/<Name>.xml` | Metadata nodes and ownership |
| Designer XML | nested source such as `CommonModules/<Name>/Ext/Module.bsl` | Module and declaration/call facts |

Format-specific readers also contain repository evidence for other accepted
metadata descriptor, module, role-right, subsystem-content, SKD, XDTO, service,
form, command, and subordinate artifact paths. Sprint 19 must not copy that
inventory into a second semantic allow-list: the production builders remain the
authority for whether an observed path contributes to a rebuild result.

## Relevance: confirmed, unsupported, and unknown

### Confirmed relevant candidates

- any accepted project marker creation, modification, removal, or rename;
- any production-consumed `.mdo`, `.xml`, `.bsl`, or accepted subordinate
  artifact change under a detected project;
- creation or removal of a supported project root within the configured
  discovery depth;
- a directory change that alters discovery or the existence of a
  production-consumed descendant.

For these candidates, a complete rebuild is a reliable oracle: compare the
resulting immutable snapshot, graph-query projection, diagnostics, reference
evidence, report, or deterministic build failure as applicable.

### Confirmed discovery exclusions

Changes wholly below `.git`, `.idea`, `.vscode`, `target`, or `node_modules`
cannot create a project discovered through traversal from an unrecognized
ancestor because the detector skips those directories. This does not prove
that every same-named directory below an already recognized project is ignored
by every format-specific builder.

### Unsupported or unresolved classifications

- editor swap, backup, lock, generated-output, arbitrary unknown-extension,
  and metadata-only changes have no accepted global ignore contract;
- file rename is not a semantic primitive in current adapters; only the
  resulting complete filesystem state has authority;
- raw create/modify/remove/rename event ordering, duplicate delivery,
  coalescing, overflow, rescan, and watch-root loss are not modeled;
- path case folding, Unicode normalization, symlink replacement, junctions,
  network filesystems, permission changes, and events outside the configured
  root have no accepted watcher contract;
- a byte change can rebuild to an equal semantic snapshot, but no accepted
  behavior decides whether equal results are republished or treated as a no-op.

ADR-0041 must use conservative complete rebuilds or an evidence-backed
relevance rule. It must not claim that unknown files are semantically
irrelevant merely because the reduced fixture does not contain them.

## Dependency and platform evidence

The workspace and `oneagent-runtime` manifests and `Cargo.lock` contain no
filesystem notification crate such as `notify`, no `walkdir`, and no portable
watch abstraction. The standard library exposes filesystem reads and metadata
but no cross-platform notification API. Tokio 1.53 is already a direct Runtime
dependency with the `full` feature and provides tasks, channels, blocking-task
bridging, timers, cancellation coordination, and test timeouts, but not native
filesystem watching.

Two implementation families are therefore repository-feasible:

1. A portable bounded polling/rescan source can use existing `std` filesystem
   APIs plus Tokio scheduling and an injected deterministic trigger in tests.
   It requires no production dependency but must define scan identity, period,
   cost boundary, I/O failures, coalescing, and equal-state behavior.
2. Native cross-platform notifications require a new production dependency or
   separate platform implementations. No such dependency is locked or approved
   by the repository. Selecting one requires exact version/API evidence outside
   this investigation and explicit user approval before Task 3 changes a
   manifest. Hand-written macOS and Windows backends would add unsafe/platform
   surface contrary to the current `unsafe_code = "forbid"` baseline and are
   not a bounded first slice.

This evidence does not choose polling or native notifications. It proves that
an existing-dependency first slice is implementable only as polling/rescan, and
that a native first slice has an approval prerequisite.

CI runs format, check, test, and Clippy on `macos-14` and `windows-latest`.
Linux-only watcher behavior would not satisfy the live repository gate.

## Repository-owned test oracles

### Existing deterministic seams

- `apps/runtime/tests/workspace_service.rs` copies the tracked fixture into a
  `tempfile::TempDir`, builds both production formats, asserts exact identities,
  counts, diagnostics/reference evidence, atomic startup failure, lifecycle
  state, shutdown clearing, sender closure, and equal fresh runs.
- `apps/runtime/tests/graph_query_api.rs` uses the same production fixture and
  public loopback HTTP to assert exact EDT/Designer configuration and node
  projections, immutable snapshot selection, lifecycle gating, cleanup, port
  release, and repeated equal runs.
- Runtime service tests use `oneshot`, `mpsc`, and `watch` handshakes. One-second
  timeouts are hang guards; no accepted test uses an arbitrary sleep as event
  evidence.
- Filesystem detector tests already prove EDT and Designer marker creation,
  incomplete EDT exclusion, nested-boundary stopping, conflicting markers, and
  depth limits with temporary directories.

### Candidate mutation matrix

| Case | Repository-owned mutation | Reliable observable result |
| --- | --- | --- |
| Relevant modify | Change an exact name or BSL declaration in a temporary fixture copy | New snapshot/Graph Query value or deterministic graph diff |
| Relevant add | Add a valid copied production artifact or supported project below the configured root | New configuration or graph fact after complete build |
| Relevant remove | Remove a consumed artifact or project marker in the temporary copy | Removed fact/configuration or deterministic build outcome |
| Rename-equivalent | Rename a consumed path and await the normalized change boundary | Complete resulting-state build; no raw rename identity claim |
| Irrelevant | Change a file below a confirmed ignored discovery directory outside detected projects | No rebuild signal or no published replacement, as ADR-0041 decides |
| Burst/duplicate | Perform several mutations while a deterministic gate holds the coordinator | One bounded coalesced build plus accepted follow-up behavior |
| Invalid build | Corrupt a copied required descriptor or create conflicting markers | Existing complete snapshot retained or cleared exactly as accepted; no partial publication |
| Recovery | Repair the invalid temporary input and emit a later accepted signal | Complete valid replacement becomes observable |
| Atomic readers | Hold an old `Arc`, gate the new build, and query before/after publication | Every reader observes a complete old or complete new snapshot |
| Shutdown | Cancel while observation is active or a rebuild gate is held | Accepted in-flight behavior, closed receivers, joined tasks, cleared publication |
| Repetition | Run two fresh temporary copies through the same mutation sequence | Equal normalized observations and semantic/public results |

An injected observation or scheduling port is acceptable for focused tests only
when public production evidence still traverses the real selected filesystem
source. Public native or polling evidence may use bounded timeouts to detect a
hang, but the asserted event must arrive through a watcher, coordinator, or
snapshot acknowledgement rather than a fixed sleep.

## Open decisions required in ADR-0041

1. Does the first slice use portable polling/rescan with existing dependencies,
   or a specifically approved native notification dependency?
2. Is the configured Workspace root the sole watched boundary, and how are
   discovery depth, project-boundary stopping, ignored directories, paths
   outside the root, symlinks, and root replacement handled?
3. What closed internal normalized vocabulary represents relevant change,
   rescan, observation failure, overflow, and termination without leaking raw
   platform event types?
4. Which conservative relevance rule triggers a complete rebuild, and which
   changes are safely ignored from confirmed repository evidence?
5. What bound controls polling or event coalescing, queue growth, duplicate
   input, and changes arriving while a build is running?
6. Which component owns observation, coalescing, build scheduling, blocking
   tasks, publication, errors, and terminal cleanup? Is it one extended
   Workspace service or a separately registered service with an explicit port?
7. Does an equal complete rebuild republish a new `Arc` or remain a no-op, and
   what stable comparison defines equality without creating semantic authority?
8. On post-start observation or build failure, is the last valid snapshot
   retained or cleared, does the Runtime service terminate, and how can a later
   filesystem signal recover when the service remains alive?
9. What happens to a pending or in-flight rebuild at cancellation, and at what
   exact point is the published snapshot cleared and observation closed?
10. Which typed in-process status/error evidence is exposed without changing
    health/readiness or adding an HTTP control/status contract?
11. Which focused injected tests and which public real-filesystem tests are
    mandatory on both CI operating systems?
12. Which behavior is explicitly deferred to persistent cache, CLI, Git,
    remote workspaces, incremental semantic mutation, diagnostics, and edits?

## Decision readiness

The repository contains enough production sources, immutable publication seams,
failure vocabulary, lifecycle ownership, consumer contracts, tracked fixture
provenance, temporary-mutation capability, and macOS/Windows validation to
decide a bounded first slice and test it safely. No external artifact is
required.

Task 2 may proceed. It must choose one of the dependency families, define every
open matrix row above, and keep a new production dependency gated on explicit
approval. Task 2 must stop if it cannot close watcher failure/recovery or
deterministic public evidence without inventing platform behavior.

## Deferred scope

Incremental graph/index mutation, persistent cache and invalidation, supported
CLI behavior, watch-control HTTP routes, subscriptions, streaming progress,
Git/network workspaces, IDE integration, diagnostics, edit transactions,
benchmarks, and performance/security claims remain outside Sprint 19.
