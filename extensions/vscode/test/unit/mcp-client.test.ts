import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { PassThrough, Writable } from "node:stream";
import test from "node:test";

import {
  CONTEXT_BUDGET_BYTES,
  CONTEXT_MAX_CANDIDATES,
  CONTEXT_MAX_DEPTH,
  MAX_FRAME_BYTES,
  MAX_JSON_DEPTH,
  MAX_STDERR_BYTES,
  MCP_PROTOCOL_VERSION,
  REQUEST_TIMEOUT_MS,
  RuntimeClient,
  RuntimeClientFailure,
  SHUTDOWN_TIMEOUT_MS,
  type RuntimeProcessFactory,
  type RuntimeScheduler,
  type RuntimeTimer,
} from "../../src/mcp-client";

const TOOL_NAMES = [
  "oneagent.context",
  "oneagent.diagnostics",
  "oneagent.graph",
  "oneagent.impact",
  "oneagent.query",
  "oneagent.refactor.plan",
  "oneagent.symbols",
  "oneagent.validation",
];

class FakeScheduler implements RuntimeScheduler {
  private readonly entries: Array<{
    readonly delay: number;
    readonly callback: () => void;
    cancelled: boolean;
  }> = [];

  public schedule(delay: number, callback: () => void): RuntimeTimer {
    const entry = { delay, callback, cancelled: false };
    this.entries.push(entry);
    return { cancel: () => (entry.cancelled = true) };
  }

  public fire(delay: number): void {
    const entry = this.entries.find((candidate) => !candidate.cancelled && candidate.delay === delay);
    assert.ok(entry, `expected active ${delay} ms timer`);
    entry.cancelled = true;
    entry.callback();
  }

  public get activeCount(): number {
    return this.entries.filter((entry) => !entry.cancelled).length;
  }
}

type Request = {
  readonly id: number;
  readonly method: string;
  readonly params: {
    readonly _meta: Record<string, unknown>;
    readonly name?: string;
    readonly arguments?: Record<string, unknown>;
  };
};

class FakeProcess extends EventEmitter {
  public readonly stdout = new PassThrough();
  public readonly stderr = new PassThrough();
  public readonly requests: Request[] = [];
  public readonly stdin: Writable;
  public killCalls = 0;
  public exitOnEnd = true;
  public exitOnKill = true;

  public constructor(private readonly respond: (process: FakeProcess, request: Request) => void) {
    super();
    this.stdin = new Writable({
      write: (chunk, _encoding, callback) => {
        const request = JSON.parse(String(chunk).trim()) as Request;
        this.requests.push(request);
        this.respond(this, request);
        callback();
      },
      final: (callback) => {
        callback();
        if (this.exitOnEnd) {
          queueMicrotask(() => this.emitExit(0));
        }
      },
    });
  }

  public kill(): boolean {
    this.killCalls += 1;
    if (this.exitOnKill) {
      queueMicrotask(() => this.emitExit(null, "SIGTERM"));
    }
    return true;
  }

  public emitExit(code: number | null, signal: NodeJS.Signals | null = null): void {
    this.emit("exit", code, signal);
  }

  public send(value: unknown, crlf = false): void {
    this.stdout.write(`${JSON.stringify(value)}${crlf ? "\r\n" : "\n"}`);
  }

  public sendRaw(value: Buffer | string): void {
    this.stdout.write(value);
  }
}

function discovery(id: number): unknown {
  return {
    result: {
      cacheScope: "public",
      ttlMs: 0,
      _meta: {
        "io.modelcontextprotocol/serverInfo": { version: "0.1.0", name: "oneagent" },
      },
      capabilities: { tools: {} },
      supportedVersions: [MCP_PROTOCOL_VERSION],
      resultType: "complete",
    },
    id,
    jsonrpc: "2.0",
  };
}

function toolList(id: number): unknown {
  return {
    jsonrpc: "2.0",
    id,
    result: {
      resultType: "complete",
      tools: TOOL_NAMES.map((name) => ({ name })),
      ttlMs: 0,
      cacheScope: "public",
    },
  };
}

