# User Guide

## Start and settings

Extract the Windows release archive and run `code-bundler.exe`, or follow
[DEVELOPMENT.md](DEVELOPMENT.md) to run from source. There is no installer,
account, or application configuration file. Paths, request text, and compression
are session settings; compression starts off. Native dialogs or typed paths are
accepted. Matching surrounding double quotes are removed from pasted paths;
single quotes are literal. English labels support Persian/RTL text input.

## Generate

1. Choose **Embed** and select the source project folder.
2. Optionally enter a change request and enable compression.
3. Select **Generate** and use the completion shortcuts to locate the outputs.

All outputs are stored in `code_bundler` inside your Documents folder:

| Output | When created |
| --- | --- |
| `extracted_content_<project>.txt` | Every successful generation. |
| `generated_prompt_<project>.txt` | Only for a non-empty trimmed request. |
| `<bundle-stem>_embedded/` | When restoring a bundle. |

Existing names receive `_2`, `_3`, and later suffixes. A bundle and its prompt
share a suffix. Successful generation clears the request editor; failure keeps
it. Original project files are opened read-only.

Root/nested `.gitignore` rules apply, including `!` negations; parent/global
rules, `.git/info/exclude`, and `.ignore` do not. Hidden files remain included
unless ignored. The scanner omits `.git`, `.hg`, `.svn`, and `.jj`, never follows
symlinks, and excludes the output directory when nested in the project.
Root-level generated bundle/prompt names are also excluded.

Valid UTF-8 and BOM-marked UTF-16 are stored as UTF-8. Binary, UTF-32, and invalid
text become `[Skipped: binary file]` entries; original bytes cannot be restored.
Empty folders and filesystem metadata are omitted. Source read failures are
warnings and can leave a successful bundle incomplete.

Compression reduces whitespace for supported Python and brace-style languages,
retaining recognized literals/comments; unknown extensions remain unchanged.
It is lexical rather than parser-backed. Keep the original project and review
transformed code. The app uploads nothing, but hidden unignored files can
contain secrets: inspect outputs before sharing. Outputs are plain text without
encryption or automatic deletion.

## Restore

Choose **Extract**, select a v1 bundle and optionally a modification file, then
select **Extract files**. An empty modification path restores stored text
unchanged. Output is a new `<bundle-stem>_embedded[_N]` directory under
`Documents/code_bundler`. All logical validation precedes directory creation.

Only strict v1 bundles are accepted. Hand edits can invalidate byte framing;
regenerate a bundle or use a modification file instead.

## Modification files

Use exact, case-sensitive records and portable relative paths:

```text
file:src/example.rs,line:12,new_content:replacement text
add_file:src/new.rs,new_content:first content line
delete_file:src/unused.rs
rename_file:src/old.rs,new_path:src/new-name.rs
```

- `file:` replaces one existing original line; numbers start at 1.
- `add_file:` supplies the entire new file, including continuation lines. An
  empty content field with no continuation creates an empty file.
- Replacement targets and delete/rename sources refer to the original bundle.
  Added files cannot be line-edited in the same change set.
- A file can be edited and renamed using its original path. It cannot be edited
  and deleted, or have multiple delete/rename operations.
- Duplicate line targets, missing sources, unsafe paths, duplicate final paths,
  and file/parent conflicts reject the operation. Valid parent folders are
  created automatically.

Put continuation lines directly below a replacement or add record. The next
complete record starts another operation:

```text
file:src/example.rs,line:12,new_content:if ready {
    run();
}
rename_file:src/example.rs,new_path:src/renamed.rs
```

Prefix record-like continuation content with `\`; double a real leading
backslash. The first continuation escape is removed. Blank lines and `#`
comments before the first record are ignored; after a content record starts,
non-record lines belong to its content. Delete/rename accept no continuation.

Replacements run bottom-up against original lines and retain the target's
line-ending style. Structural operations form one final tree; rename swaps and
vacated destinations are allowed when conflict-free. See
[modification_file.txt](../modification_file.txt) for an example.

## Troubleshooting and limits

| Problem | Check |
| --- | --- |
| Project rejected or files missing | Directory existence, ignore rules, portable paths, and scan warnings. |
| File skipped | Unsupported bytes cannot be recovered from this bundle. |
| Output cannot be created | Documents-folder discovery, permissions, and free space. |
| Bundle rejected | Exact v1 marker, byte framing, and safe unique paths. |
| Modification rejected | Record fields, original sources/lines, escaping, and final-tree conflicts. |
| Incomplete restored folder | A late write failed. Inspect the reported folder and retry; there is no automatic rollback or resume. |
| Dialog/shortcut unavailable | Type the path; confirm outputs exist and the OS file explorer is available. |
| Controls disabled | Wait for the running operation; there is no progress meter or cancellation. |
| Persian rendering incorrect | Record OS, display scaling, and a screenshot; fonts are embedded. |

Include app version, OS, exact status, action/options, and minimal shareable input
in defect reports. No persistent logs or crash uploads exist. There is no input
size limit, and restore loads the full bundle into memory. Build failures are
covered in [DEVELOPMENT.md](DEVELOPMENT.md#build-troubleshooting).
