import { Buffer } from "node:buffer";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import type { Readable, Writable } from "node:stream";

import type { ConnectionState } from "./status";

export const MCP_PROTOCOL_VERSION = "2026-07-28";
export const MAX_FRAME_BYTES = 1_048_576;
export const MAX_JSON_DEPTH = 128;
export const MAX_STDERR_BYTES = 4_096;
export const REQUEST_TIMEOUT_MS = 5_000;
export const SHUTDOWN_TIMEOUT_MS = 2_000;

const TOOL_NAMES = [
  "oneagent.context",
  "oneagent.diagnostics",
  "oneagent.graph",
  "oneagent.impact",
  "oneagent.query",
  "oneagent.symbols",
  "oneagent.validation",
] as const;

export const MAX_SYMBOL_QUERY_BYTES = 256;
export const MAX_SYMBOL_PATH_BYTES = 4_096;

export type SymbolKind = "module" | "procedure" | "function" | "query";

export interface SymbolSearchRequest {
  readonly query: string;
  readonly configurationId?: string;
  readonly kinds?: readonly SymbolKind[];
  readonly limit?: number;
}

export interface SourcePosition {
  readonly line: number;
  readonly column: number;
}

export interface SourceSpan {
  readonly start: SourcePosition;
  readonly end: SourcePosition;
}

export interface SymbolSearchResultItem {
  readonly configurationId: string;
  readonly configurationName: string;
  readonly nodeId: string;
  readonly name: string;
  readonly kind: SymbolKind;
  readonly location: {
    readonly path: string;
    readonly span?: SourceSpan;
  };
}

export interface SymbolSearchResult {
  readonly results: readonly SymbolSearchResultItem[];
  readonly total: number;
  readonly truncated: boolean;
}

export type RuntimeFailureCode =
  | "invalid_configuration"
  | "unsupported_workspace"
  | "spawn_failed"
  | "startup_timeout"
  | "protocol_failure"
  | "incompatible_server"
  | "stderr_overflow"
  | "process_exited"
  | "tool_failure"
  | "shutdown_failed";

const FAILURE_MESSAGES: Readonly<Record<RuntimeFailureCode, string>> = {
  invalid_configuration: "OneAgent Runtime configuration is invalid.",
  unsupported_workspace: "OneAgent requires one trusted local workspace.",
  spawn_failed: "OneAgent Runtime could not be started.",
  startup_timeout: "OneAgent Runtime did not become ready in time.",
  protocol_failure: "OneAgent Runtime returned an invalid protocol response.",
  incompatible_server: "OneAgent Runtime is not compatible with this extension.",
  stderr_overflow: "OneAgent Runtime diagnostics exceeded the safety limit.",
  process_exited: "OneAgent Runtime stopped unexpectedly.",
  tool_failure: "OneAgent symbol search failed.",
  shutdown_failed: "OneAgent Runtime could not be stopped.",
};

export class RuntimeClientFailure extends Error {
  public constructor(public readonly code: RuntimeFailureCode) {
    super(FAILURE_MESSAGES[code]);
    this.name = "RuntimeClientFailure";
    this.stack = `${this.name}: ${this.message}`;
  }
}

interface RuntimeProcess {
  readonly stdin: Pick<Writable, "write" | "end">;
  readonly stdout: Pick<Readable, "on" | "removeListener">;
  readonly stderr: Pick<Readable, "on" | "removeListener">;
  on(event: "error", listener: (error: Error) => void): this;
  on(event: "exit", listener: (code: number | null, signal: NodeJS.Signals | null) => void): this;
  on(event: "close", listener: (code: number | null, signal: NodeJS.Signals | null) => void): this;
  removeListener(event: "error", listener: (error: Error) => void): this;
  removeListener(
    event: "exit",
    listener: (code: number | null, signal: NodeJS.Signals | null) => void,
  ): this;
  removeListener(
    event: "close",
    listener: (code: number | null, signal: NodeJS.Signals | null) => void,
  ): this;
  kill(): boolean;
}

