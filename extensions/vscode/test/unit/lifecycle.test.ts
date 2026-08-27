import assert from "node:assert/strict";
import test from "node:test";

import type { ConnectionTarget } from "../../src/configuration";
import {
  ExtensionLifecycle,
  type RuntimeClientPort,
} from "../../src/lifecycle";
import {
  RuntimeClientFailure,
  type ContextRequest,
  type ContextResult,
  type SymbolSearchRequest,
  type SymbolSearchResult,
} from "../../src/mcp-client";
import type { ConnectionState } from "../../src/status";

class ControlledClient implements RuntimeClientPort {
  public connectCalls: Array<readonly [string, string]> = [];
  public disconnectCalls = 0;
  public connectFailure: RuntimeClientFailure | undefined;
  public disconnectFailure: RuntimeClientFailure | undefined;
  public symbolCalls: SymbolSearchRequest[] = [];
  public contextCalls: ContextRequest[] = [];
  public holdConnect = false;
  private connectCompletion: (() => void) | undefined;
  private connectCancelled = false;

  public constructor(
    private readonly stateChanged: (state: ConnectionState, failure?: RuntimeClientFailure) => void,
  ) {}

  public async connect(executable: string, cwd: string): Promise<ConnectionState> {
    this.connectCancelled = false;
    this.connectCalls.push([executable, cwd]);
    this.stateChanged("connecting");
    if (this.holdConnect) {
      await new Promise<void>((resolve) => (this.connectCompletion = resolve));
    }
    if (this.connectCancelled) {
      throw new RuntimeClientFailure("process_exited");
    }
    if (this.connectFailure !== undefined) {
      this.stateChanged("failed", this.connectFailure);
      throw this.connectFailure;
    }
    this.stateChanged("connected");
    return "connected";
  }

  public async disconnect(): Promise<ConnectionState> {
    this.disconnectCalls += 1;
    this.stateChanged("disconnecting");
    this.connectCancelled = true;
    this.connectCompletion?.();
    if (this.disconnectFailure !== undefined) {
      this.stateChanged("failed", this.disconnectFailure);
      throw this.disconnectFailure;
    }
    this.stateChanged("disconnected");
    return "disconnected";
  }

  public async symbols(input: SymbolSearchRequest): Promise<SymbolSearchResult> {
    this.symbolCalls.push(input);
    return { results: [], total: 0, truncated: false };
  }

  public async context(input: ContextRequest): Promise<ContextResult> {
    this.contextCalls.push(input);
    return contextResult();
  }
}

function supportedTarget(): ConnectionTarget {
  return { ok: true, executable: "oneagent-mcp", cwd: "/workspace" };
}

function harness(initialTarget: ConnectionTarget = supportedTarget()): {
  readonly lifecycle: ExtensionLifecycle;
  readonly client: ControlledClient;
  readonly states: ConnectionState[];
  readonly invalidations: () => number;
  setTarget(target: ConnectionTarget): void;
} {
  let target = initialTarget;
  const states: ConnectionState[] = [];
  let invalidationCount = 0;
  let client: ControlledClient | undefined;
  const lifecycle = new ExtensionLifecycle({
    readTarget: () => target,
    renderState: (state) => states.push(state),
    invalidateSemanticState: () => {
      invalidationCount += 1;
    },
    createClient: (stateChanged) => {
      client = new ControlledClient(stateChanged);
      return client;
    },
  });
  assert.ok(client);
  return {
    lifecycle,
    client,
    states,
    invalidations: () => invalidationCount,
    setTarget: (replacement) => (target = replacement),
  };
}

test("connects, ignores duplicate connect, and disconnects through one client", async () => {
  const { lifecycle, client, states } = harness();
  assert.equal(await lifecycle.connect(), "connected");
  assert.equal(await lifecycle.connect(), "connected");
  assert.equal(await lifecycle.disconnect(), "disconnected");
  assert.deepEqual(client.connectCalls, [["oneagent-mcp", "/workspace"]]);
  assert.equal(client.disconnectCalls, 1);
  assert.deepEqual(states, [
    "disconnected",
    "connecting",
    "connected",
    "disconnecting",
    "disconnected",
  ]);
});

test("fails preflight without touching the process client and permits explicit retry", async () => {
  const invalid: ConnectionTarget = {
    ok: false,
    code: "invalid_configuration",
    message: "OneAgent Runtime configuration is invalid.",
  };
  const { lifecycle, client, states, setTarget } = harness(invalid);
  assert.equal(await lifecycle.connect(), "failed");
  assert.equal(client.connectCalls.length, 0);
  setTarget(supportedTarget());
  assert.equal(await lifecycle.connect(), "connected");
  assert.deepEqual(states, ["disconnected", "failed", "connecting", "connected"]);
});