function compatibleResponder(process: FakeProcess, request: Request): void {
  process.send(
    request.method === "server/discover" ? discovery(request.id) : toolList(request.id),
    true,
  );
}

function symbolResult(
  id: number,
  structuredContent: unknown = {
    results: [
      {
        configurationId: "configuration-id",
        configurationName: "Configuration",
        nodeId: "node-id",
        name: "SearchProcedure",
        kind: "procedure",
        location: {
          path: "designer/CommonModules/Search/Ext/Module.bsl",
          span: {
            start: { line: 5, column: 1 },
            end: { line: 5, column: 1 },
          },
        },
      },
    ],
    total: 1,
    truncated: false,
  },
): unknown {
  return {
    jsonrpc: "2.0",
    id,
    result: {
      resultType: "complete",
      content: [{ type: "text", text: JSON.stringify(structuredContent) }],
      structuredContent,
    },
  };
}

function contextContent(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    configurationId: "configuration-id",
    rendered: "seed",
    items: [
      {
        nodeId: "node-id",
        name: "SeedProcedure",
        kind: "procedure",
        depth: 0,
        seedId: "node-id",
        reason: "seed",
        relations: [],
        costBytes: 4,
      },
    ],
    budgetBytes: CONTEXT_BUDGET_BYTES,
    usedBytes: 4,
    remainingBytes: CONTEXT_BUDGET_BYTES - 4,
    candidateTruncated: false,
    candidateOmitted: 0,
    budgetTruncated: false,
    budgetOmitted: 0,
    ...overrides,
  };
}

function contextResult(
  id: number,
  structuredContent: unknown = contextContent(),
): unknown {
  return {
    jsonrpc: "2.0",
    id,
    result: {
      resultType: "complete",
      content: [{ type: "text", text: JSON.stringify(structuredContent) }],
      structuredContent,
    },
  };
}

function symbolResponder(process: FakeProcess, request: Request): void {
  if (request.method === "tools/call") {
    process.send(symbolResult(request.id));
  } else {
    compatibleResponder(process, request);
  }
}

function factoryFor(process: FakeProcess): RuntimeProcessFactory {
  return () => process;
}

function assertReleased(process: FakeProcess, scheduler: FakeScheduler): void {
  assert.equal(scheduler.activeCount, 0);
  assert.equal(process.stdout.listenerCount("data"), 0);
  assert.equal(process.stderr.listenerCount("data"), 0);
  assert.equal(process.listenerCount("error"), 0);
  assert.equal(process.listenerCount("exit"), 0);
  assert.equal(process.listenerCount("close"), 0);
}

async function failureCode(promise: Promise<unknown>): Promise<string> {
  try {
    await promise;
  } catch (error) {
    assert.ok(error instanceof RuntimeClientFailure);
    assert.equal(error.message.includes("/private/"), false);
    return error.code;
  }
  assert.fail("expected RuntimeClientFailure");
}

test("performs the accepted sequential handshake and graceful shutdown", async () => {
  const scheduler = new FakeScheduler();
  const process = new FakeProcess(compatibleResponder);
  const states: string[] = [];
  const client = new RuntimeClient({
    processFactory: factoryFor(process),
    scheduler,
    onStateChange: (state) => states.push(state),
  });

  assert.equal(await client.connect("sensitive executable", "/private/sensitive/workspace"), "connected");
  assert.deepEqual(process.requests.map((request) => request.method), ["server/discover", "tools/list"]);
  assert.deepEqual(process.requests.map((request) => request.id), [1, 2]);
  for (const request of process.requests) {
    assert.deepEqual(request.params._meta, {
      "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
      "io.modelcontextprotocol/clientCapabilities": {},
      "io.modelcontextprotocol/clientInfo": { name: "oneagent-vscode", version: "0.1.0" },
    });
  }
  assert.equal(await client.connect("ignored", "ignored"), "connected");
  assert.equal(await client.disconnect(), "disconnected");
  assert.deepEqual(states, ["connecting", "connected", "disconnecting", "disconnected"]);
  assert.equal(process.killCalls, 0);
  assertReleased(process, scheduler);
});

