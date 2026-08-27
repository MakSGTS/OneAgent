import assert from "node:assert/strict";
import test from "node:test";

import {
  ContextPanelController,
  escapeHtmlText,
  renderContextPanel,
  type ContextClient,
  type ContextPanelPresentation,
} from "../../src/context-panel";
import type { ContextRequest, ContextResult } from "../../src/mcp-client";

interface Deferred {
  readonly promise: Promise<ContextResult>;
  readonly resolve: (value: ContextResult) => void;
  readonly reject: () => void;
}

class ControlledClient implements ContextClient {
  public readonly calls: ContextRequest[] = [];
  public readonly deferred: Deferred[] = [];

  public context(input: ContextRequest): Promise<ContextResult> {
    this.calls.push(input);
    let resolveValue: ((value: ContextResult) => void) | undefined;
    let rejectValue: (() => void) | undefined;
    const promise = new Promise<ContextResult>((resolve, reject) => {
      resolveValue = resolve;
      rejectValue = () => reject(new Error("controlled failure"));
    });
    assert.ok(resolveValue);
    assert.ok(rejectValue);
    this.deferred.push({ promise, resolve: resolveValue, reject: rejectValue });
    return promise;
  }
}

class Presentation implements ContextPanelPresentation {
  public readonly loading: boolean[] = [];
  public readonly contexts: ContextResult[] = [];
  public clears = 0;
  public failures = 0;

  public clear(): void {
    this.clears += 1;
  }

  public setLoading(loading: boolean): void {
    this.loading.push(loading);
  }

  public present(context: ContextResult): void {
    this.contexts.push(context);
  }

  public failed(): void {
    this.failures += 1;
  }
}

function context(overrides: Partial<ContextResult> = {}): ContextResult {
  return {
    configurationId: "configuration-id",
    rendered: "seed",
    items: [
      {
        nodeId: "node-id",
        name: "Seed Procedure",
        kind: "procedure",
        depth: 0,
        seedId: "node-id",
        reason: "seed",
        relations: [],
        costBytes: 4,
      },
    ],
    budgetBytes: 16_384,
    usedBytes: 4,
    remainingBytes: 16_380,
    candidateTruncated: false,
    candidateOmitted: 0,
    budgetTruncated: false,
    budgetOmitted: 0,
    ...overrides,
  };
}

async function flush(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

test("selects one canonical seed, snapshots it immutably, and reveals without another call", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new ContextPanelController(client, presentation);
  const operation = controller.select({ configurationId: "configuration-id", nodeId: "node-id" });
  assert.deepEqual(client.calls, [{ configurationId: "configuration-id", nodeId: "node-id" }]);
  assert.deepEqual(presentation.loading, [true]);
  const mutable = context();
  client.deferred[0]?.resolve(mutable);
  assert.equal(await operation, "selected");
  assert.deepEqual(presentation.loading, [true, false]);
  assert.equal(presentation.contexts.length, 1);
  assert.ok(Object.isFrozen(controller.selected));
  assert.ok(Object.isFrozen(controller.selected?.items));
  assert.ok(Object.isFrozen(controller.selected?.items[0]?.relations));
  (mutable.items[0] as { name: string }).name = "mutated";
  assert.equal(controller.selected?.items[0]?.name, "Seed Procedure");
  assert.equal(controller.reveal(), true);
  assert.equal(presentation.contexts.length, 2);
  assert.equal(client.calls.length, 1);
});

test("replacement suppresses stale completion and preserves canonical item order", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new ContextPanelController(client, presentation);
  const first = controller.select({ configurationId: "configuration-id", nodeId: "first" });
  const second = controller.select({ configurationId: "configuration-id", nodeId: "second" });
  client.deferred[1]?.resolve(context({
    rendered: "second",
    usedBytes: 6,
    remainingBytes: 16_378,
    items: [
      {
        nodeId: "second",
        name: "Second",
        kind: "procedure",
        depth: 0,
        seedId: "second",
        reason: "seed",
        relations: [],
        costBytes: 3,
      },
      {
        nodeId: "related",
        name: "Related",
        kind: "module",
        depth: 1,
        seedId: "second",
        reason: "related",
        relations: [{ direction: "incoming", edgeKind: "calls", edgeId: "edge-id" }],
        costBytes: 3,
      },
    ],
  }));
  assert.equal(await second, "selected");
  client.deferred[0]?.resolve(context());
  assert.equal(await first, "stale");
  assert.deepEqual(controller.selected?.items.map((item) => item.nodeId), ["second", "related"]);
  assert.equal(presentation.contexts.length, 1);
  assert.deepEqual(presentation.loading, [true, true, false]);
});

