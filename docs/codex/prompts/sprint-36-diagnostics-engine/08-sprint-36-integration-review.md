# Review Sprint 36 Diagnostics Engine

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/review.md`
- `docs/codex/templates/review-task.md`

## Required workflow

`docs/codex/workflows/review.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/reviews/v0.6-release-review.md`
- all committed Task 1–7 code, tests, fixtures, docs, and validation evidence

## Prerequisites / immutable review range

- Tasks 1–7 are committed in order with their required validation.
- Resolve the Sprint 36 planning commit and Task 7 head; review that exact
  immutable range and record both hashes.
- The working tree is clean before review drafting.
- The current launch authorizes exactly one fresh-context read-only reviewer.
  Launch no other subagent.

## Task

Review the integrated Sprint 36 baseline. Create
`docs/reviews/sprint-36-diagnostics-engine.md` only for `pass` or `pass with
non-blocking follow-ups`. After independent review, primary reconciliation,
complete validation, and same-reviewer artifact consistency, transition Sprint
36 to `completed`, make Sprint 37 — Rules Engine the unique `next` target, and
atomically retire the verified Sprint 35 prompt suite.

## Independent review gate

- Spawn exactly one fresh-context reviewer with no inherited turns and a
  read-only brief. Supply the immutable range, canonical authorities, exact
  acceptance criteria and exclusions, required domain/orchestration/Workspace/
  cache/MCP/LSP/public-process matrices, validation commands, and structured
  output contract. Do not supply the primary's expected decision or conclusions.
- Require exact reviewed range and initial state; file-and-line findings ordered
  by severity; acceptance-evidence and test-gap matrices; commands, counts,
  exits, skips, and environment limits; scope/API/dependency/security audits;
  missing evidence separate from defects; and one independent decision:
  `pass`, `pass with non-blocking follow-ups`, or `blocked`.
- The reviewer must not edit, create, delete, stage, commit, configure, download,
  or delegate. It may run only non-destructive repository inspection and
  validation commands.

## Primary reconciliation and validation

- Independently inspect the exact range and reproduce every reviewer claim.
  Classify each as accepted, rejected with evidence, or unresolved. The final
  decision must not be less severe than the reviewer's recommendation, and any
  unresolved disagreement blocks completion.
- Verify every ADR-0058 identity, vocabulary, suppression, ordering, bound,
  report, failure, ownership, sensitive-data, snapshot/cache, MCP, LSP, and
  compatibility criterion plus every explicit exclusion.
- Independently rerun the complete non-zero Graph/Analysis/Workspace/cache/
  watching/Runtime/MCP/LSP/public-process matrix, catalog/schema/Tool Policy/
  capability/confinement audits, and canonical full workspace validation.
- Confirm Graph and existing producers remain authoritative; no configurable
  rule registry, unsupported producer, UI, mutable-document analysis, source
  mutation, edit workflow, telemetry, remote transport, or false Coverage claim
  entered the range.
- Do not silently fix findings. A production or evidence fix requires a
  separate authorized task commit and a fresh review of the new immutable range.

## Review artifact and state transition

The review records the immutable range; requirement-to-evidence matrix;
independent findings; primary reconciliation; exact focused/public/full
commands and outcomes; domain, orchestration, Workspace/cache, MCP/LSP,
compatibility, API, dependency, sensitive-data, scope, and deferred-scope
audits; follow-ups; and effective decision.

Before state transition, send the drafted review, Roadmap diff, and exact
retirement diff to the same reviewer. Require explicit read-only confirmation
that every finding, missing-evidence item, decision, validation result, risk,
Sprint 37 hand-off, and retirement inventory is preserved without weakening.
A failed or unavailable consistency check blocks completion.

Only after all gates pass:

- mark Sprint 36 `completed` in `docs/Roadmap.md`;
- make Sprint 37 — Rules Engine the unique `next` target;
- delete exactly the seven verified tracked files under
  `docs/codex/prompts/sprint-35-external-ai-client-compatibility/` and no others;
- stage only the review, Roadmap, exact current-state documents explicitly owned
  by the review, and exact retirement paths.

## Blocking conditions

Any blocking correctness, identity, ordering, suppression, bound, snapshot,
cache, protocol, policy, confinement, compatibility, evidence, dependency,
scope, or documentation finding; failed or zero-match required validation;
reviewer mutation or incomplete output; unresolved evidence disagreement;
inconsistent artifact; retirement inventory drift; unrelated change; or failed
commit blocks the review. Preserve Sprint 36 as incomplete and keep the Sprint
35 suite.

## Suggested commit message

`Complete Sprint 36 diagnostics engine review`

## Final report additions

Report reviewer identity and fresh/read-only proof, immutable range, findings
and reconciliation, exact focused/public/full validation, effective decision,
review artifact, state transition, Sprint 37 eligibility, exact retired files,
artifact-consistency result, preserved paths, commit, and remaining changes.
