# Sprint 20 Persistent Cache Investigation

## Status and baseline

Investigation complete at committed Sprint 20 planning head
`af52b579238843acc7cd393f4eb679f8a6384bfa`.

The Sprint 19 integration review records `pass`; `docs/Roadmap.md` identifies
Sprint 20 Persistent Cache as the unique `next` target. The working tree was
clean at Task 1 start. This document records evidence and decision questions;
it selects no cache architecture and changes no production behavior.

## Confirmed current authorities

### Runtime Workspace state

`apps/runtime/src/workspace/mod.rs` owns the source-neutral Runtime snapshot
boundary accepted by ADR-0039:

- `WorkspaceSnapshot` contains configuration records in canonical
  Configuration `EntityId` order and provides read-only iteration and exact ID
  lookup;
- `WorkspaceConfigurationSnapshot` retains the discovered root path,
  `WorkspaceFormat`, exact Configuration ID and name, one immutable
  `Arc<SemanticGraph>`, ordered diagnostics, one canonical reference-request
  ledger, reference statistics, and the matching graph report;
- `WorkspaceSnapshotBuilder` performs production discovery, sequential EDT or
  complete Designer XML builds, validation, Configuration-cardinality checks,
  duplicate-Configuration rejection, and all-or-nothing normalization;
- `WorkspaceSnapshotObserver` publishes one immutable `Arc` and
  `GraphQueryService` obtains and retains one such value per call;
- `WorkspaceService` owns the initial build, change source, serialized complete
  rebuilds, publication, status, cancellation, shutdown, and sender closure.

No persisted representation, cache reader, writer, schema version, cache
configuration, cache status, fingerprint, checksum, migration, corruption
category, or cache path exists in `apps/runtime/`, `crates/workspace/`, or
`crates/graph/`.

### Startup and rebuild sequence

The current initial blocking operation performs these observable steps:

1. scan the configured root into `WorkspaceFileState`;
2. build one complete `WorkspaceSnapshot` through the production builder;
3. fail startup if the first scan failed;
4. scan the root again;
5. return both scans and the valid snapshot to the async owner;
6. publish the complete snapshot;
7. start the change source from the post-build state and enqueue one initial
   change when the scans differ.

Post-start changes increment the public attempt counter, run one owned blocking
complete build, atomically replace the published `Arc` only on success, retain
the last valid snapshot on semantic-build failure, and publish a typed update
failure. A capacity-one watch revision preserves one latest-state follow-up.
Cancellation joins the source and any in-flight build before clearing the
snapshot and closing the senders.

A cache integration must fit this owner and close an analogous scan/load/build/
write race. Existing code contains no accepted ordering for those extra steps.

### Source-state observation

`apps/runtime/src/workspace/change.rs` owns a private `WorkspaceFileState`:

- a recursively scanned `BTreeMap<PathBuf, WorkspaceFileEntry>` provides
  deterministic relative-path ordering;
- entry values distinguish directories, complete regular-file bytes, and other
  file kinds;
- directory entries themselves remain in the state while descendants of
  `.git`, `.idea`, `.vscode`, `target`, and `node_modules` are excluded;
- symlinks and other non-regular entries are recorded as `Other` and are not
  traversed;
- equal state uses exact paths, kinds, and bytes, not timestamps, sizes, inode
  values, process state, or platform event order.

The type has no public iterator, canonical byte encoding, stable digest, or
validity-key API. The accepted ignored set does not contain a OneAgent cache
directory. A cache placed under the configured root would therefore at least
change the scanned directory inventory and, unless explicitly excluded by a
future contract, could trigger its own rebuild loop. A cache outside the root
would require explicit configuration and path ownership that do not exist.

### Runtime configuration and composition

`RuntimeConfig` currently owns application name, environment, HTTP bind address,
and one Workspace root. It has no cache enabled flag, cache root, namespace,
size, age, or cleanup setting. `ServiceContext` exposes immutable `AppState`
configuration plus receiver-only cancellation. Production `main` constructs one
`WorkspaceService`, gives its snapshot observer to Graph Query, registers HTTP
before Workspace, and owns no separate cache service.

ADR-0037 through ADR-0041 require the Workspace service to remain the owner of
blocking build work and complete publication. A cache may be a component used by
that owner; current evidence does not justify a detached service, global cache,
or adapter-owned persistence.

