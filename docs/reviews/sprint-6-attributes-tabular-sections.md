# Sprint 6 Attributes and Tabular Sections Integration Review

## Review status

Pass recorded on 2026-08-19 against committed Task 7 head
`4bfbf78865e7adce3f8d8412765d903b37ca2407`.

Sprint 6 satisfies ADR-0028 and preserves the completed Sprint 3 ownership and
metadata-reference contracts. No blocking or non-blocking findings, missing
acceptance evidence, open questions, compatibility breaks, or scope violations
remain.

## Reviewed baseline and range

The Sprint 6 planning baseline is
`922ba70b443c7e77a05bf4d02c287baf6c3b5407`. The reviewed implementation and
evidence range is:

```text
922ba70b443c7e77a05bf4d02c287baf6c3b5407..4bfbf78865e7adce3f8d8412765d903b37ca2407
```

The commits are dependency ordered:

| Task | Commit | Message |
|---|---|---|
| 1 | `c0f74625bae52b009852cbff2b25ecca75c7d8af` | `Investigate Sprint 6 member source contracts` |
| 2 | `6f27f7559483df950bbab3548bf0d00107323016` | `Define Sprint 6 member semantics` |
| 3 | `c90d40d6eef9e1ffd4e94a72820ba9d3fd2ebad7` | `Implement Sprint 6 member graph model` |
| 4 | `21504fa8bc28031dfd9f253db75e7517b6776343` | `Parse Sprint 6 EDT member semantics` |
| 5 | `8e808d527e4a188cf1b80b6d73c6759fc8da8259` | `Emit Sprint 6 member ownership` |
| 6 | already complete | ADR-0028 accepts no new reference observation, endpoint, projection, diagnostic, or statistic for member synonym. Focused reference regressions passed; no empty commit was created. |
| 7 | `4bfbf78865e7adce3f8d8412765d903b37ca2407` | `Complete Sprint 6 member coverage` |

The range changes only the accepted member model, EDT structure parsing and
emission, focused graph and production evidence, Coverage, ADR-0028, source
investigation, and current-state documentation. Before this review record was
created, the worktree contained only the pre-existing untracked
`docs/codex/prompts/` directory. It is excluded from the reviewed range and the
review commit.

## Acceptance evidence matrix

| Area | Commits, files, and executed evidence | Result |
|---|---|---|
| Source evidence | Task 1 records the real grants and ownership `.mdo` artifacts, present and absent member synonym, owner nesting, UUID behavior, type observations, unsupported fields, consumer inventory, and deferred unknowns in `docs/architecture/attribute-tabular-section-source-investigation.md`. | pass |
| Accepted boundary | Task 2 adds Accepted ADR-0028. It limits content to optional member synonym, keeps identity/name/ownership/references separate, requires typed invalid and duplicate parser outcomes, rejects unrelated payload kinds, and explicitly classifies Task 6 as regression-only. | pass |
| Domain and graph model | Task 3 adds `MetadataMemberPayload` and the member-only `GraphNodePayload` variant. Controlled construction accepts only Attribute and TabularSection, rejects every unrelated current kind, preserves compatibility constructors, exposes borrowed Query-visible content, and adds no persistence or wire format. `oneagent_metadata` ran 9 tests and `oneagent_graph` ran 233 tests. | pass |
| Identity and equality | Source UUID remains canonical. UUID-less fallback remains immediate-owner scoped and excludes synonym. Equal names under distinct owners produce distinct nodes. Payload participates in node equality and snapshot content but not lookup identity. Parser, production ownership, Query, and Diff tests passed. | pass |
| Parser contract | Task 4 extends `EdtMetadataChildDescriptor` and `metadata_structure.rs` for exactly one direct non-empty `synonym/value`. Real grants and ownership fixtures prove present and absent content. Generated tests prove non-ASCII decoding, key/value order independence, repeated reads, nested-owner isolation, empty value, missing value, duplicate container/value, unsupported `content`, and malformed XML behavior. | pass |
| Production emission | Task 5 uses explicit member payload construction for present and absent Attribute/TabularSection nodes while retaining the existing two-pass node-then-ownership boundary. Real grants and ownership tests prove Query visibility, declared provenance, immediate ownership, no companion metadata-object owner, source-order independence, repeated builds, and valid graphs. | pass |
| Ownership and validation | Top-level members retain the metadata-object owner; nested Attributes retain only the nearest TabularSection owner. Graph Validation rejects missing, incompatible, and multiple owners. The ownership integration target ran 3 tests and graph validation is included in the 233-test graph suite. | pass |
| Reference compatibility | Synonym creates no reference request. Attribute, Dimension, and Resource remain the only metadata-type sources; the nine ADR-0025 target mappings and current `References` plus ADR-0017 `DependsOn` projections are unchanged. Request lifecycle, endpoint validation, diagnostics, statistics, reports, and failed/partial no-placeholder behavior passed in graph and EDT package tests. | pass |
| Query, Diff, Impact, and index | Query exposes the borrowed payload without synonym lookup. A synonym-only change is one `SemanticContent` node modification with stable node and edge identity. Impact has one direct seed without implicit propagation. Incremental node lookup state remains equivalent to a clean rebuild. Focused graph and production tests passed. | pass |
| Coverage and aggregates | Task 7 adds `SemanticPayloadPreserved` only to graph-domain and EDT `SemanticNode(Attribute)` and `SemanticNode(TabularSection)`. All four capabilities remain `Supported`, have `evidence == required_evidence`, no limitation, no missing evidence, and executable representative tests. Registry totals remain derived from live capabilities; combined Critical, High, and Medium gaps remain zero. | pass |
| Determinism | Repeated parsing and builds, reordered source observations, stable provenance, exact owner navigation, equal-name owners, payload-only Diff, request ordering, and clean-rebuild index equivalence passed. | pass |
| Scope containment | No Form, Command, Sprint 7 source contract, new reference family, new target mapping, placeholder, Unknown node, persistence, Runtime, transport, AI, MCP, IDE, dependency, benchmark, or unsupported performance claim entered the range. | pass |

## Validation

The required focused review matrix executed against committed Task 7 head and
passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-metadata` | 9 passed |
| `cargo test -p oneagent-graph` | 233 passed |
| `cargo test -p oneagent-edt` | 206 passed |
| `cargo test -p oneagent-edt --test ownership` | 3 passed |

No focused acceptance filter matched zero tests. Package doc-test targets with
zero tests are recorded separately and are not counted as acceptance evidence.

The full gate also passed against the same baseline:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace                         # 498 tests passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Workspace unit and integration tests reported 498 passed, 0 failed, and 0
ignored. Workspace doc-test targets with zero tests are not included in the
498-test count.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Compatibility breaks: none.
- Scope violations: none.

The accepted slice intentionally leaves number qualifiers, history and search
settings, produced types, line-number settings and standard attributes,
multiple locale values, alternative member synonym encodings, deeper nesting,
non-Document owner families, and duplicate-identity policy for future
evidence-backed tasks. These are explicit deferred boundaries, not Sprint 6
findings.

## Decision

`pass`

Sprint 6 is complete. Attribute and TabularSection optional display content is
preserved through the source-independent model, strict EDT parsing, canonical
production graph emission, Query, Diff, Impact, incremental indexing, and
Coverage without changing identity, ownership, references, or public
serialization. Sprint 7 Forms and Commands is eligible as the next planning
target; v0.3 remains planned until its Sprint 14 release integration review.
