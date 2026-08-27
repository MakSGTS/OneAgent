import assert from "node:assert/strict";
import test from "node:test";

import type {
  SymbolSearchResult,
  SymbolSearchResultItem,
} from "../../src/mcp-client";
import {
  SymbolSearchController,
  isSearchableQuery,
  navigationTarget,
  type SymbolSearchClient,
  type SymbolSearchPresentation,
} from "../../src/symbol-search";

interface Deferred {
  readonly promise: Promise<SymbolSearchResult>;
  readonly resolve: (value: SymbolSearchResult) => void;
  readonly reject: () => void;
}

class ControlledClient implements SymbolSearchClient {
  public readonly calls: string[] = [];
  public readonly deferred: Deferred[] = [];

  public symbols(input: { readonly query: string }): Promise<SymbolSearchResult> {
    this.calls.push(input.query);
    let resolveValue: ((value: SymbolSearchResult) => void) | undefined;
    let rejectValue: (() => void) | undefined;
    const promise = new Promise<SymbolSearchResult>((resolve, reject) => {
      resolveValue = resolve;
      rejectValue = () => reject(new Error("controlled failure"));
    });
    assert.ok(resolveValue);
    assert.ok(rejectValue);
    this.deferred.push({ promise, resolve: resolveValue, reject: rejectValue });
    return promise;
  }
}

class Presentation implements SymbolSearchPresentation {
  public readonly busy: boolean[] = [];
  public readonly results: SymbolSearchResult[] = [];
  public clears = 0;
  public failures = 0;

  public setBusy(busy: boolean): void {
    this.busy.push(busy);
  }

  public clear(): void {
    this.clears += 1;
  }

  public present(result: SymbolSearchResult): void {
    this.results.push(result);
  }

  public failed(): void {
    this.failures += 1;
  }
}

const empty = (total = 0): SymbolSearchResult => ({
  results: [],
  total,
  truncated: total > 0,
});

async function flush(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

test("coalesces sequential requests to the latest query and ignores stale results", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new SymbolSearchController(client, presentation);

  controller.update("first");
  controller.update("second");
  controller.update("latest");
  assert.deepEqual(client.calls, ["first"]);
  assert.equal(presentation.clears, 3);
  client.deferred[0]?.resolve(empty(1));
  await flush();
  assert.deepEqual(client.calls, ["first", "latest"]);
  assert.deepEqual(presentation.results, []);
  client.deferred[1]?.resolve(empty());
  await flush();
  assert.deepEqual(presentation.results, [empty()]);
  assert.deepEqual(presentation.busy, [true, false]);
});

test("clears stale results before starting a new searchable query", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new SymbolSearchController(client, presentation);

  controller.update("first");
  client.deferred[0]?.resolve(empty());
  await flush();
  assert.deepEqual(presentation.results, [empty()]);

  controller.update("second");
  assert.equal(presentation.clears, 2);
  assert.deepEqual(client.calls, ["first", "second"]);
  assert.deepEqual(presentation.results, [empty()]);
  client.deferred[1]?.resolve(empty(1));
  await flush();
  assert.deepEqual(presentation.results, [empty(), empty(1)]);
});

test("treats empty and one-over input as local state and suppresses late completion", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new SymbolSearchController(client, presentation);

  controller.update("active");
  controller.update("");
  controller.update("🙂".repeat(65));
  assert.deepEqual(client.calls, ["active"]);
  assert.equal(presentation.results.length, 2);
  client.deferred[0]?.resolve(empty(1));
  await flush();
  assert.equal(presentation.results.length, 2);
  assert.equal(presentation.failures, 0);
});

test("reports only the active failure and disposal invalidates late work", async () => {
  const firstClient = new ControlledClient();
  const firstPresentation = new Presentation();
  const first = new SymbolSearchController(firstClient, firstPresentation);
  first.update("active");
  firstClient.deferred[0]?.reject();
  await flush();
  assert.equal(firstPresentation.failures, 1);

  const lateClient = new ControlledClient();
  const latePresentation = new Presentation();
  const late = new SymbolSearchController(lateClient, latePresentation);
  late.update("late");
  late.dispose();
  lateClient.deferred[0]?.reject();
  await flush();
  assert.equal(latePresentation.failures, 0);
  assert.deepEqual(latePresentation.busy, [true, false]);
});

test("query bytes and navigation coordinates use the accepted exact boundaries", () => {
  assert.equal(isSearchableQuery(" "), true);
  assert.equal(isSearchableQuery("x".repeat(256)), true);
  assert.equal(isSearchableQuery(""), false);
  assert.equal(isSearchableQuery("x".repeat(257)), false);
  assert.equal(isSearchableQuery("🙂".repeat(64)), true);
  assert.equal(isSearchableQuery("🙂".repeat(65)), false);

  const item = symbol({
    path: "configuration/src/CommonModules/Sales/Module.bsl",
    span: {
      start: { line: 12, column: 3 },
      end: { line: 12, column: 3 },
    },
  });
  assert.deepEqual(navigationTarget(item), {
    pathSegments: ["configuration", "src", "CommonModules", "Sales", "Module.bsl"],
    selection: {
      startLine: 11,
      startCharacter: 2,
      endLine: 11,
      endCharacter: 2,
    },
  });
  assert.deepEqual(navigationTarget(symbol({ path: "configuration/Module.bsl" })), {
    pathSegments: ["configuration", "Module.bsl"],
  });
});

test("navigation rejects absolute, drive, UNC, empty, traversal, malformed and reversed values", () => {
  for (const path of [
    "/absolute.bsl",
    "C:/absolute.bsl",
    "//server/share.bsl",
    "configuration\\Module.bsl",
    "configuration//Module.bsl",
    "configuration/./Module.bsl",
    "configuration/../Module.bsl",
    "configuration/\0/Module.bsl",
  ]) {
    assert.equal(navigationTarget(symbol({ path })), undefined, path);
  }
  assert.equal(
    navigationTarget(symbol({
      path: "configuration/Module.bsl",
      span: { start: { line: 2, column: 1 }, end: { line: 1, column: 1 } },
    })),
    undefined,
  );
});

function symbol(location: SymbolSearchResultItem["location"]): SymbolSearchResultItem {
  return {
    configurationId: "configuration-id",
    configurationName: "Configuration",
    nodeId: "node-id",
    name: "Sales",
    kind: "procedure",
    location,
  };
}
