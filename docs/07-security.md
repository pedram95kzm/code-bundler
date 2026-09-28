# Security

## Security model

Code Bundler is a local desktop process with the privileges of the user who
launches it. It has no authentication, authorization, session, token, cookie,
CORS, CSRF, rate-limit, TLS, or server security layer because it exposes no
network service.

The relevant threats are malicious/malformed local input, accidental data
disclosure through generated prompts, unsafe filesystem paths, output
collisions, and dependency/build-chain compromise.

## Trust boundaries

| Boundary | Trusted? | Handling |
| --- | --- | --- |
| Selected project files | User-controlled | Paths validated; text decoded; read failures warned. |
| Selected bundle | Untrusted structured text | Strict marker, framing, path, duplicate, and hierarchy validation. |
| Modification file | Untrusted structured text | Exact operation fields, positive line numbers, original-source validation, and final-tree validation. |
| Output parent directory | OS-controlled | Files/directories use new-only creation. |
| Generated prompt | Sensitive derived data | Written locally; transfer is outside the app. |
| Dependencies and CI actions | Supply-chain input | Cargo lockfile; CI actions pinned to commits. |

## Implemented controls

### Path traversal and unsafe filenames

- Paths are relative `/`-separated components.
- Empty, dot, parent, control, forbidden-character, trailing-dot/space, and
  Windows device components are rejected.
- Absolute Windows paths fail on `:`; Unix absolute paths fail on an empty first component.
- Case-insensitive duplicate paths and file/parent conflicts are rejected.
- Restored paths are joined only after validation beneath a newly created root.

### Overwrite and data-loss prevention

- Generated files use `create_new`.
- Restore roots use `create_dir` and numeric suffixes.
- Restored files also use `create_new`.
- Original source files are read-only from the application's perspective.
- Incomplete generation artifacts are removed in handled write-failure paths.

### Scanner containment

- Symbolic links are not followed by the directory walker.
- Common version-control metadata entries are pruned at any depth.
- Parent/global ignore configuration is not consulted, keeping selection behavior local to the chosen root.

### Parser robustness

- Checked arithmetic prevents byte-length overflow.
- Content bounds and UTF-8 character boundaries are verified.
- Duplicate modifications to the same line are rejected.
- Delete/rename sources must be stored text entries from the original bundle.
- Added and renamed destinations pass portable-path validation.
- Duplicate structural sources, edit-plus-delete conflicts, final duplicate
  paths, and final file/parent conflicts are rejected.
- The complete change set is checked before an output root is created.

### Network and process execution

- Application runtime contains no HTTP client, socket handling, subprocess
  invocation, shell command construction, or dynamic code evaluation.
- Prompt content is not automatically transmitted.

## Sensitive data considerations

The application does not scan for secrets. Hidden files are included by
default, and files such as `.env`, credentials, private keys, or local
configuration are bundled when they are not excluded by `.gitignore`.

Users must review a generated bundle/prompt before sharing it. VCS metadata is
always excluded, but that is not a substitute for secret detection.

Bundles and prompts are plain text. They are not encrypted, signed, access
controlled, or automatically deleted. Their protection depends on filesystem
permissions and user handling.

## Dependency and build security

- `Cargo.lock` is present and CI uses `--locked` for lint and test.
- GitHub Actions are pinned to full commit hashes; checkout does not persist credentials.
- A RustSec audit performed for the v1 release found no known vulnerabilities.
- The audit reported the transitive `ttf-parser 0.25.1` crate as unmaintained
  (`RUSTSEC-2026-0192`). It enters through Iced's text-shaping stack and is not a direct dependency.
- The Windows executable is not Authenticode-signed; users may receive SmartScreen warnings and cannot verify publisher identity.
- SHA-256 checksums detect accidental or malicious artifact changes only when
  the checksum file itself is obtained from a trusted channel.

## Known security concerns

| Concern | Impact | Current state |
| --- | --- | --- |
| Sensitive project files in prompts | Confidentiality exposure when manually shared | No secret scanner; relies on `.gitignore` and user review. |
| Unbounded local input size | Memory/disk exhaustion | No configured limits. |
| Source file replacement race after scanning | A local concurrent actor could change a path between discovery and open | No file-handle pinning or no-follow open flag. |
| Unsigned Windows release | Publisher cannot be authenticated by Windows | Checksums exist; no code signature. |
| Custom compression | Unexpected semantic change in an untested syntax edge case | Optional and off by default. |
| Incomplete restore folder | Partial data can remain after a write failure | Error names the directory; no automatic recursive deletion. |
| Plain-text outputs | Bundle/prompt contents remain readable at rest | No encryption feature. |

## Not applicable controls

Authentication, authorization, password hashing, session security, JWTs,
CSRF, CORS, HTTP security headers, SSRF protection, SQL/NoSQL injection, API
rate limiting, and server-side audit logs are not applicable to the current
local-only architecture.