## Complete snapshot content and reconstruction evidence

### Canonical graph

`SemanticGraph` stores nodes and edges in deterministic `BTreeMap` indexes and
exposes stable `nodes()` and `edges()` iterators. Reconstruction can use:

- checked `EntityId::new` and `EntityName::new`;
- `GraphNode::new_with_payload_and_provenance`, which rejects payload/kind
  incompatibility;
- `GraphEdge::new_with_provenance` followed by
  `SemanticGraph::insert_edge`, which rejects missing endpoints;
- `SemanticGraph::insert_node`, whose replacement result lets a decoder reject
  duplicate node identities instead of silently replacing them;
- `SemanticGraph::validate`, which applies the canonical schema after complete
  reconstruction;
- `SemanticGraph::diff`, deterministic queries, and reports as clean-build
  equivalence oracles.

Private incoming/outgoing indexes need not be persisted: checked insertion
reconstructs them from canonical nodes and edges. Edge equality intentionally
ignores provenance, so exact cache evidence must compare provenance separately
or compare complete owned projections rather than relying on edge equality
alone.

### Closed graph variants

The current complete node content is a closed `GraphNodePayload` enum:

- no typed payload;
- metadata and metadata-member payloads;
- access-right payload and optional opaque row restriction;
- Data Composition Schema, Data Set, and Data Composition Field payloads;
- XDTO Type, HTTP URL Template/Method, and Web Service Operation/Parameter
  payloads.

The associated common, metadata, access-right, DCS, XDTO, HTTP, and Web Service
types expose typed getters and constructors. `NodeKind`, `MetadataKind`,
`EdgeKind`, `DataSetKind`, `XdtoTypeKind`,
`WebServiceParameterDirection`, `FactOrigin`, `Confidence`, and
`ResolutionState` are closed typed vocabularies. A complete codec can map them
through exhaustive matches; `Debug` or `Display` parsing and fixture-only
fallbacks would be unsupported.

`Provenance` exposes checked component values through `ProducerId`, source
`EntityId`, origin, confidence, and resolution state, and has a public
constructor. Node, edge, diagnostic, and reference-request provenance are
therefore observable. The cache contract must still decide whether encoded
provenance order is preserved exactly or normalized and verified.

### Diagnostics

`SemanticDiagnostic` exposes typed kind, code, message, source node, optional
target node, expected kinds, candidates, and provenance. Public constructors
and the resolution-error projection can create checked diagnostics. The current
Runtime fixture deliberately contains one missing Catalog query reference, so
the public EDT snapshot supplies non-empty recoverable diagnostic evidence;
Designer XML supplies the accepted empty diagnostic case.

### Reference requests: confirmed reconstruction gap

`SemanticReferenceRequestLedger::from_requests` deterministically orders
requests, merges equivalent provenance, and rejects conflicting duplicate
identity or terminal content. Each request exposes its source, category, typed
`SemanticReference`, expected kinds, candidate IDs, state, outcome, and combined
provenance.

Collected and partial requests have checked public constructors. Terminal
requests are created only by consuming a collected request through one of the
typed transition methods. The stored terminal request exposes combined
collection and resolution provenance, but no explicit stage partition. When
both stages use the same `ResolutionState`—for example a collected unresolved
request completed as missing—the public value does not provide an unambiguous
way to split that combined list back into the two inputs required by the
transition API.

Therefore a general Runtime-private decoder cannot prove exact round-trip
reconstruction of every valid terminal request using only the current public
API. ADR-0042 must choose a checked, source-independent remedy before Task 3,
such as a graph-domain reconstruction API or another evidence-backed complete
representation. Silently discarding requests, guessing a provenance split,
serializing private fields by layout, or reconstructing only the current fixture
would be lossy and unsafe.

### Statistics and reports

`SemanticReferenceStatistics` exposes every counter, a public `record` method,
derivation from canonical terminal requests, and combination with legacy
observations. Exact counters can be reconstructed without private field access,
including the EDT fixture's legacy observations.

`SemanticGraphReport` is a deterministic derived value constructed from graph,
diagnostics, and reference statistics. Its sections expose graph, node, edge,
diagnostic, resolution, and provenance-coverage counters. A decoder can
recompute and compare the report rather than trusting serialized derived
counters, but ADR-0042 must decide whether the persisted payload includes the
reported value, treats it only as an integrity check, or intentionally
reconstructs it.

