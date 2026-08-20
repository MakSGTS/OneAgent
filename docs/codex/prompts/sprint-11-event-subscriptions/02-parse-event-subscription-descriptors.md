# Task 02: Parse Event Subscription descriptors

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

- `docs/Roadmap.md`, Sprint 11 Task 02;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Required gate

Proceed only when Task 01 is committed and its Event Subscription metadata,
payload, References, Triggers, enum-consumer, and full validation evidence is
present without task-created uncommitted changes.

## Task

Implement a deterministic typed EDT Event Subscription descriptor parser from
repository-proven XML. Return source facts and typed parser outcomes only; do
not perform semantic resolution or emit graph nodes or edges.

## Source evidence / fixtures

- The root-ignored `OneAgent_EDTproject/src/EventSubscriptions/` corpus has 99
  descriptors, 314 `types` entries, 18 event values, and 93 unique handlers.
- `Catalogs_BeforeWrite/Catalogs_BeforeWrite.mdo` proves a bare source family.
- `AccountingRules_Posting/AccountingRules_Posting.mdo` proves 30 qualified
  Document selectors.
- `CheckSafeModeBeforeWrite/CheckSafeModeBeforeWrite.mdo` proves a large source
  list and optional comment that remains deferred.
- `OnReceiveDataFromMaster/OnReceiveDataFromMaster.mdo` proves multilingual
  synonym entries and a platform event name.
- Generated test artifacts may cover malformed cases. The tracked production
  fixture is owned by Task 05, so do not commit ignored live artifacts.

## Scope

One parser boundary that preserves stable descriptor content and ordered source
observations suitable for later resolution.

## Included

- Add a focused `event_subscription` module or the smallest equally scoped
  reader extension justified by current code.
- Locate exactly one `.mdo` descriptor through existing top-level reader
  conventions; require the Event Subscription root namespace and kind.
- Parse UUID, canonical name, existing localized synonym behavior, one direct
  source with at least one `types`, one non-empty event, and one non-empty
  handler.
- Preserve exact XML-decoded event and source/handler spelling; do not trim,
  case-fold, localize, alias, or synthesize missing values.
- Parse source selectors into one- or two-component typed observations, retain
  occurrence ordinal/context, and classify supported prefix, unsupported
  prefix, and malformed grammar without graph knowledge.
- Parse handler into exactly `CommonModule.<module>.<procedure>` with three
  non-empty components; do not validate existence or export status.
- Canonicalize returned observations independently from filesystem/XML order
  while retaining duplicate occurrence evidence.
- Return typed fatal errors for missing/duplicate required direct fields,
  empty source, invalid UUID/name/event, wrong root, malformed XML, ambiguous
  descriptor, or unreadable file.
- Add real-shape positive tests and generated absent, duplicate, malformed,
  unsupported, reordered, and repeated-read tests.

## Excluded

- Metadata, Module, or Procedure lookup; resolution outcomes; graph node,
  payload, ownership, References, or Triggers emission.
- Diagnostics/statistics projection and ADR-0024 request-ledger migration.
- Semantic interpretation of comments, source families, event vocabulary,
  handler signatures, export status, aliases, or case-insensitivity.
- Production fixture/Coverage completion or unrelated metadata readers.

## Acceptance criteria

- Representative live descriptors with 1, 30, 41, and 94 source entries parse
  to exact UUID/name/synonym/event/handler and typed source observations.
- Present, absent, non-ASCII, and multilingual synonym behavior remains
  compatible with the existing generic reader contract.
- All 18 observed event values are preserved as non-empty typed payload inputs,
  not rejected by an invented enum.
- All supported and unsupported observed prefixes are classified exactly as
  ADR-0033; one- and two-component values remain distinct.
- Exported and non-exported handler paths parse identically; Function or
  existence policy is not decided by the parser.
- Empty/missing/duplicate direct fields, empty source, bad UUID/name, wrong
  root, malformed selector depth/components, bad handler namespace/depth,
  multiple descriptor files, and unreadable input produce deterministic typed
  outcomes.
- Reordered selectors preserve semantic equality and duplicate evidence;
  repeated reads are identical.
- No graph, diagnostics/statistics, production, or Coverage behavior changes.

## Repository Safety

- Recheck Git state, reader APIs/usages, XML fixtures, ignored-source status,
  tests, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the committed prompt suite.
- Do not add dependencies; use existing quick-xml and filesystem facilities.
- Do not stage ignored live project artifacts.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib event_subscription::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Parse Sprint 11 event subscription descriptors
```

When authorized, stage only the task-owned EDT parser/source model, module
exports, focused tests, and small generated fixtures if necessary. Do not stage
graph resolution/emission, planning prompts, ignored live artifacts, or
unrelated paths; do not create an empty commit.

## Final report additions

Report source evidence, parsed contract, field/error policy, supported versus
unsupported selector classification, handler grammar, ordering/duplicates,
fixture usage, files/tests, validation, commit hash, final Git status, and the
Task 03 gate.
