# Sprint 5 Incremental Indexing Integration Review

## Review status

Pass recorded on 2026-08-19 against committed Task 7 head
`c6a30eb22356646f91a92af20992139e0adb1eb7`.

Sprint 5 satisfies ADR-0027 and the retained ADR-0026 lookup contract. No
blocking or non-blocking findings, missing acceptance evidence, open questions,
or scope violations remain.

## Reviewed baseline and range

The pre-Sprint-5 baseline is
`b55d5e280c8791d03cd6d0a9c5fa3c3b3190f6b0`, the parent of Task 1. The reviewed
range is:

```text
b55d5e280c8791d03cd6d0a9c5fa3c3b3190f6b0..c6a30eb22356646f91a92af20992139e0adb1eb7
```

The implementation and evidence commits are dependency ordered:

| Task | Commit | Message |
|---|---|---|
| 1 | `fe71060d0da6e5b2548d460b96796bc52f3d9580` | `Plan Sprint 5 incremental indexing` |
| 2 | `d83d9f1a433cb4dd654508384be1342609af24f2` | `Add deterministic incremental index changes` |
| 3 | `2372fe294891743ddc9a8449c930244c7f5e3e83` | `Update semantic node indexes incrementally` |
| 4 | `2b374a0913927108158b70dc740e2a814bec5fa2` | `Update semantic edge indexes incrementally` |
| 5 | `01825101b8ca4ea46ee786d9001a68310af7f0fa` | `Integrate incremental semantic index lifecycle` |
| 6 | `58396410ee7286709df4bcefc66f96b6a6a5923a` | `Prove incremental index rebuild equivalence` |
| 7 | `c6a30eb22356646f91a92af20992139e0adb1eb7` | `Complete incremental index integration evidence` |

The range changes only the graph index, internal Query and Resolution
integration, ADR-0027, the Sprint 5 plan, and consumer evidence. Before review
records were created, the worktree contained only the pre-existing untracked
`docs/codex/prompts/` directory; it is excluded from the range and this commit.

## Acceptance evidence matrix

| Area | Repository and executed evidence | Result |
|---|---|---|
| Canonical authority | `SemanticGraph` remains the only owner of canonical nodes, edges, payloads, provenance, and invalid states. `SemanticIndexState` contains only rebuildable owned lookup membership paired with one graph. | pass |
| Change contract | `NormalizedSemanticIndexChanges` derives from a verified directional `SemanticGraphDiff`, validates the exact snapshot pair and endpoint invariants, and emits one deterministic remove/replace/add/refresh phase order. | pass |
| Invalidation | Node identity/name/kind, edge identity/kind, outgoing/incoming adjacency, kind filters, containment owners/edges/children/name membership, and incident deletion update from normalized operations. | pass |
| Retention | Application clones accepted owned state and mutates only affected keys. Retained membership is not semantic authority and every result is checked against the current graph. | pass |
| Lifecycle | `AcceptedSemanticIndex` pairs state with an exact graph instance, rejects stale bases and wrong targets, publishes only successful complete state, preserves the prior state on failure, supports deterministic retry, rejects replay, accepts current-to-current no-op, and exposes explicit rebuild fallback. | pass |
| Node dimensions | The oracle covers empty, missing, add, remove, rename, kind change, payload/provenance refresh, duplicate exact names, reordered construction, and multi-step transitions. | pass |
| Edge dimensions | The oracle covers add, remove, provenance refresh, endpoint/kind replacement, all nine kinds, adjacency filters, containment changes, multiple and invalid owners, duplicate child names, self-loops, cycles, and incident deletion. | pass |
| Query | Public constructors and signatures are preserved. All primitives, stable identities, filters, neighbors, dependencies/usages, bounded traversal, directions, depths, reasons, and ordering compare equal with clean current-snapshot Query behavior. | pass |
| Resolution | Successes, candidate ordering, missing, ambiguous, incompatible-kind, invalid-owner, wrong-owner, child-name, child-kind, owner-kind, and owned-child results compare equal with clean current-snapshot Resolution behavior. | pass |
| Equivalence | One independent comparator derives complete previous/current/missing key universes, observes canonical node and edge content including provenance, and compares incremental state and public facades with a clean rebuild after one-step and multi-step sequences. | pass |
| Consumers | The committed consumer inventory traces Validation, Diff, build Diff, Impact, Coverage/report, reference requests, and EDT. Focused regressions prove their canonical ownership, snapshot pairing, ordering, invalid states, and repeated builds. | pass |
| Determinism | Reversed insertion, repeated normalization and transition, mixed batches, failure/retry, replay, empty transitions, cycles, and repeated EDT builds remain deterministic. | pass |
| Scope | No persistence, Runtime, transport, IDE, source-specific index, new semantic fact, public API expansion, dependency, benchmark, or unsupported performance claim entered Sprint 5. | pass |

## Validation

The complete focused review matrix executed against committed Task 7 head and
passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph --lib incremental_index::tests` | 22 passed; 111 filtered out |
| `cargo test -p oneagent-graph --lib semantic_index::tests` | 5 passed; 128 filtered out |
| `cargo test -p oneagent-graph --test query` | 13 passed |
| `cargo test -p oneagent-graph --lib resolution::tests` | 15 passed; 118 filtered out |
| `cargo test -p oneagent-graph --test validation` | 42 passed |
| `cargo test -p oneagent-graph --test diff` | 2 passed |
| `cargo test -p oneagent-graph --test build_diff` | 1 passed |
| `cargo test -p oneagent-graph --test impact` | 13 passed |
| `cargo test -p oneagent-graph --test coverage` | 14 passed |
| `cargo test -p oneagent-graph --test reference_request_build` | 7 passed |
| `cargo test -p oneagent-edt` | 199 passed; EDT doc-tests matched 0 tests |

The planned `--test incremental_index` target does not exist; the live accepted
replacement is the non-zero `--lib incremental_index::tests` filter above. No
focused acceptance filter matched zero tests. The EDT doc-test target with zero
tests is recorded separately and is not counted as evidence.

The full gate also passed:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace                         # 483 tests passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Workspace doc-test targets with zero tests are not included in the 483-test
count.

## Findings, risks, and deferred scope

- Blocking findings: none.
- Non-blocking findings: none.
- Missing evidence: none.
- Open questions: none.
- Compatibility breaks: none.
- Coverage transitions: none.

The retained state is intentionally crate-private and snapshot-paired. Runtime
or workspace orchestration, async publication, persistence and cache formats,
source-specific change generation, filesystem/Git watchers, transports, CLI,
MCP, LSP, IDE integration, benchmarks, and broader semantic facts remain in
their roadmap-owned later sprints.

## Decision

`pass`

Sprint 5 is complete. The canonical graph remains the sole semantic authority,
incremental and clean rebuild results are equivalent for every accepted change
class, public compatibility is preserved, and all downstream consumers remain
green.