test("failure, malformed state, close, and empty reveal leave no selected context", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new ContextPanelController(client, presentation);
  const failed = controller.select({ configurationId: "configuration-id", nodeId: "node-id" });
  client.deferred[0]?.reject();
  assert.equal(await failed, "failed");
  assert.equal(controller.selected, undefined);
  assert.equal(presentation.failures, 1);
  assert.equal(controller.reveal(), false);

  const malformed = controller.select({ configurationId: "configuration-id", nodeId: "node-id" });
  client.deferred[1]?.resolve({ ...context(), items: undefined } as unknown as ContextResult);
  assert.equal(await malformed, "failed");
  assert.equal(presentation.failures, 2);

  const selected = controller.select({ configurationId: "configuration-id", nodeId: "node-id" });
  client.deferred[2]?.resolve(context());
  assert.equal(await selected, "selected");
  controller.panelClosed();
  assert.equal(controller.selected, undefined);
  assert.equal(controller.reveal(), false);
});

test("dispose is repeatable and invalidates late selection completion", async () => {
  const client = new ControlledClient();
  const presentation = new Presentation();
  const controller = new ContextPanelController(client, presentation);
  const pending = controller.select({ configurationId: "configuration-id", nodeId: "node-id" });
  controller.dispose();
  controller.dispose();
  client.deferred[0]?.resolve(context());
  await flush();
  assert.equal(await pending, "disposed");
  assert.equal(controller.selected, undefined);
  assert.equal(presentation.contexts.length, 0);
  assert.equal(
    await controller.select({ configurationId: "configuration-id", nodeId: "node-id" }),
    "disposed",
  );
  assert.equal(client.calls.length, 1);
});

test("renders every canonical field under a script-free CSP and escapes hostile text", () => {
  const hostile = `&<>"'<script>alert("x")</script> Привет`;
  const value = context({
    configurationId: hostile,
    rendered: hostile,
    candidateTruncated: true,
    candidateOmitted: 2,
    budgetTruncated: true,
    budgetOmitted: 3,
    items: [
      {
        nodeId: hostile,
        name: hostile,
        kind: "unknown",
        depth: 0,
        seedId: hostile,
        reason: "seed",
        relations: [],
        costBytes: 1,
      },
      {
        nodeId: "related",
        name: "Related",
        kind: "module",
        depth: 1,
        seedId: hostile,
        reason: "related",
        relations: [{ direction: "outgoing", edgeKind: "depends_on", edgeId: hostile }],
        costBytes: 2,
      },
    ],
  });
  const html = renderContextPanel(value);
  assert.ok(html.includes("content=\"default-src 'none';\""));
  assert.equal(html.includes("<script>"), false);
  assert.equal(html.includes(hostile), false);
  assert.ok(html.includes("&amp;&lt;&gt;&quot;&#39;&lt;script&gt;"));
  assert.ok(html.includes("Привет"));
  for (const visible of [
    "Configuration ID", "Seed ID", "Budget bytes", "Used bytes", "Remaining bytes",
    "Candidate truncated", "Candidate omitted", "Budget truncated", "Budget omitted",
    "Rendered context", "Name", "Node ID", "Kind", "Depth", "Reason", "Cost bytes",
    "Direction", "Edge kind", "Edge ID", "depends_on", "true", "2", "3",
  ]) {
    assert.ok(html.includes(visible), visible);
  }
  assert.equal(/<script|<form|<iframe|<img|command:|https?:|vscode-resource:/iu.test(html), false);
  assert.equal(escapeHtmlText(hostile), "&amp;&lt;&gt;&quot;&#39;&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; Привет");
});

test("renders the exact 32-item boundary in stable order", () => {
  const items = Array.from({ length: 32 }, (_, index) => ({
    nodeId: index === 0 ? "node-id" : `node-${index}`,
    name: `Item ${index}`,
    kind: "module" as const,
    depth: index === 0 ? 0 : 1,
    seedId: "node-id",
    reason: index === 0 ? "seed" as const : "related" as const,
    relations: index === 0
      ? []
      : [{ direction: "outgoing" as const, edgeKind: "contains" as const, edgeId: `edge-${index}` }],
    costBytes: 1,
  }));
  const html = renderContextPanel(context({ items }));
  assert.equal((html.match(/<dt>Name<\/dt>/gu) ?? []).length, 32);
  assert.ok(html.indexOf("Item 0") < html.indexOf("Item 31"));
});
