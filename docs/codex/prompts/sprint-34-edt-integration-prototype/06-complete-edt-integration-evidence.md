# Complete Sprint 34 EDT Integration Evidence

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
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`
- `.github/workflows/ci.yml`
- committed Task 3-5 code, tests, package, scripts, and host evidence

## Prerequisites / required gate

Task 5 is committed and every focused unit, real-process, PDE-host, package,
install/uninstall, EDT-host, lifecycle, and repeatability check passes.

## Task

Complete cross-platform build, compatibility, package, security, scope, and
current-state evidence for the accepted Sprint 34 prototype without changing
its production behavior.

## Included scope

Frozen Maven/Tycho inputs; JDK 25 clean build and Java 17 BREE checks; complete
unit/process/PDE/EDT host matrices; exact p2 package inventory and repeatability;
macOS and Windows CI build/test/package coverage without ITS secrets; documented
optional Maven authenticated official-p2 configuration and local read-only-pool
workflow with placeholders only; Runtime/MCP and VS Code/LSP compatibility;
Architecture, Semantic Model, Roadmap, EDT README, provenance, dependency,
license and support-boundary synchronization; source/API/manifest/catalog/
dependency/license/secret/path/generated/deferred-scope/link/prompt audits.

## Excluded scope

New production behavior or dependency, external publication, signing,
credentials, bundled JRE/Runtime, proprietary EDT implementation API, semantic
UI, remote/multi-project support, telemetry, and broad unsupported claims.

## Acceptance criteria

- Every ADR-0056 criterion has non-zero public evidence or an explicitly
  classified authorized-local EDT-host result.
- API, command registration, Runtime request, tests, feature/repository,
  installation, CI and documentation agree exactly.
- No secret or personal absolute path is tracked; authenticated setup uses only
  Maven server IDs and environment/settings placeholders.
- Existing Rust Runtime/MCP and IDE clients remain compatible and the complete
  repository Definition of Done passes.

## Validation

Run frozen clean Maven build, all Java unit/process/PDE/EDT host tests, package
inventory and two-build verification, install/uninstall checks, focused
Runtime/MCP and IDE compatibility, canonical Rust workspace format/check/test/
clippy/rustdoc, CI syntax/coverage, and every audit listed above.

## Suggested commit message

`Complete Sprint 34 EDT integration evidence`

## Final report additions

Report exact commands/counts, CI platforms, supported user journey, package and
installation results, JDK/architecture boundary, p2/auth handling, dependencies,
documentation transitions, compatibility, exclusions, and remaining risks.
