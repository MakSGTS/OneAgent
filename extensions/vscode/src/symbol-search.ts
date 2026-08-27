import { Buffer } from "node:buffer";

import {
  MAX_SYMBOL_QUERY_BYTES,
  isSafeRelativeSymbolPath,
  type SourceSpan,
  type SymbolSearchResult,
  type SymbolSearchResultItem,
} from "./mcp-client";

export interface SymbolSearchClient {
  symbols(input: { readonly query: string }): Promise<SymbolSearchResult>;
}

export interface SymbolSearchPresentation {
  setBusy(busy: boolean): void;
  clear(): void;
  present(result: SymbolSearchResult): void;
  failed(): void;
}

export interface NavigationTarget {
  readonly pathSegments: readonly string[];
  readonly selection?: {
    readonly startLine: number;
    readonly startCharacter: number;
    readonly endLine: number;
    readonly endCharacter: number;
  };
}

export class SymbolSearchController {
  private generation = 0;
  private queued: string | undefined;
  private running = false;
  private active = true;

  public constructor(
    private readonly client: SymbolSearchClient,
    private readonly presentation: SymbolSearchPresentation,
  ) {}

  public update(query: string): void {
    if (!this.active) {
      return;
    }
    this.generation += 1;
    if (!isSearchableQuery(query)) {
      this.queued = undefined;
      this.presentation.present({ results: [], total: 0, truncated: false });
      return;
    }
    this.presentation.clear();
    this.queued = query;
    void this.drain();
  }

  public dispose(): void {
    if (!this.active) {
      return;
    }
    this.active = false;
    this.generation += 1;
    this.queued = undefined;
    this.presentation.setBusy(false);
  }

  private async drain(): Promise<void> {
    if (this.running || !this.active) {
      return;
    }
    this.running = true;
    this.presentation.setBusy(true);
    try {
      while (this.active && this.queued !== undefined) {
        const query = this.queued;
        const generation = this.generation;
        this.queued = undefined;
        try {
          const result = await this.client.symbols({ query });
          if (this.active && generation === this.generation) {
            this.presentation.present(result);
          }
        } catch {
          if (this.active && generation === this.generation) {
            this.presentation.failed();
          }
        }
      }
    } finally {
      this.running = false;
      if (this.active) {
        this.presentation.setBusy(false);
        if (this.queued !== undefined) {
          void this.drain();
        }
      }
    }
  }
}

export function isSearchableQuery(query: string): boolean {
  const bytes = Buffer.byteLength(query, "utf8");
  return bytes > 0 && bytes <= MAX_SYMBOL_QUERY_BYTES;
}

export function navigationTarget(item: SymbolSearchResultItem): NavigationTarget | undefined {
  if (!isSafeRelativeSymbolPath(item.location.path)) {
    return undefined;
  }
  const pathSegments = item.location.path.split("/");
  const selection = item.location.span === undefined
    ? undefined
    : zeroBasedSelection(item.location.span);
  if (item.location.span !== undefined && selection === undefined) {
    return undefined;
  }
  return selection === undefined ? { pathSegments } : { pathSegments, selection };
}

function zeroBasedSelection(span: SourceSpan): NavigationTarget["selection"] | undefined {
  if (
    !validCoordinate(span.start.line) ||
    !validCoordinate(span.start.column) ||
    !validCoordinate(span.end.line) ||
    !validCoordinate(span.end.column) ||
    span.end.line < span.start.line ||
    (span.end.line === span.start.line && span.end.column < span.start.column)
  ) {
    return undefined;
  }
  return {
    startLine: span.start.line - 1,
    startCharacter: span.start.column - 1,
    endLine: span.end.line - 1,
    endCharacter: span.end.column - 1,
  };
}

function validCoordinate(value: number): boolean {
  return Number.isInteger(value) && value >= 1 && value <= 0xffff_ffff;
}
