# ADR-0052: VS Code Extension Foundation

## Status

Accepted

## Context

Sprint 30 must establish extension packaging, demand activation, bounded
configuration, one owned Runtime connection, observable status, and cleanup
without implementing later IDE features or changing the accepted Runtime and
MCP server. Repository and pinned upstream evidence is recorded in
`docs/architecture/vscode-extension-foundation-investigation.md`.

The repository has no existing Node package or extension implementation. The
public `oneagent-mcp` process is already stateless, newline-framed, sequential,
and bounded by ADR-0050/0051. It deliberately does not implement standard MCP
`initialize` or sessions. The extension must adapt to that committed boundary
rather than introduce a second protocol owner.

## Decision

### Canonical statement and ownership

OneAgent adds one desktop VS Code workspace extension at `extensions/vscode`.
It is a thin client and lifecycle adapter around the existing `oneagent-mcp`
stdio process. TypeScript owns VS Code API registration, configuration
validation, child-process client state, request correlation, and user-visible
connection state. Rust remains the only Runtime, protocol, workspace, graph,
analysis, Tool Policy, and semantic authority.

The extension imports no Rust internals and reimplements no semantic query,
validation, diagnostic, impact, or context behavior. It validates only the
closed client-side fields needed to prove server compatibility, framing,
bounds, request identity, and lifecycle safety.

### Package identity and supported host

The package contract is:

| Field | Value |
|---|---|
| Directory | `extensions/vscode` |
| `name` | `oneagent` |
| `publisher` | `oneagent-dev` |
| Extension ID | `oneagent-dev.oneagent` |
| Version | `0.1.0` |
| License | `Apache-2.0` |
| `engines.vscode` | `^1.134.0` |
| Test editor | exactly `1.134.0` |
| Entry point | `./dist/extension.js` |
| Extension kind | `workspace` |
| Runtime | desktop Node.js extension host |

The first slice supports desktop VS Code with exactly one trusted file-backed
workspace folder. Empty windows, virtual workspaces, untrusted workspaces,
multi-root fan-out, web hosts, and remote-host compatibility claims are
unsupported. The manifest declares virtual and untrusted workspaces
unsupported so VS Code prevents activation in those modes.

The package can be built and installed locally as a VSIX. The `oneagent-dev`
publisher value is a deterministic package identity, not evidence of a
registered Marketplace publisher or publication eligibility.

### Toolchain and dependencies

The reproducible development toolchain is Node.js 24, pnpm `11.19.0`,
TypeScript `7.0.2`, `@types/node` `24.13.3`, `@types/vscode` `1.134.0`,
`@vscode/test-cli` `0.0.15`, `@vscode/test-electron` `3.1.0`, and
`@vscode/vsce` `3.9.2`. Every package version is exact and belongs only in
`devDependencies`; the pnpm lockfile preserves registry integrity.

The extension has no production Node dependency. It uses Node built-ins and
the host-provided `vscode` module. Adding an MCP SDK, process helper, runtime
library, production bundle dependency, or another package requires separate
approval and an ADR-0052 update.

TypeScript emits CommonJS JavaScript compatible with the selected desktop host.
No bundler or source map is required because runtime dependencies are absent.
Generated JavaScript and VSIX files are build artifacts and are not tracked.

### Activation, commands, and configuration

The manifest contributes exactly `oneagent.connect` and
`oneagent.disconnect`. `activationEvents` is empty because command
contributions auto-activate on the supported VS Code version. The extension
does not activate on startup, language, workspace contents, file changes, or a
wildcard. Activation registers both handlers, one status bar item, and one
configuration-change listener under `ExtensionContext.subscriptions`; it does
not spawn.

The only first-slice setting is `oneagent.runtime.executable`:

- type `string`, scope `window`, default `oneagent-mcp`;
- trimmed value must be non-empty and at most 4,096 UTF-8 bytes;
- the value is one executable path or command, never a shell fragment;
- no arguments, shell, environment override, working-directory override,
  transport, timeout, download, or update setting exists; and
- the setting is read for the selected workspace when connect begins.

A connect attempt requires `workspace.isTrusted`, exactly one
`workspace.workspaceFolders` entry, and URI scheme `file`. Failure of any gate
does not spawn. The selected folder URI filesystem path becomes the child
working directory. User settings override the default and workspace/window
settings follow VS Code precedence; the extension applies only the resolved
value and does not inspect configuration files directly.

A relevant configuration change while connected, connecting, failed, or
disconnecting requests orderly disconnection, rejects pending work, reaps the
child, and ends `disconnected`. It never reconnects automatically. A change
while already disconnected only refreshes validation on the next explicit
connect.

