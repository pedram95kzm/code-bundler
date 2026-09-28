# Code Bundler Documentation

This directory is the current-state knowledge base for Code Bundler v1.0.0.
It documents the working tree as inspected on 2026-09-28. The implementation
is the source of truth when a document becomes stale.

## Evidence labels

- **VERIFIED**: directly supported by source, configuration, tests, or packaged artifacts.
- **INFERRED**: strongly suggested by the implementation but not stated as product intent.
- **UNKNOWN**: not determinable from the repository.
- **NOT VERIFIED**: plausible, but not exercised or proven by repository evidence.

## Documentation map

| Document | Purpose | Audience | Status |
| --- | --- | --- | --- |
| [01-idea.md](01-idea.md) | Observed product idea, scope, users, and constraints | Everyone | Complete |
| [02-requirements.md](02-requirements.md) | Functional and non-functional behavior | Product, developers, QA | Complete |
| [03-prd.md](03-prd.md) | Reconstructed current-product requirements | Product, developers | Complete |
| [04-features.md](04-features.md) | Feature contracts and failure behavior | Product, QA, developers | Complete |
| [05-architecture.md](05-architecture.md) | Runtime components, boundaries, and architecture diagrams | Developers, architects | Complete |
| [06-technical-design.md](06-technical-design.md) | Modules, algorithms, formats, and implementation details | Developers | Complete |
| [07-security.md](07-security.md) | Trust boundaries, validation, risks, and dependency security | Developers, security reviewers | Complete |
| [08-testing-strategy.md](08-testing-strategy.md) | Test inventory, commands, CI, and gaps | Developers, QA | Complete |
| [09-deployment.md](09-deployment.md) | Build and Windows release procedure | Maintainers | Complete |
| [10-troubleshooting.md](10-troubleshooting.md) | Verified build and runtime failure guidance | Users, developers | Complete |
| [11-developer-guide.md](11-developer-guide.md) | Setup and safe contribution workflow | Developers | Complete |
| [12-user-guide.md](12-user-guide.md) | End-user workflows, errors, limitations, and FAQ | End users | Complete |
| [13-configuration.md](13-configuration.md) | Compile-time, Cargo, CI, and user-controlled settings | Developers, maintainers | Complete |
| [14-data-flow.md](14-data-flow.md) | Bundle, prompt, extraction, and content/file change data flows | Developers, AI agents | Complete |
| [15-glossary.md](15-glossary.md) | Product and implementation terminology | Everyone | Complete |
| [DECISIONS.md](DECISIONS.md) | Index of observed technical decisions | Developers, architects | Complete |
| [ADR-001](decisions/ADR-001-strict-v1-bundle-format.md) | Strict v1 bundle format | Developers, architects | Complete |
| [ADR-002](decisions/ADR-002-non-overwriting-portable-outputs.md) | Portable, non-overwriting output policy | Developers, security reviewers | Complete |
| [ADR-003](decisions/ADR-003-local-desktop-boundary.md) | Local desktop/application boundary | Developers, architects | Complete |
| [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) | Unknown or unverified intent | Product owners, maintainers | Complete |
| [AI-CONTEXT.md](AI-CONTEXT.md) | Compact handoff for future AI agents | AI agents, developers | Complete |

## Not applicable

| Expected area | Status | Evidence |
| --- | --- | --- |
| Database design | NOT APPLICABLE | The application has no database, schema, migration, or persistent state beyond user-selected files. |
| API design/OpenAPI | NOT APPLICABLE | The application exposes and consumes no network API. |
| Authentication/authorization design | NOT APPLICABLE | There are no accounts, roles, sessions, or protected remote resources. Filesystem access is inherited from the operating-system user. |
| External integrations | NOT APPLICABLE | The generated prompt is a file for manual use; the application does not call an AI provider or other service. |
| Service operations/runbook | NOT APPLICABLE | There is no deployed service, server process, health check, queue, scheduled job, monitoring system, or backup subsystem. |

## Documentation audit

- The root [`README.md`](../README.md) is accurate and remains the concise user/developer quick-start.
- The packaged README under `dist/` mirrors the root README for the Windows archive.
- `modification_file.txt` demonstrates all four supported change-record types;
  the full contract is documented in `04-features.md` and `06-technical-design.md`.
- `sample_prompt.txt` is both documentation and a compile-time application input.
- No prior `docs/` directory existed.
- Repository history contains only brief commit subjects, so most historical product and architecture rationale is **UNKNOWN**.
