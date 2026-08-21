# Codex Framework

The Codex Framework stores stable, reusable instructions for OneAgent Codex
tasks inside the repository. It exists to keep future task prompts short and
declarative without weakening repository safety, architecture discipline,
validation quality, or reporting standards.

`.codex/` is not part of this framework. Ordinary repository tasks must not
modify `.codex/`, global Codex configuration, or local Codex runtime state.

## Why this framework exists

OneAgent tasks repeatedly need the same safety rules, investigation steps,
Change Contract, validation discipline, final reporting, and workflow-specific
execution rules. Those permanent rules belong in version-controlled framework
modules, not in every prompt.

A normal task prompt should contain only task-specific information: selected
profile, selected template, authoritative task documents, scope, exclusions,
acceptance criteria, and task-specific validation additions.

## Layer responsibilities

### Core modules

`docs/codex/core/` contains permanent rules shared by most tasks:

- repository safety;
- repository investigation;
- Change Contract;
- validation;
- final report.

Core modules must remain independent from profiles, templates, and individual
task prompts.

### Workflow modules

`docs/codex/workflows/` defines reusable execution behavior for recurring
technical workflows:

- architecture;
- implementation;
- graph model;
- graph emission;
- semantic index;
- parser;
- persistent state;
- runtime service;
- source adapter;
- review.

Workflow modules describe how a task is executed. They must not depend on
individual profiles or task prompts.

### Profiles

`docs/codex/profiles/` defines standard combinations of Core and Workflow
modules for recurring task families. Profiles prevent ordinary prompts from
listing the same framework modules repeatedly.

A profile states its purpose, required Core modules, required Workflow modules,
and task-family expectations that are not owned by Core or Workflow modules.

### Templates

`docs/codex/templates/` defines reusable task and output contracts. Templates
specify recurring structure such as required task-specific sections, additional
acceptance requirements, additional report sections, and additional validation.

Templates must not duplicate repository safety, generic investigation, generic
validation, generic final-report rules, or generic implementation workflow.

### Task prompts

Task prompts are the only layer that contains concrete task-specific scope and
acceptance criteria. Future prompts should be concise and should reference the
smallest sufficient profile and template combination.

A stored task prompt is not a live repository baseline. After its task is
completed, treat its `HEAD`, counts, status, and implementation inventory as
historical execution context. Prepare new tasks from the current repository,
accepted architecture, and reusable framework modules instead of cloning an old
prompt unchanged.

## Directory structure

```text
docs/codex/
  README.md
  core/
    change-contract.md
    final-report.md
    repository-investigation.md
    repository-safety.md
    validation.md
  workflows/
    architecture.md
    graph-emission.md
    graph-model.md
    implementation.md
    parser.md
    persistent-state.md
    review.md
    runtime-service.md
    semantic-index.md
    sequential-sprint-execution.md
    source-adapter.md
  profiles/
    architecture.md
    graph-implementation.md
    implementation.md
    investigation.md
    parser-implementation.md
    persistent-state-implementation.md
    review.md
    runtime-service-implementation.md
    semantic-index-implementation.md
    source-adapter-implementation.md
  templates/
    architecture-task.md
    graph-emission-task.md
    graph-model-task.md
    implementation-task.md
    investigation-task.md
    parser-task.md
    persistent-state-task.md
    review-task.md
    runtime-service-task.md
    semantic-index-task.md
    sprint-execution-loop.md
    sprint-planning-task.md
    source-adapter-task.md
```

## Dependency model

```text
Task Prompt
    |
    +--> Profile
    |      |
    |      +--> Core modules
    |      +--> Workflow modules
    |
    +--> Template
    |
    +--> Authoritative ADRs / task-specific docs
```

Lower-level framework modules must not depend on individual task prompts. Core
modules must remain independent from Profiles and Templates. Workflows must not
depend on individual Profiles. Profiles may compose Core and Workflow modules.
Templates may reference Profiles, Core, and Workflows where useful, but must not
duplicate their content.

## Precedence model

From highest to lowest authority:

1. explicit current user instruction;
2. applicable `AGENTS.md`;
3. accepted ADRs and authoritative architecture documents;
4. selected Codex Framework Profile, Core, Workflow, and Template modules;
5. current task prompt;
6. local implementation conventions inferred from nearby code and tests.

The task prompt selects framework modules and supplies task-specific scope,
exclusions, acceptance criteria, and validation additions. The task prompt must
not silently override repository safety rules. Task-specific acceptance criteria
may refine the framework, but contradictions must be reported instead of guessed
through.

The Codex Framework does not override applicable `AGENTS.md` or accepted ADRs.

## Canonical prompt rules

