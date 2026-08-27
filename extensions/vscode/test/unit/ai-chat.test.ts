import assert from "node:assert/strict";
import test from "node:test";

import {
  AiChatController,
  CHAT_MESSAGES,
  MAX_CHAT_INPUT_BYTES,
  MAX_CHAT_OUTPUT_BYTES,
  MAX_CHAT_PROMPT_BYTES,
  buildContextMessage,
  escapeMarkdownText,
  linkCancellationSource,
  type CancellationSource,
  type CancellationToken,
  type ChatModel,
  type ChatRequest,
  type ChatResponseSink,
  type Disposable,
  type ModelResponse,
  type SafeMarkdown,
} from "../../src/ai-chat";
import type { ContextResult } from "../../src/mcp-client";

class TestCancellationSource implements CancellationSource {
  private cancelled = false;
  private disposed = false;
  private readonly listeners = new Set<() => void>();
  private readonly parentSubscription: Disposable | undefined;
  public cancelCalls = 0;

  public readonly token: CancellationToken;

  public constructor(parent?: CancellationToken) {
    const thisSource = this;
    this.token = {
      get isCancellationRequested() {
        return thisSource.cancelled;
      },
      onCancellationRequested: (listener) => {
        thisSource.listeners.add(listener);
        return { dispose: () => thisSource.listeners.delete(listener) };
      },
    };
    if (parent?.isCancellationRequested === true) {
      this.cancel();
    } else {
      this.parentSubscription = parent?.onCancellationRequested(() => this.cancel());
    }
  }

  public cancel(): void {
    this.cancelCalls += 1;
    if (this.cancelled) {
      return;
    }
    this.cancelled = true;
    for (const listener of this.listeners) {
      listener();
    }
  }

  public dispose(): void {
    this.disposed = true;
    this.parentSubscription?.dispose();
    this.listeners.clear();
  }

  public get isDisposed(): boolean {
    return this.disposed;
  }
}

class DeferredAlreadyCancelledToken implements CancellationToken {
  private readonly listeners = new Set<() => void>();

  public readonly isCancellationRequested = true;

  public onCancellationRequested(listener: () => void): Disposable {
    this.listeners.add(listener);
    return { dispose: () => this.listeners.delete(listener) };
  }

  public get listenerCount(): number {
    return this.listeners.size;
  }
}

class Sink implements ChatResponseSink {
  public readonly values: SafeMarkdown[] = [];

  public markdown(value: SafeMarkdown): void {
    this.values.push(value);
  }
}

class FakeModel implements ChatModel {
  public maxInputTokens = 100;
  public tokenCounts: number[] = [1, 1];
  public tokenError: unknown;
  public requestError: unknown;
  public response: ModelResponse = { text: streamOf("answer") };
  public readonly counted: Array<{ readonly message: unknown; readonly token: CancellationToken }> = [];
  public readonly requests: Array<{
    readonly messages: readonly unknown[];
    readonly options: undefined;
    readonly token: CancellationToken;
  }> = [];

  public countTokens(message: unknown, token: CancellationToken): PromiseLike<number> {
    this.counted.push({ message, token });
    if (this.tokenError !== undefined) {
      return Promise.reject(this.tokenError);
    }
    return Promise.resolve(this.tokenCounts[this.counted.length - 1] ?? 1);
  }

  public sendRequest(
    messages: readonly unknown[],
    options: undefined,
    token: CancellationToken,
  ): PromiseLike<ModelResponse> {
    this.requests.push({ messages, options, token });
    if (this.requestError !== undefined) {
      return Promise.reject(this.requestError);
    }
    return Promise.resolve(this.response);
  }
}

const neverCancelled = new TestCancellationSource().token;

function context(rendered = "semantic context"): ContextResult {
  return {
    configurationId: "configuration-id",
    rendered,
    items: [{
      nodeId: "node-id",
      name: "Seed",
      kind: "procedure",
      depth: 0,
      seedId: "node-id",
      reason: "seed",
      relations: [],
      costBytes: Buffer.byteLength(rendered, "utf8"),
    }],
    budgetBytes: 16_384,
    usedBytes: Buffer.byteLength(rendered, "utf8"),
    remainingBytes: 16_384 - Buffer.byteLength(rendered, "utf8"),
    candidateTruncated: false,
    candidateOmitted: 0,
    budgetTruncated: false,
    budgetOmitted: 0,
  };
}

