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
pins this repository under `packages/loin`.

## Status

The published `0.1.0` releases are **reserved scaffolds with namespace
contracts**, not a LOIN document reader, writer, or validator.

| Capability | Status |
| --- | --- |
| 2024 and 2022 draft namespace constants | Implemented |
| Known-namespace recognition helper | Implemented and tested |
| Two synchronized crates.io names | Implemented and structurally gated |
| LOIN XML decoding/encoding | Not implemented |
| Namespace migration | Not implemented |
| ISO 7817-3 validation | Not implemented |
| Lossless unknown-data round-trip | Not implemented |

No parser, writer, migration, or validation capability should be inferred from
the crates existing on crates.io.

## Crates

| Package | Purpose |
| --- | --- |
| [`openbim-loin`](openbim-loin/) | Canonical implementation; owns every item and behavior |
| [`loin`](loin/) | Pure re-export alias pinned to the exact canonical version |

Cargo has dependency renaming but no crates.io package aliases. Two package
records are therefore required to reserve both names. `loin` defines nothing of
its own and re-exports `openbim-loin`, so both names expose the same items rather
than compiling duplicate implementations.

## Install

Use either package name:

```bash
cargo add openbim-loin
# or
cargo add loin
```

```rust
use openbim_loin::{is_known_namespace, NAMESPACE_2024};

assert!(is_known_namespace(NAMESPACE_2024));
// The short package exposes the same items under `loin::`.
```

Do not depend on both names directly. The alias already brings in the canonical
package at an exact version.

## Architecture

- [`docs/architecture.md`](docs/architecture.md) — repository, dependency, namespace, and alias boundaries
- [`openbimrs/openbim`](https://github.com/openbimrs/openbim) — integrated workspace and facade
- [`openbim-core`](https://crates.io/crates/openbim-core) — shared openBIM vocabulary

LOIN is deliberately exchange-format focused. EN 17412-1 defines the concepts;
part 3 defines the machine-readable exchange. Drafts use both
`https://iso.org/2022/LOIN` and `https://iso.org/2024/LOIN`, so a future codec
must treat namespace migration as explicit behavior rather than silently
normalizing input.

## Standards material

No ISO, DIN, CEN, or other restricted standards artifact is tracked or packaged.
Locally available references belong under ignored `references/`. A fixture may
enter version control only when its redistribution terms are known and compatible
with this repository.

## Development

Requires Rust `1.88` or newer and Python `3.10` or newer for the semantic alias
and mutation gates.

```bash
git clone https://github.com/openbimrs/loin.git
cd loin
./scripts/gate.sh
```

The gate checks formatting, build, tests, Clippy, rustdoc, semantic alias purity,
isolated alias mutations, and crates.io package verification using command exit
codes.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Capability work must add executable
evidence and update the status table without overstating coverage.

## License

MIT — see [`LICENSE`](LICENSE).
