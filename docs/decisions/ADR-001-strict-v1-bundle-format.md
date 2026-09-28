# ADR-001: Strict v1 Bundle Format

- **Status:** Observed and active
- **Decision scope:** Persistent bundle interchange format
- **Evidence:** `src/extractor.rs`, `src/embedder.rs`, colocated tests

## Context

The application needs one text artifact that can carry many relative paths and
source contents. Source text can itself contain arbitrary delimiter-like lines,
and non-ASCII content must be preserved after decoding.

Historical product context is **UNKNOWN**.

## Problem

Restoration must determine exact entry boundaries and reject malformed or
ambiguous input before writing user-visible files.

## Decision

The current implementation:

- requires the exact marker `==code-bundler:v1==`;
- encodes paths and contents using explicit UTF-8 byte lengths;
- rejects malformed lengths, truncation, trailing/invalid structure, unsafe
  paths, duplicates, and path conflicts;
- performs logical validation before creating the restore directory.

Only v1 is accepted. There is no permissive fallback parser.

## Alternatives

Historical alternatives considered are **UNKNOWN**. Plausible formats such as
an archive, JSON, or delimiter-only text are not evidenced as prior proposals
and are therefore not attributed to the original authors.

## Reasoning

Observed benefit: byte lengths make entry boundaries independent of characters
inside content, while a strict marker prevents ambiguous format detection.

Historical rationale unknown.

## Consequences

### Positive

- Source content can contain marker-like text without ending an entry.
- Parser failures are explicit and early.
- A format version is available for compatibility decisions.

### Negative

- Hand-editing content or lengths can invalidate the bundle.
- Producers must measure UTF-8 bytes exactly.
- Future formats require an explicit compatibility/version policy, which is currently unknown.
- Unsupported binary bytes are represented by a skip message rather than preserved.