export type RuntimeProcessFactory = (executable: string, cwd: string) => RuntimeProcess;

export interface RuntimeTimer {
  cancel(): void;
}

export interface RuntimeScheduler {
  schedule(delayMilliseconds: number, callback: () => void): RuntimeTimer;
}

export interface RuntimeClientOptions {
  readonly processFactory?: RuntimeProcessFactory;
  readonly scheduler?: RuntimeScheduler;
  readonly onStateChange?: (state: ConnectionState, failure?: RuntimeClientFailure) => void;
}

interface PendingRequest {
  readonly id: number;
  readonly resolve: (result: unknown) => void;
  readonly reject: (failure: RuntimeClientFailure) => void;
  readonly timer: RuntimeTimer;
}

const systemScheduler: RuntimeScheduler = {
  schedule(delayMilliseconds, callback) {
    const handle = setTimeout(callback, delayMilliseconds);
    return { cancel: () => clearTimeout(handle) };
  },
};

export const spawnRuntimeProcess: RuntimeProcessFactory = (executable, cwd) =>
  spawn(executable, [], {
    cwd,
    env: process.env,
    shell: false,
    stdio: "pipe",
    windowsHide: true,
  }) as ChildProcessWithoutNullStreams;

export class RuntimeClient {
  private readonly processFactory: RuntimeProcessFactory;
  private readonly scheduler: RuntimeScheduler;
  private readonly onStateChange?: RuntimeClientOptions["onStateChange"];
  private currentState: ConnectionState = "disconnected";
  private process: RuntimeProcess | undefined;
  private pending: PendingRequest | undefined;
  private nextRequestId = 1;
  private stdoutBuffer = Buffer.alloc(0);
  private stderrBytes = 0;
  private exitObserved = false;
  private exitWaiters = new Set<(exited: boolean) => void>();
  private stopping = false;
  private stopPromise: Promise<boolean> | undefined;
  private symbolRequestTail: Promise<void> = Promise.resolve();

  public constructor(options: RuntimeClientOptions = {}) {
    this.processFactory = options.processFactory ?? spawnRuntimeProcess;
    this.scheduler = options.scheduler ?? systemScheduler;
    this.onStateChange = options.onStateChange;
  }

  public get state(): ConnectionState {
    return this.currentState;
  }

  public async connect(executable: string, cwd: string): Promise<ConnectionState> {
    if (this.currentState !== "disconnected" && this.currentState !== "failed") {
      return this.currentState;
    }

    if (this.process !== undefined) {
      const stopped = await this.stopOwnedProcess();
      if (!stopped) {
        const failure = new RuntimeClientFailure("shutdown_failed");
        this.setState("failed", failure);
        throw failure;
      }
    }

    this.resetConnectionBuffers();
    this.setState("connecting");

    try {
      this.process = this.processFactory(executable, cwd);
      this.attachProcess(this.process);
    } catch {
      const failure = new RuntimeClientFailure("spawn_failed");
      this.setState("failed", failure);
      throw failure;
    }

    try {
      const discovery = await this.request("server/discover");
      if (!isCompatibleDiscovery(discovery)) {
        throw new RuntimeClientFailure("incompatible_server");
      }

      const tools = await this.request("tools/list");
      if (!isCompatibleToolList(tools)) {
        throw new RuntimeClientFailure("incompatible_server");
      }

      this.setState("connected");
      return this.currentState;
    } catch (error) {
      const failure = asRuntimeFailure(error);
      const stopped = await this.stopOwnedProcess();
      if (!stopped) {
        const shutdownFailure = new RuntimeClientFailure("shutdown_failed");
        this.setState("failed", shutdownFailure);
        throw shutdownFailure;
      }
      if (this.shouldReportConnectionFailure()) {
        this.setState("failed", failure);
      }
      throw failure;
    }
  }

