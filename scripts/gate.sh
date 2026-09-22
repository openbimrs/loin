#!/usr/bin/env bash
# Complete standalone verification gate for openbimrs/loin.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo_args=()
if [[ -n ${OPENBIM_DT_PATCH_PATH:-} ]]; then
    [[ ${OPENBIM_DT_PATCH_PATH} != *'"'* && ${OPENBIM_DT_PATCH_PATH} != *$'\n'* ]] || {
        printf 'OPENBIM_DT_PATCH_PATH contains an unsupported character\n' >&2
        exit 1
    }
    [[ ! -e .cargo/config.toml ]] || {
        printf 'refusing to overwrite existing .cargo/config.toml\n' >&2
        exit 1
    }
    mkdir -p .cargo
    printf '[patch.crates-io]\nopenbim-dt = { path = "%s" }\n' \
        "$OPENBIM_DT_PATCH_PATH" >.cargo/config.toml
    cleanup_patch_config() {
        rm -f .cargo/config.toml
        rmdir .cargo 2>/dev/null || true
    }
    trap cleanup_patch_config EXIT INT TERM
fi

cargo fmt --all -- --check
bash -n scripts/test-dt-boundary.sh
bash -n scripts/test-schema-shape.sh
cargo "${cargo_args[@]}" build --workspace --all-targets --locked
cargo "${cargo_args[@]}" test --workspace --all-features --locked
cargo "${cargo_args[@]}" clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo "${cargo_args[@]}" doc --workspace --all-features --no-deps --locked
scripts/check-alias-purity.sh
python3 scripts/test_alias_purity.py
python3 - <<'PY'
import json
import os
import subprocess

command = ["cargo"]
if patch_path := os.environ.get("OPENBIM_DT_PATCH_PATH"):
    command.extend(["--config", f'patch.crates-io.openbim-dt.path="{patch_path}"'])
command.extend(["metadata", "--no-deps", "--locked", "--format-version", "1"])
metadata = json.loads(subprocess.run(
    command,
    check=True,
    capture_output=True,
    text=True,
).stdout)
packages = {package["name"]: package for package in metadata["packages"]}
canonical = packages["openbim-loin"]
alias = packages["loin"]
version = canonical["version"]
assert alias["version"] == version, (alias["version"], version)
deps = {dependency["name"]: dependency for dependency in canonical["dependencies"]}
assert set(deps) == {"openbim-dt", "quick-xml", "roxmltree"}, deps
assert deps["openbim-dt"]["req"] == "^0.2.0", deps["openbim-dt"]
assert deps["openbim-dt"].get("path") is None, deps["openbim-dt"]
assert deps["quick-xml"]["req"] == "^0.41.0", deps["quick-xml"]
assert deps["roxmltree"]["req"] == "^0.21.1", deps["roxmltree"]
alias_dep = alias["dependencies"]
assert len(alias_dep) == 1 and alias_dep[0]["name"] == "openbim-loin", alias_dep
assert alias_dep[0]["req"] == f"={version}", alias_dep[0]
PY
cargo "${cargo_args[@]}" package -p openbim-loin --locked --allow-dirty
printf 'alias package verification deferred until the canonical crate is registry-visible\\n'
./scripts/test-dt-boundary.sh
./scripts/test-schema-shape.sh
python3 scripts/test-xml-capability.py
python3 scripts/test-authoring-mutations.py
