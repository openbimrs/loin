#!/usr/bin/env python3
"""Consume the published grammar the way a third-party tool would.

Reads only `openbim-loin/loin-grammar.json` (never the Rust source) and walks
every `openbim-loin/examples/*.xml` document with it: element placement by
(parent, name), qualification, declared and required attributes, child
cardinality, sequence order, repeating-choice minimum, inherited ISO 23387
children, xsi:nil and enumeration values. The Rust drift test proves the
artifact matches the validator; this proves the artifact alone is enough to
check a document, and that it agrees with the validator on the examples.

Then it mutates the examples and requires the walker to reject each mutant,
so a walker that accepts everything cannot pass.
"""

from __future__ import annotations

import json
import pathlib
import sys
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parent.parent
CRATE = ROOT / "openbim-loin"


class Walker:
    def __init__(self, grammar: dict) -> None:
        assert grammar["format"] == "openbim-loin-grammar", grammar["format"]
        assert grammar["formatVersion"] == 1, grammar["formatVersion"]
        self.grammar = grammar
        self.loin = set(grammar["namespaces"]["loin"])
        self.prefix = {
            grammar["namespaces"]["dt"]: "dt",
            grammar["namespaces"]["xsi"]: "xsi",
        }
        self.elements = {(e["parent"], e["name"]): e for e in grammar["elements"]}
        self.checked = 0

    @staticmethod
    def split(tag: str) -> tuple[str | None, str]:
        if tag.startswith("{"):
            uri, local = tag[1:].split("}", 1)
            return uri, local
        return None, tag

    def walk(self, root: ET.Element) -> list[str]:
        errors: list[str] = []
        uri, name = self.split(root.tag)
        if uri not in self.loin or name != self.grammar["root"]:
            return [f"root {root.tag} is not a LOIN root"]
        self.visit(None, root, f"/{name}", errors)
        return errors

    def visit(self, parent: str | None, node: ET.Element, path: str, errors: list[str]) -> None:
        uri, name = self.split(node.tag)
        declaration = self.elements.get((parent, name))
        if declaration is None:
            errors.append(f"{path}: {name} is not declared below {parent}")
            return
        if parent is not None and uri is not None:
            errors.append(f"{path}: schema-local element must be unqualified")
        self.checked += 1
        if declaration["content"] == "imported":
            return

        declared = {(a["namespace"], a["name"]): a for a in declaration.get("attributes", [])}
        declared.update({(a["namespace"], a["name"]): a for a in self.grammar["globalAttributes"]})
        present = set()
        for key, value in node.attrib.items():
            attribute_uri, local = self.split(key)
            namespace = self.prefix.get(attribute_uri) if attribute_uri else None
            if (namespace, local) not in declared:
                errors.append(f"{path}: undeclared attribute {namespace}:{local}")
            present.add((namespace, local))
        for key, attribute in declared.items():
            if attribute["required"] and key not in present:
                errors.append(f"{path}: missing required attribute {attribute['name']}")
        nil = node.attrib.get("{http://www.w3.org/2001/XMLSchema-instance}nil")
        if nil in ("true", "1"):
            if not declaration.get("nillable"):
                errors.append(f"{path}: not nillable")
            if len(node) or (node.text or "").strip():
                errors.append(f"{path}: nilled element has content")
            return

        children = list(node)
        if declaration["content"] == "simple":
            if children:
                errors.append(f"{path}: simple content has child elements")
            values = declaration.get("values")
            if values is not None and (node.text or "") not in values:
                errors.append(f"{path}: {node.text!r} is not one of {values}")
            return

        inherited = declaration.get("inheritedDtChildren", [])
        dt_uri = self.grammar["namespaces"]["dt"]
        leading = 0
        while leading < len(children) and self.split(children[leading].tag)[0] == dt_uri:
            if self.split(children[leading].tag)[1] not in inherited:
                errors.append(f"{path}: undeclared inherited child {children[leading].tag}")
            leading += 1
        if inherited and leading == 0:
            errors.append(f"{path}: requires inherited ISO 23387 content")

        rules = declaration["children"]
        order = [rule["name"] for rule in rules]
        counts = dict.fromkeys(order, 0)
        last = 0
        for position, child in enumerate(children[leading:], start=leading + 1):
            _, child_name = self.split(child.tag)
            child_path = f"{path}/{child_name}[{position}]"
            if child_name not in counts:
                errors.append(f"{child_path}: not allowed below {name}")
                continue
            index = order.index(child_name)
            if declaration["content"] == "sequence" and index < last:
                errors.append(f"{child_path}: out of sequence order")
            last = max(last, index)
            counts[child_name] += 1
            self.visit(name, child, child_path, errors)
        for rule in rules:
            count = counts[rule["name"]]
            if count < rule["min"]:
                errors.append(f"{path}: needs at least {rule['min']} {rule['name']}")
            if rule["max"] is not None and count > rule["max"]:
                errors.append(f"{path}: at most {rule['max']} {rule['name']}")
        if declaration["content"] == "choice" and sum(counts.values()) == 0:
            errors.append(f"{path}: choice needs at least one item")


