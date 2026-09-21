# Code Bundler

[![CI](https://github.com/pedram95kzm/code-bundler/actions/workflows/ci.yml/badge.svg)](https://github.com/pedram95kzm/code-bundler/actions/workflows/ci.yml)

A small command-line tool that embeds a folder of source files into one portable
text document and can extract the files and folders later.

Code Bundler supports two operations:

- **Embed:** recursively combine a folder's text files into one text file, with
  optional safe compression.
- **Extract:** recreate the files and folders described by a bundled text file,
  either raw or with line-based modifications.

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
2. Choose `embed` or `extract`.
3. For **embed**, choose whether to compress, then enter the folder path.
4. For **extract**, choose `raw` or `with_modification`. Raw extraction asks only
   for the bundled `.txt` file. Modified extraction asks for the modification
   file first, then the bundled `.txt` file.

Press **Tab** to autocomplete fixed answers and paths. Quoted paths are accepted.
After success or failure, the program waits for Enter so the terminal remains
open and the result can be read.

A bundle is written as `extracted_contents.txt` inside the selected folder.
If that filename already exists, the program creates `extracted_contents_2.txt`,
then `_3`, and so on.

Extracting `extracted_contents.txt` creates `extracted_contents_embedded` next
to it. Existing folders are never overwritten; `_2`, `_3`, and so on are used
when needed. Parent folders in file paths are created automatically.

### Modification files

Each modification replaces one original line in one bundled file:

```text
file:1.php,line:100,new_content:echo "1";
file:1.php,line:200,new_content:echo "2";
file:2.php,line:300,new_content:echo "";
```

Paths are relative to the embedded folder and line numbers start at 1. To use
multiline replacement content, put its continuation lines directly below the
record. The next line beginning with a complete `file:...,line:...,new_content:`
record starts the next modification:

```text
file:1.php,line:200,new_content:if ($ready) {
    echo "2";
    echo "This replacement uses more than one line";
}
file:2.php,line:300,new_content:echo "";
```

All line numbers refer to the original, unmodified file. Replacements are
validated first and applied from the bottom of each file upward, so a multiline
replacement cannot shift the target of another modification. Duplicate targets,
missing files, and out-of-range line numbers are rejected before an output
folder is created. See [`modification_file.txt`](modification_file.txt) for a
complete example.

Files are ordered by their relative paths. New bundles include byte counts so
extraction can preserve empty files, final newlines, and content that resembles
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

Bundles made by version 1 are also accepted. That older format did not record
content lengths, so final-newline information can sometimes be ambiguous.
Bundles containing the previous `==code-extractor:v2==` marker remain fully
supported after the package rename.

Obvious binary files are listed with a skip marker and cannot be recreated,
because their data is not stored in the extract. Empty folders are not recorded.
Symbolic links are not followed, preventing directory loops. During extraction,
absolute paths and paths containing `..` are rejected so content cannot escape
the new output folder.

During embedding, `.gitignore` files in the selected folder and its nested
folders are honored with standard Git matching rules, including `!` negations.
Rules from parent folders, global Git configuration, `.git/info/exclude`, and
`.ignore` files are not applied. Hidden files remain included unless a
`.gitignore` rule excludes them.

When selected, compression during embedding only changes the combined output
file; source files are always opened read-only and are never modified. The safe
compactor preserves strings, escape sequences, comments, line-comment endings,
Rust-style raw strings, and heredoc/nowdoc content. Python uses a dedicated mode
that preserves indentation and required newlines while reducing safe intra-line
whitespace and blank lines. Unknown or whitespace-sensitive formats are kept
unchanged rather than risk breaking them.

## Project layout

- `src/main.rs` - minimal executable entry point
- `src/app.rs` - Embed/Extract application flow and status output
- `src/prompt.rs` - operation/mode choices and Tab path completion
- `src/extractor.rs` - recursive scanning and combined-file writing
- `src/embedder.rs` - bundled-text parsing, modifications, and file creation
- `src/compression/` - language routing plus separate brace-language and Python compactors

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
