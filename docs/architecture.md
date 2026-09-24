# Architecture

## Repository role

`openbimrs/loin` is the canonical source repository for the LOIN family.
`openbimrs/openbim` pins a verified commit at `packages/loin` and provides
ecosystem-level integration tests and the feature-gated `openbim` facade.

The child repository must remain buildable without the integration workspace.
Published crates therefore use explicit package metadata and versioned registry
dependencies, not inheritance from a parent workspace.

## Package identity

```text
loin  -- exact-version dependency -->  openbim-loin  -->  openbim-dt
(alias; no items)                       (all LOIN behavior)
                                              ^
openbim-loin-wasm  -- exact-version ---------+
(wasm-bindgen bindings; npm @openbim/loin)
```

`openbim-loin-wasm` is a separate package rather than a feature on
`openbim-loin` because `crate-type = ["cdylib"]` is per-package and cannot be
toggled by a feature ([ADR 0001](adr/0001-wasm-bindings-as-separate-crate.md)).
The core crate therefore never carries a wasm or JS dependency; the gate pins its
exact dependency set.

Cargo permits consumers to rename a dependency locally, but crates.io has no
publisher-side alias facility. Reserving both `openbim-loin` and `loin` requires
two package records. The short package contains only:

```rust
pub use openbim_loin::*;
```

This is not duplicated implementation. Every public item originates in
`openbim-loin`, so dependency graphs that encounter both names still have one
type identity. The alias dependency uses an exact `=` version requirement to
prevent canonical and alias releases from drifting.

## Dependency direction

```text
core / data templates / XML codec  <-  LOIN
                                      |
openbim facade  --------------------->+
```

- LOIN consumes `openbim-dt 0.3` directly and re-exports it as
  `openbim_loin::dt`. Imported GUID, language, multilingual-text, reference,
  `ConceptType`, object-type, property, group, quantity-kind, reference-document,
  unit, and dimension boundaries therefore keep one DT-owned Rust type identity.
- LOIN may consume shared vocabulary and direct XML encoding infrastructure.
- IFC, core, codec, and data-template crates must never depend on LOIN.
- The `openbim` facade may optionally re-export LOIN.
- A future IFC mapping belongs in an explicit bridge, not in the LOIN data model.

## Namespace contract

Known public drafts declare different namespaces:

- `https://iso.org/2024/LOIN`
- `https://iso.org/2022/LOIN`

The codec records the namespace observed on the root and separately tracks the
current syntax-tree namespace after migration. Every write requires an explicit
`OutputNamespace`: preserve current bindings or target a known edition.
Migration rewrites only names and declarations resolved to the source LOIN URI,
leaves DT and extension namespaces untouched, reports exact change counts, and
fails closed if rewritten attributes would collide by expanded name.

Namespace recognition is not validation. The validator checks the audited draft
schema's unqualified local-element structure and reports namespace misuse, but no
namespace is called final until the published standard establishes it.

## XML and validation boundaries

The XML parser owns a generic LOIN document syntax tree rather than borrowing
views from an input buffer. Parsing enforces XML 1.0 safety and resource budgets;
validation is a separate pass so invalid-but-well-formed documents and unknown
extensions can still round-trip and be repaired.

The validator is XSD-derived and clause-level. It covers ISO 7817-3-owned
structures, sequences, choices, cardinalities, enumerations, scalar lexemes, and
required attributes. DT GUID, language, date/time, and decimal contracts are
checked through `openbim-dt`. Imported ISO 23387 complex-type internals are
retained and deliberately not presented as completely XSD-validated. See
[`xml.md`](xml.md) for the exact guarantee.

## Workspace independence

Package version, edition, MSRV, license, authors, repository, and
cross-repository dependency versions are explicit in each published manifest.
The parent integration workspace substitutes local `openbim-dt` through
`[patch.crates-io]`, guaranteeing one package identity while exercising exact
pinned child commits.

## Standards artifacts

This repository does not track ISO, DIN, CEN, or other restricted standards
material. Local references live under ignored `references/`. Conformance
fixtures require known redistribution rights before admission.

## Cross-repository delivery

Changes spanning repositories follow dependency order:

1. land and publish lower-level contract changes;
2. update and verify `openbim-loin` standalone;
3. publish `openbim-loin`;
4. publish the exact-version `loin` alias;
5. update and verify the `openbim` submodule pin;
6. publish the integration commit.

The superproject pin is the compatibility declaration and rollback point.

## Releases

Versions are lockstep: one `vX.Y.Z` tag means every crate and the npm manifest
carry `X.Y.Z` ([ADR 0002](adr/0002-staged-npm-publish-and-lockstep-releases.md)).
The tag workflow re-runs the gate on the tagged commit, cuts the GitHub release
from `CHANGELOG.md`, and stages the npm package for manual 2FA approval. Rust
crates are published with `cargo publish` in dependency order.
