# Review Sprint 35 External AI Client Compatibility

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
- `docs/architecture/external-ai-client-compatibility-investigation.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0057-external-ai-client-compatibility.md`
- all committed Task 1-5 code, tests, fixtures, docs, and validation evidence

## Prerequisites / immutable review range

- Tasks 1-5 are committed in order with their required validation.
- Resolve the Sprint 35 planning commit and Task 5 head; review that exact
  immutable range and record both hashes.
- The working tree is clean before review drafting.
- The current launch authorizes exactly one fresh-context read-only reviewer.
  Launch no other subagent.

## Task

Review the integrated Sprint 35 baseline. Create
`docs/reviews/sprint-35-external-ai-client-compatibility.md` only for `pass` or
`pass with non-blocking follow-ups`. After independent review, primary
reconciliation, complete validation, and same-reviewer artifact consistency,
transition Sprint 35 to `completed`, make the v0.6 release integration review
eligible, and atomically retire the verified Sprint 34 prompt suite. Sprint 36
must remain `planned`.

## Independent review gate

- Spawn exactly one fresh-context reviewer with no inherited turns and a
  read-only brief. Supply the immutable range, canonical authorities, exact
  required public-client and synthetic matrices, validation commands, scope,
  and required structured report. Do not supply the primary's expected decision.
- Require file-and-line findings ordered by severity; requirements/evidence and
  test-gap matrices; commands, counts, exits, skips, and environment limits;
  scope/API/dependency/security/configuration audits; and one independent
  decision: `pass`, `pass with non-blocking follow-ups`, or `blocked`.
- The reviewer must not edit, stage, commit, download, configure, or launch an
  unapproved client. The primary owns all external-client execution and commits.

## Primary reconciliation and validation

- Reproduce every reviewer claim against the immutable range. Classify it as
  accepted, rejected with evidence, or unresolved; any unresolved blocking
  disagreement blocks completion. Do not silently fix findings in the review
  commit. A production or evidence fix requires a separate authorized task
  commit and a fresh review of the new range.
- Independently rerun the exact supported-version protocol/lifecycle/shape/
  error/isolation matrix, production public-process matrix, exact Codex and
  Cursor workflows, catalog/Tool Policy/semantic regression evidence, existing
  client compatibility, all audits, and the canonical full workspace gate.
- Confirm all filters match non-zero tests, required rows are not skipped,
  client binaries/config/logs remain ignored, global configs are unchanged,
  no personal path or secret is tracked, and every external claim is bounded by
  actual executable evidence.

## Review artifact and state transition

The review records the immutable range; exact clients and protocol revisions;
requirement-to-evidence matrix; independent findings; primary reconciliation;
focused/public-client/full commands and outcomes; compatibility, API,
dependency, configuration, security, cleanup, scope, and deferred-scope audits;
preserved modern protocol and seven-tool semantics; follow-ups; and decision.

Before state transition, send the drafted review, Roadmap diff, and exact
retirement diff to the same reviewer. Require an explicit pass for internal
consistency, exact commit/range/command/count claims, decision/state agreement,
v0.6 release-review hand-off, Sprint 36 remaining planned, and exact retirement
inventory. A failed check blocks completion.

Only after all gates pass:

- mark Sprint 35 `completed` in `docs/Roadmap.md`;
- record that the v0.6 release integration review is the next eligible gate and
  Sprint 36 remains `planned`;
- delete exactly the eight verified tracked files under
  `docs/codex/prompts/sprint-34-edt-integration-prototype/` and no others;
- stage only the review, Roadmap, and exact retirement paths.

## Blocking conditions

Any blocking correctness, compatibility, security, lifecycle, cleanup,
evidence, dependency, scope, or documentation finding; unsupported or missing
client executable; failed or skipped required row; global configuration
mutation; external-access breach; failed validation; inconsistent artifact;
retirement inventory drift; unrelated change; or failed commit blocks the
review. Preserve Sprint 35 as incomplete and keep the Sprint 34 suite.

## Suggested commit message

`Complete Sprint 35 external AI client compatibility review`

## Final report additions

Report reviewer identity and fresh/read-only proof, immutable range, findings
and reconciliation, exact client/protocol/public-process/full validation,
decision, review artifact, state transition, v0.6 review eligibility, Sprint 36
state, exact retired files, preserved paths, commit, and remaining changes.
