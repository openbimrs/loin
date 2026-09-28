# 0004 — npm trusted publishing from the `npmjs.com` environment

- **Status:** Accepted
- **Date:** 2026-09-27
- **Deciders:** Friedrich Schrödter
- **Supersedes:** the npm half of [0002](0002-staged-npm-publish-and-lockstep-releases.md)

## Context

ADR 0002 staged npm publishes with a write token and approved each one with
2FA, because OIDC trusted publishing alone had no human approval step. The
openbimrs repositories now publish from one GitHub environment per registry
(`crates.io`, `npmjs.com`, `pypi.org`), each with a required reviewer, as
`openbimrs/ifc` already does for `@openbim/ifc`.

## Decision

`release.yml` publishes `@openbim/loin` with `npm publish --provenance` from a
job in the `npmjs.com` environment, authenticated by trusted publishing. The
environment's required reviewer is the human approval step. The package's
trusted publisher on npmjs.com names `openbimrs/loin`, `release.yml` and
`npmjs.com`. The `NPM_TOKEN` secret is removed. Lockstep versions from 0002
are unchanged.

## Alternatives considered

| Option | Why not |
| --- | --- |
| Keep `npm stage publish` with `NPM_TOKEN` | A long-lived token in the repository, and a second approval path that differs from every other openbimrs package. |
| Trusted publishing plus `npm stage publish` | Two approvals for one release; the environment gate already proves presence. |

## Consequences

**Positive**

- No npm token exists; a publish needs the tag, this workflow, this environment and a reviewer's approval.
- Published versions carry npm provenance.

**Negative / costs**

- Trusted publishing cannot create a package; a new package's first version is published by hand.
- Renaming `release.yml` or the environment breaks publishing until npmjs.com follows.

## Relation to existing code

`.github/workflows/release.yml`, `scripts/build-npm-package.py`, `scripts/check-release-version.py`.
