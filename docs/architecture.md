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
loin  -- exact-version dependency -->  openbim-loin  -->  openbim-core
(alias; no items)                       (all behavior)
```

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

- LOIN may consume shared vocabulary, ISO 23387 data-template contracts, and XML
  encoding infrastructure.
- IFC, core, codec, and data-template crates must never depend on LOIN.
- The `openbim` facade may optionally re-export LOIN.
- A future IFC mapping belongs in an explicit bridge, not in the LOIN data model.

## Namespace contract

Known public drafts declare different namespaces:

- `https://iso.org/2024/LOIN`
- `https://iso.org/2022/LOIN`

The current crate only exposes and recognizes these constants. Future decoding
must preserve the observed source namespace; future writing must require an
explicit target namespace. Recognition is not validation, and no namespace
should be called final until the published standard establishes it.

## Workspace independence

Package version, edition, MSRV, license, authors, repository, and
cross-repository dependency versions are explicit in each published manifest.
The parent integration workspace substitutes its local `openbim-core` through a
`[patch.crates-io]` entry, guaranteeing one package identity while exercising
the exact pinned child commit.

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
