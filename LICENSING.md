# Licensing

## Current license

Unless a file or package states otherwise, repository-authored software and
documentation are licensed under the **MIT License** (`MIT`). See
[`LICENSE`](LICENSE).

The MIT License permits use, modification, distribution, sublicensing, and
sale, including in proprietary and networked products, provided the copyright
notice and permission notice are retained.

## Version history

This repository has carried three licensing states. Each grant remains valid
for the versions in which it was made; a later change does not revoke an
earlier one.

| Versions | License |
|---|---|
| Up to and including `0.2.0` (published) | MIT |
| Unpublished AGPL interval (`07e486a` until this commit) | AGPL-3.0-or-later |
| `0.3.0` onwards | MIT |

No release was ever published under `AGPL-3.0-or-later`: the relicense landed
after `0.2.0` and no version was released before it was reverted here.
Consequently every published version of `openbim-loin` and `loin` is MIT, and
the divergence reported in issue #11 is resolved in favour of MIT.

Verified 2026-09-21 against the crates.io versions API: `0.1.0` MIT,
`0.2.0` MIT.

## Third-party material

Dependencies, standards, schemas, catalogs, fixtures, generated material, and
other third-party content retain their own copyright and license terms. Their
notices control where they differ from this repository's license. The
OpenBIM.rs license grant does not cover material its contributors do not have
the right to license.

ISO 7817-3 and ISO 23387 schema and example artifacts are ISO copyright. They
are not tracked or packaged by this repository; local copies belong under the
ignored `references/` directory.

## Contributions

Contributions are accepted under `MIT` unless an explicitly signed agreement
says otherwise. See [`CONTRIBUTING.md`](CONTRIBUTING.md).
