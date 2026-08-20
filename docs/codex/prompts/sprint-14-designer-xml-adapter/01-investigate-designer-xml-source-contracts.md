# Investigate Sprint 14 Designer XML Source Contracts

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 14 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/architecture/edt-source-corpora.md`
- `docs/reviews/sprint-13-xdto-service-model.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0005-edt-configuration-loading.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0008-edt-metadata-object-reader.md`

## Prerequisites / Required gate

Require the committed Sprint 14 planning baseline and clean task-owned state.
The registered ignored Designer and paired EDT corpora must be locally readable;
otherwise stop because this investigation cannot invent source evidence.

## Investigation objective

Create `docs/architecture/designer-xml-source-investigation.md` with the exact
repository evidence needed to decide ADR-0036 and implement the smallest safe,
testable Designer XML adapter slice.

## Questions to answer

- Which root markers, namespaces, format/version values, and conflict cases
  distinguish a hierarchical Designer dump from EDT, flat XML, and incomplete
  directories?
- What is the configuration boundary, canonical enumeration order, traversal
  boundary, symlink/overlap behavior, and complete-versus-partial evidence?
- Which files are configuration, top-level metadata, module, or unsupported
  artifacts; what exact names, roots, joins, UUID/name/synonym values, and module
  roles do they use?
- Which accepted top-level kinds and module layouts have paired EDT evidence,
  and which canonical identities/payloads/ownership/declarations can match?
- What malformed, missing, duplicate, reordered, unknown, unreadable,
  incompatible, ambiguous, and conflicting cases can be derived safely?
- What smallest provenance-backed paired fixture and non-empty conformance
  projection can detect a controlled semantic change?
- Which public APIs, consumers, tests, Cargo surfaces, Coverage registries, and
  documentation would implementation affect?

## Evidence scope

Inspect live files under `OneAgent_DesignerXML/` and the corresponding
`OneAgent_EDTproject/src/` artifacts, current workspace/filesystem/EDT adapters,
graph and BSL public APIs, fixtures, tests, Coverage, history, and consumers.
Record exact representative paths, serialized vocabulary, counts, raw hashes,
normalization rules, and deliberate adapter-specific differences. Separate
confirmed evidence, accepted constraints, candidate decisions, and unknowns.

## Scope

### Included

- The investigation document only.
- Source matrices, negative-case inventory, conformance candidate, fixture
  provenance candidates, implementation surface, risks, and decision readiness.

### Excluded

- Architecture acceptance, production code, Cargo changes, fixtures, Coverage
  transitions, support claims, ignored-corpus changes, and prompt retirement.

## Acceptance Criteria

- Every proposed parser field, path, join, identity input, negative case, and
  oracle dimension is backed by an exact repository source.
- The document identifies the smallest coherent first slice and all deliberately
  deferred source families.
- The evidence is sufficient for ADR-0036 or records an exact blocker; it does
  not hide an external-data requirement as future implementation work.
- Ignored corpora remain unmodified and are not made a CI dependency.

## Repository Safety

Preserve `.codex/`, production code, existing suites, unrelated files, and the
ignored corpora. Stage only the investigation document when commit mode is
authorized.

## Task-specific Validation

- Recompute every cited representative SHA-256 value and count.
- Verify every cited path and repository link.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Investigate Sprint 14 Designer XML source contracts`

## Final report additions

Report confirmed detector/artifact/module/paired evidence, accepted constraints,
unknowns, first-slice readiness, exact commands, changed paths, commit, and final
Git state.
