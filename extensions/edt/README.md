# OneAgent EDT Integration

This directory contains the bounded Sprint 34 native Eclipse/1C:EDT prototype.
It contributes one command, `OneAgent: Probe Runtime Compatibility`, for one
selected local EDT configuration project. The command starts the configured
`oneagent-mcp` executable as a fresh child process, performs the closed
`server/discover` compatibility exchange, reports one fixed result, and releases
the process and plug-in lifecycle resources.

The plug-in is an adapter only. Rust remains the authority for Workspace, MCP,
Graph, Analysis, Context, Tool Policy, and semantic behavior. The Java code does
not read project sources or import a proprietary EDT implementation package.

## Supported prototype boundary

- Build launcher: Maven `3.9.16` from the checked-in Maven Wrapper.
- Build JDK: Temurin JDK 25 in local and CI evidence.
- Compiler and bundle execution environment: Java 17.
- Build target: Tycho `5.0.2` and public Eclipse 2023-12 / Eclipse 4.30 p2
  units from the checked-in target definition.
- Verified product host: x86_64 EDT 2026.1 on an x86_64 JDK 17 with the matching
  OpenJFX 17 modules available to the host.
- Public CI: `macos-14` and `windows-latest`, with no ITS credential.

The prototype does not claim semantic commands, editor navigation, Context UI,
chat, diagnostics, edits, LSP4E, automatic Runtime startup, remote or virtual
projects, multiple projects, publication, signing, telemetry, a bundled Runtime,
or a bundled JRE.

## Build and test

Build the public Runtime first, then run the clean Tycho verification with the
real-process gate enabled:

```bash
cargo build -p oneagent-runtime --bin oneagent-mcp
cd extensions/edt
export JAVA_HOME=/absolute/path/to/jdk-25
export ONEAGENT_MCP_EXECUTABLE=/absolute/path/to/oneagent-mcp
export ONEAGENT_MCP_FIXTURE=/absolute/path/to/repository/apps/runtime/tests/fixtures/workspace_service/edt
./mvnw --batch-mode --no-transfer-progress clean verify
java scripts/VerifyPackage.java repositories/com.oneagent.edt.repository/target/repository
```

On Windows, use `mvnw.cmd` from PowerShell or keep `./mvnw` when the commands run
under Bash. `ONEAGENT_MCP_EXECUTABLE` must include the `.exe` suffix. A successful
verification runs 41 tests with zero failures, errors, or skips; one of those
tests starts the real Runtime twice. A skipped real-process test is not complete
acceptance evidence.

The package auditor requires all eight Surefire suites and exactly 41 tests with
zero failures, errors, or skips. It then requires exactly seven repository files,
four p2 content units, one feature, and one production bundle. It verifies the
frozen qualifier, JDK 25 build marker, Java 17 bytecode and execution-environment
capability, public package imports, command and preference contributions,
Apache-2.0 declaration, and the absence of test, Runtime, JRE, JavaFX, native,
credential, and personal-path content.

The local p2 repository is generated at:

```text
extensions/edt/repositories/com.oneagent.edt.repository/target/repository
```

Install `OneAgent EDT Integration` from that directory through the ordinary
Eclipse/EDT install-software flow or a disposable p2 director profile. Do not use
an application bundle or a shared p2 pool as the install destination. Removal
must leave no `com.oneagent.edt` bundle in the disposable profile.

The checked-in target includes the public p2 director application and its ECF
transport so local validation never needs to run a director from an installed
EDT application.
After `clean verify`, run the fail-closed disposable lifecycle oracle with the
generated Tycho launcher and an absent work directory under `local-artifacts`:

```bash
extensions/edt/scripts/verify-disposable-p2.sh \
  /absolute/path/to/jdk-25/bin/java \
  /absolute/path/to/repository/local-artifacts/maven-repository \
  /absolute/path/to/repository/local-artifacts/sprint-34/disposable-p2-run
```

