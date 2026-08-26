# IDE and Extension Integration Workflow

Use this workflow for editor extensions, extension-host lifecycle, packaging,
configuration, Runtime connectivity, user-visible state, and supported editor
integration evidence.

## Platform authority and ownership

- Identify the authoritative editor API, manifest, packaging, and test-runner
  versions before implementation and record their provenance.
- Keep extension-host code, editor API adaptation, Runtime client transport,
  semantic/domain behavior, and packaging in explicit ownership layers.
- Treat the extension as a client of accepted Runtime or protocol contracts; it
  must not duplicate semantic authority or redefine wire behavior.
- Do not use proposed editor APIs, undocumented manifest fields, or implicit
  global state unless accepted architecture explicitly includes them.

## Build, packaging, and compatibility

- Define the supported editor, Node.js, package-manager, TypeScript, and artifact
  compatibility ranges from accepted evidence rather than current workstation
  state.
- Keep lockfiles and generated-artifact policy explicit. Build and package from
  a clean tracked checkout and prove that required runtime files are present
  while source-only, secret, test, cache, and workspace artifacts are excluded.
- Keep cross-language validation visible when the extension launches or consumes
  a Rust executable. A passing Rust workspace does not prove the extension, and
  a passing extension build does not prove the Runtime binary.
- Do not claim Marketplace publication, signing, update behavior, or another
  editor until their dedicated evidence is included.

## Activation, configuration, and user-visible state

- Use the narrowest accepted activation events and prove activation and
  deactivation through public extension entry points.
- Define configuration keys, defaults, scopes, validation, change handling, and
  precedence before consuming them. Invalid or missing configuration must fail
  with a stable user-visible state and without exposing sensitive values.
- Register commands, views, status items, disposables, and subscriptions under
  the extension context owner. Dispose or terminate every resource on
  deactivation, failed activation, configuration replacement, and test cleanup.
- Derive UI status from owned lifecycle evidence rather than log text or an
  independently mutable label.

## Runtime connectivity and process lifecycle

- Define executable resolution, workspace root selection, environment and
  argument policy, process spawning, protocol framing, initialization, ready,
  failure, restart, and shutdown behavior from accepted architecture.
- Keep protocol output, extension diagnostics, and child-process diagnostics on
  their accepted channels. Bound retained output and redact paths, arguments,
  configuration, and source values from implicit error messages.
- Own every child process, stream, listener, timer, cancellation source, and
  pending request. Reject or terminate pending work deterministically when the
  connection fails or the extension deactivates.
- Prove repeated activation/deactivation and reconnection without orphaned
  processes, duplicate registrations, leaked listeners, or timing-dependent
  sleeps.

## Integration evidence

- Separate pure unit tests for configuration and transport state from extension-
  host tests that exercise the public editor API and from real-process tests that
  exercise the supported Runtime boundary.
- Cover positive, missing, invalid, incompatible, startup-failure, unexpected-
  exit, malformed-response, repeated, reordered, and cleanup cases as applicable.
- Pin or otherwise make the editor test runtime reproducible; a test that
  downloads an unspecified latest editor build is not stable acceptance evidence.
- Run non-zero extension build, unit, extension-host, packaging, and public
  Runtime connectivity checks for every corresponding claim.

## Boundary

This workflow does not select an editor version, package manager, compiler,
bundler, test runner, packaging tool, activation event, command catalog,
configuration schema, Runtime executable, transport, restart policy, UI design,
or first production slice. Those decisions belong to accepted ADRs or the
current task. It does not require navigation, LSP, diagnostics, chat, EDT,
Marketplace publication, telemetry, remote/web extension hosts, or another
editor unless explicitly included.
