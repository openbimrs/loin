#!/usr/bin/env bash
# Prove that tests pin audited ISO 7817-3 cardinalities and imported scalar types.
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
    "date: Option<DateTime>": "date: Option<String>",
    "pub const fn new() -> Self": "pub const fn new(_guid: Guid) -> Self",
    "pub fn new(guid: Guid, name: impl Into<String>, prerequisites: Prerequisites) -> Self": "pub fn new(guid: Guid, name: impl Into<String>, prerequisites: Prerequisites, first: SpecificationPerObjectType) -> Self",
    "per_object: Vec::new()": "per_object: vec![first]",
    "pub const fn new(name: MultiLanguageText) -> Self": "pub fn new(name: MultiLanguageText, first_description: MultiLanguageText) -> Self",
    "Self {\n            name,\n            descriptions: Vec::new(),\n            registry_reference: None,\n        }": "Self {\n            name,\n            descriptions: vec![first_description],\n            registry_reference: None,\n        }",
}
for old, new in replacements.items():
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"schema mutation fixture count for {old!r}: {count}")
    text = text.replace(old, new)
path.write_text(text, encoding="utf-8")
PY

cargo_args=()
if [[ -n ${OPENBIM_DT_PATCH_PATH:-} ]]; then
    cargo_args+=(--config "patch.crates-io.openbim-dt.path=\"${OPENBIM_DT_PATCH_PATH}\"")
fi
if cargo "${cargo_args[@]}" test -p openbim-loin --test dt_contracts --locked >"$log" 2>&1; then
    printf 'ISO 7817 cardinality/type mutation escaped tests\n' >&2
    cat "$log" >&2
    exit 1
fi
printf 'mutation killed: loin-schema-shape\n'
