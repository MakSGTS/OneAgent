# Sprint 19 File Watching Integration Review

## Decision

`pass`

Sprint 19 satisfies ADR-0041 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 20
Persistent Cache may become the unique `next` target.

## Reviewed baseline

- Planning parent: `ee0be7b4803651a6631c80493aa841b7b14e5e41`
- Reviewed range: `6e220073^..cadc4b9f`
- Committed Task 5 head: `bba17c620413d573246cd0f63157bad848e5e1cc`
- Corrective evidence head: `cadc4b9f3e20e4da94df3ec91223c98f60255385`
- Review date: 2026-08-21

| Commit | Outcome |
| --- | --- |
| `6e220073c0b90f36e1997ed49898e6cb32f7b6d0` | Sprint 19 execution plan and ordered prompt suite |
| `7240bc17696ace0e9d7191f86ec69922ba3402b8` | Live filesystem, Workspace, Runtime, dependency, platform, fixture, and deterministic-test investigation |
| `f94976a9ca609f338c67bec4e792f92854e9ede6` | Accepted ADR-0041 File Watching and Workspace rebuild contract |
| `bb872694c8594000cfe78eef2f4091db97d7456c` | Portable complete-byte filesystem state source, normalized latest-revision channel, cancellation, and focused evidence |
| `051c5845bfdd6f6ea1e1c029f68dcbc3083346aa` | Startup race closure, serialized complete rebuilds, atomic publication, update status, failure recovery, and cleanup |
| `bba17c620413d573246cd0f63157bad848e5e1cc` | Public EDT/Designer XML File Watching evidence and current-state documentation |
| `cadc4b9f3e20e4da94df3ec91223c98f60255385` | Corrective public in-flight, observation failure, readiness, receiver-closure, and deterministic evidence clarification |

The range changes only the planned Runtime watcher/rebuild surface, additive
update observation, public Runtime evidence, ADR/investigation/current-state
documents, Roadmap planning, and the Sprint 19 prompt suite. It adds no
production dependency and changes no canonical graph fact, source-adapter
parser, Graph Query or health wire schema, persistence, supported CLI,
protocol-crate authority, or later-sprint behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The committed plan resolves Sprint 19 as the unique eligible target, proves the existing Runtime Service framework sufficient, orders six prerequisite-gated tasks, and preserves the exact Sprint 18 retirement gate. | pass |
| Investigation completeness | The investigation records live discovery/read behavior, build/publication ownership, lifecycle, consumers, dependency and platform constraints, fixture mutation oracles, deterministic seams, relevance limits, and every ADR question before implementation. | pass |
| Accepted architecture | ADR-0041 fixes semantic authority, dependency choice, watched state, normalized outcomes, startup race closure, bounded coalescing, serialization, publication, failure/recovery, status, lifecycle, compatibility, evidence, and deferred scope. | pass |
| Watcher ownership | `WorkspaceService` owns one source task, one capacity-one watch channel, the coordinator, every blocking build handle, status/snapshot senders, cancellation, and terminal cleanup; no detached watcher service or global registry exists. | pass |
| Relevant state and normalization | Sorted recursive scans retain normalized relative paths, entry kinds, and complete regular-file bytes, do not follow symlinks, exclude exactly five accepted directory names, and emit only `Changed` or `ObservationFailed` revisions. | pass |
| Failure normalization | The first failed scan emits one typed failure, equivalent repeated failures do not loop, the last successful state is retained, and the first later successful scan conservatively emits `Changed`. | pass |
| Startup race closure | Baseline scan, complete initial build, post-build scan, initial publication, and an immediate changed revision for unequal scans prevent a persistent startup mutation from being lost. | pass |
| Coalescing and serialization | A private single-value revision watch bounds queued work. Focused gated evidence proves one active build and one latest-state follow-up; public status proves a post-`Rebuilding` change and a multi-entry project-tree addition have the accepted attempt/publication counts. | pass |
| Publication atomicity | Only complete validated builder results replace the snapshot with one new `Arc`; held old values remain immutable, failed work publishes nothing, and readers observe a complete old or complete new snapshot. | pass |
| Failure retention and recovery | Semantic corruption and root observation failure retain the last valid queryable snapshot with exact typed status; later repair or root restoration produces one successful complete replacement. | pass |
| Lifecycle and health | Initial failures remain named startup failures, recoverable updates leave Runtime `Running`, readiness remains lifecycle-derived and ready during retained-snapshot failure, and reverse shutdown preserves the accepted service order. | pass |
| Graph Query compatibility | Requests continue to acquire exactly one immutable snapshot and retain the Sprint 18 routes, methods, schemas, errors, bounds, ordering, lifecycle gating, and transport ownership. | pass |
| Public EDT/Designer evidence | The two-test public target copies the tracked provenance fixture and traverses production polling, discovery, both production builders, immutable publication, raw loopback Graph Query, failure/recovery, and shutdown. | pass |
| Deterministic negative evidence | Controlled ticks prove equal scans, ignored changes, failure/recovery, and closure without using the production period as a test oracle; a gated complete builder proves active-build coalescing without fixed sleeps. | pass |
| Platform behavior | The implementation uses existing `std` filesystem and Tokio APIs, contains no native event vocabulary or absolute platform path, and the repository CI gate targets both `macos-14` and `windows-latest`. | pass |
| Dependency approval | No Cargo manifest or lockfile changes occur in the reviewed range; no production dependency required approval. | pass |
| Fixture provenance | Public tests mutate only disposable copies of the tracked Sprint 17 EDT/Designer fixture whose README records source provenance, reduction policy, and SHA-256 inventory. | pass |
| Cancellation and cleanup | Cancellation stops scheduling, joins the source and any started build, prevents post-cancellation publication, clears the snapshot, publishes `Stopped`, closes snapshot/update receivers, releases HTTP, and permits immediate rebind. | pass |
| Repetition | Fresh public applications return equal semantic and status observations, close all owned channels, clear publication, and leave no shared owner or listener. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap planning, investigation, ADR, and public evidence agree on the implemented bounded polling/rebuild contract and keep persistence, supported CLI, and later integrations deferred. | pass |
| Scope containment | The range adds no native watcher, incremental graph mutation, cache, source-format or graph semantics, watch-control route, streaming, Git/network workspace, authentication, benchmark, or unsupported performance/security claim. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

