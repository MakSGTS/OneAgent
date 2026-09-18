# Rules Engine Workflow

Use this workflow for deterministic source-independent rule registration,
dependency validation, configuration, execution, result production, and
integration with accepted diagnostic evidence.

## Canonical authority and inputs

- Identify every accepted immutable rule input and its canonical owner before
  implementation.
- Keep graph, validator, diagnostics, parser, adapter, Runtime, protocol, and
  editor evidence in their accepted ownership layers. Rules may evaluate
  canonical evidence but must not create a competing semantic, validation, or
  diagnostic authority.
- Define whether the engine borrows an immutable snapshot, consumes a bounded
  request, or receives precomputed evidence, and preserve that boundary
  exactly.
- Separate rule execution from source discovery, parsing, graph construction,
  validation, reporting projection, and edit application.

## Identity and registration

- Define a validated typed rule identity and the exact fields that participate
  in identity, equality, and total order.
- Define one registration owner, registration lifecycle, and closed duplicate
  or conflicting-registration behavior. Never select a registration by
  insertion or hash order.
- Define registry bounds and deterministic enumeration independently from
  construction order.
- Distinguish built-in registration, configuration, and runtime execution from
  future dynamic discovery, plugins, scripts, or remote rule acquisition.

## Dependencies and execution order

- Define the exact dependency vocabulary and whether each relation expresses
  ordering, required evidence, result consumption, or another accepted
  constraint.
- Reject missing dependencies, self-dependencies, cycles, duplicates, and
  incompatible dependency forms deterministically before affected execution.
- Define a canonical total execution order with an explicit tie-breaker for
  independent rules.
- Prove order independence for equivalent registration and dependency input.
  Do not rely on filesystem, iterator, insertion, scheduler, or hash order.

## Configuration and applicability

- Define the configuration authority, identity, defaults, validation,
  precedence, scope, lifecycle, and compatibility behavior before accepting
  configuration input.
- Distinguish disabled, inapplicable, unsupported, invalidly configured,
  dependency-blocked, cancelled, failed, and successfully completed rules.
- Absence of a repository-owned configuration source is evidence, not
  permission to invent a file grammar, environment variable, persistence
  schema, protocol shape, or UI.
- Prove empty, default, exact, unknown, duplicate, reordered, incompatible, and
  exact/over-bound configuration cases as applicable.

## Execution lifecycle and failures

- Define execution ownership, synchronous or asynchronous boundary,
  cancellation checkpoints, failure containment, and whether independent rules
  may continue after one rule fails.
- Define atomic versus partial result behavior explicitly. Never present an
  incomplete execution as complete.
- Bound rule count, dependency count, configuration values, input evidence,
  emitted results, messages, anchors, provenance, and error detail before
  cloning or publishing them.
- Keep error kinds closed and bounded. Do not echo source content, paths,
  secrets, rejected configuration, identities not intended for exposure,
  provenance payloads, or internal error chains.

## Results and diagnostic integration

- Define one typed per-rule terminal result contract and one deterministic
  aggregate execution result.
- Preserve the distinction between rule execution status, rule-produced
  evidence, diagnostics disposition, validation success, transport filtering,
  and edit availability.
- When rules produce diagnostic evidence, map only through the accepted
  diagnostic identity, duplicate/conflict, ordering, suppression, summary,
  bounds, provenance, and sensitive-data contracts.
- Define cross-rule duplicate or conflict behavior. Never silently select one
  result based on execution order.
- Keep protocol and UI projection outside the engine; project only an accepted
  immutable result through truthful capability, bounds, completeness, policy,
  confinement, and compatibility contracts.

## Snapshot and persistence composition

- Construct and validate complete rule registration, configuration, execution,
  and derived diagnostic evidence before publishing an immutable snapshot.
- If persistent state is involved, decide explicitly whether registrations,
  configuration, or results are serialized or deterministically recomputed.
  Version persisted schemas and prove clean-rebuild equivalence.
- Define invalidation when semantic inputs, rule registrations, dependencies,
  configuration, engine compatibility, or accepted result contracts change.
- Preserve existing snapshot, cache, lifecycle, protocol, and client behavior
  unless the accepted task explicitly migrates it.

## Deterministic evidence

- Cover empty, single, multiple independent, dependency chain, diamond,
  duplicate, conflict, missing dependency, self-dependency, cycle, reordered,
  disabled, inapplicable, invalid configuration, exact/over bound, rule
  failure, cancellation, repeated execution, and result collision cases as
  applicable.
- Use repository-owned graph, diagnostic, Workspace, cache, protocol, and
  public-process fixtures where applicable. Record every zero-match, skip, and
  environment limit.
- Prove producer compatibility, deterministic registry and execution order,
  complete result reconciliation, diagnostic integration, snapshot/cache
  equality, protocol truth, and full validation for every changed boundary.

## Boundary

This workflow does not choose a concrete engine owner, rule trait, identity
grammar, registration source, dependency meaning, configuration format,
execution scheduler, failure policy, result vocabulary, limit, persistence
schema, protocol shape, UI, or first rule set. Those decisions belong to
accepted ADRs and task prompts. It does not authorize dynamic plugins,
scripting, remote rules, mutable-document analysis, automatic fixes, source
edits, telemetry, or broad performance or security claims.