### Lifecycle and user-visible state

One activated extension instance owns one closed state machine:

```text
disconnected -> connecting -> connected
      ^             |             |
      |             v             v
      +---------- failed <- disconnecting
      ^                           |
      +---------------------------+
```

`connect` is accepted only from `disconnected` or `failed`; `disconnect` is
accepted from `connecting`, `connected`, or `failed`. Repeated or incompatible
commands return a stable bounded no-op outcome and never create a second child.
A successful new connect clears the prior failure category.

The extension creates exactly one primary left-aligned status bar item after
activation. It derives text, tooltip, and command only from lifecycle state:

| State | Text | Command |
|---|---|---|
| `disconnected` | `$(circle-outline) OneAgent` | `oneagent.connect` |
| `connecting` | `$(sync~spin) OneAgent` | none |
| `connected` | `$(check) OneAgent` | `oneagent.disconnect` |
| `disconnecting` | `$(sync~spin) OneAgent` | none |
| `failed` | `$(error) OneAgent` | `oneagent.connect` |

Tooltips are fixed English state labels and contain no executable, path,
argument, protocol payload, stderr content, or source value. The extension
does not create an output channel, notification stream, view, tree, webview,
diagnostic collection, telemetry event, or persisted connection state.

Deactivation uses the same disconnect owner, disposes registrations and UI,
rejects pending work, closes stdin, reaps the child, and returns only when
cleanup finishes. Repeated activation/deactivation in separate extension hosts
shares no mutable state.

### Process ownership and spawn policy

One `RuntimeClient` owns the exact configured executable, child handle, stdin,
stdout frame buffer, bounded stderr buffer, listeners, current request,
request counter, timers, and terminal cleanup promise. It spawns directly with
an empty argument array, `shell=false`, inherited environment, selected
workspace root as `cwd`, piped stdin/stdout/stderr, and hidden Windows console.
No shell expansion, PATH modification, file search, binary probing, install,
download, or fallback occurs.

The request counter starts at 1 and increases through JavaScript safe integers.
At most one request is outstanding. Reaching the safe-integer maximum makes the
client fail closed instead of wrapping or reusing an outstanding ID.

Connection readiness has two sequential stateless requests:

1. `server/discover` with ADR-0050 per-request protocol metadata;
2. `tools/list` with the same metadata and no cursor.

Each `_meta` contains protocol version `2026-07-28`, empty client
capabilities, and client info `oneagent-vscode` version `0.1.0`. Discovery must
return the supported version, `capabilities.tools={}`, server name `oneagent`,
zero TTL, and public cache scope. Tool listing must return exactly the six
ADR-0051 names in canonical order. No standard initialize, initialized
notification, session, fallback revision, or semantic tool call is part of
connection readiness.

### Framing, bounds, timeouts, and failures

The client writes compact JSON followed by one LF. It accepts one optional CR
before each input LF, rejects invalid UTF-8, enforces the exact 1,048,576-byte
payload bound before JSON parsing, rejects duplicate response IDs or a response
without the current ID, and validates at most 128 nested array/object levels.
Notifications and extra protocol frames are unsupported in this sequential
slice and fail the connection rather than being silently retained.

Each discovery/list request has a five-second deadline, matching the existing
public Runtime process-test bound. Disconnect first closes stdin and waits two
seconds for successful EOF exit. If the child remains, it calls the platform
process termination primitive and waits a final two seconds. Failure to observe
exit is `shutdown_failed`; no detached cleanup task is left.

At most 4,096 stderr bytes are retained only until terminal classification.
A one-byte-over stderr stream terminates the connection as `stderr_overflow`
without retaining further bytes. Stderr text never enters a user-visible or
implicit diagnostic. Complete stdout frames beyond the accepted bound,
unterminated EOF, invalid UTF-8, malformed/duplicate-key JSON, over-depth JSON,
invalid envelopes, incompatible discovery/list results, and unexpected exit
all fail closed and trigger owned cleanup.

The closed internal failure categories are `invalid_configuration`,
`unsupported_workspace`, `spawn_failed`, `startup_timeout`,
`protocol_failure`, `incompatible_server`, `stderr_overflow`,
`process_exited`, and `shutdown_failed`. Public messages are fixed English
sentences selected by category. `Error`, debug output, status text, tooltips,
and test failure messages do not include executable values, paths, arguments,
environment values, payloads, raw stderr, or source/provenance values.

### Package and artifact contract

`package.json`, `pnpm-lock.yaml`, `tsconfig.json`, `.vscodeignore`,
`.vscode-test.mjs`, extension documentation, TypeScript sources, and tests are
tracked. `node_modules/`, `dist/`, `.vscode-test/`, coverage, temporary user
data, and generated `*.vsix` files are ignored.

