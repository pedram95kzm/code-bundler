# Deployment and Release

## Current release model

Code Bundler is a local desktop executable, not a hosted service. Deployment
means building and distributing a Windows binary/archive. There are no
development, staging, or production servers and no database migrations.

The repository currently identifies the application as version `1.0.0` in
`Cargo.toml`. The checked-in `dist/` directory contains the current Windows
release artifacts.

## Prerequisites

- Windows with the Microsoft Visual C++ build tools required by the MSVC Rust target.
- Rust 1.88 or newer, matching `rust-version` in `Cargo.toml`.
- The committed `Cargo.lock`; release and validation commands use `--locked`.

No environment variables, secrets, service credentials, containers, or
external runtime are required.

## Release build

From the repository root:

```console
cargo build --release --locked
```

The executable is produced under `target/release/`. Release profile settings in
`Cargo.toml` enable link-time optimization, one code-generation unit, symbol
stripping, and abort-on-panic behavior.

For the `x86_64-pc-windows-msvc` target, `.cargo/config.toml` requests the
static C runtime. `build.rs` generates the application icon and embeds Windows
version metadata at build time.

## Required validation

Before packaging, run:

```console
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Also launch the release executable on a supported Windows system and manually
exercise one Generate round trip and one Extract round trip. The repository
does not automate the native GUI smoke test.

## Package shape

The current archive contains:

```text
code-bundler-v1.0.0-windows-x86_64/
  code-bundler.exe
  LICENSE
  README.md
  sample_prompt.txt
  modification_file.txt
  VAZIRMATN-OFL.txt
```

`dist/SHA256SUMS.txt` records SHA-256 checksums for the standalone executable
and ZIP archive. Packaging is currently a manual maintainer process; there is
no release script that should be treated as canonical.

## CI/CD boundary

`.github/workflows/ci.yml` validates formatting, Clippy, and tests on pushes and
pull requests. It does not:

- construct or upload release archives;
- publish a repository release;
- sign the executable;
- update checksums;
- deploy a service.

## Installation and upgrade

There is no installer or automatic updater. Users extract the archive and run
`code-bundler.exe`. Upgrading means replacing the executable/archive with a new
release. User projects and generated bundle files are not stored inside the
application and require no migration.

## Rollback

Retain a previous verified archive and checksum. To roll back, stop Code
Bundler and run the previous executable. Bundle compatibility across future
versions is **UNKNOWN**; current version 1 accepts only the strict v1 marker.

## Current release limitations

- Artifacts are not code-signed, so Windows reputation or security prompts may appear.
- Packaging and checksum production are manual and not independently reproduced by CI.
- There is no installer, automatic update channel, or crash-reporting channel.
- Packaging for other operating systems is **NOT VERIFIED** and is intentionally outside this release procedure.