  public async disconnect(): Promise<ConnectionState> {
    if (this.currentState === "disconnected" || this.currentState === "disconnecting") {
      return this.currentState;
    }

    this.setState("disconnecting");
    this.rejectPending(new RuntimeClientFailure("process_exited"));
    const stopped = await this.stopOwnedProcess();
    if (!stopped) {
      const failure = new RuntimeClientFailure("shutdown_failed");
      this.setState("failed", failure);
      throw failure;
    }
    this.setState("disconnected");
    return this.currentState;
  }

  public symbols(input: SymbolSearchRequest): Promise<SymbolSearchResult> {
    if (this.currentState !== "connected" || !isSymbolSearchRequest(input)) {
      return Promise.reject(new RuntimeClientFailure("protocol_failure"));
    }
    const operation = this.symbolRequestTail.then(() => this.executeSymbols(input));
    this.symbolRequestTail = operation.then(
      () => undefined,
      () => undefined,
    );
    return operation;
  }

  private async executeSymbols(input: SymbolSearchRequest): Promise<SymbolSearchResult> {
    if (this.currentState !== "connected") {
      throw new RuntimeClientFailure("protocol_failure");
    }
    const argumentsValue: Record<string, unknown> = { query: input.query };
    if (input.configurationId !== undefined) {
      argumentsValue.configurationId = input.configurationId;
    }
    if (input.kinds !== undefined) {
      argumentsValue.kinds = [...input.kinds];
    }
    if (input.limit !== undefined) {
      argumentsValue.limit = input.limit;
    }

    let result: unknown;
    try {
      result = await this.request("tools/call", {
        name: "oneagent.symbols",
        arguments: argumentsValue,
      });
    } catch (error) {
      const failure = asRuntimeFailure(error);
      if (this.currentState === "connected") {
        this.abort(failure);
      }
      throw failure;
    }
    const decoded = decodeSymbolToolResult(result, input.limit ?? 50);
    if (decoded === "tool_failure") {
      throw new RuntimeClientFailure("tool_failure");
    }
    if (decoded === undefined) {
      const failure = new RuntimeClientFailure("protocol_failure");
      this.abort(failure);
      throw failure;
    }
    return decoded;
  }

