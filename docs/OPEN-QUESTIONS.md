# Open Questions

These questions could not be resolved from source, configuration, tests,
artifacts, or repository history. They do not prevent understanding or using
the current v1 implementation.

## Blocking

None for documenting the current system.

For a future format change or release policy change, the relevant non-blocking
questions below may become blocking design inputs.

## Non-blocking product questions

- **UNKNOWN:** What was the original business brief and primary target persona?
- **UNKNOWN:** Is the AI-oriented workflow intended for a particular provider,
  model, or context limit? The application itself is provider-neutral.
- **UNKNOWN:** Which languages, project sizes, and repository shapes are expected
  to be officially supported?
- **UNKNOWN:** Is source compression considered production-stable for every
  routed extension, or an optional best-effort convenience?
- **UNKNOWN:** Is full UI localization intended, or is shaped Persian/RTL input
  support sufficient for the product?
- **UNKNOWN:** What end-user support and release announcement channels are intended?

## Non-blocking technical questions

- **UNKNOWN:** What historical rationale led to the current v1 delimiters and
  byte-framing layout?
- **UNKNOWN:** Is future backward compatibility with v1 bundles a product requirement?
- **UNKNOWN:** What maximum project size, file size, memory use, or duration is acceptable?
- **UNKNOWN:** Should a late restore write failure preserve the incomplete
  directory for diagnosis or remove it automatically?
- **UNKNOWN:** Should skipped binary files be recreated as placeholders,
  omitted, or supported by a future binary-safe representation?
- **UNKNOWN:** Are parent/global Git ignore sources intentionally excluded or
  simply never required?
- **UNKNOWN:** What is the intended behavior if source files change while a
  Generate operation is scanning them?
- **UNKNOWN:** What accessibility standard, keyboard workflow, display scaling,
  and screen-reader support level is required?
- **UNKNOWN:** Is code signing required before public distribution, and who owns
  the signing identity and release provenance?
- **UNKNOWN:** Should release packaging/checksum creation become automated, and
  what system is authoritative for published artifacts?

## Not verified

- Maximum practical input size and peak memory use have not been benchmarked.
- Native dialog, RTL shaping, display scaling, and accessibility behavior have
  not been covered by automated UI tests.
- Cross-version compatibility has not been tested because only v1 exists.
- Resilience to disk-full, permission changes, and concurrent output-name races
  has not been fault-tested.
- The exact list of source extensions routed to each compression algorithm is
  implemented but is not backed by a comprehensive language corpus.
