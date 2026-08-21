# ADR-0042: Runtime Workspace Persistent Cache

## Status

Accepted

## Context

Sprint 20 must persist validated Runtime Workspace semantic state with
deterministic invalidation. The
[Persistent Cache investigation](../architecture/persistent-cache-investigation.md)
confirms that the repository already provides:

- complete source-neutral immutable `WorkspaceSnapshot` values with separate
  ordered Configuration graphs, diagnostics, reference evidence, statistics,
  and reports;
- deterministic graph nodes, typed payloads, edges, provenance, queries, Diff,
  reports, and complete build validation;
- complete-byte Workspace source observation with stable relative-path order;
- one Runtime-owned initial-build and File Watching rebuild coordinator with
  serialized blocking work, atomic valid publication, last-valid failure
  retention, recovery, cancellation, and cleanup;
- public Graph Query and lifecycle/health compatibility boundaries;
- locked Runtime Serde and Serde JSON dependencies, tracked EDT and Designer XML
  production fixtures, disposable test roots, and macOS/Windows CI.

No persisted schema, semantic-build version, cache path, validity key, reader,
writer, replacement policy, compatibility rule, corruption outcome, recovery
policy, or cache observation exists. The Graph Query JSON projection is
deliberately incomplete and cannot be reused as semantic state. Terminal
`SemanticReferenceRequest` values also lack one checked reconstruction entry
point for their publicly observable combined provenance.

This ADR defines a correctness-first private Runtime cache. It does not claim a
performance target, public file-format stability, hostile-input security,
power-loss durability, or multi-process coordination.

## Decision

### Canonical authority and ownership

`WorkspaceSnapshot` and each contained `SemanticGraph` remain the only
published semantic authorities. Cache bytes are a recoverable private Runtime
representation, never a second fact model, graph index, readiness label, or
adapter input.

`oneagent-runtime` owns:

- the versioned persisted DTOs and exhaustive domain conversions;
- complete Workspace source-state capture for cache validity;
- the cache directory, file reader, writer, replacement, cleanup, and status;
- startup and post-change load/build/write orchestration;
- validated immutable publication through the existing Workspace observer.

`WorkspaceService` remains the single structured owner of cache work, semantic
builds, File Watching, publication, cancellation, and terminal cleanup. Cache
I/O and encoding execute inside its owned blocking operations. There is no
separate Runtime service, global cache, detached task, adapter-owned cache, or
HTTP owner.

The graph domain remains responsible for checked semantic reconstruction and
validation. Source adapters remain the only producers on clean builds and do
not depend on Runtime or cache types. Graph Query and health remain read-only
consumers of published Runtime state.

### Checked terminal-request reconstruction prerequisite

Task 3 adds one source-independent checked graph-domain constructor for
`SemanticReferenceRequest`. Exact Rust naming may follow local conventions; its
contract accepts the publicly observable source node, category, typed
reference, expected kinds, candidates, resolution state, terminal outcome, and
combined provenance.

The constructor must:

- normalize and require the expected-kind set;
- normalize candidates and validate candidate cardinality against the outcome;
- reject collected/non-terminal state for a persisted complete build;
- reject missing or `NotApplicable` provenance;
- validate lifecycle state/outcome compatibility;
- recompute request identity from immutable identity fields;
- preserve normalized combined provenance exactly as the canonical request
  value exposes it;
- return the existing typed request error family or a bounded additive variant.

This is an additive checked reconstruction API, not a Serde contract. It does
not guess a collection/resolution provenance split and does not weaken the
existing producer transition API. The strong complete-build validator still
checks request nodes, edges, diagnostics, statistics, and report consistency
after ledger reconstruction.

### Private persisted envelope

The cache is deterministic UTF-8 JSON encoded with the already approved Runtime
`serde` and `serde_json` dependencies. Lower-level common, metadata, and graph
types do not gain Serde dependencies or derive-based wire layout. Runtime owns
private closed DTOs and exhaustive conversions.

The top-level envelope has exactly these fields:

```json
{
  "format": "oneagent.workspace-cache",
  "schema_version": 1,
  "semantic_version": 1,
  "content_checksum": "fnv1a64:0000000000000000",
  "source": { "entries": [] },
  "workspace": { "configurations": [] }
}
```

Unknown or duplicate object fields are rejected. Required fields may not be
omitted. JSON object member order is not a compatibility guarantee for readers,
but the Runtime writer emits the struct declaration order above. Arrays use the
canonical orders defined below.

