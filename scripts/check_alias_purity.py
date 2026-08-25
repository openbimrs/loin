#!/usr/bin/env python3
"""Fail closed unless `loin` is a semantic pure alias of `openbim-loin`."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def fail(message: str) -> "NoReturn":
    print(f"alias purity: {message}", file=sys.stderr)
    raise SystemExit(1)


def package(packages: list[dict], name: str) -> dict:
    matches = [candidate for candidate in packages if candidate["name"] == name]
    if len(matches) != 1:
        fail(f"expected exactly one {name!r} package, found {len(matches)}")
    return matches[0]


def normalized(path: str | Path) -> Path:
    return Path(path).resolve()


metadata = json.loads(
    subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
)
packages = metadata["packages"]
canonical = package(packages, "openbim-loin")
alias = package(packages, "loin")

canonical_version = canonical["version"]
alias_version = alias["version"]
if alias_version != canonical_version:
    fail(
        f"package versions differ: loin={alias_version}, "
        f"openbim-loin={canonical_version}"
    )

expected_alias_manifest = normalized(ROOT / "loin/Cargo.toml")
if normalized(alias["manifest_path"]) != expected_alias_manifest:
    fail(f"loin manifest moved outside {expected_alias_manifest}")

if alias.get("features"):
    fail("loin must not define features")
if alias.get("links") is not None:
    fail("loin must not define a native links contract")

if len(alias["targets"]) != 1:
    fail("loin must contain exactly one Cargo target")
target = alias["targets"][0]
if target["kind"] != ["lib"] or target["crate_types"] != ["lib"]:
    fail("loin's only target must be a normal library")
if target["name"] != "loin":
    fail(f"loin library target has unexpected name {target['name']!r}")

source_path = normalized(target["src_path"])
expected_source_path = normalized(ROOT / "loin/src/lib.rs")
if source_path != expected_source_path:
    fail(f"loin library target must be {expected_source_path}, got {source_path}")

source_root = ROOT / "loin/src"
source_entries = {
    path.relative_to(source_root)
    for path in source_root.rglob("*")
    if path.is_file() or path.is_symlink()
}
if source_entries != {Path("lib.rs")}:
    fail(
        "loin source tree must contain only src/lib.rs, got "
        + ", ".join(str(path) for path in sorted(source_entries))
    )

meaningful_lines = [
    line.strip()
    for line in source_path.read_text(encoding="utf-8").splitlines()
    if line.strip() and not line.lstrip().startswith("//")
]
if meaningful_lines != ["pub use openbim_loin::*;"]:
    fail("loin library must contain only `pub use openbim_loin::*;`")

dependencies = alias["dependencies"]
if len(dependencies) != 1:
    fail("loin must depend only on openbim-loin")
dependency = dependencies[0]
if dependency["name"] != "openbim-loin" or dependency.get("rename") is not None:
    fail("loin's sole dependency must be the unrenamed openbim-loin package")
if dependency.get("kind") is not None or dependency.get("optional"):
    fail("openbim-loin must be a required normal dependency")
if dependency.get("target") is not None:
    fail("openbim-loin dependency must apply on every target")
if dependency.get("features"):
    fail("openbim-loin dependency must not override canonical features")
if dependency.get("uses_default_features") is not True:
    fail("openbim-loin dependency must retain canonical default features")
expected_requirement = f"={canonical_version}"
if dependency["req"] != expected_requirement:
    fail(
        f"openbim-loin requirement must be {expected_requirement}, "
        f"got {dependency['req']}"
    )
expected_dependency_path = normalized(ROOT / "openbim-loin")
if dependency.get("path") is None:
    fail("openbim-loin must be a local path dependency for workspace validation")
if normalized(dependency["path"]) != expected_dependency_path:
    fail(
        f"openbim-loin path must resolve to {expected_dependency_path}, "
        f"got {dependency['path']}"
    )
