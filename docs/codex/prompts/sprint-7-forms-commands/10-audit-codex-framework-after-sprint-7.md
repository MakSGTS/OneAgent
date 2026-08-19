# Post-Sprint 7: Audit Codex Framework task templates

Continue OneAgent development after Sprint 7 completion.

## Reporting

- Communicate with the user in Russian.
- Keep repository documentation, prompt text, identifiers, and commit messages
  in English.
- Distinguish confirmed execution evidence, reusable gaps, one-off Sprint 7
  details, and unsupported assumptions.

## Profile

`docs/codex/profiles/implementation.md`

## Template

`docs/codex/templates/implementation-task.md`

Read the Profile, Template, their required Core and Workflow modules,
`docs/codex/README.md`, and the complete current Framework before acting.

## Authoritative Framework documents

- `docs/codex/README.md`;
- `docs/codex/core/`;
- `docs/codex/workflows/`;
- `docs/codex/profiles/`;
- `docs/codex/templates/`.

## Required gate

Proceed only after Task 09 is committed with decision `pass` or
`pass with non-blocking follow-ups`, Sprint 7 is marked `completed`, and the
working tree contains no Sprint implementation or review changes. This audit is
not allowed to repair a blocked sprint.

## Task

Analyze the actual Sprint 7 prompt suite and execution evidence to determine
whether the reusable Codex Framework templates, profiles, workflows, or README
need a small evidence-backed update. Prefer `no_change`: one sprint-specific
detail is not a reusable Framework rule.

## Evidence to inspect

- `docs/codex/prompts/sprint-7-forms-commands/`;
- `docs/codex/templates/`;
- `docs/codex/profiles/`;
- `docs/codex/workflows/`;
- `docs/codex/core/`;
- `docs/codex/README.md`;
- the Sprint 7 implementation commits and execution ledger;
- `docs/reviews/sprint-7-forms-commands.md`;
- repeated prompt text, blockers, missed gates, manual recovery, validation
  omissions, staging/commit issues, and final-report gaps observed during the
  sprint.

## Scope

One evidence-based Framework retrospective with either a minimal reusable
documentation change or an explicit `no_change` result.

## Included

- Classify each finding as task-specific, already covered, documentation drift,
  or a reusable Framework gap.
- Update only the smallest correct Framework layer when live evidence proves a
  recurring rule or missing contract.
- Keep Core rules generic, workflows procedural, profiles compositional, and
  templates structural.
- Synchronize `docs/codex/README.md` only when the public Framework structure
  or selection guidance changes.
- Add a focused Roadmap evidence note only if the current Roadmap already tracks
  Framework maintenance and the update changes that state.

## Excluded

- Rewriting historical Sprint 7 prompts to make their completed execution look
  cleaner.
- Production Rust, tests, fixtures, architecture ADRs, Coverage, or Sprint
  implementation changes.
- Moving one-off Form/Command requirements into reusable templates.
- Duplicating rules already owned by Core or Workflow modules.
- Broad stylistic rewrites without execution evidence.

## Acceptance criteria

- Every proposed Framework change cites at least one concrete Sprint 7
  execution problem and explains why the rule is reusable.
- The change is made in the lowest appropriate Framework layer and introduces
  no contradictory duplicate rule.
- Profile and Template selection remains minimal and consistent with
  `docs/codex/README.md`.
- Commit authorization remains current-instruction-only; stored prompts do not
  permanently authorize Git actions.
- Sequential execution, prerequisite, `already_complete`, failure, review,
  and final-state contracts remain coherent.
- If no reusable gap is proven, no file is modified and no empty commit is
  created.

## Task-specific validation

For a `no_change` result, report the inspected evidence and exact final Git
status.

For a Framework update, run:

```bash
git diff --check
git diff -- docs/codex docs/Roadmap.md
git status --short
```

Manually verify all relative links, layer ownership, profile/template
references, and absence of contradictory duplicated rules. Discover and run an
existing Markdown linter or link checker if the repository provides one. Do not
run the Rust workspace gate for a documentation-only Framework update.

## Commit

This stored prompt is not permanent commit authorization. If the current
launching instruction explicitly authorizes the post-sprint Framework commit
and the audit produced a validated change, stage only the explicitly changed
Framework and necessary Roadmap paths and create one commit:

```text
Refine Codex task templates after Sprint 7
```

For `no_change`, do not stage or create an empty commit.

## Final report additions

Report the audit evidence, classification of every finding, whether a reusable
gap exists, files changed, layer rationale, validation, optional commit hash,
preserved historical prompts, exact Git status, and recommended next action.
