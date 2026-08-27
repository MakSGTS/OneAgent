import type {
  ContextRequest,
  ContextResult,
  ContextResultItem,
  SymbolSearchResultItem,
} from "./mcp-client";

export interface ContextClient {
  context(input: ContextRequest): Promise<ContextResult>;
}

export interface ContextPanelPresentation {
  clear(): void;
  setLoading(loading: boolean): void;
  present(context: ContextResult): void;
  failed(): void;
}

export type ContextSelectionOutcome = "selected" | "stale" | "failed" | "disposed";

export class ContextPanelController {
  private generation = 0;
  private active = true;
  private selectedContext: ContextResult | undefined;

  public constructor(
    private readonly client: ContextClient,
    private readonly presentation: ContextPanelPresentation,
  ) {}

  public get selected(): ContextResult | undefined {
    return this.selectedContext;
  }

  public async select(
    symbol: Pick<SymbolSearchResultItem, "configurationId" | "nodeId">,
  ): Promise<ContextSelectionOutcome> {
    if (!this.active) {
      return "disposed";
    }
    this.generation += 1;
    const generation = this.generation;
    this.selectedContext = undefined;
    this.presentation.clear();
    this.presentation.setLoading(true);
    try {
      const context = await this.client.context({
        configurationId: symbol.configurationId,
        nodeId: symbol.nodeId,
      });
      if (!this.active || generation !== this.generation) {
        return this.active ? "stale" : "disposed";
      }
      const snapshot = immutableContext(context);
      this.selectedContext = snapshot;
      this.presentation.present(snapshot);
      return "selected";
    } catch {
      if (!this.active || generation !== this.generation) {
        return this.active ? "stale" : "disposed";
      }
      this.selectedContext = undefined;
      this.presentation.failed();
      return "failed";
    } finally {
      if (this.active && generation === this.generation) {
        this.presentation.setLoading(false);
      }
    }
  }

  public reveal(): boolean {
    if (!this.active || this.selectedContext === undefined) {
      return false;
    }
    this.presentation.present(this.selectedContext);
    return true;
  }

  public clear(): void {
    if (!this.active) {
      return;
    }
    this.generation += 1;
    this.selectedContext = undefined;
    this.presentation.setLoading(false);
    this.presentation.clear();
  }

  public panelClosed(): void {
    this.clear();
  }

  public dispose(): void {
    if (!this.active) {
      return;
    }
    this.clear();
    this.active = false;
  }
}

export function renderContextPanel(context: ContextResult): string {
  const seed = context.items.find((item) => item.reason === "seed");
  if (seed === undefined) {
    throw new Error("Context panel requires one seed item.");
  }
  const items = context.items.map(renderItem).join("");
  return [
    "<!DOCTYPE html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="UTF-8">',
    '<meta http-equiv="Content-Security-Policy" content="default-src \'none\';">',
    "<title>OneAgent Semantic Context</title>",
    "</head>",
    "<body>",
    "<h1>OneAgent Semantic Context</h1>",
    "<dl>",
    `<dt>Configuration ID</dt><dd>${escapeHtmlText(context.configurationId)}</dd>`,
    `<dt>Seed ID</dt><dd>${escapeHtmlText(seed.nodeId)}</dd>`,
    `<dt>Budget bytes</dt><dd>${context.budgetBytes}</dd>`,
    `<dt>Used bytes</dt><dd>${context.usedBytes}</dd>`,
    `<dt>Remaining bytes</dt><dd>${context.remainingBytes}</dd>`,
    `<dt>Candidate truncated</dt><dd>${context.candidateTruncated}</dd>`,
    `<dt>Candidate omitted</dt><dd>${context.candidateOmitted}</dd>`,
    `<dt>Budget truncated</dt><dd>${context.budgetTruncated}</dd>`,
    `<dt>Budget omitted</dt><dd>${context.budgetOmitted}</dd>`,
    "</dl>",
    "<h2>Rendered context</h2>",
    `<pre>${escapeHtmlText(context.rendered)}</pre>`,
    "<h2>Items</h2>",
    `<ol>${items}</ol>`,
    "</body>",
    "</html>",
  ].join("");
}

export function escapeHtmlText(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function immutableContext(context: ContextResult): ContextResult {
  const items = Object.freeze(context.items.map((item) => Object.freeze({
    nodeId: item.nodeId,
    name: item.name,
    kind: item.kind,
    depth: item.depth,
    seedId: item.seedId,
    reason: item.reason,
    relations: Object.freeze(item.relations.map((relation) => Object.freeze({
      direction: relation.direction,
      edgeKind: relation.edgeKind,
      edgeId: relation.edgeId,
    }))),
    costBytes: item.costBytes,
  })));
  return Object.freeze({
    configurationId: context.configurationId,
    rendered: context.rendered,
    items,
    budgetBytes: context.budgetBytes,
    usedBytes: context.usedBytes,
    remainingBytes: context.remainingBytes,
    candidateTruncated: context.candidateTruncated,
    candidateOmitted: context.candidateOmitted,
    budgetTruncated: context.budgetTruncated,
    budgetOmitted: context.budgetOmitted,
  });
}

function renderItem(item: ContextResultItem): string {
  const relations = item.relations.map((relation) => [
    "<li><dl>",
    `<dt>Direction</dt><dd>${escapeHtmlText(relation.direction)}</dd>`,
    `<dt>Edge kind</dt><dd>${escapeHtmlText(relation.edgeKind)}</dd>`,
    `<dt>Edge ID</dt><dd>${escapeHtmlText(relation.edgeId)}</dd>`,
    "</dl></li>",
  ].join("")).join("");
  return [
    "<li><dl>",
    `<dt>Name</dt><dd>${escapeHtmlText(item.name)}</dd>`,
    `<dt>Node ID</dt><dd>${escapeHtmlText(item.nodeId)}</dd>`,
    `<dt>Kind</dt><dd>${escapeHtmlText(item.kind)}</dd>`,
    `<dt>Depth</dt><dd>${item.depth}</dd>`,
    `<dt>Seed ID</dt><dd>${escapeHtmlText(item.seedId)}</dd>`,
    `<dt>Reason</dt><dd>${escapeHtmlText(item.reason)}</dd>`,
    `<dt>Cost bytes</dt><dd>${item.costBytes}</dd>`,
    "</dl>",
    `<h3>Relations</h3><ol>${relations}</ol>`,
    "</li>",
  ].join("");
}
