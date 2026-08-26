# IDE Extension Task Template

## Purpose

Use this template for one accepted editor-extension build, packaging,
activation, configuration, Runtime-connectivity, UI-state, or extension-host
lifecycle implementation slice.

## Recommended profile

- `docs/codex/profiles/ide-extension-implementation.md`

## Required task-specific sections

- Authoritative editor API, manifest, toolchain, packaging, ADRs, and
  architecture documents
- Prerequisites / required gate
- Task
- Platform and version compatibility
- Build, dependency, lockfile, and generated-artifact policy
- Packaging contents and exclusions, when applicable
- Activation, registration, and extension-context ownership
- Configuration schema, precedence, validation, and change behavior
- Runtime executable, workspace, transport, and process ownership, when
  applicable
- Connection states, failures, cancellation, restart, and shutdown behavior
- User-visible state and diagnostic redaction
- Integration-test layers and deterministic oracles
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Use only stable editor APIs and compatibility behavior established by accepted
  architecture or the task scope.
- Keep extension-host adaptation separate from protocol, Runtime, and domain
  ownership; do not recreate semantic behavior in TypeScript.
- Track every command, UI object, listener, stream, process, timer, and pending
  request under an explicit disposable or lifecycle owner.
- Make invalid configuration, startup failure, protocol failure, unexpected
  process exit, deactivation, and repeated activation observable and bounded.
- Prove packaged contents from a clean build and exclude secrets, caches, tests,
  local workspaces, and unrelated repository artifacts.
- Use deterministic unit, extension-host, and real-process evidence as
  applicable; arbitrary sleeps and unspecified latest editor downloads are not
  acceptance evidence.

## Additional report sections

- Platform and toolchain authority
- Build and packaged-artifact evidence
- Activation and disposable ownership
- Configuration and user-visible state
- Runtime connection and process lifecycle
- Cross-language and extension-host validation
- Compatibility and deferred editor scope

## Additional validation

- Run non-zero TypeScript typecheck, build, and unit tests for changed extension
  code.
- Run non-zero extension-host tests when activation, VS Code API registration,
  configuration integration, disposables, or UI state is claimed.
- Run non-zero public Runtime process tests when executable discovery, spawning,
  MCP framing, initialization, request handling, failure, or shutdown is claimed.
- Build the distributable package and inspect its exact inventory when packaging
  is in scope.
- Run Rust validation required by `docs/codex/core/validation.md` when Rust,
  Cargo manifests, public APIs, Runtime behavior, or protocol behavior changes.
