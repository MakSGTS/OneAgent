const assert = require("node:assert/strict");
const vscode = require("vscode");

suite("OneAgent package activation", () => {
  test("activates on contributed command and owns package registrations", async () => {
    const extension = vscode.extensions.getExtension("oneagent-dev.oneagent");
    assert.ok(extension, "development extension must be installed");
    assert.equal(extension.packageJSON.engines.vscode, "^1.134.0");
    assert.deepEqual(extension.packageJSON.activationEvents, []);
    assert.equal(extension.packageJSON.extensionKind[0], "workspace");

    const configured = vscode.workspace
      .getConfiguration("oneagent.runtime")
      .get("executable");
    assert.equal(configured, "oneagent-mcp");

    const result = await vscode.commands.executeCommand("oneagent.connect");
    assert.equal(result, "disconnected");
    assert.equal(extension.isActive, true);

    const commands = await vscode.commands.getCommands(true);
    assert.equal(commands.includes("oneagent.connect"), true);
    assert.equal(commands.includes("oneagent.disconnect"), true);
    assert.equal(
      await vscode.commands.executeCommand("oneagent.disconnect"),
      "disconnected",
    );
  });
});
