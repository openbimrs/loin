#!/usr/bin/env bash
# Prove that DT-owned property content is retained by the typed boundary.
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
model_rel=openbim-loin/src/model.rs
log=$(mktemp)
baseline=$(mktemp -d "${TMPDIR:-/tmp}/loin-dt-baseline-XXXXXX")
sandbox=$(mktemp -d "${TMPDIR:-/tmp}/loin-dt-mutant-XXXXXX")
python3 - "$repo_root" "$baseline/repo" <<'PY'
from pathlib import Path
import shutil, sys
shutil.copytree(Path(sys.argv[1]), Path(sys.argv[2]), ignore=shutil.ignore_patterns(".git", "target", ".hermes", "references", "*.xsd", "*.xsd.xml"))
PY
chmod -R u+w "$baseline/repo"
cp -a "$baseline/repo" "$sandbox/repo"
model="$sandbox/repo/$model_rel"
cleanup() {
    rm -rf "$baseline" "$sandbox"
    rm -f "$log"
}
trap cleanup EXIT INT TERM

python3 - "$model" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
old = "        self.properties.push(value);"
new = "        let _ = value;"
text = path.read_text(encoding="utf-8")
count = text.count(old)
if count != 1:
    raise SystemExit(f"DT boundary mutation fixture count: {count}")
path.write_text(text.replace(old, new), encoding="utf-8")
PY

target="${CARGO_TARGET_DIR:-target}/loin-dt-boundary-mutation"
if ! (cd "$sandbox/repo" && CARGO_TARGET_DIR="$target" cargo check -p openbim-loin --locked >"$log" 2>&1); then
    printf 'DT boundary mutation did not compile\n' >&2
    cat "$log" >&2
    exit 1
fi
printf 'mutation compile passed: dt-owned-property-retention (CARGO_TARGET_DIR=%s)\n' "$target"
if (cd "$sandbox/repo" && CARGO_TARGET_DIR="$target" cargo test -p openbim-loin --test dt_contracts --locked specification_uses_dt_owned_iso_23387_types_at_every_imported_boundary >"$log" 2>&1); then
    printf 'DT boundary mutation escaped typed API tests\n' >&2
    cat "$log" >&2
    exit 1
fi
printf 'mutation killed: dt-owned-property-retention\n'
