import { Buffer } from "node:buffer";

export const MAX_EXECUTABLE_BYTES = 4_096;

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

function invalidConfiguration(): ExecutableConfiguration {
  return {
    ok: false,
    code: "invalid_configuration",
    message: "OneAgent Runtime configuration is invalid.",
  };
}
