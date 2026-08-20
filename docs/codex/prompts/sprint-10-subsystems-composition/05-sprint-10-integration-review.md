# Task 05: Review the integrated Sprint 10 baseline

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, tests, errors,
  public APIs, prompt text, review artifacts, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 10 execution plan and Task 05;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`;
- the committed Sprint 10 prompt suite.

## Required gate

Proceed only when the accepted Sprint 10 planning baseline and Tasks 01–04 are
committed or proven `already_complete`, all required implementation validation
passed, and no task-created uncommitted change remains.

## Review target

Review the exact committed range from the parent of the Sprint 10 planning
commit through the committed Task 04 head. Do not rely on prompt claims or
commit subjects as implementation proof.

## Reviewed baseline / commit or diff range

Resolve exact full hashes from live Git history. Verify commit order, subjects,
owned paths, prerequisite chain, and final Task 04 repository state before
reviewing behavior.

## Scope

- Every ADR-0032 source, parser, graph endpoint, identity, ownership, cycle,
  provenance, query, determinism, Coverage, compatibility, and documentation
  criterion.
- Exact agreement among `subsystems`, `parentSubsystem`, and physical nesting.
- Positive, negative, malformed, missing, duplicate, inconsistent, cyclic,
  escaped, reordered, repeated-build, and index-equivalence evidence.
- Repository-proven depth, duplicate local names, nested direct content, and
  deferred Subsystem content behavior.
- Direct generic Includes and computed transitive membership without persisted
  closure.
- Query, Diff, Impact, reports, validation, complete and incremental Semantic
  Index behavior.
- Full workspace Definition of Done, repository safety, previous-suite
  inventory, and Sprint 11 eligibility.

## Excluded

- Silent fixes to implementation, tests, ADRs, prompts, or source fixtures.
- New architecture, content prefixes, command-interface behavior, dependency or
  Impact propagation, aliases, cross-project hierarchy, recovery policy,
  persistence, runtime APIs, or Sprint 11 planning.
- Refactoring or cleanup unrelated to a review finding.
- Deletion of the current Sprint 10 suite, bootstrap prompt, older non-adjacent
  suite, untracked file, or any path outside the verified Sprint 9 suite.

## Review criteria

- Architecture conformance and one direct hierarchy/composition meaning.
- Deterministic UUID and edge identity with unchanged top-level compatibility.
- Strict, typed source-agreement parsing without inference or partial repair.
- Precise endpoints, cycle detection, production emission, provenance,
  diagnostics/statistics, and no placeholders.
- Exact direct and transitive Query semantics with no persisted closure.
- Generic consumer and complete/incremental index equivalence.
- Includes exclusion from dependency/Impact behavior.
- Truthful Coverage status/counts and current-state documentation.
- Full deferred-scope and repository-safety conformance.
- No blocking correctness, regression, missing-evidence, or documentation
  finding.

## Acceptance evidence matrix

Record pass/fail evidence for: planning/commit chain; source inventory and
provenance; parser agreement; malformed/missing/duplicate/mismatch/cycle/escape
outcomes; graph endpoint and cycle validation; node/edge identity; ownership;
direct hierarchy; nested content; provenance; generic direct Query; transitive
membership; no stored closure; Diff; Impact; reports; validation; complete
index; incremental equivalence; Coverage; documentation; top-level and unrelated
behavior; workspace gate; and deferred scope.

## Authorized review outputs and state transition

When and only when the decision is `pass` or `pass with non-blocking
follow-ups`, create `docs/reviews/sprint-10-subsystems-composition.md`, update
`docs/Roadmap.md` to mark Sprint 10 `completed` and Sprint 11 `next`, and
synchronize the final hand-off statement. Do not change implementation files.

For `blocked`, create no completion transition, prompt retirement, or partial
review commit. Report the finding and leave Sprint 10 incomplete.

## Verified previous-suite retirement

Only after a non-blocking decision and every focused/full validation command
succeeds, re-enumerate and compare the live tracked inventory with exactly:

```text
docs/codex/prompts/sprint-9-roles-access-rights/00-sprint-9-execution-loop.md
docs/codex/prompts/sprint-9-roles-access-rights/01-implement-conditional-access-right-model.md
docs/codex/prompts/sprint-9-roles-access-rights/02-emit-conditional-role-grants.md
docs/codex/prompts/sprint-9-roles-access-rights/03-complete-sprint-9-production-evidence.md
docs/codex/prompts/sprint-9-roles-access-rights/04-sprint-9-integration-review.md
```

If and only if the inventory matches, no untracked file is endangered, and no
current Sprint 10 link depends on these files, explicitly delete these five
tracked files through the normal patch mechanism. Do not use recursive deletion
or globs. Keep the complete Sprint 10 suite and
`docs/codex/prompts/run-next-sprint.md` untouched. Stage the exact deletions,
review artifact, Roadmap transition, and any explicitly required final Semantic
Model state together in the single review commit. Any mismatch blocks deletion
and the final review commit.

## Task-specific validation

Run focused review checks:

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib subsystem_hierarchy::tests
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the canonical complete workspace validation:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Record exact results and treat zero matched filters as missing evidence. After
the authorized state/deletion change, rerun `git diff --check` and manually
validate prompt/link inventories before committing.

## Suggested commit message

```text
Complete Sprint 10 subsystems and composition review
```

When authorized and the decision is non-blocking, stage only the review
artifact, Roadmap/current-state transition, and the five exact verified Sprint 9
prompt deletions. Never create a review commit for a blocked decision.

## Final report additions

Report the exact reviewed range, acceptance matrix, findings by severity,
missing evidence, scope/exclusion conformance, validation, decision, review
artifact, state transition, every retired path, retained Sprint 10 suite,
commit hash, final Git status, and Sprint 11 eligibility.