`format` is the exact file identity. `schema_version` owns envelope, DTO, field,
and closed-vocabulary compatibility. `semantic_version` owns every builder,
parser, graph, validation, normalization, and orchestration behavior that can
change the complete Workspace snapshot for equal source state. Both are
unsigned decimal JSON integers encoded from fixed-width Rust values.

The canonical content bytes are the deterministic JSON encoding of the exact
`source` and `workspace` DTO pair, in that order. `content_checksum` is lowercase
`fnv1a64:` followed by exactly sixteen hexadecimal digits computed with the
standard FNV-1a 64-bit offset basis `14695981039346656037` and prime
`1099511628211` over those content bytes.

The checksum detects accidental partial or changed content before domain
reconstruction. It is not cryptographic authentication and creates no hostile-
input or collision-resistance claim. Schema checks, exact source comparison,
checked reconstruction, and complete validation remain mandatory after a
checksum match.

Writer output is byte-deterministic for equal accepted source and Workspace
values on the same supported platform. Whitespace, alternate number spellings,
and object member reordering from another writer are not accepted canonical
bytes even if a general JSON parser could interpret them. A loaded envelope is
re-encoded canonically and must equal the original bytes before it can be a hit.

### Complete source-state DTO

`source.entries` records the existing complete `WorkspaceFileState` in its
canonical relative-path order. Each entry contains:

- `path`: a non-empty array of relative path-component strings;
- `kind`: exactly `directory`, `regular_file`, or `other`;
- `bytes`: an array of byte integers for `regular_file`, and absent for other
  kinds.

Path components must be losslessly UTF-8, non-empty, and reconstruct exactly one
platform path component without prefix, root, current-directory, or parent
components. A non-UTF-8 or otherwise non-representable source path makes the
cache unavailable for that scan but does not make the semantic build invalid.
Clean build and in-memory publication continue.

The source state retains the Sprint 19 exact relative paths, entry kinds, and
complete regular-file bytes. It uses no timestamp, file size without bytes,
inode, process identity, native event, absolute Workspace root, or directory
enumeration order.

The cache directory `.oneagent` and its complete subtree are excluded from
`WorkspaceFileState`, including the `.oneagent` directory entry itself. The five
existing ignored-directory rules and meanings remain unchanged. This one exact
new exclusion prevents cache creation, writes, replacement, removal, and
temporary cleanup from changing semantic validity or feeding the watcher loop.
Other `.oneagent` content is therefore also outside Sprint 20 source semantics;
user-facing storage under that reserved directory is not accepted.

### Complete Workspace DTO

`workspace.configurations` uses ascending canonical Configuration `EntityId`
order. Each record persists:

- discovered configuration root as UTF-8 relative components below the
  configured Workspace root;
- format as exactly `edt` or `designer_xml`;
- every graph node in canonical node-ID order;
- every graph edge in canonical edge-ID order;
- ordered diagnostics;
- canonical reference requests in request-ID order;
- exact total reference statistics.

Configuration ID and name are not duplicated. They are re-extracted from the
single Configuration graph node. The report is not persisted as independent
authority; it is deterministically recomputed from the reconstructed graph,
diagnostics, requests, and legacy statistics and stored in the returned
`WorkspaceConfigurationSnapshot`.

Graph nodes persist ID, exact name, exhaustive `NodeKind`, exhaustive
`GraphNodePayload`, and provenance. Edges persist source ID, target ID,
exhaustive `EdgeKind`, and provenance. Runtime DTOs exhaustively cover all
current metadata/common/member/access-right/DCS/XDTO/HTTP/Web Service payload
variants and all nested typed values. There is no unknown fallback. Adding any
new enum variant or persisted semantic field requires a schema or semantic
version review before writer compatibility can continue.

Diagnostics persist code, severity, kind, exact message, typed reference,
optional source node, expected kinds, optional actual kind, candidates, and
provenance. Requests persist every input to the checked reconstruction contract.
Statistics persist all outcome and provenance counters as fixed-width unsigned
values.

Root components are joined to the current configured Workspace root on load and
must remain below it under lexical component validation. A relocated Workspace
may reuse a cache only when its complete relative source state is equal; cached
absolute paths never leak from the earlier location.

### Decode, reconstruction, and validation order

A candidate becomes a `Hit` only after all of these ordered gates succeed:

