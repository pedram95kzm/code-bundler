# Code Bundler

A small command-line tool that turns a folder of source files into one portable
text document and can reconstruct the folder later.

Code Bundler supports two operations:

- **Extract:** recursively combine a folder's text files into one text file.
- **Embed:** recreate the files and folders described by an extracted text file.

## Getting started

Code Bundler requires [Rust](https://www.rust-lang.org/tools/install) 1.85 or
newer. After cloning the repository, run it directly with Cargo:

```console
cargo run --release
```

To install the `code-bundler` command locally instead:

```console
cargo install --path .
code-bundler
```

On Windows, you can also build a portable executable and open it directly:

```powershell
cargo build --release
.\target\release\code-bundler.exe
```

The Windows release build statically links the C runtime on 64-bit MSVC
targets.

## Usage

1. Run `code-bundler` from a terminal (or double-click the Windows executable).
2. Enter a folder path to **extract**, or enter an extracted `.txt` path to
   **embed**. Press **Tab** to see matching paths. Quoted paths are accepted.
3. When extracting, answer `y` or `yes` to compress the extracted copy. Answer
   `n`, `no`, or press Enter to keep the original formatting.

An extract is written as `extracted_contents.txt` inside the selected folder.
If that filename already exists, the program creates `extracted_contents_2.txt`,
then `_3`, and so on.

Embedding `extracted_contents.txt` creates `extracted_contents_embedded` next
to it. Existing folders are never overwritten; `_2`, `_3`, and so on are used
when needed. Parent folders in file paths are created automatically.

Files are ordered by their relative paths. New extracts include byte counts so
embedding can preserve empty files, final newlines, and content that resembles
a file header:

```text
==code-bundler:v2==
==1.php content==
==utf8-bytes:18==
contents of 1.php
==src/2.php content==
==utf8-bytes:22==
contents of src/2.php
```

Extracts made by version 1 are also accepted. That older format did not record
content lengths, so final-newline information can sometimes be ambiguous.
Extracts containing the previous `==code-extractor:v2==` marker remain fully
supported after the package rename.

Obvious binary files are listed with a skip marker and cannot be recreated,
because their data is not stored in the extract. Empty folders are not recorded.
Symbolic links are not followed, preventing directory loops. During embedding,
absolute paths and paths containing `..` are rejected so content cannot escape
the new output folder.

During extraction, `.gitignore` files in the selected folder and its nested
folders are honored with standard Git matching rules, including `!` negations.
Rules from parent folders, global Git configuration, `.git/info/exclude`, and
`.ignore` files are not applied. Hidden files remain included unless a
`.gitignore` rule excludes them.

When selected, compression only changes the combined output file; source files
are always opened read-only and are never modified. The safe compactor preserves
strings, escape sequences, comments, line-comment endings, Rust-style raw
strings, and heredoc/nowdoc content. Python uses a dedicated mode that preserves
indentation and required newlines while reducing safe intra-line whitespace and
blank lines. Unknown or whitespace-sensitive formats are kept unchanged rather
than risk breaking them.

## Project layout

- `src/main.rs` - minimal executable entry point
- `src/app.rs` - Extract/Embed application flow and status output
- `src/prompt.rs` - terminal input, compression choice, and Tab path completion
- `src/extractor.rs` - recursive scanning and combined-file writing
- `src/embedder.rs` - extracted-text parsing and file/folder creation
- `src/compression/` - language routing plus separate brace-language and Python compactors

## Development

Format, lint, and test the project before submitting a change:

```console
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The same checks run on Linux and Windows for every GitHub pull request and push.
