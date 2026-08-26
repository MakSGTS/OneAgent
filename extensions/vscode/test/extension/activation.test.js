const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

suite("OneAgent package activation", () => {
  const configuration = () => vscode.workspace.getConfiguration("oneagent.runtime");

  suiteTeardown(async () => {
    await configuration().update(
      "executable",
      undefined,
      vscode.ConfigurationTarget.Global,
    );
  });

  test("activates on contributed command and owns package registrations", async () => {
    const extension = vscode.extensions.getExtension("oneagent-dev.oneagent");
    assert.ok(extension, "development extension must be installed");
    assert.equal(extension.packageJSON.engines.vscode, "^1.134.0");
    assert.deepEqual(extension.packageJSON.activationEvents, []);
    assert.equal(extension.packageJSON.extensionKind[0], "workspace");

    const configured = configuration().get("executable");
    assert.equal(configured, "oneagent-mcp");

    const result = await vscode.commands.executeCommand("oneagent.disconnect");
    assert.equal(result, "disconnected");
    assert.equal(extension.isActive, true);

    const commands = await vscode.commands.getCommands(true);
    assert.equal(commands.includes("oneagent.connect"), true);
    assert.equal(commands.includes("oneagent.disconnect"), true);
  });

  test("rejects invalid configuration before process creation", async () => {
    await configuration().update("executable", "", vscode.ConfigurationTarget.Global);
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
  });

  test("connects, replaces configuration, and reconnects explicitly", async () => {
    const executable = process.env.ONEAGENT_MCP_BIN;
    assert.ok(executable, "test host must receive the built Runtime path");
    assert.equal(path.isAbsolute(executable), true);

    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");

    const missing = path.join(path.dirname(executable), "missing-oneagent-mcp");
    await configuration().update(
      "executable",
      missing,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");

    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
  });

  test("deactivation waits for cleanup and is repeatable", async () => {
    const executable = process.env.ONEAGENT_MCP_BIN;
    assert.ok(executable);
    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");

    const extensionModule = require("../../dist/extension.js");
    await extensionModule.deactivate();
    await extensionModule.deactivate();
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "disconnected");
  });
});
