import * as vscode from "vscode";

import {
  AiChatController,
  type CancellationSource,
  type CancellationToken,
  type ChatModel,
  type ChatRequest,
  type ChatResponseSink,
  type ModelResponse,
  type SafeMarkdown,
} from "./ai-chat";
import { resolveConnectionTarget } from "./configuration";
import {
  ContextPanelController,
  renderContextPanel,
  type ContextPanelPresentation,
} from "./context-panel";
import { ExtensionLifecycle } from "./lifecycle";
import {
  RuntimeClient,
  type ContextResult,
  type SymbolSearchResult,
  type SymbolSearchResultItem,
} from "./mcp-client";
import { SymbolSearchController, navigationTarget } from "./symbol-search";
import { extensionHostEvidenceEnabled, statusPresentation } from "./status";

const CONNECT_COMMAND = "oneagent.connect";
const DISCONNECT_COMMAND = "oneagent.disconnect";
const SEARCH_SYMBOLS_COMMAND = "oneagent.searchSymbols";
const INSPECT_CONTEXT_COMMAND = "oneagent.inspectContext";
const CHAT_PARTICIPANT_ID = "oneagent.chat";
const CONTEXT_PANEL_TYPE = "oneagent.contextPanel";
const EXECUTABLE_SETTING = "oneagent.runtime.executable";
const SEARCH_TITLE = "OneAgent: Search Symbols";
const SEARCH_PLACEHOLDER = "Type a symbol name";
const NOT_CONNECTED_MESSAGE = "OneAgent must be connected before searching symbols.";
const SEARCH_FAILED_MESSAGE = "OneAgent symbol search failed.";
const OPEN_FAILED_MESSAGE = "OneAgent could not open the selected symbol.";
const INSPECT_TITLE = "OneAgent: Inspect Semantic Context";
const INSPECT_PLACEHOLDER = "Type a symbol name for semantic context";
const CONTEXT_PANEL_TITLE = "OneAgent Semantic Context";
const CONTEXT_NOT_CONNECTED_MESSAGE = "OneAgent must be connected before inspecting semantic context.";
const CONTEXT_SELECTION_FAILED_MESSAGE = "OneAgent semantic Context selection failed.";
const CONTEXT_UNAVAILABLE_MESSAGE = "OneAgent semantic Context is unavailable.";
const CONTEXT_PANEL_FAILED_MESSAGE = "OneAgent semantic Context panel failed.";

let lifecycle: ExtensionLifecycle | undefined;
let ownedDisposables: readonly vscode.Disposable[] | undefined;
let activeSearch: SymbolSearchInvocation | undefined;
let activeContextSearch: ContextSelectionInvocation | undefined;
let contextPanelOwner: VsCodeContextPanelPresentation | undefined;
let contextController: ContextPanelController | undefined;
let chatController: AiChatController | undefined;
let participantHandler: vscode.ChatRequestHandler | undefined;

