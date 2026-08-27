import * as vscode from "vscode";

import { resolveConnectionTarget } from "./configuration";
import { ExtensionLifecycle } from "./lifecycle";
import { RuntimeClient, type SymbolSearchResult, type SymbolSearchResultItem } from "./mcp-client";
import { SymbolSearchController, navigationTarget } from "./symbol-search";
import { extensionHostEvidenceEnabled, statusPresentation } from "./status";

const CONNECT_COMMAND = "oneagent.connect";
const DISCONNECT_COMMAND = "oneagent.disconnect";
const SEARCH_SYMBOLS_COMMAND = "oneagent.searchSymbols";
const EXECUTABLE_SETTING = "oneagent.runtime.executable";
const SEARCH_TITLE = "OneAgent: Search Symbols";
const SEARCH_PLACEHOLDER = "Type a symbol name";
const NOT_CONNECTED_MESSAGE = "OneAgent must be connected before searching symbols.";
const SEARCH_FAILED_MESSAGE = "OneAgent symbol search failed.";
const OPEN_FAILED_MESSAGE = "OneAgent could not open the selected symbol.";

let lifecycle: ExtensionLifecycle | undefined;
let ownedDisposables: readonly vscode.Disposable[] | undefined;
let activeSearch: SymbolSearchInvocation | undefined;

type OwnedDisposable = "status" | "connect" | "disconnect" | "search" | "configuration";

interface SymbolQuickPickItem extends vscode.QuickPickItem {
  readonly symbol: SymbolSearchResultItem;
}

interface ExtensionHostTestApi {
  readonly status: () => {
    readonly text: string;
    readonly tooltip: string | undefined;
    readonly command: string | undefined;
    readonly disposed: Readonly<Record<OwnedDisposable, boolean>>;
  };
  readonly search: () => {
    readonly input: (value: string) => Promise<void>;
    readonly accept: (index: number) => Promise<void>;
    readonly navigate: (item: SymbolSearchResultItem) => Promise<void>;
    readonly hide: () => void;
    readonly snapshot: () => {
      readonly title: string | undefined;
      readonly placeholder: string | undefined;
      readonly busy: boolean;
      readonly disposed: boolean;
      readonly items: readonly {
        readonly label: string;
        readonly description: string | undefined;
        readonly detail: string | undefined;
      }[];
    };
  } | undefined;
}

class SymbolSearchInvocation {
  private readonly quickPick = vscode.window.createQuickPick<SymbolQuickPickItem>();
  private readonly controller: SymbolSearchController;
  private readonly subscriptions: vscode.Disposable[] = [];
  private readonly presentationWaiters = new Set<() => void>();
  private active = true;
  private accepting = false;
  private navigationOperation: Promise<void> | undefined;

  public constructor(
    private readonly owner: ExtensionLifecycle,
    private readonly workspaceRoot: vscode.Uri,
    private readonly onDisposed: () => void,
  ) {
    this.quickPick.title = SEARCH_TITLE;
    this.quickPick.placeholder = SEARCH_PLACEHOLDER;
    this.quickPick.canSelectMany = false;
    this.quickPick.items = [];
    this.controller = new SymbolSearchController(owner, {
      setBusy: (busy) => {
        if (this.active) {
          this.quickPick.busy = busy;
        }
      },
      present: (result) => this.present(result),
      failed: () => this.fail(),
    });
    this.subscriptions.push(
      this.quickPick.onDidChangeValue((value) => this.controller.update(value)),
      this.quickPick.onDidAccept(() => {
        const selected = this.quickPick.selectedItems[0];
        if (selected !== undefined) {
          void this.accept(selected);
        }
      }),
      this.quickPick.onDidHide(() => {
        if (!this.accepting) {
          this.dispose();
        }
      }),
    );
  }

  public show(): void {
    this.quickPick.show();
  }

  public dispose(): void {
    if (!this.active) {
      return;
    }
    this.active = false;
    this.controller.dispose();
    this.disposeQuickPick();
    this.resolvePresentationWaiters();
    this.onDisposed();
  }

