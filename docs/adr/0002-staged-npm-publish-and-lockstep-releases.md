# 0002 — Staged npm publishing and lockstep release versions

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Friedrich Schrödter
- **Supersedes:** —

## Context

Releases are cut by pushing a `v*` tag. npm is restricting tokens that bypass
2FA, and the maintainer wants a human approval step on every npm publish. Three
crates plus one npm package need versions a single tag can name unambiguously.

## Decision

We will publish npm through `npm stage publish` from CI, using a write token
that does not bypass 2FA; a maintainer approves each staged version with 2FA.
We will version every artifact in lockstep, so tag `vX.Y.Z` means every crate
and the npm manifest are `X.Y.Z`, enforced by `scripts/check-release-version.py`.

## Alternatives considered

| Option | Why not |
| --- | --- |
| Token that bypasses 2FA + `npm publish` | Removes the human approval step, and npm is phasing out 2FA-bypassing publish tokens. |
| OIDC trusted publishing | Also has no human approval step; it can be combined with staging later. Cannot perform a package's first publish. |
| Per-crate tags (`openbim-loin-wasm-v0.3.3`) | More release bookkeeping for crates that change together; nothing today needs independent cadence. |

## Consequences

**Positive**

- Every npm release has proof of presence; a CI token alone cannot publish.
- A tag can never ship an artifact whose manifest disagrees with it.

**Negative / costs**

- An unchanged crate is republished at the new version on each release.
- `npm stage publish` needs the package to exist, so the first publish was manual.

**Follow-ups / risks to watch**

- Rust crates are still published manually; automate once the order is stable.

## Relation to existing code

`.github/workflows/release.yml`, `scripts/check-release-version.py`, `scripts/build-npm-package.py`, `scripts/changelog-section.py`.
