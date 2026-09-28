# Testing Strategy

## Framework and layout

Tests use Rust's built-in `#[test]` framework and live beside implementation
code in `#[cfg(test)]` modules. There is no separate `tests/` directory, mock
framework, snapshot framework, coverage service, or E2E harness.

The current suite contains 28 tests.

| Area | Tests | Behaviors covered |
| --- | ---: | --- |
| GUI helpers/state | 2 | Quoted path input and Persian request state. |
| Brace compression | 3 | Token boundaries, line-comment endings, raw strings/heredocs. |
| Python/general compression | 2 | Indentation/multiline strings and unchanged unknown formats. |
| Portable paths | 2 | Valid path/key and rejection of traversal/non-portable names. |
| Project scanning/encoding | 5 | NUL detection, UTF-16 LE, `.gitignore`, VCS pruning, invalid-text skip. |
| Bundle restore/change set | 12 | Framing, strict marker, unsafe/conflicting paths, round trip, multiline replacements, add/delete/rename, continuation escaping, CRLF, and operation conflicts. |
| Generation workflow | 2 | Template substitution, file-operation prompt contract, and non-overwriting paired output names. |

## Test types

### Unit tests

State-machine compression, path parsing, line-span behavior, prompt rendering,
and small parser rules are direct unit tests.

### Filesystem integration-style tests

Scanner, generation, round-trip restore, and change-set tests create uniquely
named directories under the operating-system temporary directory, exercise real
filesystem I/O, assert results, and remove the directory on success.

These are inside the binary crate rather than Cargo integration-test targets,
but they test multiple modules together.

### UI and E2E tests

No automated test launches the window, exercises native file dialogs, clicks
controls, verifies layout, or tests accessibility. There is no packaged-binary
E2E test in the repository.

## Local commands

Run from the repository root:

```console
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

The declared minimum Rust version is 1.88. Compatibility can be checked with an
installed matching toolchain:

```console
cargo +1.88.0 test --all-targets --locked
```

## CI

`.github/workflows/ci.yml` runs on every push and pull request with:

- Ubuntu latest and Windows latest;
- stable Rust plus rustfmt and Clippy;
- formatting check;
- locked Clippy with warnings denied;
- locked tests;
- read-only repository permission and concurrency cancellation.

CI does not build release artifacts, run dependency auditing, measure coverage,
sign binaries, or publish a GitHub release.

## Release validation outside the automated suite

The current v1 preparation included a release build, a short Windows GUI launch
smoke test, package-content comparison, checksum verification, Rust 1.88 test,
and RustSec audit. These actions are not encoded as repository automation and
must not be assumed for future releases.

## Important gaps

- Native GUI, dialog, focus, RTL rendering, scaling, and accessibility tests.
- Cancellation and simultaneous-message behavior under the real event loop.
- Fault injection for disk-full, permission loss, short writes, and cleanup failures.
- Race tests for output-name claims and source changes during scanning.
- Large-file/project memory and duration benchmarks.
- UTF-16 BE, malformed UTF-16, UTF-32, BOM, and mixed newline edge cases.
- More adversarial bundle framing and change-parser cases, including operation
  permutations and record-like lines inside added content.
- Compression corpus tests across every routed extension.
- Fuzzing of bundle, path, and change-record parsers.
- Unix-only unusual filename tests and macOS execution.
- No coverage number is available; do not infer one from test count.

## Test data constraints

Tests use inline bytes and temporary folders; no credentials or external
services are needed. A panic before cleanup can leave a temporary directory,
because the tests do not use an automatic temp-directory guard.
