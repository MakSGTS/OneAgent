import assert from "node:assert/strict";
import { access, cp, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
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
  const workspaceRoot = await mkdtemp(path.join(tmpdir(), "oneagent-mcp-client-"));
  await cp(fixtureRoot, workspaceRoot, { recursive: true });

  try {
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const states: string[] = [];
      const client = new RuntimeClient({ onStateChange: (state) => states.push(state) });
      assert.equal(await client.connect(executable, workspaceRoot), "connected");
      const symbols = await client.symbols({ query: "FillSecurity", limit: 10 });
      assert.equal(symbols.total, 1);
      assert.deepEqual(symbols.results[0], {
        configurationId: "408a41e7-907a-4fb3-8999-83d1e8b6e093",
        configurationName: "DNSWorldEdition",
        nodeId: "dc24575c-a787-411d-93bd-494271291d73:common_module:procedure:FillSecurityCollection",
        name: "FillSecurityCollection",
        kind: "procedure",
        location: {
          path: "designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl",
          span: {
            start: { line: 5, column: 1 },
            end: { line: 5, column: 1 },
          },
        },
      });
      const selected = symbols.results[0];
      assert.ok(selected);
      const context = await client.context({
        configurationId: selected.configurationId,
        nodeId: selected.nodeId,
      });
      assert.equal(context.configurationId, selected.configurationId);
      assert.equal(context.budgetBytes, 16_384);
      assert.equal(context.usedBytes + context.remainingBytes, context.budgetBytes);
      assert.equal(Buffer.byteLength(context.rendered, "utf8"), context.usedBytes);
      assert.equal(
        context.items.reduce((total, item) => total + item.costBytes, 0),
        context.usedBytes,
      );
      assert.deepEqual(
        context.items.filter((item) => item.reason === "seed").map((item) => item.nodeId),
        [selected.nodeId],
      );
      assert.ok(context.items.some((item) => item.reason === "related"));
      assert.equal(await client.disconnect(), "disconnected");
      assert.deepEqual(states, ["connecting", "connected", "disconnecting", "disconnected"]);
    }
  } finally {
    await rm(workspaceRoot, { recursive: true, force: true });
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
