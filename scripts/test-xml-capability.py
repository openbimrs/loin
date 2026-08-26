#!/usr/bin/env python3
"""Mutation-prove the LOIN XML safety, migration, validation, and fidelity gates."""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))


def run(command: list[str], cwd: Path, expect_success: bool) -> None:
    result = subprocess.run(
        command,
        cwd=cwd,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if (result.returncode == 0) != expect_success:
        expectation = "success" if expect_success else "failure"
        raise SystemExit(
            f"expected {expectation}: {' '.join(command)}\n{result.stdout}"
        )


def replace_exact(path: Path, old: str, new: str) -> None:
    content = path.read_text()
    if content.count(old) != 1:
        raise SystemExit(f"mutation anchor count for {path}: {content.count(old)}")
    path.write_text(content.replace(old, new))


def mutate(name: str, edits: list[tuple[str, str, str]], test: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"loin-{name}-") as temp:
        mutant = Path(temp) / "repo"
        shutil.copytree(
            ROOT,
            mutant,
            ignore=shutil.ignore_patterns(".git", "target", "__pycache__", "references"),
        )
        for path in (mutant, *mutant.rglob("*")):
            path.chmod(path.stat().st_mode | 0o200)
        for relative, old, new in edits:
            replace_exact(mutant / relative, old, new)
        mutation_target = TARGET / "loin-xml-mutations" / name
        previous_target = os.environ.get("CARGO_TARGET_DIR")
        os.environ["CARGO_TARGET_DIR"] = str(mutation_target)
        try:
            run(["cargo", "check", "-p", "openbim-loin", "--locked"], mutant, True)
            print(f"[mutation] {name}: compile passed (CARGO_TARGET_DIR={mutation_target})")
            run(
                ["cargo", "test", "-p", "openbim-loin", "--test", test.split("::")[0], test.split("::")[1], "--", "--exact"],
                mutant,
                False,
            )
        finally:
            if previous_target is None:
                os.environ.pop("CARGO_TARGET_DIR", None)
            else:
                os.environ["CARGO_TARGET_DIR"] = previous_target
        print(f"[mutation] {name}: killed")


def main() -> None:
    os.environ["CARGO_TARGET_DIR"] = str(TARGET)
    mutations = [
        (
            "doctype",
            [
                ("openbim-loin/src/parser.rs", "allow_dtd: false", "allow_dtd: true"),
                (
                    "openbim-loin/src/parser.rs",
                    "            Event::DocType(_) => {\n                return Err(ParseError::new(\n                    ParseErrorKind::DoctypeForbidden,\n                    position,\n                    \"DOCTYPE declarations are disabled\",\n                ));\n            }",
                    "            Event::DocType(_) => {}",
                ),
            ],
            "xml_document::strict_decoder_rejects_unsafe_or_ambiguous_xml",
        ),
        (
            "namespace-rewrite",
            [
                (
                    "openbim-loin/src/document.rs",
                    "fn migrate_element(\n    element: &mut XmlElement,\n    source: &str,\n    target: &str,\n    report: &mut MigrationReport,\n) {\n    if element.namespace_uri.as_deref() == Some(source) {",
                    "fn migrate_element(\n    element: &mut XmlElement,\n    source: &str,\n    target: &str,\n    report: &mut MigrationReport,\n) {\n    if false {",
                )
            ],
            "xml_document::migration_rewrites_only_the_observed_loin_namespace",
        ),
        (
            "migration-collision",
            [
                (
                    "openbim-loin/src/document.rs",
                    "        if source != target {\n            ensure_migration_safe(&self.root, source.uri(), target.uri())?;\n        }\n        let mut migrated = self.clone();",
                    "        let mut migrated = self.clone();",
                )
            ],
            "xml_document::migration_fails_closed_on_expanded_attribute_collisions",
        ),
        (
            "required-recipient",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    ChildRule::one("ReceivingActor"),',
                    '    ChildRule::optional("ReceivingActor"),',
                )
            ],
            "xml_document::clause_validation_reports_order_cardinality_lexical_and_unknown_content",
        ),
        (
            "dimensionality-enum",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '        (Some("GeometricalInformation"), "Dimensionality") => Some(DIMENSIONALITY),',
                    '        (Some("GeometricalInformation"), "Dimensionality") => None,',
                )
            ],
            "validation_coverage::rejects_invalid_geometrical_enumeration",
        ),
        (
            "parent-sensitive-grammar",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "const ROOT: &[ChildRule] = &[ChildRule::many(\"Specification\", 1)];",
                    "const ROOT: &[ChildRule] = &[ChildRule::many(\"Specification\", 1), ChildRule::optional(\"ModelCoordinateSystem\")];",
                ),
                (
                    "openbim-loin/src/validation.rs",
                    "        (Some(\"LevelOfInformationNeed\"), \"Specification\") => (SPECIFICATION, true),",
                    "        (Some(\"LevelOfInformationNeed\"), \"Specification\") => (SPECIFICATION, true),\n        (Some(\"LevelOfInformationNeed\"), \"ModelCoordinateSystem\") => (MODEL_COORDINATES, true),",
                ),
            ],
            "validation_coverage::parent_sensitive_grammar_rejects_known_elements_in_forbidden_locations",
        ),
        (
            "purpose-choice-order",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '        (Some("Prerequisites"), "Purpose") => (PURPOSE, false),',
                    '        (Some("Prerequisites"), "Purpose") => (PURPOSE, true),',
                )
            ],
            "validation_coverage::purpose_repeating_choice_requires_content_and_accepts_reordered_repeated_branches",
        ),
        (
            "purpose-choice-cardinality",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    if element.local_name() == "Purpose" && declared_count == 0 {',
                    "    if false {",
                )
            ],
            "validation_coverage::purpose_repeating_choice_requires_content_and_accepts_reordered_repeated_branches",
        ),
        (
            "required-specification-name",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    if name == "Specification" && !name_seen {',
                    '    if false && !name_seen {',
                )
            ],
            "validation_coverage::required_attributes_language_and_exact_double_lexemes_are_checked",
        ),
        (
            "required-alphanumerical-guid",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '            | (\n                Some("SpecificationPerObjectType"),\n                "AlphanumericalInformation"\n            )',
                    '            | (\n                Some("SpecificationPerObjectType"),\n                "AlphanumericalInformation_DISABLED"\n            )',
                )
            ],
            "validation_coverage::alphanumerical_choice_accepts_empty_content_but_requires_identity",
        ),
        (
            "required-inherited-date",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    if name == "SpecificationPerObjectType" && !creation_seen {',
                    '    if false && !creation_seen {',
                )
            ],
            "validation_coverage::rejects_nilled_content_and_still_requires_inherited_attributes",
        ),
        (
            "exact-xs-double",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    if parent == Some("ThresholdDimension") && name == "Threshold" && !valid_xs_double(&trimmed) {',
                    '    if parent == Some("ThresholdDimension") && name == "Threshold" && trimmed.parse::<f64>().is_err() {',
                )
            ],
            "validation_coverage::required_attributes_language_and_exact_double_lexemes_are_checked",
        ),
        (
            "inherited-concept-dispatch",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "        if element.local_name() == \"SpecificationPerObjectType\"",
                    "        if false && element.local_name() == \"SpecificationPerObjectType\"",
                )
            ],
            "validation_coverage::inherited_concept_children_are_dt_qualified_ordered_and_retained",
        ),
        (
            "inherited-concept-minimum",
            [("openbim-loin/src/validation.rs", "    if element.local_name() == \"SpecificationPerObjectType\" && inherited_dt_count == 0 {", "    if false {")],
            "validation_coverage::inherited_concept_children_are_dt_qualified_ordered_and_retained",
        ),
        (
            "inherited-concept-whitelist",
            [("openbim-loin/src/validation.rs", "            if !INHERITED_DT_CONCEPT_CHILDREN.contains(&child.local_name()) {", "            if false {")],
            "validation_coverage::inherited_concept_children_are_dt_qualified_ordered_and_retained",
        ),
        (
            "schema-local-namespace",
            [
                ("openbim-loin/src/validation.rs", "        if crate::NamespaceVersion::from_uri(namespace).is_some() {", "        if false {"),
                ("openbim-loin/src/validation.rs", "        if child.namespace_uri().is_some() {", "        if false {"),
            ],
            "validation_coverage::qualified_local_loin_elements_are_retained_but_diagnosed",
        ),
        ("exact-xs-decimal", [("openbim-loin/src/validation.rs", "dt::Decimal::from_str(&trimmed).is_err()", "false")], "validation_coverage::validates_georeferencing_clause_shape_and_scalar_lexemes"),
        ("exact-xs-boolean", [("openbim-loin/src/validation.rs", "    if parent == Some(\"ModelCoordinateSystem\") && name == \"IsProjected\" {", "    if false {")], "validation_coverage::validates_georeferencing_clause_shape_and_scalar_lexemes"),
        (
            "geometrical-sequence-order",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '        (Some("SpecificationPerObjectType"), "GeometricalInformation") => (GEOMETRICAL, true),',
                    '        (Some("SpecificationPerObjectType"), "GeometricalInformation") => (GEOMETRICAL, false),',
                )
            ],
            "validation_coverage::geometrical_sequence_represents_all_current_owned_branches",
        ),
        (
            "groups-empty-container",
            [("openbim-loin/src/validation.rs", '    ChildRule::many("GroupOfProperties", 0),', '    ChildRule::many("GroupOfProperties", 1),')],
            "validation_coverage::documentation_and_alphanumerical_sequences_match_the_current_schema",
        ),
        (
            "current-document-name",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '        (Some("Documentation"), "Document") => (DOCUMENT, true),',
                    '        (Some("Documentation"), "RequiredDocument") => (DOCUMENT, true),',
                )
            ],
            "validation_coverage::documentation_and_alphanumerical_sequences_match_the_current_schema",
        ),
        (
            "alphanumerical-groups-cardinality",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    ChildRule::optional("GroupsOfProperties"),',
                    '    ChildRule::many("GroupsOfProperties", 0),',
                )
            ],
            "validation_coverage::documentation_and_alphanumerical_sequences_match_the_current_schema",
        ),
        (
            "actor-description-cardinality",
            [
                (
                    "openbim-loin/src/validation.rs",
                    '    ChildRule::optional("Description"),\n    ChildRule::optional("EMailAddress"),',
                    '    ChildRule::many("Description", 0),\n    ChildRule::optional("EMailAddress"),',
                )
            ],
            "validation_coverage::actor_sequence_and_cardinality_match_the_current_schema",
        ),
        ("actor-email-restriction", [("openbim-loin/src/validation.rs", "        && !crate::model::matches_actor_email_pattern(&value)", "        && false")], "validation_coverage::actor_sequence_and_cardinality_match_the_current_schema"),
        ("xsi-nil-scope", [("openbim-loin/src/validation.rs", "if namespace == XSI_NAMESPACE && name == \"SpecificationPerObjectType\" =>", "if namespace == XSI_NAMESPACE =>")], "validation_coverage::xsi_and_expanded_attribute_rules_match_the_declared_boundary"),
        (
            "nilled-content",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "    if nilled && (element.children().next().is_some() || has_text_content(element)) {",
                    "    if false && (element.children().next().is_some() || has_text_content(element)) {",
                )
            ],
            "validation_coverage::rejects_nilled_content_and_still_requires_inherited_attributes",
        ),
        (
            "nilled-whitespace",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "if !value.is_empty()",
                    "if !value.trim().is_empty()",
                )
            ],
            "validation_coverage::rejects_nilled_content_and_still_requires_inherited_attributes",
        ),
        (
            "text-carriage-return",
            [
                (
                    "openbim-loin/src/document.rs",
                    "            '\\r' => output.push_str(\"&#xD;\"),",
                    "            '\\r' => output.push(' '),",
                )
            ],
            "xml_document::preserves_represented_attribute_whitespace_and_text_carriage_return",
        ),
        (
            "xsd-lexical-whitespace",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "fn collapse_whitespace(value: &str) -> String {\n    value\n        .split(is_xsd_whitespace)\n        .filter(|part| !part.is_empty())\n        .collect::<Vec<_>>()\n        .join(\" \")\n}",
                    "fn collapse_whitespace(value: &str) -> String {\n    value.split_whitespace().collect::<Vec<_>>().join(\" \")\n}",
                )
            ],
            "validation_coverage::xsd_whitespace_does_not_treat_unicode_separators_as_whitespace",
        ),
        (
            "xsd-complex-content-whitespace",
            [
                (
                    "openbim-loin/src/validation.rs",
                    "text.chars().any(|character| !is_xsd_whitespace(character))",
                    "!text.trim().is_empty()",
                )
            ],
            "validation_coverage::xsd_whitespace_does_not_treat_unicode_separators_as_whitespace",
        ),
        (
            "comment-literal-content",
            [
                (
                    "openbim-loin/src/document.rs",
                    "Event::Comment(BytesText::from_escaped(value))",
                    "Event::Comment(BytesText::new(value))",
                )
            ],
            "xml_document::preserves_literal_comment_content_without_entity_escaping",
        ),
    ]
    for name, edits, test in mutations:
        mutate(name, edits, test)
    print(f"[mutation] all {len(mutations)} XML capability mutations killed")


if __name__ == "__main__":
    main()
