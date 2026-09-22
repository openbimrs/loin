#!/usr/bin/env python3
"""Extract one version's section from CHANGELOG.md for release notes.

Keeps the published release notes and the changelog from drifting: there is
one source of truth, and it is the file already under review.
"""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CHANGELOG = ROOT / "CHANGELOG.md"


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: changelog-section.py <tag>", file=sys.stderr)
        return 2
    version = argv[1][1:] if argv[1].startswith("v") else argv[1]

    lines = CHANGELOG.read_text(encoding="utf-8").splitlines()
    heading = f"## [{version}]"
    start = None
    for index, line in enumerate(lines):
        if line.startswith(heading):
            start = index + 1
            break
    if start is None:
        print(f"no CHANGELOG section for {version}", file=sys.stderr)
        return 1

    end = start
    while end < len(lines) and not lines[end].startswith("## ["):
        end += 1

    body = "\n".join(lines[start:end]).strip()
    if not body:
        print(f"CHANGELOG section for {version} is empty", file=sys.stderr)
        return 1
    print(body)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
