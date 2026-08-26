const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

suite("OneAgent package activation", () => {
  const expectedStatus = {
    disconnected: ["$(circle-outline) OneAgent", "OneAgent is disconnected", "oneagent.connect"],
    connecting: ["$(sync~spin) OneAgent", "OneAgent is connecting", undefined],
    connected: ["$(check) OneAgent", "OneAgent is connected", "oneagent.disconnect"],
    disconnecting: ["$(sync~spin) OneAgent", "OneAgent is disconnecting", undefined],
    failed: ["$(error) OneAgent", "OneAgent connection failed", "oneagent.connect"],
  };
  let testApi;
  const workspaceResource = () => vscode.workspace.workspaceFolders?.[0]?.uri;
  const configuration = () =>
    vscode.workspace.getConfiguration("oneagent.runtime", workspaceResource());
  const marker = process.env.ONEAGENT_SPAWN_MARKER;
  const probe = process.env.ONEAGENT_SPAWN_PROBE;
  const workspaceSettings = path.join(
    vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? "",
    ".vscode",
    "settings.json",
  );
  const assertStatus = (state) => {
    assert.ok(testApi);
    const snapshot = testApi.status();
    assert.deepEqual(
      [snapshot.text, snapshot.tooltip, snapshot.command],
      expectedStatus[state],
    );
  };

  suiteTeardown(async () => {
    await configuration().update(
      "executable",
      undefined,
      vscode.ConfigurationTarget.Global,
    );
    await configuration().update(
      "executable",
      undefined,
      vscode.ConfigurationTarget.Workspace,
    );
    if (marker) {
      fs.rmSync(marker, { force: true });
    }
    fs.rmSync(path.dirname(workspaceSettings), { force: true, recursive: true });
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
    testApi = await extension.activate();
    assert.ok(testApi, "Development Host must expose status evidence");
    assertStatus("disconnected");

    const commands = await vscode.commands.getCommands(true);
    assert.equal(commands.includes("oneagent.connect"), true);
    assert.equal(commands.includes("oneagent.disconnect"), true);
  });

  test("rejects invalid configuration before process creation", async () => {
    assert.ok(marker);
    assert.ok(probe);
    fs.rmSync(marker, { force: true });
    await configuration().update("executable", probe, vscode.ConfigurationTarget.Global);
    await configuration().update("executable", "", vscode.ConfigurationTarget.Global);
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");
    assertStatus("failed");
    assert.equal(fs.existsSync(marker), false);
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
  });

  test("applies workspace configuration over the user value", async () => {
    const executable = process.env.ONEAGENT_MCP_BIN;
    assert.ok(executable);
    try {
      await configuration().update(
        "executable",
        path.join(path.dirname(executable), "missing-user-runtime"),
        vscode.ConfigurationTarget.Global,
      );
      await configuration().update(
        "executable",
        executable,
        vscode.ConfigurationTarget.Workspace,
      );
      assert.equal(configuration().get("executable"), executable);
      const connection = vscode.commands.executeCommand("oneagent.connect");
      assertStatus("connecting");
      assert.equal(await connection, "connected");
      assertStatus("connected");
      const disconnection = vscode.commands.executeCommand("oneagent.disconnect");
      assertStatus("disconnecting");
      assert.equal(await disconnection, "disconnected");
      assertStatus("disconnected");
    } finally {
      await configuration().update(
        "executable",
        undefined,
        vscode.ConfigurationTarget.Workspace,
      );
      await configuration().update(
        "executable",
        undefined,
        vscode.ConfigurationTarget.Global,
      );
    }
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
    assertStatus("connected");
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");

    const missing = path.join(path.dirname(executable), "missing-oneagent-mcp");
    await configuration().update(
      "executable",
      missing,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
    assertStatus("disconnected");
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "failed");
    assertStatus("failed");

    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");
    assertStatus("connected");
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
    assertStatus("disconnected");
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
    assertStatus("connected");
    await extensionModule.deactivate();
    await extensionModule.deactivate();
    assert.deepEqual(testApi.status().disposed, {
      status: true,
      connect: true,
      disconnect: true,
      configuration: true,
    });
    await assert.rejects(vscode.commands.executeCommand("oneagent.connect"));
    await assert.rejects(vscode.commands.executeCommand("oneagent.disconnect"));
  });
});
