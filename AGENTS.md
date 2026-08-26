# LOIN repository instructions

This repository owns the OpenBIM.rs implementation of ISO 7817-3 / EN 17412-3
Level of Information Need and its short-name package alias. The crate source
implements namespace contracts and the DT-backed domain boundary. Do not
describe complete LOIN XML parsing, validation, migration, or writing as
implemented without executable evidence.

## Map

- `openbim-loin/` — canonical implementation and all public definitions
- `loin/` — pure re-export alias; no implementation or independent types
- `docs/` — repository architecture and maintained documentation
- `scripts/gate.sh` — complete local/CI verification gate
- `CHANGELOG.md` — user-visible changes using Keep a Changelog
- `references/` — ignored local standards corpus; never publish implicitly

## Commands

```bash
./scripts/gate.sh
cargo test --workspace
cargo package -p openbim-loin
cargo package -p loin
```

Trust command exit codes. Never summarize a Cargo pipeline in a way that hides
the Cargo process status.

## Boundaries

- `openbim-loin` may depend on released core, data-template, and XML contracts.
- IFC, core, codec, and data-template crates must never depend on LOIN.
- Namespace migration remains a LOIN concern and must preserve which source
  namespace was observed.
- `loin/src/lib.rs` must contain only `pub use openbim_loin::*;`.
- Cross-repository dependency versions are explicit in crate manifests; do not
  replace them with parent-workspace inheritance.
- Do not vendor ISO, DIN, CEN, or other restricted standards material without
  verified redistribution rights.

## Documentation discipline

Keep capability tables honest: distinguish constants and recognition helpers,
implemented algorithms, and conformance-tested behavior. Update README, rustdoc,
and `CHANGELOG.md` together for user-visible changes.
