# Run the next OneAgent sprint

## Launch command

Use the following current user instruction to run this prompt with one commit
per successfully completed task:

```text
Запусти следующий спринт с отдельным коммитом после каждой успешно завершённой задачи по промту docs/codex/prompts/run-next-sprint.md.
```

The shorter instruction `Запусти следующий спринт по промту
docs/codex/prompts/run-next-sprint.md` starts the workflow without commit
authorization. In that mode, stop when the next prerequisite requires a
committed baseline.

---

Continue OneAgent development.

## Launch contract

This prompt bootstraps, plans, generates, and executes the next eligible
OneAgent sprint.

Resolve commit mode only from the current user instruction that launches this
prompt:

- The current instruction must explicitly request a separate commit after each
  successfully completed task to authorize staging and committing task-owned
  paths. In that mode, the final integration-review task and its commit also
  own the conditional retirement of the immediately preceding sprint's prompt
  suite defined below; prompt retirement is not a separate extra commit.
- An instruction that only requests running the next sprint does not authorize
  commits. Do not stage or commit in that mode, and stop whenever the next
  prerequisite requires a committed baseline.
- This stored prompt never authorizes staging or committing by itself.

Stop after the first blocking failure. Do not skip, reorder, combine, or
partially commit dependent tasks.

## Reporting

- Communicate with the user in Russian.
- Keep source code, identifiers, repository documentation, comments, tests,
  error messages, public APIs, prompt files, and commit messages in English.
- Report only facts verified from the live repository, successful commands, or
  accepted authoritative documents.
- Never estimate test results, token usage, task completion, or elapsed time.

## Planning profile and template

Profile:

- `docs/codex/profiles/architecture.md`

Template:

- `docs/codex/templates/sprint-planning-task.md`

Execution template and workflow:

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Before acting, read completely:

- every applicable `AGENTS.md`;
- `docs/codex/README.md`;
- the selected Profile and Template;
- every Core and Workflow module referenced by them;
- every Profile, Template, Core module, and Workflow selected by a generated
  child task.

## Historical drafting context — revalidation required

At initial prompt drafting time:

- repository root was `/Users/maxim_tomshin/Development/oneagent`;
- `HEAD` was `5b3c3d3`, `Complete Sprint 8 registers and queries review`;
- `docs/Roadmap.md` marked Sprint 8 completed;
- Sprint 9 — Roles and Access Rights was the next planning target;
- `docs/roadmap-calendar-forecast.md` was an unrelated untracked user file;
- the accepted EDT Grants first slice, scoped AccessRight graph model,
  role-right parser, production fixtures, and tests already existed.

These are historical observations, not execution proof. Recheck all mutable
facts before planning or editing. If the live repository differs, use the live
state and report the difference.

## Initial canonical authorities

When the live target is Sprint 9, read at minimum:

