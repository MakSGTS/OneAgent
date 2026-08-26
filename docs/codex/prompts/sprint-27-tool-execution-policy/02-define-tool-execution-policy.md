# Define the Tool Execution Policy Contract

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs and architecture documents

- `docs/architecture/tool-execution-policy-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`, Sprint 27 execution plan

## Prerequisites / Required gate

The committed Task 1 investigation answers every decision input or reports no
safe architecture path. Stop before edits when required evidence is missing.

## Task

Create only `docs/adr/0049-tool-execution-policy.md` and accept the smallest
source-independent first slice for safe tool execution.

Define ownership and dependency direction; exact public value vocabulary and
bounds; validation precedence and sensitive-data behavior; conservative side-
effect classification; actor/scope/rule matching, conflicts, canonicalization,
decision precedence, revision binding, and default deny; confirmation trust,
exact binding, staleness and replay rejection; decision-to-execution gate;
substitutable executor, cancellation/timeout/no-retry and cleanup behavior;
terminal outcomes and failure precedence; bounded audit correlation, ordering,
formatting, and redaction; deterministic fake conformance; compatibility,
implementation prerequisites, risks, rejected alternatives, and deferred scope.

## Scope

### Included

- One accepted ADR grounded in Task 1 evidence.
- Exact dependency/feature approval result for Task 3.

### Excluded

Rust, Cargo, lockfile, current-state docs, real tool execution, provider/MCP wire
mapping, Runtime/UX integration, implementation, or support claims.

## Acceptance Criteria

- ADR-0049 closes every first-slice decision without contradicting ADR-0037,
  ADR-0044, or ADR-0045.
- Unknown, malformed, missing, stale, or ambiguous input fails closed.
- Request/model output/audit history alone never authorizes execution.
- Denied or unconfirmed requests cannot reach an executor; confirmation binds to
  the exact request and decision under an implementable one-use/replay contract.
- Outcomes and audit evidence are deterministic, bounded, redacted, and do not
  claim rollback, sandboxing, atomicity, or real-effect safety.
- Repository-owned fakes cover the full accepted boundary without external or
  destructive action.
- Architecture acceptance alone does not claim tool-policy support.

## Repository Safety

Modify only ADR-0049. Preserve `.codex/`, Roadmap, prompts, investigation,
existing ADRs, code, manifests, lockfile, and unrelated files.

## Task-specific Validation

- Audit ADR-0049 against every investigation question and inherited authority.
- Verify links, dependency direction, precedence, decision/rejection
  consistency, exact approval gate, and accepted-versus-deferred scope.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 27 tool execution policy`

## Final report additions

Report the accepted boundary, dependency approval result, rejected alternatives,
deferred scope, exact created path, validation, commit hash, and final Git state.