type OwnedDisposable =
  | "status"
  | "connect"
  | "disconnect"
  | "search"
  | "inspect"
  | "semantic"
  | "participant"
  | "configuration";

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
  readonly context: () => {
    readonly input: (value: string) => Promise<void>;
    readonly accept: (index: number) => Promise<void>;
    readonly hide: () => void;
    readonly snapshot: () => ReturnType<ContextSelectionInvocation["testSnapshot"]>;
  } | undefined;
  readonly panel: () => ReturnType<VsCodeContextPanelPresentation["testSnapshot"]> & {
    readonly close: () => void;
  };
  readonly chat: () => {
    readonly registered: boolean;
    readonly request: (
      prompt: string,
      chunks: readonly string[],
    ) => Promise<{ readonly category: string | undefined; readonly markdown: readonly string[] }>;
  };
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
      clear: () => {
        if (this.active) {
          this.quickPick.items = [];
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

class ContextSelectionInvocation {
  private readonly quickPick = vscode.window.createQuickPick<SymbolQuickPickItem>();
  private readonly searchController: SymbolSearchController;
  private readonly subscriptions: vscode.Disposable[] = [];
  private readonly presentationWaiters = new Set<() => void>();
  private active = true;
  private accepting = false;
  private selectionOperation: Promise<void> | undefined;

  public constructor(
    owner: ExtensionLifecycle,
    private readonly selectionController: ContextPanelController,
    private readonly cancelSelection: () => void,
    private readonly onDisposed: () => void,
  ) {
    this.quickPick.title = INSPECT_TITLE;
    this.quickPick.placeholder = INSPECT_PLACEHOLDER;
    this.quickPick.canSelectMany = false;
    this.quickPick.items = [];
    this.searchController = new SymbolSearchController(owner, {
      setBusy: (busy) => {
        if (this.active) {
          this.quickPick.busy = busy;
        }
      },
      clear: () => {
        if (this.active) {
          this.quickPick.items = [];
        }
      },
      present: (result) => this.present(result),
      failed: () => this.fail(),
    });
    this.subscriptions.push(
      this.quickPick.onDidChangeValue((value) => this.searchController.update(value)),
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
    this.finish(true);
  }

  public testInput(value: string): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }
    const completion = new Promise<void>((resolve) => this.presentationWaiters.add(resolve));
    this.quickPick.value = value;
    this.searchController.update(value);
    return completion;
  }

  public testAccept(index: number): Promise<void> {
    const item = this.quickPick.items[index];
    return item === undefined ? Promise.resolve() : this.accept(item);
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
    void vscode.window.showErrorMessage(CONTEXT_SELECTION_FAILED_MESSAGE);
    this.finish(true);
  }

  private accept(item: SymbolQuickPickItem): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }
    if (this.selectionOperation !== undefined) {
      return this.selectionOperation;
    }
    this.accepting = true;
    this.searchController.dispose();
    this.quickPick.hide();
    this.disposeQuickPick();
    const operation = this.selectionController.select(item.symbol).then(() => undefined).finally(() => {
      this.finish(false);
    });
    this.selectionOperation = operation;
    return operation;
  }

  private finish(cancel: boolean): void {
    if (!this.active) {
      return;
    }
    this.active = false;
    this.searchController.dispose();
    this.disposeQuickPick();
    this.resolvePresentationWaiters();
    if (cancel) {
      this.cancelSelection();
    }
    this.onDisposed();
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

class VsCodeContextPanelPresentation implements ContextPanelPresentation, vscode.Disposable {
  private panel: vscode.WebviewPanel | undefined;
  private panelSubscription: vscode.Disposable | undefined;
  private createCount = 0;
  private panelFailure = false;

  public constructor(private readonly onClosed: () => void) {}

  public clear(): void {
    this.panelFailure = false;
    if (this.panel !== undefined) {
      this.panel.webview.html = emptyPanelHtml();
    }
  }

  public setLoading(_loading: boolean): void {}

  public present(context: ContextResult): void {
    try {
      const panel = this.panel ?? this.createPanel();
      panel.webview.html = renderContextPanel(context);
      panel.reveal(vscode.ViewColumn.Beside, true);
    } catch (error) {
      this.panelFailure = true;
      throw error;
    }
  }

  public failed(): void {
    void vscode.window.showErrorMessage(
      this.panelFailure ? CONTEXT_PANEL_FAILED_MESSAGE : CONTEXT_UNAVAILABLE_MESSAGE,
    );
    this.close();
  }

  public close(): void {
    this.panel?.dispose();
  }

  public dispose(): void {
    this.close();
  }

  public testSnapshot(): { readonly open: boolean; readonly createCount: number; readonly html: string | undefined } {
    return {
      open: this.panel !== undefined,
      createCount: this.createCount,
      html: this.panel?.webview.html,
    };
  }

  private createPanel(): vscode.WebviewPanel {
    const panel = vscode.window.createWebviewPanel(
      CONTEXT_PANEL_TYPE,
      CONTEXT_PANEL_TITLE,
      { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
      {
        enableScripts: false,
        enableForms: false,
        enableCommandUris: false,
        localResourceRoots: [],
      },
    );
    this.createCount += 1;
    this.panel = panel;
    this.panelSubscription = panel.onDidDispose(() => {
      this.panelSubscription?.dispose();
      this.panelSubscription = undefined;
      if (this.panel === panel) {
        this.panel = undefined;
        this.onClosed();
      }
    });
    return panel;
  }
}

function emptyPanelHtml(): string {
  return "<!DOCTYPE html><html><head><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none';\"></head><body></body></html>";
}

function adaptChatRequest(request: vscode.ChatRequest): ChatRequest {
  return {
    prompt: request.prompt,
    command: request.command,
    references: request.references,
    toolReferences: request.toolReferences,
    model: adaptChatModel(request.model),
  };
}

function adaptChatModel(model: vscode.LanguageModelChat): ChatModel {
  return {
    maxInputTokens: model.maxInputTokens,
    countTokens: (message, token) => model.countTokens(
      message as vscode.LanguageModelChatMessage,
      token as vscode.CancellationToken,
    ),
    sendRequest: async (messages, _options, token): Promise<ModelResponse> => {
      const result = await model.sendRequest(
        [...messages] as vscode.LanguageModelChatMessage[],
        undefined,
        token as vscode.CancellationToken,
      );
      return { text: result.text };
    },
  };
}

function adaptChatResponse(response: vscode.ChatResponseStream): ChatResponseSink {
  return {
    markdown: (value) => response.markdown(toMarkdownString(value)),
  };
}

function toMarkdownString(value: SafeMarkdown): vscode.MarkdownString {
  const markdown = new vscode.MarkdownString(value.value, value.supportHtml);
  markdown.isTrusted = value.isTrusted;
  return markdown;
}

function linkedCancellationSource(parent: CancellationToken): CancellationSource {
  const source = new vscode.CancellationTokenSource();
  const subscription = (parent as vscode.CancellationToken).onCancellationRequested(() => source.cancel());
  return {
    token: source.token,
    cancel: () => source.cancel(),
    dispose: () => {
      subscription.dispose();
      source.dispose();
    },
  };
}

function isModelUnavailableError(error: unknown): boolean {
  return error instanceof vscode.LanguageModelError &&
    ["NoPermissions", "Blocked", "NotFound"].includes(error.code);
}

async function invokeTestParticipant(
  handler: vscode.ChatRequestHandler,
  prompt: string,
  chunks: readonly string[],
): Promise<{ readonly category: string | undefined; readonly markdown: readonly string[] }> {
  const markdown: string[] = [];
  const source = new vscode.CancellationTokenSource();
  const model = {
    name: "test model",
    id: "test-model",
    vendor: "oneagent-test",
    family: "test",
    version: "1",
    maxInputTokens: 32_768,
    countTokens: async () => 1,
    sendRequest: async () => ({
      text: (async function* () {
        for (const chunk of chunks) {
          yield chunk;
        }
      })(),
    }),
  } as unknown as vscode.LanguageModelChat;
  try {
    const result = await handler(
      {
        prompt,
        command: undefined,
        references: [],
        toolReferences: [],
        model,
      } as unknown as vscode.ChatRequest,
      { history: [] },
      {
        markdown: (value: string | vscode.MarkdownString) => {
          markdown.push(typeof value === "string" ? value : value.value);
        },
      } as vscode.ChatResponseStream,
      source.token,
    );
    const category = result !== undefined && result !== null && typeof result.metadata?.category === "string"
      ? result.metadata.category
      : undefined;
    return { category, markdown };
  } finally {
    source.dispose();
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
    inspect: false,
    semantic: false,
    participant: false,
    configuration: false,
  };
  let owner: ExtensionLifecycle;
  const invalidateSemanticState = (): void => {
    activeContextSearch?.dispose();
    chatController?.invalidate();
    contextController?.clear();
    contextPanelOwner?.close();
  };
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
    invalidateSemanticState,
    createClient: (onStateChange) => new RuntimeClient({ onStateChange }),
  });
  lifecycle = owner;
  status.show();

  const panelOwner = new VsCodeContextPanelPresentation(() => {
    chatController?.invalidate();
    contextController?.panelClosed();
  });
  const selectionController = new ContextPanelController(owner, panelOwner);
  const chatOwner = new AiChatController({
    isConnected: () => owner.state === "connected",
    selectedContext: () => selectionController.selected,
    createUserMessage: (content) => vscode.LanguageModelChatMessage.User(content),
    createCancellationSource: linkedCancellationSource,
    isModelUnavailableError,
  });
  contextPanelOwner = panelOwner;
  contextController = selectionController;
  chatController = chatOwner;

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
    activeContextSearch?.dispose();
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
  const inspect = vscode.commands.registerCommand(INSPECT_CONTEXT_COMMAND, () => {
    if (owner.state !== "connected") {
      void vscode.window.showInformationMessage(CONTEXT_NOT_CONNECTED_MESSAGE);
      return "not_connected";
    }
    activeSearch?.dispose();
    activeContextSearch?.dispose();
    chatOwner.invalidate();
    selectionController.clear();
    const invocation = new ContextSelectionInvocation(
      owner,
      selectionController,
      () => {
        chatOwner.invalidate();
        selectionController.clear();
        panelOwner.close();
      },
      () => {
        if (activeContextSearch === invocation) {
          activeContextSearch = undefined;
        }
      },
    );
    activeContextSearch = invocation;
    invocation.show();
    return "shown";
  });
  const handler: vscode.ChatRequestHandler = async (request, _chatContext, response, token) => {
    const outcome = await chatOwner.handle(
      adaptChatRequest(request),
      adaptChatResponse(response),
      token,
    );
    return { metadata: { category: outcome.category } };
  };
  participantHandler = handler;
  const participant = vscode.chat.createChatParticipant(CHAT_PARTICIPANT_ID, handler);
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
  const semantic = {
    dispose: () => {
      activeContextSearch?.dispose();
      chatOwner.dispose();
      selectionController.dispose();
      panelOwner.dispose();
    },
  };
  const disposables = [
    own("status", status),
    own("connect", connect),
    own("disconnect", disconnect),
    own("search", search),
    own("inspect", inspect),
    own("semantic", semantic),
    own("participant", participant),
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
      context: () => {
        const invocation = activeContextSearch;
        return invocation === undefined
          ? undefined
          : {
              input: (value) => invocation.testInput(value),
              accept: (index) => invocation.testAccept(index),
              hide: () => invocation.dispose(),
              snapshot: () => invocation.testSnapshot(),
            };
      },
      panel: () => ({
        ...panelOwner.testSnapshot(),
        close: () => panelOwner.close(),
      }),
      chat: () => ({
        registered: participantHandler === handler,
        request: (prompt, chunks) => invokeTestParticipant(handler, prompt, chunks),
      }),
    };
  }
}

export async function deactivate(): Promise<void> {
  const owner = lifecycle;
  const disposables = ownedDisposables;
  const search = activeSearch;
  const contextSearch = activeContextSearch;
  const panel = contextPanelOwner;
  const selection = contextController;
  const chat = chatController;
  try {
    search?.dispose();
    contextSearch?.dispose();
    chat?.dispose();
    selection?.dispose();
    panel?.dispose();
    await owner?.deactivate();
  } finally {
    for (const disposable of [...(disposables ?? [])].reverse()) {
      disposable.dispose();
    }
    lifecycle = undefined;
    ownedDisposables = undefined;
    activeSearch = undefined;
    activeContextSearch = undefined;
    contextPanelOwner = undefined;
    contextController = undefined;
    chatController = undefined;
    participantHandler = undefined;
  }
}
