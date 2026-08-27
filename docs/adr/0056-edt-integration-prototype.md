# ADR-0056: EDT Integration Prototype

## Status

Accepted

## Context

Sprint 34 must prove one native 1C:EDT user workflow over the existing public
`oneagent-mcp` process without moving source parsing or semantic authority into
Java. The repository, official upstream, installed-toolchain, public-API,
p2/authentication, Runtime-process, and executable-host evidence is recorded in
the [EDT Integration Prototype investigation](../architecture/edt-integration-prototype-investigation.md).

The Runtime already owns one immutable startup Workspace snapshot, MCP revision
`2026-07-28`, newline-framed stdio, and the exact seven-tool catalog. The
checked-in EDT project has the exact nature
`com._1c.g5.v8.dt.core.V8ConfigurationNature`, but currently fails Runtime
snapshot construction with the stable `workspace build failure` category. A
supported reduced fixture succeeds through both the public Runtime process and
a disposable EDT 2026.1 import when it includes `DT-INF/PROJECT.PMF`.

The first plug-in slice needs only public Eclipse 4.30 APIs. Authenticated 1C p2
access is therefore not a production or CI prerequisite, and the installed p2
pool remains read-only evidence rather than a repository input.

## Decision

### Canonical statement and authority

OneAgent adds one native Eclipse/1C:EDT plug-in command that probes compatibility
with an explicitly configured `oneagent-mcp` executable for exactly one selected
local EDT configuration project. The command performs one stateless bounded
`server/discover` exchange in an owned background job, closes and reaps the
process, and publishes one fixed success or redacted failure on the UI thread.

Rust remains the sole Runtime, Workspace, protocol, graph, analysis, Context,
Tool Policy, and semantic authority. Java owns only Eclipse adaptation, project
eligibility, preference validation, process and stream lifecycle, strict closed
wire decoding, command state, cancellation, and fixed UI presentation. Java
does not read project sources, parse EDT artifacts, infer semantic facts,
project tool results, or import a proprietary EDT implementation package.

The dependency direction is:

```text
public Eclipse API -> com.oneagent.edt -> oneagent-mcp process
                                        -> MCP 2026-07-28 wire contract

oneagent-mcp -> existing Rust Workspace/Graph/Analysis/Tool Policy owners
```

There is no Java-to-Rust library link, EDT-to-Runtime callback, Runtime-to-IDE
dependency, or second semantic owner.

### Package identity and public identifiers

The accepted reactor root is `extensions/edt`. Its modules and identifiers are:

| Surface | Path / identifier |
|---|---|
| Target definition | `extensions/edt/releng/com.oneagent.edt.target` |
| Production bundle | `extensions/edt/bundles/com.oneagent.edt` / `com.oneagent.edt` |
| Test fragment | `extensions/edt/tests/com.oneagent.edt.tests` / `com.oneagent.edt.tests`, fragment host `com.oneagent.edt` |
| Feature | `extensions/edt/features/com.oneagent.edt.feature` / `com.oneagent.edt.feature` |
| p2 repository | `extensions/edt/repositories/com.oneagent.edt.repository` |
| Repository category | `com.oneagent.edt.category` / `OneAgent` |
| Command category | `com.oneagent.edt.commands.category` / `OneAgent` |
| Command | `com.oneagent.edt.commands.probeRuntime` / `OneAgent: Probe Runtime Compatibility` |
| Preference page | `com.oneagent.edt.preferences` / `OneAgent` |
| Preference key | `runtimeExecutable` |

All bundle, feature, and repository versions begin at `0.1.0.qualifier`. The
feature label and provider are `OneAgent EDT Integration` and `OneAgent`.
Tracked metadata uses English. The package license is Apache-2.0 through the
repository root `LICENSE`.

The command is contributed through `org.eclipse.ui.commands`,
`org.eclipse.ui.handlers`, and `org.eclipse.ui.menus` at
`popup:org.eclipse.ui.popup.any?after=additions`. The preference page uses
`org.eclipse.ui.preferencePages`; defaults use
`org.eclipse.core.runtime.preferences`. No proprietary extension point is used.

### Supported platform, Java, and dependencies

