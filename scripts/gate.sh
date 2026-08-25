#!/usr/bin/env bash
# Complete standalone verification gate for openbimrs/loin.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
scripts/check-alias-purity.sh
python3 scripts/test_alias_purity.py
cargo package -p openbim-loin
cargo package -p loin
