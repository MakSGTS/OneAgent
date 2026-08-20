# Sprint 9 Roles and Access Rights Integration Review

## Review status

Pass recorded on 2026-08-20 against committed Task 3 head
`0a7b4d7e4d080be92f7a64ddcc9a8eb336a46165`.

Sprint 9 satisfies [ADR-0031](../adr/0031-conditional-grants-semantics.md)
and preserves ADR-0019 direct Grants semantics, ADR-0025 References endpoint
validation, unconditional AccessRight compatibility, generic graph consumers,
and the completed Sprint 8 baseline. No blocking or non-blocking findings,
missing acceptance evidence, open questions, or scope violations remain.

## Reviewed baseline and range

The parent baseline is
`5b3c3d3e093a03048b3e31b19aa7fc54a7a792a7`. The reviewed planning,
implementation, and evidence range is:

```text
5b3c3d3e093a03048b3e31b19aa7fc54a7a792a7..0a7b4d7e4d080be92f7a64ddcc9a8eb336a46165
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `a03ecbb4eb6ee0fd250bdf48cd0aa9ab46a3b9f5` | `Plan Sprint 9 roles and access rights` |
| Task 1 | `a1fa40a0a3312a1443d026f55e2c751ab2a4b9aa` | `Implement Sprint 9 conditional access rights` |
| Task 2 | `049465ff8027c16a81e648d390d6f01f03b6bcc3` | `Emit Sprint 9 conditional role grants` |
| Task 3 | `0a7b4d7e4d080be92f7a64ddcc9a8eb336a46165` | `Complete Sprint 9 production evidence` |

The audit compared the exact range with ADRs 0019, 0024, 0025, and 0031,
Semantic Model 2.0, the Roadmap, the committed Sprint 9 prompt suite, live
production code, real EDT fixtures, Coverage registries, and executed tests.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | The planning baseline and Tasks 1–3 are separate, dependency-ordered commits with only the accepted model, emission, evidence, ADR, Roadmap, and prompt paths. The Task 3 head has no task-created uncommitted changes. | pass |
| Typed graph payload | `AccessRightPayload`, `AccessRightRowRestriction`, `GraphNodePayload::AccessRight`, `GraphNode::access_right_payload`, and `SemanticGraph::insert_access_right` preserve typed optional content. Matching and wrong-kind construction tests cover every unrelated node kind. Legacy payload-free AccessRight nodes remain valid. | pass |
| Identity and unconditional compatibility | `AccessRight::new` delegates to the optional constructor with no restriction and retains the exact pre-Sprint-9 ID and display name. Length-delimited conditional identity, equal-input deduplication, absent/present separation, distinct conditions, deterministic ordering, outer Unicode whitespace trimming, internal-content preservation, and empty rejection are executable. | pass |
| Real conditional source | The repository-owned BaseUser and full Grants fixtures preserve `WHERE NOT DeletionMark` for Product `Read` and `Update`. The full filesystem builder exposes both as typed conditional AccessRight nodes with the expected Role-to-AccessRight Grants and AccessRight-to-resource References paths. | pass |
| EDT production mapping | The private resolved observation and all AccessRight, References, and Grants aggregation keys carry the optional typed condition. Real, generated mixed, duplicate, reordered, repeated, absent, and distinct-condition cases are deterministic and validate successfully. Existing 39 AccessRight, 50 Grants, and 39 AccessRight References fixture counts remain exact. | pass |
| Negative outcomes | Parser and builder tests cover false values, missing and whitespace-only conditions, malformed and missing rights artifacts, unsupported resources, missing targets, ambiguous targets, incompatible target kinds, and exact right tokens. Rejected or unresolved observations create no Grants, AccessRight, or placeholder target. | pass |
| Provenance, diagnostics, and statistics | One shared deterministic provenance encoder records restriction absence or the length-delimited canonical condition before the fact-kind suffix for AccessRight, Grants, and References facts. Existing aggregation, sorted deduplication, diagnostics, and reference statistics remain stable; duplicate observations retain per-observation resolution counts without duplicate graph facts. | pass |
| Query and complete index | Generic Query and Resolution facades find conditional nodes by stable ID and kind, expose typed payload, and navigate exact incoming Grants and outgoing References edges. Deterministic name, kind, adjacency, and full-builder repeated-snapshot behavior remain unchanged. | pass |
| Diff, Impact, reports, and validation | Graph tests prove payload-aware node Diff and direct conditional replacement Impact. Production evidence proves conditional node/edge removal Diff, reverse Impact visibility, report distributions, build Diff stability, and valid endpoint matrices. | pass |
| Incremental equivalence | `conditional_access_right_index_transitions_match_clean_rebuilds` covers conditional add, provenance refresh, removal, Grants/References adjacency changes, accepted lifecycle transitions, Query and Resolution equivalence, and equality with clean rebuilt state. | pass |
| Coverage | Dedicated graph and EDT Coverage tests retain every status and capability. Graph remains exactly 85 capabilities: 82 `Supported`, 3 `NotApplicable`; EDT remains exactly 101: 96 `Supported`, 5 `NotApplicable`. Observed real-fixture coverage records 39 provenance-backed AccessRight nodes and 50 provenance-backed Grants edges. | pass |
| Unrelated compatibility | The full workspace suites recheck Metadata, Calls, Reads, Writes, Includes, Extends, Opens, DependsOn, Grants, References, typed payload, Query, Diff, Impact, report, validation, and index behavior. No production dependency, transport, persistence, or unrelated semantic capability changed. | pass |
| Documentation and deferred scope | ADR-0031, Semantic Model 2.0, and the Roadmap describe opaque direct conditional allows only. Condition parsing/evaluation, deny, inheritance, defaults, profiles, groups, users, effective authorization, unsupported resource families, persistence, and Runtime/API surfaces remain deferred. | pass |

## Validation

The focused review matrix passed against committed Task 3 head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph` | 260 passed: 143 unit and 117 integration; doc-test target contained 0 tests |
| `cargo test -p oneagent-edt role_rights` | 7 relevant matches passed: 6 library and 1 Grants integration test |
| `cargo test -p oneagent-edt --test grants` | 7 passed |
| `cargo test -p oneagent-edt --test coverage` | 1 passed |
| `cargo test -p oneagent-edt --test semantic_index` | 1 passed |

The package-wide `role_rights` filter also invoked unrelated EDT integration
binaries with zero matching tests. Those zero-match runs are not counted as
evidence. The supplemental exact command
`cargo test -p oneagent-edt --lib role_rights` matched and passed all 6 relevant
library tests.

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
zero failures. Zero-test package and doc-test targets are not acceptance
evidence.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

`GraphNodePayload::AccessRight` is an additive public enum variant. External
consumers with exhaustive matches must add the new case; repository consumers
are updated and the full workspace gate proves internal compatibility. This is
the accepted ADR-0031 migration effect, not a review finding or a change to
unconditional AccessRight construction.

Opaque conditions remain textually distinct and are never parsed, normalized
beyond outer whitespace, or evaluated. Deny, inheritance, defaults, profiles,
groups, users, effective authorization, and unsupported protected-resource
families remain deferred boundaries rather than missing Sprint 9 work.

## Decision

`pass`

Sprint 9 is complete. Direct EDT role grants now preserve optional opaque row
restrictions as typed, deterministic AccessRight semantics while keeping
unconditional IDs and names byte-compatible and retaining the existing Grants
and References endpoint model. Sprint 10 Subsystems and Composition is eligible
as the next planning target; v0.3 remains planned through Sprint 14.