function request(model: ChatModel, overrides: Partial<ChatRequest> = {}): ChatRequest {
  return {
    prompt: "What does this do?",
    command: undefined,
    references: [],
    toolReferences: [],
    model,
    ...overrides,
  };
}

function harness(
  selected: ContextResult | undefined = context(),
  cancellationFactory?: (parent: CancellationToken) => CancellationSource,
): {
  readonly controller: AiChatController;
  readonly sources: TestCancellationSource[];
  setConnected(value: boolean): void;
  setContext(value: ContextResult | undefined): void;
} {
  let connected = true;
  let selectedContext: ContextResult | undefined = selected;
  const sources: TestCancellationSource[] = [];
  const controller = new AiChatController({
    isConnected: () => connected,
    selectedContext: () => selectedContext,
    createUserMessage: (content) => ({ role: "user", content }),
    createCancellationSource: cancellationFactory ?? ((parent) => {
      const source = new TestCancellationSource(parent);
      sources.push(source);
      return source;
    }),
    isModelUnavailableError: (error) =>
      error instanceof Error && ["NoPermissions", "Blocked", "NotFound"].includes(error.name),
  });
  return {
    controller,
    sources,
    setConnected: (value) => (connected = value),
    setContext: (value) => {
      selectedContext = value;
    },
  };
}

async function* streamOf(...chunks: string[]): AsyncIterable<string> {
  for (const chunk of chunks) {
    yield chunk;
  }
}

test("sends exactly two user messages and streams only escaped untrusted Markdown", async () => {
  const selected = context("факт <tag>");
  const { controller, sources } = harness(selected);
  const model = new FakeModel();
  model.response = { text: streamOf("[run](command:workbench.action)\n", "<b>done</b>") };
  const sink = new Sink();
  const prompt = "  exact prompt  ";
  assert.deepEqual(await controller.handle(request(model, { prompt }), sink, neverCancelled), {
    category: "complete",
  });
  const expected = [
    { role: "user", content: buildContextMessage(selected.rendered) },
    { role: "user", content: prompt },
  ];
  assert.deepEqual(model.counted.map((entry) => entry.message), expected);
  assert.deepEqual(model.requests[0]?.messages, expected);
  assert.equal(model.requests[0]?.options, undefined);
  assert.equal(model.requests[0]?.token, sources[0]?.token);
  assert.deepEqual(sink.values.map((value) => value.value), [
    "\\[run\\]\\(command:workbench\\.action\\)\n",
    "&lt;b&gt;done&lt;/b&gt;",
  ]);
  assert.ok(sink.values.every((value) => value.isTrusted === false && value.supportHtml === false));
  assert.equal(sources[0]?.isDisposed, true);
  assert.ok(expected[0]?.content.includes(`${Buffer.byteLength(selected.rendered, "utf8")} UTF-8 BYTES`));
  assert.ok(expected[0]?.content.includes(selected.rendered));
  assert.equal(JSON.stringify(expected).includes("configuration-id"), false);
  assert.equal(JSON.stringify(expected).includes("node-id"), false);
});

