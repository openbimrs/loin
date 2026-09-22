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

The current unreleased line extends the `0.2.0` ISO 23387-backed domain boundary
with a strict LOIN XML document codec, explicit namespace migration, and
XSD-derived ISO 7817-3 clause-level validation. Validation is intentionally not
described as complete W3C XML Schema validation.

| Capability | Status |
| --- | --- |
| 2024 and 2022 draft namespace contracts | Implemented and tested |
| Purpose identity/text/imported-reference fields | Uses `openbim-dt::{Guid, MultiLanguageText, Language, Reference}` directly |
| Per-object specification `ConceptType` base and `ObjectTypeType` field | Uses `openbim-dt::{Concept, ObjectType}` directly |
| Alphanumerical property, quantity-kind, group, reference-document, unit, and dimension content | Uses the corresponding owned `openbim-dt` type contracts directly |
| Actor, milestone, format, required-document, threshold, detail, datum-registry, and georeferencing fields | Typed to the current XSD shape; imported values use canonical DT contracts directly |
| Compile-time DT ownership and mutation gates | Implemented |
| LOIN XML decoding/encoding | Strict bounded XML 1.0 codec implemented |
| Namespace migration | Explicit 2022 ↔ 2024 migration with collision detection and reports |
| ISO 7817-3 validation | XSD-derived LOIN structural, cardinality, enumeration, and scalar checks; not complete XSD validation of imported DT complex types |
| Lossless LOIN document round trips | Lossless-semantic syntax tree; not byte-for-byte |

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
use openbim_loin::{LoinDocument, OutputNamespace, Purpose, dt};

let guid: dt::Guid = "11111111-1111-1111-1111-111111111111".parse()?;
let name = dt::MultiLanguageText::new("en", "Coordination")?;
let purpose = Purpose::new(guid, name);
assert_eq!(purpose.name().unwrap().text(), "Coordination");

let xml = r#"<l:LevelOfInformationNeed xmlns:l="https://iso.org/2024/LOIN" />"#;
let document = LoinDocument::parse(xml)?;
let encoded = document.to_xml_string(OutputNamespace::Preserve)?;
assert_eq!(LoinDocument::parse(&encoded)?.root(), document.root());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Writing documents

`LoinDocument::from_model` builds a document from the typed model, so documents
can be authored rather than only parsed:

```rust
use openbim_loin::{LevelOfInformationNeed, LoinDocument, OutputNamespace};

# fn demo(model: &LevelOfInformationNeed) -> Result<(), Box<dyn std::error::Error>> {
let document = LoinDocument::from_model(model)?;
let xml = document.to_xml_string(OutputNamespace::Preserve)?;
assert!(document.validate().is_empty());
# Ok(())
# }
```

Children are written in the order the schema declares, and the output reparses
and revalidates unchanged.

**Scope.** LOIN-owned content is written in full. ISO 23387-owned complex
content (`ObjectType`, `Property`, `QuantityKind`, `Dimension`, `Unit`,
`ReferenceDocument`, `GroupOfProperties`) is **not** writable here: the
`openbim-dt` 0.2 owned types expose no serializer and `dt::Element` cannot be
constructed downstream. Those cases return
`AuthoringError::UnwritableDtContent`, naming the element, instead of emitting
invented ISO 23387 syntax. To edit such documents today, parse one and mutate
it through `XmlElement::nodes_mut` / `attributes_mut`, which keeps the DT
subtrees verbatim.

`openbim-loin::dt` re-exports the exact `openbim-dt` contract version consumed by
this release. Do not create lookalike GUID, text, reference, or property types in
LOIN clients.

## Install

Use either package name:

```bash
cargo add openbim-loin@0.3
# or
cargo add loin@0.3
```

Do not depend on both names directly. The alias already brings in the canonical
package at an exact version.

## Architecture

- [`docs/architecture.md`](docs/architecture.md) — repository, dependency, namespace, DT, and alias boundaries
- [`docs/xml.md`](docs/xml.md) — codec guarantees, migration semantics, and exact validation coverage
- [`openbimrs/dt`](https://github.com/openbimrs/dt) — canonical ISO 23387 contracts
- [`openbimrs/openbim`](https://github.com/openbimrs/openbim) — integrated workspace and facade


Drafts use both `https://iso.org/2022/LOIN` and
`https://iso.org/2024/LOIN`. The codec preserves the observed namespace and
every write requires an explicit preserve-or-target policy. Migration rewrites
only LOIN bindings and fails closed on expanded-attribute collisions.

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

The wasm checks need `wasm-bindgen-cli` (matching the locked `wasm-bindgen`
version, see `scripts/wasm-bindgen-version.py`), the `wasm32-unknown-unknown`
target, and Node. Without them the gate skips those checks locally. CI sets
`LOIN_WASM_STRICT=1`, which turns a skip into a failure so the assertions
cannot silently no-op.

## Releasing

Pushing a `v*` tag runs `.github/workflows/release.yml`, which refuses a tag
that disagrees with `openbim-loin-wasm/Cargo.toml` and its npm manifest, runs
the full gate on the tagged commit, then creates a GitHub release with notes
taken from this changelog and publishes `@openbimrs/loin` to npm.

```bash
git tag -a v0.3.4 -m "openbim-loin-wasm 0.3.4"
git push origin v0.3.4
```

Rust crates are still published manually with `cargo publish`; the workflow
covers the npm package and the GitHub release only.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Capability work must add executable
evidence and update the status table without overstating coverage.

## License

MIT — see [`LICENSE`](LICENSE).
