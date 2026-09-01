# Portable Prompt Context Management Guide

This guide extracts OneAgent's Prompt Contract v2 into a project-neutral
pattern. Use it when a coding-agent workflow has reusable instructions,
multi-step execution, large authority documents, or tool-heavy validation.

## Goals

- keep instructions stable without repeating them in every task;
- admit only evidence needed for the current decision;
- reserve context for source, diffs, tools, validation, and the final answer;
- make every long task restartable from a durable repository state; and
- validate generated prompts before execution.

## Minimal architecture

```text
Repository instructions
    -> Core rules
        -> Profiles compose Core + Workflows
            -> Specialized Templates add task-family fields
                -> Child prompt supplies concrete scope and Context Manifest
```

Keep one owner for each rule. A lower layer references its owner and supplies
only a concrete refinement or delta.

## Budget policy

Treat the model context window as an allocatable resource:

| Allocation | Default |
|---|---:|
| Static instructions | at most 15% |
| Initial authorities | at most 20% |
| Pre-work warning | above 35% |
| Pre-work hard stop | 50% |
| Working set | at least 35% |
| Response and safety reserve | at least 15% |

Use runtime token telemetry when available. Otherwise use a conservative,
clearly labelled estimate only to decide whether material may be loaded. Never
report estimated usage as measured usage.

If pre-work material crosses the warning threshold, narrow whole documents to
sections, symbols, diffs, or bounded searches. If it crosses the hard stop,
split the task or stop. Do not depend on automatic truncation.

## Context Manifest

Every executable child prompt should declare:

```markdown
## Context manifest

### Must read

- `<project-docs>/roadmap.md` — sections: current milestone and its
  prerequisites.
- `src/domain.rs` — symbols: `Request`, `Result`, and their consumers.

### Lookup on demand

- `<project-docs>/history.md` — trigger: only when current decisions conflict.
- integration logs — trigger: only after a focused validation failure.

### Excluded from initial context

- unrelated historical milestones;
- complete generated files and successful full test logs.

### Preflight

- effective context window: runtime telemetry or `unknown`;
- admission basis: telemetry or conservative estimate;
- static and authority allocation: values plus pass/warn/block decision.
```

For large documents, a bare path is not enough. Name the exact heading, symbol,
line range, diff range, or search query and explain why it is required.

## Fresh-context execution

Use one fresh execution context per coherent task. Pass only:

- the child prompt;
- exact committed prerequisite;
- current Git status;
- selected Core/Profile/Template/Workflow modules; and
- admitted `Must read` material.

Persist a compact ledger in the repository or orchestration layer:

```text
task id | status | starting head | ending head | validation summary | commit
```

Do not pass previous implementation transcripts to the next task. If a runtime
cannot create a guaranteed fresh context, stop at the boundary and provide a
copy-ready continuation instruction.

## Tool-output policy

Return small outputs directly. For large outputs, retain the complete log in a
project-owned ignored directory and put only this summary in the conversation:

```text
command | exit | meaningful count | concise failure excerpt | artifact path
```

Never retain secrets, credentials, raw sensitive payloads, or unnecessary
source content in either location.

## Prompt Contract v2 front matter

OneAgent uses flat YAML fields so a dependency-free shell linter can validate
them:

```yaml
---
prompt_contract: v2
task_kind: implementation
profile: <agent-root>/profiles/implementation.md
template: <agent-root>/templates/implementation-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---
```

Adapt paths and task kinds to the project, but keep the percentages coherent and
machine checked.

## Adoption checklist

1. Inventory instruction layers and exact byte sizes.
2. Identify repeated rules and assign one canonical owner to each.
3. Add a base child-prompt contract and Context Manifest.
4. Add context-budget front matter and a preflight admission gate.
5. Move whole-document authorities to section- or symbol-level selectors.
6. Split long workflows into fresh-context tasks with durable prerequisites.
7. Centralize the full validation matrix; prompts add only deltas.
8. Store large logs outside the conversation and retain compact summaries.
9. Add a linter to CI and validate only the new contract version during
   migration.
10. Compare representative tasks before and after migration for correctness,
    total tokens, tool calls, retries, and completeness.

## Migration strategy

Do not rewrite historical prompts. Introduce a versioned contract and require it
only for newly generated prompts. First validate one example and one real task,
then migrate the generator or bootstrap. Remove duplicated prose in small groups
and rerun representative validations after each group.

## Project-specific decisions

This guide does not choose:

- a model or exact context-window size;
- a tokenizer or telemetry API;
- orchestration tools or subagent permissions;
- repository branch, review, or release workflow;
- validation commands; or
- locations for ignored artifacts.

Each adopting project must bind those choices through its own authoritative
instructions.
