# ADR-002: Non-Overwriting Portable Outputs

- **Status:** Observed and active
- **Decision scope:** Output safety and restored path policy
- **Evidence:** `src/paths.rs`, `src/workflow.rs`, `src/embedder.rs`, colocated tests

## Context

Code Bundler writes files derived from user-selected input. Bundle paths may be
created on filesystems with different naming and case rules, and generated
output names may already exist.

Historical product context is **UNKNOWN**.

## Problem

Restoration must not allow traversal or ambiguous/conflicting paths, and normal
Generate/Extract actions must not destroy existing user data.

## Decision

The current implementation:

- accepts only relative portable bundle paths;
- applies the same portable-path policy to added and renamed destinations;
- rejects empty, dot, parent, forbidden/control, trailing-dot/space, and
  Windows-device-name components;
- detects duplicates case-insensitively and rejects file/parent conflicts;
- validates the complete final tree after simultaneous delete/rename/add operations;
- creates generated files and restored files exclusively;
- allocates numeric suffixes instead of intentionally overwriting existing output;
- claims generated bundle and prompt names as a pair.

## Alternatives

Historical alternatives considered are **UNKNOWN**. Overwrite confirmation,
platform-native-only validation, and staging-directory commits are not present
in the current implementation.

## Reasoning

Observed benefit: portable validation avoids ambiguous restoration across
common filesystems, and exclusive creation reduces accidental data loss and
time-of-check/time-of-use overwrite risk.

Historical rationale unknown.

## Consequences

### Positive

- Traversal and absolute-path extraction are rejected.
- Common cross-filesystem collisions are rejected before output creation.
- Existing normal outputs are retained.
- Bundle and prompt suffixes remain paired.

### Negative

- Some paths valid on a particular host are intentionally rejected.
- Repeated operations create additional suffixed outputs that users must manage.
- A filesystem error after restore directory creation can still leave partial output.
- Concurrent name allocation behavior has not been stress-tested.
