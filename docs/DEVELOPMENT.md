# Development and Releases

## Setup

Use Git, Rust 1.88 or newer (edition 2024), and native tooling for the selected
target. Windows builds need MSVC build tools. Ubuntu uses these CI dependencies:

```sh
sudo apt-get update
sudo apt-get install -y pkg-config libwayland-dev libxkbcommon-dev
```

Run from the repository root with `cargo run --locked`. GUI execution needs a
graphical session and working dialogs. The app uses the OS Documents folder for
`code_bundler` outputs; no application account, service, or secret setup is needed.

## Validation and CI

```console
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Use `cargo fmt --all` to apply formatting. Check the declared minimum with
`cargo +1.88.0 test --all-targets --locked` when that toolchain is installed.

[ci.yml](../.github/workflows/ci.yml) runs formatting, strict Clippy, and locked
tests on Ubuntu/Windows latest for pushes and pull requests. It installs Linux
packages, uses stable Rust, grants read-only repository access, pins actions to
commits, and cancels superseded runs. It performs no release build, dependency
audit, coverage measurement, signing, or publishing. Commit workflow changes
alongside source changes when fixing CI.

## Configuration and contributions

Use the [module map](ARCHITECTURE.md#runtime-and-modules) to keep policy in core
modules and GUI code focused on state/task dispatch. Errors cross workflow
boundaries as user-facing strings.

| Input | Contract |
| --- | --- |
| [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock) | Version, Rust minimum, dependencies, and locked graph; crate publication is disabled. |
| [.cargo/config.toml](../.cargo/config.toml) | Static C runtime for x86_64 Windows MSVC. |
| [sample_prompt.txt](../sample_prompt.txt) | Compiled prompt; preserve both renderer placeholders. |
| [assets/](../assets/) and [build.rs](../build.rs) | Embedded fonts/icon and Windows resources. |
| [output.rs](../src/output.rs) | Documents-folder discovery and shared output location. |

Rebuild after template/font/icon/code changes. Preferences are not saved, and
there is no application-specific runtime config or CLI interface.

Preserve byte framing, portable paths, full change validation, exclusive output
creation, paired suffixes, original-line semantics, and opt-in compression.
Update the relevant guide and [DECISIONS.md](DECISIONS.md) when contracts change;
distinguish proposed behavior from current implementation.

## Tests and gaps

Colocated Rust `#[test]` functions cover GUI helper/state logic, compactors,
paths, discovery/encoding, framing, round trips, change conflicts/escaping,
line endings, and generation naming/templates. Filesystem tests use unique
temporary directories and real I/O; panics before cleanup can leave files.

Format/path/workflow changes need focused success and adversarial failure
cases. Manually check native windows/dialogs, layout, RTL, focus, scaling, and
accessibility; no GUI harness exists. Fault injection, concurrent name races,
large-project benchmarks, parser fuzzing, and broader encoding/compression
corpora remain gaps. No coverage percentage or verified macOS release is recorded.

## Windows release

After validation, manually exercise Generate and Extract in
`target/release/code-bundler.exe`. Release builds strip symbols, enable LTO and
one code-generation unit, and abort on panic. Windows builds embed a multi-size
ICO/version metadata and suppress the release console window.

Package the executable with `LICENSE`, `README.md`, `sample_prompt.txt`,
`modification_file.txt`, and the Vazirmatn license from
[assets/fonts/OFL.txt](../assets/fonts/OFL.txt). Produce and verify SHA-256
checksums for executable/archive. Packaging is manual; no canonical script,
installer, updater, or signing pipeline exists. Other-platform packaging is
unverified.

Review dependency health separately before release; earlier audit notes flagged
an unmaintained transitive `ttf-parser` dependency. Retain a verified previous
archive for rollback. Replacing the executable upgrades the app; future bundle
versions need an explicit compatibility policy, and the current parser accepts
only v1.

## Build troubleshooting

| Failure | Action |
| --- | --- |
| `wayland-client.pc` missing | Install `libwayland-dev` and `pkg-config`; confirm the Linux setup is in the committed workflow. |
| Clippy warning fails CI | Run the strict command above and fix warnings; fixed-size groups use `as_chunks::<2>()`. |
| Rust version rejected | Compare active toolchain, manifest, and dependency requirements. |
| Cargo rejects `--locked` | Inspect manifest/lockfile differences before reviewing/regenerating dependencies. |
| Windows resource build fails | Check MSVC tooling and PNG/font/resource assets. |
| Local checks differ from CI | Compare OS/target, toolchain, packages, and the commit/workflow being tested. |

For runtime failures, reduce the input and preserve exact GUI status text.
Rebuild after template edits; inspect UTF-8 byte counts for framing issues.
Recovery guidance is in [USER-GUIDE.md](USER-GUIDE.md#troubleshooting-and-limits).
