# Run the next OneAgent sprint

## Launch command

Use this current user instruction to authorize one commit per completed task:

```text
Запусти следующий спринт с отдельным коммитом после каждой успешно завершённой задачи по промту docs/codex/prompts/run-next-sprint.md.
```

Without the explicit per-task commit clause, plan without staging or committing
and stop before the first committed prerequisite.

## Launch contract

Plan the one eligible OneAgent sprint, generate a Prompt Contract v2 suite, and
dispatch its children in dependency order.

The current user instruction that launches this prompt authorizes:

- one guaranteed fresh-context task runner for each manifest child, executed
  sequentially and without further delegation; and
- one guaranteed fresh-context read-only reviewer when the selected Review
  workflow requires it.

This authorization does not permit unrelated subagents, external side effects,
or commits unless the launch instruction explicitly authorizes commits. If the
runtime cannot guarantee a required fresh context, stop at that boundary and
report the exact prompt path and committed prerequisite for manual continuation.

Stop after the first blocking failure. Do not skip, reorder, combine, or
partially commit dependent tasks.

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts, identifiers, comments, errors, public APIs, prompt
  files, and commit messages in English.
- Report verified repository evidence and measured runtime telemetry only.
- A conservative context estimate is allowed only for preflight admission and
  must be labelled as an estimate.

## Selected framework

- Planning Profile: `docs/codex/profiles/architecture.md`
- Planning Template: `docs/codex/templates/sprint-planning-task.md`
- Base child Template: `docs/codex/templates/task-prompt.md`
- Execution Template: `docs/codex/templates/sprint-execution-loop.md`
- Execution Workflow: `docs/codex/workflows/sequential-sprint-execution.md`
- Context Core: `docs/codex/core/context-management.md`

Read applicable `AGENTS.md`, this prompt, the listed planning modules, and only
the initial material admitted by the planning Context Manifest. Do not preload
framework modules or authorities belonging only to future child tasks.

## Planning Context Manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, Git
  branch/release workflow, and GUI validation.
- `docs/Roadmap.md` — sections: status table, prompt-template readiness stage
  for the eligible sprint, preceding sprint completion, and eligible sprint
  section.
- `docs/codex/README.md` — sections: layer model, Prompt Contract v2, routing,
  sprint lifecycle, and validation.
- preceding sprint review — sections: decision, remaining blockers, state
  transition, and next-sprint hand-off.
- repository definitions, consumers, fixtures, and tests found by bounded `rg`
  queries for the eligible sprint capability.

### Lookup on demand

- `docs/Architecture.md` — trigger: the eligible Roadmap section names an
  unresolved ownership or compatibility boundary; load only matching sections.
- `docs/architecture/semantic-model-2.md` — trigger: the sprint changes semantic
  identity, authority, resolution, graph, or impact; load only matching sections.
- historical ADRs and reviews — trigger: the current sprint or preceding review
  cites them as live prerequisites; load only decision and boundary sections.
- implementation history — trigger: live APIs or accepted scope remain
  ambiguous after current source and tests are inspected.

### Excluded from initial context

- complete Roadmap, Architecture, and semantic-model documents;
- unrelated historical sprint plans and prompt suites;
- complete generated projects, fixture corpora, and successful command logs;
- every future child prompt and its framework modules.

### Preflight

Apply `docs/codex/core/context-management.md`. Record effective context window
or `unknown`, telemetry or labelled estimate basis, static and authority
allocation, and `pass|warning|blocked`. Narrow selectors at warning and stop at
the hard limit.

## Phase 1: resolve eligibility and branch workflow

1. Resolve repository root, `HEAD`, branch, status, relevant history, Roadmap
   state, and existing suite directories.
2. Select the unique `active` sprint with a committed plan; otherwise select the
   unique `next` sprint whose predecessor has a committed non-blocking review.
3. Stop on ambiguity, incomplete prerequisites, conflicting worktree changes,
   or multiple eligible sprints.
4. Reconcile the complete branch, review, remediation, merge, and immediate-push
   rules from `AGENTS.md` before the first write. Record the exact current and
   required branch. A stored prompt never overrides those rules.
5. Resolve commit mode only from the current launch instruction.

## Phase 2: evidence and framework readiness

Before writing:

1. Inspect the admitted current evidence, accepted decisions, definitions,
   consumers, fixtures, tests, Coverage state, and validation entry points.
2. Separate confirmed facts, accepted constraints, compatibility requirements,
   unknowns, and external blockers.
3. Verify repository-owned data, a deterministic oracle, focused checks, and a
   known complete validation cycle for the smallest coherent sprint objective.
