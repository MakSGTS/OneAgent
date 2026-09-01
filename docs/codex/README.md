# Codex Framework

The Codex Framework stores stable reusable instructions for OneAgent tasks.
Its goal is to keep executable prompts small without weakening repository
safety, accepted architecture, validation, or reporting.

`.codex/`, global Codex configuration, and local runtime state are outside this
framework and must not be modified by ordinary repository tasks.

## Layer model

```text
Current user instruction
    -> applicable AGENTS.md
        -> accepted ADRs and architecture
            -> Profile: composes Core + Workflows
                -> Base Template + specialized Template
                    -> child prompt: concrete task + Context Manifest
```

| Layer | Owner |
|---|---|
| `core/` | Permanent safety, investigation, context, validation, and report rules |
| `workflows/` | Reusable execution behavior |
| `profiles/` | Standard Core and Workflow compositions plus family invariants |
| `templates/task-prompt.md` | Common Prompt Contract v2 child structure |
| specialized `templates/` | Task-family fields and evidence additions |
| `prompts/` | Concrete scope, authorities, prerequisites, acceptance, and deltas |

State each instruction once in its canonical owner. Profiles compose rather
than copy modules. Specialized Templates add only family-specific requirements.
Child prompts do not copy permanent safety, validation, reporting, review, or
context rules.

## Precedence

From highest to lowest authority:

1. explicit current user instruction;
2. applicable `AGENTS.md`;
3. accepted ADRs and authoritative architecture documents;
4. selected Core, Workflow, Profile, and Template modules;
5. the current child prompt;
6. local implementation conventions confirmed from code and tests.

The child prompt narrows concrete scope and supplies task-specific evidence. It
must not weaken a higher-priority rule. Report a real contradiction instead of
guessing through it.

## Prompt Contract v2

Every newly generated executable child prompt must:

- use the front matter and sections in
  `docs/codex/templates/task-prompt.md`;
- follow `docs/codex/core/context-management.md`;
- select the smallest sufficient Profile and specialized Template;
- start in a guaranteed fresh execution context;
- contain an exact bounded Context Manifest;
- reference accepted decisions instead of restating them;
- state only task-specific validation additions; and
- pass `scripts/validate-codex-prompts.sh`.

Historical prompts are immutable execution evidence and remain legacy
compatible. Do not rewrite them solely to adopt a later contract version.

## Context policy

Prompt Contract v2 uses these default context-window allocations:

| Allocation | Contract |
|---|---:|
| Static instructions | maximum 15% |
| Initial authorities | maximum 20% |
| Normal pre-work target | maximum 35% |
| Pre-work hard stop | 50% |
| Source, diff, tool, and validation working set | minimum 35% |
| Final response and safety reserve | minimum 15% |

The Context Manifest separates `Must read`, `Lookup on demand`, and material
excluded from initial context. Whole large Roadmap, Architecture,
semantic-model, generated, fixture, or log inputs are forbidden when a section,
symbol, diff, range, or bounded query is sufficient.

Runtime telemetry is authoritative. A conservative estimate may be used only
for preflight admission and must never be reported as actual usage. See
`docs/adr/0062-codex-prompt-context-management.md` for the decision and
`docs/codex/context-management-guide.md` for the portable pattern.

## Routing

Use the narrowest matching pair:

| Task family | Profile | Specialized Template |
|---|---|---|
| General implementation | `implementation.md` | `implementation-task.md` |
| Investigation | `investigation.md` | `investigation-task.md` |
| Architecture | `architecture.md` | `architecture-task.md` |
| Review | `review.md` | `review-task.md` |
| Graph model or emission | `graph-implementation.md` | `graph-model-task.md` or `graph-emission-task.md` |
| Parser | `parser-implementation.md` | `parser-task.md` |
| Source adapter | `source-adapter-implementation.md` | `source-adapter-task.md` |
| Semantic Index | `semantic-index-implementation.md` | `semantic-index-task.md` |
| Context Engine | `context-engine-implementation.md` | `context-engine-task.md` |
| Persistent State | `persistent-state-implementation.md` | `persistent-state-task.md` |
| Runtime Service | `runtime-service-implementation.md` | `runtime-service-task.md` |
| LLM Provider | `llm-provider-implementation.md` | `llm-provider-task.md` |
| AI Tool Policy | `ai-tool-policy-implementation.md` | `ai-tool-policy-task.md` |
| MCP Protocol | `mcp-protocol-implementation.md` | `mcp-protocol-task.md` |
| IDE Extension | `ide-extension-implementation.md` | `ide-extension-task.md` |
| Diagnostics Engine | `diagnostics-engine-implementation.md` | `diagnostics-engine-task.md` |
| Rules Engine | `rules-engine-implementation.md` | `rules-engine-task.md` |
| Git Change Adapter | `git-change-adapter-implementation.md` | `git-change-adapter-task.md` |
| Sprint planning | `architecture.md` | `sprint-planning-task.md` |
| Sprint dispatch | child-selected | `sprint-execution-loop.md` |

All Profile paths are relative to `docs/codex/profiles/`; all specialized
Template paths are relative to `docs/codex/templates/`.

Do not add a Profile for one task. Add one only when repeated tasks need a
distinct module composition. Do not add a Workflow for concrete scope; a
Workflow must describe reusable execution behavior.

## Canonical task rules

1. Keep one coherent owned outcome per task.
2. Declare every prerequisite or explicitly state that none exists.
3. Recheck mutable `HEAD`, status, counts, and repository evidence live.
4. Use real fixtures and source evidence; do not invent formats or APIs.
5. Keep accepted architecture fixed during implementation.
6. Task-specific exclusions and validation add deltas; they do not copy Core.
7. Suggested commit messages are metadata, not authorization.
8. Resolve staging, commit, branch, merge, review, and push behavior from the
   current user instruction and applicable `AGENTS.md`.
9. Treat zero matched tests as missing evidence.
10. Preserve unsupported and deferred scope unless explicitly accepted.

## Sprint lifecycle

`docs/codex/prompts/run-next-sprint.md` plans a sprint and creates a Prompt
Contract v2 suite. The suite master is a dispatcher and ledger. Planning, every
child task, and independent review use separate fresh contexts. If fresh
context is unavailable, stop at the boundary and emit the exact next prompt
path and prerequisite.

The sequential workflow owns task ordering and failure behavior. The Review
workflow owns independent reviewer behavior. Repository `AGENTS.md` owns branch,
merge, remediation, and push rules. Stored prompt text does not authorize a
commit by itself.

## Validation and logs

`docs/codex/core/validation.md` is the single canonical validation matrix.
Templates and child prompts state only additions.

Keep conversation output bounded. Retain materially large complete logs under
`local-artifacts/codex-runs/<run-id>/` and report only command, exit status,
meaningful count, concise failure excerpt, and artifact path. Never persist
secrets or unrelated source content.

## Extending or adopting the framework

Before changing the framework:

1. identify the concrete repeated gap;
2. update only its canonical layer;
3. avoid adding another copy of an existing rule;
4. validate routing, links, Prompt Contract v2, and representative prompts;
5. compare correctness and token usage before and after the change.

For another project, start with
`docs/codex/context-management-guide.md`, then bind project-specific Profiles,
Templates, validation, artifact paths, and Git workflow.
