#!/usr/bin/env python3
"""Print the wasm-bindgen version recorded in Cargo.lock.

The wasm-bindgen CLI refuses to generate bindings unless its version matches
the crate exactly, so CI reads the version from the lockfile instead of
pinning it by hand where it would silently drift on the next dependency bump.
"""

from __future__ import annotations

import pathlib
import re
import sys

PATTERN = re.compile(
    r'\[\[package\]\]\nname = "wasm-bindgen"\nversion = "([^"]+)"'
)


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    match = PATTERN.search((root / "Cargo.lock").read_text(encoding="utf-8"))
    if match is None:
        print("wasm-bindgen not found in Cargo.lock", file=sys.stderr)
        return 1
    print(match.group(1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
