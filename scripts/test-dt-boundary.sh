#!/usr/bin/env bash
# Prove that compile-time API tests detect replacement of DT-owned types.
set -euo pipefail

cd "$(dirname "$0")/.."
model=openbim-loin/src/model.rs
backup=$(mktemp)
log=$(mktemp)
cp "$model" "$backup"
cleanup() {
    cp "$backup" "$model"
    rm -f "$backup" "$log"
}
trap cleanup EXIT INT TERM

python3 - "$model" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
replacements = {
    "properties: Vec<Property>": "properties: Vec<Concept>",
    "value: Property": "value: Concept",
    "-> &[Property]": "-> &[Concept]",
}
for old, new in replacements.items():
    if old not in text:
        raise SystemExit(f"DT boundary mutation fixture missing: {old}")
    text = text.replace(old, new)
path.write_text(text, encoding="utf-8")
PY

cargo_args=()
if [[ -n ${OPENBIM_DT_PATCH_PATH:-} ]]; then
    cargo_args+=(--config "patch.crates-io.openbim-dt.path=\"${OPENBIM_DT_PATCH_PATH}\"")
fi
if cargo "${cargo_args[@]}" test -p openbim-loin --test dt_contracts --locked >"$log" 2>&1; then
    printf 'DT boundary mutation escaped compile-time API tests\n' >&2
    cat "$log" >&2
    exit 1
fi
printf 'mutation killed: dt-owned-types\n'
