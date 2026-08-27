const assert = require("node:assert/strict");
const fs = require("node:fs");
const vscode = require("vscode");

suite("OneAgent unsupported workspace hosts", () => {
  const marker = process.env.ONEAGENT_SPAWN_MARKER;
  const probe = process.env.ONEAGENT_SPAWN_PROBE;
  const configuration = () => vscode.workspace.getConfiguration("oneagent.runtime");

  suiteSetup(async () => {
    assert.ok(marker);
    assert.ok(probe);
    fs.rmSync(marker, { force: true });
    await configuration().update("executable", probe, vscode.ConfigurationTarget.Global);
  });

  suiteTeardown(async () => {
    await configuration().update("executable", undefined, vscode.ConfigurationTarget.Global);
    fs.rmSync(marker, { force: true });
  });

  test("the selected unsupported window never spawns", async () => {
    const hostCase = process.env.ONEAGENT_HOST_CASE;
    const folders = vscode.workspace.workspaceFolders;
    if (hostCase === "empty") {
      assert.equal(folders, undefined);
    } else if (hostCase === "virtual") {
      assert.equal(folders?.length, 1);
      assert.equal(folders?.[0]?.uri.scheme, "oneagent-test");
    } else if (hostCase === "multi-root") {
      assert.equal(folders?.length, 2);
      assert.deepEqual(folders?.map((folder) => folder.uri.scheme), ["file", "file"]);
    } else {
      assert.fail("unexpected Extension Host case");
    }
    assert.equal(
      await vscode.commands.executeCommand("oneagent.searchSymbols"),
      "not_connected",
    );
    assert.equal(
      await vscode.commands.executeCommand("oneagent.inspectContext"),
      "not_connected",
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");
    assert.equal(fs.existsSync(marker), false);
  });
});
