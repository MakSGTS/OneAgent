# ADR-0061: Change Impact Analysis

## Status

Accepted.

## Context

Sprint 39 must expose one bounded product-facing explanation of the semantic
impact between two successive complete Workspace publications. The
[investigation](../architecture/change-impact-analysis-investigation.md)
confirms that Graph already owns canonical directional diff and impact
semantics, while Runtime currently publishes only the latest complete
`WorkspaceSnapshot` and retains no predecessor or transition report.

The existing MCP `oneagent.impact` operation compares two distinct
Configuration identities inside one immutable startup snapshot. It does not
describe a Configuration across publications. Filesystem and Git inputs only
request a complete rebuild and intentionally discard paths, statuses, baseline,
and operation order before semantic construction. Persistent cache schema `1`
contains one current complete semantic endpoint but no publication history.

This decision adds a product report without creating a second diff, dependency,
propagation, or impact authority. It preserves complete source discovery and
build, Graph validation, immutable atomic Workspace publication, Tool Policy,
MCP revision `2026-07-28`, and the seven-tool catalog.

## Decision

### Authority, owner, and dependency direction

`oneagent-analysis::change_impact` owns the source-independent product report,
its publication endpoint identity, Configuration transition vocabulary,
admission bounds, reconciliation, and closed errors. The accepted direction is:

```text
oneagent-graph diff and impact
    -> oneagent-analysis::change_impact product report
        -> oneagent-runtime Workspace publication
            -> MCP projection through Tool Policy
```

`oneagent-analysis` already depends on `oneagent-graph`, and Runtime already
depends on Analysis. No Cargo manifest, third-party package, feature, license,
unsafe-code, or reverse dependency change is accepted.

Graph remains the sole authority for semantic facts, node and edge identity,
diff, dependency classification, ownership traversal, propagation, impact
seeds, statuses, availability, reasons, and traversal completeness. Analysis
must call `SemanticGraph::diff` and `SemanticImpactAnalyzer`; it must not copy
or reinterpret either algorithm. Workspace remains the publication and source
lifecycle owner. MCP remains a projection and authorization boundary.

Diagnostics, Rules, Graph validation, build reports, and reference statistics
remain separate evidence. A diagnostic, rule result, source-format change,
path, Git status, or non-semantic build-report change is never an impact seed.
Diagnostic suppression does not apply to impact. An impact item is present,
excluded by requested depth, omitted by an explicit protocol limit, or rejected
with the whole report under this ADR; it is never suppressed.

### Canonical inputs and fixed impact policy

The report engine accepts exactly:

1. one process-local non-zero previous publication identifier plus the complete
   ordered previous Configuration inputs;
2. the immediately succeeding process-local publication identifier plus the
   complete ordered current Configuration inputs; and
3. a cooperative cancellation observer.

Each Configuration input contains only its canonical `EntityId` and borrowed
complete validated `SemanticGraph`. Names, roots, source formats, diagnostics,
reports, source fingerprints, cache metadata, update attempts, filesystem
events, and repository evidence are not engine inputs.

For every matched transition, Analysis computes the canonical Graph diff
itself and invokes Graph impact with maximum depth `4`, all Graph-default
dependency kinds, ownership disabled, and provenance changes direct-only. This
fixed policy matches the existing MCP defaults except for retaining the complete
depth-four product result. Callers cannot supply a diff, edge filter, ownership
mode, provenance mode, or alternate product depth. MCP may later project a
depth `0` through `4` view without rerunning or widening Graph semantics.

Analysis checks cancellation before normalization, before every Configuration,
after each Graph invocation, and before returning the report. Graph's individual
synchronous invocation is not interruptible; Runtime joins it before shutdown.
Cancellation returns no partial report and owns no task.

### Publication and Configuration identity

`ChangeImpactPublicationId` is a checked non-zero `u64` sequence local to one
running `WorkspaceService`. The initial successful publication is `1`; every
later successful publication increments it with checked arithmetic. Failed,
cancelled, stale, or rejected attempts consume no identifier. Identifiers reset
to `1` for a fresh service run, are not wall-clock time or global identity, and
are neither cached nor stable across processes.

A complete `ChangeImpactReport` is identified by the ordered adjacent pair
`(previous_publication_id, current_publication_id)`. One
`ConfigurationImpact` is further identified by its canonical Configuration
`EntityId`. No hash, UUID, path, name, format, Git commit, revision, or attempt
counter participates in identity.