  private async request(
    method: "server/discover" | "tools/list" | "tools/call",
    fields: Readonly<Record<string, unknown>> = {},
  ): Promise<unknown> {
    const child = this.process;
    if (child === undefined || this.pending !== undefined) {
      throw new RuntimeClientFailure("protocol_failure");
    }
    if (!Number.isSafeInteger(this.nextRequestId) || this.nextRequestId > Number.MAX_SAFE_INTEGER) {
      throw new RuntimeClientFailure("protocol_failure");
    }

    const id = this.nextRequestId;
    this.nextRequestId += 1;
    const frame = JSON.stringify({
      jsonrpc: "2.0",
      id,
      method,
      params: {
        ...fields,
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

    return new Promise<unknown>((resolve, reject) => {
      const timer = this.scheduler.schedule(REQUEST_TIMEOUT_MS, () => {
        if (this.pending?.id !== id) {
          return;
        }
        const failure = new RuntimeClientFailure("startup_timeout");
        this.rejectPending(failure);
      });
      this.pending = { id, resolve, reject, timer };
      child.stdin.write(`${frame}\n`, (error?: Error | null) => {
        if (error !== undefined && error !== null && this.pending?.id === id) {
          this.rejectPending(new RuntimeClientFailure("process_exited"));
        }
      });
    });
  }

  private attachProcess(child: RuntimeProcess): void {
    child.stdout.on("data", this.handleStdout);
    child.stderr.on("data", this.handleStderr);
    child.on("error", this.handleProcessError);
    child.on("exit", this.handleProcessExit);
    child.on("close", this.handleProcessClose);
  }

  private detachProcess(child: RuntimeProcess): void {
    child.stdout.removeListener("data", this.handleStdout);
    child.stderr.removeListener("data", this.handleStderr);
    child.removeListener("error", this.handleProcessError);
    child.removeListener("exit", this.handleProcessExit);
    child.removeListener("close", this.handleProcessClose);
  }

  private readonly handleStdout = (chunk: Buffer | string): void => {
    if (this.currentState !== "connecting" && this.currentState !== "connected") {
      return;
    }
    const bytes = typeof chunk === "string" ? Buffer.from(chunk, "utf8") : chunk;
    let offset = 0;
    while (offset < bytes.length) {
      const newline = bytes.indexOf(0x0a, offset);
      const segmentEnd = newline < 0 ? bytes.length : newline;
      const segment = bytes.subarray(offset, segmentEnd);
      const combinedLength = this.stdoutBuffer.length + segment.length;
      if (combinedLength > MAX_FRAME_BYTES + 1) {
        this.abort(new RuntimeClientFailure("protocol_failure"));
        return;
      }
      this.stdoutBuffer = Buffer.concat([this.stdoutBuffer, segment], combinedLength);
      if (newline < 0) {
        const allowedTrailingCr =
          this.stdoutBuffer.length === MAX_FRAME_BYTES + 1 &&
          this.stdoutBuffer[MAX_FRAME_BYTES] === 0x0d;
        if (this.stdoutBuffer.length > MAX_FRAME_BYTES && !allowedTrailingCr) {
          this.abort(new RuntimeClientFailure("protocol_failure"));
        }
        return;
      }

      let frame = this.stdoutBuffer;
      this.stdoutBuffer = Buffer.alloc(0);
      if (frame.at(-1) === 0x0d) {
        frame = frame.subarray(0, -1);
      }
      if (frame.length > MAX_FRAME_BYTES) {
        this.abort(new RuntimeClientFailure("protocol_failure"));
        return;
      }
      this.acceptFrame(frame);
      if (this.hasFailed()) {
        return;
      }
      offset = newline + 1;
    }
  };

  private readonly handleStderr = (chunk: Buffer | string): void => {
    if (this.currentState !== "connecting" && this.currentState !== "connected") {
      return;
    }
    const bytes = typeof chunk === "string" ? Buffer.byteLength(chunk, "utf8") : chunk.length;
    this.stderrBytes += bytes;
    if (this.stderrBytes > MAX_STDERR_BYTES) {
      this.abort(new RuntimeClientFailure("stderr_overflow"));
    }
  };

  private readonly handleProcessError = (): void => {
    if (
      this.stopping ||
      (this.currentState !== "connecting" && this.currentState !== "connected")
    ) {
      return;
    }
    const code: RuntimeFailureCode = this.currentState === "connecting" ? "spawn_failed" : "process_exited";
    this.abort(new RuntimeClientFailure(code));
  };

  private readonly handleProcessExit = (): void => {
    this.observeProcessTermination();

    if (!this.stopping && (this.currentState === "connecting" || this.currentState === "connected")) {
      const code: RuntimeFailureCode =
        this.stdoutBuffer.length === 0 ? "process_exited" : "protocol_failure";
      this.abort(new RuntimeClientFailure(code));
    } else if (!this.stopping && this.currentState === "failed") {
      this.releaseTerminatedProcess();
    }
  };

  private readonly handleProcessClose = (): void => {
    const wasObserved = this.exitObserved;
    this.observeProcessTermination();
    if (
      !wasObserved &&
      !this.stopping &&
      (this.currentState === "connecting" || this.currentState === "connected")
    ) {
      const code: RuntimeFailureCode =
        this.stdoutBuffer.length === 0 ? "process_exited" : "protocol_failure";
      this.abort(new RuntimeClientFailure(code));
    } else if (!this.stopping && this.currentState === "failed") {
      this.releaseTerminatedProcess();
    }
  };

  private observeProcessTermination(): void {
    this.exitObserved = true;
    for (const waiter of this.exitWaiters) {
      waiter(true);
    }
    this.exitWaiters.clear();
  }

  private acceptFrame(frame: Buffer): void {
    const pending = this.pending;
    if (pending === undefined) {
      this.abort(new RuntimeClientFailure("protocol_failure"));
      return;
    }

    let value: unknown;
    try {
      const text = new TextDecoder("utf-8", { fatal: true }).decode(frame);
      value = new UniqueJsonParser(text).parse();
    } catch {
      this.abort(new RuntimeClientFailure("protocol_failure"));
      return;
    }

    if (!isRecord(value) || value.jsonrpc !== "2.0" || value.id !== pending.id) {
      this.abort(new RuntimeClientFailure("protocol_failure"));
      return;
    }
    const hasResult = Object.hasOwn(value, "result");
    const hasError = Object.hasOwn(value, "error");
    if (hasResult === hasError) {
      this.abort(new RuntimeClientFailure("protocol_failure"));
      return;
    }
    if (hasError) {
      this.abort(new RuntimeClientFailure("protocol_failure"));
      return;
    }

    this.pending = undefined;
    pending.timer.cancel();
    pending.resolve(value.result);
  }

  private abort(failure: RuntimeClientFailure): void {
    this.rejectPending(failure);
    this.setState("failed", failure);
    void this.stopOwnedProcess().then((stopped) => {
      if (!stopped) {
        this.setState("failed", new RuntimeClientFailure("shutdown_failed"));
      }
    });
  }

  private rejectPending(failure: RuntimeClientFailure): void {
    const pending = this.pending;
    if (pending === undefined) {
      return;
    }
    this.pending = undefined;
    pending.timer.cancel();
    pending.reject(failure);
  }

  private async stopOwnedProcess(): Promise<boolean> {
    if (this.stopPromise !== undefined) {
      return this.stopPromise;
    }
    this.stopPromise = this.performStopOwnedProcess();
    try {
      return await this.stopPromise;
    } finally {
      this.stopPromise = undefined;
    }
  }

  private async performStopOwnedProcess(): Promise<boolean> {
    const child = this.process;
    if (child === undefined) {
      return true;
    }

    this.stopping = true;
    child.stdin.end();
    let exited = await this.waitForExit(SHUTDOWN_TIMEOUT_MS);
    if (!exited) {
      child.kill();
      exited = await this.waitForExit(SHUTDOWN_TIMEOUT_MS);
    }
    if (exited) {
      this.releaseTerminatedProcess();
    }
    this.stopping = false;
    return exited;
  }

  private waitForExit(timeoutMilliseconds: number): Promise<boolean> {
    if (this.exitObserved) {
      return Promise.resolve(true);
    }
    return new Promise<boolean>((resolve) => {
      let timer: RuntimeTimer | undefined;
      const finish = (exited: boolean): void => {
        timer?.cancel();
        this.exitWaiters.delete(finish);
        resolve(exited);
      };
      this.exitWaiters.add(finish);
      timer = this.scheduler.schedule(timeoutMilliseconds, () => finish(false));
    });
  }

  private resetConnectionBuffers(): void {
    this.stdoutBuffer = Buffer.alloc(0);
    this.stderrBytes = 0;
    this.exitObserved = false;
    this.exitWaiters.clear();
  }

  private releaseTerminatedProcess(): void {
    const child = this.process;
    if (child === undefined) {
      return;
    }
    this.detachProcess(child);
    this.process = undefined;
    this.resetConnectionBuffers();
  }

  private setState(state: ConnectionState, failure?: RuntimeClientFailure): void {
    this.currentState = state;
    if (failure === undefined) {
      this.onStateChange?.(state);
    } else {
      this.onStateChange?.(state, failure);
    }
  }

  private hasFailed(): boolean {
    return this.currentState === "failed";
  }

  private shouldReportConnectionFailure(): boolean {
    return this.currentState === "connecting";
  }
}

function asRuntimeFailure(error: unknown): RuntimeClientFailure {
  return error instanceof RuntimeClientFailure
    ? error
    : new RuntimeClientFailure("protocol_failure");
}

function isCompatibleDiscovery(value: unknown): boolean {
  if (!isRecord(value) || value.resultType !== "complete") {
    return false;
  }
  const versions = value.supportedVersions;
  const capabilities = value.capabilities;
  const metadata = value._meta;
  return (
    Array.isArray(versions) &&
    versions.length === 1 &&
    versions[0] === MCP_PROTOCOL_VERSION &&
    isRecord(capabilities) &&
    isRecord(capabilities.tools) &&
    Object.keys(capabilities.tools).length === 0 &&
    isRecord(metadata) &&
    isRecord(metadata["io.modelcontextprotocol/serverInfo"]) &&
    metadata["io.modelcontextprotocol/serverInfo"].name === "oneagent" &&
    value.ttlMs === 0 &&
    value.cacheScope === "public"
  );
}

function isCompatibleToolList(value: unknown): boolean {
  if (
    !isRecord(value) ||
    value.resultType !== "complete" ||
    value.ttlMs !== 0 ||
    value.cacheScope !== "public" ||
    !Array.isArray(value.tools) ||
    value.tools.length !== TOOL_NAMES.length
  ) {
    return false;
  }
  return value.tools.every(
    (tool, index) => isRecord(tool) && tool.name === TOOL_NAMES[index],
  );
}

function isSymbolSearchRequest(value: SymbolSearchRequest): boolean {
  if (
    typeof value.query !== "string" ||
    Buffer.byteLength(value.query, "utf8") === 0 ||
    Buffer.byteLength(value.query, "utf8") > MAX_SYMBOL_QUERY_BYTES ||
    (value.configurationId !== undefined && !isNonEmptyBoundedText(value.configurationId)) ||
    (value.limit !== undefined &&
      (!Number.isSafeInteger(value.limit) || value.limit < 1 || value.limit > 100))
  ) {
    return false;
  }
  if (value.kinds === undefined) {
    return true;
  }
  if (!Array.isArray(value.kinds) || value.kinds.length === 0) {
    return false;
  }
  const kinds = new Set<SymbolKind>();
  for (const kind of value.kinds) {
    if (!isSymbolKind(kind) || kinds.has(kind)) {
      return false;
    }
    kinds.add(kind);
  }
  return true;
}

function decodeSymbolToolResult(
  value: unknown,
  limit: number,
): SymbolSearchResult | "tool_failure" | undefined {
  if (!isRecord(value) || value.resultType !== "complete") {
    return undefined;
  }
  const expectedKeys = value.isError === true
    ? ["content", "isError", "resultType", "structuredContent"]
    : ["content", "resultType", "structuredContent"];
  if (!hasExactKeys(value, expectedKeys) || !Array.isArray(value.content) || value.content.length !== 1) {
    return undefined;
  }
  const block = value.content[0];
  if (
    !isRecord(block) ||
    !hasExactKeys(block, ["text", "type"]) ||
    block.type !== "text" ||
    typeof block.text !== "string"
  ) {
    return undefined;
  }
  let textValue: unknown;
  try {
    textValue = new UniqueJsonParser(block.text).parse();
  } catch {
    return undefined;
  }
  if (!jsonEqual(textValue, value.structuredContent)) {
    return undefined;
  }
  if (value.isError === true) {
    return isToolError(value.structuredContent) ? "tool_failure" : undefined;
  }
  return decodeSymbolSearchResult(value.structuredContent, limit);
}

function decodeSymbolSearchResult(value: unknown, limit: number): SymbolSearchResult | undefined {
  if (
    !isRecord(value) ||
    !hasExactKeys(value, ["results", "total", "truncated"]) ||
    !Array.isArray(value.results) ||
    value.results.length > limit ||
    !Number.isSafeInteger(value.total) ||
    typeof value.total !== "number" ||
    value.total < value.results.length ||
    typeof value.truncated !== "boolean" ||
    value.truncated !== (value.total > value.results.length)
  ) {
    return undefined;
  }
  const results: SymbolSearchResultItem[] = [];
  for (const item of value.results) {
    const decoded = decodeSymbolSearchItem(item);
    if (decoded === undefined) {
      return undefined;
    }
    results.push(decoded);
  }
  return { results, total: value.total, truncated: value.truncated };
}

function decodeSymbolSearchItem(value: unknown): SymbolSearchResultItem | undefined {
  if (
    !isRecord(value) ||
    !hasExactKeys(value, [
      "configurationId",
      "configurationName",
      "kind",
      "location",
      "name",
      "nodeId",
    ]) ||
    !isNonEmptyBoundedText(value.configurationId) ||
    !isNonEmptyBoundedText(value.configurationName) ||
    !isNonEmptyBoundedText(value.nodeId) ||
    !isNonEmptyBoundedText(value.name) ||
    !isSymbolKind(value.kind) ||
    !isRecord(value.location) ||
    !hasExactKeys(
      value.location,
      value.location.span === undefined ? ["path"] : ["path", "span"],
    ) ||
    !isSafeRelativeSymbolPath(value.location.path)
  ) {
    return undefined;
  }
  const rawSpan = value.location.span;
  const span = rawSpan === undefined ? undefined : decodeSourceSpan(rawSpan);
  if (rawSpan !== undefined && span === undefined) {
    return undefined;
  }
  const location = span === undefined
    ? { path: value.location.path }
    : { path: value.location.path, span };
  return {
    configurationId: value.configurationId,
    configurationName: value.configurationName,
    nodeId: value.nodeId,
    name: value.name,
    kind: value.kind,
    location,
  };
}

function isToolError(value: unknown): boolean {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["code", "message"]) &&
    isNonEmptyBoundedText(value.code) &&
    isNonEmptyBoundedText(value.message)
  );
}

function decodeSourceSpan(value: unknown): SourceSpan | undefined {
  if (
    !isRecord(value) ||
    !hasExactKeys(value, ["end", "start"]) ||
    !isSourcePosition(value.start) ||
    !isSourcePosition(value.end)
  ) {
    return undefined;
  }
  if (!(
    value.end.line > value.start.line ||
    (value.end.line === value.start.line && value.end.column >= value.start.column)
  )) {
    return undefined;
  }
  return {
    start: { line: value.start.line, column: value.start.column },
    end: { line: value.end.line, column: value.end.column },
  };
}

function isSourcePosition(value: unknown): value is SourcePosition {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["column", "line"]) &&
    isUnsigned32(value.line) &&
    isUnsigned32(value.column) &&
    value.line > 0 &&
    value.column > 0
  );
}

