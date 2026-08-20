# Define Sprint 14 Designer XML Adapter Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 14 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/architecture/designer-xml-source-investigation.md`
- ADRs 0004, 0005, 0007, and 0008

## Prerequisites / Required gate

Require committed Task 1 evidence sufficient to decide every included source,
identity, completeness, failure, provenance, and conformance contract. Stop
without edits when a required decision lacks source evidence.

## Task

Create `docs/adr/0036-designer-xml-adapter.md` and synchronize only the
planning-level Semantic Model text required to identify the accepted source
adapter boundary.

## Scope

### Included

- Exact project markers, accepted format/version scope, detection conflicts,
  recursion/project boundaries, ordering, overlaps, and symlink policy.
- Explicit complete and partial caller contract; required/optional/repeated/
  unsupported artifacts; assembly keys; fatal and recoverable failure scope.
- Accepted configuration, top-level metadata, module, BSL declaration, payload,
  ownership, identity, and provenance mapping using existing public semantics.
- One non-empty cross-adapter conformance projection, controlled-change oracle,
  deliberate source-specific differences, determinism rules, first slice,
  implementation prerequisites, Coverage completion criteria, and deferred scope.
- Rejected alternatives, including EDT-tree conversion and whole-graph equality.

### Excluded

- Production Rust/Cargo changes, fixtures, parser/emitter implementation,
  support/Coverage transitions, release review, and prompt retirement.

## Acceptance Criteria

- ADR-0036 is accepted, internally consistent, and fully evidence-backed.
- Canonical identity is source-format independent; paths, producer identifiers,
  serialization order, BOM, and line endings are not semantic identity.
- Complete versus partial input is explicit and never inferred from absence.
- Parser and orchestration failure scopes prevent placeholder or partial facts
  from malformed required input.
- The conformance oracle cannot pass on empty or incomplete output and names all
  excluded dimensions.
- Existing graph/public APIs are sufficient or an exact prerequisite blocks
  implementation; no speculative model expansion is accepted.
- Sprint 14 remains incomplete and all later source families remain explicit.

## Repository Safety

Do not modify production code, Cargo files, fixtures, `.codex/`, prompt suites,
ignored corpora, or unrelated documentation. Stage only exact task-owned docs
when commit mode is authorized.

## Task-specific Validation

- Validate links, headings, ADR status, evidence citations, and Roadmap agreement.
- Verify accepted versus deferred scope and unchanged Sprint 14 state.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 14 Designer XML adapter contract`

## Final report additions

Report the accepted detector, completeness, assembly, mapping, provenance,
conformance, failure, deferred, and Coverage contracts; rejected alternatives;
implementation prerequisites; validation; commit; and Git state.
