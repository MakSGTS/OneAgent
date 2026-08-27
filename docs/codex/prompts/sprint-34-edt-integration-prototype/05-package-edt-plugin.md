# Package Sprint 34 EDT Integration Prototype

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
- committed Task 3-4 Maven, bundle, command, and test artifacts

## Prerequisites / required gate

Task 4 is committed and the complete client, command, lifecycle, real-process,
and PDE-host matrices pass.

## Task

Complete the accepted OSGi bundle, feature, category, and p2 repository package
and prove installable, removable, repeatable behavior in a disposable profile
based on the authorized EDT 2026.1 installation.

## Included scope

Exact bundle manifest/imports and Java execution environment; feature and
category metadata; deterministic Tycho repository assembly; package inventory;
two clean build comparison; disposable installation/configuration/workspace;
install, list, launch/test, uninstall and clean relaunch; positive and negative
command workflow; repeated activation/invocation; timeout/cancellation/stop;
logs and cleanup; and scripts or test harnesses that never modify the authorized
applications or read-only p2 pool.

## Excluded scope

External publication, signing, Marketplace metadata, bundled Runtime/JRE,
credentials, writes to authorized external paths, proprietary implementation
API, new user behavior, new dependencies, Rust/MCP changes, and final CI/docs.

## Acceptance criteria

- The produced repository contains only the accepted feature and bundles with
  exact metadata and no personal path, credential, JRE, Runtime, or test fixture.
- Installation and removal operate only on a disposable profile and leave the
  authorized installations and p2 pool unchanged.
- The installed command proves the accepted workflow in EDT 2026.1 with JDK 17
  and remains compatible with the JDK 25 Maven/PDE build.
- Clean repeated builds and host runs are deterministic under the accepted
  comparison rules.

## Validation

Run frozen clean Tycho build/test/package, exact repository inventory, two-build
comparison, install/list/host/uninstall/relaunch matrix, external-path snapshot
comparison, secret/path/generated/dependency/license audits, and
`git diff --check`.

## Suggested commit message

`Package Sprint 34 EDT integration prototype`

## Final report additions

Report artifact names and inventories, Java/BREE compatibility, exact install
and host commands/outcomes, disposable paths, external read-only compliance,
repeatability, and residual host constraints.
