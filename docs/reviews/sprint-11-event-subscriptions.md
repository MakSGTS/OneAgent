# Sprint 11 Event Subscriptions Integration Review

## Review status

Pass recorded on 2026-08-20 against committed Task 5 head
`ea2294e12505f80dce0d55e43a30fab8f2b78756`.

Sprint 11 satisfies
[ADR-0033](../adr/0033-event-subscription-semantics.md) and preserves the
ADR-0024 single-target request lifecycle, ADR-0025 References endpoint
contract, existing dependency and Impact policy, generic graph consumers, and
the completed Sprint 10 baseline. No blocking or non-blocking findings,
missing acceptance evidence, open questions, or scope violations remain.

## Reviewed baseline and range

The parent baseline is
`62d22c53d0e0c7f077d477398fe899c311dd5cc3`. The reviewed planning,
implementation, and evidence range is:

```text
62d22c53d0e0c7f077d477398fe899c311dd5cc3..ea2294e12505f80dce0d55e43a30fab8f2b78756
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `b90b2ca050b853a54a904dc8f93f5748b4926072` | `Plan Sprint 11 event subscriptions` |
| Task 1 | `fc9bd19364b3169af5f230f28c85fb566ccc2d09` | `Implement Sprint 11 event subscription graph model` |
| Task 2 | `9f1b885f37908240b34050fa69d476fb4d04193e` | `Parse Sprint 11 event subscription descriptors` |
| Task 3 | `630a009de930ae3f192f628517650cc314e87255` | `Resolve Sprint 11 event subscription targets` |
| Task 4 | `8c681b0a4e5073bf1924929b85adca33fa8a4101` | `Emit Sprint 11 event subscription semantics` |
| Task 5 | `ea2294e12505f80dce0d55e43a30fab8f2b78756` | `Complete Sprint 11 production evidence` |

The audit compared the exact range with ADRs 0007, 0008, 0012, 0016, 0023,
0024, 0025, and 0033, Semantic Model 2.0, the source investigation, the
Roadmap, the committed Sprint 11 prompt suite, production code, the tracked
reduced fixture, Coverage registries, and executed tests. The range contains
46 changed files, 7,213 insertions, and 100 deletions; per-commit path review
found no unrelated change.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | Planning and Tasks 1-5 are separate, linear, dependency-ordered commits with accepted owned paths. Task 5 leaves no task-created uncommitted change. | pass |
| Source corpus and fixture provenance | The live corpus contains 99 descriptors and 314 source-selector occurrences. The fixture README records exact live paths, line selections, Git object IDs, source SHA-256 values, reduction treatment, and fixture SHA-256 values. All ten fixture artifact hashes were recomputed and matched. | pass |
| Metadata, payload, and public enums | `MetadataKind::EventSubscription`, stable code `event_subscription`, the closed event payload, and `EdgeKind::Triggers` are present. Payload compatibility is exact, event changes preserve UUID identity and appear as semantic-content modifications, and exhaustive repository consumers compile. | pass |
| Parser fields and fatal errors | The dedicated reader preserves UUID, name, optional synonym, event, exact handler spelling, supported, unsupported, malformed, bare, qualified, duplicate, and reordered source observations. Root, namespace, UUID, required-field, source-container, handler-shape, filesystem, and XML failures are typed and fatal before partial production output. | pass |
| Source resolution | Qualified selectors use exact name and mapped kind. Bare selectors return the complete stable-ID-ordered family and an empty family is missing rather than successful. Manager/Object overlaps aggregate graph facts while retaining observation evidence; malformed and unsupported families stay recoverable typed rejections. | pass |
| Handler ownership and export policy | Resolution proves Common Module metadata ownership of one Module and that Module's ownership of one named Procedure. Missing, ambiguous, wrong-kind, Function, and wrong-owner paths are distinct outcomes. Export status is not consulted, and the fixture proves an owned non-exported Procedure remains valid. | pass |
| Node and edge identity | Event Subscription node identity is the stable EDT UUID. Source order, event, handler, payload, and provenance do not enter identity. References and Triggers use canonical source-target-kind edge identity and deterministic ordering. | pass |
| Ownership | Every accepted Event Subscription is directly owned by Configuration through one provenance-backed Contains edge. Handler module ownership remains ordinary metadata-to-Module-to-Procedure containment. | pass |
| References | Accepted source metadata and handler Procedure targets emit only direct EventSubscription-to-target References. The positive endpoint matrix covers all eight accepted metadata kinds and Procedure; reversed, unsupported, unknown, Function, Module, and unrelated endpoints are rejected. | pass |
| Triggers | One resolved handler emits one direct EventSubscription-to-Procedure Triggers fact. Reversed and unrelated endpoints are invalid, and no reverse or derived closure fact is stored. | pass |
| Provenance | Nodes, ownership, source References, handler References, Triggers, and diagnostics carry deterministic non-empty producer evidence. Equivalent observations aggregate sorted deduplicated provenance without changing fact identity. | pass |
| Diagnostics and statistics | Missing, ambiguous, incompatible, invalid-owner, malformed-format, and unsupported-prefix outcomes are typed and deterministic. Each selector or handler is counted once; projecting both handler relations does not double-count resolution statistics. No failed observation creates a placeholder relation. | pass |
| Public request ledger | Event Subscription observations remain adapter-private legacy observations. Family selectors do not enter or distort the ADR-0024 single-target ledger, and existing request counts and behavior remain unchanged. | pass |
| Generic Query | Generic lookup, ownership, adjacency, edge-kind filtering, traversal, dependency, and usage APIs expose canonical Event Subscription facts in deterministic order without an event-specific API. | pass |
| Diff | Add/remove subscription, event payload modification, source add/remove/retarget, handler retarget, and References/Triggers transitions are represented through existing stable identities and build diff summaries. | pass |
| Dependency and Impact policy | Handler References remains dependency-like. Triggers is queryable and diffable but is excluded from independent dependency and Impact propagation, preventing duplicate propagation for one declaration. | pass |
| Reports and validation | Reports count Event Subscription nodes, References, Triggers, diagnostics, provenance, and statistics. Validation accepts every canonical fact and deterministically rejects missing provenance or invalid endpoints. | pass |
| Complete index | Complete Semantic Index lookups expose metadata kind, payload, ownership, References, Triggers, provenance, and generic dependency behavior from the canonical graph. | pass |
| Incremental equivalence | Subscription add/remove, event change, source add/remove/retarget, and handler retarget transitions match complete clean rebuilds for nodes, edges, adjacency, payload, provenance, Query, Resolution, and reports. | pass |
| Coverage | Graph Domain is exactly 88 capabilities: 84 `Supported` and 4 `NotApplicable`. EDT is exactly 104 capabilities: 99 `Supported` and 5 `NotApplicable`. Both have zero Critical, High, or Medium gaps. | pass |
| Documentation | ADR-0033, the source investigation, fixture README, Semantic Model 2.0, Roadmap, and implementation agree on current behavior and the export-audit correction. | pass |
| Unrelated compatibility | Projects without Event Subscriptions retain prior behavior. Full workspace tests recheck Metadata, Calls, Reads, Writes, Includes, Extends, Opens, DependsOn, Grants, References, Query, Resolution, Validation, Diff, Impact, reports, and indexes. | pass |
| Workspace gate and repository safety | Every focused filter matched tests, the complete gate passed, the previous-suite inventory matched exactly six tracked files, no current Markdown link depends on them, and both user-owned untracked files remain outside the transition. | pass |
| Deferred scope | Unsupported metadata families, public multi-target requests, partial-workspace family resolution, handler signatures, runtime dispatch, Triggers dependency policy, event-specific APIs, persistence, Designer XML, SKD, XDTO, services, Runtime, API, CLI, MCP, LSP, and IDE work remain deferred. | pass |

## Validation

The focused review matrix passed against committed Task 5 head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-metadata` | 10 passed; metadata doc-test target contained 0 tests |
| `cargo test -p oneagent-graph` | 274 passed: 148 unit and 126 integration; graph doc-test target contained 0 tests |
| `cargo test -p oneagent-edt --lib event_subscription::tests` | 7 passed; 226 filtered out |
| `cargo test -p oneagent-edt --lib event_subscription_resolution::tests` | 9 passed; 224 filtered out |
| `cargo test -p oneagent-edt --test event_subscriptions` | 7 passed |
| `cargo test -p oneagent-edt --test coverage` | 3 passed |
| `cargo test -p oneagent-edt --test semantic_index` | 3 passed |

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
zero failures and executed 636 tests. Fixture integrity checks recomputed all
ten tracked artifact SHA-256 values without mismatch.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

