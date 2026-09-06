# Sprint 40.1 Refactoring Planner Remediation Review

## Decision

`pass`

The effective decision matches the independent reviewer recommendation. The
Sprint 40.1 correction restores the canonical ADR-0063 publication owner,
preserves the old Change Impact name as the exact same type, migrates Runtime
vocabulary without changing behavior, and synchronizes the evidence with the
immutable reviewed head. No blocking or non-blocking finding remains.

Sprint 40 remains administratively completed. Sprint 40.1 is completed by this
review, and Sprint 41 Safe Edit Transactions becomes the unique `next` target.
This decision does not authorize source mutation or implement transactions,
atomic edits, rollback, recovery, or post-edit semantic validation.

## Reviewed baselines

The immutable combined product range is:

`8d28ba8acacd00efd902eb2aa4ab3194f1636c05..dd55365a62a12f8733ca91185237eaff2f09aa95`

It contains 42 commits, 123 paths, 18,515 additions, and 1,976 deletions. Git
numstat classifies two fixtures as binary-treated entries even though
`git check-attr text diff` reports both attributes as unset. They are intentional
text inputs: the Designer fixture uses UTF-8 BOM plus CRLF, and the EDT fixture
uses ASCII plus LF.

The immutable corrective range is:

`427a78cd809a16bae2ee160867b20bb64c1d415e..dd55365a62a12f8733ca91185237eaff2f09aa95`

It contains nine commits, 18 paths, 1,128 additions, 106 deletions, and no
binary path. The Task 2 implementation range
`7a3bba517d4a6b0ecc0bac0a0f17e4335fcc43fd..afc8bb0794c66af5e82383cca5f4cf262a183025`
contains nine paths, 127 additions, 64 deletions, 191 lines of textual churn,
and no binary path. It satisfies the committed stop-loss of ten paths, 600
lines of churn, and no binary path.

Implementation merge `dd55365a62a12f8733ca91185237eaff2f09aa95`
has parents `41e3e9eae31c181f61443b7be73be819822dbe86` and
`afc8bb0794c66af5e82383cca5f4cf262a183025` and subject
`Merge Sprint 40.1 implementation`. Initial and final review status was clean
on `codex/v0.7-sprint-40.1-review`, whose local and remote heads matched that
commit.

## Independent review and primary reconciliation

The guaranteed fresh-context reviewer was
`/root/s40_1_integration_review`. It received only the repository root, exact
immutable ranges, authorities, criteria, exclusions, validation matrix, and
output contract. It did not receive implementation transcripts or an expected
decision, did not delegate, and did not mutate source, review state, or Git.
Ordinary ignored compiler artifacts were confined to `target/**` and
`extensions/vscode/dist*/**`. Its context preflight was `pass`; the effective
context window and token telemetry were unavailable.

The primary independently inspected both ranges, reran the complete focused,
enumeration, canonical, client, API, dependency, cache, protocol, governance,
sensitive-data, scope, artifact, prompt, link, and cleanliness matrix, and
reached the same `pass` decision. There is no unresolved evidence disagreement,
and the effective decision is not less severe than the reviewer decision.

## Findings and missing evidence

Blocking, high, medium, and low findings: none.

Missing required evidence: none.

The reviewer initially treated the two ordinary TypeScript emit compilations
as incompatible with its read-only contract. Reconciliation with the same
contract established that ignored validation outputs are permitted in the same
way as Cargo and Rustdoc outputs. Both emit compilations then passed. An
unsuccessful attempt to use a nonexistent TypeScript 7 in-memory compiler API
is not acceptance evidence and did not change the repository.

Two other corrected reviewer command attempts are not acceptance evidence. An
initial explicit prompt-validator invocation mistakenly included the
orchestration `00-*` loop and exited 1 with 29 expected contract errors; the
corrected `01` through `03` invocation passed all three child prompts. An
initial mutation audit used an unsupported regular-expression look-around; the
corrected PCRE2 audit passed. No required check remains failed or unexecuted.

## Sprint 40.1 invariant matrix

