# Testing Guide

The test suite is designed to be deterministic, isolated, and runnable without a
DeepSeek API key or external network access.

## Test layers

- **Unit tests** live beside the implementation under `src/`. Use them for
  deterministic local behavior such as configuration defaults, serialization,
  and validation.
- **Integration tests** live under `tests/`. Use them for behavior across
  component boundaries, such as the HTTP client contract and the agent's
  tool-call loop.
- **Test support** lives under `tests/support/`. Keep shared helpers small and
  test-only; helpers used by just one test should stay with that test.

## Running checks

Run the full test suite and the checks used by CI:

```sh
cargo test --all
cargo fmt --all -- --check
cargo clippy --all-targets
```

To run a focused integration test:

```sh
cargo test --test api_contract
cargo test --test agent_workflows
```

## API contract tests

`tests/api_contract.rs` exercises the real `DeepSeekClient` against a tiny local
HTTP server from `tests/support/`. The server binds to an ephemeral localhost
port, returns configured responses, and captures the request. Use it to test
request headers and JSON payloads, response parsing, HTTP errors, and streamed
server-sent events without calling the live API.

Add response fixtures only when they are reused. Keep assertions focused on the
observable request/response contract rather than reqwest implementation details.

## Agent workflow tests

`Agent::with_backend` accepts a `ChatBackend`, allowing workflow tests to use a
scripted backend instead of HTTP. A fake backend should return a finite sequence
of `ApiTurn`s and record the messages it receives. Cover externally observable
behavior such as final replies, tool results in follow-up requests, iteration
limits, and usage accumulation.

Keep API transport assertions in the API contract tests; agent tests should not
depend on HTTP details.

## Isolation guidelines

- Do not require credentials, live services, or external network access in the
  default test suite.
- Use `tempfile` for filesystem tests so temporary paths are unique and cleaned
  up even if an assertion fails.
- Avoid changing process-wide environment variables or the current directory;
  pass explicit paths and context instead.
- Keep tests independent so they remain safe under Cargo's parallel runner.
- Assert public behavior and error messages where meaningful; avoid relying on
  private implementation details.
