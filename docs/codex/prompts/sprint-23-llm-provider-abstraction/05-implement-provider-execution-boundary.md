# Implement Sprint 23 Provider Execution Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/architecture/llm-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- committed Tasks 3-4 provider domain and request boundary

## Prerequisites / Required gate

Require committed Task 4, successful provider-neutral package validation, and
clean task-owned state. Stop rather than changing accepted domain, request,
capability, secret, timeout, retry, cancellation, or error architecture.

## Task

Implement only the ADR-0045 provider-neutral asynchronous discovery/execution
seam and its accepted cancellation, execution-policy, terminal-error, and
cleanup behavior. Prove it with deterministic repository-owned fake providers;
do not implement a concrete provider or transport.

## Provider-neutral ownership and API boundary

Implement the exact object-safe or otherwise substitutable async provider seam
accepted by ADR-0045, including provider identity and owned future/result
lifetimes. Keep provider construction, Runtime registration, transport, and
global execution outside the crate.

## Model identity, discovery, and capability contract

Expose accepted asynchronous model discovery and canonical owned results through
the same provider identity boundary. Fake results must exercise order,
duplicates, empty success, and typed failure as applicable.

## Request, response, usage, finish, and compatibility contract

Accept only Task 4 validated requests and return only Task 3 response values.
No adapter may bypass compatibility validation or return provider-specific
domain data through the shared seam.

## Configuration and secret-handling contract

Pass only the accepted secret-safe configuration inputs. Provider errors and
fake observation must not copy or expose credential contents.

## Timeout, retry, cancellation, and cleanup contract

Implement exactly the first-slice policy from ADR-0045. If timeout/retry are
representation-only or disabled, prove that no hidden clock, delay, replay, or
automatic attempt occurs. Propagate accepted receiver-only cancellation through
discovery/execution futures, define terminal precedence, and prove no fake task,
future, handle, or observation remains after completion/cancellation.

## Error taxonomy and provider mapping

Return the accepted provider-neutral validation, compatibility, provider,
transport/protocol placeholder, timeout, retry/exhaustion, cancellation, and
internal classifications only as ADR-0045 defines them. Preserve bounded and
redacted diagnostic data.

## Contract corpus, fake, fixture, or controlled-endpoint oracle

Create minimum deterministic fake providers under the package tests. Cover
successful discovery/execution, empty discovery, invalid/incompatible preflight,
provider failure, cancellation before and during execution, every enabled or
disabled policy path, redaction, reorder, and repeated fresh execution as
applicable. Use explicit synchronization, not sleeps or live network.

## Consumer and provider-adapter compatibility

Prove multiple independent fake implementations satisfy one public seam. Keep
Context Engine and Runtime source-compatible and unmodified unless ADR-0045
explicitly requires a minimal compile-only compatibility seam.

## Scope

### Included

- Provider-neutral provider/future/execution context interfaces, discovery and
  execution delegation, accepted cancellation/policy/error behavior, Rustdoc,
  unit tests, and deterministic public fake/conformance evidence assigned here.

### Excluded

Concrete OpenAI-compatible/LM Studio/Ollama adapters, HTTP/JSON/SSE, live
providers/credentials, environment/configuration loading, Runtime service or
route, Context-to-prompt mapping, tokenizer, streaming, tools, structured
output/media, conversations, persistence, automatic behavior not accepted by
ADR-0045, current-state docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- Independent fake providers are substitutable through one accepted public
  async seam without provider-specific shared-domain values.
- Discovery and execution use exact provider/model identity, capability-aware
  validated requests, owned responses, and typed terminal errors.
- Cancellation, timeout/retry policy, attempt/replay behavior, precedence, and
  cleanup match ADR-0045 exactly; unsupported behavior is observably absent.
- Invalid/incompatible requests cannot reach provider execution.
- Secret and sensitive request/response data remain within accepted access and
  redaction boundaries, including failure paths.
- Deterministic tests prove positive, negative, cancellation, ordering, cleanup,
  and repeated fresh cases without network, credentials, sleeps, or detached
  work; the focused targets are non-zero and the workspace remains green.

## Repository Safety

Modify only the provider-neutral crate files and minimum focused/public tests
accepted for Task 5. Preserve manifests unless mechanically required by an
accepted repository-local or dev-only test dependency, analysis, Runtime,
protocol, adapters, docs, prompts, `.codex/`, credentials, and unrelated paths.
Stage only task-owned files.

## Task-specific Validation

- Run non-zero provider/discovery/execution/preflight/cancellation/policy/error/
  redaction/cleanup/reorder/repetition tests.
- Run the provider-neutral package tests and list public conformance cases.
- Run affected analysis or Runtime tests only when observable compatibility
  could be affected.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 23 provider execution boundary`

## Final report additions

Report provider seam and lifetimes, discovery/execution behavior, cancellation,
timeout/retry policy, errors/redaction, fake/conformance matrix, cleanup,
compatibility, focused/full validation, changed paths, commit, and final Git
state.