Configuration inputs are normalized independently on each endpoint by
`EntityId`:

- exact duplicate `(EntityId, SemanticGraph)` inputs collapse;
- the same ID with different graph content on one endpoint is
  `ConflictingConfiguration`;
- an ID in both endpoints is `Compared` even when its name or source format
  changed outside the engine;
- a previous-only ID is `Removed` and is compared with one canonical empty
  graph as the current endpoint;
- a current-only ID is `Added` and is compared with the same canonical empty
  graph as the previous endpoint;
- an ID change is one removal plus one addition; continuity is never inferred
  from name, root, format, discovery order, path, or Git rename evidence; and
- an empty-to-empty Workspace produces a complete report with no
  Configuration transitions.

Equal graphs for a stable ID still produce a `Compared` transition with a
complete empty Graph impact result. Every successful equal Workspace rebuild
gets a new publication identifier and a distinct complete empty transition
report; the previous report is not reused.

### Report vocabulary and immutable content

Every published snapshot contains one immutable `WorkspaceChangeImpact` with
one of two closed states:

- `NoPreviousPublication { current_publication_id }`; or
- `Available(ChangeImpactReport)`.

`NoPreviousPublication` is availability, not an empty comparison. It is used
for the first cold build, a first warm cache hit, and every standalone
`WorkspaceSnapshotBuilder` result. It never invents an empty historical
endpoint.

An available report owns its adjacent publication IDs, canonical ordered
Configuration transitions, and a checked `ChangeImpactSummary`. Each transition
owns its Configuration ID, `Compared|Added|Removed` kind, the complete
depth-four Graph `SemanticImpactResult`, and a checked transition summary. The
Graph result remains the source of node status
`DirectlyChanged|TransitivelyAffected|Removed`, availability
`PreviousOnly|CurrentOnly|Both`, minimum depth, typed seeds and reasons, and
`CompleteWithinRequestedDepth`.

Product report completeness is the closed value
`CompleteWithinConfiguredDepth`. It means all admitted Configuration
transitions and all Graph results through fixed depth four are present. It does
not claim unbounded transitive closure. `NoPreviousPublication` has no
completeness value and is never called complete or truncated.

The in-memory report never truncates. Canonical total order is:

1. Configuration `EntityId`;
2. Graph-owned affected-node order by `NodeId`; and
3. Graph-owned reason order and deduplication.

Endpoint input order, discovery order, source format, trigger order, cache
load, and repeated construction cannot affect the report. Analysis does not
re-sort inside Graph-owned values with a competing comparator.

The report summary contains total Configurations and counts by
`Compared|Added|Removed`, aggregate seed-node and seed-edge changes, direct,
transitive, removed, previous-only, current, and total affected nodes, and the
maximum reached and configured depths. All additions and conversions are
checked. Transition counts reconcile to total Configurations; Graph category
counts reconcile to aggregate affected counts; any overflow is a whole-report
failure.

### Admission bounds and closed failures

The complete in-memory first slice accepts these exact inclusive limits:

| Item | Limit |
|---|---:|
| Unique Configurations in either endpoint | 4,096 |
| Configuration, node, or edge identifier bytes | 4,096 |
| Affected nodes in one complete report | 65,536 |
| Reasons for one affected node | 256 |
| Reasons in one complete report | 262,144 |
| Product traversal depth | 4 |

Bounds are checked before retaining report-owned values and aggregate counts
are checked while each canonical Configuration result is admitted. The engine
returns no partial, omitted, or truncated in-memory report.

Closed `ChangeImpactErrorKind` values are
`TooManyConfigurations`, `IdentifierTooLarge`, `TooManyAffectedNodes`,
`TooManyReasonsForNode`, `TooManyReasons`, `ConflictingConfiguration`,
`InconsistentGraphEvidence`, `SummaryOverflow`, and `Cancelled`. The public
error exposes only the kind and, where applicable, bounded `actual` and
`maximum` counts. It never echoes an identifier, name, path, source content,
provenance, Git value, raw Graph error, rejected value, or internal chain.

Graph's typed inconsistent-diff error is mapped to
`InconsistentGraphEvidence`. Because Analysis computes the diff from the same
borrowed graph pair, that result represents a closed invariant failure rather
than accepted partial evidence.