`vscode:prepublish` runs the accepted clean compile. Packaging uses the locked
local `@vscode/vsce` with production dependencies disabled because none exist.
The extension payload contains only the manifest, license, README, changelog,
and production `dist` JavaScript. TypeScript, tests, source maps, configs,
lockfile, caches, local workspaces, `.env` files, Rust sources/targets, Git
state, and unrelated repository artifacts are excluded. Task 3 must record the
exact `vsce ls` inventory and assert byte-identical path ordering across two
clean package builds before claiming reproducibility.

Marketplace publication, publisher registration, authentication, signing, and
update distribution are not implied by a valid local VSIX.

### Validation and CI

The extension package defines exact scripts for clean locked install,
typecheck, compile, Node built-in unit tests, pinned Extension Development Host
tests, real `oneagent-mcp` process tests, VSIX packaging, and inventory. Test
filters must execute non-zero cases.

Unit tests use injected process, clock, configuration, and UI ports with
explicit completion events. They cover configuration bounds and precedence;
all lifecycle transitions; duplicate commands; spawn, timeout, protocol,
compatibility, stderr, EOF, exit, and shutdown failures; exact/one-over frame
and depth limits; reordered JSON members; repeated fresh clients; and zero
surviving handles/listeners/timers.

Extension-host tests use exactly VS Code `1.134.0`, isolated user data,
`--disable-extensions`, and tracked trusted/untrusted or unsupported workspace
fixtures as applicable. They prove command activation, manifest contributions,
configuration integration, status mapping, no-spawn invalid cases, replacement,
deactivation, disposables, and repetition through the public VS Code API.

Real-process tests build the public Rust binary, pass its exact test path as
configuration, use a tracked supported workspace, and prove the selected
working directory, discovery/list compatibility, channel purity, disconnect/
EOF, startup failure, and repeated fresh children. They do not modify Runtime
or depend on a globally installed binary.

CI adds a separate Node 24/pnpm `11.19.0` job on `macos-14` and
`windows-latest`. It performs a frozen install, extension gate, public Runtime
build/process gate, pinned editor test, package/list audit, and artifact
exclusion checks. The existing Rust CI job remains unchanged. Sprint completion
also requires the canonical full Rust workspace gate locally.

### Implementation sequence and compatibility

Task 3 establishes only the locked package, manifest, minimal activation owner,
unit build, and package inventory. Task 4 adds the editor-independent process
client and real-process evidence. Task 5 connects it to configuration, commands,
status UI, and extension-host lifecycle. Task 6 completes CI, public matrices,
packaging audits, and current-state documentation.

No Rust code, Cargo manifest, Runtime behavior, MCP wire contract, semantic
owner, HTTP/CLI behavior, or Coverage Registry status changes in the accepted
first slice. A concrete incompatibility with ADR-0050/0051 stops implementation
instead of introducing client fallback or server modification.

## Consequences

Users can install a reproducible local desktop VSIX and explicitly connect one
trusted file workspace to an existing `oneagent-mcp` binary. Connection state
and cleanup are observable and bounded, but no navigation, language feature,
diagnostic, chat, or semantic UI is exposed yet. Users must configure or
otherwise make the binary resolvable; the extension does not install it.

## Rejected alternatives

- Standard MCP initialize/session behavior contradicts ADR-0050.
- Startup, wildcard, language, or workspace-content activation is broader than
  explicit first-slice demand.
- A web extension cannot own the accepted local stdio child.
- An MCP SDK or process library adds an unnecessary production dependency and
  duplicate protocol owner.
- Shell strings, configurable arguments/environment, automatic discovery,
  download, install, or fallback expand the process-execution boundary.
- Automatic reconnect hides failure and makes configuration replacement and
  cleanup ownership timing-dependent.
- Multiple workspace children, concurrent requests, and request-ID reuse are
  unnecessary before product features consume the client.
- Mutable latest editor tests and mock-only validation cannot prove
  reproducible extension-host or real-process behavior.
- Bundling without production dependencies adds a tool and artifact boundary
  without a first-slice benefit.

## Deferred scope

Navigation and symbol search, LSP, diagnostics engine/UI, chat/context panel,
EDT, remote and web extension hosts, multi-root operation, background
activation, watcher/reload, concurrent MCP, progress/cancellation
notifications, semantic calls as connection probes, Runtime discovery/download/
update, Marketplace registration/publication/signing, telemetry,
authentication, other editors, semantic changes, and broad performance or
security claims remain deferred.
