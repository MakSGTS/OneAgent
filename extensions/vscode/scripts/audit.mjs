import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const extensionRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const repositoryRoot = path.resolve(extensionRoot, "../..");
const manifest = JSON.parse(readFileSync(path.join(extensionRoot, "package.json"), "utf8"));
const workspacePolicy = readFileSync(path.join(extensionRoot, "pnpm-workspace.yaml"), "utf8");

assert.equal(manifest.engines.vscode, "^1.134.0");
assert.deepEqual(manifest.activationEvents, []);
assert.deepEqual(manifest.extensionKind, ["workspace"]);
assert.equal(manifest.main, "./dist/extension.js");
assert.deepEqual(
  manifest.contributes.commands.map((entry) => entry.command),
  ["oneagent.connect", "oneagent.disconnect"],
);
assert.deepEqual(manifest.contributes.configuration.properties["oneagent.runtime.executable"], {
  type: "string",
  default: "oneagent-mcp",
  scope: "window",
  description: "Executable path or command used for the OneAgent Runtime.",
});
assert.equal(manifest.dependencies, undefined, "production dependencies are forbidden");
assert.deepEqual(manifest.devDependencies, {
  "@types/node": "24.13.3",
  "@types/vscode": "1.134.0",
  "@vscode/test-cli": "0.0.15",
  "@vscode/test-electron": "3.1.0",
  "@vscode/vsce": "3.9.2",
  typescript: "7.0.2",
});
assert.match(workspacePolicy, /'@vscode\/vsce-sign': false/u);
assert.match(workspacePolicy, /keytar: false/u);

const productionList = run(command("pnpm"), ["list", "--prod", "--depth", "0", "--json"]);
const productionPackages = JSON.parse(productionList);
assert.equal(productionPackages.length, 1);
assert.equal(productionPackages[0].dependencies, undefined);

const licenses = JSON.parse(run(command("pnpm"), ["licenses", "list", "--json"]));
assert.equal(Object.hasOwn(licenses, "Unlicensed"), false);
const unknownLicenses = licenses.Unknown ?? [];
assert.ok(unknownLicenses.length > 0, "the pinned vsce signing tooling exception must remain explicit");
assert.equal(
  unknownLicenses.every((entry) => /^@vscode\/vsce-sign(?:-|$)/u.test(entry.name)),
  true,
  "only non-production vsce signing tooling may have unknown registry license metadata",
);

const tracked = run("git", ["ls-files", "--", "extensions/vscode"])
  .trim()
  .split(/\r?\n/u)
  .filter(Boolean);
assert.ok(tracked.length > 0);
const prohibitedTracked = tracked.filter((entry) =>
  /\/(?:dist|dist-test|node_modules|\.vscode-test|coverage)\/|\.vsix$/u.test(entry),
);
assert.deepEqual(prohibitedTracked, []);

for (const ignored of [
  "extensions/vscode/dist/probe.js",
  "extensions/vscode/dist-test/probe.js",
  "extensions/vscode/node_modules/probe",
  "extensions/vscode/.vscode-test/probe",
  "extensions/vscode/coverage/probe",
  "extensions/vscode/probe.vsix",
]) {
  const result = spawnSync("git", ["check-ignore", "--quiet", ignored], { cwd: repositoryRoot });
  assert.equal(result.status, 0, `${ignored} must remain ignored`);
}

const extensionSource = readFileSync(path.join(extensionRoot, "src/extension.ts"), "utf8");
for (const commandName of ["oneagent.connect", "oneagent.disconnect"]) {
  assert.ok(extensionSource.includes(commandName));
}
assert.ok(extensionSource.includes("oneagent.runtime.executable"));
for (const forbidden of [
  "onStartupFinished",
  "createOutputChannel",
  "createDiagnosticCollection",
  "createWebviewPanel",
  "shell: true",
  "exec(",
  "telemetry",
]) {
  assert.equal(extensionSource.includes(forbidden), false, `${forbidden} is deferred`);
}

const testGroups = [
  ["test/unit", ".test.ts"],
  ["test/integration", ".test.ts"],
  ["test/extension", ".test.js"],
];
for (const [directory, suffix] of testGroups) {
  const matches = tracked.filter(
    (entry) => entry.includes(`extensions/vscode/${directory}/`) && entry.endsWith(suffix),
  );
  assert.ok(matches.length > 0, `${directory} must contain tracked tests`);
  const cases = matches.reduce(
    (count, entry) => count + (readFileSync(path.join(repositoryRoot, entry), "utf8").match(/\btest\(/gu)?.length ?? 0),
    0,
  );
  assert.ok(cases > 0, `${directory} must contain non-zero test cases`);
}

for (const entry of tracked) {
  if (entry === "extensions/vscode/scripts/audit.mjs") {
    continue;
  }
  const content = readFileSync(path.join(repositoryRoot, entry));
  if (content.includes(0)) {
    continue;
  }
  const text = content.toString("utf8");
  for (const forbidden of [
    /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/u,
    /\bAKIA[0-9A-Z]{16}\b/u,
    /\/Users\/[A-Za-z0-9._-]+\//u,
    /[A-Za-z]:\\Users\\[^\\]+\\/u,
  ]) {
    assert.equal(forbidden.test(text), false, `${entry} contains forbidden secret/path material`);
  }
}

for (const document of [
  path.join(repositoryRoot, "docs/Architecture.md"),
  path.join(repositoryRoot, "docs/Roadmap.md"),
  path.join(extensionRoot, "README.md"),
]) {
  checkLocalLinks(document);
}

process.stdout.write(
  `audited ${tracked.length} tracked extension files, ${Object.keys(licenses).length} license groups, and 3 documents\n`,
);

function command(name) {
  return process.platform === "win32" ? `${name}.cmd` : name;
}

function run(executable, args) {
  const result = spawnSync(executable, args, {
    cwd: executable === "git" ? repositoryRoot : extensionRoot,
    encoding: "utf8",
    env: process.env,
  });
  assert.equal(
    result.status,
    0,
    `${executable} ${args.join(" ")} failed with bounded output:\n${`${result.stdout ?? ""}${result.stderr ?? ""}`.slice(-4_096)}`,
  );
  return result.stdout;
}

function checkLocalLinks(document) {
  const text = readFileSync(document, "utf8");
  for (const match of text.matchAll(/\[[^\]]*\]\(([^)]+)\)/gu)) {
    const target = match[1];
    if (
      target === undefined ||
      target.startsWith("#") ||
      /^[a-z][a-z0-9+.-]*:/iu.test(target)
    ) {
      continue;
    }
    const decoded = decodeURIComponent(target.split("#", 1)[0] ?? "");
    assert.ok(decoded.length > 0);
    const result = spawnSync("git", ["cat-file", "-e", `HEAD:${path.relative(repositoryRoot, path.resolve(path.dirname(document), decoded)).replaceAll(path.sep, "/")}`], {
      cwd: repositoryRoot,
    });
    assert.equal(result.status, 0, `${path.relative(repositoryRoot, document)} has missing link ${target}`);
  }
}
