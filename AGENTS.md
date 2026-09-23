# LOIN repository instructions

This repository owns the OpenBIM.rs implementation of ISO 7817-3 / EN 17412-3
Level of Information Need, its short-name package alias, and its browser
bindings. Parsing, validation, namespace migration and writing are implemented
and gate-verified; see the README status table for exact scope. Keep claims
backed by executable evidence: writing from the typed model covers LOIN-owned
content only, and there is no document-to-model reader yet (see
`PLAN-model-io.md`).

## Map

- `openbim-loin/` — canonical implementation and all public definitions
- `loin/` — pure re-export alias; no implementation or independent types
- `openbim-loin-wasm/` — wasm-bindgen browser/Node bindings; the only crate
  that may depend on wasm/JS crates. `npm/package.json` is the `@openbim/loin`
  manifest
- `docs/` — repository architecture and maintained documentation
- `scripts/gate.sh` — complete local/CI verification gate; every evidence
  script must print its completion marker (`scripts/check-evidence.py`)
- `.github/workflows/release.yml` — `v*` tag: lockstep version check, gate,
  GitHub release, then `npm stage publish` awaiting 2FA approval
- `docs/adr/` — accepted decisions; read before re-proposing an alternative
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

Pitfalls:

- Never commit a `Cargo.lock` produced while `OPENBIM_DT_PATCH_PATH` was set;
  it records `openbim-dt` as a path and breaks `--locked` for everyone else.
  The gate rejects it (`scripts/check-lockfile-sources.py`).
- Set `LOIN_WASM_STRICT=1` to make missing wasm tooling fail rather than skip.
  CI sets it; a local run without the toolchain skips and says so.
- Releases are lockstep: every crate and the npm manifest carry the tag's
  version (`scripts/check-release-version.py`).

## Boundaries

- `openbim-loin` may depend on released core, data-template, and XML contracts.
- IFC, core, codec, and data-template crates must never depend on LOIN.
- Namespace migration remains a LOIN concern and must preserve which source
  namespace was observed.
- `loin/src/lib.rs` must contain only `pub use openbim_loin::*;`.
- `openbim-loin` must never depend on wasm/JS crates; the gate pins its exact
  dependency set.
- Cross-repository dependency versions are explicit in crate manifests; do not
  replace them with parent-workspace inheritance.
- Do not vendor ISO, DIN, CEN, or other restricted standards material without
  verified redistribution rights.

## Documentation discipline

Keep capability tables honest: distinguish constants and recognition helpers,
implemented algorithms, and conformance-tested behavior. Update README, rustdoc,
and `CHANGELOG.md` together for user-visible changes.
