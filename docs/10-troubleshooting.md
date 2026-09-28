# Troubleshooting

## User-visible messages

The status line in the main window reports completion and errors. The
application has no log file, debug console, telemetry, or crash-report upload.
Preserve the exact status text and the selected paths when reporting a problem.

## Generate problems

### The project path is rejected

Confirm that the path exists and is a directory. A path pasted with one
matching pair of surrounding double quotes is accepted and normalized by the
GUI; single quotes are treated as path characters.

### No files appear in the bundle

Check the project's `.gitignore` files. The scanner honors root and nested
`.gitignore` rules. It also always prunes `.git`, `.hg`, `.svn`, and `.jj`
directories and excludes Code Bundler's own root-level generated outputs.

Hidden files are not excluded merely because they are hidden.

### A file is represented by a skip message

Files containing a NUL byte or bytes that cannot be decoded as supported text
are recorded with a readable skip marker instead of their original bytes.
UTF-16 BOM input is converted to UTF-8. UTF-32 and arbitrary binary files are
not bundled as binary data.

### Compression has little or no effect

Compression is optional and extension-aware. Unsupported file extensions are
left unchanged. The compressor is conservative around strings/comments and is
not a general-purpose archive compressor.

### Output creation fails

Verify that the selected project directory is writable and has free space.
The application writes the bundle and prompt inside the project directory. It
does not overwrite an existing output; it chooses a numeric suffix. If a paired
write fails, it attempts to remove incomplete outputs.

## Extract problems

### The bundle is rejected before files are written

Only bundles starting with `==code-bundler:v1==` are accepted. Recreate the
bundle with this version if the marker or length framing was edited or damaged.

The entire bundle is parsed and validated before restoration. Rejection can
also mean that an entry has an unsafe/non-portable path, duplicate path,
case-insensitive collision, parent/file conflict, or invalid byte length.

### A modification file is rejected

Use one of these exact, case-sensitive record shapes:

```text
file:path/to/file.ext,line:LINE_NUMBER,new_content:REPLACEMENT
add_file:path/to/new-file.ext,new_content:FIRST_CONTENT_LINE
delete_file:path/to/existing-file.ext
rename_file:path/to/existing-file.ext,new_path:path/to/new-name.ext
```

Replacement, delete, and rename sources must name stored text files in the
original bundle. Replacement line numbers are one-based, must exist, and may
not be repeated for the same file. For multiline replacement or added-file
content, put continuation lines directly below the record. The next complete
record begins the next operation.

If continuation content must begin with a change-record prefix, add one leading
`\`. Double a real leading backslash. The parser removes the first continuation
escape before storing the content.

The complete final tree must have portable, case-insensitively unique paths and
no file/parent conflict. A source cannot be both deleted and renamed, and a
deleted file cannot also be line-edited. Use the original path to edit a file
that is renamed in the same change set.

### No modification file is desired

Leave the modification path empty. It is optional.

### The restored folder has an unexpected suffix

This is expected non-overwriting behavior. The application creates
`<bundle-stem>_embedded`; if that exists it tries `_embedded_2`, `_embedded_3`,
and so on.

### An incomplete restored folder remains

The bundle and modification instructions are validated before output starts,
but a later filesystem error can leave a partially written output directory.
The status message reports it as incomplete. Inspect it before removing it;
the application does not automatically retry or resume.

## GUI problems

### File or folder dialogs do not open

Native dialogs are provided by the operating system through `rfd`. Try typing
or pasting the path directly. Confirm the application has permission to reach
the location.

### Persian text renders incorrectly

The Vazirmatn font is embedded and advanced shaping is enabled. If rendering
still fails, record the Windows version, display scaling, and a screenshot.
There is no runtime font configuration.

### The window stops accepting another operation

While work is running, action controls are disabled by design. Wait for a
success or error status. There is no cancellation button or progress meter.

## Build failures

### Cargo refuses `--locked`

The manifest and `Cargo.lock` disagree, or the lockfile is missing. For a
release, investigate the repository change; do not silently regenerate the
lockfile without reviewing dependency changes.

### Rust version error

Install Rust 1.88 or newer. `Cargo.toml` declares 1.88 as the minimum supported
toolchain.

### Windows resource build fails

Confirm that the expected asset files exist and that the MSVC build tools are
installed. `build.rs` reads the application icon source and embeds resources
during the build.

### CI and local results differ

Use the locked commands from [08-testing-strategy.md](08-testing-strategy.md),
check the active toolchain and target, and compare operating systems. CI runs
the validation matrix defined in `.github/workflows/ci.yml`.

## Escalation information

There is no formal support channel in the repository. A useful defect report
includes the app version, operating system, exact status message, action and
options selected, whether the input is sensitive, and minimal reproducible
input when it can be shared safely.
