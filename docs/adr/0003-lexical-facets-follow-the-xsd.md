# 0003 — Lexical facets follow the XSD exactly, not a plausible reading

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Friedrich Schrödter
- **Supersedes:** —

## Context

Issue #4 reported that `EmailAddress` accepted addresses a reader would call
invalid, and proposed requiring exactly one `@`. The ISO 7817-3 schema declares
the facet `[^@]+@[^\.]+\..+`, whose trailing `.+` matches further `@`
characters. The proposal would have rejected schema-valid documents.

## Decision

We will implement every lexical restriction as the XSD states it, and prove it
with a differential test against the literal pattern. A stricter reading is a
separate opt-in validation, never a silent change to conformance.

## Alternatives considered

| Option | Why not |
| --- | --- |
| Exactly one `@` (issue proposal) | Rejects 234 of 5,460 generated schema-valid values; the library would reject files other conforming tools accept. |
| Full RFC 5322 validation | Not what the schema requires; same conformance break in the other direction. |

## Consequences

**Positive**

- Documents valid under the schema are never rejected by this library.
- The differential test (`tests/email_pattern.rs`) fails if the implementation drifts from the pattern.

**Negative / costs**

- Values such as `a@b@c.de` validate, which surprises users expecting RFC rules.

**Follow-ups / risks to watch**

- Apply the same differential approach to other restricted simple types as they gain typed parsers.

## Relation to existing code

`openbim-loin/src/model.rs` (`matches_actor_email_pattern`), `openbim-loin/tests/email_pattern.rs`.