  public testInput(value: string): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }
    const completion = new Promise<void>((resolve) => this.presentationWaiters.add(resolve));
    this.quickPick.value = value;
    this.controller.update(value);
    return completion;
  }

  public testAccept(index: number): Promise<void> {
    const item = this.quickPick.items[index];
    if (item === undefined) {
      return Promise.resolve();
    }
    return this.accept(item);
  }

  public testNavigate(item: SymbolSearchResultItem): Promise<void> {
    return this.navigate(item);
  }

  public testSnapshot(): {
    readonly title: string | undefined;
    readonly placeholder: string | undefined;
    readonly busy: boolean;
    readonly disposed: boolean;
    readonly items: readonly {
      readonly label: string;
      readonly description: string | undefined;
      readonly detail: string | undefined;
    }[];
  } {
    return {
      title: this.quickPick.title,
      placeholder: this.quickPick.placeholder,
      busy: this.quickPick.busy,
      disposed: !this.active,
      items: this.quickPick.items.map((item) => ({
        label: item.label,
        description: item.description,
        detail: item.detail,
      })),
    };
  }

  private present(result: SymbolSearchResult): void {
    if (!this.active) {
      return;
    }
    this.quickPick.items = result.results.map((symbol) => ({
      label: symbol.name,
      description: `${symbol.kind} — ${symbol.configurationName}`,
      detail: symbol.location.span === undefined
        ? symbol.location.path
        : `${symbol.location.path}:${symbol.location.span.start.line}`,
      symbol,
    }));
    this.resolvePresentationWaiters();
  }

  private fail(): void {
    if (!this.active) {
      return;
    }
    void vscode.window.showErrorMessage(SEARCH_FAILED_MESSAGE);
    this.dispose();
  }

  private accept(item: SymbolQuickPickItem): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }
    if (this.navigationOperation !== undefined) {
      return this.navigationOperation;
    }
    this.accepting = true;
    this.controller.dispose();
    this.quickPick.hide();
    this.disposeQuickPick();
    const operation = this.navigate(item.symbol).finally(() => this.dispose());
    this.navigationOperation = operation;
    return operation;
  }

  private async navigate(item: SymbolSearchResultItem): Promise<void> {
    const target = navigationTarget(item);
    if (target === undefined || !this.active) {
      if (this.active) {
        void vscode.window.showErrorMessage(SEARCH_FAILED_MESSAGE);
      }
      return;
    }
    const uri = vscode.Uri.joinPath(this.workspaceRoot, ...target.pathSegments);
    if (!isConfinedUri(this.workspaceRoot, uri)) {
      void vscode.window.showErrorMessage(SEARCH_FAILED_MESSAGE);
      return;
    }
    try {
      const document = await vscode.workspace.openTextDocument(uri);
      if (!this.active) {
        return;
      }
      const editor = await vscode.window.showTextDocument(document);
      if (!this.active || target.selection === undefined) {
        return;
      }
      const selection = new vscode.Selection(
        target.selection.startLine,
        target.selection.startCharacter,
        target.selection.endLine,
        target.selection.endCharacter,
      );
      editor.selection = selection;
      editor.revealRange(selection, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
    } catch {
      if (this.active) {
        void vscode.window.showErrorMessage(OPEN_FAILED_MESSAGE);
      }
    }
  }

  private disposeQuickPick(): void {
    for (const subscription of this.subscriptions.splice(0).reverse()) {
      subscription.dispose();
    }
    this.quickPick.dispose();
  }

  private resolvePresentationWaiters(): void {
    for (const resolve of this.presentationWaiters) {
      resolve();
    }
    this.presentationWaiters.clear();
  }
}

function isConfinedUri(root: vscode.Uri, candidate: vscode.Uri): boolean {
  const rootPath = root.path.endsWith("/") ? root.path : `${root.path}/`;
  return (
    candidate.scheme === root.scheme &&
    candidate.authority === root.authority &&
    candidate.path.startsWith(rootPath) &&
    candidate.path.length > rootPath.length
  );
}

export function activate(context: vscode.ExtensionContext): void | ExtensionHostTestApi {
  const status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  const disposed: Record<OwnedDisposable, boolean> = {
    status: false,
    connect: false,
    disconnect: false,
    search: false,
    configuration: false,
  };
  let owner: ExtensionLifecycle;
  const renderState = (state: Parameters<typeof statusPresentation>[0]): void => {
    const presentation = statusPresentation(state);
    status.text = presentation.text;
    status.tooltip = presentation.tooltip;
    status.command = presentation.command;
    if (state !== "connected") {
      const invocation = activeSearch;
      queueMicrotask(() => {
        if (activeSearch === invocation && owner.state !== "connected") {
          invocation?.dispose();
        }
      });
    }
  };

  owner = new ExtensionLifecycle({
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
  const search = vscode.commands.registerCommand(SEARCH_SYMBOLS_COMMAND, () => {
    if (owner.state !== "connected") {
      void vscode.window.showInformationMessage(NOT_CONNECTED_MESSAGE);
      return "not_connected";
    }
    const root = vscode.workspace.workspaceFolders?.[0]?.uri;
    if (root?.scheme !== "file") {
      void vscode.window.showInformationMessage(NOT_CONNECTED_MESSAGE);
      return "not_connected";
    }
    activeSearch?.dispose();
    const invocation = new SymbolSearchInvocation(owner, root, () => {
      if (activeSearch === invocation) {
        activeSearch = undefined;
      }
    });
    activeSearch = invocation;
    invocation.show();
    return "shown";
  });
  const configuration = vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration(EXECUTABLE_SETTING)) {
      void owner.configurationChanged();
    }
  });

  const own = (name: OwnedDisposable, disposable: vscode.Disposable): vscode.Disposable => ({
    dispose: () => {
      disposable.dispose();
      disposed[name] = true;
    },
  });
  const disposables = [
    own("status", status),
    own("connect", connect),
    own("disconnect", disconnect),
    own("search", search),
    own("configuration", configuration),
  ];
  ownedDisposables = disposables;
  context.subscriptions.push(...disposables);

  const hostCase = process.env.ONEAGENT_HOST_CASE;
  if (
    extensionHostEvidenceEnabled(
      context.extensionMode === vscode.ExtensionMode.Production,
      hostCase,
    )
  ) {
    return {
      status: () => ({
        text: status.text,
        tooltip: typeof status.tooltip === "string" ? status.tooltip : undefined,
        command: typeof status.command === "string" ? status.command : undefined,
        disposed: { ...disposed },
      }),
      search: () => {
        const invocation = activeSearch;
        return invocation === undefined
          ? undefined
          : {
              input: (value) => invocation.testInput(value),
              accept: (index) => invocation.testAccept(index),
              navigate: (item) => invocation.testNavigate(item),
              hide: () => invocation.dispose(),
              snapshot: () => invocation.testSnapshot(),
            };
      },
    };
  }
}

export async function deactivate(): Promise<void> {
  const owner = lifecycle;
  const disposables = ownedDisposables;
  const search = activeSearch;
  lifecycle = undefined;
  ownedDisposables = undefined;
  activeSearch = undefined;
  try {
    search?.dispose();
    await owner?.deactivate();
  } finally {
    for (const disposable of [...(disposables ?? [])].reverse()) {
      disposable.dispose();
    }
  }
}
