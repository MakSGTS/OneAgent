import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_EXECUTABLE_BYTES,
  resolveConnectionTarget,
  validateExecutable,
} from "../../src/configuration";

test("validates default, trimmed, and exact-bound executable values", () => {
  assert.deepEqual(validateExecutable("oneagent-mcp"), {
    ok: true,
    executable: "oneagent-mcp",
  });
  assert.deepEqual(validateExecutable("  /opt/oneagent-mcp  "), {
    ok: true,
    executable: "/opt/oneagent-mcp",
  });
  assert.deepEqual(validateExecutable("x".repeat(MAX_EXECUTABLE_BYTES)), {
    ok: true,
    executable: "x".repeat(MAX_EXECUTABLE_BYTES),
  });
});

test("rejects missing, empty, wrong-type, and one-byte-over values", () => {
  const expected = {
    ok: false,
    code: "invalid_configuration",
    message: "OneAgent Runtime configuration is invalid.",
  };

  for (const value of [undefined, null, 42, {}, "", "   "]) {
    assert.deepEqual(validateExecutable(value), expected);
  }
  assert.deepEqual(
    validateExecutable("x".repeat(MAX_EXECUTABLE_BYTES + 1)),
    expected,
  );
});

test("applies the executable limit to UTF-8 bytes", () => {
  assert.equal(
    validateExecutable("Ж".repeat(MAX_EXECUTABLE_BYTES / 2)).ok,
    true,
  );
  assert.equal(
    validateExecutable(`Ж${"x".repeat(MAX_EXECUTABLE_BYTES - 1)}`).ok,
    false,
  );
});

test("resolves exactly one trusted file workspace with the bounded executable", () => {
  assert.deepEqual(
    resolveConnectionTarget({
      trusted: true,
      folders: [{ scheme: "file", fsPath: "/workspace" }],
      executable: "  oneagent-mcp  ",
    }),
    { ok: true, executable: "oneagent-mcp", cwd: "/workspace" },
  );
});

test("rejects untrusted, missing, multi-root, virtual, and empty-path workspaces first", () => {
  const expected = {
    ok: false,
    code: "unsupported_workspace",
    message: "OneAgent requires one trusted local workspace.",
  };
  const workspaceCases = [
    { trusted: false, folders: [{ scheme: "file", fsPath: "/workspace" }] },
    { trusted: true, folders: undefined },
    { trusted: true, folders: [] },
    {
      trusted: true,
      folders: [
        { scheme: "file", fsPath: "/one" },
        { scheme: "file", fsPath: "/two" },
      ],
    },
    { trusted: true, folders: [{ scheme: "vscode-remote", fsPath: "/workspace" }] },
    { trusted: true, folders: [{ scheme: "file", fsPath: "" }] },
  ];
  for (const inputs of workspaceCases) {
    assert.deepEqual(resolveConnectionTarget({ ...inputs, executable: "" }), expected);
  }
});
