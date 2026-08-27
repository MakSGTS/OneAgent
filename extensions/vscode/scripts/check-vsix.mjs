import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, rmSync } from "node:fs";

const expected = [
  "extension.vsixmanifest",
  "[Content_Types].xml",
  "extension/package.json",
  "extension/readme.md",
  "extension/LICENSE.txt",
  "extension/changelog.md",
  "extension/dist/symbol-search.js",
  "extension/dist/status.js",
  "extension/dist/mcp-client.js",
  "extension/dist/lifecycle.js",
  "extension/dist/extension.js",
  "extension/dist/configuration.js",
];

const outputs = ["oneagent-first.vsix", "oneagent-second.vsix"];
try {
  const inventories = outputs.map((output) => {
    run(command("pnpm"), ["run", "clean"]);
    run(command("vsce"), [
      "package",
      "--no-dependencies",
      "--allow-missing-repository",
      "--out",
      output,
    ]);
    return zipInventory(readFileSync(output));
  });
  assert.deepEqual(inventories[0], expected);
  assert.deepEqual(inventories[1], expected);
  assert.deepEqual(inventories[1], inventories[0]);
  process.stdout.write(`validated ${expected.length} VSIX archive files across two clean builds\n`);
} finally {
  for (const output of outputs) {
    rmSync(output, { force: true });
  }
}

function command(name) {
  return process.platform === "win32" ? `${name}.cmd` : name;
}

function run(executable, args) {
  const result = spawnSync(executable, args, {
    encoding: "utf8",
    env: process.env,
    stdio: "pipe",
  });
  assert.equal(
    result.status,
    0,
    `${executable} ${args.join(" ")} failed with bounded output:\n${bounded(result)}`,
  );
}

function bounded(result) {
  return `${result.stdout ?? ""}${result.stderr ?? ""}`.slice(-4_096);
}

function zipInventory(archive) {
  const end = findEndOfCentralDirectory(archive);
  const entryCount = archive.readUInt16LE(end + 10);
  let offset = archive.readUInt32LE(end + 16);
  const entries = [];
  for (let index = 0; index < entryCount; index += 1) {
    assert.equal(archive.readUInt32LE(offset), 0x02014b50, "invalid ZIP central directory");
    const fileNameLength = archive.readUInt16LE(offset + 28);
    const extraLength = archive.readUInt16LE(offset + 30);
    const commentLength = archive.readUInt16LE(offset + 32);
    const size = archive.readUInt32LE(offset + 24);
    const name = archive.subarray(offset + 46, offset + 46 + fileNameLength).toString("utf8");
    assert.ok(size > 0, `${name} must not be empty`);
    entries.push(name);
    offset += 46 + fileNameLength + extraLength + commentLength;
  }
  return entries;
}

function findEndOfCentralDirectory(archive) {
  const minimum = Math.max(0, archive.length - 65_557);
  for (let offset = archive.length - 22; offset >= minimum; offset -= 1) {
    if (archive.readUInt32LE(offset) === 0x06054b50) {
      return offset;
    }
  }
  assert.fail("missing ZIP end-of-central-directory record");
}