| Invariant | Independent and primary evidence | Result |
| --- | --- | --- |
| One canonical checked non-zero Workspace publication identity | `crates/analysis/src/publication.rs` owns the only newtype; zero rejection, initial value, checked successor, overflow, and retention tests pass | pass |
| Change Impact remains source-compatible without a second sequence | `change_impact::ChangeImpactPublicationId` is an exact public re-export of the canonical type; the public compile/runtime identity oracle passes and no second struct or conversion exists | pass |
| Refactoring and Change Impact observe the same publication | Refactoring keeps its compatibility re-export; Runtime uses the canonical name and retains one checked sequence atomically with snapshot and impact | pass |
| Public behavior remains read-only and wire-compatible | Protocol, Tool Policy, MCP semantic/stdio/process, and VS Code matrices pass with `readOnly=true` and `editAuthorization="none"` | pass |
| Final evidence matches the immutable reviewed head | Both parties enumerate 85 targets, 81 non-zero targets, four expected zero-test binaries, and 1,365 tests | pass |
| Governance changes remain outside product scope | The six named governance commits have a net effect of 453 additions and 15 deletions across eight governance paths and no Rust, Cargo, product, or API effect | pass |

## ADR-0063 acceptance matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Graph remains the semantic authority | 298 Graph tests and zero corrective Graph diff | pass |
| Analysis owns source and planner contracts | 12 source-evidence tests, 17 planner tests, 159 Analysis tests, and strict Rustdoc | pass |
| Adapters capture, Runtime publishes, MCP projects | EDT 351, Designer 46, Runtime 124, and MCP 10/8/19 | pass |
| Only `bsl_callable_rename_v1` is supported | Closed family and target tests | pass |
| EDT and Designer supported matrices are exact | Source-evidence 6/6 and paired conformance 4 | pass |
| Unsupported or ambiguous evidence fails closed while unrelated evidence does not block | Adapter ledger and planner relevance tests | pass |
| Document identity and retained evidence are complete | Source-evidence identity, path, content, ordering, and completeness tests | pass |
| Content version is raw length plus canonical SHA-256 | Common 6 and content-version tests | pass |
| UTF-8, BOM, line endings, scalar boundaries, and ranges are exact | BSL 54 and paired fixtures | pass |
| Capture occurs once and planning never reopens source | Adapter capture and retained Workspace tests | pass |
| Occurrence ledger and lexical ownership are complete | Adapter ledger and Analysis byte-validation tests | pass |
| One checked publication sequence is shared | One canonical newtype, exact aliases, and Runtime lifecycle tests | pass |
| Target identity is complete | Planner owner, source, kind, and identity tests | pass |
| Desired-name grammar, equality, bounds, and collisions are closed | Planner name and collision tests | pass |
| Request and precondition fields are closed | Domain and MCP schema audits | pass |
| Plan and operation identities are canonical | Reorder, repetition, and conflict tests | pass |
| Operation vocabulary is closed and has zero dependencies | Domain dependency tests | pass |
| Operation fields and total order are exact | Operation, preview, and paired-plan tests | pass |
| Duplicates collapse and conflicts reject atomically | Domain and source conflict tests | pass |
| Successful plans and summaries are complete | Three-operation and public summary tests | pass |
| Preview is deterministic, structured, and redacted | Position, line-ending, repetition, and MCP tests | pass |
| Inclusive bounds and one-over failures are exact | Source, planner, MCP, and Tool Policy tests | pass |
| Failures are closed, ordered, and redacted | Domain, Workspace, and MCP error tests | pass |
| Cancellation is checked and shutdown joins work | Planner, Runtime, stdio, and process lifecycle tests | pass |
| Workspace publication is atomic and complete | Runtime 124, Workspace 9, and adapter failure tests | pass |
| Failed, cancelled, or stale builds do not publish or consume an ID | Runtime, watching, Git-input, cache, and live MCP tests | pass |
| Cache schema and semantic compatibility remain exact | Schema `1`, semantic `8`, version-7 invalidation, and persistent-cache 4 | pass |
| Cold and warm plans are equal and IDs/plans are not persisted | Cache round-trip and replacement tests | pass |
| Eight ordered read-only tools serve three revisions | Protocol 53, MCP matrices, and VS Code 62 | pass |
| Refactor request schema and limit are exact | MCP schema and argument tests | pass |
| Public output is complete, bounded, and redacted | MCP semantic/process and output-size tests | pass |
| Tool Policy remains read-only | Tool Policy 33 and denial/output tests | pass |
| Legacy protocols, framing, processes, and clients remain compatible | Protocol, MCP, Graph Query, HTTP, LSP, CLI, and VS Code tests | pass |
| No dependency, Graph, Coverage, unsafe, API, or deferred-scope expansion entered | Diff, Cargo, API, unsafe, artifact, and scope audits | pass |