test("calls symbols with exact arguments and validates the bounded result", async () => {
  const process = new FakeProcess(symbolResponder);
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");
  const result = await client.symbols({
    query: "Поиск",
    configurationId: "configuration-id",
    kinds: ["procedure", "function"],
    limit: 2,
  });
  assert.equal(result.results[0]?.name, "SearchProcedure");
  assert.deepEqual(process.requests[2], {
    jsonrpc: "2.0",
    id: 3,
    method: "tools/call",
    params: {
      name: "oneagent.symbols",
      arguments: {
        query: "Поиск",
        configurationId: "configuration-id",
        kinds: ["procedure", "function"],
        limit: 2,
      },
      _meta: {
        "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": {
          name: "oneagent-vscode",
          version: "0.1.0",
        },
      },
    },
  });
  await client.disconnect();
});

test("serializes repeated symbol calls without failing the connected process", async () => {
  let held: Request | undefined;
  const process = new FakeProcess((owner, request) => {
    if (request.method !== "tools/call") {
      compatibleResponder(owner, request);
      return;
    }
    held = request;
  });
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");

  const first = client.symbols({ query: "first" });
  await Promise.resolve();
  assert.equal(process.requests.length, 3);
  const firstRequest = held;
  assert.ok(firstRequest);

  const second = client.symbols({ query: "second" });
  await Promise.resolve();
  assert.equal(process.requests.length, 3, "only one symbol request may be in flight");
  process.send(symbolResult(firstRequest.id));
  await first;
  await Promise.resolve();

  assert.equal(process.requests.length, 4);
  const secondRequest = held;
  assert.ok(secondRequest);
  assert.notEqual(secondRequest.id, firstRequest.id);
  process.send(symbolResult(secondRequest.id));
  await second;
  assert.equal(client.state, "connected");
  await client.disconnect();
});

test("calls context with fixed arguments and accepts repeated reordered bounded results", async () => {
  let contextCalls = 0;
  const process = new FakeProcess((owner, request) => {
    if (request.method !== "tools/call") {
      compatibleResponder(owner, request);
      return;
    }
    contextCalls += 1;
    const content = contextContent();
    const reordered = Object.fromEntries(Object.entries(content).reverse());
    owner.send(contextResult(request.id, contextCalls === 1 ? content : reordered));
  });
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");

  for (let attempt = 0; attempt < 2; attempt += 1) {
    const result = await client.context({
      configurationId: "configuration-id",
      nodeId: "node-id",
    });
    assert.equal(result.items[0]?.nodeId, "node-id");
    assert.equal(result.usedBytes, Buffer.byteLength(result.rendered, "utf8"));
  }
  assert.deepEqual(process.requests[2]?.params.arguments, {
    configurationId: "configuration-id",
    nodeId: "node-id",
    direction: "both",
    maxDepth: CONTEXT_MAX_DEPTH,
    maxCandidates: CONTEXT_MAX_CANDIDATES,
    budgetBytes: CONTEXT_BUDGET_BYTES,
  });
  assert.equal(process.requests[2]?.params.name, "oneagent.context");
  await client.disconnect();
});

test("accepts exactly 32 context items and rejects one over the bound", async () => {
  const items = Array.from({ length: CONTEXT_MAX_CANDIDATES }, (_, index) =>
    index === 0
      ? {
          nodeId: "node-id",
          name: "Seed",
          kind: "procedure",
          depth: 0,
          seedId: "node-id",
          reason: "seed",
          relations: [],
          costBytes: 1,
        }
      : {
          nodeId: `related-${index}`,
          name: `Related ${index}`,
          kind: "module",
          depth: 1,
          seedId: "node-id",
          reason: "related",
          relations: [{ direction: "outgoing", edgeKind: "calls", edgeId: `edge-${index}` }],
          costBytes: 1,
        });
  let content = contextContent({
    rendered: "x".repeat(CONTEXT_MAX_CANDIDATES),
    items,
    usedBytes: CONTEXT_MAX_CANDIDATES,
    remainingBytes: CONTEXT_BUDGET_BYTES - CONTEXT_MAX_CANDIDATES,
  });
  const process = new FakeProcess((owner, request) => {
    if (request.method === "tools/call") {
      owner.send(contextResult(request.id, content));
    } else {
      compatibleResponder(owner, request);
    }
  });
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");
  assert.equal((await client.context({ configurationId: "configuration-id", nodeId: "node-id" })).items.length, 32);
  content = { ...content, items: [...items, items[1]] };
  assert.equal(
    await failureCode(client.context({ configurationId: "configuration-id", nodeId: "node-id" })),
    "protocol_failure",
  );
  assert.equal(client.state, "failed");
});

