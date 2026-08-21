# Sprint 20 Persistent Cache Integration Review

## Decision

`pass`

Sprint 20 satisfies ADR-0042 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 21 CLI
Client may become the unique `next` target.

## Reviewed baseline

- Planning parent: `9694d3e81f20376660ce67f1d64d002ffbabe92b`
- Reviewed range: `af52b579^..c6e69721`
- Committed Task 6 head: `c6e6972105b4c09668682a2a5ff6642a5b892c22`
- Review date: 2026-08-22

| Commit | Subject | Owned paths |
| --- | --- | --- |
| `af52b579238843acc7cd393f4eb679f8a6384bfa` | `Plan Sprint 20 Persistent Cache` | `docs/Roadmap.md`; the eight files in `docs/codex/prompts/sprint-20-persistent-cache/` |
| `41efa71ae1560ba3f62648c7b616c98fc4322b2c` | `Investigate Sprint 20 Persistent Cache` | `docs/architecture/persistent-cache-investigation.md` |
| `251f0e1aa7c4ee357d0c92ba54ea8d902b9a9ad8` | `Define Sprint 20 Persistent Cache contract` | `docs/adr/0042-persistent-cache.md` |
| `33ba1084e585f58cf6dfb51687cedf852247c07d` | `Implement Sprint 20 snapshot cache codec` | `apps/runtime/src/workspace/cache.rs`, `apps/runtime/src/workspace/mod.rs`, `crates/graph/src/reference_request.rs` |
| `6e48e8ded0ad4bfb8432e6a50a3b9fcc0aea254b` | `Implement Sprint 20 cache storage and invalidation` | `apps/runtime/src/workspace/cache.rs`, `apps/runtime/src/workspace/change.rs` |
| `31cbaea3097f9ef7285cc1a381767aae62981b84` | `Integrate Sprint 20 Runtime cache lifecycle` | `apps/runtime/src/http/mod.rs`, `apps/runtime/src/lib.rs`, `apps/runtime/src/workspace/cache.rs`, `apps/runtime/src/workspace/mod.rs`, `apps/runtime/tests/graph_query_api.rs`, `apps/runtime/tests/workspace_service.rs` |
| `c6e6972105b4c09668682a2a5ff6642a5b892c22` | `Complete Sprint 20 Persistent Cache evidence` | `README.md`, `apps/runtime/tests/persistent_cache.rs`, `docs/Architecture.md`, `docs/architecture/semantic-model-2.md` |

The range changes only the planned private cache codec/store, Workspace Runtime
integration and observation, the minimum graph reconstruction surface, public
production evidence, ADR/investigation/current-state documents, Roadmap
planning, and the Sprint 20 prompt suite. No Cargo manifest or lockfile changed,
so no production dependency required approval. The range changes no source
parser, canonical graph fact vocabulary, public Graph Query or health wire
schema, supported CLI behavior, protocol authority, or later-sprint behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The committed plan resolves Sprint 20 from the completed Sprint 19 baseline, proves the Persistent State framework ready, orders seven prerequisite-gated tasks, and preserves the exact Sprint 19 retirement gate. | pass |
| Investigation | The repository-backed investigation inventories canonical snapshot content, reconstruction and validation APIs, source identity, Runtime and watcher ownership, filesystem behavior, consumers, dependencies, fixtures, platform constraints, and deterministic seams before selecting a contract. | pass |
| Accepted architecture | ADR-0042 fixes authority, ownership, envelope and payload, versions, identity, storage, typed outcomes, recovery, Runtime sequencing, evidence, rejected alternatives, and deferred scope. | pass |
| Canonical authority and schema | `WorkspaceSnapshot` and graph-domain APIs remain canonical. The private `oneagent.workspace-cache` envelope uses explicit schema and semantic versions and cannot become a second semantic authority. | pass |
| Variant completeness | Focused exhaustive tests cover every current graph node payload, edge kind, provenance value, diagnostic value, semantic reference, request outcome, statistics field, and EDT/Designer configuration format. | pass |
| Checked reconstruction | Decoding rebuilds checked IDs, names, payloads, nodes, edges, requests, statistics, reports, and configuration snapshots through domain APIs rather than deserializing private fields. | pass |
| Complete validation | Reconstructed graphs, diagnostics, request ledgers, legacy observations, reports, and complete configuration snapshots pass the canonical validator and snapshot constructor before a hit can be returned. | pass |
| Deterministic bytes | Canonical ordering, strict JSON DTOs, a content checksum, byte-for-byte canonical re-encoding checks, repeated encoding, and decode/re-encode evidence make equal accepted inputs stable. | pass |
| Identity and invalidation | The exact sorted complete source state includes relative paths, entry kinds, and regular-file bytes; source mismatch and schema or semantic-version mismatch are distinct typed rejections. | pass |
| Path containment | The fixed `.oneagent/cache/workspace-v1.json` owner remains under the Workspace root; source paths are relative and checked, and owner/candidate symlinks or wrong-kind components are rejected. | pass |
| Replacement and cleanup | Writes encode and self-decode first, create only real directories, use an exclusive temporary file, sync and read it back, replace only a regular candidate, and clean stale or failed temporary state. Focused failure injection covers every replacement stage. | pass |
| Compatibility and corruption | Missing, changed, incompatible, corrupt, partial, noncanonical, checksum-invalid, semantically invalid, and unavailable candidates map to the accepted closed outcomes without publication. | pass |
| Recovery | Every rejected load falls back to a clean production build. Write failure is nonfatal, does not expose partial bytes, and a later clean run restores a reusable hit. | pass |
| Runtime cold and warm startup | Startup performs S0/load/S1, accepts a hit only across equal observations, otherwise builds once and performs S2 before writing or scheduling a follow-up. Public evidence proves cold miss/write and warm hit equivalence without an adapter rebuild. | pass |
| File Watching integration | Rebuilds perform R0/build/R1, persist only stable complete results, preserve bounded serialized follow-up behavior, and exclude the complete `.oneagent` subtree from source observation. | pass |
| Publication atomicity | Cache work completes before one immutable `Arc` replacement; readers retain an old complete snapshot or acquire the new complete snapshot, and failed semantic rebuilds retain the last valid publication. | pass |
| Query and health compatibility | Public raw-loopback evidence preserves Graph Query responses, errors, lifecycle gating, single-snapshot ownership, HTTP liveness/readiness, and listener ownership across cold, warm, rejected, and watched-cache runs. | pass |
| Failure handling | Cache load rejection and write failure remain contained cache status, while source/build failures retain their accepted Workspace status and last-valid behavior. Cache state never changes readiness authority. | pass |
| Cancellation and cleanup | Cancellation joins source and blocking rebuild work, prevents post-cancellation publication, clears the snapshot, closes update/cache observation, releases HTTP, and preserves only the complete reusable cache entry. | pass |
| Public EDT/Designer evidence | The dedicated four-test public target copies the tracked mixed fixture and exercises production detection, both builders, cache I/O, file observation, Workspace publication, Graph Query, health, shutdown, and fresh reuse. | pass |
| Platform behavior | Paths use checked `Path` components and existing portable `std` filesystem operations. Tests use disposable roots, loopback address publication, no arbitrary sleeps, and repository CI retains macOS and Windows targets. | pass |
| Dependency approval | `Cargo.toml` and `Cargo.lock` are unchanged in the reviewed range; the implementation uses existing `serde_json`, Tokio, and standard-library facilities. | pass |
| Fixture provenance | The public target copies only the tracked Sprint 17 mixed EDT/Designer fixture; its README records origin, reduction policy, and SHA-256 inventory, which remains unchanged. | pass |
| Repetition and ownership | Repeated fresh cold/warm/recovery/watch runs produce equal observations, close owned receivers, clear publication, release listeners, and leave no shared Runtime owner. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap, investigation, ADR-0042, and public evidence agree on the implemented private first slice and the CLI hand-off. | pass |
| Scope containment | The range adds no incremental persistence, partial publication, cache API, cross-process or remote cache, compression, encryption, eviction, migration machinery, CLI behavior, benchmark, or unsupported performance/security claim. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

