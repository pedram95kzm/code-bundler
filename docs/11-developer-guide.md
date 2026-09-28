# Developer Guide

## Start here

Code Bundler is a single Rust binary crate with an Iced desktop UI. It performs
local filesystem transformations and has no server, database, or external API.

Prerequisites:

- Git;
- Rust 1.88 or newer;
- native build prerequisites for Iced and the selected Rust target;
- Windows/MSVC tooling for the current release build.

Clone the repository, then validate the checkout:

```console
cargo test --all-targets --locked
cargo run --locked
```

No environment file, secret, database, or service bootstrap is needed.

## Project structure

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Process entry point, embedded font setup, Iced launch. |
| `src/app.rs` | GUI state, messages, view construction, dialog/task wiring. |
| `src/workflow.rs` | Generate orchestration, prompt rendering, paired output naming/writes. |
| `src/extractor.rs` | Project scanning, ignore behavior, decoding, and bundle serialization. |
| `src/embedder.rs` | Strict bundle parsing, content/file change application, safe restoration. |
| `src/paths.rs` | Portable path validation and collision keys. |
| `src/compression/` | Optional extension-routed source compression. |
| `sample_prompt.txt` | Prompt template compiled into the executable. |
| `assets/` | Compile-time application icon and embedded font inputs. |
| `build.rs` | Windows resource/icon generation. |
| `.cargo/config.toml` | Target-specific Cargo/rustc settings. |
| `.github/workflows/ci.yml` | Formatting, lint, and test validation. |
| `docs/` | Current-state project knowledge base. |

The internal names `extractor` and `embedder` describe transformations, but
can be counterintuitive: `extractor` reads a project into a bundle, while
`embedder` restores a bundle into a directory.

## Development loop

```console
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo run --locked
```

Before release work, also run:

```console
cargo build --release --locked
```

Use `cargo +1.88.0 test --all-targets --locked` when explicitly checking the
declared minimum Rust version and that toolchain is installed.

## Where to make changes

- UI labels, layout, and interaction state: `src/app.rs`.
- Generate sequence, output naming, or prompt substitutions: `src/workflow.rs`.
- Traversal, ignored files, text decoding, framing output: `src/extractor.rs`.
- Bundle validation, restore behavior, line replacements, or file operations: `src/embedder.rs`.
- Cross-platform path safety: `src/paths.rs`.
- Compression routing or algorithms: `src/compression/`.
- Prompt wording: `sample_prompt.txt` (requires rebuilding the binary).
- Application metadata/icon: `build.rs` and `assets/`.

Keep policy in the existing domain module rather than adding it to the GUI.
I/O-heavy work is dispatched through Iced tasks so the event loop can remain
responsive.

## Critical invariants

Do not change these casually:

- The bundle begins with the exact v1 marker and uses byte-length framing.
- Restore validates the complete input before creating output.
- Bundle paths remain relative, portable, traversal-free, and collision-free.
- Output files/directories are never intentionally overwritten.
- A generated bundle and generated prompt claim the same suffix as a pair.
- Replacement line numbers are one-based and duplicate targets are rejected.
- Line replacements are applied in descending line order per file.
- Delete/rename sources use original bundle paths; added/renamed destinations
  must leave one valid, portable final tree.
- All content and structural changes are validated before output creation.
- Compression is opt-in; unchanged content must round-trip byte-for-byte after decoding.
- The prompt template is a compile-time input, not a runtime file lookup.

See [06-technical-design.md](06-technical-design.md) before changing a format or
algorithm and [DECISIONS.md](DECISIONS.md) before changing a recorded decision.

## Testing changes

Add focused unit tests beside the changed module. Filesystem workflows use
unique temporary directories and should clean them up after assertions. A
format/path change needs success and adversarial failure cases. A GUI change
requires manual exercise because there is no UI automation.

The full test inventory and known gaps are in
[08-testing-strategy.md](08-testing-strategy.md).

## Debugging

- Run a debug build with `cargo run --locked`.
- Reduce failures to a small temporary project or bundle.
- Inspect the GUI's exact status text; no persistent log exists.
- For bundle problems, verify the marker and UTF-8 byte lengths, not character counts.
- For path failures, check every component against `src/paths.rs` rules.
- For prompt changes, rebuild before testing because `include_str!` embeds the template.

## Coding conventions observed

- Rustfmt is authoritative for formatting.
- Clippy warnings fail CI.
- Errors are returned as user-facing strings across workflow/task boundaries.
- Tests are colocated with modules.
- Unsafe code is not used in the inspected source.
- Dependencies are locked for validation and release commands.

## Common mistakes

- Treating the filename length as a character count instead of UTF-8 bytes.
- Following symlinks or broadening ignore sources without documenting the security effect.
- Creating output before all input/path/conflict validation completes.
- Reusing a path with different case, which can collide on case-insensitive filesystems.
- Overwriting an existing generated output instead of allocating a suffix.
- Editing generated release contents without regenerating their checksums.
- Changing `sample_prompt.txt` and testing an old executable.
- Documenting proposed behavior as current behavior.

## Documentation workflow

When behavior changes, update the relevant numbered document, this guide if
developer workflow changes, `AI-CONTEXT.md` for critical constraints, and an
ADR when a durable architectural decision changes. Keep numbered documents in
a continuous sequence.