### Workspace computation, storage, and atomic publication

`WorkspaceSnapshot` adds a read-only publication identifier and embedded
`WorkspaceChangeImpact`. The standalone builder, cold startup, and warm startup
construct publication `1` with `NoPreviousPublication`. For a live update,
Runtime's existing serialized rebuild coordinator:

1. retains one `Arc` to the last successfully published snapshot;
2. constructs and validates the complete candidate snapshot by the existing
   discovery/build path;
3. assigns the checked next publication identifier;
4. builds the complete report from the retained predecessor and candidate;
5. verifies that the predecessor is still the current published value; and
6. performs one existing atomic `send_replace` of the candidate snapshot and
   its embedded report.

No observer can see a new graph with an old report or a report without its
current graph. After publication, Runtime retains no predecessor or history
for impact; the report owns the required Graph result, and ordinary external
`Arc` clones keep only snapshots explicitly retained by consumers.

Discovery, build, validation, cache, impact-bound, summary, cancellation, or
stale-predecessor failure publishes neither candidate nor report, consumes no
publication ID, and retains the last valid snapshot. Update status records its
existing bounded failure category; raw Analysis failures are not exposed.
A later successful request compares against that last successful publication,
not against a failed attempt, and is normal recovery.

The coordinator remains single-writer. A change arriving during computation is
coalesced by the existing capacity-one follow-up rule. A completed candidate is
a valid adjacent publication even if a later request is pending; the follow-up
then compares against it. Any impossible predecessor mismatch discards the
stale result and fails closed. There is no speculative publication, detached
impact task, mutable global state, lock-shared report, or report observer
separate from the snapshot.

Shutdown closes inputs, joins the source and any active bounded blocking work,
clears the snapshot, publishes the existing stopped state, and closes
observers. Already cloned snapshots remain immutable. A repeated fresh service
owns fresh channels, starts at publication `1`, and has no predecessor from a
prior run.

### Persistent cache

Transition reports and publication identifiers are not serialized. Cache
schema remains `1`; semantic compatibility advances from `4` to `5` because a
decoded `WorkspaceSnapshot` gains the new publication/report behavior. Older,
newer, corrupt, checksum-invalid, source-stale, or otherwise incompatible
entries follow the existing invalidation and clean-rebuild path.

Cold build and accepted version-`5` warm decode reconstruct the same complete
semantic Configuration content and both publish ID `1` with
`NoPreviousPublication`. Derived validation, rules, and diagnostics retain
their current recomputation behavior. A later live publication computes impact
from the actually published current Arc, regardless of whether that Arc came
from cold construction or cache. Cache read/write failure preserves the
existing failure/recovery policy and never manufactures history.

Persisting a transition, two snapshots, a publication sequence, or history log
requires a future schema and validity decision and is not accepted here.

### Filesystem and Git input equivalence

Impact is a pure function of the two complete admitted semantic endpoint sets.
If filesystem observation and ADR-0060 Git input cause equal previous/current
Configuration graphs, they must produce equal report content apart from the
process-local numeric publication IDs.

Repository root, relative path, status, baseline commit, endpoint completeness,
change-set order, staged/unstaged origin, untracked state, source identity, and
filesystem event order must not enter Configuration matching, Graph diff,
impact seeds or reasons, report summaries, failures, cache state, or wire
output. Non-empty Git input remains only a source-neutral request for a complete
rebuild. Equal semantic end states produce a complete empty transition even
when the triggering evidence was non-empty.

### Compatible MCP `oneagent.impact` contract

The catalog remains exactly seven lexicographically ordered tools,
`capabilities.tools={}` remains truthful and unchanged, and MCP revision
`2026-07-28`, framing, annotations, errors, and Tool Policy actor, request,
revision, effects, policy, and output bounds remain unchanged. No new tool or
output schema is advertised.

`oneagent.impact` has two explicit request modes:

- legacy mode requires distinct `previousConfigurationId` and
  `currentConfigurationId`, accepts optional `maxDepth` `0..=4` and `limit`
  `1..=100`, and preserves the ADR-0051 request validation, same-snapshot Graph
  computation, result JSON, error behavior, defaults, and field order exactly;
- publication mode requires exactly `configurationId`, accepts optional
  `maxDepth` `0..=4` default `1`, `limit` `1..=100` default `50`, and
  `reasonLimit` `1..=100` default `50`, and reads the latest embedded report.