The public exhaustive enums add `MetadataKind::EventSubscription`,
`MetadataSpecificPayload::EventSubscription`, and `EdgeKind::Triggers`.
Repository consumers are migrated and the full workspace gate proves internal
compatibility. External exhaustive consumers must accept the additive variants;
this is the accepted ADR-0033 migration effect, not a review finding.

All 93 unique live handlers are exported. Four multiline declarations were
initially misclassified by a line-oriented audit, and the final investigation
and fixture disclose the correction. The recomposed non-exported fixture case
tests the accepted ownership rule without claiming a nonexistent live handler
binding.

Unsupported source families remain observable diagnostics without speculative
metadata or placeholder nodes. Multi-target family selectors remain outside
the public request ledger, and Triggers remains outside dependency and Impact
propagation. These are accepted deferred boundaries rather than missing Sprint
11 work.

## Decision

`pass`

Sprint 11 is complete. EDT Event Subscriptions now preserve stable UUID
identity, typed event content, configuration ownership, deterministic exact and
family source resolution, exact owned Procedure handlers, direct References,
direct Triggers, provenance, and recoverable typed failure evidence across
generic consumers and complete/incremental indexes. Sprint 12 SKD and Report
Model is eligible as the next planning target; v0.3 remains planned through
Sprint 14.
