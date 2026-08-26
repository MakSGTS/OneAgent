# AI Tool Policy Implementation Profile

## Purpose

Use this profile for source-independent AI tool authorization, side-effect
classification, confirmation, execution gating, failure containment, and audit
evidence contracts.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/ai-tool-policy.md`
- `docs/codex/workflows/runtime-service.md` when Runtime lifecycle, owned
  execution, cancellation, transport exposure, or audit delivery changes
- `docs/codex/workflows/llm-provider.md` when provider capability or tool-call
  mapping changes

## Task-family expectations

- Keep source-independent tool identities, requests, policy decisions,
  confirmation, outcomes, and audit evidence separate from concrete tool and
  transport schemas.
- Fail closed for missing, unknown, malformed, stale, or ambiguous policy input
  and require an accepted decision before invoking an executor.
- Define side effects, confirmation binding, cancellation, timeout, retry,
  partial completion, cleanup, redaction, and audit behavior only from accepted
  architecture and repository evidence.
- Prove policy and gating behavior with deterministic repository-owned fakes
  that perform no real external or destructive action.
- Keep MCP schemas, provider wire formats, Runtime orchestration, IDE/CLI
  confirmation UX, policy persistence, and concrete tools outside a task unless
  separately accepted and explicitly included.