Maven `3.9.16` running explicitly on arm64 JDK 25 is the local build launcher.
The pinned build is Tycho `5.0.2` against public Eclipse 2023-12 / Eclipse 4.30.
The bundle declares `Bundle-RequiredExecutionEnvironment: JavaSE-17` and Maven
compiles production and test bytecode with release 17, so the artifact loads in
the x86_64 EDT 2026.1 host on the verified x86_64 JDK 17. PDE evidence also runs
the x86_64 Eclipse 2025-12 development product on JDK 25.

Production code may use only Java 17 and exported packages from:

- `org.eclipse.core.commands`;
- `org.eclipse.core.jobs`;
- `org.eclipse.core.resources`;
- `org.eclipse.core.runtime` and `org.eclipse.core.runtime.preferences`;
- `org.eclipse.jface`;
- `org.eclipse.swt`;
- `org.eclipse.ui`, `org.eclipse.ui.ide`, and `org.eclipse.ui.workbench`; and
- the OSGi framework packages supplied by the platform.

The nature string is compared as data. No `com._1c.g5.v8.dt.*`, internal,
restricted, reflective, split-package, MCP SDK, JSON library, process helper,
logging framework, DI framework, or other production dependency is accepted.
JUnit and Tycho test harness artifacts are test/build inputs only. Adding a
production dependency requires explicit user approval and an ADR update.

The target file contains only the exact public Eclipse 2023-12 repository and
required public units. Builds and CI do not contact a private 1C endpoint. A
developer may configure authenticated official p2 access outside tracked files
for later proprietary experiments, but credentials, Maven settings, personal
paths, and resolved secret-bearing URLs are never copied, logged, packaged, or
required. The user-authorized local p2 pool is never written, published through
symlinks, or treated as repository metadata.

### Eligible project and selection contract

The command is enabled only when the current structured workbench selection
contains exactly one element and that element adapts to one `IProject` that:

1. exists, is open, and is accessible;
2. is neither linked nor virtual;
3. has the exact nature
   `com._1c.g5.v8.dt.core.V8ConfigurationNature`;
4. has non-null `getLocation()` and `getLocationURI()`;
5. has a `file` location URI and an absolute local filesystem path; and
6. maps to an existing readable directory.

Nature, resource, URI, security, and filesystem inspection failure rejects the
project. Empty, missing, inaccessible, closed, non-EDT, file/folder, linked,
virtual, remote, and multiple selections are ineligible. No fallback to active
editor, workspace root, project name, source file, first selection, nested
resource, or all projects is permitted.

The public handler repeats the complete eligibility check at execution time so
a stale enablement result cannot spawn. A race that becomes ineligible returns
`Select one local EDT configuration project.` without starting a process.

### Runtime executable configuration

The `runtimeExecutable` preference is an instance-scoped string with default
`oneagent-mcp`. Its UTF-8-trimmed value must be 1 through 4,096 bytes, contain no
NUL, CR, or LF, and represent one executable token rather than a shell command.
The trim is the applied value.

A value containing a platform path separator must parse as an absolute,
normalized path to an existing readable regular file that is executable on
platforms exposing an executable bit. A bare value must match
`[A-Za-z0-9._-]+` and is passed directly to `ProcessBuilder` for ordinary OS
search. Relative paths with separators, URI values, arguments, quotes, shell
operators, wildcards, environment expansion, and directories are invalid.

The preference page uses one file field and validation message but never tests
by launching the executable. The configured value is read once when the command
is accepted. Configuration changes invalidate the generation, cancel any
running job, reap its process, and return the controller to idle. They never
start or restart a process automatically.

### Exact process contract

One accepted invocation creates one `ProcessBuilder` with:

- the validated executable as the only command-array element;
- no arguments and no shell;
- inherited environment without mutation or inspection;
- the eligible project directory as `directory`;
- separate piped stdin, stdout, and stderr; and
- no redirect, terminal, download, probing, PATH modification, or fallback.

The client owns the process handle, all three streams, reader tasks, deadline,
cancellation registration, retained stderr count, terminal result, and cleanup.
No process or stream is shared across invocations. Every terminal outcome closes
all streams, terminates a live child as required below, waits for exit, stops
reader tasks, and releases executor/future resources before completion.

### Exact compatibility request and response

Every fresh client uses integer request ID 1 and writes exactly one compact
UTF-8 JSON object followed by LF:

```json
{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"io.modelcontextprotocol/clientInfo":{"name":"oneagent-edt","version":"0.1.0"}}}}
```

