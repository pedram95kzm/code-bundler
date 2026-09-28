# Code Bundler

A small desktop application that embeds a folder of source files into one
portable text document and can extract the files and folders later. The GUI is
fully centered and supports shaped right-to-left Persian text in every input.

Code Bundler supports two operations:

- **Embed:** recursively combine a folder's text files into one text file,
  generate an AI-ready prompt from `sample_prompt.txt`, and optionally apply
  safe compression.
- **Extract:** recreate the files and folders described by a bundled text file,
  either raw or with validated line replacements and file add/delete/rename operations.

## Getting started

Code Bundler requires [Rust](https://www.rust-lang.org/tools/install) 1.88 or
newer. After cloning the repository, run it directly with Cargo:

```console
cargo run --release
```

On Windows, you can also build a portable executable and open it directly:

```powershell
cargo build --release
.\target\release\code-bundler.exe
```

The Windows release build statically links the C runtime on 64-bit MSVC
targets.

## Usage

1. Open Code Bundler and choose **Embed** or **Extract**.
2. For **Embed**, choose the project folder, optionally enter an AI request,
   optionally enable compression, and select **Generate**.
3. For **Extract**, choose the bundled TXT file and optionally choose a
   modification file, then select **Extract files**.

Embed creates two files inside the selected project folder. For a folder named
`my-project`, they are:

- `extracted_content_my-project.txt` - the bundled repository
- `generated_prompt_my-project.txt` - the filled prompt containing the bundle
  and optional request

The prompt structure comes from [`sample_prompt.txt`](sample_prompt.txt), which
is embedded into the application at build time. Existing output files are never
overwritten; both new files receive the same `_2`, `_3`, and subsequent suffix.
Previous generated bundles and prompts are excluded from later bundles.

Extracting a bundle creates a folder next to the selected TXT file. Existing
folders are never overwritten; `_2`, `_3`, and subsequent suffixes are used
when needed. Parent folders in bundled file paths are created automatically.

### Change files

A modification file can replace existing lines and add, delete, or rename files:

```text
file:src/OldService.php,line:20,new_content:return "updated";
rename_file:src/OldService.php,new_path:src/Service.php
delete_file:src/Unused.php
add_file:src/NewHelper.php,new_content:<?php
function helper(): string
{
    return "ready";
}
```

Paths are relative to the embedded folder and must pass the same portable-path
validation as bundled files. Line numbers start at 1 and always refer to the
original bundled file. To use multiline replacement or new-file content, put
its continuation lines directly below the record. The next complete change
record starts the next operation:

```text
file:1.php,line:200,new_content:if ($ready) {
    echo "2";
    echo "This replacement uses more than one line";
}
rename_file:1.php,new_path:src/1.php
```

If a continuation line should literally begin with `file:`, `add_file:`,
`delete_file:`, or `rename_file:`, prefix it with `\`. If its real content
begins with `\`, write `\\`; the first backslash is the continuation escape.

All line numbers refer to the original, unmodified file. Replacements are
validated first and applied from the bottom of each file upward, so a multiline
replacement cannot shift another target. File operations are then evaluated as
one final tree: sources refer to original bundle paths, parent directories for
new/renamed files are created automatically, and final path collisions are
rejected. A file may be edited and renamed, but it cannot be edited and deleted.
Duplicate targets, missing sources, unsafe paths, and conflicting final paths
are rejected before an output folder is created. See
[`modification_file.txt`](modification_file.txt) for a complete example.

Files are ordered by their relative paths. New bundles include byte counts so
extraction can preserve empty files, final newlines, and content that resembles
a file header:

```text
==code-bundler:v1==
==1.php content==
==utf8-bytes:18==
contents of 1.php
==src/2.php content==
==utf8-bytes:22==
contents of src/2.php
```

Only the versioned Code Bundler v1 format is accepted. Unknown or unversioned
inputs are rejected instead of being interpreted using historical heuristics.

Binary files and files that are not valid UTF-8 or UTF-16 text are listed with
a skip marker and cannot be recreated, because their data is not stored in the
bundle. Empty folders are not recorded. Symbolic links are not followed,
preventing directory loops. File names must be valid Unicode and portable;
unsafe paths, reserved device names, case-only collisions, absolute paths, and
paths containing `..` are rejected before files are written.

During embedding, `.gitignore` files in the selected folder and its nested
folders are honored with standard Git matching rules, including `!` negations.
Rules from parent folders, global Git configuration, `.git/info/exclude`, and
`.ignore` files are not applied. Hidden files remain included unless a
`.gitignore` rule excludes them, but version-control metadata directories such
as `.git`, `.hg`, `.svn`, and `.jj` are always excluded.

When selected, compression during embedding only changes the combined output
file; source files are always opened read-only and are never modified. The safe
compactor preserves strings, escape sequences, comments, line-comment endings,
Rust-style raw strings, and heredoc/nowdoc content. Python uses a dedicated mode
that preserves indentation and required newlines while reducing safe intra-line
whitespace and blank lines. Unknown or whitespace-sensitive formats are kept
unchanged rather than risk breaking them.

## Project layout

- `src/main.rs` - desktop window setup and embedded application icon
- `src/app.rs` - responsive Embed/Extract GUI and background jobs
- `src/workflow.rs` - paired bundle/prompt generation and output naming
- `src/extractor.rs` - recursive scanning and combined-file writing
- `src/embedder.rs` - bundled-text parsing, content/file changes, and safe file creation
- `src/compression/` - language routing plus separate brace-language and Python compactors
- `assets/` - application artwork and the embedded Vazirmatn Persian font
- `sample_prompt.txt` - template used to generate AI-ready prompts

## Development

Format, lint, and test the project before submitting a change:

```console
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The same checks run on Linux and Windows for every GitHub pull request and push.

## License

This project is available under the [MIT License](LICENSE).

The bundled Vazirmatn font is available under the SIL Open Font License 1.1;
its notice is included at [`assets/fonts/OFL.txt`](assets/fonts/OFL.txt).
