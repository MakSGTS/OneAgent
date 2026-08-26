import * as vscode from "vscode";

import { resolveConnectionTarget } from "./configuration";
import { ExtensionLifecycle } from "./lifecycle";
import { RuntimeClient } from "./mcp-client";
import { statusPresentation } from "./status";

const CONNECT_COMMAND = "oneagent.connect";
const DISCONNECT_COMMAND = "oneagent.disconnect";
const EXECUTABLE_SETTING = "oneagent.runtime.executable";

let lifecycle: ExtensionLifecycle | undefined;
let ownedDisposables: readonly vscode.Disposable[] | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  const renderState = (state: Parameters<typeof statusPresentation>[0]): void => {
    const presentation = statusPresentation(state);
    status.text = presentation.text;
    status.tooltip = presentation.tooltip;
    status.command = presentation.command;
  };

  const owner = new ExtensionLifecycle({
    readTarget: () => {
      const folders = vscode.workspace.workspaceFolders;
      const resource = folders?.length === 1 ? folders[0]?.uri : undefined;
      return resolveConnectionTarget({
        trusted: vscode.workspace.isTrusted,
        folders: folders?.map((folder) => ({
          scheme: folder.uri.scheme,
          fsPath: folder.uri.fsPath,
        })),
        executable: vscode.workspace
          .getConfiguration("oneagent.runtime", resource)
          .get("executable"),
      });
    },
    renderState,
    createClient: (onStateChange) => new RuntimeClient({ onStateChange }),
  });
  lifecycle = owner;
  status.show();

  const connect = vscode.commands.registerCommand(CONNECT_COMMAND, () => owner.connect());
  const disconnect = vscode.commands.registerCommand(DISCONNECT_COMMAND, () => owner.disconnect());
  const configuration = vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration(EXECUTABLE_SETTING)) {
      void owner.configurationChanged();
    }
  });

  const disposables = [
    status,
    connect,
    disconnect,
    configuration,
  ];
  ownedDisposables = disposables;
  context.subscriptions.push(...disposables);
}

export async function deactivate(): Promise<void> {
  const owner = lifecycle;
  const disposables = ownedDisposables;
  lifecycle = undefined;
  ownedDisposables = undefined;
  try {
    await owner?.deactivate();
  } finally {
    for (const disposable of [...(disposables ?? [])].reverse()) {
      disposable.dispose();
    }
  }
}
