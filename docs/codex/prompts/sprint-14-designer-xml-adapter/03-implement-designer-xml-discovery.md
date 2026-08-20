# Implement Sprint 14 Designer XML Discovery

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/source-adapter-implementation.md`

## Template

`docs/codex/templates/source-adapter-task.md`

## Authoritative documents

- `docs/adr/0036-designer-xml-adapter.md`
- `docs/architecture/designer-xml-source-investigation.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0005-edt-configuration-loading.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

Require committed Task 2 and a clean task-owned state. Treat ADR-0036 as fixed.

## Task

Implement only the accepted hierarchical Designer XML workspace detection,
configuration artifact assembly/loading, explicit build scope, and typed failure
boundary. Add the dedicated adapter crate only as required for this outcome.

## Source evidence / paired fixtures

Use exact real marker and configuration shapes from the registered corpus and
the smallest provenance-backed copies permitted by ADR-0036. Do not invent dump
markers or configuration XML.

## Scope

### Included

- Workspace-format detection, project boundaries, canonical ordering, marker
  conflicts, recursion, applicable overlap/symlink/unreadable behavior.
- Configuration.xml parsing for accepted UUID/name/payload, explicit complete or
  partial scope, and deterministic typed errors.
- Focused public production entry points and positive/negative/reordered/repeated
  tests; unchanged EDT detection.

### Excluded

- Top-level metadata enumeration, module parsing, graph contribution,
  conformance claims, Coverage transitions, and deferred source formats.

## Acceptance Criteria

- Accepted Designer roots produce `WorkspaceFormat::DesignerXml` and stop
  nested traversal; EDT roots remain unchanged.
- Missing, malformed, conflicting, unsupported, unreadable, and scope cases
  match ADR-0036 without guessed partiality or placeholder configuration.
- Configuration identity and accepted payload match the paired EDT source.
- Discovery and loading are independently testable and deterministic.
- No new production dependency is added without the existing workspace-approved
  dependency surface or explicit blocker handling.

## Repository Safety

Modify only the exact Cargo/workspace/filesystem/new-adapter paths required by
the Change Contract. Preserve `.codex/`, EDT implementation, corpora, and suites.

## Task-specific Validation

- Focused `oneagent-workspace`, `oneagent-workspace-fs`, and new-adapter tests.
- Assert every focused filter matches non-zero tests.
- Run the complete workspace validation gate from `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 14 Designer XML discovery`

## Final report additions

Report detector markers/conflicts, scope policy, configuration mapping, errors,
existing-adapter compatibility, exact tests, commit, and Git state.