test("maps client startup failure to failed and allows user-initiated reconnect", async () => {
  const { lifecycle, client } = harness();
  client.connectFailure = new RuntimeClientFailure("spawn_failed");
  assert.equal(await lifecycle.connect(), "failed");
  client.connectFailure = undefined;
  assert.equal(await lifecycle.connect(), "connected");
  assert.equal(client.connectCalls.length, 2);
});

test("configuration replacement disconnects without automatic reconnect", async () => {
  const { lifecycle, client, states } = harness();
  await lifecycle.connect();
  assert.equal(await lifecycle.configurationChanged(), "disconnected");
  assert.equal(client.connectCalls.length, 1);
  assert.equal(client.disconnectCalls, 1);
  assert.deepEqual(states.slice(-2), ["disconnecting", "disconnected"]);
  assert.equal(await lifecycle.configurationChanged(), "disconnected");
  assert.equal(client.disconnectCalls, 1);
});

test("forwards symbol search only while the owned client is connected", async () => {
  const { lifecycle, client } = harness();
  await assert.rejects(lifecycle.symbols({ query: "Sales" }), RuntimeClientFailure);
  await lifecycle.connect();
  assert.deepEqual(await lifecycle.symbols({ query: "Sales" }), {
    results: [],
    total: 0,
    truncated: false,
  });
  assert.deepEqual(client.symbolCalls, [{ query: "Sales" }]);
  await lifecycle.disconnect();
  await assert.rejects(lifecycle.symbols({ query: "Sales" }), RuntimeClientFailure);
});

test("forwards Context only while connected and invalidates semantic state before exit", async () => {
  const { lifecycle, client, invalidations } = harness();
  const input = { configurationId: "configuration-id", nodeId: "node-id" };
  await assert.rejects(lifecycle.context(input), RuntimeClientFailure);
  await lifecycle.connect();
  assert.deepEqual(await lifecycle.context(input), contextResult());
  assert.deepEqual(client.contextCalls, [input]);
  assert.equal(invalidations(), 1, "connecting invalidates stale semantic state");
  await lifecycle.disconnect();
  assert.equal(invalidations(), 3, "disconnecting and disconnected both invalidate idempotently");
  await assert.rejects(lifecycle.context(input), RuntimeClientFailure);
});

test("disconnect cancels an in-flight connect and shares one cleanup operation", async () => {
  const { lifecycle, client } = harness();
  client.holdConnect = true;
  const connection = lifecycle.connect();
  const first = lifecycle.disconnect();
  const second = lifecycle.disconnect();
  assert.equal(await first, "disconnected");
  assert.equal(await second, "disconnected");
  assert.equal(await connection, "disconnected");
  assert.equal(client.disconnectCalls, 1);
});

test("deactivation is repeatable and waits for process cleanup", async () => {
  const { lifecycle, client } = harness();
  await lifecycle.connect();
  await lifecycle.deactivate();
  await lifecycle.deactivate();
  assert.equal(lifecycle.state, "disconnected");
  assert.equal(client.disconnectCalls, 1);
  assert.equal(await lifecycle.connect(), "disconnected");
});

test("shutdown failure remains failed and bounded", async () => {
  const { lifecycle, client } = harness();
  await lifecycle.connect();
  client.disconnectFailure = new RuntimeClientFailure("shutdown_failed");
  assert.equal(await lifecycle.disconnect(), "failed");
  assert.equal(lifecycle.state, "failed");
  await assert.rejects(lifecycle.deactivate(), (error: unknown) => {
    assert.ok(error instanceof RuntimeClientFailure);
    assert.equal(error.code, "shutdown_failed");
    return true;
  });
});

function contextResult(): ContextResult {
  return {
    configurationId: "configuration-id",
    rendered: "seed",
    items: [{
      nodeId: "node-id",
      name: "Seed",
      kind: "procedure",
      depth: 0,
      seedId: "node-id",
      reason: "seed",
      relations: [],
      costBytes: 4,
    }],
    budgetBytes: 16_384,
    usedBytes: 4,
    remainingBytes: 16_380,
    candidateTruncated: false,
    candidateOmitted: 0,
    budgetTruncated: false,
    budgetOmitted: 0,
  };
}
