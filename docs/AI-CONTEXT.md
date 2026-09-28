# AI Agent Context

## Mission

Code Bundler v1.0.0 is a local Rust/Iced desktop application. **Generate** turns
a source directory into a strict text bundle plus an AI-oriented prompt.
**Extract** restores a valid bundle into a new directory and can apply optional
line replacements plus file add/delete/rename operations. The application
performs no network calls.

Treat implementation as truth in this order when evidence conflicts:

```text
source code > configuration > tests > existing documentation
```

Record unresolved discrepancies; do not silently invent intent.

## Architecture at a glance

```text
Iced GUI (app.rs)
  -> Generate orchestration (workflow.rs)
       -> scan/serialize (extractor.rs)
       -> optional compression (compression.rs)
       -> paired bundle + prompt files
  -> Extract orchestration (embedder.rs)
       -> parse/validate bundle
       -> portable paths (paths.rs)
       -> optional content and file operations
       -> unique restored directory
```

There is no API, database, account system, external integration, server,
container, queue, background daemon, runtime configuration file, or telemetry.

## Critical files

| File | Why it matters |
| --- | --- |
| `src/main.rs` | Entry point and embedded-font/Iced setup. |
| `src/app.rs` | UI state machine, fixed English view, native dialogs, async task dispatch. |
| `src/workflow.rs` | Generate workflow, prompt substitution, paired non-overwrite output. |
| `src/extractor.rs` | Scanner rules, decoding, skip behavior, bundle writer. |
| `src/embedder.rs` | Bundle parser, content/file change engine, restoration. |
| `src/paths.rs` | Portable relative-path and collision policy. |
| `src/compression/` | Optional source transformations. |
| `sample_prompt.txt` | Compile-time prompt contract. |
| `Cargo.toml` / `Cargo.lock` | Version, MSRV, build profiles, locked dependencies. |
| `.cargo/config.toml` / `build.rs` | Windows release linking and resources. |
| `.github/workflows/ci.yml` | Automated validation, not artifact publication. |

## Core invariants

- Only the exact `==code-bundler:v1==` marker is accepted.
- Entry sizes are UTF-8 byte lengths, not character counts.
- Scanning honors local/nested `.gitignore`, includes unignored hidden files,
  prunes `.git`, `.hg`, `.svn`, and `.jj`, and does not follow symlinks.
- NUL/unsupported text becomes a skip message; the original binary is not recoverable.
- UTF-16 BOM text is converted to UTF-8; arbitrary binary/UTF-32 is unsupported.
- Compression is opt-in, extension-aware, and occurs before serialization.
- All bundle paths must be relative and portable. Reject traversal, reserved
  names, case-insensitive duplicates, and file/parent conflicts.
- Restore validates all logical input before creating the output directory.
- Outputs use exclusive creation and suffix allocation; never intentionally overwrite.
- Generated bundle and prompt names are claimed as a pair with the same suffix.
- Replacement lines are one-based; duplicate targets are rejected and valid
  replacements are applied in descending line order.
- Delete/rename sources refer to original bundle paths. Added and renamed paths
  are validated together with unchanged entries as one final portable tree.
- A file may be line-edited and renamed, but it cannot be line-edited and deleted.
- A late filesystem failure can leave an explicitly reported incomplete restore directory.
- `sample_prompt.txt` and the font are compile-time inputs; rebuild after edits.

## Commands

```console
cargo run --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Minimum declared Rust version: 1.88. Current release packaging is Windows-only
and manual; CI validates code but does not publish artifacts.

## Do not change casually

- Bundle marker, delimiters, or length framing: this is a persisted format contract.
- Path validation or collision normalization: this is the extraction security boundary.
- Ignore/hidden/symlink behavior: changes affect both correctness and secret exposure.
- Non-overwrite/exclusive-creation semantics: changes can destroy user data.
- Prompt placeholders: renderer and compiled template form one contract.
- Change-record syntax, file-operation semantics, or line semantics: generated
  AI instructions depend on them.
- Compression rules: transformed source may no longer round-trip to original text.
- `Cargo.lock`, release profile, resource inputs, and license files: release integrity depends on them.

Before changing a durable constraint, read [06-technical-design.md](06-technical-design.md),
[07-security.md](07-security.md), and [DECISIONS.md](DECISIONS.md). Add or
supersede an ADR when a decision changes.

## Known risks and gaps

- Bundle restoration reads the full bundle into memory; neither workflow has a
  configured size limit, progress reporting, or cancellation.
- No transactional rollback after restore output creation.
- Hidden/unignored secrets can be bundled and later shared manually.
- Native GUI and fault-injection paths lack automation.
- Release artifact generation/checksums are manual and the executable is unsigned.
- Dependency audit identified an unmaintained transitive `ttf-parser` advisory;
  no known vulnerability was reported by the inspected audit.
- Internal module names can mislead: `extractor` generates a bundle and
  `embedder` restores it.

## Documentation map

- Product behavior: [01-idea.md](01-idea.md) through [04-features.md](04-features.md)
- Architecture/design: [05-architecture.md](05-architecture.md), [06-technical-design.md](06-technical-design.md), [14-data-flow.md](14-data-flow.md)
- Assurance: [07-security.md](07-security.md), [08-testing-strategy.md](08-testing-strategy.md)
- Build/use: [09-deployment.md](09-deployment.md) through [13-configuration.md](13-configuration.md)
- Terminology/intent: [15-glossary.md](15-glossary.md), [DECISIONS.md](DECISIONS.md), [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md)

Update documentation with behavior changes. Numbered documents must remain a
continuous sequence with no reserved gaps.
