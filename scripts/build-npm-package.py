#!/usr/bin/env python3
"""Assemble the publishable npm package from the wasm-bindgen output.

Mirrors exactly what the gate verifies: the manifest lists only files the
generator actually emits, so a broken tarball fails here rather than after
it reaches the registry, where a version can never be reused.
"""

from __future__ import annotations

import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CRATE = ROOT / "openbim-loin-wasm"
MANIFEST = CRATE / "npm" / "package.json"


def run(command: list[str], cwd: pathlib.Path | None = None) -> None:
    subprocess.run(command, cwd=cwd, check=True)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: build-npm-package.py <output-dir>", file=sys.stderr)
        return 2
    out = ROOT / argv[1]
    if out.exists():
        shutil.rmtree(out)
    package = out / "package"
    package.mkdir(parents=True)

    run(
        [
            "cargo",
            "build",
            "-p",
            "openbim-loin-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ],
        cwd=ROOT,
    )
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    target_root = pathlib.Path(metadata["target_directory"])
    wasm = target_root / "wasm32-unknown-unknown" / "release" / "openbim_loin_wasm.wasm"
    if not wasm.is_file():
        print(f"missing build artifact: {wasm}", file=sys.stderr)
        return 1

    run(["wasm-bindgen", "--target", "nodejs", "--out-dir", str(package), str(wasm)])

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    shutil.copy2(MANIFEST, package / "package.json")
    shutil.copy2(ROOT / "LICENSE", package / "LICENSE")
    shutil.copy2(CRATE / "README.md", package / "README.md")

    for name in [*manifest["files"], manifest["main"], manifest["types"]]:
        if not (package / name).is_file():
            print(f"npm manifest lists a missing file: {name}", file=sys.stderr)
            return 1

    run(["npm", "pack", "--pack-destination", str(out)], cwd=package)
    print(f"built npm package {manifest['name']}@{manifest['version']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
