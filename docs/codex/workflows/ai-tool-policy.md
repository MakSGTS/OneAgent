# AI Tool Policy Workflow

Use this workflow for source-independent AI tool authorization and execution
policy boundaries.

## Tool and request boundary

- Identify the owner of tool identity, declared operation, arguments, execution
  request, policy input, decision, confirmation, result, and audit evidence.
- Keep policy decisions independent from MCP, provider, IDE, CLI, transport,
  filesystem, shell, network, and other concrete execution schemas.
- Define stable validation, size bounds, ordering, and redaction for every value
  admitted to policy evaluation or retained as evidence.
- Reject unknown or malformed identities, operation classes, policy values, and
  result states explicitly instead of interpreting them as permission.

## Authorization and denial

- Make the default behavior explicit and fail closed when no applicable rule,
  actor, scope, or trustworthy classification exists.
- Define deterministic rule precedence, scope matching, conflicts, and the
  distinction between allow, deny, and confirmation-required outcomes.
- Preserve a policy decision as data that can be inspected before execution;
  never treat a request, model output, tool declaration, or prior success as
  authorization by itself.
- Re-evaluate or reject stale decisions when accepted identity, arguments,
  policy revision, or execution context changes.

## Side effects and confirmation

- Use an accepted closed side-effect vocabulary and define conservative
  handling for unknown, mixed, conditional, or understated effects.
- State which effects may proceed automatically, require explicit confirmation,
  or are denied, including destructive, irreversible, external, privileged,
  secret-bearing, and third-party-visible actions when applicable.
- Bind confirmation to the exact request and policy decision with bounded
  lifetime or one-use semantics when the accepted architecture requires it.
- Treat missing, mismatched, expired, replayed, or ambiguous confirmation as a
  denial and perform no tool side effect.

## Execution and failure containment

- Separate policy evaluation from the executor that performs the concrete
  action and require an accepted decision before crossing the side-effect
  boundary.
- Define cancellation, timeout, partial completion, duplicate submission,
  retry, rollback, and cleanup behavior before implementation; do not invent
  automatic retries or reversibility.
- Contain executor failures to the current request and preserve enough bounded
  terminal evidence to distinguish denied, cancelled, failed, partial, and
  completed outcomes.
- Do not claim sandboxing, rollback, idempotency, atomicity, or external effect
  prevention without deterministic evidence for the concrete executor.

## Audit evidence and sensitive data

- Define stable correlation between request, policy input, decision,
  confirmation, execution attempt, and terminal result without retaining
  unrestricted arguments, secrets, outputs, or external payloads.
- Make audit ordering and repeated evaluation deterministic and identify which
  fields are safe for implicit formatting, serialization, logging, and tests.
- Redact or omit sensitive content before it reaches diagnostics, snapshots,
  fixtures, audit records, or error displays.
- Treat audit evidence as an observation of decisions and outcomes, not as a
  source of future authorization.

## Contract evidence

- Use repository-owned deterministic policies, fake executors, and bounded
  fixtures that require no external service, credential, or destructive action.
- Cover allow, explicit deny, default deny, confirmation, mismatch, expiry or
  replay, malformed, unknown, duplicate, reordered, cancelled, timeout,
  executor failure, partial outcome, redaction, and repeated cases as
  applicable.
- Prove that denied or unconfirmed requests never invoke the fake executor and
  that every attempted execution has one inspectable terminal outcome.
- Audit consumers before changing public policy values, decision semantics, or
  execution gates.

## Boundaries

This workflow does not select a crate owner, rule language, actor model, policy
storage, confirmation user experience, executor trait, async runtime, audit
sink, serialization format, transport, MCP schema, provider tool-call wire
format, or concrete tool catalog. Those decisions belong to accepted ADRs or
the current task. It does not authorize real external effects, credentials, or
privileged execution.