### Workspace record construction

`WorkspaceConfigurationSnapshot` and `WorkspaceSnapshot` fields are private,
but a codec implemented inside the Runtime `workspace` module can construct
them after validation. `snapshot_from_parts` already centralizes Configuration
node cardinality and exact identity extraction for clean builds, but it is
private and currently accepts adapter build parts rather than persisted
evidence. ADR-0042 must decide whether reconstruction reuses/refactors this
function and how it verifies serialized root/format/identity/name against the
canonical Configuration node.

## Compatibility and semantic-validity inputs

No persisted schema has ever been released or committed. There is therefore no
repository-owned historical byte format from which to define a real field-level
migration. The evidence supports an explicit first schema plus deterministic
clean-rebuild replacement for absent or unsupported versions; it does not
support inventing a legacy schema solely to claim migration coverage.

A source-content key alone is insufficient. Equal source bytes built by changed
parser, graph, validation, normalization, or build-orchestration semantics could
produce different canonical state. The accepted cache identity must account for
both:

- complete relevant source state under the configured Workspace root;
- an explicit semantic-build compatibility input owned and changed with any
  behavior that can alter the complete snapshot;
- persisted schema version and encoding vocabulary;
- every Runtime option that the future ADR proves affects output;
- the relationship between configured root/path evidence and semantic identity.

The repository has no stable semantic-build version constant, automatic schema
compatibility marker, stable source digest API, or accepted rule for absolute
versus relative root paths. Git commit identity, package version `0.1.0`, file
timestamps, and process executable bytes are not current semantic contracts.

## Encoding and dependency evidence

`oneagent-runtime` already has locked production dependencies on
`serde 1.0.228` with derive support and `serde_json 1.0.150` for the accepted
HTTP projections. Runtime-private persisted DTOs can therefore be encoded with
existing approved dependencies if ADR-0042 accepts that option and the checked
domain reconstruction gap is closed.

`oneagent-common`, `oneagent-metadata`, and `oneagent-graph` do not depend on
Serde and their canonical types do not implement `Serialize` or `Deserialize`.
Adding Serde to those lower-level production crates, adding a binary codec,
checksum, cryptographic hash, compression, database, locking, or filesystem
helper would be a production dependency or API decision and requires explicit
approval where applicable. The live dependency graph proves no such addition is
required merely to investigate or to represent a Runtime-private JSON option;
ADR-0042 must still choose the complete encoding and validation boundary.

The accepted Graph Query wire vocabulary is intentionally a lossy projection:
it omits payloads, provenance, diagnostics, requests, reports, and roots.
Reusing that HTTP schema as cache state would violate ADR-0039 completeness and
cannot satisfy the Sprint 20 objective.

## Filesystem and replacement evidence

Runtime production code has no cache filesystem abstraction, atomic-write
helper, generation manager, stable temporary naming rule, fsync policy,
cross-platform replacement wrapper, path-containment validator, corrupt-entry
quarantine, or cleanup policy. Existing tests use `tempfile` only as a dev
dependency and use `std::fs` for disposable source mutations.

The repository CI matrix runs on `macos-14` and `windows-latest`. A plan that
assumes Unix rename-over-existing, open-file deletion, advisory locking,
permissions, or symlink behavior would lack current cross-platform proof. The
ADR must define a replacement sequence testable with `std` and disposable
directories, or identify an exact approved dependency and approval gate.

Required filesystem cases are observable with controlled test-owned paths:

- missing cache directory or candidate;
- directory creation and repeated existing directory;
- candidate is a directory or other wrong kind;
- complete current entry;
- truncated or malformed bytes;
- incompatible schema/build version;
- decoded but semantically invalid content;
- read/open/create/write/flush/rename/remove failures when deterministically
  injectable or portably constructible;
- stale temporary state and cleanup;
- interrupted replacement outcome;
- repeated fresh processes using the same disposable cache;
- source/cache path alias, containment, and symlink cases only to the extent the
  accepted platform-neutral evidence can prove them.

No current source proves cross-process concurrent writers, remote/shared cache,
security hardening, durability after power loss, eviction, or size limits.

