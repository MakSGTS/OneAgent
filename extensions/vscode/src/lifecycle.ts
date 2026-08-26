import type { ConnectionTarget } from "./configuration";
import { RuntimeClientFailure } from "./mcp-client";
import type { ConnectionState } from "./status";

export interface RuntimeClientPort {
  connect(executable: string, cwd: string): Promise<ConnectionState>;
  disconnect(): Promise<ConnectionState>;
}

export type RuntimeClientFactory = (
  onStateChange: (state: ConnectionState, failure?: RuntimeClientFailure) => void,
) => RuntimeClientPort;

export interface LifecycleOptions {
  readonly readTarget: () => ConnectionTarget;
  readonly renderState: (state: ConnectionState) => void;
  readonly createClient: RuntimeClientFactory;
}

export class ExtensionLifecycle {
  private readonly client: RuntimeClientPort;
  private currentState: ConnectionState = "disconnected";
  private connectOperation: Promise<ConnectionState> | undefined;
  private disconnectOperation: Promise<ConnectionState> | undefined;
  private deactivationOperation: Promise<void> | undefined;
  private deactivating = false;
  private disposed = false;

  public constructor(private readonly options: LifecycleOptions) {
    this.client = options.createClient((state) => this.transition(state));
    this.options.renderState(this.currentState);
  }

  public get state(): ConnectionState {
    return this.currentState;
  }

  public connect(): Promise<ConnectionState> {
    if (
      this.disposed ||
      this.deactivating ||
      this.connectOperation !== undefined ||
      (this.currentState !== "disconnected" && this.currentState !== "failed")
    ) {
      return Promise.resolve(this.currentState);
    }

    const target = this.options.readTarget();
    if (!target.ok) {
      this.transition("failed");
      return Promise.resolve(this.currentState);
    }

    const operation = this.connectClient(target.executable, target.cwd);
    this.connectOperation = operation;
    void operation.finally(() => {
      if (this.connectOperation === operation) {
        this.connectOperation = undefined;
      }
    });
    return operation;
  }

  public disconnect(): Promise<ConnectionState> {
    if (this.disconnectOperation !== undefined) {
      return this.disconnectOperation;
    }
    if (this.currentState === "disconnected") {
      return Promise.resolve(this.currentState);
    }

    const operation = this.disconnectClient();
    this.disconnectOperation = operation;
    void operation.finally(() => {
      if (this.disconnectOperation === operation) {
        this.disconnectOperation = undefined;
      }
    });
    return operation;
  }

  public configurationChanged(): Promise<ConnectionState> {
    if (this.currentState === "disconnected") {
      return Promise.resolve(this.currentState);
    }
    return this.disconnect();
  }

  public deactivate(): Promise<void> {
    if (this.disposed) {
      return Promise.resolve();
    }
    if (this.deactivationOperation !== undefined) {
      return this.deactivationOperation;
    }

    const operation = this.deactivateClient();
    this.deactivationOperation = operation;
    void operation.then(
      () => (this.deactivationOperation = undefined),
      () => (this.deactivationOperation = undefined),
    );
    return operation;
  }

  private async deactivateClient(): Promise<void> {
    this.deactivating = true;
    try {
      await this.disconnect();
      if (this.connectOperation !== undefined) {
        await this.connectOperation;
      }
      if (this.currentState !== "disconnected") {
        await this.disconnect();
      }
      if (this.currentState !== "disconnected") {
        throw new RuntimeClientFailure("shutdown_failed");
      }
      this.disposed = true;
    } finally {
      this.deactivating = false;
    }
  }

  private async connectClient(executable: string, cwd: string): Promise<ConnectionState> {
    try {
      const state = await this.client.connect(executable, cwd);
      this.transition(state);
    } catch {
      if (this.currentState !== "disconnecting" && this.currentState !== "disconnected") {
        this.transition("failed");
      }
    }
    return this.currentState;
  }

  private async disconnectClient(): Promise<ConnectionState> {
    try {
      const state = await this.client.disconnect();
      this.transition(state);
    } catch {
      this.transition("failed");
    }
    return this.currentState;
  }

  private transition(state: ConnectionState): void {
    if (this.currentState === state) {
      return;
    }
    this.currentState = state;
    this.options.renderState(state);
  }
}
