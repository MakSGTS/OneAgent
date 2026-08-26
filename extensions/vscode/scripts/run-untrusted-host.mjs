import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { downloadAndUnzipVSCode } from "@vscode/test-electron";

const extensionRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const testRoot = path.join(extensionRoot, ".vscode-test");
const userData = path.join(testRoot, "p/u");
const extensionsDirectory = path.join(testRoot, "e/u");
const marker = path.join(testRoot, "spawn-untrusted.marker");
const fixture = path.join(extensionRoot, "test/fixtures/untrusted-workspace");
const probe = path.join(
  extensionRoot,
  "test/fixtures",
  process.platform === "win32" ? "spawn-probe.cmd" : "spawn-probe.sh",
);
const require = createRequire(import.meta.url);
const testCliEntry = require.resolve("@vscode/test-cli");
const testRunner = path.join(path.dirname(testCliEntry), "runner.cjs");
const testFile = path.join(extensionRoot, "test/extension/untrusted-workspace.test.js");

rmSync(userData, { force: true, recursive: true });
rmSync(extensionsDirectory, { force: true, recursive: true });
rmSync(marker, { force: true });

const executable = await downloadAndUnzipVSCode({
  version: "1.134.0",
  cachePath: testRoot,
  extensionDevelopmentPath: extensionRoot,
});
const environment = {
  ...process.env,
  ONEAGENT_HOST_CASE: "untrusted",
  ONEAGENT_SPAWN_MARKER: marker,
  ONEAGENT_SPAWN_PROBE: probe,
  VSCODE_TEST_OPTIONS: JSON.stringify({
    colorDefault: true,
    files: [testFile],
    mochaOpts: { timeout: 20_000 },
    preload: [],
  }),
};
const args = [
  fixture,
  "--no-sandbox",
  "--disable-gpu-sandbox",
  "--disable-updates",
  "--disable-extensions",
  "--skip-welcome",
  "--skip-release-notes",
  "--no-cached-data",
  `--user-data-dir=${userData}`,
  `--extensions-dir=${extensionsDirectory}`,
  `--extensionTestsPath=${testRunner}`,
  `--extensionDevelopmentPath=${extensionRoot}`,
];

const exitCode = await new Promise((resolve, reject) => {
  const child = spawn(executable, args, { env: environment, stdio: "inherit" });
  const timer = setTimeout(() => {
    child.kill();
    reject(new Error("untrusted Extension Host exceeded its 30 second deadline"));
  }, 30_000);
  child.once("error", (error) => {
    clearTimeout(timer);
    reject(error);
  });
  child.once("exit", (code, signal) => {
    clearTimeout(timer);
    if (signal !== null) {
      reject(new Error(`untrusted Extension Host exited by ${signal}`));
    } else {
      resolve(code);
    }
  });
});

assert.equal(exitCode, 0, "untrusted Extension Host tests must pass");
