# Reconstructed Product Requirements Document

## Status

This PRD is reconstructed from the v1.0.0 implementation. No original PRD is
present, and historical product priorities are **UNKNOWN**.

## Product summary

Code Bundler is a local desktop application for packaging project text into an
AI-ready document and safely restoring that document, optionally with structured
line replacements and file-tree changes. It deliberately avoids a backend,
account system, and AI API.

## Product goals

1. Make a multi-file source project usable in a single-text workflow.
2. Generate consistent AI instructions tied to exact original file lines and paths.
3. Restore accepted content without escaping the output directory or overwriting existing work.
4. Keep normal operation local, portable, and credential-free.
5. Support Persian and other shaped text in the GUI.

No analytics, adoption targets, latency target, or commercial success metric is defined.

## User personas

| Persona | Evidence status | Need |
| --- | --- | --- |
| AI-assisted developer | INFERRED | Package code and a change request, then apply structured returned edits. |
| Developer transporting a text snapshot | INFERRED | Restore a project tree from one versioned text file. |
| Persian/RTL user | INFERRED | Enter paths and requests with correct text shaping. |

## Core journeys

### Journey A — generate AI input

1. Open the application in Embed mode.
2. Choose a project folder.
3. Optionally enter a change request and enable compression.
4. Select **Generate**.
5. Receive paths to a v1 bundle and generated prompt.
6. Review the prompt and manually submit it to the user's chosen AI tool.

### Journey B — restore without modifications

1. Switch to Extract mode.
2. Choose a strict v1 bundle.
3. Leave the modification path empty.
4. Select **Extract files**.
5. Receive a new sibling output folder containing stored text files.

### Journey C — restore with a change set

1. Choose a strict v1 bundle and a modification file.
2. Start extraction.
3. The application validates the full bundle, every replacement target, and
   every add/delete/rename operation.
4. Replacements are applied against original line numbers; structural operations
   are evaluated against original paths to form one conflict-free final tree.
5. A new sibling folder is created and the final tree is written.

## Implemented feature set

| Feature | Current role |
| --- | --- |
| Desktop mode-based GUI | Primary interaction surface |
| Recursive project scanning | Builds the source inventory |
| Strict v1 bundle format | Stable interchange contract for this release |
| Prompt templating | Creates AI instructions without network access |
| Optional lexical compression | Reduces selected source text |
| Safe restore | Recreates stored text beneath a new directory |
| Change records | Apply existing-line replacements and add/delete/rename files |
| Portable path policy | Prevents traversal and cross-platform naming conflicts |
| Non-overwriting naming | Protects existing user outputs |
| Warning/error notices | Communicates skipped inputs and failures |

Feature priority was not recorded. The implementation and README present Embed
and Extract as equally core v1 functionality.

## Acceptance behavior

- A valid project produces a bundle and prompt with a matching numeric suffix.
- Re-running generation does not rebundle prior root-level generated outputs.
- A valid uncompressed text round trip preserves exact stored contents.
- An invalid marker, unsafe path, missing operation source, duplicate final path,
  file/folder conflict, or bad line replacement aborts before an extraction
  directory is created.
- An existing output name is never overwritten.
- A skipped binary marker does not produce a restored file.
- Busy state prevents concurrent user-triggered operations.

## Out of scope

- Direct AI submission or provider configuration.
- Source-control operations.
- In-place patching or arbitrary diff application to the selected source project.
- Creating or modifying binary files through change records.
- Binary and empty-directory backup.
- Multi-user collaboration, cloud synchronization, or project history.
- Automatic release publishing or updating.

## Future functionality

**UNKNOWN.** No roadmap, backlog, or explicit future feature is present.
