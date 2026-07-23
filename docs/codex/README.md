# Codex Framework

The Codex Framework stores stable, reusable instructions for Codex tasks in the
repository. It exists to reduce oversized task prompts without weakening safety,
architecture discipline, validation quality, or reporting standards.

`.codex/` is not part of this framework. Ordinary repository tasks must not
modify `.codex/`, global Codex configuration, or local Codex runtime state.

## Design principles

- Keep permanent rules version-controlled.
- Keep future prompts short, specialized, and explicit.
- Select only the modules needed for the task.
- Treat accepted ADRs and architecture documents as authoritative.
- Split work into coherent outcomes, not arbitrary files or functions.
- Avoid duplicating the same rule across many modules.
- Prefer precise blockers over invented architecture.

## Directory overview

- `core/` contains task rules that are commonly reused across task types.
- `workflows/` contains specialized rules for architecture, implementation,
  graph model, parser, graph emission, and review tasks.
- `templates/` contains concise prompt templates that reference modules instead
  of repeating their contents.

## Precedence model

From highest to lowest authority:

1. explicit current user instruction;
2. applicable `AGENTS.md`;
3. accepted ADRs and authoritative architecture documents;
4. selected Codex Framework core and workflow modules;
5. current task prompt;
6. local implementation conventions inferred from nearby code and tests.

The task prompt selects framework modules and supplies task-specific scope,
exclusions, acceptance criteria, and validation additions. The task prompt must
not silently override repository safety rules. Task-specific acceptance criteria
may refine the framework, but contradictions must be reported instead of guessed
through.

The Codex Framework does not override applicable `AGENTS.md` or accepted ADRs.

## Selecting modules

Every future task prompt should explicitly list the framework modules Codex must
read. Core safety rules are appropriate for almost all tasks. Specialized
workflow modules should be selected by task type.

Example:

```text
Continue OneAgent.

Read and follow:
- docs/codex/core/repository-safety.md
- docs/codex/core/repository-investigation.md
- docs/codex/core/change-contract.md
- docs/codex/core/validation.md
- docs/codex/core/final-report.md
- docs/codex/workflows/graph-model.md

Authoritative architecture:
- docs/adr/NNNN-example.md

Task:
<task>

Scope:
<scope>

Do not:
<exclusions>

Acceptance criteria:
<criteria>

The final user-visible report must be written in Russian.
```

## Task-size guidance

A task should usually produce one coherent outcome such as:

- one ADR for one unresolved architectural capability;
- one graph-model prerequisite;
- one parser for one source artifact family;
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
framework files. Update validation commands when repository CI changes. Add new
workflow modules only when repeated tasks need a distinct reusable process.
