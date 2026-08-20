# Task 03: Parse HTTP and Web Service descriptors

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/parser-implementation.md`

## Template

`docs/codex/templates/parser-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 13 Task 03;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Required gate

Proceed only when Tasks 01–02 are committed, the graph contract is live, the
XDTO parser proves exact package namespace/direct-type input, and no task-created
uncommitted change remains.

## Task

Implement deterministic typed HTTP Service and Web Service descriptor parsers
for the exact ADR-0035 declaration structure. Preserve package, type, and
callable reference expressions without resolving or emitting graph facts.

## Source evidence / fixtures

Use the live two HTTP and eight Web Services under
`OneAgent_EDTproject/src/{HTTPServices,WebServices}/` and the source
investigation inventory. Preserve representative real shapes in focused test
fixtures; generated XML may cover negative cases.

## Scope

One coherent service-descriptor parser boundary sharing common typed XDTO
declaration parsing where justified.

## Included

- Reuse already discovered top-level UUID/name/synonym/kind/path and validate
  exact service roots/namespaces.
- Parse HTTP root URL, UUID-backed direct URL Templates with exact template
  text, and UUID-backed nested Methods with optional explicit HTTP method and
  required handler name.
- Parse Web namespace, optional typed `xdtoPackages` Reference/String value,
  UUID-backed direct Operations, UUID-backed nested Parameters, return/value
  `(nsUri, name)` declarations, optional Boolean nillability, optional accepted
  `Out`/`InOut` transfer direction, and required Procedure name.
- Preserve internal package grammar, external namespaces, internal/external
  type declarations, and handler names as typed unresolved parser output.
- Canonicalize children by stable UUID and reject duplicate/conflicting UUIDs
  or owner-local names according to ADR-0035.
- Define typed missing/empty/duplicate/malformed/invalid Boolean, invalid
  package grammar/type wrapper, unsupported explicit direction, wrong hierarchy,
  wrong root/namespace, and XML/filesystem errors.
- Prove source/XML/filesystem reordering and repeated-read equality.

## Excluded

- XDTO schema parser changes, graph insertion, payload enrichment, public
  request creation, resolution, References/Triggers, diagnostics/statistics,
  tracked production fixture, or Coverage.
- Inferred HTTP verbs, route grammar/parameters, sessions, descriptor/WSDL,
  transport, publication, runtime behavior, external schema resolution, or BSL
  body analysis.
- New dependencies or parallel top-level metadata identity.

## Acceptance criteria

- The live corpus parses exactly 2/35/35 HTTP Service/URL Template/Method and
  8/119/360 Web Service/Operation/Parameter declarations with stable UUIDs.
- All 154 handler/procedure expressions are retained exactly without parser-
  time symbol resolution.
- Two internal package references, five external package namespaces, one absent
  package declaration, 478 external type occurrences, and one internal type
  occurrence have exact typed classifications.
- Explicit/absent HTTP method, explicit/absent nillability, and explicit
  `Out`/`InOut`/absent direction remain distinguishable; no source default is
  guessed.
- Missing required fields, empty values, duplicate UUID/name, conflicting
  wrapper type, malformed `XDTOPackage.<name>`, invalid Boolean/direction,
  wrong root/namespace/hierarchy, malformed XML, and unreadable input produce
  exact deterministic outcomes.
- Reordered siblings/filesystem traversal and repeated reads yield equal
  canonical parser output and error ordering.
- No graph, request, diagnostic, statistics, or Coverage fact is emitted, and
  existing metadata/module behavior plus full workspace validation succeeds.

## Repository Safety

- Recheck Git state, live service paths, generic reader/module behavior,
  quick-xml conventions, tests, and applicable `AGENTS.md` before editing.
- Preserve ignored live source and unrelated user files; do not stage full live
  descriptors/modules.
- Do not change graph behavior, XDTO parser semantics, builder emission,
  Coverage, docs, prompts, or dependencies.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib service_descriptor::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Treat a zero-match
filter as missing evidence.

## Suggested commit message

```text
Parse Sprint 13 HTTP and Web service descriptors
```

When authorized, stage only task-owned service parser code, focused tests, and
necessary crate-local module exports. Do not stage production emission,
fixtures, Coverage/docs, prompts, ignored live files, or unrelated paths; do
not create an empty commit.

## Final report additions

Report live source inventory, parser APIs, accepted/optional/error contracts,
package/type/handler classification counts, ordering, files/tests, validation,
commit hash, final Git status, and the Task 04 gate.