test("serializes symbols and context through one FIFO and settles queued work on disconnect", async () => {
  let held: Request | undefined;
  const process = new FakeProcess((owner, request) => {
    if (request.method === "tools/call") {
      held = request;
    } else {
      compatibleResponder(owner, request);
    }
  });
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");

  const symbol = client.symbols({ query: "first" });
  await Promise.resolve();
  const context = client.context({ configurationId: "configuration-id", nodeId: "node-id" });
  await Promise.resolve();
  assert.equal(process.requests.length, 3);
  assert.equal(held?.params.name, "oneagent.symbols");
  const symbolRequest = held;
  assert.ok(symbolRequest);
  process.send(symbolResult(symbolRequest.id));
  await symbol;
  await Promise.resolve();
  assert.equal(process.requests.length, 4);
  assert.equal(held?.params.name, "oneagent.context");
  const contextRequest = held;
  assert.ok(contextRequest);
  process.send(contextResult(contextRequest.id));
  await context;

  const pendingCode = failureCode(client.symbols({ query: "pending" }));
  await Promise.resolve();
  const queuedCode = failureCode(client.context({ configurationId: "configuration-id", nodeId: "node-id" }));
  await Promise.resolve();
  const requestCount = process.requests.length;
  assert.equal(await client.disconnect(), "disconnected");
  assert.equal(await pendingCode, "process_exited");
  assert.equal(await queuedCode, "protocol_failure");
  assert.equal(process.requests.length, requestCount);
});

test("rejects invalid context identities locally without touching the process", async () => {
  const process = new FakeProcess(symbolResponder);
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");
  for (const input of [
    { configurationId: "", nodeId: "node-id" },
    { configurationId: "configuration-id", nodeId: " " },
    { configurationId: "x".repeat(65_537), nodeId: "node-id" },
  ]) {
    assert.equal(await failureCode(client.context(input)), "protocol_failure");
  }
  assert.equal(process.requests.length, 2);
  await client.disconnect();
});

test("keeps context tool errors bounded and aborts malformed context results", async () => {
  const toolError = { code: "not_found", message: "The semantic node was not found." };
  const toolProcess = new FakeProcess((owner, request) => {
    if (request.method !== "tools/call") {
      compatibleResponder(owner, request);
      return;
    }
    owner.send({
      jsonrpc: "2.0",
      id: request.id,
      result: {
        resultType: "complete",
        content: [{ type: "text", text: JSON.stringify(toolError) }],
        structuredContent: toolError,
        isError: true,
      },
    });
  });
  const toolClient = new RuntimeClient({ processFactory: factoryFor(toolProcess) });
  await toolClient.connect("runtime", "/workspace");
  assert.equal(
    await failureCode(toolClient.context({ configurationId: "configuration-id", nodeId: "node-id" })),
    "tool_failure",
  );
  assert.equal(toolClient.state, "connected");
  await toolClient.disconnect();

  const base = contextContent();
  const missingItems = Object.fromEntries(
    Object.entries(base).filter(([key]) => key !== "items"),
  );
  for (const malformed of [
    { ...base, extra: true },
    missingItems,
    { ...base, configurationId: "other" },
    { ...base, budgetBytes: CONTEXT_BUDGET_BYTES - 1 },
    { ...base, usedBytes: 3 },
    { ...base, candidateTruncated: true },
    { ...base, budgetOmitted: 1 },
    { ...base, items: [] },
    { ...base, items: [{ ...(base.items as Record<string, unknown>[])[0], kind: "class" }] },
    { ...base, items: [{ ...(base.items as Record<string, unknown>[])[0], seedId: "other" }] },
    { ...base, items: [{ ...(base.items as Record<string, unknown>[])[0], relations: [{}] }] },
    { ...base, items: [{ ...(base.items as Record<string, unknown>[])[0], costBytes: 5 }] },
  ]) {
    const process = new FakeProcess((owner, request) => {
      if (request.method === "tools/call") {
        owner.send(contextResult(request.id, malformed));
      } else {
        compatibleResponder(owner, request);
      }
    });
    const client = new RuntimeClient({ processFactory: factoryFor(process) });
    await client.connect("runtime", "/workspace");
    assert.equal(
      await failureCode(client.context({ configurationId: "configuration-id", nodeId: "node-id" })),
      "protocol_failure",
    );
    assert.equal(client.state, "failed");
  }
});

