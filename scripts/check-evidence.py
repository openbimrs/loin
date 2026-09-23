#!/usr/bin/env python3
"""Require every evidence script to have reported completion.

An evidence script that exits 0 without doing its work is indistinguishable
from one that passed, unless something checks what it actually printed. The
gate tees all evidence output into one file; this asserts each expected
completion marker is present, so a silent no-op fails the gate.
"""

import os
import sys

# (owner, marker). Every marker is a line the owning script prints only after
# all of its assertions ran.
REQUIRED = [
    ("test_alias_purity.py", "alias purity mutations passed"),
    ("test-dt-boundary.sh", "mutation killed: dt-owned-property-retention"),
    ("test-schema-shape.sh", "all typed-model mutations killed"),
    ("test-xml-capability.py", "XML capability mutations killed"),
    ("test-authoring-mutations.py", "leaked=0"),
]

WASM = [
    "wasm package OK:",
    "structured errors OK:",
    "migration OK:",
    "npm manifest OK:",
]
WASM_SKIP = "skipping wasm package test:"


def main() -> int:
    text = open(sys.argv[1], encoding="utf-8").read()
    missing = [f"{owner}: {marker!r}" for owner, marker in REQUIRED if marker not in text]
    strict = os.environ.get("LOIN_WASM_STRICT") == "1"
    wasm_missing = [m for m in WASM if m not in text]
    if wasm_missing:
        if strict or WASM_SKIP not in text:
            missing += [f"test-wasm-package.sh: {m!r}" for m in wasm_missing]
        else:
            print("evidence: wasm checks skipped locally (not strict)")
    if missing:
        print("evidence incomplete; these scripts did not report completion:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 1
    print(f"evidence complete: {len(REQUIRED)} scripts + wasm")
    return 0


if __name__ == "__main__":
    sys.exit(main())
