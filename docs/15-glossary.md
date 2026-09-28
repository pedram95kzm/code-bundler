# Glossary

| Term | Meaning in this project |
| --- | --- |
| Bundle | A UTF-8 text file containing framed relative paths and decoded file contents. Generated filenames begin with `extracted_content_`. |
| Bundle entry | One relative path plus its exact UTF-8 byte-length-framed content. |
| Byte-length framing | Recording content size in UTF-8 bytes so delimiters inside source content do not determine where an entry ends. |
| Code Bundler v1 | The only accepted bundle format, identified by `==code-bundler:v1==`. |
| Compression | Optional, extension-aware whitespace reduction before bundling. Recognized comments and literals are preserved; it is not archive compression. |
| Extract | The user-facing action that restores a bundle into a new directory. Internally this is primarily handled by `embedder.rs`. |
| Extractor | Internal module that scans a source project and serializes a bundle; the name does not mean the UI's Extract action. |
| Embedder | Internal module that parses a bundle and writes a restored directory; it implements most of the UI's Extract action. |
| Generate | The user-facing action that produces a bundle and a generated prompt from a project directory. |
| Generated prompt | A text file based on `sample_prompt.txt`, containing the user's request and the bundle location/instructions. |
| Ignore rule | A project-local `.gitignore` rule used to omit paths during scanning. |
| Added-file record | `add_file:<path>,new_content:<first line>` plus optional continuation lines; creates a complete new text entry. |
| Delete-file record | `delete_file:<original path>`; omits one original stored text file from restored output. |
| Modification file | Optional UTF-8/UTF-16 text containing line replacements and file add/delete/rename records. |
| Modification record | `file:<path>,line:<number>,new_content:<replacement>` instruction applied to an original stored text file during restore. |
| Rename-file record | `rename_file:<original path>,new_path:<new path>`; changes a stored file's final relative path. |
| Non-overwriting output | Policy of allocating a base name or numeric suffix rather than replacing an existing output. |
| Paired output | The generated bundle and prompt, which use the same suffix and are treated as one naming operation. |
| Portable path | A validated relative path that avoids traversal, reserved device names, forbidden/control characters, trailing dot/space components, and collisions across common filesystems. |
| Preferred line ending | The newline style detected from a decoded file and retained when applying line replacements. |
| Restored directory | New sibling directory produced from a bundle, named with `_embedded` and an optional numeric suffix. |
| Skip message | Human-readable bundle content substituted for binary/NUL or unsupported text bytes; it is not the original file data. |
| Strict marker | The exact first-line identifier required to parse a bundle: `==code-bundler:v1==`. |
| VCS directory | Version-control metadata directory pruned by the scanner: `.git`, `.hg`, `.svn`, or `.jj`. |
