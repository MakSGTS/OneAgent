# Incremental Semantic Index Consumer Integration Evidence

## Scope and baseline

This evidence was collected on 2026-08-19 against Task 6 commit
`58396410ee7286709df4bcefc66f96b6a6a5923a`. It covers every live consumer of
the shared Semantic Index boundary and the graph snapshot comparison APIs.

The canonical `SemanticGraph` remains the only semantic authority. Incremental
state is crate-private, derived from one explicitly paired graph snapshot, and
is exposed internally only through the existing Query and Resolution facades.
No consumer owns or reproduces index invalidation policy.

## Live consumer inventory

| Consumer | Live ownership and snapshot contract | Executed evidence |
|---|---|---|
| Shared index, Query, and Resolution | `crates/graph/src/semantic_index.rs` owns the only retained lookup state. `crates/graph/src/query.rs` and `crates/graph/src/resolution.rs` either rebuild from one supplied graph or receive an accepted state paired with that graph. | 22 incremental tests, 13 Query tests, and 15 Resolution tests passed. The full-rebuild oracle compares all public lookup, traversal, ordering, candidate, and typed-error behavior over previous/current/missing key universes. |
| Validation | `crates/graph/src/validation.rs` reads canonical graph facts directly for edge, ownership, provenance, report, diagnostic, request, and statistics invariants. It does not consume a normalized index view that could hide invalid facts. | 42 tests passed, including empty graphs, missing and multiple owners, invalid endpoints, cycles, self-loops, provenance, deterministic ordering, and build-report mismatches. |
| Graph Diff | `crates/graph/src/diff.rs` compares complete canonical previous/current snapshots directionally and uses the shared stable edge identity. It does not consume retained index state. | 2 public integration tests passed; the incremental normalization suite also proves snapshot-pair verification, deterministic ordering, modifications, removals, additions, and incident deletion. |
| Build Diff and reference requests | `crates/graph/src/build_diff.rs` composes canonical `SemanticGraphDiff` with diagnostics, reference-request, resolution-statistics, report, and provenance comparisons. The reference-request ledger remains its own accepted domain model, not a semantic node/edge index. | 1 public build Diff test and 7 reference-request build tests passed, covering direction, identity, mutable aspects, ordering, missing projections, and invalid lifecycle/report states. |
| Impact | `crates/graph/src/impact.rs` constructs separate Query views from the supplied previous and current graphs and pairs seeds and propagation with the correct snapshot. | 13 tests passed, covering empty and inconsistent diffs, additions, removals, modifications, previous/current edge pairing, filters, reasons, direction, depth, ownership options, cycles, self-loops, and reordered repeated results. |
| Coverage and reports | `crates/graph/src/coverage.rs` and graph reports count canonical node, edge, provenance, diagnostic, and request facts. Index maintenance does not change capability declarations or observed occurrence. | 14 Coverage tests passed. No capability or status transition was made in Sprint 5. |
| EDT producer and build result | `adapters/edt` produces canonical graphs and uses graph-owned Query, Resolution, Validation, Diff, build Diff, Impact, Coverage, reports, diagnostics, statistics, and request ledgers. It does not emit index events or own invalidation. | 199 EDT tests passed: 170 unit tests plus 6 Grants, 4 Includes, 2 ownership, 3 payload, 8 Reads, and 6 Writes integration tests. Clean, repeated, reordered, changed, deleted, missing, malformed, ambiguous, incompatible, and deterministic build behavior remains covered. |

Repository-wide usage inspection found no additional production constructor or
owner of `SemanticIndexState`, `AcceptedSemanticIndex`, or
`NormalizedSemanticIndexChanges`. Their direct usages remain inside the graph
crate. All adapter lookups enter through public graph-owned facades, and all
snapshot comparisons enter through Diff or build Diff.

## Incremental sequence compatibility

The Task 6 oracle independently constructs every current canonical graph,
transitions the accepted state from the prior graph, and compares it with a
clean rebuild. It covers empty and no-op transitions, node and edge changes,
duplicate names, provenance refresh, endpoint and kind replacement, adjacency,
containment, invalid and multiple owners, duplicate child names, self-loops,
cycles, incident deletion, mixed batches, reversed construction, repeated and
multi-step execution, stale input, failure followed by retry, replay, and the
explicit clean-rebuild fallback.

This equivalence closes the integration boundary for Query and Resolution.
Consumers that read canonical facts remain independent of index history;
Impact's previous/current Query pairing remains explicit; EDT remains a graph
producer rather than an index lifecycle owner.

## Focused validation

The complete Task 7 focused matrix passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph --lib incremental_index::tests` | 22 passed; 111 filtered out |
| `cargo test -p oneagent-graph --test query` | 13 passed |
| `cargo test -p oneagent-graph --lib resolution::tests` | 15 passed; 118 filtered out |
| `cargo test -p oneagent-graph --test validation` | 42 passed |
| `cargo test -p oneagent-graph --test diff` | 2 passed |
| `cargo test -p oneagent-graph --test build_diff` | 1 passed |
| `cargo test -p oneagent-graph --test impact` | 13 passed |
| `cargo test -p oneagent-graph --test coverage` | 14 passed |
| `cargo test -p oneagent-graph --test reference_request_build` | 7 passed |
| `cargo test -p oneagent-edt` | 199 passed; EDT doc-tests matched 0 tests |

No focused acceptance filter matched zero tests. The EDT doc-test target with
zero tests is recorded separately and is not counted as evidence.

The full workspace gate also passed:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo check --workspace` | passed |
| `cargo test --workspace` | 483 tests passed; workspace doc-test targets with 0 tests are excluded from this count |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | passed |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | passed |
| `git diff --check` | passed |

## Compatibility result

- Public APIs and semantic contracts changed: none.
- Production or test corrections required: none.
- Coverage capability or status changes: none.
- Duplicate semantic authorities or consumer-owned indexes introduced: none.
- Source-specific invalidation, persistence, Runtime, transport, IDE, cache,
  benchmark, or unsupported performance work introduced: none.

The consumer integration result is `pass`. The Sprint 5 integration review
reruns this matrix against the committed Task 7 head.
