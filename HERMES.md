# OpenBIM.rs LOIN

Canonical repository: <https://github.com/openbimrs/loin>
Integration repository: <https://github.com/openbimrs/openbim>

Read `AGENTS.md` before changing the repository and the nested `AGENTS.md`
before editing a crate. Keep both packages independently buildable; the parent
OpenBIM.rs workspace pins this repository as a submodule but is not required for
standalone development.

## Verification

Run `./scripts/gate.sh`. It is the authoritative local and CI gate and decides
success from command exit codes.

## Project conventions

- Rust 2021, MSRV 1.88, Python 3.10+ for gate scripts, MIT.
- Pure Rust; unsafe code is forbidden.
- `openbim-loin` owns every implementation and type.
- `loin` is a pure, exact-version re-export alias and defines no types.
- LOIN consumes data-template contracts directly; add core or XML dependencies
  only when implementation source uses them. Lower layers never depend on LOIN.
- Namespace versioning is explicit because existing drafts use different URIs.
- Never commit standards PDFs, schemas, or other artifacts without confirmed
  redistribution rights. Local material belongs under ignored `references/`.
- Use Keep a Changelog and distinguish implemented from reserved capabilities.
