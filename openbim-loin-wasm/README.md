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

```ts
validate(xml: string): Diagnostic[]   // throws LoinParseError
rewrite(xml: string): string          // throws LoinParseError | LoinWriteError
migrate(xml: string, target: NamespaceVersion): Migration
isWellFormed(xml: string): boolean
```

Everything crossing the boundary is machine-readable. A `Diagnostic` is
`{severity, code, path, message}`; a `LoinParseError` additionally carries
`kind` and `position` as real properties, so an editor can place a marker
at the byte offset without regexing the message.

```js
try { validate(xml); } catch (e) {
  if (e.name === "LoinParseError") marker(e.position, e.kind);
}
```

`code`, `severity` and `kind` strings are an explicit, exhaustive mapping
in `src/lib.rs`, not derived from Rust's `Debug`. A variant rename upstream
breaks this crate's build rather than silently changing the JS contract.

### Namespace migration

`migrate` moves a document between LOIN namespace versions (`Draft2022`,
`Draft2024`) and reports what changed:

```js
const { xml, report } = migrate(input, "Draft2024");
// report: { source, target, changedNames, changedDeclarations }
```

Migrating to the version already in use is a no-op that still returns a
report with both counts zero. An unknown target throws `TypeError` rather
than falling back to a default. When a migration would make two attributes
share one expanded name it throws `LoinMigrationError`, carrying `element`
and `localName` so the caller can point at the collision.

## Build

```sh
cargo build -p openbim-loin-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target nodejs --out-dir pkg \
  target/wasm32-unknown-unknown/release/openbim_loin_wasm.wasm
```

`--target web` or `bundler` work equally; `nodejs` is what the gate uses.

`npm/package.json` is the npm manifest: copy it next to the generated
files and `npm pack` to produce a publishable `@openbimrs/loin` tarball.

## Verification

`scripts/test-wasm-package.sh` builds the module, generates the package,
asserts the payload was not stripped, and runs `tests/node_smoke.js` under
Node. It is part of the repository gate.

## License

MIT — see [`LICENSE`](../LICENSE).