test("aborts timed-out and exited context operations without exposing inputs", async () => {
  const scheduler = new FakeScheduler();
  const timedOutProcess = new FakeProcess((owner, request) => {
    if (request.method !== "tools/call") {
      compatibleResponder(owner, request);
    }
  });
  const timedOutClient = new RuntimeClient({
    processFactory: factoryFor(timedOutProcess),
    scheduler,
  });
  await timedOutClient.connect("runtime", "/workspace");
  const timedOut = failureCode(
    timedOutClient.context({ configurationId: "configuration-id", nodeId: "node-id" }),
  );
  await Promise.resolve();
  scheduler.fire(REQUEST_TIMEOUT_MS);
  assert.equal(await timedOut, "startup_timeout");
  assert.equal(timedOutClient.state, "failed");

  const exitedProcess = new FakeProcess((owner, request) => {
    if (request.method === "tools/call") {
      owner.emitExit(12);
    } else {
      compatibleResponder(owner, request);
    }
  });
  const exitedClient = new RuntimeClient({ processFactory: factoryFor(exitedProcess) });
  await exitedClient.connect("runtime", "/workspace");
  assert.equal(
    await failureCode(exitedClient.context({ configurationId: "configuration-id", nodeId: "node-id" })),
    "process_exited",
  );
  assert.equal(exitedClient.state, "failed");
});

test("rejects invalid symbol requests locally without touching the process", async () => {
  const process = new FakeProcess(symbolResponder);
  const client = new RuntimeClient({ processFactory: factoryFor(process) });
  await client.connect("runtime", "/workspace");
  for (const input of [
    { query: "" },
    { query: "🙂".repeat(65) },
    { query: "x", kinds: [] },
    { query: "x", kinds: ["module", "module"] },
    { query: "x", limit: 101 },
  ]) {
    assert.equal(
      await failureCode(client.symbols(input as Parameters<RuntimeClient["symbols"]>[0])),
      "protocol_failure",
    );
  }
  assert.equal(process.requests.length, 2);
  await client.disconnect();
});

