# Sprint 10 Subsystems and Composition Integration Review

## Review status

Pass recorded on 2026-08-20 against committed Task 4 head
`f02e5fdbf34d5c68dc017e313ad5787471788046`.

Sprint 10 satisfies
[ADR-0032](../adr/0032-subsystem-hierarchy-semantics.md) and preserves the
ADR-0020 direct Subsystem content contract, ADR-0024 provenance contract,
ADR-0025 References endpoint validation, existing top-level Subsystem
behavior, generic graph consumers, and the completed Sprint 9 baseline. No
blocking or non-blocking findings, missing acceptance evidence, open
questions, or scope violations remain.

## Reviewed baseline and range

The parent baseline is
`4ff5d5cf0788c732fcb194ecac816e2e9e2ed34d`. The reviewed planning,
implementation, and evidence range is:

```text
4ff5d5cf0788c732fcb194ecac816e2e9e2ed34d..f02e5fdbf34d5c68dc017e313ad5787471788046
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `acf238fa2ebabb161a73ea860394b1a30d817e02` | `Plan Sprint 10 subsystems and composition` |
| Task 1 | `793a985ffabeb88f38b077d07a3b7f893308b6c1` | `Implement Sprint 10 subsystem hierarchy graph rules` |
| Task 2 | `27b8c99309a6ad5b4d4f6e41f88db15be1397378` | `Parse Sprint 10 nested subsystem hierarchy` |
| Task 3 | `431e22b241ff0484373ae82734d7cb845c92784b` | `Emit Sprint 10 nested subsystem composition` |
| Task 4 | `f02e5fdbf34d5c68dc017e313ad5787471788046` | `Complete Sprint 10 production evidence` |

The audit compared the exact range with ADRs 0020, 0024, 0025, and 0032,
Semantic Model 2.0, the source investigation, the Roadmap, the committed
Sprint 10 prompt suite, live production code, the provenance-backed reduced
fixture, Coverage registries, and executed tests.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | The planning baseline and Tasks 1–4 are separate, dependency-ordered commits with only their accepted graph, parser, emission, evidence, fixture, ADR, model, Roadmap, and prompt paths. The Task 4 head has no task-created uncommitted changes. | pass |
| Source inventory and fixture provenance | The source investigation covers 127 parseable live descriptors, five hierarchy levels, 114 nested relations, and duplicate local names. The reduced fixture records exact live paths, selected fields, Git object IDs, source SHA-256 values, and fixture SHA-256 values for every derived artifact. | pass |
| Recursive source agreement | The parser requires exact agreement among parent `subsystems`, child `parentSubsystem`, and immediate physical nesting. Focused tests cover depths 1–5, duplicate local names, missing, extra, duplicate, malformed, mismatched, cyclic, escaped, and unreadable structures with deterministic typed failures and no partial graph output. | pass |
| Graph endpoints and cycle validation | Includes validation additively accepts flat Subsystem-to-Subsystem hierarchy and preserves metadata member endpoints. Self-loops and directed hierarchy cycles are rejected deterministically; unrelated endpoint combinations remain invalid. | pass |
| Identity, ownership, and direct hierarchy | Nested metadata and flat Subsystem nodes retain existing UUID-derived identities. Configuration Contains ownership is preserved, direct parent-child Includes edges are canonical, and no transitive closure edge is persisted. Duplicate local names under different parents remain distinct. | pass |
| Nested content composition | Nested descriptors use the existing ADR-0020 direct content pipeline. Supported members resolve to exact metadata endpoints; unresolved and unsupported observations preserve the accepted diagnostic and statistics behavior. `Subsystem.*` content tokens remain deferred rather than acquiring semantic meaning. | pass |
| Provenance, diagnostics, and statistics | Direct hierarchy and content facts carry deterministic aggregated provenance. Reordered and repeated builds are identical, diagnostics remain sorted and deduplicated, and reference statistics preserve per-observation outcomes without duplicate graph facts. | pass |
| Direct and transitive Query | The public Query facade exposes exact direct Includes and cycle-safe transitive metadata membership. Results are unique and stable-ID ordered across shared members, duplicate paths, unknown and wrong-kind inputs, invalid cycles, and repeated construction. | pass |
| Diff, Impact, reports, and validation | Hierarchy and content additions, removals, reparenting, and replacements are visible through canonical Diff and report distributions. Validation sees invalid canonical facts, while dependency and Impact policy remains unchanged because Includes is not promoted to a dependency edge. | pass |
| Complete and incremental index | Complete index lookups and incremental add, remove, reparent, and content-replacement transitions match clean rebuilds for nodes, edges, Query, Resolution, adjacency, provenance, and reports. | pass |
| Coverage | Graph remains exactly 85 capabilities: 82 `Supported`, 3 `NotApplicable`; EDT remains exactly 101: 96 `Supported`, 5 `NotApplicable`. The fixture evidence changes no registry status or speculative aggregate. | pass |
| Top-level and unrelated compatibility | Existing top-level Subsystem IDs, Includes facts, diagnostics, statistics, and repeated builds remain compatible. Full workspace tests recheck Metadata, Calls, Reads, Writes, Includes, Extends, Opens, DependsOn, Grants, References, Query, Resolution, Validation, Diff, Impact, reports, and index behavior. | pass |
| Documentation and deferred scope | ADR-0032, Semantic Model 2.0, source investigation, fixture README, and Roadmap agree on direct canonical hierarchy and computed transitive membership. Command-interface files, inventory hierarchy, aliases, extensions, external Subsystems, new content semantics, hierarchy-aware dependency or Impact, persistence, Runtime, API, CLI, MCP, LSP, IDE, and Sprint 11 behavior remain deferred. | pass |

## Validation

The focused review matrix passed against committed Task 4 head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph` | 264 passed: 145 unit and 119 integration; graph doc-test target contained 0 tests |
| `cargo test -p oneagent-edt --lib subsystem_hierarchy::tests` | 13 passed |
| `cargo test -p oneagent-edt --test subsystem_hierarchy` | 5 passed |
| `cargo test -p oneagent-edt --test includes` | 4 passed |
| `cargo test -p oneagent-edt --test coverage` | 2 passed |
| `cargo test -p oneagent-edt --test semantic_index` | 2 passed |

Every focused command matched the intended tests. Zero-test package and
doc-test targets are not counted as acceptance evidence.

The complete workspace gate passed at the reviewed baseline:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

All commands exited successfully. Workspace unit and integration targets had
zero failures; the Rust workspace executed 600 unit and integration tests.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

The additive flat Subsystem-to-Subsystem Includes endpoint means external
consumers that exhaustively encode valid endpoint matrices must accept this
case. Repository consumers are updated and the full workspace gate proves
internal compatibility. This is the accepted ADR-0032 migration effect, not a
review finding.

Transitive membership remains a computed Query projection over direct
canonical facts. It is not persisted and does not change dependency or Impact
propagation. `Subsystem.*` content tokens, command-interface files,
configuration inventory hierarchy, aliases, extensions, cross-project
hierarchy, external Subsystems, and partial recovery from hierarchy source
errors remain deferred boundaries rather than missing Sprint 10 work.

## Decision

`pass`

Sprint 10 is complete. Nested EDT Subsystems now preserve strict source
agreement, UUID-derived identity, configuration ownership, direct hierarchy,
direct metadata composition, provenance, and deterministic computed
transitive membership without stored closure. Sprint 11 Event Subscriptions is
eligible as the next planning target; v0.3 remains planned through Sprint 14.
