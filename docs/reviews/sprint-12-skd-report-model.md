# Sprint 12 SKD and Report Model Integration Review

## Review status

Pass recorded on 2026-08-20 against committed Task 4 head
`ba9f8350bc78784052a56ab95680a019719a1792`.

Sprint 12 satisfies
[ADR-0034](../adr/0034-report-data-composition-semantics.md) and preserves the
existing Report discovery path, generic semantic consumers, ADR-0030
all-or-nothing query-source boundary, and the completed Sprint 11 baseline. No
blocking or non-blocking findings, missing acceptance evidence, open questions,
or scope violations remain.

## Reviewed baseline and range

The parent baseline is
`8b0d22ef955129d4bf6eb88549529a81baf9c466`. The reviewed planning,
implementation, and evidence range is:

```text
8b0d22ef955129d4bf6eb88549529a81baf9c466..ba9f8350bc78784052a56ab95680a019719a1792
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `46120c5b76b83edf25bda629d57eaf9f406e3eff` | `Plan Sprint 12 data composition and reports` |
| Task 1 | `12774229a42c70e909d7140cf210082d7d813df8` | `Implement Sprint 12 data composition graph model` |
| Task 2 | `295c02684331daf2fafb48c43dc6c34ecf17fe84` | `Parse Sprint 12 report data composition schemas` |
| Task 3 | `23dee7edb35649780ac59ff2877b72c5af74cf40` | `Emit Sprint 12 report data composition semantics` |
| Task 4 | `ba9f8350bc78784052a56ab95680a019719a1792` | `Complete Sprint 12 production evidence` |

The audit compared the exact range with ADRs 0006, 0007, 0008, 0021, 0023,
0024, 0030, and 0034, Semantic Model 2.0, the source investigation, the
Roadmap, the committed Sprint 12 prompt suite, production code, the tracked
reduced fixture, Coverage registries, and executed tests. The range contains 46
changed files, 6,949 insertions, and 70 deletions; per-commit path review found
no unrelated change.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | Planning and Tasks 1-4 are separate, linear, dependency-ordered commits. Task 4 leaves no task-created uncommitted change. | pass |
| Source corpus and fixture provenance | The investigation records 56 schemas, 70 direct Data Sets, 970 direct named Fields, and 38 direct Queries. The tracked fixture covers Query, Object, Union, empty main, non-main, nested-deferred, and folder-deferred shapes. Its README records exact live paths and selections, source blob/SHA-256 values, reduction treatment, and reduced hashes. All 15 live-source rows and all 15 reduced-artifact hashes were recomputed without mismatch. | pass |
| Node, payload, and public enum model | The three additive node kinds and three closed payload variants preserve exact main, kind/source, and data-path content. Payload compatibility is exact, repository exhaustive consumers compile, and accepted public migration impact is documented. | pass |
| Identity and collision behavior | Schema identity is the declared UUID. Data Set and Field IDs use length-prefixed owner/name tuples; metadata-owned Query uses a fixed role. Delimiter-collision tests pass, and payload, query text, position, traversal, and provenance remain outside identity. | pass |
| Parser fields and artifact join | The dedicated reader verifies Report UUID/name, DCS template UUID/name/type, optional exact main selector, exact artifact correspondence, root/namespace, local `DataSource1`, direct Data Set kind/source, named direct Fields, and complete Query text. Output is identity-sorted. | pass |
| Parser failures | Missing, extra, and ambiguous artifacts; duplicate UUID/name; malformed main selection; identity/root/namespace/XML/read failures; invalid sources, names, paths, query cardinality, and duplicate direct entities are typed fatal errors. A fatal build exposes no partial result. | pass |
| Main, non-main, empty, Query, Object, and Union | Parser and production tests exercise every accepted first-slice source shape and exact typed content. Empty schemas remain valid and non-main is distinct from missing schema declaration. | pass |
| Nested, folder, and unsupported outcomes | Nested Data Sets, field folders, and unknown Data Set/Field types remain recoverable typed observations. They produce deterministic warnings and one legacy rejected-observation count per occurrence, without nodes, placeholder identities, or relations. | pass |
| Ownership | Only Report-to-Schema, Schema-to-DataSet, DataSet-to-Field, and DataSet-to-Query pairs were added. Every emitted child has one immediate owner; reverse, transitive, unrelated, missing, multiple-owner, and cycle states remain invalid. Existing Procedure/Function Query ownership is preserved. | pass |
| Provenance | Every accepted node and Contains edge carries deterministic non-empty producer evidence with project-relative artifact path, identity, role, and mutable semantic content required by Diff. Deferred diagnostics carry deterministic parsed evidence. | pass |
| Diagnostics and statistics | Fatal structural errors remain source errors. Deferred and unsupported observations use distinct stable diagnostic kinds/codes and deterministic statistics without duplicating terminal observations. | pass |
| Metadata-owned Query | Exactly one generic Query node is owned by each direct Query Data Set. Its fixed-role ID survives text changes; changed complete text is observable as provenance modification. No report-specific query authority was added. | pass |
| Query-source relation absence | DCS Query text never enters the BSL parser or public QuerySource request ledger. Fixture and transition tests prove no DCS `Reads`, `DependsOn`, `References`, candidates, or query-language diagnostics. Existing BSL behavior remains unchanged. | pass |
| Generic Query | Canonical lookup, node-kind filtering, owner/children navigation, adjacency, and repeated construction expose all new facts deterministically through the existing Query API. | pass |
| Diff | Add/remove and main, Data Set kind/source, Field path, Query text, ownership, and deferred-observation transitions use stable identities and exact graph/build-level change scopes. | pass |
| Impact policy | The new ownership facts are Contains-only and create no dependency or usage propagation. Direct Query-text changes affect only the Query node. | pass |
| Reports and Validation | Reports count the stored node/edge/diagnostic/statistics distributions. Validation accepts canonical graphs and rejects incompatible payloads, invalid endpoints, missing or multiple owners, cycles, and inconsistent report state deterministically. | pass |
| Complete Semantic Index | Complete indexes expose typed payloads, all four ownership paths, kind lookups, Query navigation, and dependency exclusion from canonical graph state. Repeated index/report/validation results are equal. | pass |
| Incremental equivalence | Schema/DataSet/Field/Query add/remove, main role, Data Set kind/source, Field path, Query text, ownership, and deferred-observation transitions match complete clean rebuilds. | pass |
| Coverage | Graph Domain is exactly 91 capabilities: 87 `Supported` and 4 `NotApplicable`. EDT is exactly 110 capabilities: 105 `Supported` and 5 `NotApplicable`. Required evidence and representative tests are executable; both registries have zero gaps. | pass |
| Documentation | ADR-0034, source investigation, fixture README, Semantic Model 2.0, Roadmap, implementation, and executable aggregate counts agree on accepted and deferred behavior. | pass |
| Unrelated compatibility | Full workspace tests recheck Metadata, Contains, Calls, References, Reads, Writes, Grants, Includes, Extends, Opens, Triggers, DependsOn, Query, Diff, Impact, reports, validation, request ledger, and indexes. | pass |
| Workspace gate and repository safety | Every focused filter matched tests and every canonical command exited successfully. The previous-suite inventory matches exactly seven tracked Sprint 11 files, no Markdown link depends on them, and both user-owned untracked files remain outside the transition. | pass |
| Deferred scope | Nested Union entities, field folders, broader DCS grammar, virtual tables, batches, temporary tables, lineage, runtime composition, non-Report schemas, persistence, Designer XML, XDTO/services, Runtime, API, CLI, MCP, LSP, and IDE work remain deferred. | pass |

## Validation

The focused review matrix passed against committed Task 4 head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-metadata` | 10 passed; metadata doc-test target contained 0 tests |
| `cargo test -p oneagent-graph` | 285 passed: 154 unit and 131 integration; graph doc-test target contained 0 tests |
| `cargo test -p oneagent-edt --lib report_data_composition::tests` | 8 passed; 233 filtered out |
| `cargo test -p oneagent-edt --test report_data_composition` | 8 passed |
| `cargo test -p oneagent-edt --test coverage` | 4 passed |
| `cargo test -p oneagent-edt --test semantic_index` | 4 passed |

Every focused command matched the intended tests. Zero-test doc-test targets
are not counted as acceptance evidence.

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
zero failures and executed 665 tests. The fixture audit recomputed 15 live
source blob/SHA-256 pairs and 15 tracked reduced-artifact SHA-256 values without
mismatch.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

The three additive `NodeKind` and `GraphNodePayload` variants expand public
exhaustive enums. Repository consumers are migrated and the full workspace gate
proves internal compatibility. External exhaustive consumers must accept the
additive variants; this is the accepted ADR-0034 migration effect, not a review
finding.

Opaque DCS Query text remains intentionally outside query-source parsing and
dependency projection because none of the 46 investigated direct-or-nested
queries satisfies the complete-source grammar. Nested Data Sets and field
folders remain typed deferred observations. These are accepted boundaries, not
missing Sprint 12 work.

## Decision

`pass`

Sprint 12 is complete. EDT Reports now preserve stable Report-template UUID,
owner-scoped direct Data Set and Field identities, fixed-role metadata-owned
Queries, exact typed content, immediate Contains ownership, provenance, and
typed deferred evidence across generic consumers and complete/incremental
indexes. Sprint 13 XDTO and Service Model is eligible as the next planning
target; v0.3 remains planned through Sprint 14.
