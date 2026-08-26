import assert from "node:assert/strict";
import test from "node:test";

import { MAX_EXECUTABLE_BYTES, validateExecutable } from "../../src/configuration";

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
