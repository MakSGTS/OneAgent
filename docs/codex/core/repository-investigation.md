# Repository Investigation

Investigate before planning or editing. Keep the investigation proportional to
the task.

## Standard investigation steps

1. Read applicable instructions, including `AGENTS.md`.
2. Record Git state with `git status --short`.
3. Locate current definitions, modules, APIs, tests, and existing behavior
   relevant to the task.
4. Inspect API usages that may be affected.
5. Read accepted ADRs and architecture documents relevant to the task.
6. Inspect fixtures and real source artifacts when source parsing is involved.
7. Inspect Coverage Registry, Roadmap, and documentation impact when capability
   status or architecture state may change.
8. Identify validation commands that match the changed components.

## Evidence categories

Distinguish clearly between:

- facts confirmed from repository code, tests, fixtures, or documentation;
- accepted decisions from ADRs and authoritative architecture documents;
- assumptions that still need validation;
- unresolved implementation evidence.

## Scope control

Do not require broad repository-wide analysis when focused search is enough.
Architecture tasks need broader decision context. Small documentation or local
implementation tasks should inspect only the relevant directories, docs, tests,
and consumers.

Already accepted architecture must not be re-litigated in implementation tasks.
Implementation may inspect repository APIs and technical constraints, but it
must not repeat architecture selection unless concrete repository evidence shows
that the accepted contract cannot be implemented.
