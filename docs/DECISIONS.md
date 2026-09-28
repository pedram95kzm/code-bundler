# Architecture Decision Index

This register records durable decisions visible in the current implementation.
It does not claim that reconstructed context was the original historical
reasoning.

| ADR | Decision | Status | Primary evidence |
| --- | --- | --- | --- |
| [ADR-001](decisions/ADR-001-strict-v1-bundle-format.md) | Use a strict, byte-length-framed v1 text bundle | Observed and active | `src/extractor.rs`, `src/embedder.rs`, tests |
| [ADR-002](decisions/ADR-002-non-overwriting-portable-outputs.md) | Validate portable paths and never intentionally overwrite output | Observed and active | `src/paths.rs`, `src/workflow.rs`, `src/embedder.rs`, tests |
| [ADR-003](decisions/ADR-003-local-desktop-boundary.md) | Keep the application local and user-mediated | Observed and active | `src/main.rs`, `src/app.rs`, dependency/config inspection |

## Additional observed decisions

These are documented in the numbered design documents but do not have enough
known context to justify separate ADRs:

- Project-local `.gitignore` behavior with explicit VCS-directory pruning.
- Symbolic links are not followed during project scanning.
- Binary/unsupported data becomes a skip message rather than binary framing.
- Optional compression is extension-aware and disabled by default.
- The prompt and font are compiled into the executable.
- Restoration validates the full bundle and change set before output creation.
- Line replacements are applied in descending original-line order; add/delete/rename
  operations are then evaluated together and the complete final tree is revalidated.
- I/O work is dispatched via Iced tasks instead of running directly in UI update handling.

Historical rationale for these choices is **UNKNOWN** unless an ADR explicitly
states otherwise. Proposed changes belong in a new/superseding ADR and must not
be rewritten into these records as if they were historical facts.