MUTANTS = [
    # (example, old, new, what the walker must notice)
    ("site-georeferencing.loin.xml", "<Language>en</Language>", "<Lang>en</Lang>", "undeclared element"),
    ("site-georeferencing.loin.xml", ' name="Site survey handover"', "", "missing required attribute"),
    ("site-georeferencing.loin.xml", ' Date="', ' date="', "attribute name is case-sensitive"),
    ("site-georeferencing.loin.xml", "<Type>ProjectedCRS</Type>", "<Type>Projected</Type>", "enumeration"),
    ("office-fit-out.loin.xml",
     "<Dimensionality>3D</Dimensionality>\n        <Appearance>SymbolicByMapping</Appearance>",
     "<Appearance>SymbolicByMapping</Appearance>\n        <Dimensionality>3D</Dimensionality>",
     "sequence order"),
    ("deferred-object-type.loin.xml", 'dt:GUID="3c9d0e21-6f7a-4b8c-9d0e-1f2a3b4c5d20" dateOfCreation',
     'dateOfCreation', "missing dt:GUID on nilled element"),
    ("office-fit-out.loin.xml", "<dt:Name language=\"en\">Internal doors for cost planning</dt:Name>\n", "",
     "per-object specification without inherited content"),
]


def mutants(examples: dict[str, str]) -> list[tuple[str, str]]:
    out = []
    for name, old, new, why in MUTANTS:
        text = examples[name]
        if text.count(old) != 1:
            raise SystemExit(f"mutant anchor for {name} ({why}) matched {text.count(old)} times")
        out.append((f"{name}: {why}", text.replace(old, new, 1)))
    return out


def main() -> int:
    grammar = json.loads((CRATE / "loin-grammar.json").read_text(encoding="utf-8"))
    walker = Walker(grammar)
    paths = sorted((CRATE / "examples").glob("*.xml"))
    if not paths:
        print("no examples found", file=sys.stderr)
        return 1
    examples = {path.name: path.read_text(encoding="utf-8") for path in paths}
    failed = False
    for name, text in examples.items():
        errors = walker.walk(ET.fromstring(text.encode("utf-8")))
        for error in errors:
            print(f"{name}: {error}", file=sys.stderr)
        failed |= bool(errors)
    if failed:
        return 1
    checked = walker.checked
    if checked < 140:
        print(f"only {checked} elements checked", file=sys.stderr)
        return 1

    for label, text in mutants(examples):
        if not walker.walk(ET.fromstring(text.encode("utf-8"))):
            print(f"mutant survived: {label}", file=sys.stderr)
            return 1
        print(f"[grammar] mutant rejected: {label}")
    print(
        f"grammar artifact OK: {len(examples)} examples, {checked} elements checked, "
        f"{len(MUTANTS)} mutants rejected"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
