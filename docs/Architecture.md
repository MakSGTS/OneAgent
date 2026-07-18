# Architecture

OneAgent uses a modular Cargo workspace.

The initial layers are:

1. Applications: Runtime and CLI.
2. Domain crates: workspace, metadata and protocol.
3. Adapters: IDE, EDT, Designer, Git and filesystem.
4. Extensions: VS Code and future clients.