1. read one complete bounded file into memory;
2. parse the exact envelope and reject duplicate/unknown/missing fields;
3. match `format`, `schema_version`, and `semantic_version`;
4. re-encode and require canonical byte equality;
5. recompute and match the FNV-1a content checksum;
6. reconstruct and compare the complete source DTO exactly with the current
   pre-load `WorkspaceFileState`;
7. validate every relative path and fixed-width integer conversion;
8. construct checked IDs, names, payloads, nodes, edges, diagnostics, requests,
   ledger, and statistics;
9. reject duplicate node, edge, request, or Configuration identity;
10. recompute canonical request statistics and derive legacy observations by
    checked per-counter subtraction from persisted total statistics;
11. recompute the report;
12. run the strongest request-aware complete-build validation with the graph,
    diagnostics, ledger, legacy observations, and report;
13. require exactly one Configuration node and verify canonical ordering and
    relative root/format record consistency;
14. construct one complete immutable `WorkspaceSnapshot`.

Any underflow, count inconsistency, invalid request lifecycle, payload/kind
mismatch, missing endpoint, invalid graph, inconsistent diagnostic/request edge,
invalid report, duplicate, or path error rejects the whole candidate. No partial
Configuration or graph is returned.

### Version and invalidation contract

The first writer uses `schema_version = 1` and `semantic_version = 1`.

- exact versions plus exact source state proceed to decode and validation;
- any schema version other than `1` is `Incompatible`;
- any semantic version other than `1` is `Incompatible`;
- exact versions with unequal source state are `SourceChanged`;
- missing candidate is `Missing`;
- there is no field-level migration from a historical schema because none
  exists in repository evidence.

Migration for `Incompatible`, `SourceChanged`, or rejected current-version
content is one complete clean build followed by current-version replacement.
This is the only accepted Sprint 20 migration. Forward compatibility, downgrade
support, lossy conversion, or invented version-zero fixtures are rejected.

The semantic-version constant must be changed in the same logical change as any
production behavior that can change complete snapshot content for equal source
state. This includes parser, adapter, graph payload, identity, provenance,
diagnostic, reference, report, validation, complete-scope, or normalization
behavior. A schema-version review is also required when persisted vocabulary or
reconstruction changes. This manual compatibility responsibility must be named
in source documentation and tests; package version and Git commit are not
substitutes.

### Cache path and containment

The single-process cache path is fixed relative to the configured Workspace
root:

```text
<workspace>/.oneagent/cache/workspace-v1.json
<workspace>/.oneagent/cache/workspace-v1.tmp
```

There is no enabled flag or cache-root override in the first slice. A temporary
Workspace used by tests therefore owns a disposable cache automatically.

Runtime does not canonicalize or rewrite the configured Workspace root. Before
accessing cache content, it uses `symlink_metadata` to require existing
`.oneagent` and `cache` components to be real directories rather than symlinks
or other kinds. The candidate and temporary path must be absent or a regular
file and must never be followed through a symlink. Missing directories are a
load `Missing` and may be created during a later write. A wrong kind, symlink,
or I/O failure is `Unavailable`.

Lexically fixed names plus no symlink traversal contain task-owned cache files
under the accepted root for non-adversarial single-process use. Filesystem races,
malicious replacement between checks, hostile permissions, and external writers
are outside the first-slice security contract.

### Load outcomes

The transport-neutral public in-process load outcome vocabulary is exactly:

- `NotAttempted`;
- `Hit`;
- `Missing`;
- `SourceChanged`;
- `Incompatible`;
- `Corrupt`;
- `Unavailable`.

Malformed/truncated/noncanonical JSON, checksum mismatch, invalid paths,
duplicates, checked-construction errors, invalid graph/build evidence, and
inconsistent counters/report are `Corrupt`. Version mismatch is
`Incompatible`. Missing directories/file are `Missing`. Exact source mismatch is
`SourceChanged`. Read/path/symlink/wrong-kind failures are `Unavailable`.

Every outcome except `Hit` falls back to a clean build. Cache rejection is not a
Workspace startup failure when the clean build succeeds.

### Complete write and replacement

The writer accepts only a complete validated `WorkspaceSnapshot`, the exact
stable source state paired with its build, and current versions. It encodes the
complete DTO, checksum, and canonical envelope in memory and decodes/validates
those bytes before filesystem replacement.

The bounded replacement sequence is:

1. validate `.oneagent` and `cache` components or create missing real
   directories one component at a time;
