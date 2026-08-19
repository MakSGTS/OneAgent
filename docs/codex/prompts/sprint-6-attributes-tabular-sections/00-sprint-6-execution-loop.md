# Sprint 6 Attributes and Tabular Sections Execution Loop

Execute this master prompt in the current Codex task. Load and execute every
eligible child prompt in manifest order; do not merely summarize them.

## Reporting

- Communicate with the user in Russian.
- Keep repository code, identifiers, documentation, tests, errors, public APIs,
  prompt contents, and commit messages in English.
- Report only live repository evidence or accepted architecture.
- Send concise progress updates and continue automatically after a successful
  child task.

## Canonical authority

- `docs/Roadmap.md`, especially the Sprint 6 execution plan;
- `docs/architecture/semantic-model-2.md`;
- accepted ADRs in `docs/adr/`;
- the live Codex Framework selected by each child prompt.

Stored prompts are not a live baseline. Recheck HEAD, Roadmap, APIs, tests,
fixtures, Coverage, and accepted decisions before every child task.

## Ordered manifest

| # | Prompt | Required gate | Commit message |
|---:|---|---|---|
| 01 | `01-investigate-member-source-contracts.md` | Sprint 5 and the v0.2 review are completed; Sprint 6 is the live target. | `Investigate Sprint 6 member source contracts` |
| 02 | `02-define-member-semantics.md` | Task 01 is committed and its investigation is decision-ready. | `Define Sprint 6 member semantics` |
| 03 | `03-implement-member-graph-model.md` | Task 02 accepted the Sprint 6 semantic contract. | `Implement Sprint 6 member graph model` |
| 04 | `04-parse-edt-member-semantics.md` | Required Task 03 model prerequisites are completed or proven unnecessary. | `Parse Sprint 6 EDT member semantics` |
| 05 | `05-emit-member-ownership.md` | Task 04 parser contract is implemented and committed. | `Emit Sprint 6 member ownership` |
| 06 | `06-integrate-member-references.md` | Task 05 is committed and the accepted ADR authorizes additional reference work. | `Integrate Sprint 6 member references` |
| 07 | `07-complete-member-coverage.md` | Tasks 03–06 are completed or explicitly proven unnecessary under the accepted ADR. | `Complete Sprint 6 member coverage` |
| 08 | `08-sprint-6-integration-review.md` | Task 07 is committed and the implementation tree is clean apart from the preserved prompt suite. | `Complete Sprint 6 attributes and tabular sections review` |

All paths are relative to this file's directory.

## Initial audit

1. Resolve the repository root with `git rev-parse --show-toplevel`.
2. Confirm `Cargo.toml`, `docs/codex/README.md`, and
   `docs/architecture/semantic-model-2.md`.
3. Read applicable `AGENTS.md`, `docs/codex/README.md`, the Sprint 6 Roadmap
   plan, and the Codex Core safety, investigation, validation, Change Contract,
   and final-report modules.
4. Run `git status --short`, `git rev-parse --short HEAD`, and
   `git log -15 --oneline`.
5. Record every pre-existing tracked and untracked change. This prompt suite may
   be untracked and must never be staged by a child task.
6. Verify that all manifest prompts exist and are readable.
7. Create an execution ledger containing task, prompt, state, starting HEAD,
   ending HEAD, validation result, commit, and blocker.

Ledger states are `pending`, `in_progress`, `completed`, `already_complete`,
`waiting_for_gate`, `failed`, and `blocked`.

## Execution loop

For each manifest entry:

1. Refresh Git state, Roadmap state, relevant history, APIs, tests, fixtures,
   Coverage, and predecessor evidence.
2. Enforce the child gate against committed repository evidence. Stop at the
   first unmet gate; never edit documentation to manufacture a prerequisite.
3. Read the child prompt, selected profile, template, required Core and Workflow
   modules, and authoritative documents completely.
4. Treat accepted ADRs as fixed. If source evidence contradicts one, stop and
   report the contradiction instead of inventing replacement semantics.
5. Print the exact child Change Contract before editing.
6. Execute only the child scope. Keep investigation, architecture, graph model,
   parser, emission, reference integration, Coverage, and review boundaries
   separate.
7. Run the child-focused checks followed by the required full validation cycle.
8. Review the complete diff. Stage only child-owned paths with explicit
   `git add <path>` commands. Never use `git add .` and never stage anything
   under `docs/codex/prompts/`.
9. After all acceptance criteria and validation commands succeed, create one
   commit with the manifest message. The current user explicitly authorizes
   this commit.
10. Record the commit hash and confirm that no child-created change remains.
11. Continue automatically to the next child.

If live evidence proves that a conditional implementation task is unnecessary
because the accepted contract is already fully satisfied, mark it
`already_complete` only when a specific existing commit and passing validation
prove every acceptance criterion. Do not create an empty commit.

## Failure and review gates

- On implementation, validation, staging, or commit failure, stop immediately,
  preserve the diff, and record the exact failure.
- If Task 01 is not decision-ready, stop before Task 02.
- If Task 02 does not accept an executable production slice, stop before code.
- If Task 08 records `blocked`, commit the completed review record when its own
  checks succeed, leave Sprint 6 incomplete, and stop.
- Mark Sprint 6 `completed` only when Task 08 records `pass` or
  `pass with non-blocking follow-ups` and every completion gate passes.

Do not push, amend, rebase, reset, clean, restore, or rewrite history.

## Final report

Report in Russian:

| Task | Purpose | Status | Commit or blocker |
|---|---|---|---|

Also report exact validation results, final HEAD, exact `git status --short`,
preserved prompt files, the Sprint 6 completion decision, and the next safe
action. Do not claim a later task executed after an unmet gate.
