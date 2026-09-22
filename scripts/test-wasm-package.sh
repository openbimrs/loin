#!/usr/bin/env bash
# Builds openbim-loin-wasm for wasm32, generates the wasm-bindgen package and
# runs it under Node. A compiling wasm binary proves nothing on its own: the
# linker happily strips an unused cdylib to a few hundred bytes, so this
# asserts the exported functions actually execute and return correct data.
set -euo pipefail

cd "$(dirname "$0")/.."

# The tooling guards below let this degrade on a developer machine without the
# wasm toolchain. On CI that would be a silent bypass: the gate would report
# success having executed none of these assertions. Setting LOIN_WASM_STRICT=1
# turns every skip into a hard failure, and CI sets it.
strict="${LOIN_WASM_STRICT:-0}"

unavailable() {
    if [ "$strict" = "1" ]; then
        printf 'wasm package test cannot run: %s\n' "$1" >&2
        printf 'LOIN_WASM_STRICT=1 forbids skipping. Install the toolchain.\n' >&2
        exit 1
    fi
    printf 'skipping wasm package test: %s\n' "$1" >&2
    exit 0
}

if ! command -v wasm-bindgen >/dev/null; then
    unavailable 'wasm-bindgen CLI not installed'
fi
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    unavailable 'wasm32-unknown-unknown target missing'
fi
if ! command -v node >/dev/null; then
    unavailable 'node not installed'
fi

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

cargo build -p openbim-loin-wasm --target wasm32-unknown-unknown --release
target_dir="${CARGO_TARGET_DIR:-target}"
wasm="$target_dir/wasm32-unknown-unknown/release/openbim_loin_wasm.wasm"

wasm-bindgen --target nodejs --out-dir "$out" "$wasm"

# A stripped cdylib would still "build". Require a realistic payload.
size=$(stat -c%s "$out/openbim_loin_wasm_bg.wasm")
if [ "$size" -lt 50000 ]; then
    printf 'wasm payload suspiciously small (%s bytes): exports were stripped\n' "$size" >&2
    exit 1
fi

LOIN_WASM_PKG="$out" node openbim-loin-wasm/tests/node_smoke.js

# The npm manifest must stay in step with the crate version and must list
# files that the generator actually emits, or the published tarball is broken.
python3 - "$out" <<'PY'
import json
import pathlib
import sys

out = pathlib.Path(sys.argv[1])
manifest = json.loads(pathlib.Path("openbim-loin-wasm/npm/package.json").read_text())
cargo = pathlib.Path("openbim-loin-wasm/Cargo.toml").read_text()
version = next(
    line.split('"')[1] for line in cargo.splitlines() if line.startswith("version")
)
assert manifest["version"] == version, (manifest["version"], version)
for name in [*manifest["files"], manifest["main"], manifest["types"]]:
    assert (out / name).is_file(), f"npm manifest lists a missing file: {name}"
print("npm manifest OK:", manifest["name"], manifest["version"])
PY
