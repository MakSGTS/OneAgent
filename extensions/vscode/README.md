# OneAgent for VS Code

This desktop workspace extension connects one trusted, file-backed VS Code
workspace to an existing `oneagent-mcp` executable. It provides an explicit,
bounded connection lifecycle and does not install or discover the Runtime.

## Requirements

- Desktop VS Code `1.134.0` or a compatible `^1.134.0` release.
- Exactly one trusted local workspace folder with the `file` URI scheme.
- An existing compatible `oneagent-mcp` executable.

Opening a workspace or activating the extension does not start a process. The
extension activates on its contributed commands and starts the Runtime only
after `OneAgent: Connect` passes every workspace and configuration check.

## Configuration

`oneagent.runtime.executable` selects the executable path or command. The
window-scoped default is `oneagent-mcp`. The trimmed value must contain between
1 and 4,096 UTF-8 bytes.

OneAgent spawns this value directly with no shell and no arguments, using the
workspace folder as the working directory and inheriting the extension-host
environment. It does not accept configurable arguments, environment changes,
working-directory overrides, downloads, updates, or fallback executables.

## Commands

- `OneAgent: Connect`
- `OneAgent: Disconnect`

Connection readiness sends the accepted stateless `server/discover` and
`tools/list` MCP requests and requires the exact six read-only OneAgent tools.
Changing the executable while a connection is active disconnects the current
child and never reconnects automatically. A later connection always requires
another explicit command.

## Connection status

The left status bar item reports one of five fixed states without executable,
path, protocol payload, stderr, or source values:

| State | Status | Available action |
|---|---|---|
| Disconnected | `$(circle-outline) OneAgent` | Connect |
| Connecting | `$(sync~spin) OneAgent` | None |
| Connected | `$(check) OneAgent` | Disconnect |
| Disconnecting | `$(sync~spin) OneAgent` | None |
| Failed | `$(error) OneAgent` | Connect |

Failures are closed and redacted. Use an explicit disconnect or a new connect
attempt after correcting the workspace or executable. Deactivation closes
stdin, waits for the owned child, applies the bounded termination policy when
needed, and does not leave background reconnect work.

## Contributor validation

The package uses Node.js 24, pnpm `11.19.0`, TypeScript `7.0.2`, and a pinned VS
Code `1.134.0` Extension Host. There are no production Node dependencies.

From the repository root, build the public Runtime and install the locked
development dependencies:

```bash
cargo build -p oneagent-runtime --bin oneagent-mcp
pnpm --dir extensions/vscode install --frozen-lockfile
```

Run the extension and public-process gates:

```bash
pnpm --dir extensions/vscode run typecheck
pnpm --dir extensions/vscode test
ONEAGENT_MCP_BIN="$PWD/target/debug/oneagent-mcp" pnpm --dir extensions/vscode run test:process
pnpm --dir extensions/vscode run package:check
pnpm --dir extensions/vscode run package:verify
pnpm --dir extensions/vscode run audit
```

On Windows, set `ONEAGENT_MCP_BIN` to the absolute
`target/debug/oneagent-mcp.exe` path. The pinned Extension Host configuration
selects the same platform-specific repository binary automatically.

The VSIX contains only the manifest, license, README, changelog, and five
compiled production JavaScript modules. Source, tests, lockfiles, caches,
workspace files, Rust artifacts, secrets, and local configuration are excluded.

## Deferred scope

Navigation, symbol search, LSP, diagnostics, chat/context UI, EDT integration,
remote and web extension hosts, multi-root operation, Runtime installation or
updates, Marketplace publication/signing, telemetry, authentication, and
external-client compatibility are not included.
