#!/usr/bin/env python3
"""Verify a release tag agrees with the versions in the repository.

A tag is the only human input to the release workflow, so it is the only
place a mistake can silently ship the wrong artifact: without this check a
`v0.4.0` tag would publish whatever version the manifests happen to hold.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WASM_MANIFEST = ROOT / "openbim-loin-wasm" / "Cargo.toml"
NPM_MANIFEST = ROOT / "openbim-loin-wasm" / "npm" / "package.json"


def crate_version(manifest: pathlib.Path) -> str:
    for line in manifest.read_text(encoding="utf-8").splitlines():
        if line.startswith("version = "):
            return line.split('"')[1]
    raise SystemExit(f"no version in {manifest}")


def npm_version(manifest: pathlib.Path) -> str:
    match = re.search(
        r'"version"\s*:\s*"([^"]+)"', manifest.read_text(encoding="utf-8")
    )
    if match is None:
        raise SystemExit(f"no version in {manifest}")
    return match.group(1)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check-release-version.py <tag>", file=sys.stderr)
        return 2
    tag = argv[1]
    if not tag.startswith("v"):
        print(f"tag {tag!r} does not start with 'v'", file=sys.stderr)
        return 1
    expected = tag[1:]

    crate = crate_version(WASM_MANIFEST)
    npm = npm_version(NPM_MANIFEST)

    failures = []
    if crate != expected:
        failures.append(f"openbim-loin-wasm/Cargo.toml is {crate}")
    if npm != expected:
        failures.append(f"npm/package.json is {npm}")

    if failures:
        print(f"tag {tag} expects version {expected}, but:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(f"tag {tag} matches crate and npm version {expected}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
