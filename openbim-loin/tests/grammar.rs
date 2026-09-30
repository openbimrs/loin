//! The published grammar: committed JSON artifact, and agreement with the
//! validator through the public API.
//!
//! Regenerate the artifact after a deliberate grammar change with
//! `LOIN_BLESS_GRAMMAR=1 cargo test -p openbim-loin --test grammar`.

use openbim_loin::grammar::{self, AttributeNamespace, Content, ValueType};
use openbim_loin::{DiagnosticCode, LoinDocument};

const ARTIFACT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/loin-grammar.json");

#[test]
fn artifact_matches_validator_tables() {
    let rendered = grammar::to_json();
    if std::env::var_os("LOIN_BLESS_GRAMMAR").is_some() {
        std::fs::write(ARTIFACT_PATH, &rendered).expect("write grammar artifact");
    }
    let committed = std::fs::read_to_string(ARTIFACT_PATH).expect("grammar artifact exists");
    assert!(
        committed == rendered,
        "loin-grammar.json is stale; rerun with LOIN_BLESS_GRAMMAR=1 and review the diff"
    );
}

#[test]
fn every_declared_child_is_itself_declared() {
    let elements = grammar::elements();
    assert!(elements.len() >= 80, "{} elements", elements.len());
    for element in elements {
        for child in element.children() {
            assert!(
                grammar::element(Some(element.name()), child.name()).is_some(),
                "{} below {}",
                child.name(),
                element.name()
            );
        }
    }
    assert!(grammar::element(None, "LevelOfInformationNeed").is_some());
    assert!(grammar::element(Some("LevelOfInformationNeed"), "Purpose").is_none());
}

#[test]
fn issue_10_asymmetries_are_stated() {
    let per_object = grammar::element(Some("Specification"), "SpecificationPerObjectType").unwrap();
    let find = |name: &str| {
        per_object
            .attributes()
            .iter()
            .find(|attribute| attribute.name() == name)
            .copied()
            .unwrap()
    };
    assert_eq!(find("GUID").namespace(), AttributeNamespace::Dt);
    assert_eq!(find("about").namespace(), AttributeNamespace::Dt);
    assert_eq!(
        find("dateOfCreation").namespace(),
        AttributeNamespace::Unqualified
    );
    assert!(find("dateOfCreation").is_required());
    assert!(per_object.is_nillable());
    assert!(per_object.inherited_dt_children().contains(&"Name"));

    let milestone =
        grammar::element(Some("Prerequisites"), "InformationDeliveryMilestone").unwrap();
    let date = milestone
        .attributes()
        .iter()
        .find(|attribute| attribute.name() == "Date")
        .unwrap();
    assert_eq!(date.value_type(), ValueType::DateTime);
    assert!(!date.is_required());

    // Context sensitivity: the same local name differs by parent.
    let purpose_name = grammar::element(Some("Purpose"), "Name").unwrap();
    let datum_name = grammar::element(Some("Datum"), "Name").unwrap();
    assert!(purpose_name
        .attributes()
        .iter()
        .any(|attribute| attribute.name() == "language" && attribute.is_required()));
    assert!(datum_name.attributes().is_empty());
    assert!(matches!(
        grammar::element(Some("CoordinateReferenceSystem"), "Type")
            .unwrap()
            .content(),
        Content::Simple(ValueType::Enumeration(_))
    ));
    assert!(matches!(
        grammar::element(Some("Datum"), "Type").unwrap().content(),
        Content::Sequence(_)
    ));
}

/// Builds the smallest document the grammar requires, from the grammar alone,
/// and checks the validator accepts it. A consumer generating documents from
/// the published description gets a valid one.
#[test]
fn minimal_document_generated_from_the_grammar_validates() {
    fn lexeme(value_type: ValueType) -> &'static str {
        match value_type {
            ValueType::Language => "en",
            ValueType::Boolean => "true",
            ValueType::Decimal | ValueType::Double => "1",
            ValueType::DateTime => "2026-01-01T00:00:00Z",
            ValueType::Guid => "20000000-0000-4000-8000-000000000001",
            ValueType::AnyUri => "urn:example",
            ValueType::EmailAddress => "a@example.org",
            ValueType::Enumeration(values) => values[0],
            _ => "text",
        }
    }
    fn write(parent: Option<&str>, name: &str, out: &mut String) {
        let element = grammar::element(parent, name).expect("declared");
        out.push('<');
        if parent.is_none() {
            out.push_str("l:");
        }
        out.push_str(name);
        if parent.is_none() {
            out.push_str(
                " xmlns:l=\"https://iso.org/2024/LOIN\" \
                 xmlns:dt=\"https://standards.iso.org/iso/23387/ed-2/en/\"",
            );
        }
        for attribute in element.attributes().iter().filter(|a| a.is_required()) {
            let prefix = match attribute.namespace() {
                AttributeNamespace::Dt => "dt:",
                AttributeNamespace::Xsi => "xsi:",
                _ => "",
            };
            out.push_str(&format!(
                " {prefix}{}=\"{}\"",
                attribute.name(),
                lexeme(attribute.value_type())
            ));
        }
        out.push('>');
        if let Some(first) = element.inherited_dt_children().first() {
            out.push_str(&format!("<dt:{first} language=\"en\">x</dt:{first}>"));
        }
        match element.content() {
            Content::Simple(value_type) => out.push_str(lexeme(value_type)),
            Content::Sequence(children) => {
                for child in children.iter().filter(|child| child.min() > 0) {
                    for _ in 0..child.min() {
                        write(Some(name), child.name(), out);
                    }
                }
            }
            // A choice needs one item overall; take the first branch.
            Content::Choice(children) => write(Some(name), children[0].name(), out),
            _ => {}
        }
        out.push_str("</");
        if parent.is_none() {
            out.push_str("l:");
        }
        out.push_str(name);
        out.push('>');
    }

    let mut xml = String::new();
    write(None, grammar::ROOT, &mut xml);
    let document = LoinDocument::parse(&xml).expect("generated document parses");
    let diagnostics = document.validate();
    assert!(diagnostics.is_empty(), "{xml}\n{diagnostics:#?}");

    // And dropping a required child is caught, so the check above is not vacuous.
    let broken = xml.replacen("<ReceivingActor", "<Unexpected", 1).replacen(
        "</ReceivingActor>",
        "</Unexpected>",
        1,
    );
    let codes: Vec<_> = LoinDocument::parse(&broken)
        .unwrap()
        .validate()
        .iter()
        .map(|d| d.code())
        .collect();
    assert!(
        codes.contains(&DiagnosticCode::MissingRequiredChild),
        "{codes:?}"
    );
}
