#!/usr/bin/env bash
# Builds openbim-loin-wasm for wasm32, generates the wasm-bindgen package and
# runs it under Node. A compiling wasm binary proves nothing on its own: the
# linker happily strips an unused cdylib to a few hundred bytes, so this
# asserts the exported functions actually execute and return correct data.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v wasm-bindgen >/dev/null; then
    printf 'wasm-bindgen CLI not installed; skipping wasm package test\n' >&2
    exit 0
fi
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    printf 'wasm32-unknown-unknown target missing; skipping wasm package test\n' >&2
    exit 0
fi
if ! command -v node >/dev/null; then
    printf 'node not installed; skipping wasm package test\n' >&2
    exit 0
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
