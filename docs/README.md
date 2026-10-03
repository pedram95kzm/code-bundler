# Code Bundler Documentation

Code Bundler is a local Rust/Iced desktop application for packaging a source
project into text and restoring it with optional structured edits. **Embed /
Generate** creates a bundle and an optional AI prompt; **Extract** restores
stored text into a new directory. Users transfer prompts and change files to
external AI tools manually.

## Reading guide

| File | Contents |
| --- | --- |
| [USER-GUIDE.md](USER-GUIDE.md) | Workflows, settings, change records, limitations, and troubleshooting. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Modules, data flow, bundle format, decoding, compression, and path safety. |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Setup, configuration, checks, CI, testing, and Windows releases. |
| [DECISIONS.md](DECISIONS.md) | The three observed architecture decisions and their tradeoffs. |

The root [README](../README.md) provides the quick start. These four guides and
this overview are the complete documentation set.

## Scope and essential contracts

- Processing is local. The English GUI supports shaped Persian/RTL input.
- Outputs live in `code_bundler` under the user's Documents folder.
- Only `==code-bundler:v1==` is accepted; lengths count UTF-8 bytes.
- Decoded text round-trips exactly when compression is off. Skipped binary data
  and empty directories cannot be recovered from a bundle.
- Portable paths and the complete change set are validated before restore output.
- Outputs use exclusive creation and numeric suffixes instead of overwriting.
- Change records use original paths and one-based original line numbers.

Binary backup, in-place patching, direct AI integration, accounts, cloud storage,
and collaboration are outside the current feature set. No formal roadmap or
product success metrics are recorded.

## Terminology and maintenance

A **bundle** stores framed paths and text; a **prompt** combines that bundle with
a request; a **modification file** specifies replacements, additions, deletions,
and renames. Internally, `extractor.rs` creates bundles and `embedder.rs`
restores them, despite their counterintuitive names.

For implementation work, start with architecture and development. Resolve
conflicting evidence in this order: source code, configuration, tests, then
documentation. Update the relevant guide when behavior changes and the decision
register when a durable contract changes. Historical intent is unknown unless
recorded.
