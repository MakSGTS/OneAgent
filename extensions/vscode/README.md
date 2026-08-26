# OneAgent for VS Code

This package is the desktop VS Code foundation for connecting one trusted,
file-backed workspace to an existing `oneagent-mcp` executable.

Sprint 30 provides package, activation, configuration, connection lifecycle,
and cleanup foundations. Navigation, symbol search, LSP, diagnostics, chat,
EDT integration, remote/web hosts, automatic Runtime installation, and
Marketplace publication are not included.

## Configuration

`oneagent.runtime.executable` selects the executable path or command. The
default is `oneagent-mcp`. OneAgent never invokes a shell and does not accept
configurable arguments or environment overrides.

## Commands

- `OneAgent: Connect`
- `OneAgent: Disconnect`

Opening a workspace or activating the extension does not start a process.
Connection requires an explicit command in a trusted workspace.
