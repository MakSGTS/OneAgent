# Diagnostics Engine Workflow

Use this workflow for deterministic source-independent diagnostic identity,
normalization, orchestration, suppression, bounded reporting, and projection
over accepted canonical evidence.

## Canonical authority and inputs

- Identify every accepted immutable diagnostic input family and its canonical
  producer before implementation.
- Keep graph, validator, parser, adapter, Runtime, protocol, and editor evidence
  in their accepted ownership layers. The engine may normalize evidence but
  must not create a competing semantic or validation authority.
- Distinguish producer-local diagnostic precursors from canonical published
  inputs. Do not admit an internal parser value merely because it is named a
  diagnostic.
- Define whether validation is caller-supplied or engine-invoked and preserve
  the accepted validation boundary exactly.

## Identity, vocabulary, and collisions

- Define a closed family discriminator and the exact typed fields that make one
  diagnostic identity.
- Keep severity, category, code, kind, message, anchors, provenance, and other
  observable content separate from identity unless accepted architecture says
  otherwise.
- Define total order independently from insertion or hash order.
- Define exact duplicate, same-identity/different-content, and cross-family
  collision behavior. Never silently select one conflicting value.
- Preserve stable source vocabularies and document every additive public string
  projection.

## Suppression and disposition

- Define the exact suppression authority, match key, default, scope, and
  lifecycle. Absence of a repository-owned suppression source is evidence, not
  permission to invent a configuration grammar.
- Keep suppression distinct from deletion, severity changes, validation
  success, unsupported input, bounds, and transport filtering.
- Retain deterministic evidence and counts for suppressed diagnostics whenever
  the accepted report contract requires them.
- Keep configurable rule registration, discovery, dependency ordering, dynamic
  execution, scripting, plugins, and rule-produced findings outside a
  diagnostic-orchestration task unless a separate accepted Rules Engine
  contract explicitly includes them.

## Bounds, errors, and reporting

- Define input, identity-component, message, anchor, provenance, suppression,
  result, and output bounds before accepting untrusted or producer-owned data.
- Define whether each boundary rejects, truncates, omits, or returns a partial
  result. Make every non-complete outcome explicit and testable.
- Use checked summary arithmetic and prove that every distribution reconciles
  with total, active, suppressed, omitted, and returned counts.
- Keep error kinds closed and diagnostics bounded. Do not echo rejected source
  content, paths, identities, references, provenance, secrets, or internal
  error chains.
- Preserve deterministic filtering and page/projection order without
  reconstructing a summary from a truncated subset.

## Locations, provenance, and sensitive data

- Treat semantic IDs and typed provenance as canonical evidence, not implicit
  permission to expose a filesystem path or source fragment.
- Define the exact node or fact anchor used for location lookup and the behavior
  for missing, multiple, conflicting, span-less, escaping, or incompatible
  location evidence.
- Reuse accepted root confinement and coordinate conversions at protocol or UI
  boundaries. Never guess the exact offending token from a declaration anchor.
- Audit every public result and implicit error for path, source-content,
  provenance, candidate, credential, and rejected-input leakage.

## Snapshot, persistence, and protocol composition

- When an immutable snapshot stores a diagnostic report, construct and validate
  it before publication and prove equality across repeated builds.
- If persistent state is involved, decide explicitly whether the report is
  serialized or deterministically recomputed. Version persisted schemas and
  prove canonical equivalence when derived evidence is stored.
- Keep domain normalization out of protocol handlers. Protocols project one
  accepted immutable result through their own truthful schema, bounds,
  completeness, error, lifecycle, and sensitive-data rules.
- Preserve existing catalogs, capabilities, policy gates, compatibility
  revisions, and clients unless the accepted task explicitly migrates them.

## Deterministic evidence

- Cover empty, each family/severity/category/disposition, exact duplicate,
  conflicting identity, cross-family collision, reordered, exact/over bound,
  missing evidence, suppression, filtering, and repeated evaluation cases.
- Use repository-owned graph, validator, Workspace, cache, and public-process
  fixtures as applicable. Record every zero-match, skip, and environment limit.
- Prove producer compatibility, snapshot/cache equality, protocol truth, and
  complete validation for every changed public or observable boundary.

## Boundary

This workflow does not choose a concrete engine owner, input family, identity,
severity/category vocabulary, suppression mechanism, limit, persistence schema,
protocol shape, UI, or rule system. Those decisions belong to accepted ADRs and
task prompts. It does not authorize new diagnostic producers, Rules Engine
implementation, mutable-document analysis, fixes, edits, telemetry, remote
transport, or broad performance/security claims.
