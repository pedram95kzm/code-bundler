# Requirements

The following requirements describe implemented behavior, not an aspirational
roadmap.

## Functional requirements

| ID | Current requirement | Evidence |
| --- | --- | --- |
| FR-01 | Present a desktop window with mutually exclusive Embed and Extract modes. | `src/main.rs`, `src/app.rs` |
| FR-02 | Let the user select or type the project, bundle, and optional modification paths. | `src/app.rs`, `rfd` dependency |
| FR-03 | Recursively discover files while honoring root and nested `.gitignore` files. | `src/extractor.rs` |
| FR-04 | Include hidden files unless ignored, but prune `.git`, `.hg`, `.svn`, and `.jj` entries. | `src/extractor.rs` |
| FR-05 | Generate one strict v1 bundle and one prompt in the selected project directory. | `src/workflow.rs`, `src/extractor.rs` |
| FR-06 | Never overwrite a generated bundle, prompt, or extraction directory; add numeric suffixes instead. | `src/workflow.rs`, `src/embedder.rs` |
| FR-07 | Exclude root-level generated bundle and prompt names from subsequent bundles. | `src/extractor.rs` |
| FR-08 | Preserve valid text bytes, empty files, final newlines, and header-like content when compression is off. | Length-framed format and round-trip tests |
| FR-09 | Mark binary or invalid text files as skipped rather than storing their data. | `src/extractor.rs` |
| FR-10 | Optionally compact supported languages without changing source files. | `src/compression/` |
| FR-11 | Reject unknown, unversioned, malformed, duplicate, conflicting, or unsafe bundle entries. | `src/embedder.rs`, `src/paths.rs` |
| FR-12 | Restore accepted bundle entries beneath a newly created sibling directory. | `src/embedder.rs` |
| FR-13 | Parse optional records for line replacement and file addition, deletion, and rename; validate the complete change set before output creation. | `src/embedder.rs` |
| FR-14 | Preserve the target file's preferred line ending in replacement content. | `src/embedder.rs` |
| FR-15 | Disable editing and mode changes while an operation is running, then show success, warning, or error text. | `src/app.rs` |
| FR-16 | Support shaped Persian/RTL text through embedded Vazirmatn fonts and advanced shaping. | `src/main.rs`, `src/app.rs`, `Cargo.toml` |

## Non-functional requirements visible in the implementation

### Correctness and data integrity

- Output files use `create_new`; existing outputs are not overwritten.
- Bundle content lengths are UTF-8 byte counts, not character counts.
- Paths are compared case-insensitively to prevent bundles that fail on
  case-insensitive filesystems.
- Added and renamed paths are checked together with unchanged final paths for
  duplicates and file/parent conflicts.
- Prompt generation removes incomplete files when its paired write fails.
- Source read failures are warnings; output write failures abort the operation.

### Security

- Absolute paths, traversal components, control characters, Windows-invalid
  characters, reserved device names, and ambiguous components are rejected.
- Symbolic links are not followed by the scanner.
- Version-control metadata directories are pruned.
- No network access, credentials, or executable command invocation exists in
  the application runtime.

See [07-security.md](07-security.md) for limitations and trust boundaries.

### Performance

- Files are processed sequentially and one source file is buffered at a time.
- The prompt copies the completed bundle through buffered I/O rather than
  holding a second complete repository string in memory.
- Bundle extraction reads the selected bundle into memory before parsing.
- There are no size limits, progress metrics, cancellation controls, or benchmarks.

### Reliability and availability

- The application is local and has no remote availability dependency.
- Generated names avoid routine collisions.
- A failed restore can leave a clearly reported incomplete extraction folder.
- Release builds use `panic = "abort"`; unexpected panics terminate the process.

### Maintainability

- The code is divided by responsibility: GUI, workflow, scanning/serialization,
  parsing/restoration, path safety, and compression.
- Cargo locks the full dependency graph.
- CI runs formatting, Clippy with warnings denied, and tests on Ubuntu and Windows.

### Accessibility

- VERIFIED: advanced text shaping and an embedded font support Persian/RTL input.
- NOT VERIFIED: screen-reader semantics, full keyboard navigation, focus order,
  contrast conformance, and display scaling have no dedicated tests.

### Observability

- User-visible notices report completion, warnings, and errors.
- There is no log file, structured logging, telemetry, metrics, tracing, or crash reporting.

## Technology and deployment constraints

- Rust edition 2024; minimum declared toolchain 1.88.
- Iced uses `tiny-skia`, X11, advanced shaping, and a thread-pool executor.
- The x86_64 MSVC configuration statically links the C runtime.
- Windows resources are compiled from the PNG icon during the build.
- Crates.io publication is disabled with `publish = false`.
- No environment variables are required at application runtime.

## Assumptions

- **INFERRED:** The user controls and trusts the selected local project, bundle,
  and modification files.
- **INFERRED:** The user manually transports generated prompt content to and
  from an AI tool.
- **VERIFIED:** Replacement line numbers and delete/rename sources refer to the
  original, unmodified bundle tree.
- **UNKNOWN:** Maximum expected project size and acceptable operation duration.

## Known limitations

- Binary data and unsupported encodings cannot be restored.
- UTF-16 source files are converted to UTF-8 content in the bundle; original
  encoding bytes are not preserved.
- Empty directories are omitted.
- Compression is lexical rather than parser-backed and is limited to a fixed
  extension list.
- Line replacement records operate only on existing original files; newly added
  files receive their complete content from the add record.
- File operations do not preserve or directly manipulate empty directories or skipped binary files.
- Read errors skip files with warnings, so a successful bundle may be incomplete.
- There is no operation cancellation or per-file progress.
- The Windows executable is not Authenticode-signed.