The request is fixed and has no project path, source value, preference,
credential, tool call, argument, or extension field. There is no MCP initialize,
notification, session, `tools/list`, semantic call, retry, or fallback revision.

The client accepts member reordering but otherwise requires one closed response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "resultType": "complete",
    "supportedVersions": ["2026-07-28"],
    "capabilities": {"tools": {}},
    "_meta": {
      "io.modelcontextprotocol/serverInfo": {
        "name": "oneagent",
        "version": "0.1.0"
      }
    },
    "ttlMs": 0,
    "cacheScope": "public"
  }
}
```

Every displayed field, object, and array member is required; unknown or
duplicate members fail closed. Numbers must use an integer lexical form within
signed 64-bit range. Strings must be valid Unicode without unpaired surrogates.
The response must have no `error`. Exact IDs, values, array length/order, empty
objects, and server version are compatibility requirements for this repository
version.

The dependency-free parser accepts JSON whitespace and member reordering, but
rejects duplicate keys at every depth, trailing content, invalid escapes,
invalid UTF-8, non-finite/fraction/exponent numbers, and object/array nesting
deeper than 128. The complete stdout frame before optional CR and LF is at most
1,048,576 bytes. EOF before LF, a second frame, bytes after the accepted frame,
or any stdout content after it is a protocol failure.

### Timeouts, stderr, exit, and termination

The compatibility response deadline is 5,000 ms from successful spawn. At most
4,096 stderr bytes may be consumed, and successful compatibility requires
stderr to be empty. Stderr overflow terminates immediately; retained stderr is
discarded and never enters a public value, exception message, log, test name, or
UI. stdout and stderr are drained concurrently so neither pipe can deadlock.

After a compatible response the client closes stdin and waits up to 2,000 ms
for exit 0 and complete stream EOF. Non-zero exit, early exit, stderr, extra or
unterminated stdout, reader failure, or incomplete cleanup is a process failure.

On failure, cancellation, timeout, interruption, configuration change, or
bundle stop, cleanup closes stdin, waits at most 2,000 ms, calls `destroy()` and
waits at most 2,000 ms, then calls `destroyForcibly()` and waits at most 2,000
ms. Failure to observe final exit is `shutdown_failed`. Cancellation is checked
before spawn, before write, while awaiting response, and before presentation;
it supersedes every non-terminal result. Thread interruption is restored.

### Client result and closed failures

The Java client returns only `compatible` or throws one closed internal category:

| Category | Meaning |
|---|---|
| `invalid_configuration` | Local executable value is invalid before spawn |
| `spawn_failed` | Direct process start failed |
| `timeout` | The 5,000 ms response deadline elapsed |
| `protocol_failure` | Framing, UTF-8, JSON, identity, or closed shape failed |
| `incompatible_server` | A well-formed closed response has a non-accepted compatibility value |
| `stderr_overflow` | More than 4,096 stderr bytes were observed |
| `process_failed` | Stderr, stream, EOF, or non-zero/early exit failed |
| `shutdown_failed` | Final child exit could not be observed within termination bounds |
| `cancelled` | User or lifecycle cancellation won |

Exceptions contain only their category. They retain no source cause, command,
path, environment, project, payload, stderr, host, or filesystem value in
`getMessage()`, `toString()`, or public presentation. Test-only injected ports
may retain deterministic symbolic events, never user data.

### Command state, threading, and visible outcomes

One activated bundle owns one controller state machine:

```text
idle -> running -> idle
  |        |
  v        v
