import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { PassThrough, Writable } from "node:stream";
import test from "node:test";

import {
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
  process.send(request.method === "server/discover" ? discovery(request.id) : toolList(request.id), true);
}

function factoryFor(process: FakeProcess): RuntimeProcessFactory {
  return () => process;
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
  assert.equal(scheduler.activeCount, 0);
  assert.equal(process.listenerCount("exit"), 0);
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
  assert.equal(scheduler.activeCount, 0);
  assert.equal(process.listenerCount("exit"), 0);
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

  firstProcess.emitExit(null, "SIGTERM");
  assert.equal(firstProcess.listenerCount("exit"), 0);
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
  assert.equal(scheduler.activeCount, 0);
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