2. reject symlink or wrong-kind components;
3. remove a stale regular `workspace-v1.tmp`; reject a symlink or other kind;
4. create the temporary file with `create_new`;
5. write all canonical bytes and call `sync_all` on the temporary file;
6. close it, read it back, and require the complete decode/validation gates;
7. remove an existing regular `workspace-v1.json`; reject a symlink or other
   kind;
8. rename the validated closed temporary file to the now-absent final path;
9. remove a remaining regular temporary file after any failure where possible.

This sequence is portable across the repository's macOS and Windows CI and
never exposes a partial file at the final path. If final removal succeeds and
rename fails, there is no current cache entry; the in-memory valid snapshot is
still publishable and the next run clean-builds. Directory fsync and survival
across power loss are not promised.

The public in-process write outcome vocabulary is exactly:

- `NotAttempted`;
- `Succeeded`;
- `SkippedUnstableSource`;
- `Failed`.

Write failure is recoverable cache evidence, not semantic-build failure. It does
not convert a valid clean build into a Runtime startup or update failure.

### Public cache observation

`WorkspaceService` gains one cloneable transport-neutral
`WorkspaceCacheObserver` before registration, parallel to its snapshot and
update observers. It exposes the current immutable `WorkspaceCacheStatus` and a
watch subscription.

Status contains only the latest typed load and write outcomes above. It starts
with both `NotAttempted`. Startup publishes the load outcome, then the write
outcome when applicable. Later stable File Watching writes update the write
outcome. Status contains no paths, source bytes, graph payloads, error prose,
timestamps, durations, checksums, or secrets.

The service owns the only sender. Shutdown closes the channel after owned work
finishes. Cache status is testability and in-process observability only. It does
not alter lifecycle, readiness, liveness, HTTP, Graph Query, protocol, or CLI
schemas.

### Startup orchestration and race closure

One owned blocking startup operation performs:

1. scan source state `S0` with the complete cache exclusion;
2. attempt to load an exact cache candidate against `S0`;
3. scan source state `S1`;
4. when load is `Hit` and `S0 == S1`, return the validated cached snapshot
   without invoking discovery or either semantic adapter;
5. otherwise build one complete clean snapshot after `S1`;
6. scan source state `S2` after the build;
7. when `S1 == S2`, attempt to write the clean snapshot paired with `S2`;
8. when `S1 != S2`, skip the write as `SkippedUnstableSource`;
9. return the complete valid snapshot, final source state, load/write status,
   and whether a latest-state follow-up is required.

The async owner publishes the returned snapshot only after the blocking handle
joins. A successful cache hit counts as the first Workspace build attempt and
first publication for compatibility with `WorkspaceUpdateStatus`; it is not a
semantic adapter build. Cold clean-build failure retains the existing named
Workspace startup-failure behavior. Cache load or write rejection is nonfatal
when the clean build succeeds.

The change source starts from `S1` for a stable hit or `S2` for a clean build.
It receives one initial changed revision when the accepted before/after states
show that a follow-up is required. No source change observed before publication
is silently lost.

### File Watching rebuild integration

After a relevant change, the existing serialized coordinator performs one
owned blocking operation:

1. scan pre-build source state `R0`;
2. run the existing complete builder;
3. scan post-build state `R1`;
4. when the build succeeds and `R0 == R1`, write the validated snapshot paired
   with `R1`;
5. when the build succeeds and `R0 != R1`, skip the write;
6. return the valid snapshot and cache write outcome.

The async owner atomically publishes the complete snapshot after the blocking
operation joins, even when the cache write failed or was skipped. A write failure
updates only cache status. A semantic-build failure preserves the Sprint 19
last-valid snapshot and update failure and performs no cache write. The existing
latest revision triggers the accepted follow-up for changes observed during the
operation.

An older cache file remains keyed by its embedded old source state and cannot be
a hit after source change. On a later process start it becomes `SourceChanged`
and is replaced only after a stable clean build.

### Lifecycle, health, queries, cancellation, and shutdown

- Runtime remains `Initializing` until cache load or clean build and any attempted
  startup write finish and Workspace acknowledges start.
- Cache failures do not add lifecycle states or change readiness.
- Snapshot publication remains one complete immutable `Arc`; held older values
  remain immutable.
- Graph Query continues to acquire exactly one published snapshot per call and
  retains every Sprint 18 route, schema, bound, error, and order.
