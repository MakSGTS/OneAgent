import * as vscode from "vscode";

import { statusPresentation } from "./status";

const CONNECT_COMMAND = "oneagent.connect";
const DISCONNECT_COMMAND = "oneagent.disconnect";

export function activate(context: vscode.ExtensionContext): void {
  const status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  const presentation = statusPresentation("disconnected");
  status.text = presentation.text;
  status.tooltip = presentation.tooltip;
  status.command = presentation.command;
  status.show();

  const connect = vscode.commands.registerCommand(CONNECT_COMMAND, () =>
    Promise.resolve("disconnected"),
  );
  const disconnect = vscode.commands.registerCommand(DISCONNECT_COMMAND, () =>
    Promise.resolve("disconnected"),
  );

  context.subscriptions.push(status, connect, disconnect);
}

export function deactivate(): void {}
