# Source Adapter Task Template

## Purpose

Use this template for one bounded source-adapter implementation slice involving
multi-artifact discovery, assembly, parsing, source-independent mapping, or
cross-adapter conformance.

## Recommended profile

- `docs/codex/profiles/source-adapter-implementation.md`

## Required task-specific sections

- Authoritative ADRs / source-format and architecture documents
- Prerequisites / required gate
- Task
- Source evidence / paired fixtures
- Discovery boundary and format detection
- Artifact roles and assembly contract
- Completeness and failure policy
- Canonical mapping and identity compatibility
- Cross-adapter conformance oracle, when applicable
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Use real source evidence; do not invent markers, files, XML fields, nesting,
  joins, defaults, or source relationships.
- Define deterministic project detection, canonical artifact enumeration, and
  exact behavior for missing, duplicate, overlapping, unreadable, unsupported,
  malformed, ambiguous, and conflicting inputs as applicable.
- State whether the supplied workspace is complete or explicitly partial and
  keep that classification separate from artifact absence.
- Keep discovery, artifact assembly, parsing, and semantic contribution
  independently testable.
- Map to accepted source-independent identities and semantics without exposing
  adapter-local structures through public domain or graph APIs.
- For conformance, compare a declared canonical projection and document every
  intentionally excluded adapter-specific dimension.
- Prove positive and negative behavior through the production adapter entry
  point, including reordering and repeated runs.
- Preserve existing adapters and unsupported source forms unless the task
  explicitly and authoritatively includes them.

## Additional report sections

- Source and discovery evidence
- Artifact assembly and completeness policy
- Canonical mapping
- Conformance matrix and deliberate differences
- Production entry point
- Unsupported or deferred source cases
- Existing-adapter compatibility

## Additional validation

- Run focused discovery, parser, assembly, and production-builder tests for the
  changed adapter slice.
- Run paired conformance fixtures through their exact non-zero test target when
  cross-adapter equivalence is claimed.
- Run affected package checks and the complete workspace validation when Rust,
  a public API, parser behavior, or production contribution changes.
