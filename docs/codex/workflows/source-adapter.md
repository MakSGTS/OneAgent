# Source Adapter Workflow

Use this workflow for deterministic ingestion of one source format whose
configuration is represented by multiple files or directories and must map to
the existing source-independent model.

## Required source evidence

- Use repository-owned source artifacts or provenance-backed reductions.
- Record the exact project-root markers, artifact roles, path relationships,
  namespace or version discriminators, and join keys used by discovery.
- Distinguish required, optional, repeated, unknown, and conflicting artifacts.
- Do not infer a serialized field, path convention, default, or relationship
  from another adapter merely because both adapters represent the same fact.

## Discovery and assembly contract

- Define the configuration boundary and the rule that prevents traversal into
  an already accepted project root.
- Discover into canonical path order before parsing or contribution.
- Define duplicate, overlap, symlink escape, unreadable entry, missing root,
  incomplete project, and conflicting format-marker behavior explicitly.
- Assemble related artifacts by accepted source keys rather than encounter
  order, directory ordinal, or first-match selection.
- Keep discovery, artifact assembly, parsing, and semantic contribution as
  independently testable stages even when one production builder orchestrates
  them.

## Completeness and failure policy

- The caller or accepted source contract must classify the input as complete or
  explicitly partial; absence alone must not guess partiality.
- Define which missing or malformed artifacts are fatal for discovery, one
  configuration, one entity, or one observation.
- Preserve typed missing, malformed, unsupported, ambiguous, incompatible, and
  partial outcomes through the production boundary without placeholder facts.
- Do not silently downgrade malformed required input to an absent optional
  value or silently discard a valid sibling because another sibling fails.

## Canonical mapping

- Map source observations to existing source-independent identifiers, names,
  payloads, ownership, references, and relations defined by accepted
  architecture.
- Source paths, serialization order, adapter names, and parser-local values do
  not become semantic identity unless an accepted decision explicitly says so.
- Keep adapter-specific observations out of metadata and graph public APIs
  unless a source-independent contract has first been accepted.
- Reuse canonical graph validation, Query, Diff, Impact, request, report, and
  Coverage behavior instead of creating an adapter-specific semantic authority.

## Cross-adapter conformance

- Define the exact equivalence oracle before claiming that two source formats
  represent the same configuration.
- Compare canonical entity and relation identities, kinds, names, payloads,
  ownership, requests, terminal outcomes, and consumer-visible results that are
  accepted for both adapters.
- Identify deliberate adapter-specific differences, including source paths,
  producer identifiers, and source-format diagnostics, and exclude only those
  dimensions from equivalence.
- Prove conformance through public production entry points over paired
  provenance-backed fixtures. Parser-only equality is insufficient.
- Cover both equivalent inputs and a controlled semantic change so the oracle
  cannot pass by comparing empty or incomplete results.

## Determinism and validation

- Test reordered directory entries, reordered equivalent XML, duplicate
  observations, repeated discovery, repeated parsing, and repeated end-to-end
  builds.
- Cover complete, partial, missing, malformed, unsupported, ambiguous, and
  conflicting cases applicable to the accepted source contract.
- Preserve existing adapter behavior and run focused detector, parser, builder,
  conformance, validation, and Coverage checks before the required package or
  workspace gate.

## Boundary

This workflow does not select semantic identity, payload, ownership, endpoint,
or partial-workspace architecture. Those contracts must come from accepted ADRs
or a preceding architecture task. It does not require a new graph model merely
because a new serialized source format is added.
