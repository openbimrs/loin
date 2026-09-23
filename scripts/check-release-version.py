#!/usr/bin/env python3
"""Verify a release tag agrees with every version in the repository.

Releases here are lockstep: one `vX.Y.Z` tag means every published artifact
(each workspace crate and the npm package) is exactly `X.Y.Z`. That makes a
tag unambiguous: it names one commit and one version, never "the wasm crate
moved but the core did not". A tag is the only human input to the release
workflow, so this is the one place a mismatch can be caught before it ships.
"""

from __future__ import annotations

import json
import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
NPM_MANIFEST = ROOT / "openbim-loin-wasm" / "npm" / "package.json"


def versions() -> dict[str, str]:
    """Every releasable version, keyed by a human-readable source."""
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    found = {}
    for member in workspace["workspace"]["members"]:
        path = ROOT / member / "Cargo.toml"
        package = tomllib.loads(path.read_text(encoding="utf-8"))["package"]
        found[f"{member}/Cargo.toml ({package['name']})"] = package["version"]
    npm = json.loads(NPM_MANIFEST.read_text(encoding="utf-8"))
    found[f"npm/package.json ({npm['name']})"] = npm["version"]
    return found


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check-release-version.py <tag>", file=sys.stderr)
        return 2
    tag = argv[1]
    if not tag.startswith("v"):
        print(f"tag {tag!r} does not start with 'v'", file=sys.stderr)
        return 1
    expected = tag[1:]
    found = versions()
    wrong = {source: v for source, v in found.items() if v != expected}
    if wrong:
        print(f"tag {tag} expects every artifact at {expected}, but:", file=sys.stderr)
        for source, version in wrong.items():
            print(f"  - {source} is {version}", file=sys.stderr)
        print("releases are lockstep: bump every crate and the npm manifest together",
              file=sys.stderr)
        return 1
    print(f"tag {tag} matches all {len(found)} artifacts at {expected}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
