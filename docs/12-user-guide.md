# User Guide

## What Code Bundler does

Code Bundler converts a source project into one structured text bundle plus an
AI prompt. It can later reconstruct that bundle as a new directory and,
optionally, apply line replacements and add, delete, or rename files from a
modification file.

All processing is local. The application does not upload a project or contact
an AI service; you manually choose where to use the generated prompt and
bundle.

## Start the application

For the packaged Windows release, extract the ZIP and run `code-bundler.exe`.
There is no installer and no account or configuration setup.

The interface labels are English. Its embedded font and text shaping support
entering Persian and other right-to-left text in input fields.

## Generate a bundle and optional prompt

1. Choose or enter the source project directory.
2. Optionally enter a request describing the work you want performed.
3. Decide whether to enable source compression.
4. Select **Generate**.
5. Wait for the completion status, then use the output shortcuts to open a
   generated file or its folder in the system file explorer.

The bundle is always produced:

- `extracted_content_<project>.txt`: the structured source bundle.

When the request contains non-whitespace text, this file is also produced:

- `generated_prompt_<project>.txt`: instructions containing the bundle and
  your request.

An empty request does not create a generated prompt. If either required name
already exists, the new output receives a numeric suffix. No existing file is
intentionally overwritten.

### Which project files are included

- `.gitignore` rules in the project and nested directories are honored.
- `.git`, `.hg`, `.svn`, and `.jj` metadata directories are omitted.
- Hidden files are included unless ignored by a project rule.
- Symbolic links are not followed.
- Previously generated root-level bundle/prompt outputs are excluded.
- Binary or unsupported text data is represented by a skip message.

Review the bundle before sharing it. Hidden files can contain credentials or
other private data.

### Compression

Compression is off by default. When enabled, Code Bundler reduces whitespace
for supported source extensions while preserving comments, literals, and other
language constructs it recognizes. Unsupported extensions are kept unchanged.

Use compression only when a smaller prompt input is worth reduced readability.
Keep the uncompressed project as the source of truth.

## Use the generated files with an AI tool

The application does not submit anything automatically. Open the generated
prompt, follow its instructions in the AI tool you choose, and provide the
bundle only after reviewing its contents.

If changes are requested, the prompt can produce four record types:

```text
file:src/example.rs,line:12,new_content:replacement text
add_file:src/new.rs,new_content:first content line
delete_file:src/unused.rs
rename_file:src/old.rs,new_path:src/new-name.rs
```

The AI tool and its privacy, context-size, and retention policies are outside
Code Bundler.

## Restore a bundle

1. Choose the v1 bundle file.
2. Optionally choose a modification file.
3. Select **Extract**.
4. Wait for the status message and open the reported output directory.

The restored directory is created beside the bundle as
`<bundle-name>_embedded`. If that directory exists, a numeric suffix is used.
Existing directories are not overwritten.

### Modification file rules

- `file:` replaces one existing original line.
- `add_file:` creates a new text file; its `new_content:` field and continuation
  lines are the complete file contents. An empty field creates an empty file.
- `delete_file:` removes one original stored text file from restored output.
- `rename_file:` changes a file's path and can move it into a new directory.
- Paths are relative and must satisfy Code Bundler's portable path rules.
- Replacement line numbers are one-based and refer to original file contents.
- Put multiline replacement or added-file continuation lines directly below
  the record; the next complete record starts the next operation.
- Prefix continuation content that begins with a record prefix with `\`. Write
  `\\` when the real content begins with `\`; the first backslash is removed.
- The same file/line target cannot appear twice, and one original source cannot
  have more than one delete/rename operation.
- A file can be line-edited and renamed by using its original path. It cannot be
  line-edited and deleted.
- All final paths must be case-insensitively unique and free of file/parent conflicts.
- An empty modification path means restore without changes.

All instructions are checked before output creation. A malformed or unsafe
instruction rejects the operation.

## Errors and recovery

- A rejected project path must name an existing directory.
- A rejected bundle may have the wrong marker, damaged byte lengths, unsafe
  paths, duplicates, or path conflicts.
- A rejected modification may use invalid syntax, reference a missing original
  source or line, combine incompatible operations, or produce conflicting final paths.
- A write failure usually indicates permissions, unavailable storage, or disk space.
- A late write failure can leave a folder explicitly reported as incomplete;
  inspect it before deleting it and retry into a new output name.

See [10-troubleshooting.md](10-troubleshooting.md) for detailed checks.

## Limitations

- Only the strict Code Bundler v1 text format is accepted for restoration.
- Binary files are not preserved as binary content.
- There is no progress bar, cancellation, pause, or resume.
- Large projects are processed in memory and have no configured size limit.
- No backup, synchronization, AI connection, or version control operation is performed.
- The application has no automatic updates, telemetry, or persistent logs.
- Output executables are currently unsigned.

## FAQ

### Does Code Bundler send my code anywhere?

No. The implemented application only reads and writes local files. You decide
whether to share the output elsewhere.

### Will it overwrite my project?

No. Generate creates text files inside the selected project, and Extract
creates a new directory beside the bundle. It allocates suffixes when names
already exist.

### Does `.gitignore` guarantee secrets are excluded?

No. It only applies rules present in the project. Hidden, unignored files can
still be included. Always inspect output before sharing.

### Can I edit the bundle by hand?

Ordinary edits can invalidate its strict UTF-8 byte-length framing. Use a
modification file for line replacements and file operations, or regenerate a
bundle after changing the source project.

### Do I need the original project to extract a bundle?

No. Restoration uses the bundle and optional modification file. Files skipped
during generation cannot be recovered from the bundle.
