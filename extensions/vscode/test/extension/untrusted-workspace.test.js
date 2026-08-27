const assert = require("node:assert/strict");
const fs = require("node:fs");
const vscode = require("vscode");

suite("OneAgent untrusted workspace host", () => {
  test("fails closed and never spawns in Restricted Mode", async () => {
    const marker = process.env.ONEAGENT_SPAWN_MARKER;
    const probe = process.env.ONEAGENT_SPAWN_PROBE;
    assert.equal(process.env.ONEAGENT_HOST_CASE, "untrusted");
    assert.ok(marker);
    assert.ok(probe);
    fs.rmSync(marker, { force: true });
    assert.equal(vscode.workspace.isTrusted, false);

    const extension = vscode.extensions.getExtension("oneagent-dev.oneagent");
    assert.ok(extension);
    assert.equal(extension.isActive, false);
    const configuration = vscode.workspace.getConfiguration("oneagent.runtime");
    await configuration.update("executable", probe, vscode.ConfigurationTarget.Global);
    try {
      assert.equal(
        await vscode.commands.executeCommand("oneagent.inspectContext"),
        "not_connected",
      );
      assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");
      assert.equal(fs.existsSync(marker), false);
    } finally {
      await configuration.update(
        "executable",
        undefined,
        vscode.ConfigurationTarget.Global,
      );
    }
  });
});
