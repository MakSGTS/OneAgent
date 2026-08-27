import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";

const expected = [
  "package.json",
  "README.md",
  "LICENSE",
  "CHANGELOG.md",
  "dist/symbol-search.js",
  "dist/status.js",
  "dist/mcp-client.js",
  "dist/lifecycle.js",
  "dist/extension.js",
  "dist/context-panel.js",
  "dist/configuration.js",
  "dist/ai-chat.js",
];

function inventory() {
  const command = process.platform === "win32" ? "vsce.cmd" : "vsce";
  const result = spawnSync(command, ["ls", "--no-dependencies"], {
    encoding: "utf8",
    env: process.env,
  });
  assert.equal(result.status, 0, "vsce inventory command must succeed");
  assert.equal(result.stderr, "", "vsce inventory must not emit diagnostics");
  return result.stdout.trim().split(/\r?\n/u);
}

const first = inventory();
const second = inventory();
assert.deepEqual(first, expected);
assert.deepEqual(second, expected);
assert.deepEqual(second, first);
process.stdout.write(`validated ${first.length} packaged files\n`);
