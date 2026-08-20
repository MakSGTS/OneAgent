# Task 02: Parse XDTO Package schemas

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

- `docs/Roadmap.md`, Sprint 13 Task 02;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Required gate

Proceed only when Task 01 is committed and its public XDTO node/payload/
identity contract is the live baseline. No task-created uncommitted change may
remain.

## Task

Implement one deterministic typed EDT parser that joins an already discovered
XDTO Package descriptor to its exact `Package.xdto` artifact and preserves only
the accepted direct Value/Object type slice plus typed deferred observations.
Do not emit graph facts.

## Source evidence / fixtures

Use the 20 live pairs under `OneAgent_EDTproject/src/XDTOPackages/` and the
exact inventory in the source investigation. Positive test evidence must copy
or reduce real XML shapes into test-local fixtures with provenance comments or
use deterministic read-only live-audit tests that skip no required evidence.
Generated XML may cover malformed and boundary cases.

## Scope

One XDTO descriptor/schema parser and its focused tests.

## Included

- Reuse the existing descriptor's UUID, name, synonym, kind, and descriptor
  path; do not create another top-level reader.
- Require exactly one `Package.xdto` in the XDTO Package object directory.
- Validate exact schema root/namespace, required target namespace, and equality
  with the descriptor namespace parsed from the exact metadata root.
- Parse direct `valueType` and `objectType` required names into canonical typed
  declarations sorted by exact name.
- Reject duplicate names across both direct type families deterministically.
- Preserve direct imports and every nested enumeration/property/pattern/typeDef
  occurrence as typed deferred counts/observations sufficient for production
  diagnostics and provenance without inventing child identities.
- Define typed filesystem, unreadable, malformed XML, wrong root/namespace,
  missing/extra/ambiguous artifact, missing/empty/duplicate name, and namespace
  mismatch errors with deterministic paths and ordering.
- Prove filesystem/XML reorder and repeated-read equality.

## Excluded

- HTTP or Web Service parsing.
- Graph nodes/edges, public requests, resolution, provenance emission,
  diagnostics/statistics projection, or Coverage changes.
- XDTO property/import/base/restriction/enum semantics, QName resolution,
  dependency edges, external nodes, or Designer XML.
- New dependencies or generic XML DOM exposure in public APIs.

## Acceptance criteria

- All 20 live descriptor/artifact namespaces match and all 12,666 direct type
  declarations parse with exact family/name and no ordinal identity.
- Representative one-type, mixed small, and large package shapes are covered;
  the parser is streaming/bounded according to existing quick-xml conventions
  and does not require graph construction.
- Missing directory/artifact, ambiguous extra artifact, unreadable input,
  malformed XML, wrong root/namespace, missing/empty target namespace,
  descriptor/artifact mismatch, missing/empty direct name, and duplicate name
  across same/different direct families return exact typed errors.
- Direct imports and 61,435 nested properties remain deferred observations and
  never become accepted types, Unknown values, placeholders, or ordinals.
- Unknown direct elements have a deterministic typed deferred or error outcome
  consistent with ADR-0035; they are never silently accepted as types.
- Source and filesystem reordering and repeated reads return equal canonical
  descriptors and errors.
- Existing generic metadata-object behavior remains unchanged and full
  workspace validation succeeds.

## Repository Safety

- Recheck Git state, exact live XDTO paths, quick-xml conventions, generic
  reader consumers, tests, and applicable `AGENTS.md` before editing.
- Preserve ignored live source and the two unrelated untracked user files; do
  not stage copied full production schemas.
- Do not change graph behavior, service parsers, builder emission, Coverage,
  docs, prompts, or Cargo dependencies.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib xdto_package::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Treat a zero-match
filter as missing evidence.

## Suggested commit message

```text
Parse Sprint 13 XDTO package schemas
```

When authorized, stage only the task-owned XDTO parser module, focused parser
tests, and any necessary crate-local module export. Do not stage graph changes,
service work, production fixtures, Coverage/docs, prompts, ignored live files,
or unrelated paths; do not create an empty commit.

## Final report additions

Report source corpus evidence, parser API/model, exact accepted/deferred/error
inventory, live counts, ordering/memory behavior, files/tests, validation,
commit hash, final Git status, and the Task 03 gate.
