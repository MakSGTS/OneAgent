import { fileURLToPath } from "node:url";
import { rmSync } from "node:fs";
import path from "node:path";

import { defineConfig } from "@vscode/test-cli";

const root = fileURLToPath(new URL(".", import.meta.url));
const runtimeExecutable = path.resolve(
  root,
  "../../target/debug",
  process.platform === "win32" ? "oneagent-mcp.exe" : "oneagent-mcp",
);
const spawnProbe = path.resolve(
  root,
  "test/fixtures",
  process.platform === "win32" ? "spawn-probe.cmd" : "spawn-probe.sh",
);
const shared = {
  version: "1.134.0",
  extensionDevelopmentPath: root,
  mocha: { timeout: 20_000 },
};
const environment = (hostCase) => ({
  ...process.env,
  ONEAGENT_HOST_CASE: hostCase,
  ONEAGENT_MCP_BIN: runtimeExecutable,
  ONEAGENT_SPAWN_MARKER: path.resolve(root, ".vscode-test", `spawn-${hostCase}.marker`),
  ONEAGENT_SPAWN_PROBE: spawnProbe,
});
const isolatedLaunchArgs = (hostCase) => {
  const profileIds = {
    trusted: "t",
    empty: "e",
    virtual: "v",
    "multi-root": "m",
  };
  const profileId = profileIds[hostCase];
  const userData = path.resolve(root, ".vscode-test/p", profileId);
  const extensions = path.resolve(root, ".vscode-test/e", profileId);
  rmSync(userData, { force: true, recursive: true });
  rmSync(extensions, { force: true, recursive: true });
  return [
    "--disable-extensions",
    "--disable-workspace-trust",
    `--user-data-dir=${userData}`,
    `--extensions-dir=${extensions}`,
  ];
};

export default defineConfig([
  {
    ...shared,
    label: "packageActivation",
    files: "test/extension/activation.test.js",
    workspaceFolder: fileURLToPath(
      new URL("test/fixtures/workspace", import.meta.url),
    ),
    env: environment("trusted"),
    launchArgs: isolatedLaunchArgs("trusted"),
  },
  {
    ...shared,
    label: "emptyWorkspace",
    files: "test/extension/unsupported-workspace.test.js",
    env: environment("empty"),
    launchArgs: isolatedLaunchArgs("empty"),
  },
  {
    ...shared,
    label: "virtualWorkspace",
    files: "test/extension/unsupported-workspace.test.js",
    workspaceFolder: fileURLToPath(
      new URL("test/fixtures/virtual.code-workspace", import.meta.url),
    ),
    env: environment("virtual"),
    launchArgs: isolatedLaunchArgs("virtual"),
  },
  {
    ...shared,
    label: "multiRootWorkspace",
    files: "test/extension/unsupported-workspace.test.js",
    workspaceFolder: fileURLToPath(
      new URL("test/fixtures/multi-root.code-workspace", import.meta.url),
    ),
    env: environment("multi-root"),
    launchArgs: isolatedLaunchArgs("multi-root"),
  },
]);
