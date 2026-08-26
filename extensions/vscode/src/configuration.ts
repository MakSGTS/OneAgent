import { Buffer } from "node:buffer";

export const MAX_EXECUTABLE_BYTES = 4_096;

export interface WorkspaceFolderInput {
  readonly scheme: string;
  readonly fsPath: string;
}

export interface ConnectionInputs {
  readonly trusted: boolean;
  readonly folders: readonly WorkspaceFolderInput[] | undefined;
  readonly executable: unknown;
}

export type ConnectionTarget =
  | { readonly ok: true; readonly executable: string; readonly cwd: string }
  | {
      readonly ok: false;
      readonly code: "invalid_configuration" | "unsupported_workspace";
      readonly message:
        | "OneAgent Runtime configuration is invalid."
        | "OneAgent requires one trusted local workspace.";
    };

export type ExecutableConfiguration =
  | { readonly ok: true; readonly executable: string }
  | {
      readonly ok: false;
      readonly code: "invalid_configuration";
      readonly message: "OneAgent Runtime configuration is invalid.";
    };

export function validateExecutable(value: unknown): ExecutableConfiguration {
  if (typeof value !== "string") {
    return invalidConfiguration();
  }

  const executable = value.trim();
  if (
    executable.length === 0 ||
    Buffer.byteLength(executable, "utf8") > MAX_EXECUTABLE_BYTES
  ) {
    return invalidConfiguration();
  }

  return { ok: true, executable };
}

export function resolveConnectionTarget(inputs: ConnectionInputs): ConnectionTarget {
  if (
    !inputs.trusted ||
    inputs.folders === undefined ||
    inputs.folders.length !== 1 ||
    inputs.folders[0]?.scheme !== "file" ||
    inputs.folders[0].fsPath.length === 0
  ) {
    return {
      ok: false,
      code: "unsupported_workspace",
      message: "OneAgent requires one trusted local workspace.",
    };
  }

  const executable = validateExecutable(inputs.executable);
  if (!executable.ok) {
    return executable;
  }
  return { ok: true, executable: executable.executable, cwd: inputs.folders[0].fsPath };
}

function invalidConfiguration(): ExecutableConfiguration {
  return {
    ok: false,
    code: "invalid_configuration",
    message: "OneAgent Runtime configuration is invalid.",
  };
}