## Runtime integration and observability evidence

The existing public observations are:

- `WorkspaceSnapshotObserver` for complete immutable publication;
- `WorkspaceUpdateObserver` with attempt, publication, phase, and failure kind;
- lifecycle-derived liveness/readiness;
- exact Graph Query responses over one selected snapshot;
- named startup/service failures and sender closure.

None distinguishes a cold build from a warm cache hit or exposes cache load and
write outcomes. Focused tests can inject a controlled detector/builder only
inside the current private Workspace module. A public test can compare complete
snapshots and queries across fresh runs, but cannot presently prove that a warm
run avoided semantic adapters. ADR-0042 must decide the minimum stable
in-process cache observation or testability seam required for that claim without
adding an HTTP/CLI cache API or making cache state a readiness authority.

The first integration also needs decisions for:

- scan-before-load and scan-after-load handling;
- source changes during load, build, encoding, or write;
- whether a cache write must finish before startup acknowledgement/publication;
- whether load/write failure is a startup failure, recoverable miss, update
  failure, or separate additive observation;
- when a successful File Watching rebuild becomes the new cache candidate;
- whether a failed write can coexist with a newly published valid snapshot;
- cancellation while load/write blocking work is active;
- cleanup and cache visibility after ordinary shutdown;
- preservation of Sprint 19 attempt/publication counters and failure meanings.

## Consumers and compatibility constraints

| Consumer or owner | Confirmed dependency on persisted behavior |
| --- | --- |
| Workspace snapshot observer | Must receive only one complete validated immutable snapshot or `None`; no partial cache value. |
| Graph Query service and HTTP | Must retain one-snapshot selection, canonical order, routes, bounds, success/error schemas, and lifecycle gating. |
| Health service | Must remain lifecycle-derived; cache status cannot become a second readiness label. |
| File Watching coordinator | Must retain complete-byte relevance, bounded latest revision, serialized builds, last-valid failure retention, recovery, and terminal cleanup. |
| EDT and Designer XML adapters | Remain semantic producers on clean builds and must not read Runtime cache state. |
| Graph domain | Remains canonical fact/validation authority; persistence cannot accept invalid nodes, payloads, edges, requests, or reports. |
| Sprint 21 CLI | May consume existing Runtime contracts later; no cache management or new transport is justified now. |

Current public constructors and HTTP behavior must remain source-compatible
unless ADR-0042 proves an additive Runtime configuration or observation surface.
No consumer currently depends on a cache file layout, so the first persisted
format can remain an explicitly versioned private Runtime contract.

## Repository-owned fixtures and deterministic oracles

The tracked `apps/runtime/tests/fixtures/workspace_service/` root provides:

- one EDT and one Designer XML production project with distinct canonical
  Configuration IDs;
- non-empty graph payload/provenance and ownership/call/reference facts;
- an EDT missing-Catalog request with terminal ledger, recoverable diagnostic,
  legacy reference observations, and a non-empty report;
- Designer XML empty diagnostic/request evidence;
- exact provenance and SHA-256 inventory in the fixture README.

Public tests already copy this fixture to temporary roots and exercise production
discovery, both builders, File Watching, Graph Query, health, cancellation, and
cleanup. Task 1 listed 52 Runtime unit tests and 21 non-zero public integration
tests:

- 2 File Watching tests;
- 3 Graph Query API tests;
- 4 HTTP health tests;
- 6 service-container tests;
- 6 Workspace service tests.

The following Sprint 20 oracles can be added without external data:

| Case | Deterministic oracle |
| --- | --- |
| Codec completeness | Encode a production mixed snapshot; decode; compare configuration fields, graph Diff, payloads/provenance, diagnostics, request ledger/statistics, report, validation, and stable re-encoding. |
| Cold miss/write | Empty disposable cache plus tracked source copy; observe one clean build, valid publication, and complete candidate creation. |
| Warm hit | Reuse the disposable source/cache and an accepted builder-call/cache-status seam; prove no adapter build and equal complete observation/query. |
| Source invalidation | Change exact fixture bytes/path/kind; prove old candidate is not used and the clean result matches the mutation. |
| Semantic/schema invalidation | Inject an accepted compatibility-version mismatch; prove typed rejection and clean replacement. |
| Corruption/partial | Mutate disposable cache bytes or structured fields; prove no publication from them and accepted recovery. |
| Write failure/interruption | Use an accepted injected storage port or portable wrong-kind/closed handle seam; prove no partial current entry and exact Runtime outcome. |
| File Watching replacement | Warm start, mutate copied source, observe one complete rebuild/publication/write, then start fresh and restore the new result. |
| Reader atomicity | Hold the previous `Arc` and issue Graph Query calls around replacement; observe complete old or complete new values only. |
| Shutdown/repetition | Close snapshot/update/cache observations as accepted, release resources, and repeat independent applications/cache roots with equal results. |

