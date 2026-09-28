# Features

## 1. Desktop workflow and file selection

| Attribute | Current behavior |
| --- | --- |
| Purpose | Provide a single-window interface for generation and restoration. |
| User | Local desktop user. |
| Trigger | Launch `code-bundler`; choose Embed or Extract. |
| Preconditions | A graphical environment supported by Iced and native dialogs. |
| Main flow | Inputs update `CodeBundlerApp`; browse actions use `rfd`; action buttons dispatch background tasks. |
| Alternative flow | Paths may be typed; one matching pair of surrounding double quotes is removed. |
| Failure cases | Missing/incorrect paths produce an error notice. Dialog cancellation leaves state unchanged. |
| Inputs | Folder path, bundle path, modification path, request, compression toggle. |
| Outputs | Working, completion, warning, or error notice. |
| Permissions | Current operating-system user's filesystem permissions. |
| Source | `src/main.rs`, `src/app.rs` |
| Acceptance behavior | Controls are disabled while busy; the state becomes editable again after `Finished`. |

## 2. Project bundle generation

| Attribute | Current behavior |
| --- | --- |
| Purpose | Serialize project text into one strict v1 document. |
| Trigger | **Generate** in Embed mode. |
| Preconditions | Selected path is a readable directory with portable file names. |
| Main flow | Discover files, validate paths, sort case-insensitively, read sequentially, optionally compress, and write framed entries. |
| Alternative flows | Binary/invalid text is represented by a skip marker; source read failures become warnings. |
| Failure cases | Unreadable root, non-portable/colliding paths, output collision race, or output write failure. |
| Inputs | Directory and compression boolean. |
| Outputs | `extracted_content_<project>.txt` plus counts and warnings. |
| Dependencies | `ignore`, filesystem, `src/paths.rs`, `src/compression/`. |
| Source | `src/extractor.rs`, `src/workflow.rs` |
| Acceptance behavior | Source files are opened read-only; existing outputs are not overwritten. |

Discovery semantics:

- Root and nested `.gitignore` rules apply, including negations.
- Parent, global, `.git/info/exclude`, and `.ignore` rules do not apply.
- Hidden entries are included unless ignored.
- `.git`, `.hg`, `.svn`, and `.jj` entries are always pruned.
- Symbolic links are not followed.
- Generated bundle/prompt prefixes are excluded only at the selected root.

## 3. AI prompt generation

| Attribute | Current behavior |
| --- | --- |
| Purpose | Combine the repository bundle and optional request with strict content/file change instructions. |
| Trigger | Automatically follows successful bundle creation. |
| Preconditions | Both placeholders exist in the compile-time template. |
| Main flow | Stream template prefix, bundle, template middle, trimmed request, and template suffix into a new prompt file. |
| Failure cases | Missing template placeholder, bundle reopen failure, prompt create/write failure. |
| Inputs | Bundle, `sample_prompt.txt`, request. |
| Outputs | `generated_prompt_<project>.txt`. |
| Dependencies | Local buffered filesystem I/O only. |
| Source | `src/workflow.rs`, `sample_prompt.txt` |
| Acceptance behavior | Bundle and prompt share a suffix; failed prompt creation removes the generated bundle when possible. |

## 4. Optional source compression

| Attribute | Current behavior |
| --- | --- |
| Purpose | Reduce bundle size while attempting to preserve lexical meaning. |
| Trigger | User enables **Compress source code safely**. |
| Preconditions | File is decoded text. |
| Main flow | Route by lowercase extension to Python, brace-style, or unchanged handling. |
| Failure cases | No explicit error path; this is a custom lexical transformation rather than a compiler/parser validation. |
| Inputs | File path and decoded text. |
| Outputs | Transformed bundle content; original file is unchanged. |
| Source | `src/compression/mod.rs`, `brace.rs`, `python.rs` |

Python routing applies to `py`, `pyi`, and `pyw`. Brace-style routing applies to
PHP variants, C/C++ headers and sources, C#, Java, Kotlin, Rust, CSS-family
files, and JSON. All other extensions remain unchanged.

## 5. Strict bundle restoration

| Attribute | Current behavior |
| --- | --- |
| Purpose | Recreate stored text files beneath a new output directory. |
| Trigger | **Extract files** in Extract mode. |
| Preconditions | Selected file decodes as UTF-8/BOM UTF-16 and begins `==code-bundler:v1==`. |
| Main flow | Parse and validate all entries, optionally modify them, create a unique sibling directory, then write files sequentially. |
| Alternative flow | Skip binary-marker entries. |
| Failure cases | Unsupported marker/encoding, malformed lengths, unsafe or duplicate paths, invalid change set, or filesystem write failure. |
| Inputs | Bundle and optional modification file. |
| Outputs | `<bundle-stem>_embedded`, then `_2`, `_3`, and so on. |
| Source | `src/embedder.rs`, `src/paths.rs` |
| Acceptance behavior | No output directory exists until bundle and complete change-set validation succeed. |

## 6. Content and file changes

| Attribute | Current behavior |
| --- | --- |
| Purpose | Apply structured line and file-tree changes without interpreting free-form AI prose. |
| Replace record | `file:<path>,line:<1-based number>,new_content:<first replacement line>` |
| Add record | `add_file:<new path>,new_content:<first content line>` |
| Delete record | `delete_file:<original path>` |
| Rename record | `rename_file:<original path>,new_path:<new path>` |
| Continuation | Subsequent non-record lines belong to the current replacement or added file. |
| Escaping | A continuation line beginning with a record prefix uses one leading `\`; a real leading `\` is doubled. |
| Preconditions | Replace/delete/rename sources exist as stored text entries in the original bundle; replacement lines exist. |
| Main flow | Parse all records, validate sources and duplicate operations, apply line changes bottom-up, form the final file tree, then validate all final paths. |
| Failure cases | Unsafe path, malformed field, missing source/line, duplicate target/source, edit-plus-delete conflict, or final path/file-parent collision. |
| Outputs | Changed in-memory entries written by the restore flow. |
| Source | `src/embedder.rs`, `sample_prompt.txt`, `modification_file.txt` |
| Acceptance behavior | All sources and line numbers reference the original bundle; no filesystem output begins unless the final tree is valid. |

The original target line ending remains after the replacement. Physical
continuation lines in replacement text are normalized to the target file's
selected line ending.

An existing file can be line-edited and renamed in the same change set by using
its original path. A deleted file cannot also be line-edited or renamed. Added
files take their complete UTF-8 content from the add record and cannot be
line-edited in that same change set. Rename destinations may move files into
new parent directories, which are created during output.

## 7. Portable path and collision policy

| Attribute | Current behavior |
| --- | --- |
| Purpose | Ensure bundled, added, and renamed entries cannot escape the output or become ambiguous on common filesystems. |
| Rejected | Empty, `.`/`..`, absolute-like empty components, trailing dot/space, control characters, `< > : " \\ | ? *`, and reserved Windows device basenames. |
| Collision handling | Full paths are lowercased for duplicate and file/parent conflict checks. |
| Source | `src/paths.rs`, validation in `src/extractor.rs` and `src/embedder.rs` |
| Acceptance behavior | Generation/restoration fails before creating the primary output when path validation fails. |

## 8. Release presentation

- The icon is embedded in the window and converted to a multi-size Windows ICO at build time.
- Vazirmatn Regular and Bold are embedded into the executable.
- Windows release mode suppresses the console subsystem window.
- The current packaged release contains the executable, README, MIT license,
  font license, ZIP, and SHA-256 checksums under `dist/`.
