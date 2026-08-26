# VS Code Extension Foundation Investigation

This investigation records the committed repository and pinned upstream
evidence available before ADR-0052. It does not accept an extension
architecture or implement production behavior.

## Confirmed repository baseline

- Planning HEAD is `f0958292`, with framework prerequisite `90695c74` in its
  ancestry. Sprint 29 is completed by `2ce0a845`; Sprint 30 is the unique
  active target after this investigation starts.
- `extensions/` is empty. The repository has no Node package manifest,
  lockfile, TypeScript configuration, JavaScript/TypeScript source, VS Code
  extension test, VSIX build, or Node CI job.
- The Rust workspace remains rooted at `Cargo.toml`. Existing CI in
  `.github/workflows/ci.yml` runs Rust format, check, test, and Clippy on
  `macos-14` and `windows-latest`.
- `apps/runtime/src/bin/oneagent-mcp.rs` is the dedicated public process. It
  builds one immutable workspace snapshot from its current directory, owns
  stdin/stdout/stderr, and exits successfully on complete EOF or cancellation.
- `apps/runtime/src/mcp.rs` owns newline framing, an exact 1,048,576-byte
  payload bound, optional CR before LF, sequential dispatch, output flush,
  cancellation, EOF, and closed redacted failure categories.
- `apps/runtime/tests/mcp_process.rs` proves public process spawn, exact working
  directory ownership, protocol-channel purity, discovery, all six semantic
  tool families, malformed and oversized input, bounded stderr, repeated
  execution, EOF, and startup failure.
- ADR-0050 explicitly rejects the legacy MCP `initialize`/`initialized`
  session. Every request is stateless and carries protocol version and client
  capabilities in `_meta`. The supported revision is exactly `2026-07-28`.
- `server/discover` is the readiness and compatibility probe. Its result
  advertises version `2026-07-28`, `capabilities.tools={}`, server identity
  `oneagent`, zero TTL, and public cache scope. A following `tools/list`
  response lists the immutable six-tool ADR-0051 catalog.
- Request identifiers are bounded strings or JSON integers. Method names are
  at most 256 UTF-8 bytes, JSON nesting is at most 128 object/array levels, and
  complete messages are at most 1,048,576 bytes. Dispatch is sequential and
  no request ID may be outstanding twice.

## Pinned upstream platform evidence

The selected platform candidate is stable desktop VS Code `1.134.0`. Its
version-selection provenance is pinned to the official release tag and full
source commit `474a349ad5b745e512ef86b864d1c74f7264dd7a`, published on
2026-08-19. The mutable update-service inventory was observed on 2026-08-26
only to discover the then-current stable candidate; it is not an immutable
authority for the historical selection:

