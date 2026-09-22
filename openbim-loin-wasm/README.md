# openbim-loin-wasm

Browser and Node bindings for [`openbim-loin`](../openbim-loin): parse,
validate and rewrite ISO 7817-3 LOIN documents in a JS runtime.

## Why a separate crate

`wasm-bindgen` output requires `crate-type = ["cdylib"]`, which is a
per-crate manifest key and cannot be toggled by a Cargo feature. Putting
these bindings behind a feature on `openbim-loin` would therefore emit a
dynamic library for every native build, and would put `wasm-bindgen`,
`js-sys` and `serde` in the dependency tree of consumers that never touch
a browser. The core crate stays dependency-light; this crate pays for JS.

## API

- `validate(xml)` -> array of `{severity, code, path, message}`
- `rewrite(xml)` -> string, re-serialised preserving the observed namespace
- `isWellFormed(xml)` -> boolean

Parse failures throw a JS `Error`. Diagnostics are structured objects, so
callers branch on `code` rather than parsing message text.

## Build

```sh
cargo build -p openbim-loin-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target nodejs --out-dir pkg \
  target/wasm32-unknown-unknown/release/openbim_loin_wasm.wasm
```

`--target web` or `bundler` work equally; `nodejs` is what the gate uses.

## Verification

`scripts/test-wasm-package.sh` builds the module, generates the package,
asserts the payload was not stripped, and runs `tests/node_smoke.js` under
Node. It is part of the repository gate.

## License

MIT — see [`LICENSE`](../LICENSE).
