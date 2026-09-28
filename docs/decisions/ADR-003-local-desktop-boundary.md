# ADR-003: Local Desktop Boundary

- **Status:** Observed and active
- **Decision scope:** Runtime and integration boundary
- **Evidence:** `src/main.rs`, `src/app.rs`, manifest and source inspection

## Context

The product prepares project data and instructions for a user-mediated AI
workflow. Source projects may contain private code and credentials.

Historical product context is **UNKNOWN**.

## Problem

The application needs a user interface for selecting local inputs and writing
derived outputs without requiring an application-operated backend.

## Decision

Code Bundler is implemented as a local Iced desktop binary. It reads and writes
only user-accessible filesystem paths. It does not provide or consume a network
API, manage accounts, persist preferences, or submit data to an AI service.
Users manually transfer generated content to any external tool they choose.

## Alternatives

Historical alternatives considered are **UNKNOWN**. A hosted service, CLI,
automatic AI-provider integration, and browser application are not implemented.

## Reasoning

Observed benefit: the application itself has no server-side data custody,
credential management, service availability, or network integration surface.

Historical rationale unknown.

## Consequences

### Positive

- Core operation works without a project-specific account or network service.
- There are no server operations, database migrations, or service secrets.
- Users control when and where generated data leaves the machine.

### Negative

- The application cannot enforce the privacy behavior of tools users choose afterward.
- There is no remote synchronization, collaboration, telemetry, or automatic update channel.
- Native GUI behavior requires platform-specific validation.
- Support depends on user-reported status text because no logs or remote diagnostics exist.

