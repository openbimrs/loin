#!/usr/bin/env python3
"""Prove the authoring tests can actually fail.

Each mutation reintroduces a specific defect; the named test must fail.
A mutation that survives means the test is decoration.
"""
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "openbim-loin" / "src"

MUTATIONS = [
    (
        "issue4-leading-at-accepted",
        "model.rs",
        "    if at == 0 {\n        return false;\n    }",
        "    if false {\n        return false;\n    }",
        "email_pattern",
        "leading_at_is_rejected",
    ),
    (
        "issue5-setter-drops-duplicates",
        "model.rs",
        """        Ok(())
    }
    pub fn set_definition""",
        """        { let mut d = false; self.items.retain(|i| { if matches(i) { let f = !d; d = true; return f; } true }); }
        Ok(())
    }
    pub fn set_definition""",
        "purpose_choice",
        "set_language_preserves_repeated_branches",
    ),
    (
        "issue2-dt-language-unchecked",
        "validation.rs",
        "    validate_dt_multilingual_text(element, path, diagnostics);\n",
        "",
        "dt_language",
        "issue2_dt_multilingual_text_requires_language",
    ),
    (
        "issue6-parent-path-for-out-of-order",
        "validation.rs",
        """                DiagnosticCode::ChildOutOfOrder,
                &child_path,
                format!("{} occurs outside its XSD sequence position", child.qname()),""",
        """                DiagnosticCode::ChildOutOfOrder,
                path,
                format!("{} occurs outside its XSD sequence position", child.qname()),""",
        "dt_language",
        "issue6_child_out_of_order_reports_the_child_path",
    ),
    (
        "issue9-threshold-dimension-first",
        "model.rs",
        """    pub features: Option<Features>,
    pub threshold_dimension: Option<ThresholdDimension>,""",
        """    pub threshold_dimension: Option<ThresholdDimension>,
    pub features: Option<Features>,""",
        "schema_conformance",
        "shape_influence_field_order_matches_xsd_sequence",
    ),
    (
        "issue5-append-instead-of-replace",
        "model.rs",
        """        match self.items.iter().position(&matches) {
            Some(index) => match replacement {
                Some(replacement) => self.items[index] = replacement,
                None => {
                    self.items.remove(index);
                }
            },""",
        """        self.items.retain(|item| !matches(item));
        match None::<usize> {
            Some(_index) => unreachable!(),""",
        "authoring",
        "purpose_setters_replace_in_place_without_reordering",
    ),
    (
        "loin-namespace-as-default",
        "document.rs",
        """            .with_attribute(XmlAttribute::namespace_declaration(
                Some(LOIN_PREFIX),
                NAMESPACE_2024,
            ))""",
        """            .with_attribute(XmlAttribute::namespace_declaration(None, NAMESPACE_2024))""",
        "authoring",
        "authored_output_reparses_and_still_validates",
    ),
    (
        "qualified-schema-local-children",
        "document.rs",
        """    pub fn new(local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: local_name.clone(),
            prefix: None,
            local_name,
            namespace_uri: None,""",
        """    pub fn new(local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: local_name.clone(),
            prefix: None,
            local_name,
            namespace_uri: Some(Arc::from(NAMESPACE_2024)),""",
        "authoring",
        "authored_document_serializes_and_validates",
    ),
    (
        "dt-content-silently-dropped",
        "authoring.rs",
        '''    let _ = value;
    Err(unwritable("ObjectType"))''',
        '''    let _ = value;
    Ok(XmlElement::new("SpecificationPerObjectType"))''',
        "authoring",
        "dt_owned_content_is_refused_by_name",
    ),
    (
        "geo-referencing-dropped",
        "authoring.rs",
        "        element = element.with_child(geo_referencing_element(geo)?);",
        "        let _ = geo_referencing_element(geo)?;",
        "authoring",
        "georeferencing_is_written_and_validates",
    ),
    (
        "crs-type-misspelled",
        "authoring.rs",
        '        CoordinateReferenceSystemKind::ProjectedCrs => "ProjectedCRS",',
        '        CoordinateReferenceSystemKind::ProjectedCrs => "ProjectedCrs",',
        "authoring",
        "georeferencing_is_written_and_validates",
    ),
    (
        "registry-reference-silently-dropped",
        "authoring.rs",
        '        return Err(unwritable("RegistryReference"));',
        '        let _ = unwritable("RegistryReference");',
        "authoring",
        "georeferencing_registry_reference_is_refused_as_dt_content",
    ),
    (
        "enum-value-dropped",
        "model.rs",
        '''    MultipleObjects => "MultipleObjects",''',
        '''    MultipleObjects => "MultipleObjectz",''',
        "enumerations",
        "enumerations_match_the_xsd_and_round_trip",
    ),
    (
        "dimensionality-lexical-swapped",
        "model.rs",
        '''    ZeroD => "0D",''',
        '''    ZeroD => "ZeroD",''',
        "enumerations",
        "enumerations_match_the_xsd_and_round_trip",
    ),
    # --- reader (phase 4): each probe re-opens a silent-acceptance hole ---
    (
        "reader-text-accepted",
        "reader.rs",
        "        if node.has_text() {",
        "        if false && node.has_text() {",
        "reader",
        "invalid_values_are_refused_at_their_element",
    ),
    (
        "reader-nil-not-refused",
        "reader.rs",
        '                    "nil" => {',
        '                    "nil-disabled" => {',
        "reader",
        "nilled_per_object_is_refused_not_dropped",
    ),
    (
        "reader-trailing-children-accepted",
        "reader.rs",
        "        match self.elements.get(self.cursor) {\n            None => Ok(()),",
        "        match None::<&(&XmlElement, String)> {\n            None => Ok(()),",
        "reader",
        "out_of_order_and_missing_children_are_refused",
    ),
    (
        "reader-purpose-order-lost",
        "reader.rs",
        "        items.push(item);",
        "        items.insert(0, item);",
        "reader",
        "purpose_choice_keeps_document_order",
    ),
    (
        "reader-boolean-inverted",
        "reader.rs",
        '        "true" | "1" => Some(true),',
        '        "true" | "1" => Some(false),',
        "reader",
        "georeferencing_is_read_in_full",
    ),
    (
        "reader-dt-error-unrooted",
        "reader.rs",
        '        format!("{path}/{inner}")',
        "        inner.to_owned()",
        "reader",
        "dt_errors_are_rooted_at_the_loin_path",
    ),
    (
        "reader-concept-prefix-skipped",
        "reader.rs",
        "            if element.namespace_uri() != Some(dt::NAMESPACE) {\n                break;",
        "            if element.namespace_uri().is_some() || true {\n                break;",
        "reader",
        "per_object_content_is_read_through_openbim_dt",
    ),
    (
        "writer-milestone-date-dropped",
        "authoring.rs",
        "    if let Some(date) = value.date() {",
        "    if let Some(date) = value.date().filter(|_| false) {",
        "authoring",
        "optional_attributes_survive_write_then_read",
    ),
]