disposed <- cancelling
```

Exactly one invocation may run. A second accepted invocation while running or
cancelling starts no process and shows
`OneAgent Runtime probe is already running.` Configuration change and bundle
stop increment an ownership generation and cancel the current job. A completion
whose generation is stale publishes nothing.

Selection and preference snapshots are obtained on the UI thread. Process,
stream, parsing, waiting, and cleanup work runs in one owned Eclipse `Job` and
never blocks the UI thread. Only the current, non-cancelled generation schedules
one UI-thread presentation. The job returns a stable Eclipse status without a
source exception or value.

The dialog title is always `OneAgent`. The closed messages are:

| Outcome | Kind | Message |
|---|---|---|
| compatible | information | `OneAgent Runtime is compatible.` |
| stale ineligible selection | error | `Select one local EDT configuration project.` |
| invalid configuration | error | `Configure a valid OneAgent Runtime executable.` |
| busy | information | `OneAgent Runtime probe is already running.` |
| spawn failed | error | `OneAgent Runtime could not be started.` |
| incompatible server | error | `OneAgent Runtime is incompatible.` |
| timeout | error | `OneAgent Runtime probe timed out.` |
| protocol/process/stderr/shutdown failure | error | `OneAgent Runtime probe failed.` |

User cancellation, configuration replacement, bundle stop, and host shutdown
are silent. No dialog, status, tooltip, log, console, output view, notification,
telemetry, persistence, or exception exposes a path, project name, executable,
payload, server value, stderr, raw error, or source content.

### Activation and resource ownership

Installing, starting Eclipse, opening a project, changing a preference, and
selecting a resource do not spawn. Command demand activates the lazy bundle and
is the only process-start trigger.

The `AbstractUIPlugin` activator owns one controller and one preference-change
listener. The controller owns the active job, generation, cancellation source,
client, process, streams, reader tasks, and UI presentation port. The Eclipse
registry owns declared command, handler, menu, preference page, and preference
initializer instances; handler and page `dispose()` release their local
listeners/references.

Bundle stop removes the preference listener, increments the generation,
cancels and joins the active job after its bounded process cleanup, disposes the
controller, and clears the singleton. Cleanup is idempotent. No non-daemon
thread, process, stream, job, listener, preference page, dialog callback, timer,
or static mutable owner survives deactivation.

### Bundle, feature, repository, and artifact contract

The Maven reactor contains one parent, one target-definition module at
`releng/com.oneagent.edt.target` with
`com.oneagent.edt.target.target`, the production bundle, one test fragment, one
feature, and one p2 repository. Tycho builds from tracked sources with UTC
reproducible timestamps where supported and no local absolute path in effective
metadata. The production bundle exports no package. The test fragment attaches
only to `com.oneagent.edt`, accesses package-private injected seams, and is
absent from the feature and production repository.

The feature contains exactly the production bundle. The p2 repository contains
exactly that feature and its bundle plus one `OneAgent` category. It contains no
JRE, Runtime binary, credential, Maven settings, private-p2 metadata, local pool,
test bundle, source archive, native executable, signed artifact, application
bundle, or generated user configuration. Generated `target/`, repository zip,
and local host workspaces are ignored and never tracked.

The install oracle creates a disposable host destination and p2 agent/profile
outside both authorized application bundles and the read-only pool. It installs
from the locally built repository, runs the public command workflow, uninstalls,
and verifies absence. Original applications and their signatures are never
modified, copied over, re-signed, or used as writable destinations. Host runs
are sequential and use exact exit/status/log checks; a successful final shell
pipeline code cannot mask a failed stage.

### Deterministic evidence and CI

Task 3 implements the dependency-free Java client behind injected process,
clock/deadline, and cancellation ports. Non-zero Java tests cover exact and
reordered compatible responses; request bytes; duplicate/missing/unknown fields;
all wrong values and error envelopes; UTF-8/escape/number/depth/frame exact and
one-over cases; malformed, extra, and unterminated frames; timeout; stderr exact
and one-over; spawn/exit/stream/shutdown failures; cancellation at every
boundary; repeated fresh clients; and zero surviving resources. A real built
`oneagent-mcp` process over the supported reduced fixture proves cwd, request,
response, EOF, exit, repetition, and the checked-in full-project failure.

Task 4 implements public Eclipse adaptation behind injected selection,
preference, job, process-client, and presenter ports. Non-zero unit and PDE-host
tests cover every eligibility rule, enablement/execution agreement, executable
validation, fixed dialogs, background/UI separation, busy repetition,
configuration-change and stop cancellation, stale-generation suppression,
handler/page/activator disposal, and no spawn on activation or invalid input.

Task 5 creates the feature and repository and runs clean Tycho package inventory,
disposable install, positive/negative/repeated authorized EDT 2026.1 workflow,
shutdown, uninstall, absence, clean-host, and original-application/pool
immutability evidence. PDE and EDT GUI-dependent runs execute only on an
authorized host with their matching Java/architecture and exact process status.

Task 6 freezes Maven/Tycho and validation commands and adds public macOS and
Windows build/test/package CI that requires no ITS secret. It reruns the complete
Java unit/process/PDE/package matrix, applicable authorized local host evidence,
the focused MCP process compatibility tests, and the canonical Rust workspace
gate. It audits dependencies, licenses, target units, exported/internal packages,
bundle imports, category/package inventory, generated files, credentials,
personal paths, application/pool writes, links, current-state documentation,
and unchanged MCP catalog. Every filter must match at least one case.

### First-slice user journey

1. The user installs the local `OneAgent EDT Integration` feature into a
   disposable or supported EDT profile.
2. In `Preferences > OneAgent`, the user keeps `oneagent-mcp` or selects one
   executable file. No credential or argument is configured.
3. The user selects exactly one eligible local EDT configuration project and
   invokes `OneAgent: Probe Runtime Compatibility` from its context menu.
4. EDT remains responsive while one background job runs the bounded probe.
5. The user sees exactly one fixed compatibility or redacted failure dialog.
6. Repetition uses a fresh process; cancellation, preference replacement,
   bundle stop, and host shutdown leave no owned resource.

This proves the native EDT integration boundary and user workflow. It does not
claim that the current full checked-in project builds successfully, that any
semantic tool was called, or that broader EDT capabilities are complete.

### Compatibility and migration

The decision is additive. It changes no Rust, Cargo, MCP, Runtime, Workspace,
Graph, Analysis, Tool Policy, HTTP, CLI, LSP, VS Code extension, provider,
source-adapter, or Coverage Registry behavior. Existing clients and the exact
seven-tool catalog remain unchanged. There is no persisted plug-in state beyond
the standard instance preference.

If a later Runtime version changes discovery, a later ADR must define explicit
version compatibility; this client fails closed. If proprietary EDT services
become necessary, evidence and dependency approval must precede target/import
changes. Removing the feature and its instance preference restores the prior
host behavior; no project file migration is required.

## Consequences

OneAgent gains a reproducible, installable native EDT boundary with explicit
user demand, exact local-project eligibility, bounded process effects, a closed
compatibility response, deterministic cleanup, and path-free presentation.
Public CI remains independent of ITS credentials. The prototype is deliberately
small: a successful dialog proves compatibility only, not semantic usefulness
for the current full configuration.

The strict exact-version response means a future compatible Runtime release
requires a deliberate ADR and client update. The manual JSON parser adds test
surface but avoids a production dependency and duplicate MCP owner. EDT 2026.1
host evidence remains architecture-specific and local, while build evidence is
portable.

## Rejected alternatives

- Proprietary or internal EDT Java APIs are unnecessary for exact nature
  recognition and would make public builds depend on authenticated p2 access.
- A writable local-pool publisher or symlink target violates authorization and
  is not a reproducible repository.
- Standard MCP initialize, tools/list, a semantic call, session state, fallback
  revisions, or retries exceed the compatibility probe.
- A JSON/MCP/process/UI framework adds an unnecessary production dependency and
  second protocol or lifecycle owner.
- Java source parsing, EDT model traversal, or copied semantic projections
  creates a competing semantic authority.
- Startup activation, automatic connection/restart, persistent children,
  watchers, and background refresh hide process effects and complicate cleanup.
- Shell commands, arguments, environment settings, relative executable paths,
  downloads, installation, and PATH mutation expand the execution boundary.
- Multiple selections, one process per project, project aggregation, remote or
  virtual projects, and editor-derived fallbacks are unnecessary for the proof.
- Logging stderr, raw errors, project names, paths, or payloads is incompatible
  with the redacted public contract.
- Mock-only evidence cannot prove the real Runtime or host; the full checked-in
  project cannot currently be the sole positive process oracle.
- Modifying or re-signing either authorized application bundle is prohibited and
  would invalidate compatibility evidence.

## Deferred scope

New Rust or MCP capabilities; semantic tool invocation; Java source reads,
parsing, graph or Context projection; BSL editor navigation, symbol search,
Context UI, chat, diagnostics, edits, LSP4E, file watching, persistence,
automatic startup/restart, multiple simultaneous processes, multi-project,
remote or virtual workspaces, proprietary EDT services, authenticated/private
p2 as a build requirement, target-version ranges, Runtime discovery/download/
bundling/update, bundled JRE, command arguments/environment, additional UI,
logging, telemetry, Marketplace publication, signing, update distribution, and
broad compatibility, quality, performance, or security claims remain deferred.