The schema advertises both alternatives. Mixed selectors, neither selector,
one legacy selector, equal legacy selectors, mode-inapplicable fields, unknown
fields, malformed values, and out-of-range values are `invalid_arguments`.
Consequently the externally exercised `{}` negative request remains invalid.

Publication mode finds the requested Configuration in the report's union of
previous and current IDs, so a removed Configuration remains queryable until
the next publication. On `NoPreviousPublication`, an ID present in the current
snapshot returns a successful unavailable result; an absent ID is `not_found`.
An available report with no matching transition is `not_found`.

The publication result has closed `mode="publication"`, publication IDs when
available, `configurationId`, availability
`available|no_previous_publication`, transition
`compared|added|removed` when available, completeness
`complete_within_requested_depth` when available, requested/configured depths,
the complete depth-filtered transition summary, ordered affected nodes with
Graph status and availability, bounded ordered reasons, `total`, `truncated`,
and `omittedReasons`. `total` and summary describe the complete requested-depth
view before protocol limits. `truncated` means affected nodes were omitted by
`limit`; `omittedReasons` is the checked count omitted by `reasonLimit` across
returned nodes. Item and reason truncation never claims report incompleteness.
The unavailable result contains no invented previous ID, transition, summary,
node, reason, total, or completeness.

Depth projection includes an affected node only when its minimum depth is at
most the request and includes only reasons whose depth is at most the request.
It recomputes the view summary with checked arithmetic from the complete owned
Graph result; it never reruns traversal. Protocol item/reason truncation is
applied only after that complete view. A value that still exceeds Tool Policy's
65,536-byte output bound fails closed as existing `result_too_large`; strings
are never byte-truncated.

All known calls, including both impact modes, pass the existing exact Tool
Policy evaluation and `execute_tool` path. Denial remains `policy_denied`;
failed, partial, timed-out, cancelled, malformed-output, or domain execution
remains a stable closed tool error. No result or error contains root,
provenance path, source content, source format, repository value, policy
internal, or raw error chain.

### MCP process and supported-consumer migration

The public `oneagent-mcp` process changes from an immutable startup snapshot to
one Runtime-owned live `WorkspaceService` plus the existing sequential stdio
transport. Runtime adds an observer-backed semantic-server constructor. The
handler clones exactly one current `Arc<WorkspaceSnapshot>` at the beginning of
each call, so every call is internally immutable and atomic while later calls
may observe a newer publication. All seven tools use that same per-call rule;
legacy impact still compares two Configurations in one cloned snapshot.

Startup waits for one complete initial Workspace publication before reading
frames. Initial build failure retains the existing bounded non-zero exit.
Workspace failure after startup retains the last valid snapshot and does not
terminate protocol service. EOF, Ctrl-C cancellation, transport failure, and
Workspace failure use one structured Runtime ownership tree; cancellation is
propagated, all tasks are joined, and no watcher, channel, or blocking job is
detached.

The existing public `semantic_server(WorkspaceSnapshot)` constructor remains a
supported immutable in-memory boundary for tests and embedders. Existing
discovery, catalog, legacy requests/results, and static-constructor behavior
remain compatible. VS Code's revision and seven-name assertions require no
change. External Codex/Cursor catalog evidence remains valid, and its `{}`
negative impact call remains invalid. The deliberate observable migration is
that the executable may expose newer complete Workspace publications between
calls; clients that require a fixed view must keep using the immutable
constructor or a single call.

HTTP, CLI, LSP, EDT, Graph Query, providers, source adapters, diagnostics,
rules, Coverage Registries, and their wires do not gain an impact operation or
change behavior in Sprint 39.

### Acceptance evidence and task ownership

Task 3 implements only `oneagent-analysis::change_impact`: typed publication,
transition, report, summary, completeness, bounds, cancellation, failures,
Graph reuse, deterministic order, and focused domain tests. It changes no
Workspace, cache, MCP, or process behavior.

Task 4 embeds the report in Workspace snapshots and implements Configuration
matching composition, publication numbering, atomic replacement,
failure/recovery, semantic cache version `5`, filesystem/Git equivalence,
cancellation/shutdown, and repeated-service evidence. It changes no MCP schema
or projection.