test("keeps tool errors bounded and closes malformed symbol result processes", async () => {
  const toolError = { code: "invalid_arguments", message: "The semantic tool arguments are invalid." };
  const toolProcess = new FakeProcess((owner, request) => {
    if (request.method !== "tools/call") {
      compatibleResponder(owner, request);
      return;
    }
    owner.send({
      jsonrpc: "2.0",
      id: request.id,
      result: {
        resultType: "complete",
        content: [{ type: "text", text: JSON.stringify(toolError) }],
        structuredContent: toolError,
        isError: true,
      },
    });
  });
  const toolClient = new RuntimeClient({ processFactory: factoryFor(toolProcess) });
  await toolClient.connect("runtime", "/workspace");
  assert.equal(await failureCode(toolClient.symbols({ query: "x" })), "tool_failure");
  assert.equal(toolClient.state, "connected");
  await toolClient.disconnect();

  for (const malformed of [
    { results: [], total: 0, truncated: false, extra: true },
    { results: [], total: 1, truncated: false },
    {
      results: [{
        configurationId: "configuration-id",
        configurationName: "Configuration",
        nodeId: "node-id",
        name: "Node",
        kind: "module",
        location: { path: "../escape.bsl" },
      }],
      total: 1,
      truncated: false,
    },
    {
      results: [{
        configurationId: "configuration-id",
        configurationName: "Configuration",
        nodeId: "node-id",
        name: "Node",
        kind: "procedure",
        location: {
          path: "configuration/Module.bsl",
          span: { start: { line: 0, column: 1 }, end: { line: 1, column: 1 } },
        },
      }],
      total: 1,
      truncated: false,
    },
  ]) {
    const process = new FakeProcess((owner, request) => {
      if (request.method === "tools/call") {
        owner.send(symbolResult(request.id, malformed));
      } else {
        compatibleResponder(owner, request);
      }
    });
    const client = new RuntimeClient({ processFactory: factoryFor(process) });
    await client.connect("runtime", "/workspace");
    assert.equal(await failureCode(client.symbols({ query: "x" })), "protocol_failure");
    assert.equal(client.state, "failed");
  }
});

test("classifies spawn errors without exposing process inputs", async () => {
  const client = new RuntimeClient({
    processFactory: () => {
      throw new Error("/private/sensitive/workspace executable --secret");
    },
  });
  assert.equal(await failureCode(client.connect("secret", "/private/sensitive/workspace")), "spawn_failed");
  assert.equal(client.state, "failed");
});

test("times out a pending startup request and reaps the child", async () => {
  const scheduler = new FakeScheduler();
  const process = new FakeProcess(() => undefined);
  const states: Array<[string, string | undefined]> = [];
  const client = new RuntimeClient({
    processFactory: factoryFor(process),
    scheduler,
    onStateChange: (state, failure) => states.push([state, failure?.code]),
  });
  const connection = client.connect("runtime", "/workspace");
  scheduler.fire(REQUEST_TIMEOUT_MS);
  assert.equal(await failureCode(connection), "startup_timeout");
  assert.equal(client.state, "failed");
  assertReleased(process, scheduler);
  assert.deepEqual(states, [
    ["connecting", undefined],
    ["failed", "startup_timeout"],
  ]);
});

test("classifies failed startup cleanup and releases ownership after a late exit", async () => {
  const scheduler = new FakeScheduler();
  const firstProcess = new FakeProcess(() => undefined);
  firstProcess.exitOnEnd = false;
  firstProcess.exitOnKill = false;
  const secondProcess = new FakeProcess(compatibleResponder);
  const processes = [firstProcess, secondProcess];
  const states: Array<[string, string | undefined]> = [];
  let spawnCalls = 0;
  const client = new RuntimeClient({
    processFactory: () => processes[spawnCalls++] as FakeProcess,
    scheduler,
    onStateChange: (state, failure) => states.push([state, failure?.code]),
  });

  const connection = client.connect("runtime", "/workspace");
  scheduler.fire(REQUEST_TIMEOUT_MS);
  await new Promise<void>((resolve) => setImmediate(resolve));
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  await new Promise<void>((resolve) => setImmediate(resolve));
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);

  assert.equal(await failureCode(connection), "shutdown_failed");
  assert.equal(client.state, "failed");
  assert.equal(firstProcess.killCalls, 1);
  assert.equal(firstProcess.listenerCount("exit"), 1, "the unobserved child remains owned");
  assert.equal(scheduler.activeCount, 0);
  assert.deepEqual(states, [
    ["connecting", undefined],
    ["failed", "shutdown_failed"],
  ]);

  firstProcess.stderr.write("late".padEnd(MAX_STDERR_BYTES + 1, "x"));
  firstProcess.emit("error", new Error("late process error"));
  assert.equal(client.state, "failed");
  assert.equal(scheduler.activeCount, 0);
  assert.deepEqual(states, [
    ["connecting", undefined],
    ["failed", "shutdown_failed"],
  ]);

  firstProcess.emitExit(null, "SIGTERM");
  assertReleased(firstProcess, scheduler);
  assert.equal(await client.connect("runtime", "/workspace"), "connected");
  assert.equal(spawnCalls, 2);
  assert.equal(await client.disconnect(), "disconnected");
});

