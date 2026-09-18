# Sprint 40.1 Refactoring Planner Remediation Design Review

## Decision

`pass`

The remediation design places the single canonical checked publication
identity, compatibility alias, Runtime sequencing boundary, negative oracles,
live evidence synchronization, and governance-only audit at the accepted
owners. No blocking contradiction with ADR-0063 or the committed scope baseline
was found. Production implementation has not started.

## Reviewed baseline

- Branch: `codex/v0.7-sprint-40.1`.
- Start: `427a78cd809a16bae2ee160867b20bb64c1d415e`, merge subject
  `Merge Sprint 40 remediation`.
- End: `f2483088d1a463bb811d698651191bdcd93727e6`, merge subject
  `Merge v0.7 roadmap into Sprint 40.1`.
- Exact range:
  `427a78cd809a16bae2ee160867b20bb64c1d415e..f2483088d1a463bb811d698651191bdcd93727e6`.
- Inventory: six commits, nine Markdown paths, 880 additions, 42 deletions,
  and no binary or production paths.
- Initial and final `git status --short`: empty.

The later roadmap merge remains inside the immutable reviewed range as
governance scope. From `68dbe889` through the reviewed end, only
`docs/Roadmap.md` and `docs/roadmap-calendar-forecast.md` changed. The Sprint
40.1 Roadmap section is byte-identical before and after that merge, with
SHA-256
`82893581c7b90a9c628423c22544d40ef1348df8cc8351ddd31de707ebc62d41`.

## Independent reviewer

The guaranteed fresh-context read-only reviewer was
`/root/s40_1_design_review`. It received the repository root, child prompt,
authorities, exact committed range, criteria, exclusions, validation contract,
and output contract without implementation transcripts or an expected
decision. It did not delegate, mutate files, stage, commit, push, or otherwise
change Git state.

The reviewer reported an effective context window of `unknown`, admitted only
the bounded Context Manifest and trigger-selected lookup material, and returned
a context preflight result of `pass`. Runtime token telemetry was unavailable.

## Contract reconciliation

The Roadmap and master prompt contain byte-identical sprint efficiency,
invariant-matrix, design-review-gate, and implementation-baseline records. Both
extractions have SHA-256
`4279ddbc7f195f0cf6b3e823b579fcfbad0de000a5f1157eda730e0348138684`.
The accepted implementation baseline is at most ten task-owned paths, 600
textual additions plus deletions, no binary paths, nine focused check groups,
and one stable full workspace gate.

## ADR invariant matrix

| Accepted invariant | Owner, ordering, and retention | Negative oracle and validation | Decision |
| --- | --- | --- | --- |
| One canonical checked non-zero Workspace publication identity | `crates/analysis/src/publication.rs` owns `WorkspacePublicationId`, exported through `crates/analysis/src/lib.rs`; construction and checked successor precede publication replacement | Public Analysis tests reject zero and preserve the existing closed overflow behavior; Runtime proves overflow cannot replace a publication | pass |
| Change Impact remains source-compatible without a second sequence | `crates/analysis/src/change_impact.rs` exports `ChangeImpactPublicationId` as an alias or exact projection of the canonical type before report construction | One public compile/runtime test imports both names and proves type and value identity; duplicate-newtype and conversion searches remain empty | pass |
| Refactoring and Change Impact observe the same publication | `crates/analysis/src/refactoring.rs` consumes the canonical identity and `apps/runtime/src/workspace/mod.rs` owns one checked sequence and atomic snapshot/impact retention | Refactoring, Workspace, watching, Git-input, and cache tests preserve stale, overflow, failed-build, recovery, and fresh-service behavior | pass |
| Public behavior remains read-only and wire-compatible | Runtime projects the canonical Rust identity to the unchanged bounded numeric wire value; Protocol, Tool Policy, catalog, clients, and cache retain their current contracts | Legacy revision/catalog, denial, oversize, EOF, repeated-session, and explicit no-edit assertions remain required | pass |
| Final evidence matches the immutable reviewed head | `docs/architecture/refactoring-planner-evidence.md` and matching Roadmap evidence are derived after the stable implementation diff and before review dispatch | Any stale, additive, zero-match, filtered, or unreconciled count blocks completion; focused and all-target inventories must be enumerated live | pass |
| Governance changes do not become product scope | Commits `dce7470e`, `25019f17`, `f1698840`, `89ce1402`, `6e038660`, and `8a8e4734` are classified separately before product acceptance | Their combined net effect is confined to `docs/codex/**` and `scripts/validate-codex-prompts.sh`; any Rust, product, or API effect would block review | pass |

## Findings

There are no blocking, high, or medium findings.

Low: the implementation prompt says to preserve checked zero and overflow
tests, but the baseline contains no explicit `WorkspacePublicationId::new(0)`
or `ChangeImpactPublicationId::new(0)` oracle. The overflow oracle exists. Task
2 must add an explicit zero-rejection oracle together with the public-path and
alias-identity regression rather than treat zero coverage as already present.

## Missing pre-implementation evidence

The following evidence is expected to be absent before Task 2 and is not a
design defect:

- the canonical `publication` module, forward compatibility alias, and
  canonical Runtime vocabulary;
- a public compile/runtime alias oracle and explicit zero-rejection test;
- post-remediation live focused and all-target counts;
- the stable full workspace gate and actual path, churn, and binary totals;
- the final implementation net-diff and integration scope audit.

No evidence required for the design decision itself is missing.

## Scope decision

The reviewed planning range is documentation-only and contains no Cargo,
Rust product, protocol, cache, Graph, client, source-mutation, transaction, or
Sprint 41 implementation path. The committed stop-loss is sufficient for the
planned owner move, compatibility alias, direct consumer migration, focused
regressions, and evidence update. Task 2 must calculate actual scope from this
review commit and stop if it exceeds the committed limit.

## Validation

- `bash -n scripts/validate-codex-prompts.sh` passed.
- Explicit prompt validation passed all three child prompts.
- Default prompt validation passed all 14 discovered prompt files.
- The bounded Markdown link check inspected 431 local links and found zero
  broken targets.
- `git diff --check` and range-specific `git diff --check` passed.
- Bounded Rust inventory found 73 publication-name occurrences in nine paths,
  one existing newtype in `change_impact.rs`, and no canonical module, forward
  alias, or explicit zero oracle on the pre-implementation baseline.
- Governance range checks found no product-path net diff.

Cargo tests and the canonical full workspace gate were intentionally not run
for this documentation-only targeted design review. No Cargo test filter was
reported as evidence.

## Next action

Commit and push this decision artifact as
`Approve Sprint 40.1 remediation design`. Task 2 may then implement the bounded
contract remediation from that exact committed prerequisite.