function isUnsigned32(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff;
}

function isSymbolKind(value: unknown): value is SymbolKind {
  return value === "module" || value === "procedure" || value === "function" || value === "query";
}

export function isSafeRelativeSymbolPath(value: unknown): value is string {
  if (
    typeof value !== "string" ||
    Buffer.byteLength(value, "utf8") === 0 ||
    Buffer.byteLength(value, "utf8") > MAX_SYMBOL_PATH_BYTES ||
    value.includes("\\") ||
    value.includes("\0") ||
    value.startsWith("/") ||
    /^[A-Za-z]:/u.test(value)
  ) {
    return false;
  }
  return value.split("/").every((component) => component !== "" && component !== "." && component !== "..");
}

function isNonEmptyBoundedText(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.trim().length > 0 &&
    Buffer.byteLength(value, "utf8") <= 65_536
  );
}

function hasExactKeys(value: Record<string, unknown>, expected: readonly string[]): boolean {
  const keys = Object.keys(value).sort();
  return keys.length === expected.length && keys.every((key, index) => key === [...expected].sort()[index]);
}

function jsonEqual(left: unknown, right: unknown): boolean {
  if (left === right) {
    return true;
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    return (
      Array.isArray(left) &&
      Array.isArray(right) &&
      left.length === right.length &&
      left.every((item, index) => jsonEqual(item, right[index]))
    );
  }
  if (!isRecord(left) || !isRecord(right)) {
    return false;
  }
  const leftKeys = Object.keys(left).sort();
  const rightKeys = Object.keys(right).sort();
  return (
    leftKeys.length === rightKeys.length &&
    leftKeys.every((key, index) => key === rightKeys[index] && jsonEqual(left[key], right[key]))
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

class UniqueJsonParser {
  private index = 0;

  public constructor(private readonly source: string) {}

  public parse(): unknown {
    const value = this.parseValue(0);
    this.skipWhitespace();
    if (this.index !== this.source.length) {
      throw new SyntaxError("invalid JSON");
    }
    return value;
  }

  private parseValue(depth: number): unknown {
    this.skipWhitespace();
    const token = this.source[this.index];
    if (token === "{") {
      return this.parseObject(depth);
    }
    if (token === "[") {
      return this.parseArray(depth);
    }
    if (token === '"') {
      return this.parseString();
    }
    if (this.source.startsWith("true", this.index)) {
      this.index += 4;
      return true;
    }
    if (this.source.startsWith("false", this.index)) {
      this.index += 5;
      return false;
    }
    if (this.source.startsWith("null", this.index)) {
      this.index += 4;
      return null;
    }
    return this.parseNumber();
  }

  private parseObject(depth: number): Record<string, unknown> {
    this.assertDepth(depth);
    this.index += 1;
    const result = Object.create(null) as Record<string, unknown>;
    const keys = new Set<string>();
    this.skipWhitespace();
    if (this.consume("}")) {
      return result;
    }
    while (true) {
      this.skipWhitespace();
      if (this.source[this.index] !== '"') {
        throw new SyntaxError("invalid JSON");
      }
      const key = this.parseString();
      if (keys.has(key)) {
        throw new SyntaxError("duplicate JSON key");
      }
      keys.add(key);
      this.skipWhitespace();
      this.expect(":");
      result[key] = this.parseValue(depth + 1);
      this.skipWhitespace();
      if (this.consume("}")) {
        return result;
      }
      this.expect(",");
    }
  }

  private parseArray(depth: number): unknown[] {
    this.assertDepth(depth);
    this.index += 1;
    const result: unknown[] = [];
    this.skipWhitespace();
    if (this.consume("]")) {
      return result;
    }
    while (true) {
      result.push(this.parseValue(depth + 1));
      this.skipWhitespace();
      if (this.consume("]")) {
        return result;
      }
      this.expect(",");
    }
  }

  private parseString(): string {
    const start = this.index;
    this.index += 1;
    while (this.index < this.source.length) {
      const token = this.source[this.index];
      if (token === '"') {
        this.index += 1;
        return JSON.parse(this.source.slice(start, this.index)) as string;
      }
      if (token === "\\") {
        this.index += 2;
      } else {
        if (token !== undefined && token.charCodeAt(0) < 0x20) {
          throw new SyntaxError("invalid JSON");
        }
        this.index += 1;
      }
    }
    throw new SyntaxError("invalid JSON");
  }

  private parseNumber(): number {
    const remaining = this.source.slice(this.index);
    const match = /^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/u.exec(remaining);
    if (match === null) {
      throw new SyntaxError("invalid JSON");
    }
    this.index += match[0].length;
    const value = Number(match[0]);
    if (!Number.isFinite(value)) {
      throw new SyntaxError("invalid JSON number");
    }
    return value;
  }

  private assertDepth(depth: number): void {
    if (depth >= MAX_JSON_DEPTH) {
      throw new SyntaxError("maximum JSON depth exceeded");
    }
  }

  private skipWhitespace(): void {
    while (
      this.source[this.index] === " " ||
      this.source[this.index] === "\t" ||
      this.source[this.index] === "\r" ||
      this.source[this.index] === "\n"
    ) {
      this.index += 1;
    }
  }

  private expect(token: string): void {
    if (!this.consume(token)) {
      throw new SyntaxError("invalid JSON");
    }
  }

  private consume(token: string): boolean {
    if (this.source[this.index] !== token) {
      return false;
    }
    this.index += 1;
    return true;
  }
}
