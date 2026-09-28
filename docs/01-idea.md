# Idea

## Observed product

**Project:** Code Bundler
**Evidence status:** VERIFIED from the implementation; original product brief is UNKNOWN.

Code Bundler is a local desktop utility that converts a source-code folder into
a single, human-readable text bundle and an AI-oriented prompt. It can later
restore the bundle into a new folder, optionally applying structured line
replacements and file add/delete/rename operations supplied in a separate text file.

## Problem being solved

Source repositories are inconvenient to paste into text-only tools, and AI
responses are inconvenient to apply safely when they are returned as prose or
unstructured snippets. Code Bundler provides two explicit file contracts:

1. A length-framed repository bundle containing portable relative paths and text.
2. A change-record format for original-line replacement plus file addition,
   deletion, and rename/move operations.

The application does not contact an AI service. Users transfer the generated
prompt and returned change text manually.

## Target users

- **INFERRED — developers using AI-assisted editing:** generate a prompt containing repository context and a requested change.
- **INFERRED — developers moving text snapshots:** restore a bundle without overwriting an existing output directory.
- **INFERRED — Persian-speaking or RTL users:** the GUI embeds Vazirmatn and enables advanced text shaping.

No market segment, commercial model, telemetry goal, or formal persona is
recorded in the repository.

## Value proposition

- Local and credential-free operation.
- One portable text representation for many source files.
- Exact preservation of supported text content when compression is disabled.
- Deterministic, validated content and file-tree changes.
- Non-overwriting outputs and portable path validation.

## Major use cases

1. Select a project, optionally describe a requested change, and generate a bundle plus prompt.
2. Generate a smaller bundle by enabling source-aware whitespace compression.
3. Restore a v1 bundle into a newly created sibling folder.
4. Restore a bundle while applying AI-produced line replacements and file operations.

## Scope

- Desktop GUI with Embed and Extract modes.
- Recursive text-file discovery with project-local `.gitignore` handling.
- Strict Code Bundler v1 serialization and parsing.
- Optional lexical compression for selected extensions.
- Local prompt generation from a compile-time template.
- Safe creation of new files and directories.
- Windows release packaging; the source is also compiled and tested on Ubuntu in CI.

## Out of scope

- Network calls or direct AI-provider integration.
- Authentication, accounts, collaboration, or cloud storage.
- Binary-file archiving.
- Empty-directory preservation.
- In-place project modification or overwrite.
- General-purpose compression or cryptographic archive protection.
- Project indexing, search, semantic analysis, or language-server behavior.

## Constraints and assumptions

- Rust 1.88 or newer and edition 2024.
- GUI behavior depends on Iced 0.14 and native window/file-dialog support.
- Bundle paths use `/` and must pass cross-platform portability checks.
- Source text must be valid UTF-8 or BOM-marked UTF-16; unsupported data is represented only by a skip marker.
- Users are expected to inspect generated prompts for sensitive content before sharing them.
- Line replacements and file-operation sources refer to the original bundle;
  their paths and line numbers must remain stable while the change set is produced.
