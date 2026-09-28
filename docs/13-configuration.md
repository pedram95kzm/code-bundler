# Configuration

## Configuration model

Code Bundler has no runtime configuration file, environment variables, secret
store, command-line options, registry settings, or user preference persistence.
Configuration is split between controls selected in the GUI and values compiled
into the executable.

## User-controlled settings

| Setting | Location | Default/behavior | Persistence |
| --- | --- | --- | --- |
| Source project path | Generate view | Empty until selected/entered | Session only |
| Request text | Generate view | Empty; included in generated prompt | Session only |
| Compression | Generate view | Off | Session only |
| Bundle path | Extract view | Empty until selected/entered | Session only |
| Modification path | Extract view | Empty and optional | Session only |

Paths can be selected with native dialogs or typed directly. A matching pair of
surrounding double quotes is removed from path input; single quotes are not
special. UI labels are fixed English text, while shaped Persian/RTL input is
supported by the embedded fonts.

## Project-controlled scan configuration

The selected project's `.gitignore` files control ignored paths. Nested rules
are honored. The scanner intentionally does not use parent-repository rules,
global Git ignore files, `.ignore`, or Git's exclude file.

Regardless of ignore rules, these version-control directories are pruned:

```text
.git
.hg
.svn
.jj
```

Root-level generated files matching Code Bundler's bundle/prompt output naming
are excluded to prevent re-bundling prior output.

## Compile-time application inputs

| Input | Effect | Change consequence |
| --- | --- | --- |
| `sample_prompt.txt` | Template for generated prompts | Rebuild executable. |
| `assets/fonts/Vazirmatn-Regular.ttf` and `Vazirmatn-Bold.ttf` | Embedded Persian-capable UI fonts | Rebuild executable; retain license obligations. |
| `assets/code-bundler-icon.png` | Source for generated Windows icon | Rebuild executable. |
| `Cargo.toml` package metadata | Version and build/dependency policy | Update deliberately and rebuild. |
| `build.rs` | Windows icon and executable metadata | Rebuild on Windows. |

The prompt template includes placeholders replaced by `src/workflow.rs`.
Changing the placeholder contract without changing the renderer can leave
literal placeholders or omit required content.

## Build configuration

`Cargo.toml` declares:

- package version `1.0.0`;
- Rust edition 2024;
- minimum Rust version 1.88;
- `publish = false`;
- release optimization, stripping, LTO, one code-generation unit, and abort-on-panic.

`.cargo/config.toml` adds the static C runtime flag for the
`x86_64-pc-windows-msvc` target. `Cargo.lock` pins the dependency graph and
should be honored with `--locked` during validation and release builds.

## CI configuration

`.github/workflows/ci.yml` defines the repository validation matrix and runs
formatting, Clippy, and tests. It grants read-only repository contents access
and cancels superseded runs for the same concurrency group.

CI configuration does not publish or sign releases.

## Fixed implementation policies

The following are not exposed as configuration:

- strict `==code-bundler:v1==` format marker;
- output naming patterns and non-overwrite suffixing;
- portable path restrictions;
- VCS-directory pruning;
- symlink non-following behavior;
- supported decoding and binary detection;
- compression extension routing;
- change-record syntax and file-operation semantics;
- font shaping and theme/layout choices.

Changing these is a code/format decision, not a user configuration change.

## Environment and secrets

No environment variable is read by application code, and no secret is required
to build or run the application. Standard Cargo/rustc environment variables may
affect the toolchain externally, but they are not an application configuration
contract.
