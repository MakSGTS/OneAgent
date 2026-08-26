import { fileURLToPath } from "node:url";
import path from "node:path";

import { defineConfig } from "@vscode/test-cli";

const root = fileURLToPath(new URL(".", import.meta.url));
const runtimeExecutable = path.resolve(
  root,
  "../../target/debug",
  process.platform === "win32" ? "oneagent-mcp.exe" : "oneagent-mcp",
);

export default defineConfig({
  label: "packageActivation",
  files: "test/extension/**/*.test.js",
  version: "1.134.0",
  extensionDevelopmentPath: root,
  workspaceFolder: fileURLToPath(
    new URL("test/fixtures/workspace", import.meta.url),
  ),
  env: { ...process.env, ONEAGENT_MCP_BIN: runtimeExecutable },
  launchArgs: ["--disable-extensions", "--disable-workspace-trust"],
  mocha: {
    timeout: 20_000,
  },
});
