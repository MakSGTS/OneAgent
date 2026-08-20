# Task 02: Parse nested Subsystem hierarchy

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

- `docs/Roadmap.md`, Sprint 10 Task 02;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`.

## Required gate

Proceed only when Task 01 is committed and its additive Includes endpoint,
cycle validation, transitive Query, and full validation are present in the live
baseline without task-created uncommitted changes.

## Task

Implement a deterministic recursive EDT Subsystem hierarchy source model that
requires exact agreement among parent `subsystems`, child `parentSubsystem`,
and immediate physical nesting. Parse and return source facts only; do not emit
semantic graph nodes or edges.

## Source evidence / fixtures

- The bootstrap-selected live `OneAgent_EDTproject/src/Subsystems/` corpus
  contains 127 descriptors: 13 top-level and 114 nested through depth 5. The
  root `.gitignore` excludes this tree, so recheck it as source evidence rather
  than treating it as a committed fixture.
- `OneAgent_EDTproject/src/Subsystems/DNSCore/DNSCore.mdo` declares direct
  children with `<subsystems>`.
- `OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Common.mdo`
  declares `<parentSubsystem>Subsystem.DNSCore</parentSubsystem>`.
- The ReceiptPrinters descriptor cited by the source investigation proves a
  multi-component qualified parent.
- Six duplicate local names under different parents prove that local name alone
  is not an identity or resolution key.
- Generated test trees may cover malformed cases. A later tracked reduced
  fixture with documented derivation must provide the immutable production
  oracle; the live ignored EDT artifacts provide source vocabulary here.

## Scope

One parser/discovery boundary that produces ordered nested descriptor and
direct parent-child observations suitable for later graph emission.

## Included

- Add a focused `subsystem_hierarchy` source module or the smallest equally
  scoped existing-reader extension supported by live code.
- Begin from the existing top-level Subsystems directory and recurse only
  through direct declared `Subsystems/<Name>` children.
- Parse only direct `subsystems` and direct `parentSubsystem` fields with the
  exact namespace/root and qualified grammar accepted by ADR-0032.
- Preserve each descriptor UUID, name, descriptor path, ancestor path, immediate
  parent identity, raw declarations, and inputs needed by the existing content
  reader.
- Sort output independently from filesystem and XML order.
- Reuse the existing metadata-object and direct-content readers where doing so
  preserves their contracts; avoid a competing XML authority.
- Return typed fatal errors for missing/extra/duplicate declarations or child
  directories, malformed qualified parents, mismatched paths/parents, invalid
  names, self-parent/cycles, descriptor failures, unreadable paths, and project-
  root escapes.
- Add positive real-source tests and generated positive, negative, duplicate-
  name, reordered, and repeated-read cases.

## Excluded

- Graph insertion, hierarchy/content Includes emission, provenance, semantic
  resolution, diagnostics/statistics projection, Query, Diff, or index changes.
- Directory-only inference, one-projection precedence, recovery/repair, partial
  successful hierarchy output, symlink escape, aliases, localization, or case
  folding.
- Semantic meaning for `Subsystem.<...>` content tokens.
- Command-interface parsing, configuration inventory, unrelated metadata
  families, Coverage changes, or documentation completion.

## Acceptance criteria

- The live source descriptors at depths 1–5 parse to 127 unique UUID-backed
  descriptors and 114 direct hierarchy observations when rechecked.
- Every real nested relation proves agreement among all three projections.
- Duplicate local names under different parents remain distinct by UUID and
  full ancestor path.
- Top-level parent absence is valid; nested missing/multiple parent fields are
  typed errors.
- Missing/extra/duplicate child declarations or directories, malformed tokens,
  wrong prefixes, path mismatch, self-parent, cycles, descriptor ambiguity,
  unreadable paths, and escapes have deterministic typed outcomes.
- Reordered declarations/directories and repeated reads return equal ordered
  source models.
- No graph, Coverage, or production emission behavior changes.

## Repository Safety

- Recheck Git state, reader APIs, consumers, fixtures, tests, and applicable
  `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the committed prompt suite.
- Do not add dependencies without explicit approval; use the existing XML and
  filesystem facilities.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib subsystem_hierarchy::tests
cargo test -p oneagent-edt --lib metadata_object::tests
cargo test -p oneagent-edt --lib subsystem_content::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Parse Sprint 10 nested subsystem hierarchy
```

When authorized, stage only task-owned EDT parser/discovery code, its focused
tests, and repository-owned fixture files proven necessary. Do not stage graph
emission, planning prompts, or unrelated files; do not create an empty commit.

## Final report additions

Report source evidence, parsed contract, hierarchy agreement/error policy,
fixture provenance, ordering, unsupported/unknown cases, files/tests,
validation, commit hash, final Git status, and the Task 03 gate.
