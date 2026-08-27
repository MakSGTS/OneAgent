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
    assert.deepEqual(extension.packageJSON.contributes.chatParticipants, [{
      id: "oneagent.chat",
      name: "oneagent",
      fullName: "OneAgent",
      description: "Ask the selected model about explicitly inspected OneAgent semantic context.",
    }]);

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
    assert.equal(commands.includes("oneagent.searchSymbols"), true);
    assert.equal(commands.includes("oneagent.inspectContext"), true);
    assert.equal(
      await vscode.commands.executeCommand("oneagent.searchSymbols"),
      "not_connected",
    );
    assert.equal(
      await vscode.commands.executeCommand("oneagent.inspectContext"),
      "not_connected",
    );
    assert.equal(testApi.chat().registered, true);
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

  test("searches through the public command and opens exact source navigation", async () => {
    const executable = process.env.ONEAGENT_MCP_BIN;
    assert.ok(executable);
    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");

    assert.equal(await vscode.commands.executeCommand("oneagent.searchSymbols"), "shown");
    const procedureSearch = testApi.search();
    assert.ok(procedureSearch);
    assert.deepEqual(procedureSearch.snapshot(), {
      title: "OneAgent: Search Symbols",
      placeholder: "Type a symbol name",
      busy: false,
      disposed: false,
      items: [],
    });
    await procedureSearch.input("FillSecurity");
    assert.deepEqual(procedureSearch.snapshot().items, [
      {
        label: "FillSecurityCollection",
        description: "procedure — DNSWorldEdition",
        detail: "designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl:5",
      },
    ]);
    await procedureSearch.accept(0);
    const modulePath = path.join(
      vscode.workspace.workspaceFolders[0].uri.fsPath,
      "designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl",
    );
    assert.equal(vscode.window.activeTextEditor.document.uri.fsPath, modulePath);
    assert.deepEqual(
      [
        vscode.window.activeTextEditor.selection.start.line,
        vscode.window.activeTextEditor.selection.start.character,
        vscode.window.activeTextEditor.selection.end.line,
        vscode.window.activeTextEditor.selection.end.character,
      ],
      [4, 0, 4, 0],
    );

    assert.equal(await vscode.commands.executeCommand("oneagent.searchSymbols"), "shown");
    const moduleSearch = testApi.search();
    assert.ok(moduleSearch);
    await moduleSearch.input("DynamicSecurityOverridable");
    await moduleSearch.accept(0);
    assert.equal(vscode.window.activeTextEditor.document.uri.fsPath, modulePath);
    assert.equal(vscode.window.activeTextEditor.selection.start.line, 4);

    assert.equal(await vscode.commands.executeCommand("oneagent.searchSymbols"), "shown");
    const missingSearch = testApi.search();
    assert.ok(missingSearch);
    await missingSearch.navigate({
      configurationId: "configuration-id",
      configurationName: "Configuration",
      nodeId: "missing-node",
      name: "Missing",
      kind: "module",
      location: { path: "designer/Missing.bsl" },
    });
    assert.equal(vscode.window.activeTextEditor.document.uri.fsPath, modulePath);
    missingSearch.hide();

    assert.equal(await vscode.commands.executeCommand("oneagent.searchSymbols"), "shown");
    const replaced = testApi.search();
    assert.ok(replaced);
    const replacedInput = replaced.input("FillSecurity");
    assert.equal(await vscode.commands.executeCommand("oneagent.searchSymbols"), "shown");
    assert.equal(replaced.snapshot().disposed, true);
    await replacedInput;
    const replacement = testApi.search();
    assert.ok(replacement);
    await replacement.input("DynamicSecurityOverridable");
    assert.deepEqual(replacement.snapshot().items, [
      {
        label: "DynamicSecurityOverridable",
        description: "module — DNSWorldEdition",
        detail: "designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl",
      },
    ]);
    assertStatus("connected");
    replacement.hide();
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
  });

  test("inspects, reuses, closes, and invalidates Context for the registered participant", async () => {
    const executable = process.env.ONEAGENT_MCP_BIN;
    assert.ok(executable);
    await configuration().update(
      "executable",
      executable,
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.connect"), "connected");

    assert.equal(await vscode.commands.executeCommand("oneagent.inspectContext"), "shown");
    const first = testApi.context();
    assert.ok(first);
    assert.deepEqual(first.snapshot(), {
      title: "OneAgent: Inspect Semantic Context",
      placeholder: "Type a symbol name for semantic context",
      busy: false,
      disposed: false,
      items: [],
    });
    await first.input("FillSecurity");
    assert.deepEqual(first.snapshot().items, [{
      label: "FillSecurityCollection",
      description: "procedure — DNSWorldEdition",
      detail: "designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl:5",
    }]);
    await first.accept(0);
    const firstPanel = testApi.panel();
    assert.equal(firstPanel.open, true);
    assert.equal(firstPanel.createCount, 1);
    assert.ok(firstPanel.html.includes("OneAgent Semantic Context"));
    assert.ok(firstPanel.html.includes("FillSecurityCollection"));
    assert.ok(firstPanel.html.includes("default-src 'none'"));
    assert.equal(/<script|<form|<iframe|command:/iu.test(firstPanel.html), false);

    assert.deepEqual(await testApi.chat().request("Explain it", ["**answer**", " <tag>"]), {
      category: "complete",
      markdown: ["\\*\\*answer\\*\\*", " &lt;tag&gt;"],
    });

    assert.equal(await vscode.commands.executeCommand("oneagent.inspectContext"), "shown");
    const replacement = testApi.context();
    assert.ok(replacement);
    assert.equal(testApi.panel().open, true);
    assert.equal(testApi.panel().createCount, 1);
    await replacement.input("DynamicSecurityOverridable");
    await replacement.accept(0);
    const replacedPanel = testApi.panel();
    assert.equal(replacedPanel.open, true);
    assert.equal(replacedPanel.createCount, 1, "replacement must reuse the owned panel");
    assert.ok(replacedPanel.html.includes("DynamicSecurityOverridable"));

    replacedPanel.close();
    assert.equal(testApi.panel().open, false);
    assert.deepEqual(await testApi.chat().request("late", ["must not run"]), {
      category: "context_required",
      markdown: ["Inspect semantic context before asking OneAgent."],
    });

    assert.equal(await vscode.commands.executeCommand("oneagent.inspectContext"), "shown");
    const beforeDisconnect = testApi.context();
    assert.ok(beforeDisconnect);
    await beforeDisconnect.input("FillSecurity");
    await beforeDisconnect.accept(0);
    assert.equal(testApi.panel().open, true);
    await configuration().update(
      "executable",
      path.join(path.dirname(executable), "replacement-oneagent-mcp"),
      vscode.ConfigurationTarget.Global,
    );
    assert.equal(await vscode.commands.executeCommand("oneagent.disconnect"), "disconnected");
    assert.equal(testApi.panel().open, false);
    assert.deepEqual(await testApi.chat().request("after disconnect", ["must not run"]), {
      category: "context_required",
      markdown: ["Inspect semantic context before asking OneAgent."],
    });
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
      search: true,
      inspect: true,
      semantic: true,
      participant: true,
      configuration: true,
    });
    await assert.rejects(vscode.commands.executeCommand("oneagent.connect"));
    await assert.rejects(vscode.commands.executeCommand("oneagent.disconnect"));
    await assert.rejects(vscode.commands.executeCommand("oneagent.searchSymbols"));
    await assert.rejects(vscode.commands.executeCommand("oneagent.inspectContext"));
    assert.equal(testApi.chat().registered, false);
  });
});
