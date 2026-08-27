# EDT Integration Prototype Investigation

This investigation records the repository, official upstream, installed
toolchain, and executable-host evidence available before ADR-0056. It does not
accept an architecture, add a dependency, or implement production behavior.

## Status and scope

- The planning baseline is commit `793ad400` (`Plan Sprint 34 EDT Integration
  Prototype`), descended from the completed Sprint 33 review commit
  `19ba2671`.
- Sprint 34 is the unique active target after this investigation starts.
- The bounded question is whether a native Eclipse/EDT command can recognize
  one local EDT configuration project, run the existing public
  `oneagent-mcp` process in that project, perform a compatibility probe, show a
  stable result, and release all owned resources.
- Runtime semantics, the seven-tool catalog, source parsing, graph ownership,
  and existing IDE behavior remain unchanged.
- Credentials, application installations, and the user-authorized p2 pool are
  read-only evidence. Personal absolute paths and credentials are intentionally
  absent from this document.

## Pinned authorities and provenance

The immutable example source is the official
[`1C-Company/dt-example-plugins`](https://github.com/1C-Company/dt-example-plugins)
repository at commit
[`ae9c1f06a01de4f3ee7fe32bf35e284f25e3915f`](https://github.com/1C-Company/dt-example-plugins/commit/ae9c1f06a01de4f3ee7fe32bf35e284f25e3915f),
committed on 2026-07-15. The checkout used for this investigation produced
these content hashes:

| Evidence | SHA-256 |
|---|---|
| `targets/default/default.target` | `054c5e24b72fcdd3fe238b233e630ebfd73fd27f814b9bfa74a9acde1909a4de` |
| `bom/pom.xml` | `adeb2077672bc0994397958403a2695511365f292223011e31428ffd747c0526` |
| `bundles/org.example.ui/META-INF/MANIFEST.MF` | `defd80dc37d2784b544c20630dccda552901bc47f450654649189ef2aa3caca1` |
| `bundles/org.example.ui/plugin.xml` | `ef5b7543a579078644356e54c20a75d31505f91e6f1e0b42cae0ee034cc1dc08` |

That revision uses Maven `3.9.9`, Tycho `5.0.2`, compiler release 25, the
Eclipse 2025-12 p2 repository, an EDT 2026.2 repository, and the conventional
target/bundle/feature/repository/test reactor. It is authoritative evidence for
current 1C plug-in-project conventions, not a requirement to copy its Java 25
execution environment or proprietary imports into this narrower EDT 2026.1
prototype.

The following official pages define the public platform surface used by the
candidate and are living documentation. Exact compatibility must therefore be
proved against pinned artifacts and installed hosts, not inferred from page
wording alone:

- [1C:EDT plug-in development](https://edt.1c.ru/dev/ru/docs/plugins/dev/)
- [1C:EDT extension points](https://edt.1c.ru/dev/ru/docs/plugins/dev/extension-points/)
- [1C:EDT development environment setup](https://edt.1c.ru/dev/ru/docs/plugins/project/env-setup/)
- [1C:EDT public services](https://edt.1c.ru/dev/ru/docs/plugins/dev/public-services/)
- [Eclipse command extension point](https://help.eclipse.org/latest/topic/org.eclipse.platform.doc.isv/reference/extension-points/org_eclipse_ui_commands.html)
- [Eclipse menu extension point](https://help.eclipse.org/latest/topic/org.eclipse.platform.doc.isv/reference/extension-points/org_eclipse_ui_menus.html)
- [Eclipse preference-page extension point](https://help.eclipse.org/latest/topic/org.eclipse.platform.doc.isv/reference/extension-points/org_eclipse_ui_preferencePages.html)
- [Eclipse handler guide](https://help.eclipse.org/latest/topic/org.eclipse.platform.doc.isv/guide/workbench_cmd_handlers.htm)
- [Tycho 5.0.2 target-platform reference](https://tycho.eclipseprojects.io/doc/5.0.2/TargetPlatform.html)

## Repository baseline

- `OneAgent_EDTproject/.project` declares the exact nature
  `com._1c.g5.v8.dt.core.V8ConfigurationNature`.
- `OneAgent_EDTproject/DT-INF/PROJECT.PMF` declares `Runtime-Version: 8.3.27`.
- `apps/runtime/src/bin/oneagent-mcp.rs` starts from the process current
  directory, builds one immutable workspace snapshot, owns stdio, and exits
  successfully on complete EOF or cancellation.
- `apps/runtime/src/mcp.rs` owns LF-delimited JSON framing, a 1,048,576-byte
  frame bound, sequential dispatch, output flushing, cancellation, EOF, and
  stable redacted stderr categories.
- `extensions/vscode/src/mcp-client.ts` is the existing client-lifecycle
  precedent: protocol `2026-07-28`, one pending request, 5,000 ms request
  timeout, 2,000 ms shutdown timeout, a 1,048,576-byte frame bound, correlated
  identifiers, bounded stderr, EOF/exit handling, and deterministic disposal.
- `server/discover` is a stateless compatibility probe. A successful response
  identifies server `oneagent`, version `0.1.0`, protocol `2026-07-28`,
  `capabilities.tools={}`, zero TTL, and public cache scope.
- The immutable catalog remains exactly `oneagent.context`,
  `oneagent.diagnostics`, `oneagent.graph`, `oneagent.impact`,
  `oneagent.query`, `oneagent.symbols`, and `oneagent.validation`.

The full checked-in `OneAgent_EDTproject` is not a positive process oracle at
this revision: starting `target/debug/oneagent-mcp` from that directory fails
with the stable category `oneagent-mcp: workspace build failure` and exit code
1. This is accepted negative evidence and a first-slice user-visible failure
case, not a reason to weaken Runtime startup validation.

The supported reduced fixture
`apps/runtime/tests/fixtures/workspace_service/edt` is a positive Runtime
oracle. The built public process started from that directory, answered
`server/discover` with the compatible values above, and exited 0 on EOF. The
focused public-process test
`public_mcp_process_serves_requests_and_exits_cleanly_on_eof` also passed with
one matched test and no failure.

## Installed product and toolchain inventory

| Component | Observed version | Architecture / execution role |
|---|---:|---|
| Plug-in development Eclipse | Eclipse `4.38.0.I20251120-1800` / 2025-12 | x86_64 native host; requires Java 25 |
| 1C:EDT | `2026.1.2.2`, application/bundle `1.35.2`, Eclipse 4.30 | x86_64 native host; requires Java 17 |
| Maven | `3.9.16` | Build launcher; must be explicitly bound to JDK 25 |
| Temurin JDK 25 | `25.0.4.1` | arm64 build JDK for Maven |
| Temurin JDK 17 | `17.0.20.1+1` | verified portable x86_64 runtime for the x86_64 EDT host |

The architecture boundary is deliberate. Maven/Tycho compilation uses arm64
JDK 25 because the current development Eclipse and official example require
Java 25. The EDT 2026.1 product remains an x86_64 application with a Java 17
requirement, so host validation uses the verified x86_64 JDK 17. Maven commands
must set `JAVA_HOME` and `PATH` explicitly; an ordinary shell may otherwise
select Homebrew JDK 26 and invalidate the claimed matrix. The proposed bundle
may still declare `Bundle-RequiredExecutionEnvironment: JavaSE-17` because its
production bytecode must load in EDT 2026.1.

The installed EDT 2026.1 configuration and read-only p2 pool expose this
relevant exact bundle inventory:

| Bundle | Version |
|---|---:|
| `org.eclipse.core.commands` | `3.11.200` |
| `org.eclipse.core.jobs` | `3.15.100` |
| `org.eclipse.core.resources` | `3.20.0` |
| `org.eclipse.core.runtime` | `3.30.0` |
| `org.eclipse.equinox.common` | `3.18.200` |
| `org.eclipse.jface` | `3.32.0` |
| `org.eclipse.swt` | `3.124.200` |
| `org.eclipse.ui` | `3.205.0` |
| `org.eclipse.ui.ide` | `3.22.0` |
| `org.eclipse.ui.workbench` | `3.131.0` |
| `com._1c.g5.v8.dt.core` | `27.0.2` |
| `com._1c.g5.v8.dt.product.application` | `1.35.2` |

## Public API boundary

Bytecode inspection of the installed Eclipse 4.30 bundles confirms the public
methods required by the candidate:

- `IProject.hasNature(String)` and `IProject.isOpen()` for exact project
  identity and open-state validation;
- `IResource.isAccessible()`, `isLinked()`, `isVirtual()`, `getLocation()`, and
  `getLocationURI()` for accessible local filesystem eligibility;
- `HandlerUtil.getCurrentStructuredSelection(ExecutionEvent)` and
  `IStructuredSelection.size()`, `getFirstElement()`, and `toArray()` for an
  exact one-selection gate;
- `AbstractHandler.isEnabled()`, `setEnabled(Object)`, and `dispose()` for
  command lifecycle;
- `Job.create(...)`, `schedule()`, `cancel()`, state inspection, and `join()`
  for owned background work;
- `AbstractUIPlugin.start(BundleContext)`, `stop(BundleContext)`, and
  `getPreferenceStore()` for OSGi and preference ownership;
- `FieldEditorPreferencePage`, `FileFieldEditor`, and
  `IWorkbenchPreferencePage` for a bounded executable setting; and
- the public workbench window/shell plus SWT `Display` asynchronous execution
  surface for publishing the completed result on the UI thread.

The exact nature string is data, not a Java dependency. A bundle compiled only
against public Eclipse 4.30 APIs can recognize an EDT project without importing
`com._1c.g5.v8.dt.*`. Internal packages, restricted APIs, reflection into EDT,
and copying EDT services are rejected candidates. The public Eclipse 2023-12
p2 repository is the matching reproducible build target for Eclipse 4.30.

## p2, authentication, and dependency findings

- The current EDT 2026.1 setup references
  `https://services.1c.dev/repository/edt-p2/2026.1/`. An unauthenticated probe
  returns HTTP 401 with Basic authentication. The older example-shaped
  `https://edt.1c.ru/downloads/releases/ruby/2026.1/` location returns HTTP 404.
- The official setup page says that no extra Maven settings are required for
  EDT 2021.1 and later. That statement conflicts with the observed current 401
  endpoint. ADR-0056 must not turn this unresolved documentation/authentication
  discrepancy into a repository secret requirement.
- The user-authorized local p2 pool contains installed bundle and feature
  files, but no repository-level `content.jar` or `artifacts.jar`; it is not
  directly a Tycho p2 repository. It remains strictly read-only.
- A discarded publisher experiment through symlinks attempted writes toward
  the pool while returning exit 0. It is invalid evidence and proves that exit
  status alone is insufficient when logs report forbidden effects. The
  experiment will not be repeated.
- The first slice needs no proprietary EDT Java API. It can therefore build
  entirely against public Eclipse 2023-12 with Tycho 5.0.2 and require no ITS
  credential in CI. Authenticated EDT p2 access may be documented as an
  optional local validation input only.
- A dependency-free Java client can use `ProcessBuilder`, buffered byte streams,
  bounded UTF-8 decoding, a small closed JSON parser, futures/executors, and the
  Java 17 standard library. A JSON, MCP, EDT, DI, logging, or UI framework would
  be a new production dependency and requires separate approval; none is
  justified by current evidence.

## Candidate workflow, without architecture acceptance

1. The user selects exactly one open, accessible, local, non-linked,
   non-virtual `IProject` with the exact EDT configuration nature.
2. The command resolves one explicitly configured non-empty executable path.
3. One owned background job starts that executable without a shell or extra
   arguments, using the selected project location as its working directory and
   piped stdio.
4. The client sends one bounded newline-framed `server/discover` request with
   protocol `2026-07-28` and correlates exactly one response.
5. It accepts only the closed compatibility projection, closes stdin, waits for
   clean EOF/exit within fixed bounds, and terminates a failed or cancelled
   process.
6. The UI thread publishes one stable success or redacted failure result.
7. Configuration replacement, cancellation, repeat invocation, bundle stop,
   and host shutdown dispose every process, stream, job, listener, and UI
   resource owned by the plug-in.

This workflow is evidence-backed but remains a candidate until ADR-0056 fixes
the exact identifiers, bounds, messages, and lifecycle transitions.

## Executable oracle matrix

| Area | Required observable cases |
|---|---|
| Selection | one eligible project; empty; missing; inaccessible; closed; non-EDT; linked; virtual; file/folder; multiple selection |
| Configuration | default/unset; valid executable; empty; malformed; missing; directory; inaccessible; configuration change during idle and running states |
| Protocol | compatible discover; malformed JSON; missing/duplicate/unknown ID; wrong result type; incompatible protocol/server/capability; oversized/deep input; duplicate response |
| Process | spawn failure; positive reduced fixture; full-project workspace-build failure; bounded stderr; timeout; cancellation; EOF; non-zero exit; repeated fresh invocation; no orphan |
| Command/UI | enablement agreement; one owned job; blocking work off UI thread; one UI-thread result; repeat invocation policy; deactivation and disposal |
| Package/host | clean Tycho build; exact bundle/feature/repository inventory; disposable install; positive, negative, and repeated EDT-host workflow; uninstall; clean host; unchanged authorized applications/pool |
| CI/security | macOS and Windows build/test/package; no private-p2 secret; dependency/license inventory; no credentials, personal paths, generated local artifacts, or proprietary package imports |

Tests must observe explicit process, job, future, stream, and host completion.
Arbitrary sleeps, zero-match filters, pipeline status without `pipefail`, a
mutable host, mocks alone, or sandbox execution of AppKit/Cocoa-dependent
launchers are not acceptance evidence.

## Host evidence and invalid evidence

The authorized EDT 2026.1 host was launched outside the workspace sandbox with
the verified x86_64 JDK 17. `1cedtcli -command help` and import-help completed
with exit 0. Importing the original reduced fixture timed out because it lacked
`DT-INF/PROJECT.PMF`; the EDT log named that exact missing file. A disposable
copy with `Runtime-Version: 8.3.27` then imported successfully with exit 0.
`project-info RuntimeWritesFixture` completed with exit 0 and returned the
expected project name and disposable location. The import also logged a
non-blocking installed-product error for
`com.e1c.edt.ai.ui.BuildTrackingParticipant`; it is environmental evidence and
must not be misreported as a clean-log result.

Earlier ordinary-sandbox executions of native Eclipse/EDT launchers terminated
with SIGABRT in AppKit `_RegisterApplication`. A pipeline masked at least one
upstream failure as exit 0. Those attempts are explicitly invalid and support
no compatibility conclusion. All future Eclipse, SWT/Cocoa, Electron/VS Code
Extension Host, and `1cedt*` validation must run sequentially on the authorized
host with the exact process status captured. No application bundle was or may
be modified or re-signed.

## Rejected candidates

- Direct imports of proprietary or internal EDT packages are unnecessary for
  exact nature recognition and would make public CI depend on ITS access.
- Treating the local p2 pool as a writable or directly consumable repository is
  inconsistent with its contents and authorization.
- Bundling credentials, a JRE, or `oneagent-mcp` expands the package and secret
  boundary beyond the prototype.
- Parsing EDT sources or reproducing Runtime semantics in Java creates a second
  semantic authority.
- Persistent processes, automatic startup/restart, multi-project aggregation,
  remote/virtual workspaces, editor integration, navigation, Context, chat,
  diagnostics, edits, and file watching are not needed to prove the boundary.
- Shell execution, configurable arguments/environment, PATH mutation, and
  automatic downloads expand the process-execution surface.
- Unit mocks alone cannot prove the real Runtime or EDT host; the checked-in
  full project cannot currently serve as the only positive process fixture.

## Decisions required from ADR-0056

ADR-0056 must select and freeze:

1. bundle, feature, repository, command, menu, preference, and category IDs;
2. the exact public Eclipse 4.30 target and absence of proprietary production
   imports;
3. Java 25 build execution, JavaSE-17 bundle bytecode, and the x86_64 EDT-host
   validation boundary;
4. exact project selection and local/accessibility/nature eligibility rules;
5. executable configuration key, default, validation, and redaction behavior;
6. process executable, arguments, environment, working directory, and ownership;
7. the exact discover request/accepted response projection, parser rules,
   framing and byte/depth/stderr bounds;
8. request, graceful shutdown, forced termination, and job-join timeouts;
9. single-invocation concurrency, repeat, cancellation, configuration-change,
   process-exit, and bundle-stop state transitions;
10. background/UI-thread ownership and exact stable visible outcomes;
11. dependency policy and optional authenticated-p2 handling without CI secrets;
12. bundle/feature/repository packaging, disposable install/uninstall, PDE-host,
    EDT-host, real-process, CI, and clean-host oracles; and
13. first-slice limits, deferred scope, migration impact, and residual risk from
    the current full-project Runtime failure and installed EDT AI plug-in error.

All decisions have a public build path and deterministic oracle. The live p2
authentication discrepancy does not block the dependency-free candidate and
must remain optional unless later repository evidence proves a proprietary API
unavoidable.

## First-slice boundary

The coherent first slice is one dependency-free Java 17 probe client, one
native Eclipse command, exact local EDT-project recognition, explicit Runtime
executable configuration, one owned background operation, one stable UI result,
an installable bundle/feature/p2 repository, public macOS/Windows build evidence,
and disposable PDE/EDT host proof. It adds no Rust capability, semantic tool,
production dependency, proprietary API, automatic activation, persistent
connection, remote behavior, Marketplace publication, signing, telemetry, or
credential storage.

## Validation ledger

| Check | Outcome |
|---|---|
| Official example revision and four file hashes | Exact revision and hashes recorded above |
| Installed product, launcher, Java, Maven, architecture, and bundle inventory | Exact versions and roles recorded above |
| EDT nature and project metadata | Exact nature and Runtime version confirmed in tracked files |
| Public API bytecode inspection | Required public resource, job, handler, preference, UI, and OSGi methods confirmed |
| Runtime reduced-fixture discover/EOF | Exit 0 with compatible response and clean EOF |
| Focused `mcp_process` test | 1 matched, 1 passed, 0 failed |
| Runtime full-project startup | Expected negative: stable workspace-build failure, exit 1 |
| EDT disposable fixture import | Exit 0 after adding required disposable `PROJECT.PMF` |
| EDT `project-info` | Exit 0 with the expected imported project |
| Read-only p2 inventory | Bundles/features present; repository metadata absent; no accepted write |
| Tracked behavior | Documentation-only investigation; no production file changed |
