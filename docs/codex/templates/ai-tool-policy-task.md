# AI Tool Policy Task Template

## Purpose

Use this template for source-independent AI tool requests, authorization,
side-effect classification, confirmation, execution gating, terminal outcomes,
or audit evidence.

## Recommended profile

- `docs/codex/profiles/ai-tool-policy-implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Prerequisites / required gate
- Task
- Tool identity, request, and argument boundary
- Actor, scope, and policy input contract
- Rule precedence and default-deny behavior
- Side-effect classification and confirmation boundary
- Decision-to-execution binding
- Cancellation, timeout, retry, partial outcome, and cleanup contract
- Audit evidence, ordering, retention, and redaction
- Deterministic policy and fake-executor oracle
- Consumer and concrete-tool compatibility
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Keep shared policy values independent from MCP, provider, IDE, CLI,
  transport, filesystem, shell, network, and concrete executor schemas.
- Fail closed for unknown or malformed identities, rules, effects,
  confirmations, and execution context; request construction or model output is
  never authorization by itself.
- Bind every execution attempt to an accepted current policy decision and prove
  that deny or missing confirmation performs no executor action.
- Define deterministic precedence, side-effect conservatism, confirmation
  mismatch/replay behavior, terminal outcomes, audit ordering, and redaction.
- Prove supported behavior through repository-owned deterministic evidence that
  performs no real external, destructive, privileged, or credentialed action.
- Audit all consumers before changing a public policy request, decision,
  confirmation, outcome, or audit contract.

## Additional report sections

- Policy ownership and public boundary
- Authorization and default-deny behavior
- Side effects and confirmation
- Execution gating and failure containment
- Audit evidence and redaction
- Contract-test evidence
- Consumer and executor impact
- Deferred tool, transport, provider, Runtime, and UX scope

## Additional validation

- Run focused identity, request, rule, precedence, deny, confirmation,
  execution-gating, cancellation, timeout, failure, audit, redaction, and
  repeated-operation tests applicable to the changed slice.
- Run non-zero fake-executor conformance tests for every execution behavior
  claimed as supported.
- Run affected LLM Provider, Runtime, protocol, MCP, IDE, CLI, or consumer
  checks when their public or observable behavior changes.
- Run full workspace validation for production policy behavior, public APIs,
  Cargo manifests, or execution-boundary changes as required by
  `docs/codex/core/validation.md`.