test("applies unsupported, busy, prompt, connected, and Context precedence", async () => {
  const { controller, setConnected, setContext } = harness();
  const model = new FakeModel();
  const unsupported = new Sink();
  assert.deepEqual(
    await controller.handle(request(model, { command: "explain", references: [{}] }), unsupported, neverCancelled),
    { category: "unsupported_input" },
  );
  assert.equal(unsupported.values[0]?.value, CHAT_MESSAGES.unsupportedInput);

  let releaseRequest: ((value: ModelResponse) => void) | undefined;
  model.sendRequest = () => new Promise<ModelResponse>((resolve) => (releaseRequest = resolve));
  const pending = controller.handle(request(model), new Sink(), neverCancelled);
  await Promise.resolve();
  await Promise.resolve();
  const busy = new Sink();
  assert.deepEqual(
    await controller.handle(request(model, { prompt: "" }), busy, neverCancelled),
    { category: "busy" },
  );
  assert.equal(busy.values[0]?.value, CHAT_MESSAGES.busy);
  const unsupportedWhileBusy = new Sink();
  assert.deepEqual(
    await controller.handle(request(model, { toolReferences: [{}] }), unsupportedWhileBusy, neverCancelled),
    { category: "unsupported_input" },
  );
  assert.ok(releaseRequest);
  releaseRequest({ text: streamOf("done") });
  await pending;

  for (const prompt of ["", "x".repeat(MAX_CHAT_PROMPT_BYTES + 1)]) {
    const sink = new Sink();
    assert.deepEqual(await controller.handle(request(new FakeModel(), { prompt }), sink, neverCancelled), {
      category: "invalid_prompt",
    });
    assert.equal(sink.values[0]?.value, CHAT_MESSAGES.invalidPrompt);
  }
  setConnected(false);
  const disconnected = new Sink();
  assert.deepEqual(await controller.handle(request(new FakeModel()), disconnected, neverCancelled), {
    category: "context_required",
  });
  setConnected(true);
  setContext(undefined);
  const absent = new Sink();
  assert.deepEqual(await controller.handle(request(new FakeModel()), absent, neverCancelled), {
    category: "context_required",
  });
});

test("accepts prompt and assembled byte boundaries and rejects one over", async () => {
  const model = new FakeModel();
  const exactPrompt = "🙂".repeat(MAX_CHAT_PROMPT_BYTES / 4);
  assert.deepEqual(
    await harness().controller.handle(request(model, { prompt: exactPrompt }), new Sink(), neverCancelled),
    { category: "complete" },
  );
  assert.deepEqual(
    await harness().controller.handle(request(new FakeModel(), { prompt: " ".repeat(MAX_CHAT_PROMPT_BYTES) }), new Sink(), neverCancelled),
    { category: "complete" },
  );

  const prompt = "p";
  const exactRendered = renderedForAssembledBytes(MAX_CHAT_INPUT_BYTES, prompt);
  assert.equal(
    Buffer.byteLength(buildContextMessage(exactRendered), "utf8") + Buffer.byteLength(prompt, "utf8"),
    MAX_CHAT_INPUT_BYTES,
  );
  assert.deepEqual(
    await harness(context(exactRendered)).controller.handle(request(new FakeModel(), { prompt }), new Sink(), neverCancelled),
    { category: "complete" },
  );
  const sink = new Sink();
  assert.deepEqual(
    await harness(context(`${exactRendered}x`)).controller.handle(request(new FakeModel(), { prompt }), sink, neverCancelled),
    { category: "model_input_too_large" },
  );
  assert.equal(sink.values[0]?.value, CHAT_MESSAGES.inputTooLarge);
});

test("admits exact token capacity and fails closed for over, invalid, and unsafe counts", async () => {
  const exact = new FakeModel();
  exact.maxInputTokens = 10;
  exact.tokenCounts = [4, 6];
  assert.deepEqual(await harness().controller.handle(request(exact), new Sink(), neverCancelled), {
    category: "complete",
  });
  for (const [counts, maximum] of [
    [[4, 7], 10],
    [[-1, 1], 10],
    [[1.5, 1], 10],
    [[Number.MAX_SAFE_INTEGER, 1], Number.MAX_SAFE_INTEGER],
    [[1, 1], -1],
    [[1, 1], Number.MAX_SAFE_INTEGER + 1],
  ] as const) {
    const model = new FakeModel();
    model.tokenCounts = [...counts];
    model.maxInputTokens = maximum;
    const sink = new Sink();
    assert.deepEqual(await harness().controller.handle(request(model), sink, neverCancelled), {
      category: "model_input_too_large",
    });
    assert.equal(model.requests.length, 0);
    assert.equal(sink.values[0]?.value, CHAT_MESSAGES.inputTooLarge);
  }
});