Timeouts may guard hangs. Poll periods, arbitrary sleeps, host-global cache state,
external services, ignored corpora, and network access are not acceptance
evidence.

## Confirmed unsupported or deferred behavior

- incremental graph, index, per-file, or per-configuration persistence;
- partial snapshot publication or adapter-specific cache authorities;
- cross-process concurrent writers, locks, leases, or shared caches;
- remote/network cache, compression, encryption, eviction, age/size policy, and
  user-facing cache maintenance;
- historical field-level migration without a real previous schema;
- stable public cache-file compatibility beyond the accepted version contract;
- cache HTTP, CLI, streaming, subscription, metrics, tracing, benchmark,
  performance, durability, or security certification;
- Git/network workspace ingestion, edits, restart, and forced termination.

## ADR-0042 decision matrix

ADR-0042 must close every row before implementation:

| Area | Required decision |
| --- | --- |
| Authority | Canonical snapshot/graph owner, persisted DTO owner, store owner, Runtime owner, dependency direction, and no-second-authority rule. |
| Complete payload | Exact envelope and inclusion/reconstruction of configuration, graph variants, provenance, diagnostics, requests, statistics, and report. |
| Reconstruction gap | Checked, source-independent way to reconstruct every terminal reference request without guessing provenance stages. |
| Schema and vocabulary | Magic/type identity, schema version, semantic-build compatibility version, exhaustive enum vocabulary, ordering, deterministic-byte promise, and future-version behavior. |
| Identity and invalidation | Complete source-state representation/digest, semantic compatibility inputs, configured-root treatment, options, ignored paths, and unverifiable-state behavior. |
| Location and containment | Default/override path, relation to watched root, enabled behavior, namespacing, path escape, existing kinds, symlinks, and permissions. |
| Load | Candidate selection, scan/load race closure, exact hit, miss, incompatible, corrupt, invalid, unreadable, and duplicate behavior. |
| Write and replacement | When writes occur, serialization/validation order, temporary naming, complete replacement, interruption, previous entry, cleanup, and cancellation. |
| Compatibility and migration | Current/older/newer/unknown versions, absence of a historical schema, clean-rebuild replacement, rollback, and downgrade claims. |
| Recovery | Startup and post-watch behavior for rejected loads, failed builds, failed writes, repeated failures, repair, and last-valid publication. |
| Runtime lifecycle | Blocking ownership, startup acknowledgement, update counters/status, watcher coalescing, Graph Query, health, cancellation, shutdown, and observer closure. |
| Observability/testability | Minimum typed in-process hit/miss/load/write evidence and deterministic builder/storage seams without a transport API. |
| Dependencies | Existing Serde/JSON option versus any lower-level Serde, digest, codec, or filesystem dependency and its approval gate. |
| Evidence | Non-zero focused and public EDT/Designer cold/warm/invalidation/corruption/failure/recovery/watch/query/cleanup/repetition matrix on macOS and Windows. |
| Deferred scope | Incremental, concurrent/shared/remote, compression/encryption/eviction, management APIs, CLI, later integrations, performance, and security. |

## Decision readiness

The repository contains enough production data, checked graph construction,
validation, complete snapshot evidence, source-state observation, consumers,
fixtures, platform coverage, and deterministic failure seams to decide a bounded
first Persistent Cache architecture and test it without external data.

Task 2 is ready only if it explicitly resolves the terminal-reference-request
reconstruction gap and the absent cache identity/location/replacement/
observability contracts. If it cannot close any row above with repository-owned
evidence, it must stop rather than invent a lossy payload, historical migration,
atomic filesystem guarantee, or public compatibility claim.