test("disconnects an in-flight connection without a transient failed state", async () => {
  const process = new FakeProcess(() => undefined);
  const states: string[] = [];
  const client = new RuntimeClient({
    processFactory: factoryFor(process),
    onStateChange: (state) => states.push(state),
  });
  const connection = client.connect("runtime", "/workspace");
  const disconnection = client.disconnect();
  assert.equal(await failureCode(connection), "process_exited");
  assert.equal(await disconnection, "disconnected");
  assert.deepEqual(states, ["connecting", "disconnecting", "disconnected"]);
});

test("rejects malformed, duplicate-key, unmatched, and unsolicited frames", async () => {
  const frames: Array<Buffer | string> = [
    "not-json\n",
    '{"jsonrpc":"2.0","id":1,"id":1,"result":{}}\n',
    '{"jsonrpc":"2.0","id":99,"result":{}}\n',
    '{"jsonrpc":"2.0","method":"notice"}\n',
    Buffer.from([0xff, 0x0a]),
  ];
  for (const frame of frames) {
    const process = new FakeProcess((owner) => owner.sendRaw(frame));
    const client = new RuntimeClient({ processFactory: factoryFor(process) });
    assert.equal(await failureCode(client.connect("runtime", "/workspace")), "protocol_failure");
  }
});

test("accepts the exact frame and depth bounds and rejects one-over values", async () => {
  const exactProcess = new FakeProcess((owner, request) => {
    const value = request.method === "server/discover" ? discovery(request.id) : toolList(request.id);
    const json = JSON.stringify(value);
    owner.sendRaw(`${json}${" ".repeat(MAX_FRAME_BYTES - Buffer.byteLength(json))}\n`);
  });
  const exactClient = new RuntimeClient({ processFactory: factoryFor(exactProcess) });
  assert.equal(await exactClient.connect("runtime", "/workspace"), "connected");
  await exactClient.disconnect();

  const oversized = new FakeProcess((owner) => owner.sendRaw(`${" ".repeat(MAX_FRAME_BYTES + 1)}\n`));
  const oversizedClient = new RuntimeClient({ processFactory: factoryFor(oversized) });
  assert.equal(await failureCode(oversizedClient.connect("runtime", "/workspace")), "protocol_failure");

  const nested = (levels: number): string => `${"[".repeat(levels)}null${"]".repeat(levels)}`;
  for (const [levels, expected] of [
    [MAX_JSON_DEPTH - 1, "incompatible_server"],
    [MAX_JSON_DEPTH, "protocol_failure"],
  ] as const) {
    const process = new FakeProcess((owner, request) => {
      owner.sendRaw(`{"jsonrpc":"2.0","id":${request.id},"result":${nested(levels)}}\n`);
    });
    const client = new RuntimeClient({ processFactory: factoryFor(process) });
    assert.equal(await failureCode(client.connect("runtime", "/workspace")), expected);
  }
});

test("enforces the retained stderr bound without exposing stderr text", async () => {
  const exact = new FakeProcess((owner, request) => {
    if (request.id === 1) {
      owner.stderr.write("x".repeat(MAX_STDERR_BYTES));
    }
    compatibleResponder(owner, request);
  });
  const exactClient = new RuntimeClient({ processFactory: factoryFor(exact) });
  assert.equal(await exactClient.connect("runtime", "/workspace"), "connected");
  await exactClient.disconnect();

  const over = new FakeProcess((owner) => {
    owner.stderr.write("secret".padEnd(MAX_STDERR_BYTES + 1, "x"));
  });
  const overClient = new RuntimeClient({ processFactory: factoryFor(over) });
  assert.equal(await failureCode(overClient.connect("runtime", "/workspace")), "stderr_overflow");
});

