# Current Project Review — 2026-08-26

## Review decision

`non-blocking finding`

The complete current repository was reviewed from a clean worktree at commit
`5940ae69345b86dd2d1fd689d476353193c49bb9`. The review covered all 17 Rust
workspace packages, their production code and tests, workspace and CI
configuration, and the relevant architecture documentation and ADRs.

One actionable Medium-severity correctness finding was identified. No Critical,
High, or Low findings were identified. No source or configuration changes were
made as part of this review.

## Findings by severity

### Critical

None.

### High

None.

### Medium

#### Whitespace-only EDT source selectors panic instead of producing a typed malformed observation

**Location:** `adapters/edt/src/event_subscription.rs:602-616`

The EDT Event Subscription parser preserves XML text whitespace and checks a
source selector and its dot-separated components with `is_empty()`. It then
constructs `EntityName` values with `expect`. `EntityName::new`, however,
rejects values whose trimmed representation is empty. Consequently, a
structurally valid descriptor containing either of the following values reaches
an `expect` and panics:

```xml
<source><types> </types></source>
```

```xml
<source><types>CatalogObject. </types></source>
```

The same failure applies when the family component contains only whitespace,
for example `" .Products"`.

This behavior contradicts ADR-0033, which defines malformed selectors in a
structurally valid descriptor as rejected observations that do not prevent
accepted selectors or a valid handler from being processed. Existing parser
tests cover an empty value, an empty component, and additional components, but
do not cover whitespace-only values or components.

**Impact:** malformed or externally supplied EDT input can unwind through the
adapter instead of returning the declared typed observation. In the Runtime
initial-build path, the failed blocking build task prevents startup. During a
watched rebuild, the build fails and the previous snapshot is retained. Direct
adapter consumers receive a panic rather than the documented parser result.

**Recommended remediation:** classify `raw_selector.trim().is_empty()` as
`EmptyValue`, classify whitespace-only components as `EmptyComponent`, and
replace the two infallible `expect` calls with fallible classification. Add
parser and production graph-build coverage for `" "`, `"CatalogObject. "`, and
`" .Products"`.

### Low

None.

## Residual risks and coverage gaps

- MCP retained-CR maximum-frame overlap still lacks the two explicit boundary
  tests already documented in the Sprint 28 review.
- Provider tests use controlled local HTTP peers; live provider interoperability
  and operational behavior remain intentionally deferred.
- Validation covered the current macOS host. Platform-specific filesystem,
  watcher, and process behavior on Linux and Windows was not revalidated by
  this review.

These are coverage or deferred-scope notes, not additional actionable findings
from this review.

## Validation

The clean-context reviewer completed the following checks:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo check --workspace --all-targets` | passed |
| `cargo test --workspace --all-targets` | passed with local loopback permission; the initial sandboxed run failed only because two CLI tests could not bind loopback sockets |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | passed |
| `cargo doc --workspace --no-deps` | passed |
| `cargo test --workspace --doc` | passed; all 17 crate doctest suites contained zero doctests |
| `git diff --check` | passed |

The reviewer also confirmed that the repository remained clean after all
read-only inspection and validation commands. Cargo generated or refreshed only
ignored artifacts under `target/`.

## Conclusion

The current project has one confirmed non-blocking Medium-severity parser defect
at the reviewed baseline. The rest of the review produced no actionable
Critical, High, or Low findings. Remediation was intentionally not performed.