Task 5 adds the compatible publication selector and projection, retains exact
legacy behavior, routes both modes through Tool Policy, adds observer-backed
composition, and migrates the public MCP executable with in-memory and real-
process evidence.

Task 6 runs the complete Graph/Analysis/Workspace/cache/input/MCP/process and
workspace validation matrix; audits dependency, public API, compatibility,
sensitive data, scope, and ignored/filtered tests; and synchronizes only the
accepted current-state documentation.

Repository-owned evidence must cover:

- empty/equal, direct, transitive, removed, added, every status/availability/
  reason, reordered/repeated, duplicate/conflict, exact/over bound, summary,
  cancellation, failure redaction, and Graph-equivalence cases;
- stable ID, name change, format change, previous/current-only, empty and
  multiple Configurations, ID change without rename inference, and duplicates;
- initial/warm no-predecessor, equal publication, atomic observation, failed
  attempt retention, recovery, coalesced follow-up, stale rejection,
  cancellation, shutdown, Arc retention, and fresh service;
- cache version/corruption/source invalidation/write failure and cold/warm/
  rebuild equality;
- filesystem/Git equivalent complete endpoints and absence of repository
  evidence from all reports and failures;
- both MCP modes, unavailable/equal/removed reports, schema/catalog truth,
  every malformed combination and exact/over bound, deterministic JSON, Tool
  Policy allow/deny/failure, output-bound failure, channel purity, EOF,
  cancellation, live replacement, and fresh process; and
- Graph, Analysis, Runtime, diagnostics, rules, cache, HTTP, CLI, MCP, LSP,
  VS Code, EDT, dependencies/features/licenses/unsafe, public API/Rustdoc,
  Coverage, and current-state documentation compatibility.

No acceptance depends on a live external client, network, credential, user
repository, remote transport, GUI process, real signal, or unsupported source
format.

## Consequences

OneAgent gains one deterministic report for the latest successful semantic
transition, paired atomically with its current snapshot. It intentionally
retains no historical query service. Complete report admission can reject a
Workspace update after semantic construction; the last valid publication stays
observable and a later bounded update can recover.

The public MCP executable becomes live across calls while every call remains
immutable. The catalog, protocol, policy, legacy impact mode, and static server
constructor remain compatible. Cache storage does not grow, but semantic
compatibility version `4` entries rebuild once under version `5`.

## Rejected alternatives

- Runtime-owned product semantics would couple a reusable report to lifecycle
  code and duplicate the established Analysis reporting boundary.
- Graph-owned publication identity, Configuration matching, report summaries,
  or protocol bounds would make Graph a product/history authority.
- A caller-supplied diff would permit graph-pair/diff disagreement and create a
  second freshness contract outside Graph.
- Matching by name, root, format, path, or Git rename/status evidence is not
  canonical semantic identity.
- A separate report observer cannot guarantee an atomic graph/report pair
  without recreating snapshot identity and publication ownership.
- Retaining two snapshots, computing on demand, or storing a history log adds
  memory lifetime, selection, eviction, and stale-result contracts absent from
  the bounded objective.
- Treating startup or a cache hit as empty-to-current fabricates an unobserved
  product transition.
- Serializing the latest report cannot prove that its prior endpoint is the
  predecessor of a new process's current source state.
- Partial in-memory reports or silent reason omission make completeness and
  summary reconciliation ambiguous.
- Replacing the legacy two-ID request or adding a new tool breaks accepted
  clients and catalog evidence without necessity.
- Keeping the executable immutable would make the product report unreachable
  after a successor publication.
- Using Git paths/statuses as seeds would bypass complete source construction
  and Graph authority.

## Deferred scope

Selective or incremental parsing, Graph mutation or report maintenance;
changed-path-to-node inference; new Graph facts, edges, dependency kinds,
propagation, ownership, or provenance policy; diagnostic/rule production or
suppression; scoring, probability, risk prediction, prioritization, telemetry,
or benchmarks; persisted history or cross-process publication identity;
history queries, arbitrary endpoint selection, cross-workspace or remote
impact; mutable documents; refactoring plans, code actions, source edits,
transactions, rollback; Git mutation, remote access, submodule traversal, or
repository UI; new HTTP, CLI, LSP, VS Code, EDT, or product UI; concurrent MCP
dispatch, progress, per-client sessions, or cancellation notifications; and
broad performance, security, or interoperability claims remain outside Sprint
39.
