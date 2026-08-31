# Git Change Adapter Workflow

Use this workflow for deterministic ingestion of bounded Git repository change
evidence into accepted source-independent Workspace change inputs.

## Authority and repository boundary

- Identify the canonical Workspace, source-adapter, Graph, Analysis, and
  diagnostic owners before implementation. Git supplies change evidence and
  must not become a semantic, validation, impact, or edit authority.
- Define the accepted repository root and its relationship to the Workspace
  root. Confine every emitted path to that boundary before publication.
- Define nested repositories, worktrees, submodules, symlinks, ignored paths,
  bare repositories, missing metadata, and unsupported layouts explicitly when
  they are applicable to the accepted slice.
- Do not infer semantic entity identity, dependency, diagnostic meaning, or
  edit safety from a path or Git status alone.

## Baseline and current endpoint contract

- Define typed baseline and current endpoints, their ownership, resolution,
  validation, compatibility, and equality before reading change evidence.
- State which repository layers are included, such as committed trees, index,
  worktree, untracked paths, or conflicts. Absence of an accepted layer is not
  permission to include it implicitly.
- Reject missing, ambiguous, incompatible, moving, or out-of-bound endpoints
  with closed typed outcomes rather than silently selecting a fallback.
- Define concurrent repository mutation behavior and whether one result is a
  complete stable snapshot, a retryable observation, or a terminal failure.

## Change identity and normalization

- Define the exact fields participating in normalized change identity,
  equality, and total order independently from Git output order.
- Define a closed status vocabulary and the representation of old and new paths
  and object kinds. Preserve absence instead of inventing an endpoint for an
  addition or deletion.
- Define modifications, additions, deletions, rename and copy candidates, type
  changes, conflicts or unmerged states, untracked paths, ignored paths, and
  submodule changes explicitly when applicable.
- Treat rename and copy detection as an accepted evidence policy, not as
  semantic object identity. Define threshold, ambiguity, tie, and disabled
  behavior if similarity detection is included.
- Normalize separators and path components without reading outside the accepted
  root, resolving an escaping path, or losing byte-level incompatibility.

## Ordering, duplicates, bounds, and failures

- Produce one canonical total order with explicit tie-breakers for equivalent
  change sets regardless of traversal, process, hash, or platform order.
- Define duplicate and conflicting observation behavior before aggregation.
  Never choose one observation by encounter order.
- Bound endpoint count, path count, path length, output count, process output,
  retries, and error detail before cloning or publication.
- Keep errors closed, bounded, and redacted. Do not expose absolute paths,
  repository configuration, credentials, environment data, source content,
  raw command output, or internal error chains unless an accepted public
  contract requires an explicitly confined value.

## Workspace change-input integration

- Map normalized Git observations to an accepted source-independent Workspace
  change-input contract. Keep Git-only endpoint and status evidence out of
  public Workspace or semantic APIs unless architecture accepts it there.
- Define the equivalence oracle between one Git-derived change set and the
  existing filesystem observation or complete Workspace end state. Equivalent
  semantic inputs must trigger equivalent rebuild behavior without requiring
  identical adapter provenance.
- Preserve complete rebuild, coalescing, atomic publication, last-valid
  snapshot, recovery, cache, lifecycle, and cancellation contracts unless the
  accepted task explicitly migrates them.
- A normalized empty or irrelevant change must not claim semantic work; a
  Git-observed change must not bypass production discovery, parsing,
  validation, or complete snapshot construction.

## Process, dependency, and compatibility boundary

- Select a library, process, or injected-reader boundary only through accepted
  architecture and repository evidence. Do not assume a Git library or shell
  grammar exists.
- When a process is used, define executable discovery, arguments, environment,
  working directory, stdin/stdout/stderr ownership, exit and signal mapping,
  output encoding, cancellation, timeout, cleanup, and test injection.
- When a production dependency is added, audit version, features, transitive
  dependencies, licenses, unsafe surface, and cross-platform support before
  changing Cargo manifests.
- Preserve supported macOS and Windows behavior and use repository-owned
  temporary repositories or provenance-backed fixtures for executable evidence.

## Deterministic evidence

- Cover empty, added, modified, deleted, reordered, repeated, exact duplicate,
  rename or copy candidate, type-change, conflict, untracked, ignored, missing,
  incompatible, malformed, out-of-bound, concurrent-mutation, cancellation,
  and exact/over-bound cases applicable to the slice.
- Prove equivalent results for equivalent repository states created through
  different operation orders and for repeated fresh adapter runs.
- Run production entry-point tests plus affected Workspace, cache, lifecycle,
  adapter, Graph, Analysis, protocol, and client checks for every changed
  boundary. Record zero matches, skips, platform limits, and unavailable Git
  capabilities separately.

## Boundary

This workflow does not choose a Git implementation dependency, executable,
repository-discovery rule, endpoint vocabulary, included state layers, status
model, rename policy, path representation, limits, persistence schema, Runtime
surface, protocol, UI, or first production slice. Those decisions belong to
accepted ADRs and task prompts. It does not authorize remote repository access,
credentials, semantic impact analysis, refactoring, source mutation, safe edit
transactions, telemetry, or broad performance or security claims.