- [Official `1.134.0` release](https://github.com/microsoft/vscode/releases/tag/1.134.0)
- [Immutable `1.134.0` source commit](https://github.com/microsoft/vscode/commit/474a349ad5b745e512ef86b864d1c74f7264dd7a)
- [Dated mutable release-inventory observation](https://update.code.visualstudio.com/api/releases/stable)
- [Extension manifest reference](https://code.visualstudio.com/api/references/extension-manifest)
- [Extension anatomy](https://code.visualstudio.com/api/get-started/extension-anatomy)
- [Activation events](https://code.visualstudio.com/api/references/activation-events)
- [Extension host](https://code.visualstudio.com/api/advanced-topics/extension-host)
- [Contribution points](https://code.visualstudio.com/api/references/contribution-points)
- [Common capabilities](https://code.visualstudio.com/api/extension-capabilities/common-capabilities)
- [Status Bar UX](https://code.visualstudio.com/api/ux-guidelines/status-bar)
- [Workspace Trust](https://code.visualstudio.com/api/extension-guides/workspace-trust)
- [Testing extensions](https://code.visualstudio.com/api/working-with-extensions/testing-extension)
- [Continuous integration](https://code.visualstudio.com/api/working-with-extensions/continuous-integration)
- [Bundling extensions](https://code.visualstudio.com/api/working-with-extensions/bundling-extension)
- [Publishing and VSIX packaging](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)

The `code.visualstudio.com` references above explain the documented extension
model but are living documentation. Reproducibility of the selected platform
and Extension Host evidence depends on the immutable source commit, the exact
manifest engine and `@types/vscode` versions, the locked development
dependencies, and the exact `1.134.0` test download rather than on mutable
ordering or wording at those documentation URLs.

Confirmed platform facts are:

- every extension has a root `package.json`; `name`, `version`, `publisher`,
  and non-wildcard `engines.vscode` are required, while `main` selects the
  Node.js extension entry point;
- a desktop local or remote extension host uses Node.js; a workspace extension
  runs where workspace contents and executable access are available;
- commands declared by an extension auto-activate it on invocation for VS Code
  1.74 and later, so explicit command demand does not require
  `onStartupFinished` or wildcard activation;
- an entry point exports `activate`, and optional `deactivate` owns shutdown;
- configuration contributions declare type, default, description, and scope;
  `window` settings can be set at user, workspace, or remote level;
- commands are contributed statically and registered through the VS Code API;
  status bar items are workspace-level UI and should use one short primary
  item without custom color;
- an extension that can execute a workspace-selected binary must explicitly
  participate in Workspace Trust; declaring untrusted workspaces unsupported
  prevents activation until trust is granted;
- `@vscode/test-cli` plus `@vscode/test-electron` runs tests inside an
  Extension Development Host, and the editor version can be fixed rather than
  left at mutable `stable`; and
- `@vscode/vsce` builds a VSIX, runs `vscode:prepublish`, supports explicit
  package exclusions, and omits development dependencies.

## Toolchain and dependency evidence

The available approved external validation runtime reported Node.js `24.19.0`
and pnpm `11.19.0`. Ordinary repository shells do not expose `node`; Sprint
validation must therefore use the explicitly approved bundled runtime locally
and Node 24 in CI.

Read-only npm registry queries on 2026-08-26 returned this exact candidate
development-only set:

| Package | Version | Engine evidence | License | Registry integrity |
|---|---:|---|---|---|
| `typescript` | `7.0.2` | Node `>=16.20.0` | Apache-2.0 | `sha512-8FYau96o3NKOhbjKi/qNvG/W5jhzxkbdm5sj9AbZ/5T5sWqn3hJgLfGx27sRKZWTvyzCP8dLRBTf5tBTSRVUNA==` |
| `@types/vscode` | `1.134.0` | Matches selected VS Code API | MIT | `sha512-NDEu0hg4sF7+vvFsADsktqUJ6f80LHSZvVK2Ovo1XiQ0/VHck1O3zst+ZZyVA/uvz6vo6LcuoqU2q48YMqOwWw==` |
| `@types/node` | `24.13.3` | Matches Node 24 major | MIT | `sha512-Dh8vAsV36ig5wa9OX4pXvMc9D3Veibfw2wix0CUwYODLD8nkj9UsLjASr49nPg+2eKzxhBV+v7L8pXvT4e639Q==` |
| `@vscode/test-cli` | `0.0.15` | Node `>=22` | MIT | `sha512-nAxk2X79wuXS7aOhyFFhFcCqd7EBUoMesu7ZgsYE/4eFjyBMuyIweVE94BxdKH1RieN8eOz2SIrljrZt6Lk9fQ==` |
| `@vscode/test-electron` | `3.1.0` | Node `>=22` | MIT | `sha512-CRqv5u+YYoseuNVJ6Tyo4k0sF0mx4qnKMihRB0PjsUF8Dc0WKtCXo6CNL6nWWm5esfFQsQA/pejMj4ZbpJVLTw==` |
| `@vscode/vsce` | `3.9.2` | Node `>=20` | MIT | `sha512-XSxMosEEDO6vLxELAHVkwmhC0qe0ijZni2jB9Rcs8kQsW4lhTDQ/wMzmwFs/buotAWSnpmUp/dRWD2ufG3UYKA==` |

Exact versions plus the generated pnpm lockfile can preserve the registry
tarball integrity. Node built-ins provide process, stream, event, path, and
test behavior. The VS Code host supplies the `vscode` module. No production
Node dependency is required by the bounded first slice.

## Decision-ready extension surface

The evidence supports these bounded ADR-0052 candidates without selecting
them here:

- package path `extensions/vscode`, extension name `oneagent`, publisher
  `oneagent-dev`, extension ID `oneagent-dev.oneagent`, version `0.1.0`,
  Apache-2.0 license, desktop `main` entry point, and workspace extension kind;
- `engines.vscode` based on `1.134.0`, an exact `1.134.0` Extension Development
  Host test version, Node 24 build/CI, pnpm `11.19.0`, and exact development
  dependency versions above;
- explicit commands `oneagent.connect` and `oneagent.disconnect` as the only
  activation demand, with no startup, language, wildcard, or workspace-content
  activation;
- one `window`-scoped `oneagent.runtime.executable` string whose default is
  `oneagent-mcp`, with a non-empty UTF-8 byte bound and no configurable
  arguments, shell, environment, transport, or automatic download;
- exactly one trusted, file-backed workspace folder; empty windows, virtual
  workspaces, untrusted workspaces, and multi-root fan-out fail before spawn;
- one primary status bar item derived from closed lifecycle state, not logs;
- one child process with exact configured executable, no arguments, inherited
  environment, selected workspace root as current directory, piped standard
  streams, and one explicit owner;
- a stateless readiness sequence of `server/discover` then `tools/list`, each
  with required per-request metadata. This is not standard MCP initialize and
  creates no session;
- at most one outstanding request, monotonically increasing safe integer IDs,
  exact LF framing, 1,048,576-byte frame/input/output bounds, 128-level JSON
  validation, bounded retained stderr, typed redacted failures, and no payload
  logging;
- no automatic connect or restart; explicit connect, disconnect, configuration
  replacement, unexpected exit, and deactivation terminate pending work and
  reap the process before a terminal state;
- compiled CommonJS output without a production bundle because the slice has
  no production dependency; sources, source maps, tests, configs, lockfile,
  caches, local workspaces, Rust target output, and secrets remain outside the
  VSIX.

## Ownership and compatibility constraints

- TypeScript owns editor adaptation and client process state only. Rust remains
  the protocol, Runtime, workspace, graph, analysis, and semantic authority.
- The extension must not import or reproduce semantic graph behavior. It may
  validate only the closed client-side fields needed to establish compatibility
  and lifecycle safety.
- `server/discover` and `tools/list` requests must follow ADR-0050/0051 exactly.
  No standard initialize fallback, older revision, remote transport, session,
  or concurrent dispatch can be inferred.
- Existing Rust behavior, Cargo dependencies, HTTP/CLI surfaces, and CI checks
  remain unchanged. A separate Node CI job may add evidence without weakening
  the Rust job.
- The executable setting can cause process execution. Workspace Trust and
  explicit user command demand are mandatory safety evidence; package install,
  folder open, or activation alone must not spawn.

## Deterministic evidence matrix

| Layer | Required observable cases |
|---|---|
| Manifest/package | Exact identity, engine, host, activation, commands, setting, trust/virtual-workspace declarations, clean build, repeated VSIX inventory, and prohibited-file absence |
| Configuration | Default, explicit valid, empty, wrong type, over-bound, no folder, multi-root, virtual, untrusted, replacement while disconnected, and replacement while connected |
| Pure client | Spawn failure, discover success/failure, tools/list agreement, malformed JSON, duplicate/unknown ID, wrong result, oversized/deep frame, stderr bound, EOF, unexpected exit, sequential repetition, disconnect, and pending rejection |
| Extension lifecycle | Demand activation, connect, disconnect, status sequence, duplicate command, invalid configuration without spawn, replacement, process failure, repeated activation/deactivation, disposable cleanup, and no orphan |
| Real process | Built public binary, exact working directory, discovery and six-tool catalog, one semantic call if required by ADR, channel purity, EOF/disconnect, repeated fresh child, and startup failure |
| CI/package | Clean locked install, typecheck, compile, non-zero unit and extension-host tests, exact VS Code `1.134.0`, real-process tests, VSIX build/list, Rust workspace gate, macOS and Windows coverage |

Tests must use explicit events, child handles, promise completion, and process
exit. Arbitrary sleeps, a mutable latest editor download, zero-match filters,
or handler-only tests are not acceptance evidence.

## Rejected investigation candidates

- A web extension cannot spawn the accepted local stdio Runtime and is outside
  the desktop first slice.
- `onStartupFinished`, wildcard, language, or workspace-content activation
  would perform work before explicit OneAgent demand and is unnecessary.
- Standard MCP `initialize` conflicts with accepted ADR-0050.
- An MCP SDK or other production dependency adds an unneeded semantic/protocol
  owner and requires separate approval.
- Shell execution, configurable arguments/environment, automatic binary
  download, PATH mutation, and implicit workspace discovery expand the effect
  and security boundary.
- Mutable `stable` test downloads cannot prove reproducible compatibility.
- Unit mocks alone cannot prove VS Code activation or the real Runtime process;
  extension-host tests alone cannot prove pure failure/state coverage.

## Remaining ADR-0052 decisions

ADR-0052 must select the exact manifest values, configuration byte bound,
client/server validation subset, stderr bound, graceful-to-forced shutdown
policy, lifecycle state vocabulary, status text/command mapping, package
inventory, test scripts, CI job shape, and whether a semantic call beyond
discovery/list is required for connection readiness. All alternatives have
repository or upstream evidence and deterministic oracles; no external-data
blocker remains.

## Deferred scope

Navigation and symbol search, LSP, diagnostics, chat/context panel, EDT, remote
and web extension hosts, multi-root operation, background activation, workspace
watching/reload, concurrent requests, cancellation/progress notifications,
Runtime discovery/download/update, Marketplace publication/signing, telemetry,
authentication, other editors, semantic changes, and broad performance or
security claims remain deferred.
