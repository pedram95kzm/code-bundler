# Architecture Decisions

The three original ADRs are consolidated here. All are **observed and active**.
Benefits describe current behavior; original historical rationale and
alternatives considered are unknown.

## ADR-001: Strict v1 bundle format

**Decision:** require `==code-bundler:v1==` and UTF-8 content byte lengths;
validate framing, paths, duplicates, and hierarchy before restore output.
Reject unknown/unversioned formats without fallback.

**Benefit:** exact boundaries preserve non-ASCII, empty files, final newlines,
and delimiter-like content; malformed input fails early.

**Tradeoffs:** hand edits can invalidate lengths, binary data is skipped, and
future formats need an explicit compatibility policy.

**Evidence:** [extractor.rs](../src/extractor.rs),
[embedder.rs](../src/embedder.rs), and their tests. See the
[format contract](ARCHITECTURE.md#bundle-format-v1).

## ADR-002: Non-overwriting portable outputs

**Decision:** require portable relative paths, detect case-insensitive and
file/parent conflicts, and validate the final changed tree. Create files and
directories exclusively, allocate suffixes, and pair bundle/prompt suffixes.

**Benefit:** reject traversal/ambiguous paths and retain existing outputs.

**Tradeoffs:** some host-valid names are rejected, repeated operations leave
additional outputs, concurrent allocation needs more testing, and late restore
I/O failures can leave partial directories.

**Evidence:** [paths.rs](../src/paths.rs), [workflow.rs](../src/workflow.rs),
[embedder.rs](../src/embedder.rs), and their tests.

## ADR-003: Local desktop boundary

**Decision:** use a local Iced binary with OS permissions and user-selected
files. Provide no network API, accounts, saved preferences, or automatic AI
submission. Users choose external tools and transfer content manually.

**Benefit:** normal operation needs no application backend/account/service
credentials, and users control when project data leaves the machine.

**Tradeoffs:** external tools set their own privacy policies; native GUI behavior
needs platform testing; support relies on reported status. There is no remote
sync, telemetry, or automatic update channel.

**Evidence:** [main.rs](../src/main.rs), [app.rs](../src/app.rs), and
[Cargo.toml](../Cargo.toml).

## Related policies and changes

Local ignore rules, VCS pruning, no symlink following, skipped binary data,
opt-in lexical compression, compiled templates/fonts, and original-line change
semantics are also observed policies with unknown historical rationale.

When a durable decision changes, update its status, evidence, and migration or
compatibility effect here. Retain ADR identifiers in this register instead of
creating additional documentation files.
