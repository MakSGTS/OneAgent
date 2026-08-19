# Sprint 4 Semantic Index Integration Review

## Review status

Pass recorded on 2026-08-19 against committed Task 4 head
`58fb7566221a1ca35aa20d658a76f32a1a69b4fc`.

Sprint 4 satisfies the accepted complete-snapshot Semantic Index boundary in
[ADR-0026](../adr/0026-semantic-index-boundary.md). No blocking or non-blocking
findings, missing acceptance evidence, or unresolved review questions remain.

## Reviewed baseline and range

The pre-implementation baseline is
`a0a5d7449b5f48a293b8f272434a0f243995c497`, the parent of Task 1. The reviewed
range is:

```text
a0a5d7449b5f48a293b8f272434a0f243995c497..58fb7566221a1ca35aa20d658a76f32a1a69b4fc
```

The implementation commits are dependency ordered:

| Task | Commit | Message |
|---|---|---|
| 1 | `25fc6e0c0656a35a95e9d7576b9f18ef3e61c07a` | `Add semantic snapshot identity index` |
| 2 | `efa2d8758a96f3d4fbde0aff8d017ec77a0d5b55` | `Add semantic adjacency and containment index` |
| 3 | `3507ded15c3ec7f0a239bc195331b9bc79bd095b` | `Migrate semantic resolution to shared index` |
| 4 | `58fb7566221a1ca35aa20d658a76f32a1a69b4fc` | `Migrate semantic queries to shared index` |

The range contains only the expected graph implementation, graph tests, and
Sprint 4 activation update. Before the review record was created, the working
tree contained only the pre-existing untracked `docs/codex/prompts/` directory;
it is excluded from this review and its commit.

## Acceptance evidence matrix

| Area | Repository and executed evidence | Result |
|---|---|---|
| Authority | `SemanticGraph` still owns canonical nodes, edges, provenance, mutation, and validation input. `SemanticIndex` stores borrowed `GraphNode` and `GraphEdge` references only. | pass |
| Representation | `crates/graph/src/semantic_index.rs` defines the only shared crate-internal lookup representation. Query and Resolution both delegate to it; Resolution-private lookup maps were removed. | pass |
| Identity and classification | `semantic_index::tests` compares node id, exact name, node kind, stable edge id, and every edge kind with canonical graph iteration. `crates/graph/src/edge_identity.rs` is the single edge-id encoder used by Query, Diff, Validation, and the index. | pass |
| Adjacency | Index tests compare incoming, outgoing, and kind-filtered adjacency with independent canonical scans, including unknown nodes and reversed construction. Results retain stable `EdgeId` order. | pass |
| Containment | Index, Query, Resolution, and Validation tests cover owner edges, all owners, children, child kinds, same-named children, missing owners, multiple owners, self-loops, and wrong-owner membership without repairing invalid facts. | pass |
| Resolution | `SemanticResolutionIndex` retains its public constructors, resolver signatures, borrowed returns, error variants, payloads, kind checks, and sorted candidates while owning one shared snapshot index. Fifteen focused tests pass. | pass |
| Query | `SemanticGraphQuery` retains its public type and method signatures, covariance, `Clone`, and `const` constructors. Thirteen focused tests cover every indexed primitive, independent scan equivalence, filters, deduplication, cycles, self-loops, depth, empty snapshots, and repeated construction. | pass |
| Consumers | Focused Impact, Validation, Diff, build Diff, Coverage, and complete EDT suites pass without changes to their canonical responsibilities or accepted results. | pass |
| Determinism | Index and facade tests cover empty, missing, duplicate-name, ambiguous, multiple-owner, reversed-order, self-loop, cyclic, and repeated-construction cases. All ordered results match canonical or pre-migration behavior. | pass |
| Lifecycle | Resolution owns one index borrowing one complete graph snapshot. Query constructs the same borrowed complete-snapshot representation for primitive lookup while preserving its existing `const` and lifetime compatibility. No index state survives graph mutation or crosses snapshots. | pass |
| Scope | The range adds no incremental mutation, invalidation, structural sharing, persistence, Runtime, transport, adapter-specific indexing, IDE integration, search behavior, dependencies, benchmarks, or Coverage transitions. | pass |

Task 1 established identity/classification equivalence before facade migration.
Task 2 established adjacency and containment scan equivalence. Task 3 then
proved Resolution compatibility and removed its duplicate maps before Task 4
removed eligible Query scan paths. This order satisfies the replacement gate.

## Validation

The complete focused matrix executed against committed Task 4 head and passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph --lib semantic_index::tests` | 5 passed; 106 filtered out |
| `cargo test -p oneagent-graph --test query` | 13 passed |
| `cargo test -p oneagent-graph --lib resolution::tests` | 15 passed; 96 filtered out |
| `cargo test -p oneagent-graph --test impact` | 13 passed |
| `cargo test -p oneagent-graph --test validation` | 42 passed |
| `cargo test -p oneagent-graph --test diff` | 2 passed |
| `cargo test -p oneagent-graph --test build_diff` | 1 passed |
| `cargo test -p oneagent-graph --test coverage` | 14 passed |
| `cargo test -p oneagent-edt` | 199 passed across unit and integration targets; EDT doc-tests matched 0 tests |

No focused filter matched zero tests. The 0 EDT doc-tests are recorded
separately and are not counted as acceptance evidence.

The full validation gate also passed:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace                         # 461 tests passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Workspace doc-test targets that matched 0 tests are not counted in the 461-test
total.

## Findings and missing evidence

- Confirmed defects: none.
- Missing acceptance evidence: none.
- Open review questions: none.
- Scope violations: none.

## Lifecycle, risk, and deferred scope

The index is derived from and borrows one immutable complete graph snapshot.
It cannot become a competing semantic authority. A new graph snapshot requires
a newly constructed facade and index.

Query intentionally materializes the shared complete-snapshot representation
for primitive lookup rather than retaining borrowed lazy state: retaining it
would either break the existing lifetime covariance or the accepted `const`
construction surface. No performance target or unsupported optimization claim
is made for Sprint 4.

Incremental maintenance, invalidation, retained state across graph changes,
structural sharing, persistence, cache formats, Runtime services, transports,
source-adapter-specific indexes, IDE integration, fuzzy or ranked search, and
benchmark-backed optimization remain deferred to their accepted later scope.

## Decision

`pass`

Sprint 4 is complete. The graph remains the single semantic authority, all
ADR-0026 lookup dimensions have deterministic equivalence evidence, public
Query and Resolution compatibility is preserved, consumers remain green, and
deferred Sprint 5 or later concerns were not pulled into the implementation.