test("fails closed when the request identifier reaches safe-integer exhaustion", async () => {
  const atMaximumScheduler = new FakeScheduler();
  const atMaximum = new FakeProcess(compatibleResponder);
  const atMaximumClient = new RuntimeClient({
    processFactory: factoryFor(atMaximum),
    scheduler: atMaximumScheduler,
  });
  (atMaximumClient as unknown as { nextRequestId: number }).nextRequestId = Number.MAX_SAFE_INTEGER;
  assert.equal(await failureCode(atMaximumClient.connect("runtime", "/workspace")), "protocol_failure");
  assert.deepEqual(atMaximum.requests.map((request) => request.id), [Number.MAX_SAFE_INTEGER]);
  assertReleased(atMaximum, atMaximumScheduler);

  const overMaximumScheduler = new FakeScheduler();
  const overMaximum = new FakeProcess(compatibleResponder);
  const overMaximumClient = new RuntimeClient({
    processFactory: factoryFor(overMaximum),
    scheduler: overMaximumScheduler,
  });
  (overMaximumClient as unknown as { nextRequestId: number }).nextRequestId =
    Number.MAX_SAFE_INTEGER + 1;
  assert.equal(await failureCode(overMaximumClient.connect("runtime", "/workspace")), "protocol_failure");
  assert.deepEqual(overMaximum.requests, []);
  assertReleased(overMaximum, overMaximumScheduler);
});

test("classifies incompatible discovery and tool catalogs", async () => {
  for (const incompatibleMethod of ["server/discover", "tools/list"] as const) {
    const process = new FakeProcess((owner, request) => {
      if (request.method === incompatibleMethod) {
        owner.send({ jsonrpc: "2.0", id: request.id, result: { resultType: "complete" } });
      } else {
        compatibleResponder(owner, request);
      }
    });
    const client = new RuntimeClient({ processFactory: factoryFor(process) });
    assert.equal(await failureCode(client.connect("runtime", "/workspace")), "incompatible_server");
  }
});

test("classifies unexpected exit and unterminated EOF", async () => {
  const exiting = new FakeProcess((owner) => owner.emitExit(12));
  const exitingClient = new RuntimeClient({ processFactory: factoryFor(exiting) });
  assert.equal(await failureCode(exitingClient.connect("runtime", "/workspace")), "process_exited");

  const eof = new FakeProcess((owner) => {
    owner.sendRaw('{"jsonrpc":"2.0"');
    owner.emitExit(0);
  });
  const eofClient = new RuntimeClient({ processFactory: factoryFor(eof) });
  assert.equal(await failureCode(eofClient.connect("runtime", "/workspace")), "protocol_failure");
});

test("forces shutdown after the graceful deadline", async () => {
  const scheduler = new FakeScheduler();
  const process = new FakeProcess(compatibleResponder);
  process.exitOnEnd = false;
  const client = new RuntimeClient({ processFactory: factoryFor(process), scheduler });
  await client.connect("runtime", "/workspace");
  const disconnection = client.disconnect();
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  assert.equal(await disconnection, "disconnected");
  assert.equal(process.killCalls, 1);
  assertReleased(process, scheduler);
});

test("fails closed when neither graceful nor forced shutdown observes exit", async () => {
  const scheduler = new FakeScheduler();
  const process = new FakeProcess(compatibleResponder);
  process.exitOnEnd = false;
  process.exitOnKill = false;
  let spawnCalls = 0;
  const client = new RuntimeClient({
    processFactory: () => {
      spawnCalls += 1;
      return process;
    },
    scheduler,
  });
  await client.connect("runtime", "/workspace");
  const disconnection = client.disconnect();
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  await Promise.resolve();
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  assert.equal(await failureCode(disconnection), "shutdown_failed");
  assert.equal(client.state, "failed");
  assert.equal(process.killCalls, 1);
  assert.equal(scheduler.activeCount, 0);
  const reconnection = client.connect("runtime", "/workspace");
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  await Promise.resolve();
  scheduler.fire(SHUTDOWN_TIMEOUT_MS);
  assert.equal(await failureCode(reconnection), "shutdown_failed");
  assert.equal(spawnCalls, 1);
});