The script accepts only the generated repository and Tycho builder inside this
checkout. It copies the builder configuration into the disposable root, pins
`user.home`, p2 data, configuration, instance, destination, profile, and bundle
pool to that root, and performs install, list, uninstall, and a fresh list.
Existing application bundles, user p2 data, and installed bundle pools are not
valid inputs or destinations. The final check inspects the current disposable
profile state; an unregistered artifact may remain in its disposable bundle-pool
cache until that whole ignored validation directory is discarded.

## EDT host validation

The verified EDT 2026.1 host is x86_64 and therefore uses an x86_64 JDK 17. Its
current environment checker also requires `javafx-swt.jar` under
`${java.home}/lib`. The host VM arguments include the matching OpenJFX 17 SDK:

```text
--module-path /absolute/path/to/javafx-sdk-17/lib
--add-modules=javafx.controls,javafx.fxml,javafx.web
```

Use a disposable JDK copy when the host requires the additional
`lib/javafx-swt.jar`; never modify or re-sign an EDT application bundle. GUI
validation must run sequentially on an authorized host and preserve the real
exit status. A pipeline must enable `set -o pipefail` or inspect every stage.

## Optional authenticated official p2 access

The production target and CI do not use the private 1C repository. Optional
local proprietary experiments may add the current EDT 2026.1 repository through
an untracked Maven settings file. Keep the repository ID identical in the
`server` and profile entries so Maven can associate credentials with the p2
repository:

```xml
<settings xmlns="http://maven.apache.org/SETTINGS/1.2.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.2.0 https://maven.apache.org/xsd/settings-1.2.0.xsd">
  <servers>
    <server>
      <id>oneagent-1c-edt-p2</id>
      <username>${env.ONEC_ITS_USERNAME}</username>
      <password>${env.ONEC_ITS_PASSWORD}</password>
    </server>
  </servers>
  <profiles>
    <profile>
      <id>oneagent-1c-edt-p2</id>
      <repositories>
        <repository>
          <id>oneagent-1c-edt-p2</id>
          <url>https://services.1c.dev/repository/edt-p2/2026.1/</url>
          <layout>p2</layout>
          <releases><enabled>true</enabled></releases>
          <snapshots><enabled>false</enabled></snapshots>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>
```

Set the two environment variables without printing them, then opt in explicitly:

```bash
./mvnw --settings /absolute/path/to/untracked-settings.xml -Poneagent-1c-edt-p2 verify
```

The endpoint required Basic authentication during the Sprint 34 investigation,
while older official setup guidance said extra Maven settings were unnecessary
for EDT 2021.1 and later. Treat access as entitlement- and endpoint-dependent.
Never pass the password on the command line, enable Maven debug logging while
credentials are configured, or commit the settings file. Authenticated p2 access
is not Sprint 34 build or CI evidence.

## Read-only installed p2 pool

An installed p2 pool is not a Tycho repository unless it has repository-level
`content` and `artifacts` metadata. It may be inventoried as read-only evidence:

```bash
export ONEAGENT_P2_POOL=/absolute/path/to/p2/pool
test -d "$ONEAGENT_P2_POOL"
find "$ONEAGENT_P2_POOL/plugins" "$ONEAGENT_P2_POOL/features" -type f -print | sort
```

Do not point a publisher or install destination at that path, create symlinks
into it, or use it as a target-platform repository. These commands do not write
to the pool.

## Provenance, dependencies, and license

The accepted contract is [ADR-0056](../../docs/adr/0056-edt-integration-prototype.md),
with upstream and installed-toolchain provenance in the
[investigation](../../docs/architecture/edt-integration-prototype-investigation.md).
The production bundle has no third-party library dependency: it uses Java 17,
public Eclipse/OSGi packages supplied by the host, and one EDT nature string as
data. Tycho, Maven Wrapper, JUnit, Hamcrest, and Eclipse platform artifacts are
build or test inputs and are not packaged as production dependencies.

The feature declares Apache-2.0 and the repository root `LICENSE` contains the
license text. The p2 output is unsigned and is not published externally.
