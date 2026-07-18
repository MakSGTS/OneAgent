# OneAgent Architecture Audit

Generated automatically by `oneagent-audit.sh`.

## Environment

- Project: `/Users/maxim_tomshin/Development/oneagent`
- Git branch: `main`
- Git commit: `a9f70a4`
- Rust host: `aarch64-apple-darwin`
- Rust version: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo version: `cargo 1.97.1 (c980f4866 2026-06-30)`

## Summary

| Check | Result |
|---|---:|
| Cargo manifests found | 11 |
| Workspace packages | 9 |
| Untracked Git entries | 25 |
| Backup files | 6 |
| Duplicate or obsolete paths | 1 |
| TODO/FIXME/HACK markers | 0 |
| `cargo check --workspace` | PASS |
| `cargo test --workspace` | PASS |
| `cargo clippy ... -D warnings` | PASS |
| `cargo doc --workspace --no-deps` | PASS |

## Workspace Packages

| Path | Package |
|---|---|
| `adapters/edt` | `oneagent-edt` |
| `adapters/filesystem` | `oneagent-workspace-fs` |
| `apps/cli` | `cli` |
| `apps/runtime` | `oneagent-runtime` |
| `crates/common` | `oneagent-common` |
| `crates/graph` | `oneagent-graph` |
| `crates/metadata` | `oneagent-metadata` |
| `crates/protocol` | `oneagent-protocol` |
| `crates/workspace` | `oneagent-workspace` |

## All Cargo Manifests

```text
./adapters/edt/Cargo.toml
./adapters/filesystem/Cargo.toml
./apps/cli/Cargo.toml
./apps/runtime/Cargo.toml
./Cargo.toml
./crates/common/Cargo.toml
./crates/graph/Cargo.toml
./crates/metadata/Cargo.toml
./crates/protocol/Cargo.toml
./crates/workspace/Cargo.toml
./runtime/Cargo.toml
```

## Git Status

```text
?? .github/
?? .gitignore
?? Cargo.lock
?? Cargo.toml
?? Cargo.toml.backup-20260718-142100
?? LICENSE
?? README.md
?? adapters/edt/Cargo.toml
?? adapters/edt/src/lib.rs.backup-20260718-180510
?? adapters/filesystem/
?? apps/
?? crates/
?? docs/Architecture.md
?? docs/Development.md
?? docs/Roadmap.md
?? docs/Vision.md
?? docs/adr/0001-project-vision.md
?? docs/adr/0002-runtime-composition-root.md
?? docs/adr/0003-semantic-domain-model.md
?? docs/adr/0004-filesystem-workspace-discovery.md
?? docs/adr/0005-edt-configuration-loading.md
?? docs/adr/0006-semantic-graph.md
?? docs/adr/0007-edt-to-semantic-graph.md
?? runtime/
?? rustfmt.toml
```

## Duplicate or Obsolete Paths

```text
runtime/ exists alongside apps/runtime/
```

## Backup Files

```text
./adapters/edt/src/lib.rs.backup-20260718-180510
./Cargo.toml.backup-20260718-142100
./crates/common/src/lib.rs.backup-20260718-151921
./crates/metadata/src/lib.rs.backup-20260718-151921
./crates/workspace/src/lib.rs.backup-20260718-151921
./crates/workspace/src/lib.rs.backup-20260718-172841
```

## TODO Markers

```text
none
```

## Check Logs

### cargo check — PASS

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
```

### cargo test — PASS

```text
test tests::tree_returns_children_by_parent ... ok
test tests::tree_filters_objects_by_kind ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/oneagent_protocol-e1f86fb6e64ba6d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/oneagent_runtime-2855a55740cacb9f)

running 5 tests
test app::builder::tests::builder_creates_application ... ok
test app::builder::tests::builder_requires_configuration ... ok
test app::lifecycle::tests::invalid_transition_returns_error ... ok
test app::lifecycle::tests::valid_lifecycle_sequence_succeeds ... ok
test config::tests::default_configuration_is_valid ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/oneagent_workspace-3858d760a60b507a)

running 1 test
test tests::workspace_finds_configuration_by_identifier ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/oneagent_workspace_fs-4e11b5b674fe130e)

running 3 tests
test tests::ignores_incomplete_edt_project ... ok
test tests::detects_edt_project ... ok
test tests::respects_depth_limit ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_common

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_edt

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_graph

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_metadata

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_protocol

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_workspace

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests oneagent_workspace_fs

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### cargo clippy — PASS

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
```

### cargo doc — PASS

```text
 Documenting oneagent-workspace-fs v0.1.0 (/Users/maxim_tomshin/Development/oneagent/adapters/filesystem)
 Documenting oneagent-workspace v0.1.0 (/Users/maxim_tomshin/Development/oneagent/crates/workspace)
 Documenting oneagent-graph v0.1.0 (/Users/maxim_tomshin/Development/oneagent/crates/graph)
 Documenting oneagent-metadata v0.1.0 (/Users/maxim_tomshin/Development/oneagent/crates/metadata)
 Documenting oneagent-edt v0.1.0 (/Users/maxim_tomshin/Development/oneagent/adapters/edt)
 Documenting oneagent-protocol v0.1.0 (/Users/maxim_tomshin/Development/oneagent/crates/protocol)
 Documenting oneagent-common v0.1.0 (/Users/maxim_tomshin/Development/oneagent/crates/common)
 Documenting cli v0.1.0 (/Users/maxim_tomshin/Development/oneagent/apps/cli)
 Documenting oneagent-runtime v0.1.0 (/Users/maxim_tomshin/Development/oneagent/apps/runtime)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.51s
   Generated /Users/maxim_tomshin/Development/oneagent/target/doc/cli/index.html and 8 other files
```

## Recommended Actions

- Remove or migrate obsolete duplicate paths before adding new crates.
- Delete backup files or move them outside the repository.
- Review and explicitly add or ignore all untracked files.
