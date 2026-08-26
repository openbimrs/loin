# OpenBIM.rs LOIN

[![CI](https://github.com/openbimrs/loin/actions/workflows/ci.yml/badge.svg)](https://github.com/openbimrs/loin/actions/workflows/ci.yml)
[![openbim-loin](https://img.shields.io/crates/v/openbim-loin.svg)](https://crates.io/crates/openbim-loin)
[![loin](https://img.shields.io/crates/v/loin.svg)](https://crates.io/crates/loin)
[![docs.rs](https://docs.rs/openbim-loin/badge.svg)](https://docs.rs/openbim-loin)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://www.rust-lang.org)

Pure-Rust infrastructure for ISO 7817-3 / EN 17412-3 Level of Information Need
(LOIN): machine-readable requirements for geometric information, alphanumeric
information, and documentation at a given purpose and milestone.

This repository is the canonical home of the LOIN family in
[OpenBIM.rs](https://github.com/openbimrs/openbim). The integration repository
pins it under `packages/loin`.

## Status

Version `0.2.0` implements the **ISO 23387-backed domain boundary** used by the
LOIN schema. It is not yet a LOIN XML codec or schema validator.

| Capability | Status |
| --- | --- |
| 2024 and 2022 draft namespace contracts | Implemented and tested |
| Purpose identity/text/reference fields | Uses `openbim-dt::{Guid, MultiLanguageText, Language, Reference}` directly |
| Per-object specification `ConceptType` base and `ObjectTypeType` field | Uses `openbim-dt::{Concept, ObjectType}` directly |
| Alphanumerical property, quantity-kind, group, reference-document, unit, and dimension content | Uses the corresponding owned `openbim-dt` type contracts directly |
| Actor, milestone, format, document, threshold, detail, and registry-reference DT fields | Uses canonical DT value/type contracts directly |
| Compile-time DT ownership and mutation gates | Implemented |
| LOIN XML decoding/encoding | Not implemented |
| Namespace migration | Not implemented |
| ISO 7817-3 XSD or clause-level validation | Not implemented |
| Lossless LOIN document round trips | Not implemented |

Parsing a DT element with `openbim-dt` does not imply that this crate can parse a
whole LOIN document.

## Crates

| Package | Purpose |
| --- | --- |
| [`openbim-loin`](openbim-loin/) | Canonical implementation; owns every LOIN item and behavior |
| [`loin`](loin/) | Pure re-export alias pinned to the exact canonical version |

Cargo has dependency renaming but no crates.io package aliases. The `loin`
package defines nothing and re-exports `openbim-loin`, preserving one canonical
API and type identity.

## Example

```rust
use openbim_loin::{Purpose, dt};

let guid: dt::Guid = "11111111-1111-1111-1111-111111111111".parse()?;
let name = dt::MultiLanguageText::new("en", "Coordination")?;
let purpose = Purpose::new(guid, name);
assert_eq!(purpose.name().text(), "Coordination");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`openbim-loin::dt` re-exports the exact `openbim-dt` contract version consumed by
this release. Do not create lookalike GUID, text, reference, or property types in
LOIN clients.

## Install

Use either package name:

```bash
cargo add openbim-loin@0.2
# or
cargo add loin@0.2
```

Do not depend on both names directly. The alias already brings in the canonical
package at an exact version.

## Architecture

- [`docs/architecture.md`](docs/architecture.md) — repository, dependency, namespace, DT, and alias boundaries
- [`openbimrs/dt`](https://github.com/openbimrs/dt) — canonical ISO 23387 contracts
- [`openbimrs/openbim`](https://github.com/openbimrs/openbim) — integrated workspace and facade


Drafts use both `https://iso.org/2022/LOIN` and
`https://iso.org/2024/LOIN`. A future codec must preserve the observed namespace
and require an explicit output target rather than silently migrating it.

## Standards material

No ISO, DIN, CEN, or other restricted standards artifact is tracked or packaged.
Locally available references belong under ignored `references/`. A fixture may
enter version control only when its redistribution terms are known and compatible
with this repository.

## Development

Requires Rust `1.88` or newer and Python `3.10` or newer.

```bash
git clone https://github.com/openbimrs/loin.git
cd loin
./scripts/gate.sh
```

The gate checks formatting, build, tests, Clippy, rustdoc, DT contract
ownership, semantic alias purity, isolated mutations, and canonical crates.io
package verification using command exit codes. The alias package is verified
after its exact canonical version is registry-visible.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Capability work must add executable
evidence and update the status table without overstating coverage.

## License

MIT — see [`LICENSE`](LICENSE).