The review counted only non-zero focused targets. The required matrix contains
18 codec/store tests, 6 cache-lifecycle tests, 7 Workspace-service lifecycle
tests, 4 public Persistent Cache tests, 6 public Workspace tests, 2 public File
Watching tests, 3 public Graph Query tests, and 4 public health tests.

## Validation

The review reran the focused Runtime matrix:

- `cargo test -p oneagent-runtime --lib workspace::cache::tests -- --nocapture` — 18 passed.
- `cargo test -p oneagent-runtime --lib workspace_cache_ -- --nocapture` — 6 passed.
- `cargo test -p oneagent-runtime --lib workspace_service_ -- --nocapture` — 7 passed.
- `cargo test -p oneagent-runtime --test persistent_cache -- --nocapture` — 4 passed.
- `cargo test -p oneagent-runtime --test workspace_service -- --nocapture` — 6 passed.
- `cargo test -p oneagent-runtime --test file_watching -- --nocapture` — 2 passed.
- `cargo test -p oneagent-runtime --test graph_query_api -- --nocapture` — 3 passed.
- `cargo test -p oneagent-runtime --test http_health -- --nocapture` — 4 passed.
- `cargo test -p oneagent-runtime` — 78 library and 25 public integration tests
  passed, including 6 service-container tests; zero-test binary and doctest
  harnesses also completed successfully.

The canonical complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies loopback bind without additional local permission;
targets containing local TCP evidence ran with bounded loopback permission. No
external network or service was used.

## Deferred scope

Incremental graph or index persistence, partial configuration publication,
per-file repair, cross-process writers and locking, shared or remote cache,
compression, encryption, eviction, retention policy, user-facing cache
management, automatic historical migration, native notifications, Git/network
workspaces, dynamic configuration, authentication, metrics, benchmarks, and
performance/security certification remain deferred. Supported CLI behavior is
owned by Sprint 21.

## Risk assessment

Residual risk is bounded to the accepted first slice. Complete-byte source
identity and complete snapshot replacement favor correctness over large-
workspace performance; the cache uses one process-owned entry without locking;
and cache outcomes have no HTTP projection. The local review ran on macOS while
Windows behavior remains guarded by portable APIs, Windows-compatible path
tests, and repository CI. These accepted limits do not block ADR-0042.

## Previous-suite retirement

After the `pass` decision, `git ls-files` and the filesystem both contained
exactly the seven planned Sprint 19 prompt files and no additional or untracked
file. Repository search found no retained Markdown link dependency on an
individual deleted prompt. The exact suite is retired atomically with this
review; the complete Sprint 20 suite,
`docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and `.codex/`
remain unchanged.