1. Future prompts must be concise.
2. Permanent rules belong in Core modules.
3. Repeated execution behavior belongs in Workflow modules.
4. Repeated module combinations belong in Profiles.
5. Repeated task/output structure belongs in Templates.
6. Concrete task details belong only in the Task Prompt.
7. Accepted ADRs are referenced, not restated.
8. Task prompts must not duplicate framework text merely to be self-contained.
9. Prompts are written in English.
10. All user-visible Codex reports are written in Russian.
11. A task should reference the smallest sufficient profile/template combination.
12. Task-specific validation supplements framework validation; it does not
    restate it.
13. Task-specific exclusions supplement repository safety rules; they do not
    copy them.
14. If a reusable rule appears repeatedly in task prompts, move it into the
    appropriate framework layer instead of continuing duplication.
15. A suggested commit message is planning metadata only. It does not authorize
    staging or committing. Include a commit action only when the current user
    instruction explicitly authorizes it consistently with Repository Safety.
16. Historical `HEAD` values, coverage counts, and repository baselines are
    context, not live evidence. Recheck every mutable fact before using it to
    select or execute a new task.
17. Every task prompt declares its prerequisites or explicitly states that it
    has none. Enforce the required gate before implementation.
18. Stored prompt text does not permanently authorize staging or committing.
    Resolve commit authorization from the current user instruction that launches
    execution.

## Choosing a profile

- Use `docs/codex/profiles/implementation.md` for accepted implementation work
  that does not need graph- or parser-specific behavior.
- Use `docs/codex/profiles/graph-implementation.md` for graph model or graph
  emission implementation tasks.
- Use `docs/codex/profiles/semantic-index-implementation.md` for repeated
  complete-snapshot or incremental Semantic Index implementation tasks.
- Use `docs/codex/profiles/parser-implementation.md` for real source parser
  tasks.
- Use `docs/codex/profiles/source-adapter-implementation.md` for multi-artifact
  discovery, assembly, mapping, or cross-adapter conformance implementation.
- Use `docs/codex/profiles/persistent-state-implementation.md` for persisted
  schemas, storage, invalidation, compatibility, migration, corruption, or
  recovery implementation.
- Use `docs/codex/profiles/runtime-service-implementation.md` for long-running
  service lifecycle, Runtime-owned concurrency, transport adapters, health, or
  supported client implementation.
- Use `docs/codex/profiles/architecture.md` for architecture-only work and ADRs.
- Use `docs/codex/profiles/investigation.md` for read-only evidence gathering.
- Use `docs/codex/profiles/review.md` for review-only tasks.

Do not create or select a broader profile when a narrower profile is sufficient.
Do not introduce a new profile unless repeated tasks need a distinct reusable
module composition.

## Choosing a template

- Use `docs/codex/templates/implementation-task.md` for general implementation
  task contracts.
- Use `docs/codex/templates/investigation-task.md` for read-only evidence
  gathering.
- Use `docs/codex/templates/graph-model-task.md` for public graph model changes.
- Use `docs/codex/templates/graph-emission-task.md` for semantic graph producer
  emission.
- Use `docs/codex/templates/semantic-index-task.md` for complete-snapshot or
  incremental Semantic Index work.
- Use `docs/codex/templates/parser-task.md` for parser implementation.
- Use `docs/codex/templates/source-adapter-task.md` for multi-artifact source
  adapter ingestion or cross-adapter conformance.
- Use `docs/codex/templates/persistent-state-task.md` for persisted schema,
  storage, invalidation, compatibility, migration, corruption-recovery, or
  integration work.
- Use `docs/codex/templates/runtime-service-task.md` for Runtime service,
  lifecycle, health, transport-adapter, observability, or supported client work.
- Use `docs/codex/templates/architecture-task.md` for architecture output.
- Use `docs/codex/templates/review-task.md` for review output.
- Use `docs/codex/templates/sprint-planning-task.md` for a sprint kickoff,
  readiness audit, and ordered Roadmap execution plan.
- Use `docs/codex/templates/sprint-execution-loop.md` for a master prompt that
  executes an accepted sprint plan in dependency order.

A template defines task structure. It is not a complete long-form prompt.

## Task family differences

Architecture tasks decide or document contracts and prerequisites. They do not
implement production behavior unless explicitly scoped.

Investigation tasks gather evidence and may stop with confirmed unknowns. They
must not invent source formats or architecture.

Implementation tasks apply accepted architecture in code or documentation. They
must not reselect architecture during implementation.

Parser tasks require real source evidence and define source-format behavior.
They do not emit graph facts unless graph emission is explicitly included.

Source adapter tasks orchestrate deterministic project discovery, artifact
assembly, parsing, source-independent mapping, and cross-adapter conformance.
They preserve parser separation and do not create an adapter-specific semantic
authority.

Persistent state tasks implement accepted persisted representations, storage,
invalidation, compatibility, migration, corruption handling, and recovery.
They keep canonical semantic state authoritative and do not select unresolved
formats, identities, filesystem behavior, or Runtime integration architecture.