## Executable evidence

Both the reviewer and primary ran the complete focused Rust matrix. Every
required target was non-zero and reported zero failed, ignored, measured, or
filtered tests:

| Area | Passed |
| --- | ---: |
| Common / BSL / Graph | 6 / 54 / 298 |
| Analysis source / planner / package | 12 / 17 / 159 |
| EDT source / package | 6 / 351 |
| Designer source / conformance / package | 6 / 4 / 46 |
| Runtime lib / Workspace / watching / Git-input / cache | 124 / 9 / 2 / 3 / 4 |
| Protocol / Tool Policy | 53 / 33 |
| MCP semantic / stdio / process | 10 / 8 / 19 |
| Graph Query / HTTP / LSP stdio / LSP process / CLI | 3 / 4 / 5 / 8 / 2 |

Each side independently enumerated 85 targets, 81 non-zero targets, four
expected zero-test binaries, and 1,365 tests. The zero-test binaries are
`oneagent-cli`, `oneagent-runtime`, `oneagent-mcp`, and `oneagent-lsp`; they are
inventory rather than acceptance evidence. No required test filter matched
zero tests.

The reviewer and primary each ran the canonical gate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Every command exited zero. Both full test runs reported 1,365 passed and zero
failed, ignored, measured, or filtered tests. The primary's wrapper log did not
print its temporary `RUSTDOCFLAGS` assignment, so the primary repeated the
strict Rustdoc command explicitly with the assignment recorded; that run also
exited zero.

Both sides also ran production and test TypeScript no-emit checks, production
and test emit compilations, and fresh VS Code unit tests. All compiler commands
exited zero; each unit run reported 62 passed and zero failed, cancelled,
skipped, or todo tests. No Electron Extension Host or EDT/Eclipse GUI launch was
required because no client or EDT production path changed in the correction.

## API, dependency, cache, protocol, and scope audits

- `oneagent_analysis::publication::WorkspacePublicationId` owns the only
  publication newtype and checked successor. Change Impact and Refactoring
  compatibility paths resolve to that exact type.
- The corrective range changes no Protocol, Tool Policy, Graph, adapter,
  client, manifest, or lockfile path. Runtime wire values and lifecycle remain
  unchanged.
- Combined Cargo changes contain only three internal workspace dependency
  edges. No third-party package, feature, license, workspace member, native
  library, or public API removal was found.
- Cache schema remains `1`, semantic compatibility remains `8`, version `7`
  invalidates, source bytes stay private, and publication IDs and plans are not
  persisted.
- Production results remain bounded and redacted with `readOnly=true` and
  `editAuthorization="none"`. Sensitive-data scans found no credential,
  private-key, bearer, API-key, or personal absolute path.
- No production source-write, repository-write, editor action, apply,
  transaction, rollback, recovery, unsafe, generated tracked artifact, or
  Sprint 41 behavior entered either reviewed range.
- Prompt validation passed all three explicit Sprint 40.1 children and all 14
  discovered prompts before retirement. Combined and corrective Markdown link
  checks inspected 435 and 432 local links respectively with zero broken links.

## Prompt retirement and hand-off

The verified immediately preceding suite contained exactly the eight tracked
files listed by the Sprint 40 plan under
`docs/codex/prompts/sprint-39-change-impact-analysis/`. Artifact consistency
was checked by the same reviewer before their deletion. The complete Sprint 40
and Sprint 40.1 suites are retained.

Sprint 41 owns the next bounded decision and implementation for checked,
reversible edit transactions. It must recheck publication identity, document
version, range, expected token, path confinement, and authorization immediately
before any mutation. A Sprint 40 plan remains evidence, not edit permission.

## Retained logs

Primary complete validation logs are ignored local artifacts:

- `local-artifacts/codex-runs/sprint40-1-task3-primary/rust-validation.log`
- `local-artifacts/codex-runs/sprint40-1-task3-primary/strict-rustdoc.log`
- `local-artifacts/codex-runs/sprint40-1-task3-primary/vscode-validation.log`

They contain no credentials or external payloads and are not tracked review
inputs.