test("redacts unavailable and unknown model failures at count and request boundaries", async () => {
  for (const stage of ["count", "request"] as const) {
    for (const name of ["NoPermissions", "Blocked", "NotFound", "Unknown"]) {
      const model = new FakeModel();
      const error = new Error("secret model-id /private/context");
      error.name = name;
      if (stage === "count") {
        model.tokenError = error;
      } else {
        model.requestError = error;
      }
      const sink = new Sink();
      const category = ["NoPermissions", "Blocked", "NotFound"].includes(name)
        ? "model_unavailable"
        : "model_request_failed";
      assert.deepEqual(await harness().controller.handle(request(model), sink, neverCancelled), { category });
      assert.equal(
        sink.values[0]?.value,
        category === "model_unavailable" ? CHAT_MESSAGES.modelUnavailable : CHAT_MESSAGES.requestFailed,
      );
      assert.equal(JSON.stringify(sink.values).includes("secret"), false);
      assert.equal(JSON.stringify(sink.values).includes("/private/"), false);
    }
  }
});

test("admits the exact output bound and rejects a whole chunk that crosses it", async () => {
  const exactModel = new FakeModel();
  exactModel.response = { text: streamOf("x".repeat(MAX_CHAT_OUTPUT_BYTES)) };
  const exactSink = new Sink();
  assert.deepEqual(
    await harness().controller.handle(request(exactModel), exactSink, neverCancelled),
    { category: "complete" },
  );
  assert.equal(exactSink.values[0]?.value.length, MAX_CHAT_OUTPUT_BYTES);

  const overModel = new FakeModel();
  overModel.response = { text: streamOf("partial", "x".repeat(MAX_CHAT_OUTPUT_BYTES)) };
  const overSink = new Sink();
  const overHarness = harness();
  assert.deepEqual(
    await overHarness.controller.handle(request(overModel), overSink, neverCancelled),
    { category: "model_response_too_large" },
  );
  assert.deepEqual(overSink.values.map((value) => value.value), [
    "partial",
    CHAT_MESSAGES.responseTooLarge,
  ]);
  assert.equal(overHarness.sources[0]?.cancelCalls, 1);
});

test("keeps partial text, redacts stream errors, and rejects non-text values", async () => {
  const failingModel = new FakeModel();
  failingModel.response = {
    text: (async function* () {
      yield "partial";
      throw new Error("raw output and provider secret");
    })(),
  };
  const sink = new Sink();
  assert.deepEqual(await harness().controller.handle(request(failingModel), sink, neverCancelled), {
    category: "model_response_failed",
  });
  assert.deepEqual(sink.values.map((value) => value.value), ["partial", CHAT_MESSAGES.responseFailed]);

  const nonText = new FakeModel();
  nonText.response = { text: streamOf({ toolCall: "forbidden" } as unknown as string) };
  assert.deepEqual(await harness().controller.handle(request(nonText), new Sink(), neverCancelled), {
    category: "model_response_failed",
  });
});

test("cancellation is silent before model use and during token, request, and stream boundaries", async () => {
  const alreadyCancelled = new TestCancellationSource();
  alreadyCancelled.cancel();
  const beforeModel = new FakeModel();
  const beforeSink = new Sink();
  assert.deepEqual(
    await harness().controller.handle(request(beforeModel), beforeSink, alreadyCancelled.token),
    { category: "cancelled" },
  );
  assert.equal(beforeModel.counted.length, 0);
  assert.deepEqual(beforeSink.values, []);

  const releaseCounts: Array<(value: number) => void> = [];
  const countingModel = new FakeModel();
  countingModel.countTokens = () => new Promise<number>((resolve) => releaseCounts.push(resolve));
  const countingHarness = harness();
  const countingSink = new Sink();
  const counting = countingHarness.controller.handle(request(countingModel), countingSink, neverCancelled);
  countingHarness.controller.invalidate();
  assert.equal(releaseCounts.length, 2);
  for (const release of releaseCounts) {
    release(1);
  }
  assert.deepEqual(await counting, { category: "cancelled" });
  assert.deepEqual(countingSink.values, []);
  assert.equal(countingModel.requests.length, 0);

  let releaseRequest: ((value: ModelResponse) => void) | undefined;
  const requestingModel = new FakeModel();
  requestingModel.sendRequest = () => new Promise<ModelResponse>((resolve) => (releaseRequest = resolve));
  const requestingHarness = harness();
  const requesting = requestingHarness.controller.handle(request(requestingModel), new Sink(), neverCancelled);
  await new Promise<void>((resolve) => setImmediate(resolve));
  requestingHarness.controller.invalidate();
  assert.ok(releaseRequest);
  releaseRequest({ text: streamOf("late") });
  assert.deepEqual(await requesting, { category: "cancelled" });

  let releaseStream: (() => void) | undefined;
  const streamingModel = new FakeModel();
  streamingModel.response = {
    text: (async function* () {
      yield "first";
      await new Promise<void>((resolve) => (releaseStream = resolve));
      yield "late";
    })(),
  };
  const streamingHarness = harness();
  const streamingSink = new Sink();
  const streaming = streamingHarness.controller.handle(request(streamingModel), streamingSink, neverCancelled);
  await new Promise<void>((resolve) => setImmediate(resolve));
  streamingHarness.controller.invalidate();
  assert.ok(releaseStream);
  releaseStream();
  assert.deepEqual(await streaming, { category: "cancelled" });
  assert.deepEqual(streamingSink.values.map((value) => value.value), ["first"]);
});

