import assert from "node:assert/strict";
import { access } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import { RuntimeClient, RuntimeClientFailure } from "../../src/mcp-client";

const repositoryRoot = path.resolve(process.cwd(), "../..");
const fixtureRoot = path.join(repositoryRoot, "apps/runtime/tests/fixtures/workspace_service");
const startupFailureRoot = path.join(
  process.cwd(),
  "test/fixtures/runtime-startup-failure",
);

function runtimeExecutable(): string {
  const executable = process.env.ONEAGENT_MCP_BIN;
  assert.ok(executable, "ONEAGENT_MCP_BIN must select the built public Runtime binary");
  return executable;
}

test("public oneagent-mcp handshake and EOF shutdown repeat without orphaned clients", async () => {
  const executable = runtimeExecutable();
  await access(executable);
  await access(fixtureRoot);

  for (let attempt = 0; attempt < 2; attempt += 1) {
    const states: string[] = [];
    const client = new RuntimeClient({ onStateChange: (state) => states.push(state) });
    assert.equal(await client.connect(executable, fixtureRoot), "connected");
    assert.equal(await client.disconnect(), "disconnected");
    assert.deepEqual(states, ["connecting", "connected", "disconnecting", "disconnected"]);
  }
});

test("public process uses the selected cwd and keeps Runtime startup failure redacted", async () => {
  const executable = runtimeExecutable();
  await access(executable);
  await access(startupFailureRoot);
  const client = new RuntimeClient();
  await assert.rejects(client.connect(executable, startupFailureRoot), (error: unknown) => {
    assert.ok(error instanceof RuntimeClientFailure);
    assert.equal(error.code, "process_exited");
    assert.equal(error.message.includes(repositoryRoot), false);
    assert.equal(error.message.includes(startupFailureRoot), false);
    return true;
  });
  assert.equal(client.state, "failed");
});
