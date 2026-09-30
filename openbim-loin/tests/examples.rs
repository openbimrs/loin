//! The shipped `examples/*.xml` documents stay valid and round-trippable.
//!
//! Each example states what it demonstrates; this pins each claim so an
//! example cannot rot silently. `examples/check_examples.rs` runs the same
//! stages as a user-facing program.

use std::collections::BTreeSet;
use std::path::Path;

use openbim_loin::{
    dt, grammar, AuthoringError, LevelOfInformationNeed, LoinDocument, OutputNamespace,
    ReadErrorKind, XmlElement,
};

/// What the typed-model stages must do for one example.
enum Model {
    /// Reads, and writing it back from the model is the identity.
    WritesBack,
    /// Reads, but `from_model` refuses ISO 23387-owned content (PLAN-model-io phase 5).
    ReadsOnly,
    /// The reader refuses it with this kind, by design.
    Refused(ReadErrorKind),
}

const EXAMPLES: &[(&str, Model)] = &[
    ("office-fit-out.loin.xml", Model::ReadsOnly),
    ("site-georeferencing.loin.xml", Model::WritesBack),
    (
        "deferred-object-type.loin.xml",
        Model::Refused(ReadErrorKind::NilledElement),
    ),
];

fn load(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn every_example_file_is_listed() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut on_disk: Vec<String> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".xml"))
        .collect();
    on_disk.sort();
    let mut listed: Vec<String> = EXAMPLES
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    listed.sort();
    assert_eq!(on_disk, listed);
}

#[test]
fn examples_validate_with_no_diagnostics() {
    for (name, _) in EXAMPLES {
        let document = LoinDocument::parse(&load(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let diagnostics = document.validate();
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:#?}");
    }
}

#[test]
fn examples_round_trip_byte_for_byte() {
    for (name, _) in EXAMPLES {
        let xml = load(name);
        let written = LoinDocument::parse(&xml)
            .unwrap()
            .to_xml_string(OutputNamespace::Preserve)
            .unwrap();
        assert!(written == xml, "{name}: Preserve output differs from input");
    }
}

#[test]
fn examples_meet_their_typed_model_contract() {
    for (name, expected) in EXAMPLES {
        let document = LoinDocument::parse(&load(name)).unwrap();
        let read = LevelOfInformationNeed::from_document(&document);
        match expected {
            Model::Refused(kind) => {
                let refusal = read.expect_err(name);
                assert_eq!(refusal.kind(), *kind, "{name}");
            }
            Model::ReadsOnly => {
                let model = read.unwrap_or_else(|e| panic!("{name}: {e}"));
                assert!(
                    matches!(
                        LoinDocument::from_model(&model),
                        Err(AuthoringError::UnwritableDtContent { .. })
                    ),
                    "{name}: now writable; promote it to WritesBack"
                );
            }
            Model::WritesBack => {
                let model = read.unwrap_or_else(|e| panic!("{name}: {e}"));
                let authored = LoinDocument::from_model(&model).unwrap();
                assert!(authored.validate().is_empty(), "{name}");
                let reread = LevelOfInformationNeed::from_document(&authored).unwrap();
                assert_eq!(reread, model, "{name}");
            }
        }
    }
}

type Seen<'a> = BTreeSet<(Option<&'a str>, &'a str, Option<&'a str>)>;

/// Records `(parent, element, attribute)` for every LOIN-local element and
/// each of its attributes (`None` for the element itself).
fn collect<'a>(parent: Option<&'a str>, element: &'a XmlElement, seen: &mut Seen<'a>) {
    if element.namespace_uri() == Some(dt::NAMESPACE) {
        return;
    }
    let name = element.local_name();
    seen.insert((parent, name, None));
    for attribute in element.attributes() {
        seen.insert((parent, name, Some(attribute.local_name())));
    }
    for child in element.children() {
        collect(Some(name), child, seen);
    }
}

#[test]
fn complete_example_exercises_every_grammar_element() {
    // The office example is the complete one: every element the published
    // grammar describes occurs in it.
    let document = LoinDocument::parse(&load("office-fit-out.loin.xml")).unwrap();
    let mut seen = Seen::new();
    collect(None, document.root(), &mut seen);
    let missing: Vec<_> = grammar::elements()
        .iter()
        .filter(|element| !seen.contains(&(element.parent(), element.name(), None)))
        .map(|element| format!("{}/{}", element.parent().unwrap_or("-"), element.name()))
        .collect();
    assert!(
        missing.is_empty(),
        "absent from the complete example: {missing:?}"
    );
}

#[test]
fn examples_together_exercise_every_declared_attribute() {
    let documents: Vec<_> = EXAMPLES
        .iter()
        .map(|(name, _)| LoinDocument::parse(&load(name)).unwrap())
        .collect();
    let mut seen = Seen::new();
    for document in &documents {
        collect(None, document.root(), &mut seen);
    }
    let mut checked = 0;
    let mut missing = Vec::new();
    for element in grammar::elements() {
        for attribute in element.attributes() {
            checked += 1;
            if !seen.contains(&(element.parent(), element.name(), Some(attribute.name()))) {
                missing.push(format!("{}@{}", element.name(), attribute.name()));
            }
        }
    }
    assert!(checked >= 40, "only {checked} declared attributes");
    assert!(missing.is_empty(), "never used by an example: {missing:?}");
}