- `docs/Roadmap.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/reviews/sprint-8-registers-queries.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `crates/graph/src/access_right.rs`;
- `adapters/edt/src/role_rights.rs`;
- `adapters/edt/tests/grants.rs`;
- `adapters/edt/tests/fixtures/grants_project/`;
- relevant real artifacts under `OneAgent_EDTproject/src/Roles/`.

For every live target, discover and read the Roadmap section, preceding sprint
review, ADRs, architecture documents, implementation, fixtures, tests,
Coverage entries, and consumers that govern that sprint. Do not treat the
Sprint 9 list as authoritative for a later sprint.

## Target sprint resolution

1. Resolve the repository root with `git rev-parse --show-toplevel`.
2. Confirm this is OneAgent using current repository evidence, including
   `Cargo.toml`, `docs/codex/README.md`, and
   `docs/architecture/semantic-model-2.md`.
3. Record exact starting `HEAD`, `git status --short`, relevant Git history,
   Roadmap state, and the existing per-sprint prompt-suite directories.
4. Select the live `active` sprint when one has an accepted committed planning
   baseline and incomplete tasks.
5. Otherwise select the sprint marked `next` whose preceding sprint has a
   committed non-blocking integration review.
6. Stop if Roadmap state is ambiguous, prerequisites are incomplete, or more
   than one sprint appears eligible. Do not guess sprint ownership.

## Phase 1 — Read-only readiness and evidence investigation

Before making any change:

1. Inspect applicable instructions, Git status, recent relevant commits,
   source and test structure, definitions, consumers, fixtures, Coverage
   registries, Roadmap, accepted ADRs, and architecture documents.
2. Separate:
   - confirmed repository evidence;
   - accepted architecture requirements;
   - already implemented compatibility constraints;
   - unknown or unsupported behavior.
3. Identify the smallest coherent sprint objective supported by the live
   Roadmap and repository evidence.
4. Do not automatically implement every capability currently described as
   deferred. For Sprint 9, deny semantics, inheritance, effective
   authorization, runtime user assignments, access groups, and access profiles
   remain excluded unless current source evidence and an accepted architecture
   decision explicitly bring a bounded slice into scope.
5. Determine whether the sprint requires:
   - investigation evidence only;
   - a new or updated architecture decision;
   - graph-model work;
   - parser work;
   - request or resolution work;
   - graph emission;
   - production fixtures and Coverage evidence;
   - documentation synchronization;
   - integration review.
6. Keep investigation, architecture acceptance, graph model, parser,
   resolution, emission, evidence completion, and review as separate task
   boundaries unless the repository's sprint-planning contract proves a
   smaller combined boundary is coherent.

## Mandatory data and testability gate

Before editing planning documents, templates, production code, or prompt files,
verify that the selected sprint scope has enough evidence to plan and test
safely.

Required evidence includes, where applicable:

- real repository-owned source artifacts or provenance-backed fixtures;
- exact serialized fields, value vocabulary, nesting, identity inputs, and
  negative or malformed cases for source parsing;
- existing or accepted semantic meaning, direction, identity, endpoint,
  provenance, validation, and Coverage contracts;
- discoverable production entry points and affected consumers;
- representative positive, negative, missing, ambiguous, incompatible,
  partial, duplicate, reordered, and repeated-build cases as applicable;
- a reliable test oracle or observable acceptance result;
- executable focused validation commands;
- a known full validation cycle for the affected change class.

If any required evidence is unavailable or insufficient:

1. Do not modify files.
2. Do not generate speculative task prompts.
3. Do not implement production behavior.
4. Do not stage or commit anything.
5. Stop the entire sprint run.
6. Print a Russian blocker report beginning with:

   `SPRINT_BLOCKED_MISSING_DATA`

7. Include:
   - sprint number and name;
   - intended capability;
   - exact missing data or test evidence;
   - paths and searches inspected;
   - why implementation or validation would be unsafe;
   - what artifact, fixture, decision, or user input is required;
   - starting and final `HEAD`;
   - final `git status --short`;
   - elapsed investigation time;
   - token usage only if provided by the runtime, otherwise `недоступно`.

An investigation task may be planned only when it can produce the missing
evidence from repository-owned sources. Do not use an investigation task to
hide a hard external-data blocker.

## Architecture planning decision

After the evidence gate:

- Reuse accepted architecture when it fully governs the selected slice.
- Create or update an ADR only when semantics, direction, identity, endpoint
  compatibility, provenance, validation, or first-slice scope is genuinely
  unresolved.
- Record confirmed evidence, alternatives, the accepted decision, rejected
  alternatives, implementation prerequisites, deferred scope, and Coverage
  completion criteria.
- Do not implement production behavior during architecture planning.
- Do not mark a capability Supported from architecture documentation alone.
- If accepted architecture conflicts with real source evidence, stop and
  report the conflict instead of silently replacing the contract.

## Codex Framework and template readiness audit

Inspect:

- the Task prompt template readiness forecast in `docs/Roadmap.md`;
- `docs/codex/README.md`;
- relevant Profiles;
- relevant Templates;
- relevant Workflows;
- recent sprint prompt suites.

Determine whether the smallest existing Profile, Template, and Workflow set can
express every planned task's evidence, prerequisite, safety, validation, and
reporting contract.

If existing framework contracts are sufficient:

- record that decision in the sprint plan;
- do not modify framework files;
- do not add a post-sprint framework audit task without concrete evidence.

If a concrete reusable framework gap exists:

1. Explain the exact gap and the Roadmap stage that requires it.
2. Update only the correct reusable framework layer.
3. Keep framework changes separate from production implementation.
4. Validate links, routing, precedence, and non-duplication.
5. If commit mode is authorized, commit the framework update as one explicit
   prerequisite commit before detailed sprint decomposition.
6. Restart the readiness audit against the committed framework baseline.
7. If the framework gap cannot be resolved safely, stop the sprint.

Do not create speculative Profiles, Templates, or Workflows.

## Phase 2 — Sprint plan and prompt-suite generation

After all readiness gates pass:

1. Define the exact sprint number, name, objective, included scope, excluded
   scope, accepted planning baseline, risks, and completion gates.
2. Produce a dependency-ordered list of the smallest coherent tasks.
3. Give every task:
   - stable order and title;
   - one owned outcome;
   - selected Profile and Template;
   - authoritative documents;
   - required committed prerequisite;
   - exact included and excluded scope;
   - observable acceptance criteria;
   - exact task-specific validation additions;
   - expected files or areas, when discoverable;
   - one concise English commit message.
4. Include an integration-review task as the final task.
5. Add investigation or architecture tasks before implementation only when
   evidence proves they are required.
6. Define `already_complete`, failure, review, sprint-completion, and
   next-sprint hand-off gates.
7. Update `docs/Roadmap.md` with the detailed execution plan without marking
   the sprint completed during planning.
8. Synchronize architecture documents only when the planning evidence requires
   it.
9. Create the prompt suite under:

   `docs/codex/prompts/sprint-<N>-<english-slug>/`

10. Create:
    - `00-sprint-<N>-execution-loop.md` as the master execution prompt;
    - one numbered English prompt file per task:
      `01-<task-slug>.md`, `02-<task-slug>.md`, and so on.

The directory above is the exclusive owner of every generated master and child
prompt for Sprint `<N>`:

- do not create sprint task prompts directly under `docs/codex/prompts/`;
- do not place one sprint's task prompts in another sprint's directory;
- reserve `docs/codex/prompts/run-next-sprint.md` as the reusable bootstrap
  prompt, not as a sprint task;
- keep prompt numbering contiguous inside the sprint directory;
- record the exact immediately preceding suite directory
  `docs/codex/prompts/sprint-<N-1>-<verified-slug>/`, or committed evidence
  that it is already absent, in the generated master prompt and final
  integration-review child prompt;
- make retirement of that preceding suite an explicit conditional output of
  the final integration-review task, never a planning or implementation-task
  side effect.

Every child prompt must be self-contained for a new Codex context and normally
contain:

1. `Continue OneAgent development.`
2. `Reporting`
3. `Profile`
4. `Template`
5. `Authoritative ADRs` and architecture documents
6. `Prerequisites / Required gate`
7. `Task`
8. `Scope` with `Included` and `Excluded`
9. `Acceptance Criteria`
10. `Repository Safety`
11. `Task-specific Validation`
12. `Suggested commit message`
13. `Final report additions`

The generated final integration-review prompt must additionally state that,
only after it has issued `pass` or `pass with non-blocking follow-ups` and all
required validation has succeeded, it owns the previous prompt-suite
retirement procedure from this bootstrap prompt. Its Change Contract and
suggested final review commit must include the exact verified previous suite
files as deletions together with the review artifact and sprint-state
transition.

Use exact paths, APIs, test targets, commits, and artifacts only when verified.
Turn unknowns into explicit investigation questions or blockers. Never invent
XML fields, semantic concepts, identities, fixtures, APIs, tests, or completion
claims.

The generated master prompt must use:

- `docs/codex/templates/sprint-execution-loop.md`;
- `docs/codex/workflows/sequential-sprint-execution.md`.

Its manifest must identify, for every task:

- order and prompt path;
- required committed prerequisite;
- task-owned outcome;
- validation additions;
- suggested commit message.

## Planning Change Contract

Before writing planning files, print in Russian:

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

Use exact repository-relative paths. Explicitly list every pre-existing
modified, staged, or untracked path and explain how it will be preserved.

Unless live scope explicitly proves otherwise, do not modify:

- `.codex/`;
- unrelated application or library code during planning;
- unrelated user changes;
- `docs/roadmap-calendar-forecast.md`.

## Planning validation and commit gate

At minimum run:

```bash
git diff --check
git status --short
```

Also validate manually or through discovered repository tooling:

- Markdown structure and internal links;
- prompt numbering;
- manifest order;
- prerequisite graph;
- Profile and Template paths;
- authoritative document paths;
- commit-message agreement between Roadmap, master prompt, and child prompts;
- accepted versus deferred scope;
- unchanged `next` status during planning;
- every generated sprint prompt is inside the one current sprint directory;
- no generated child or master prompt exists directly under
  `docs/codex/prompts/`;
- the generated master and final review prompt name the same verified
  immediately preceding suite directory for conditional retirement, or the
  same `already_retired` evidence;
- absence of unrelated changes.

Do not run production Rust tests for a documentation-only planning change
unless repository evidence requires them.

If validation fails, apply only the smallest relevant fix, rerun the failed
check, and rerun the planning validation cycle.

If commit mode is authorized:

- explicitly enumerate planning-owned paths;
- stage only those paths;
- never use `git add .`;
- commit the complete immutable planning baseline with:

  `Plan Sprint <N> <english sprint subject>`

- verify the commit and clean task-owned state before execution.

If commit mode is not authorized, do not stage or commit. Stop before execution
when Task 01 requires a committed planning baseline.

## Phase 3 — Sequential sprint execution

After the planning baseline is committed, read the generated master prompt and
execute it in the same run.

For every task in dependency order:

1. Record task start time, current `HEAD`, `git status --short`, and available
   runtime token telemetry.
2. Refresh relevant repository evidence and enforce the committed prerequisite.
3. Read the task prompt and all selected framework modules completely.
4. Print the task Change Contract before edits.
5. Implement only the task-owned outcome.
6. Preserve unrelated and pre-existing work.
7. Run focused validation and every package or full-workspace check required by
   `docs/codex/core/validation.md`.
8. Verify acceptance criteria, exclusions, task diff, and repository state.
9. Treat zero matched test filters as missing evidence, not a pass.
10. If commit mode is authorized:
    - stage only explicitly enumerated task-owned paths;
    - create exactly one logical commit using the task's manifest message;
    - verify the resulting commit hash and committed paths.
11. Record task end time, elapsed wall-clock time, status, exact validation
    results, commit hash, ending `HEAD`, final task Git status, and available
    token telemetry.
12. Continue only when the next committed prerequisite is satisfied and no
    uncommitted task-created change remains.

Valid task statuses are:

- `completed`;
- `already_complete`;
- `blocked`;
- `failed`;
- `not_started`.

Use `already_complete` only when current committed evidence and successful
required validation prove every acceptance criterion. Record the proving commit
or baseline. Do not create an empty commit.

## Failure behavior

Stop immediately after the first prerequisite, implementation, validation,
staging, commit, or review failure.

On failure:

- preserve the failing task's evidence and diff;
- do not start dependent tasks;
- do not repair unrelated changes;
- do not create a partial or knowingly failing commit;
- report the exact failed command and result;
- report the blocker, stopping task, affected paths, recoverability, and
  required next action;
- leave all later tasks `not_started`.

## Integration-review gate

Run the final integration-review task only after all preceding tasks are
committed or proven `already_complete` and all required validation succeeds.

The review must:

- inspect the exact planning and task commit range;
- verify every accepted criterion and exclusion;
- rerun the required focused and full validation matrix;
- create only explicitly authorized review artifacts;
- issue `pass`, `pass with non-blocking follow-ups`, or `blocked`;
- avoid silently fixing findings in the review commit;
- retire the verified immediately preceding sprint prompt suite only through
  the post-decision procedure below.

Only `pass` or `pass with non-blocking follow-ups`, plus successful required
validation, may:

- transition the sprint to `completed`;
- make the next sprint eligible for planning;
- authorize the final review commit.

A blocked review leaves the sprint incomplete.

## Previous sprint prompt-suite retirement gate

Retire prompt files for Sprint `<N-1>` only as the final bounded action of the
Sprint `<N>` integration-review task and only when all of these conditions are
true:

1. The current Sprint `<N>` review has already issued `pass` or `pass with
   non-blocking follow-ups` from complete executed evidence.
2. Every required focused and full validation command has succeeded.
3. The Roadmap transition marks Sprint `<N>` `completed` and makes Sprint
   `<N+1>` eligible.
4. Commit mode is authorized, so the review artifact, state transition, and
   prompt retirement can be committed atomically as the final review task.
5. Exactly one immediately preceding suite identity was verified during
   planning. Its expected path is:

   `docs/codex/prompts/sprint-<N-1>-<verified-slug>/`

   The suite either still exists at that exact path or is proven
   `already_retired` from committed repository state.
6. When the suite exists, every deletion target is a tracked prompt file inside
   that exact directory.

When the gate passes and the suite exists:

1. Re-enumerate the directory and compare it with the tracked file list.
2. Delete every tracked prompt file in that exact previous-suite directory.
3. Do not delete the current sprint suite, `run-next-sprint.md`, an older
   non-adjacent suite, an untracked file, or any path outside the verified
   previous-suite directory.
4. Do not use `rm -rf`, recursive globs, `git clean`, or another broad or
   destructive command. Delete explicitly enumerated files through the normal
   file-editing mechanism and stage each deleted path explicitly.
5. Verify that the current sprint suite remains complete, the previous tracked
   suite is absent, no Markdown link introduced or retained by the current
   sprint depends on a deleted file, and `git diff --check` succeeds.
6. Include the review artifact, Roadmap/current-state transition, and exact
   previous-suite deletions in the single final integration-review task commit.

If the previous suite is already absent, record it as `already_retired`, retain
the ordinary final review commit, and do not create an empty cleanup commit. If
the directory is ambiguous, contains an untracked file that would be
endangered, includes a path outside the verified boundary, or differs from the
planned tracked inventory, stop before deletion and report the cleanup blocker.
Never retire prompts after a blocked or failed review, and never leave
destructive prompt cleanup as an uncommitted change when commit mode is
unavailable.

## Validation policy

Use focused checks during each task.

Run the canonical full workspace validation whenever required by
`docs/codex/core/validation.md`, including for production Rust, Cargo manifest,
public API, graph model, parser, or graph-emission changes:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Never claim a command passed unless it completed successfully.

## Timing and token accounting

Record a wall-clock timestamp at:

- sprint start;
- each task start;
- each task end;
- sprint end.

Calculate elapsed time for every task and the whole sprint from recorded
timestamps. Do not infer task duration from commit timestamps.

At the same boundaries, query token or goal telemetry only when the active
runtime exposes it.

Report:

- per-task input, output, and total tokens when available;
- total sprint token usage when available;
- `недоступно` for unavailable dimensions.

Do not estimate or back-calculate token counts. If only total sprint usage is
available, report only the total and explicitly state that task-level breakdown
is unavailable.

## Required final report

Always produce a final Russian report, including on failure.

Report:

- sprint number and name;
- sprint objective;
- starting and ending sprint state;
- starting and ending `HEAD`;
- sprint start, end, and total elapsed time;
- initial and final `git status --short`;
- changed and preserved paths;
- whether `.codex/` remained untouched;
- whether anything remains staged or uncommitted;
- integration-review decision, when reached;
- current sprint prompt-suite directory;
- previous sprint prompt-suite retirement result (`retired`,
  `already_retired`, `not_reached`, or `blocked`) and every deleted path;
- next sprint eligibility.

Provide an ordered task table with:

| Order | Task | Status | Started | Finished | Elapsed | Commit | Validation | Tokens |
|---:|---|---|---|---|---|---|---|---|

For every commit, include its short hash and exact subject.

Also report:

- planning and template readiness decision;
- architecture decision or confirmation that existing architecture was
  sufficient;
- already-complete evidence;
- exact failed commands and errors;
- blocker and stopping point;
- tasks left `not_started`;
- per-task token usage when available;
- total sprint token usage when available;
- total sprint elapsed time;
- final repository state.

If the sprint stops before planning because required data is missing, use the
mandatory `SPRINT_BLOCKED_MISSING_DATA` report instead of claiming that a sprint
plan or task list was completed.

## Completion criteria

The run is complete only when one of these terminal conditions is reached:

1. Successful completion:
   - readiness and data gates passed;
   - framework readiness was decided;
   - the ordered sprint plan and prompt suite were created and validated;
   - the planning baseline was committed when authorized;
   - every task was committed or proven `already_complete`;
   - the integration review issued a non-blocking decision;
   - the sprint state and documentation match live evidence;
   - the immediately preceding sprint prompt suite was safely retired or
     proven `already_retired` under the retirement gate;
   - required validation passed;
   - the final report was produced.

2. Safe stop:
   - required data or architecture evidence is missing;
   - a prerequisite is not satisfied;
   - implementation or validation failed;
   - commit authorization is insufficient for the next gate;
   - the integration review is blocked;
   - the exact blocker and repository state were reported.

Do not continue past a safe-stop condition.