4. Stop with `SPRINT_BLOCKED_MISSING_DATA` before edits when required data or an
   oracle is unavailable. Report the exact missing evidence, searches, unsafe
   consequence, required artifact or decision, `HEAD`, status, and next action.
5. Audit the smallest sufficient Profile, Workflow, and specialized Template set.
   Modify the framework only for a concrete reusable gap and keep that change a
   separate committed prerequisite when commits are authorized.

Do not create an investigation task to hide a hard external-data blocker. Do not
create speculative framework modules.

## Phase 3: plan and generate Prompt Contract v2

Create one dependency-ordered plan with the smallest coherent child tasks. Each
child owns one outcome and declares:

- stable order, kind, title, Profile, and specialized Template;
- exact committed prerequisite;
- exact included and excluded scope;
- observable acceptance criteria;
- task-specific validation additions;
- expected files or areas when confirmed; and
- one concise English suggested commit message.

Create:

```text
docs/codex/prompts/sprint-<N>-<slug>/
  00-sprint-<N>-execution-loop.md
  01-<task>.md
  02-<task>.md
  ...
```

Every child must use `prompt_contract: v2`, the base task Template, a bounded
Context Manifest, and `fresh_context: required`. Do not copy permanent Core or
Workflow text. Use exact sections, symbols, ranges, diffs, or queries for large
authorities. Put optional evidence in `Lookup on demand` with an explicit
trigger.

The final child is an integration review. It selects the Review Profile,
Template, and Workflow, receives the exact immutable range and validation
matrix, launches the separately authorized fresh read-only reviewer, reconciles
evidence, and performs same-reviewer artifact consistency before state change.

The master prompt uses the Sprint Execution Loop Template and is only a
dispatcher plus durable ledger. Its manifest records:

| Order | Prompt | Prerequisite | Outcome | Validation additions | Commit message |
|---:|---|---|---|---|---|

Record the exact immediately preceding suite inventory for conditional
retirement by the final review. Do not retire prompts during planning or
implementation.

Update `docs/Roadmap.md` with the plan without marking the sprint completed.
Synchronize other architecture documents only when current evidence requires
it.

## Planning Change Contract

Before writing, print in Russian:

```text
Goal
Files to create
Files to modify
Files not to modify
Behavioral impact
Risks
Validation commands
Suggested commit message
```

List and preserve every pre-existing modified, staged, or untracked path.

## Planning validation

Run:

```bash
set -o pipefail
find docs/codex/prompts/sprint-<N>-<slug> -maxdepth 1 -type f -name '[0-9][0-9]-*.md' ! -name '00-*' -print0 \
  | xargs -0 scripts/validate-codex-prompts.sh
git diff --check
git status --short
```

Also verify links, contiguous numbering, manifest and prerequisite order,
commit-message agreement, Context Manifest selectors, budget preflight,
accepted versus deferred scope, unchanged `next` state, previous-suite
inventory, review handoff, and absence of unrelated changes.

Use `docs/codex/core/validation.md` as the only canonical validation source. Do
not copy its full command matrix into generated prompts.

When commit mode is authorized, stage only enumerated planning-owned paths,
commit the immutable plan as `Plan Sprint <N> <subject>`, immediately follow the
applicable push rule, and stop on push failure. Without commit authorization,
do not stage and stop before a committed prerequisite.

## Phase 4: fresh-context dispatch

After the planning baseline is committed, execute
`docs/codex/workflows/sequential-sprint-execution.md`.

For each child:

1. create one guaranteed fresh context;
2. pass only the child prompt, current `HEAD` and status, exact committed
   prerequisite, applicable instructions, selected framework modules, and
   admitted Context Manifest;
3. receive a compact structured outcome rather than its transcript;
4. verify repository state, validation, commit, and required push from the
   parent context; and
5. record the durable ledger before dispatching the next child.

Never continue a second child inside the first child's context. Never pass one
child's implementation reasoning to another child or to the independent
reviewer.

## Failure and final report

On the first failure, preserve the task diff and evidence, leave later tasks
`not_started`, and report the exact command, error, affected paths,
recoverability, and next action.

The final Russian report includes:

- sprint, objective, starting and ending state, `HEAD`, branch, and status;
- planning/framework decision and Context Manifest preflight;
- ordered task outcomes, commits, pushes, and validation summaries;
- blockers, already-complete evidence, and tasks not started;
- reviewer identity, fresh/read-only evidence, reconciliation, and consistency;
- current and retired suite result;
- measured token telemetry when available, otherwise `недоступно`;
- retained large-log artifact paths;
- next-sprint eligibility and one exact next action.

Do not repeat the same metric in several report sections. Do not present
estimated token usage as measured usage.