- HTTP liveness/readiness remain exactly lifecycle-derived under ADR-0038.
- Cancellation waits for the current blocking load/build/write operation to
  join under the existing non-interruptible policy, stops future work, joins the
  change source, clears the snapshot, publishes terminal update status, and
  closes snapshot/update/cache observers.
- Cache files persist after ordinary shutdown for the next fresh process; they
  are not temporary Runtime resources.

There is no retry timer. Recovery occurs on the current startup clean-build path
or a later accepted File Watching change.

### Error and diagnostic containment

Cache display errors may retain a bounded path and source error chain inside
Runtime diagnostics, but public cache status exposes only the closed outcome
vocabularies. No source bytes, serialized graph, diagnostic collection,
reference ledger, checksum input, backtrace, or unbounded path set is logged or
published as cache status.

### Deterministic test contract

Focused codec tests must cover:

- empty, EDT, Designer XML, and mixed snapshots;
- every current node/payload/edge/provenance/diagnostic/reference variant;
- terminal-request reconstruction and conflicting/invalid lifecycle values;
- deterministic canonical bytes and re-encoding;
- malformed, truncated, noncanonical, duplicate, incompatible, checksum,
  path, payload, endpoint, graph, request, counter, and report failures;
- clean-build graph Diff, exact build evidence, report, and query equivalence.

Focused store tests must cover:

- missing/current/source-changed/incompatible/corrupt/unavailable candidates;
- UTF-8 component validation and cache containment;
- `.oneagent` watcher exclusion;
- directory creation, stale temporary cleanup, wrong kinds, symlinks where
  portable, create/write/sync/readback/remove/rename failure injection;
- no partial final entry, recovery, and repeated use.

Focused Runtime tests use controlled builder and store seams to prove:

- warm hit invokes no detector or adapter builder;
- cold and rejected loads invoke exactly one complete clean build;
- stable builds write; unstable builds skip; failed builds do not write;
- load/write failure behavior, publication, status, follow-up coalescing,
  cancellation, sender closure, and fresh repetition.

Public tests copy the tracked EDT/Designer fixture into disposable roots and
prove cold miss/write, warm hit, complete snapshot/query equality, source and
version invalidation, corruption, unavailable/write failure where publicly
constructible, clean rebuild, File Watching replacement followed by warm reuse,
health/query compatibility, shutdown, cleanup, and fresh repetition.

Tests use channels and watch observations; timeouts are hang guards. Arbitrary
sleeps, production polling duration, external services, host-global cache state,
ignored corpora, and network access are not acceptance evidence.

### Dependency decision

The first slice uses only existing Runtime production dependencies and Rust
standard-library filesystem APIs. It adds no production dependency and requires
no manifest or lockfile change.

Serde remains confined to Runtime-private DTOs. FNV-1a is implemented as a small
fully specified private function. Adding Serde to lower-level crates or adding a
hash, binary codec, database, compression, encryption, locking, or atomic-file
dependency is not accepted by this ADR and would require a new architecture
decision plus explicit dependency approval.

### Compatibility and Coverage

Existing Runtime constructors, service ordering, Workspace build/update status,
snapshot observers, Graph Query and health HTTP wire contracts, graph/domain
APIs, adapters, and Coverage meanings remain compatible except for these
additive surfaces:

- one checked graph-domain terminal-request reconstruction constructor;
- one cache observer and closed cache status/outcome types;
- the reserved `.oneagent` source-observation exclusion.

The private file format is versioned but not a public API. Removing or changing
it requires current-version invalidation and clean rebuild, not end-user
migration tooling.

Graph-domain, EDT, and Designer Coverage registries do not change status.
Persistence consumes complete accepted semantic results and adds Runtime
integration evidence; it does not expand supported semantic kinds or source
formats.

## First production slice

Sprint 20 implements only:

1. the checked terminal-reference reconstruction prerequisite;
2. one complete Runtime-private JSON schema with schema/semantic version `1`;
3. exhaustive complete snapshot encoding, checksum, checked decoding, and
   strongest build validation;
4. complete source-state equality and one fixed contained Workspace-local cache;
5. portable complete-file replacement with no partial final entry;
6. closed cache load/write status observation;
7. startup hit/miss/clean-build/write and File Watching stable-build writes;
8. public EDT/Designer cold/warm/invalidation/corruption/recovery/query/lifecycle
   evidence.

## Rejected alternatives

### Persist the Sprint 18 Graph Query JSON projection

