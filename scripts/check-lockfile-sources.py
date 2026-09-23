#!/usr/bin/env python3
"""Reject a Cargo.lock that resolves a registry dependency to a local path.

Generating the lockfile while OPENBIM_DT_PATCH_PATH is set records openbim-dt
without a `source` line. The build still works locally, but CI resolves from
crates.io and fails `--locked`. Only workspace members may lack a source.
"""

import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent


def main() -> int:
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    members = set()
    for member in workspace["workspace"]["members"]:
        manifest = tomllib.loads((ROOT / member / "Cargo.toml").read_text(encoding="utf-8"))
        members.add(manifest["package"]["name"])
    sourceless = sorted(
        f"{p['name']} {p['version']}"
        for p in lock["package"]
        if "source" not in p and p["name"] not in members
    )
    if sourceless:
        print("Cargo.lock resolves non-workspace packages to local paths:", file=sys.stderr)
        for item in sourceless:
            print(f"  - {item}", file=sys.stderr)
        print("regenerate it without OPENBIM_DT_PATCH_PATH set", file=sys.stderr)
        return 1
    print(f"lockfile sources OK ({len(lock['package'])} packages)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