Runtime service tasks implement accepted long-running lifecycle, structured
task ownership, cancellation, shutdown, health, transport, or supported client
contracts. They keep composition, service execution, adapters, and clients in
their accepted layers and do not select unresolved concurrency architecture.

Graph model tasks change public graph representation or graph APIs. They must
preserve deterministic identity and define validation/query impact.

Graph emission tasks connect production source facts to semantic graph nodes or
edges. They must preserve provenance, determinism, and Coverage evidence.

Semantic Index tasks build deterministic derived views without creating a
second semantic authority. They must define lifecycle, staleness, compatibility,
ordering, and equivalence with canonical query or full-rebuild behavior.

Review tasks inspect existing work. They may create explicitly authorized review
artifacts and Roadmap state transitions, but they do not modify implementation
files unless the user changes the task from review to implementation.

Sprint planning tasks audit the live baseline, decide whether the reusable
framework is ready, and record an ordered executable plan without implementing
production behavior.

Sprint execution-loop prompts orchestrate accepted child prompts. They do not
replace the child task's selected Profile or Template, and they obtain commit
authorization only from the current instruction that launches the loop.

## Canonical short task prompt

```text
Continue OneAgent development.

Reporting:
- Prompt: English.
- User-visible reports: Russian.

Profile:
docs/codex/profiles/graph-implementation.md

Template:
docs/codex/templates/graph-emission-task.md

Authoritative ADRs:
- docs/adr/NNNN-example-semantics.md

Prerequisites / Required gate:
- The accepted ADR is committed and the working tree contains no conflicting
  task-created change.

Task:
Implement one accepted semantic edge production slice.

Scope:
Included:
- EDT producer path for the confirmed source artifact.
- Graph emission, provenance, tests, and Coverage evidence.

Excluded:
- New graph model concepts.
- Unrelated parser families.

Acceptance Criteria:
- The edge is emitted deterministically.
- Provenance is attached.
- Repeated builds are identical.
- Coverage transitions only after complete evidence.

Task-specific Validation:
- cargo test -p oneagent-edt <focused_filter>
- cargo test -p oneagent-graph <focused_filter>
```

Permanent safety, investigation, Change Contract, validation, and final-report
rules are inherited through the selected Profile and Template and must not be
copied into the prompt.

## Extending the framework

### Adding a profile

Add a profile only when repeated tasks need a distinct composition of Core and
Workflow modules or task-family expectations. A profile must state:

- purpose;
- required Core modules;
- required Workflow modules;
- task-family expectations.

Do not copy full Core or Workflow content into a profile.

### Changing a template

Templates should remain compact contracts. When changing a template:

- keep normative task structure explicit;
- reference profiles instead of listing all Core and Workflow modules;
- avoid embedding complete example prompts unless the file is explicitly an
  example;
- move repeated execution rules into Workflows;
- move repeated module combinations into Profiles;
- move permanent safety and validation rules into Core modules.

### Adding a workflow

Add a workflow only when repeated tasks need a distinct reusable execution model.
Keep workflow modules focused on how the task is executed, not the concrete task
scope.

## Task-size guidance

A task should usually produce one coherent outcome such as:

- one ADR for one unresolved architectural capability;
- one graph-model prerequisite;
- one snapshot-index or incremental-index implementation slice;
- one parser for one source artifact family;
- one source-adapter discovery, assembly, mapping, or conformance slice;
- one resolver/emitter production slice;
- one review of one completed implementation;
- one documentation synchronization task.

Tasks that are too large combine unrelated or sequentially dependent outcomes,
such as architecture selection plus parser plus resolver plus graph emission plus
Coverage transition when major prerequisites do not exist, multiple unrelated
semantic capabilities, or broad refactoring plus a new feature.

Tasks that are too small split work below a coherent review boundary, such as
adding one enum variant without its identity/tests/integration when those belong
to the same model task, adding one isolated test, modifying one file when the
logical outcome requires several, or splitting resolver and edge insertion when
they form one small production slice.

Optimize task boundaries for coherent outcome, reviewability, bounded context,
independent validation, and minimal cross-task temporary states.

## Authoritative decisions

Accepted ADRs and architecture documents define semantics, direction, identity,
endpoint compatibility, and scope. Implementation tasks must treat those
decisions as fixed.

Reopen an accepted decision only when repository evidence proves the accepted
contract is impossible, contradictory, or incompatible with the actual source
format. In that case Codex must stop the affected implementation, describe the
blocker, identify the authoritative decision that cannot be implemented, avoid
inventing an alternative architecture, and leave unrelated work unchanged.

## Maintenance

Keep modules focused and reasonably small. Do not copy entire task prompts into
framework files. Update validation commands when repository CI changes. Move
repeated prompt text into the proper framework layer instead of duplicating it in
future prompts.