Rejected because it omits payloads, provenance, diagnostics, reference evidence,
reports, and roots and cannot reconstruct ADR-0039 state.

### Derive Serde on every canonical domain type

Rejected because private field layout would become the implicit cache schema,
lower-level crates would gain production dependencies, incidental indexes could
leak, and checked reconstruction/version vocabulary would be obscured.

### Persist only `SemanticGraph`

Rejected because Runtime snapshots intentionally preserve diagnostics,
reference ledger/statistics, report, format, and root evidence consumed by
current public APIs and validation.

### Trust decoded DTOs without canonical domain reconstruction

Rejected because cache bytes would bypass payload/kind, endpoint, request,
report, cardinality, duplicate, and validation invariants and become a second
semantic authority.

### Use timestamps, sizes, package version, or Git commit as validity

Rejected because none completely identifies current source bytes and semantic
behavior, and several are platform/process/repository-state dependent.

### Use only a short source hash

Rejected as the authority because hash collision would admit stale state. The
accepted entry stores and compares complete source state; the non-cryptographic
checksum is only corruption evidence.

### Place cache outside the Workspace by default

Rejected because Runtime has no accepted global or user cache root, tests would
touch host-global state, cleanup ownership would be ambiguous, and Workspace
identity across roots is unresolved. The fixed reserved local directory is
disposable and source-excluded.

### Rename a temporary file over an existing target

Rejected because replace-over-existing behavior is not portable to the accepted
Windows CI. The accepted sequence removes only a verified regular target before
renaming a verified complete temporary file and tolerates a recoverable gap.

### Fail Runtime whenever cache load or write fails

Rejected because canonical sources remain available, persistence is a recovery
optimization, and a valid clean build remains the semantic authority. Clean-
build failure retains existing startup/update failure semantics.

### Publish before cache work finishes

Rejected for the first slice because detached or overlapping write ownership
would complicate shutdown and source pairing. The old snapshot remains available
during post-start blocking work, and startup remains not ready until the owned
operation joins.

### Invent a version-zero migration fixture

Rejected because no historical schema exists. Unsupported versions migrate by
clean rebuild and current replacement.

### Add cross-process locking, remote cache, eviction, or performance work

Rejected because repository evidence covers one Runtime owner only and provides
no product contract or acceptance oracle for those capabilities.

## Deferred scope

- incremental graph, Semantic Index, per-file, or per-configuration persistence;
- partial snapshots and adapter-specific cache formats;
- more than one process, locks, leases, shared or remote cache;
- cache root configuration, disable/clear/status transport APIs, CLI management,
  streaming, subscriptions, metrics, and tracing export;
- compression, encryption, authentication, cryptographic integrity, signing,
  eviction, age/size policy, hostile-input hardening, directory fsync, and
  power-loss durability;
- field-level migrations when a real prior schema exists;
- non-UTF-8 cacheable source paths;
- native notifications, Git/network workspaces, edits, restart, and forced
  termination;
- benchmarks and performance/security certification;
- supported CLI behavior owned by Sprint 21 and all later protocol/IDE/AI
  integrations.

## Implementation prerequisites

1. Add the checked terminal-request reconstruction API and exhaustive tests.
2. Implement private DTOs and closed exhaustive conversions without lower-level
   Serde changes.
3. Implement complete codec, canonical bytes, checksum, reconstruction, legacy-
   statistic derivation, report recomputation, and strongest validation.
4. Extend `WorkspaceFileState` with exact DTO conversion/equality inputs and the
   reserved `.oneagent` exclusion.
5. Implement the contained reader/writer and deterministic filesystem failure
   seams.
6. Add cache observer/status before Runtime integration.
7. Integrate startup and rebuild blocking operations without changing existing
   publication, query, health, or coalescing authority.
8. Complete focused and public tracked-fixture evidence and the full workspace
   gate before review.

## Consequences

- Equal accepted source and semantic versions can restore one complete validated
  Workspace snapshot without invoking semantic adapters.
- Every stale, incompatible, corrupt, partial, invalid, or unavailable entry is
  contained and can recover through canonical clean build.
- Correctness requires storing complete source bytes alongside complete semantic
  state, so the first format may be large and startup still scans source bytes.
- The fixed Workspace-local cache is private, deterministic, disposable, and
  excluded from File Watching.
- Cache writes delay the owned startup/update operation but never publish a
  partial snapshot or detached task.
- The absence of multi-process, durability, performance, and security claims is
  explicit and testable as deferred scope.
