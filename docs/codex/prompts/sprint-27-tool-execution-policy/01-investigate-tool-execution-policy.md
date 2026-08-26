# Investigate the Tool Execution Policy Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 27 execution plan
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-23-llm-provider-abstraction.md`
- `docs/reviews/sprint-26-ollama-integration.md`

## Prerequisites / Required gate

The committed Sprint 27 planning baseline is current and the working tree has
no conflicting task-created change.

## Investigation objective

Create only
`docs/architecture/tool-execution-policy-investigation.md` with the complete
repository-owned evidence needed to decide ADR-0049 safely.

## Questions to answer

- Which crate must own source-independent tool, request, policy, confirmation,
  execution, outcome, error, and audit values, and what dependency direction
  preserves Analysis, LLM Provider, Runtime, protocol, and future MCP boundaries?
- Which identity and bounded argument-summary values are required, which inputs
  are sensitive, and which traits or formatting would leak them?
- What conservative closed side-effect vocabulary and malformed/unknown/mixed
  handling can the first slice prove?
- What actor, scope, rule, precedence, conflict, policy-revision, and default-
  deny model is sufficient without inventing authentication or persistence?
- How can a confirmation be bound to the exact request and current decision,
  and how are missing, mismatched, stale, duplicate, and replayed evidence
  rejected before execution?
- What substitutable execution seam, cancellation/timeout ownership, no-retry,
  partial-failure, cleanup, terminal-outcome, and error precedence can be proven
  without a concrete tool or async-runtime dependency?
- Which bounded correlation and audit facts are safe to retain, order, compare,
  format, and test without storing unrestricted arguments, secrets, or output?
- Which consumers and public APIs must remain unchanged, and which future MCP,
  provider, Runtime, IDE, CLI, persistence, and concrete-tool concerns remain
  deferred?
- Which repository-owned deterministic fakes and cases prove allow, explicit
  deny, default deny, confirmation, no-invocation, failure containment,
  redaction, reordering, and repeated evaluation on supported platforms?

## Evidence scope and sources

- Workspace manifests and dependency graph; `crates/llm`, `crates/analysis`,
  `apps/runtime`, and `crates/protocol` definitions, tests, and consumers.
- Accepted ADRs, current architecture, Roadmap, relevant reviews, Codex AI Tool
  Policy contracts, Rust/toolchain evidence, and Git history.
- Deterministic owned-value, redaction, boxed-future, cancellation, and fake
  patterns already proven by repository tests.

## Excluded

Architecture acceptance, Rust/Cargo changes, real tool execution, external
research or network access, credentials, provider/MCP wire design, Runtime
composition, current-state docs, support claims, or Sprint completion.

## Completion Criteria

- Confirmed repository facts, accepted constraints, candidate alternatives,
  unknowns, and unsupported assumptions are separated.
- Every ADR-0049 decision input and deterministic oracle is documented, or the
  sprint stops with an exact evidence blocker.
- No external data or real side effect is required for the accepted first-slice
  test oracle.
- Only the named investigation document changes.

## Repository Safety

Preserve `.codex/`, Roadmap, prompt suites, ADRs, Rust/Cargo, current-state
documentation, source artifacts, and unrelated files.

## Task-specific Validation

- Re-run focused definition, consumer, dependency, test, and history searches.
- Audit the document against every investigation question and accepted ADR.
- Verify internal links and run `git diff --check`.

## Suggested commit message

`Investigate Sprint 27 tool execution policy`

## Final report additions

Report confirmed facts, accepted constraints, unresolved decisions, deterministic
oracle, exact created path, validation, commit hash, and final Git state.