An initial Task 6 prerequisite audit found that the first Task 5 public target
did not directly expose observation failure/readiness, post-`Rebuilding`
follow-up counts, or receiver closure. The separately committed corrective
evidence at `cadc4b9f` added those public assertions and documented why exact
ignored-change and active-build negatives use deterministic focused seams
instead of the production polling period. The complete review reran only after
that corrective commit; no review finding was silently fixed in the review
change.

Zero-match outputs from unrelated binaries were not counted. The named focused
review inventory contains 31 distinct non-zero tests: 4 change-source tests, 6
Workspace service orchestration tests, 2 public File Watching tests, 6 public
Workspace tests, 3 public Graph Query tests, 4 public health tests, and 6 public
service-container tests. The complete workspace inventory contains 811 tests.

## Validation

The review reran the complete focused Runtime matrix:

- `cargo test -p oneagent-runtime workspace::change::tests -- --list` — exactly 4 tests.
- `cargo test -p oneagent-runtime workspace::tests::workspace_service_ -- --list` — exactly 6 tests.
- `cargo test -p oneagent-runtime --test file_watching -- --list` — exactly 2 tests.
- `cargo test -p oneagent-runtime --test workspace_service -- --list` — exactly 6 tests.
- `cargo test -p oneagent-runtime --test graph_query_api -- --list` — exactly 3 tests.
- `cargo test -p oneagent-runtime --test http_health -- --list` — exactly 4 tests.
- `cargo test -p oneagent-runtime --test service_container -- --list` — exactly 6 tests.
- `cargo test -p oneagent-runtime` — 52 unit and 21 public integration tests passed.

The complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace` — all 811 listed tests passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies loopback bind with `PermissionDenied`; all focused
and complete targets containing local TCP evidence ran with bounded local-
network permission. No external network or service was used.

## Deferred scope

Persistent cache schema, deterministic invalidation, versioning, migration,
corruption, recovery, and clean-rebuild equivalence remain Sprint 20. Supported
CLI update behavior remains Sprint 21. Native notification backends,
configurable polling, incremental semantic mutation, stable generation IDs,
changed-configuration sets, watch-control routes, streaming, Git/network
workspaces, protocol/IDE/AI surfaces, authentication, metrics, benchmarks, and
performance/security certification remain outside Sprint 19.

## Risk assessment

Residual risk is bounded to accepted first-slice limits. Complete-byte polling
and complete rebuilds favor correctness over large-workspace performance, the
250 millisecond schedule is internal and not configurable, and recoverable
failure status has no HTTP projection. These accepted limits do not block the
Sprint 19 contract and remain explicit future work.

## Previous-suite retirement

After the `pass` decision and successful focused/full gates, `git ls-files` and
the filesystem both contained exactly the seven planned Sprint 18 prompt files
and no additional or untracked file. Repository search found no retained
Markdown link dependency on an individual deleted file. The exact suite is
retired atomically with this review; the complete Sprint 19 suite,
`docs/codex/prompts/run-next-sprint.md`, all older non-adjacent suites, and
`.codex/` remain unchanged.
