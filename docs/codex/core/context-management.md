# Context Management

Use this module for every Prompt Contract v2 task.

## Context budget

Treat the effective model context window as a task resource:

- static instructions: maximum 15%;
- initially loaded authorities: maximum 20%;
- normal pre-work target: maximum 35%;
- pre-work hard stop: 50%;
- source, diffs, tools, and validation working set: minimum 35%; and
- final response and safety reserve: minimum 15%.

Runtime token telemetry is authoritative when available. Otherwise a
conservative character-based estimate may be used only for preflight admission
and must be labelled as an estimate. Never report an estimate as measured token
usage.

Before substantive investigation, record the effective window or `unknown`, the
measurement basis, admitted static and authority material, and a
`pass|warning|blocked` decision. At warning, narrow the manifest. At the hard
stop, split the task or report a context-budget blocker before loading more.

## Context Manifest

Every child prompt must contain:

- `Must read`: exact repository paths plus headings, symbols, ranges, or bounded
  queries and their purpose;
- `Lookup on demand`: optional sources plus the evidence trigger for loading
  each source;
- `Excluded from initial context`: broad, historical, generated, or unrelated
  material that must not be loaded initially; and
- `Preflight`: the budget evidence and admission decision.

Do not load an entire large Roadmap, Architecture, semantic-model, generated
artifact, fixture corpus, or command log when a section, symbol, diff, or query
is sufficient. Do not preload `Lookup on demand` material.

## Context lifecycle

- Start every child task in a guaranteed fresh context.
- Load only its prompt, committed prerequisite, applicable instructions,
  selected framework modules, and admitted `Must read` material.
- Refresh mutable facts from the repository instead of passing prior
  implementation transcripts.
- Persist only a compact task ledger across child boundaries.
- Stop at the boundary when fresh context cannot be guaranteed; provide the
  exact next prompt path and prerequisite instead of continuing in the current
  context.
- Treat automatic compaction as runtime resilience, not as evidence that the
  task remains within budget.

## Tool and validation output

Keep direct output bounded. When a complete log is materially large and must be
retained, write it under `local-artifacts/codex-runs/<run-id>/` and keep only the
command, exit status, meaningful count, concise failure excerpt, and artifact
path in the conversation or ledger.

Do not persist secrets, credentials, environment dumps, raw sensitive payloads,
or unrelated source content. A log artifact is evidence only when the command
status and relationship to the task are recorded.

## Duplication rule

State a permanent instruction once in its canonical Core or Workflow owner.
Profiles compose modules, Templates add task-family fields, and child prompts
add concrete task data. Do not copy permanent safety, validation, reporting,
review, or context rules into child prompts.

