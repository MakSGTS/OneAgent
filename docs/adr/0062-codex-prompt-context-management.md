# ADR-0062: Codex Prompt Context Management

## Status

Accepted.

## Context

The repository Codex Framework successfully separates permanent safety,
workflow, profile, template, and task-specific instructions. Its execution
model nevertheless accumulated the same rules across several layers and loaded
large authorities into one long-running sprint context.

The 2026-09-01 audit established this baseline before remediation:

| Artifact | Bytes |
|---|---:|
| `docs/codex/README.md` | 22,985 |
| `docs/codex/prompts/run-next-sprint.md` | 29,366 |
| Core modules | 7,040 |
| Profiles | 24,500 |
| Templates | 48,795 |
| Workflows | 67,511 |
| `docs/Roadmap.md` | 606,022 |
| `docs/Architecture.md` | 73,035 |
| `docs/architecture/semantic-model-2.md` | 143,406 |

The earlier bootstrap required planning and all child tasks to execute in one
conversation, required complete rereads of selected framework modules, named
whole large documents without section selectors, duplicated validation and
review rules, and collected token telemetry only after work was performed.
Those properties made instruction loss after context compaction more likely and
left too little space for source, diffs, test evidence, and the final report.

[Official OpenAI model guidance](https://developers.openai.com/api/docs/guides/latest-model#favor-leaner-prompts)
recommends lean prompts, stating each instruction once, limiting exposed
material to what the task needs, and tracking context both initially and as a
run grows. The repository needs a model-independent contract that applies those
principles without relying on a particular context-window size or tokenizer.

## Decision

### Prompt Contract v2

Every newly generated child task prompt uses `prompt_contract: v2` front
matter and the base contract in
`docs/codex/templates/task-prompt.md`. Historical prompts remain valid legacy
evidence and are not rewritten.

The front matter identifies the task kind, selected Profile and Template,
fresh-context requirement, and context-budget policy. A repository linter
checks the machine-readable contract and required headings before a new suite
can be accepted.

### Context budget

Budget percentages are allocation limits over the effective model context
window, not billing limits and not claims of actual token usage:

| Allocation | Contract |
|---|---:|
| Static instructions | maximum 15% |
| Initial authoritative material | maximum 20% |
| Pre-work context | warning above 35%; hard stop at 50% |
| Source, diffs, tools, and validation working set | minimum 35% |
| Final response and safety reserve | minimum 15% |

The unallocated space between the normal 35% pre-work target and the 50% hard
stop is flexible working margin. Runtime token telemetry is authoritative when
available. A conservative character-based estimate may be used only for
preflight admission and must be labelled as an estimate; final reports never
present it as actual token usage.

When the mandatory initial material exceeds the hard stop, the task must narrow
the manifest, split the task, or stop with a context-budget blocker. It must not
silently continue and rely on truncation or compaction.

### Context Manifest

Every child prompt contains one `Context manifest` with:

- `Must read`: exact files plus sections, symbols, ranges, or bounded queries;
- `Lookup on demand`: optional sources plus the trigger that justifies loading
  them;
- `Excluded from initial context`: explicitly deferred broad material; and
- a preflight record of the effective window, telemetry or estimate basis, and
  admitted allocation.

Large documents such as Roadmap, Architecture, semantic-model, generated
artifacts, and test logs must not be selected as whole-document `Must read`
inputs when a section, symbol, diff, or query can answer the task.

### Fresh execution contexts

Sprint planning, each child task, and independent integration review are
separate context boundaries. The master prompt is a dispatcher and durable
ledger; it does not retain implementation transcripts for later children.

The current instruction that launches a Prompt Contract v2 sprint explicitly
authorizes one sequential fresh-context task runner per manifest child and the
mandatory independent reviewer when required. Each runner receives only its
prompt, exact committed prerequisite, current repository state, and admitted
manifest. Runners may not delegate further.

If the active runtime cannot guarantee a fresh context, execution stops at the
boundary and reports the exact child prompt that must be launched next. It does
not fall back to accumulating the next task in the existing context.

### Layer ownership and validation

- Core owns permanent safety, context, validation, and reporting rules.
- Profiles only compose Core and Workflow modules and add family invariants.
- The base task template owns common child-prompt structure.
- Specialized Templates add only family-specific fields and evidence.
- Workflows own reusable execution behavior.
- Child prompts own only concrete scope, authorities, acceptance, and deltas.

`docs/codex/core/validation.md` is the single canonical workspace validation
source. Bootstrap, Templates, Workflows, and generated prompts reference it and
state only task-specific additions.

### Tool-output retention

Large complete command logs belong under
`local-artifacts/codex-runs/<run-id>/` when they are needed for diagnosis or
review. Conversation and prompt ledgers retain the command, exit status,
meaningful test count, concise failure excerpt, and artifact path. Secrets and
sensitive values must not be written to either destination.

### Git workflow

Sprint dispatch must reconcile current branch, merge, review, remediation, and
push behavior with the applicable `AGENTS.md` before the first write. Prompt
metadata never weakens or replaces those higher-priority rules.

## Consequences

- New prompts carry a small amount of structured metadata but remove much more
  repeated prose.
- Long sprint execution requires runtime support for fresh contexts or explicit
  user continuation at task boundaries.
- Prompt generation becomes testable in CI without rewriting historical suites.
- Context admission becomes deterministic enough to audit while remaining
  independent from one model or tokenizer.
- Full logs remain available without consuming the working conversation.
- Projects adopting the portable guide must supply their own Profiles,
  Templates, validation commands, branch policy, and context-window telemetry.

## Rejected alternatives

- **Rely only on automatic compaction.** Compaction is a recovery mechanism,
  not proof that high-priority constraints and exact evidence remain available.
- **Use one absolute token limit.** Model windows and runtime telemetry vary;
  percentage allocation plus a hard admission gate transfers across projects.
- **Rewrite historical prompt suites.** They are execution evidence and must not
  be retroactively changed.
- **Duplicate safety rules in every child prompt.** Repetition increases tokens
  and creates drift; the child references the base contract and selected
  modules instead.
- **Store complete test output in the conversation.** This consumes context
  without improving later semantic decisions.

## Implementation

- `docs/codex/core/context-management.md`
- `docs/codex/templates/task-prompt.md`
- `docs/codex/context-management-guide.md`
- `docs/codex/examples/task-prompt-v2.md`
- `scripts/validate-codex-prompts.sh`
- the compact bootstrap and sequential execution workflow
- the CI prompt-contract validation step
