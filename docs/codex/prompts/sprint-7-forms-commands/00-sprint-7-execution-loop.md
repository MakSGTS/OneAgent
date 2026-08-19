# Sprint 7 Forms and Commands Execution Loop

Execute this master prompt in the current Codex task. Load and execute every
eligible Sprint 7 child prompt in manifest order; do not merely summarize them.
After a non-blocking Sprint 7 review, execute the post-sprint Codex Framework
audit as a separate follow-up step.

## Reporting

- Communicate with the user in Russian.
- Keep repository code, identifiers, documentation, tests, errors, public APIs,
  prompt contents, and commit messages in English.
- Report only live repository evidence or accepted architecture.
- Send concise progress updates and continue automatically after each
  successful committed child task.

## Required workflow and template

- `docs/codex/workflows/sequential-sprint-execution.md`
- `docs/codex/templates/sprint-execution-loop.md`

Read both files and `docs/codex/README.md` completely before execution.

## Canonical authority

- `docs/Roadmap.md`, especially the Sprint 7 execution plan;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`;
- the accepted ADRs referenced by ADR-0029;
- the live Codex Framework selected by each child prompt.

Stored prompts and historical HEAD values are not a live baseline. Recheck the
Roadmap, APIs, tests, fixtures, Coverage, accepted decisions, and Git state
before every child task.

## Sprint objective and starting state

Sprint 7 adds only the accepted executable-module, Command parameter-reference,
and explicit static Form-navigation slice. Existing Form and Command
declarations and their metadata ownership are compatibility baselines, not work
to recreate.

Before Task 01:

1. Confirm Sprint 6 is completed and Sprint 7 is the live target.
2. Confirm the planning baseline containing the source investigation, accepted
   ADR-0029, Sprint 7 Roadmap plan, and synchronized Semantic Model boundary is
   committed. Do not use uncommitted planning files as a prerequisite.
3. Record exact starting `HEAD` and `git status --short`.
4. Classify and preserve every pre-existing tracked, staged, and untracked
   path, including this prompt suite.
5. Verify every manifest prompt and authoritative document exists.
6. Require explicit commit authorization from the current instruction that
   launches this execution loop.

## Commit authorization mode

This stored prompt is not permanent authorization to stage or commit. Start the
ordered loop only when the current launching instruction explicitly authorizes
the complete Sprint 7 commit sequence. Otherwise stop before Task 01 and report
that authorization is missing.

When authorized, every successfully completed child task must produce exactly
one logical commit with its manifest message. Stage only explicitly enumerated
task-owned paths. Never use broad staging. An `already_complete` task produces
no empty commit and must identify the proving committed baseline.

## Ordered Sprint 7 manifest

| # | Prompt | Required committed gate | Owned outcome | Focused validation | Commit message |
|---:|---|---|---|---|---|
| 01 | `docs/codex/prompts/sprint-7-forms-commands/01-implement-form-command-graph-prerequisites.md` | Accepted Sprint 7 planning baseline. | Source-independent graph contract for module ownership, Command references, and `Opens`. | Graph library, validation, Diff, then full workspace gate. | `Define Sprint 7 graph navigation model` |
| 02 | `docs/codex/prompts/sprint-7-forms-commands/02-parse-form-command-module-layouts.md` | Task 01 committed or proven `already_complete`. | Deterministic Form and Command module observations. | EDT module-reader tests, then full workspace gate. | `Parse Sprint 7 form and command modules` |
| 03 | `docs/codex/prompts/sprint-7-forms-commands/03-emit-form-command-modules.md` | Task 02 committed or proven `already_complete`. | Canonical modules, ownership, and existing BSL semantics. | EDT producer and affected graph tests, then full workspace gate. | `Emit Sprint 7 form and command modules` |
| 04 | `docs/codex/prompts/sprint-7-forms-commands/04-parse-command-parameter-references.md` | Task 03 committed or proven `already_complete`. | Typed Command parameter-reference observations. | EDT metadata-structure tests, then full workspace gate. | `Parse Sprint 7 command parameter references` |
| 05 | `docs/codex/prompts/sprint-7-forms-commands/05-integrate-command-parameter-references.md` | Task 04 committed or proven `already_complete`. | Public request lifecycle and resolved Command projections. | Reference-request and EDT tests, then full workspace gate. | `Integrate Sprint 7 command references` |
| 06 | `docs/codex/prompts/sprint-7-forms-commands/06-parse-static-form-navigation.md` | Task 05 committed or proven `already_complete`. | Typed complete-statement static `OpenForm` candidates. | BSL tests, then full workspace gate. | `Parse Sprint 7 static form navigation` |
| 07 | `docs/codex/prompts/sprint-7-forms-commands/07-emit-form-navigation.md` | Task 06 committed or proven `already_complete`. | Canonical resolved `Procedure --Opens--> Form` facts. | Graph validation and EDT tests, then full workspace gate. | `Emit Sprint 7 form navigation` |
| 08 | `docs/codex/prompts/sprint-7-forms-commands/08-complete-sprint-7-production-evidence.md` | Tasks 01–07 committed or proven `already_complete`. | Production matrix, Coverage transitions, aggregates, and current-state docs. | Graph/EDT Coverage plus full workspace gate. | `Complete Sprint 7 production evidence` |
| 09 | `docs/codex/prompts/sprint-7-forms-commands/09-sprint-7-integration-review.md` | Task 08 committed and all implementation validation successful. | Review decision, evidence record, and allowed Sprint transition. | Complete focused matrix and full workspace gate. | `Complete Sprint 7 forms and commands review` |

The manifest is an execution plan, not proof that any task is complete.

## Execution ledger and loop

Create a ledger with task, prompt, state, starting HEAD, ending HEAD, validation,
commit, and blocker. States are `pending`, `in_progress`, `completed`,
`already_complete`, `waiting_for_gate`, `failed`, and `blocked`.

For each manifest row:

1. Refresh Git, Roadmap, history, relevant APIs, tests, fixtures, Coverage, and
   predecessor evidence.
2. Enforce the committed prerequisite before edits.
3. Read the child prompt, selected Profile, Template, required Core and Workflow
   modules, and authoritative documents completely.
4. Print the exact child Change Contract before editing.
5. Execute only the child-owned outcome and preserve unrelated changes.
6. Run focused validation followed by the complete required workspace gate.
7. Confirm every filter ran meaningful tests, inspect the complete diff, and
   recheck all acceptance and exclusion clauses.
8. Stage only child-owned paths and create the one manifest commit.
9. Record the commit hash and verify no child-created change remains.
10. Continue automatically only when the next committed gate is satisfied.

Do not stage or modify files under
`docs/codex/prompts/sprint-7-forms-commands/` during child execution.

## Failure and review gates

- Stop on the first prerequisite, architecture, implementation, validation,
  staging, or commit failure. Never skip a blocked task.
- Do not repair unrelated pre-existing changes to obtain a clean status.
- A blocked Task 09 leaves Sprint 7 incomplete and Sprint 8 ineligible.
- Mark Sprint 7 `completed` only after a Task 09 decision of `pass` or
  `pass with non-blocking follow-ups` and successful required validation.
- Do not push, amend, rebase, reset, clean, restore, or rewrite history.

## Post-sprint Codex Framework audit

Only after Task 09 is committed with a non-blocking decision and the Roadmap
shows Sprint 7 completed, execute:

`docs/codex/prompts/sprint-7-forms-commands/10-audit-codex-framework-after-sprint-7.md`

This is not a Sprint 7 implementation task. If the audit proves a reusable
Framework gap and commit mode is authorized, update only the smallest correct
Framework layer and create one separate commit. If no reusable gap is proven,
record `no_change` and do not create an empty commit. Failure of this
post-sprint audit does not rewrite an already justified Sprint 7 review
decision, but it must be reported.

## Final report

Report in Russian:

| Task | Purpose | Status | Commit or blocker |
|---|---|---|---|

Also report starting and ending HEAD, exact validation results, already-complete
evidence, preserved paths, all commits, the Sprint 7 decision, the Framework
audit outcome and optional commit, exact final `git status --short`, and the
next safe action.
