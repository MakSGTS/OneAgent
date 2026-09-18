# Refactoring and Safe Edits Workflow

Use this workflow for deterministic semantic refactoring planning and checked,
reversible source-edit transactions over accepted canonical evidence.

## Authority and phase boundary

- Identify the canonical owners of semantic identity, source locations,
  provenance, diagnostics, impact, Workspace publications, repository state,
  and authorization before implementation.
- Keep planning, preview, authorization, application, and post-edit validation
  as explicit phases. A successful earlier phase is evidence for the next phase,
  not implicit permission to enter it.
- Treat diagnostics, rules, impact reports, paths, Git statuses, and model output
  as bounded evidence only. None independently authorizes an edit or becomes a
  competing semantic authority.
- Define the exact supported refactoring family and source formats from
  repository evidence. Reject unsupported targets or source forms explicitly.

## Target, snapshot, and precondition contract

- Resolve targets through accepted stable semantic identity and an immutable
  complete input snapshot. Do not infer a semantic rename from a path, display
  name, Git status, or impact relation.
- Define every precondition and its evidence owner, including target existence
  and kind, expected source identity or content version, graph publication,
  repository baseline when applicable, source format, and required completeness.
- Bind preconditions to the plan without exposing sensitive source content or
  absolute paths. Define equality, compatibility, staleness, and process-lifetime
  limits explicitly.
- Reject missing, ambiguous, conflicting, stale, incompatible, out-of-bound, or
  incomplete inputs atomically. Do not select a candidate by encounter order.

## Plan identity and deterministic operations

- Define immutable plan identity separately from display labels, mutable
  content, preview rendering, and execution-local metadata.
- Define a closed operation vocabulary and the exact target, source anchor,
  expected content, replacement, dependency, and ordering fields applicable to
  the accepted slice.
- Produce one canonical total order with explicit tie-breakers independent of
  graph traversal, hash, discovery, filesystem, or request order.
- Define exact duplicates, overlapping edits, same-anchor conflicts,
  dependency cycles, incompatible operations, and cross-file ordering before
  aggregation. Never resolve a conflict through last-writer-wins behavior.
- Bound targets, files, operations, dependencies, identifier and replacement
  bytes, preview output, and error detail before cloning or publication.

## Preview, completeness, and failure behavior

- Make preview a deterministic projection of the immutable plan and accepted
  source snapshot. Preview must not mutate source, repository, Workspace,
  cache, editor, or protocol state.
- Reconcile requested, planned, conflicted, rejected, omitted, and returned
  counts with checked arithmetic. Truncation or omission must be explicit and
  must not produce an apparently complete executable plan.
- Use closed typed failures and redact rejected source content, replacement
  text, absolute paths, repository configuration, credentials, environment
  data, and internal error chains unless an accepted public contract requires a
  confined value.
- Prove empty, positive, negative, duplicate, conflict, reordered, repeated,
  exact-limit, over-limit, stale, incompatible, and missing-evidence cases.

## Checked edit transaction

Apply this section only when the accepted task explicitly includes mutation.

- Recheck every bound precondition immediately before the first mutation and
  define concurrent-change behavior. A stale preview or plan must fail before
  writing.
- Confine every source path to the accepted Workspace or repository root before
  reading, staging, writing, renaming, removing, or restoring it. Define
  symlink, traversal, separator, case, Unicode, non-text, permission, and
  unsupported-filesystem behavior applicable to the slice.
- Define the exact write set, temporary and backup ownership, durability
  assumptions, atomicity boundary, commit point, cleanup, and recovery behavior.
  Do not claim cross-file atomicity when the implementation cannot prove it.
- Preserve the pre-edit state required for rollback before mutation. Define
  rollback order, rollback-failure precedence, retained recovery evidence, and
  whether a successful transaction is mechanically reversible.
- Cover injected failures before, during, and after each observable mutation;
  cancellation; process interruption where deterministically testable;
  concurrent source change; rollback failure; repeated apply; and cleanup.
- Do not mutate Git history, index, remotes, credentials, hooks, configuration,
  ignored files, or unrelated paths unless a separate accepted contract owns
  that behavior.

## Post-edit semantic validation

- Rebuild through the accepted production source adapter and complete Workspace
  path after application. A parser-only or text comparison is not semantic
  validation.
- Define success through the accepted semantic target, graph, diagnostic,
  impact, and compatibility invariants. Do not treat a clean parse alone as a
  successful refactoring.
- If validation fails, preserve the accepted atomicity and rollback contract,
  report the primary and recovery outcomes distinctly, and publish no partial
  successful snapshot or edit result.
- Prove successful end-state equivalence, rollback equivalence, repeated fresh
  runs, and unchanged unrelated source and semantic evidence.

## Runtime, policy, protocol, and client composition

- Keep domain planning and transaction logic outside transport and UI handlers.
  Public surfaces project one accepted immutable result through truthful schema,
  bounds, completeness, lifecycle, and sensitive-data rules.
- Preserve existing Tool Policy classification and confirmation boundaries.
  Read-only plan or preview behavior must not make mutation implicitly allowed.
- Define cancellation, shutdown, repeated request, snapshot replacement, cache,
  and multi-client behavior for every changed Runtime boundary.
- Audit compatibility for every affected protocol, client, editor lifecycle,
  filesystem watcher, cache, Git adapter, diagnostics, and impact consumer.

## Deterministic evidence

- Use repository-owned provenance fixtures and temporary confined workspaces.
  Synthetic text alone is insufficient for a production source-edit claim.
- Prove input-order independence, stable identity and ordering, exact and
  one-over bounds, redacted failures, repeated planning, and unchanged source
  state for every planning-only task.
- For mutation, prove preview/apply agreement, precondition recheck, confined
  writes, all accepted failure points, rollback and reversibility, cleanup,
  production rebuild, semantic validation, and unaffected-file preservation.
- Run affected Graph, Analysis, Workspace, adapter, cache, Runtime, policy,
  protocol, public-process, and client checks for every changed boundary.
  Record zero matches, skips, platform limits, and unavailable host evidence.

## Boundary

This workflow does not choose a refactoring family, plan owner, target or plan
identity, operation vocabulary, source format, content-version scheme, limit,
transaction algorithm, filesystem primitive, rollback mechanism, validation
policy, persistence schema, Runtime surface, protocol, client UI, or first
production slice. Those decisions belong to accepted ADRs and task prompts. It
does not authorize source mutation, Git mutation, remote access, credentials,
model-generated edits, telemetry, benchmarks, or broad performance or security
claims.
