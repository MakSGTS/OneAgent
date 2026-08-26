import { fileURLToPath } from "node:url";

import { defineConfig } from "@vscode/test-cli";

const root = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  label: "packageActivation",
  files: "test/extension/**/*.test.js",
  version: "1.134.0",
  extensionDevelopmentPath: root,
  workspaceFolder: fileURLToPath(
    new URL("test/fixtures/workspace", import.meta.url),
  ),
  launchArgs: ["--disable-extensions", "--disable-workspace-trust"],
  mocha: {
    timeout: 20_000,
  },
});