test("production cancellation adapter synchronously inherits an already-cancelled parent", async () => {
  const parent = new DeferredAlreadyCancelledToken();
  const source = new TestCancellationSource();
  const { controller } = harness(
    context(),
    (token) => linkCancellationSource(token, source),
  );
  const model = new FakeModel();
  const sink = new Sink();

  assert.deepEqual(await controller.handle(request(model), sink, parent), {
    category: "cancelled",
  });
  assert.equal(source.cancelCalls, 1);
  assert.equal(source.isDisposed, true);
  assert.equal(parent.listenerCount, 0);
  assert.equal(model.counted.length, 0);
  assert.equal(model.requests.length, 0);
  assert.deepEqual(sink.values, []);
});

test("disposal cancels active work and repeated requests do not retain the model", async () => {
  const releases: Array<(value: number) => void> = [];
  const firstModel = new FakeModel();
  firstModel.countTokens = () => new Promise<number>((resolve) => releases.push(resolve));
  const owner = harness();
  const pending = owner.controller.handle(request(firstModel), new Sink(), neverCancelled);
  owner.controller.dispose();
  owner.controller.dispose();
  assert.equal(releases.length, 2);
  for (const release of releases) {
    release(1);
  }
  assert.deepEqual(await pending, { category: "cancelled" });
  assert.equal(owner.sources[0]?.isDisposed, true);
  assert.deepEqual(
    await owner.controller.handle(request(new FakeModel()), new Sink(), neverCancelled),
    { category: "disposed" },
  );

  const repeatOwner = harness();
  const first = new FakeModel();
  const second = new FakeModel();
  assert.deepEqual(await repeatOwner.controller.handle(request(first), new Sink(), neverCancelled), {
    category: "complete",
  });
  assert.deepEqual(await repeatOwner.controller.handle(request(second), new Sink(), neverCancelled), {
    category: "complete",
  });
  assert.equal(first.requests.length, 1);
  assert.equal(second.requests.length, 1);
});

test("Markdown escaping preserves newlines and neutralizes links, images, HTML, and punctuation", () => {
  assert.equal(
    escapeMarkdownText("\\`*_{}[]()#+-.!| <tag> &\n![x](command:run)"),
    "\\\\\\`\\*\\_\\{\\}\\[\\]\\(\\)\\#\\+\\-\\.\\!\\| &lt;tag&gt; &amp;\n\\!\\[x\\]\\(command:run\\)",
  );
});

function renderedForAssembledBytes(targetBytes: number, prompt: string): string {
  for (let length = targetBytes; length >= 0; length -= 1) {
    const rendered = "x".repeat(length);
    const total = Buffer.byteLength(buildContextMessage(rendered), "utf8") + Buffer.byteLength(prompt, "utf8");
    if (total === targetBytes) {
      return rendered;
    }
  }
  assert.fail("expected an exact assembled-byte fixture");
}