def run(workdir: pathlib.Path, suite: str, test: str) -> int:
    # A wrong --test target makes cargo exit 0 with "0 tests run", which
    # would silently mark every mutation as surviving. Assert the test ran.
    result = subprocess.run(
        ["cargo", "test", "--offline", "-p", "openbim-loin", "--test", suite, test],
        cwd=workdir,
        capture_output=True,
        text=True,
    )
    if "running 1 test" not in result.stdout:
        raise SystemExit(
            f"harness bug: {suite}::{test} did not run\n{result.stdout}"
        )
    return result.returncode


def main() -> int:
    leaked = 0
    for name, filename, old, new, suite, test in MUTATIONS:
        with tempfile.TemporaryDirectory(prefix="loin-mutate-") as tmp:
            sandbox = pathlib.Path(tmp) / "repo"
            # Copy a disposable sandbox; never mutate the real tree.
            shutil.copytree(
                ROOT,
                sandbox,
                ignore=shutil.ignore_patterns("target", ".git", ".worktrees"),
            )
            target = sandbox / "openbim-loin" / "src" / filename
            text = target.read_text(encoding="utf-8")
            if text.count(old) != 1:
                print(f"SKIP {name}: anchor found {text.count(old)} times")
                leaked += 1
                continue
            target.write_text(text.replace(old, new), encoding="utf-8")
            if run(sandbox, suite, test) == 0:
                print(f"SURVIVED {name}: {test} still passes")
                leaked += 1
            else:
                print(f"killed {name}")
    print(f"leaked={leaked}")
    return 1 if leaked else 0


if __name__ == "__main__":
    sys.exit(main())
