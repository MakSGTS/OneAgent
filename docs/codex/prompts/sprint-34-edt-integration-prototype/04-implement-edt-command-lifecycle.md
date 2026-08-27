# Implement Sprint 34 EDT Command Lifecycle

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/ide-extension-implementation.md`
- `docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0056-edt-integration-prototype.md`
- `docs/architecture/edt-integration-prototype-investigation.md`
- `docs/adr/0052-vscode-extension-foundation.md`

## Prerequisites / required gate

Task 3 is committed and the complete Runtime-probe unit and real-process matrix
passes.

## Task

Implement the accepted native Eclipse command, exact EDT project eligibility
gate, executable configuration, background execution, visible outcome, and
bundle lifecycle ownership over the committed Runtime client.

## Included scope

Public Eclipse command/menu/property APIs accepted by ADR-0056; exact single
selection adaptation; local/open/accessibility/nature/location validation;
bounded executable configuration; stable enablement and handler behavior;
owned background job; UI-thread result publication; accepted invocation
serialization or rejection; timeout/cancellation; configuration-change and
bundle-stop invalidation; activator/disposable ownership; stable localized or
declared messages; pure controller/adaptation seams; unit tests; and PDE host
tests with public observable behavior.

## Excluded scope

Semantic navigation/search/context/chat/diagnostics, BSL editor integration,
source access, persistent Runtime, automatic activation/start, multiple
projects, remote/virtual workspaces, feature/repository packaging, external
publication, new dependencies, Rust/MCP changes, and final docs/CI.

## Acceptance criteria

- The command is available only for exactly one accepted EDT configuration
  project and uses that project location as the Runtime cwd.
- Blocking work never runs on the UI thread; stale or cancelled work cannot
  publish success after replacement or disposal.
- Every handler, job, process, listener and UI callback is owned and cleaned up.
- Positive, unsupported, invalid configuration, spawn, protocol, timeout,
  cancellation, repeated invocation, configuration-change and stop cases have
  deterministic non-zero evidence.

## Validation

Run complete non-zero command/controller/configuration/lifecycle unit tests,
real Runtime process regressions, PDE host tests, Maven clean verification,
manifest/API/dependency/generated-artifact audits, and `git diff --check`.

## Suggested commit message

`Implement Sprint 34 EDT command lifecycle`

## Final report additions

Report command and configuration identifiers, eligibility rules, threading,
visible outcomes, lifecycle ownership, Host cases, exact counts, and preserved
Runtime/editor behavior.
