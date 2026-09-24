//! Reading the typed model from a parsed document (`from_document`).
//!
//! The fixture is synthetic (no ISO example material) and populates every
//! LOIN-owned element and attribute. Each test states what the reader must
//! produce or refuse; refusals are checked by kind and path, not message text.

use openbim_loin::{
    dt::Language, Appearance, Connections, CoordinateReferenceSystemKind, Dimensionality,
    InsideGeometry, LevelOfInformationNeed, LoinDocument, ParametricBehaviour, PurposeItem,
    ReadErrorKind, RelativeOrAbsolute, Severity, ShapeAssembly,
};

const MAXIMAL: &str = include_str!("fixtures/reader-maximal.xml");

fn read(xml: &str) -> LevelOfInformationNeed {
    let document = LoinDocument::parse(xml).expect("fixture parses");
    LevelOfInformationNeed::from_document(&document).expect("fixture reads")
}

/// Reads `xml` and returns the refusal, which the caller checks.
fn refusal(xml: &str) -> openbim_loin::ReadError {
    let document = LoinDocument::parse(xml).expect("mutated fixture still parses");
    LevelOfInformationNeed::from_document(&document).expect_err("reader must refuse")
}

/// The maximal fixture with `from` replaced by `to` exactly once.
fn mutated(from: &str, to: &str) -> String {
    assert_eq!(
        MAXIMAL.matches(from).count(),
        1,
        "anchor must be unique: {from}"
    );
    MAXIMAL.replacen(from, to, 1)
}

#[test]
fn fixture_is_schema_valid() {
    let document = LoinDocument::parse(MAXIMAL).expect("fixture parses");
    let errors: Vec<_> = document
        .validate()
        .into_iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn prerequisites_are_read_in_full() {
    let model = read(MAXIMAL);
    let specification = &model.specifications()[0];
    assert_eq!(specification.name(), "Synthetic maximal");
    let prerequisites = specification.prerequisites();

    let milestone = prerequisites.milestone();
    assert_eq!(milestone.name().text(), "Gate 2");
    assert_eq!(milestone.descriptions().len(), 1);
    assert_eq!(milestone.reference_documents().len(), 1);
    assert_eq!(
        milestone.date().map(|date| date.as_str()),
        Some("2026-03-04T05:06:07Z")
    );

    let providing = prerequisites.providing_actor();
    assert_eq!(providing.first_name(), Some("Ada"));
    assert_eq!(providing.middle_name(), Some("B."));
    assert_eq!(providing.last_name(), Some("Author"));
    assert_eq!(providing.affiliation(), Some("Synthetic Works"));
    assert!(providing.description().is_some());
    assert_eq!(
        providing.email_address().map(|email| email.as_str()),
        Some("ada@example.org")
    );
    let receiving = prerequisites.receiving_actor();
    assert_eq!(receiving.role().text(), "Reviewer");
    assert!(receiving.first_name().is_none() && receiving.email_address().is_none());
}

#[test]
fn purpose_choice_keeps_document_order() {
    let model = read(MAXIMAL);
    let items = model.specifications()[0].prerequisites().purpose().items();
    let kinds: Vec<&str> = items
        .iter()
        .map(|item| match item {
            PurposeItem::Name(_) => "Name",
            PurposeItem::Definition(_) => "Definition",
            PurposeItem::ReferenceDocument(_) => "ReferenceDocument",
            PurposeItem::Description(_) => "Description",
            PurposeItem::Language(_) => "Language",
            PurposeItem::Region(_) => "Region",
            PurposeItem::DictionaryRef(_) => "DictionaryRef",
        })
        .collect();
    // The fixture deliberately puts Description before Name.
    assert_eq!(
        kinds,
        [
            "Description",
            "Name",
            "Definition",
            "ReferenceDocument",
            "Language",
            "Region",
            "DictionaryRef"
        ]
    );
    let language = items.iter().find_map(|item| match item {
        PurposeItem::Language(language) => Some(language),
        _ => None,
    });
    assert_eq!(language, Some(&"de-CH".parse::<Language>().unwrap()));
}

#[test]
fn per_object_content_is_read_through_openbim_dt() {
    let model = read(MAXIMAL);
    let per_object = &model.specifications()[0].per_object()[0];
    assert_eq!(per_object.concept().names()[0].text(), "Spec for a wall");
    assert_eq!(
        per_object.object_type().subject().concept().names()[0].text(),
        "Wall"
    );

    let alpha = per_object
        .alphanumerical_information()
        .expect("alphanumerical");
    assert_eq!(alpha.properties().len(), 1);
    let property = &alpha.properties()[0];
    assert_eq!(property.data_type().possible_values()[0].values().len(), 2);
    assert_eq!(alpha.groups_of_properties().len(), 1);
    assert_eq!(alpha.group_refs().len(), 1);

    let documentation = per_object.documentation().expect("documentation");
    let document = &documentation.documents()[0];
    assert_eq!(document.document_type(), Some("Drawing"));
    assert_eq!(document.form(), Some("Digital"));
    assert_eq!(document.content(), Some("Plan view"));
    assert_eq!(document.reference_documents().len(), 1);
    assert_eq!(document.format().specifications().len(), 1);
}

#[test]
fn geometry_enumerations_and_location_are_typed() {
    let model = read(MAXIMAL);
    let geometry = model.specifications()[0].per_object()[0]
        .geometrical_information()
        .expect("geometry");
    assert_eq!(geometry.placeholder(), Some(false));
    assert_eq!(geometry.dimensionality(), Some(Dimensionality::ThreeD));
    assert_eq!(geometry.appearance(), Some(Appearance::SingularMaterial));
    assert_eq!(
        geometry.parametric_behaviour(),
        Some(ParametricBehaviour::Requested)
    );
    let detail = geometry.detail().expect("detail");
    assert!(detail.dictionary().is_some());
    assert_eq!(
        detail.shape_assembly(),
        Some(ShapeAssembly::SingleObjectSingularShape)
    );
    let influence = detail.shape_influence().expect("shape influence");
    assert_eq!(
        influence.inside_geometry,
        Some(InsideGeometry::NoInsideGeometry)
    );
    assert_eq!(
        influence.connections,
        Some(Connections::ConnectionsAsPartOfShape)
    );
    let location = geometry.location().expect("location");
    assert_eq!(location.relative_or_absolute, RelativeOrAbsolute::Relative);
    assert_eq!(location.reference_object.as_deref(), Some("Grid A1"));
}

#[test]
fn georeferencing_is_read_in_full() {
    let model = read(MAXIMAL);
    let geo = model.specifications()[0].geo_referencing().expect("geo");
    let crs = geo.coordinate_reference_system.as_ref().expect("crs");
    assert_eq!(crs.crs_type, CoordinateReferenceSystemKind::ProjectedCrs);
    assert_eq!(crs.datum.name(), "Synthetic datum");
    assert!(crs.datum.datum_type().registry_reference().is_some());
    assert_eq!(crs.datum.datum_type().descriptions().len(), 1);
    assert!(crs.vertical_datum.is_some());
    let system = &geo.model_coordinate_systems()[0];
    assert!(system.is_projected);
    assert_eq!(system.first_coordinate.as_str(), "2600000.5");
    assert_eq!(
        system.unit_scale.as_ref().map(|v| v.as_str()),
        Some("0.001")
    );
}

#[test]
fn unknown_elements_and_attributes_are_refused_with_paths() {
    let extension = refusal(&mutated(
        "<Region>Europe</Region>",
        "<Region>Europe</Region><Extra/>",
    ));
    assert_eq!(extension.kind(), ReadErrorKind::UnexpectedElement);
    // Inserted after Region, so it is the 7th child (DictionaryRef follows).
    assert!(
        extension.path().ends_with("/Purpose[1]/Extra[7]"),
        "{}",
        extension.path()
    );

    let attribute = refusal(&mutated("<Location>", "<Location vendor=\"x\">"));
    assert_eq!(attribute.kind(), ReadErrorKind::UnexpectedAttribute);
    assert!(
        attribute.path().ends_with("/Location[5]/@vendor"),
        "{}",
        attribute.path()
    );
}

#[test]
fn out_of_order_and_missing_children_are_refused() {
    // Height before SecondCoordinate violates the declared sequence.
    let swapped = refusal(&mutated(
        "<SecondCoordinate>1200000.25</SecondCoordinate>\n        <Height>400</Height>",
        "<Height>400</Height>\n        <SecondCoordinate>1200000.25</SecondCoordinate>",
    ));
    assert_eq!(swapped.kind(), ReadErrorKind::MissingElement);

    let no_role = refusal(&mutated("<Role language=\"en\">Reviewer</Role>", ""));
    assert_eq!(no_role.kind(), ReadErrorKind::MissingElement);
    assert!(
        no_role.path().ends_with("/ReceivingActor[4]"),
        "{}",
        no_role.path()
    );

    // A declared child after the sequence has ended (a second Location):
    // every required child is present, so only the end-of-sequence check
    // can refuse it.
    let trailing = refusal(&mutated(
        "        </Location>\n      </GeometricalInformation>",
        "        </Location>\n        <Location><RelativeOrAbsolute>Absolute</RelativeOrAbsolute></Location>\n      </GeometricalInformation>",
    ));
    assert_eq!(trailing.kind(), ReadErrorKind::UnexpectedElement);
    assert!(
        trailing
            .path()
            .ends_with("/GeometricalInformation[6]/Location[6]"),
        "{}",
        trailing.path()
    );
}

#[test]
fn invalid_values_are_refused_at_their_element() {
    let enumeration = refusal(&mutated(
        "<Dimensionality>3D</Dimensionality>",
        "<Dimensionality>ThreeD</Dimensionality>",
    ));
    assert_eq!(enumeration.kind(), ReadErrorKind::InvalidValue);
    assert!(
        enumeration.path().ends_with("/Dimensionality[2]"),
        "{}",
        enumeration.path()
    );

    let boolean = refusal(&mutated(
        "<IsProjected>true</IsProjected>",
        "<IsProjected>yes</IsProjected>",
    ));
    assert_eq!(boolean.kind(), ReadErrorKind::InvalidValue);

    let date = refusal(&mutated(
        "Date=\"2026-03-04T05:06:07Z\"",
        "Date=\"tomorrow\"",
    ));
    assert_eq!(date.kind(), ReadErrorKind::InvalidValue);
    assert!(date.path().ends_with("/@Date"), "{}", date.path());

    let text = refusal(&mutated("<Detail>", "<Detail>stray text"));
    assert_eq!(text.kind(), ReadErrorKind::UnexpectedText);
}

#[test]
fn dt_errors_are_rooted_at_the_loin_path() {
    let error = refusal(&mutated(
        "<dt:Value order=\"2\">0.7</dt:Value>",
        "<dt:Value order=\"two\">0.7</dt:Value>",
    ));
    assert_eq!(error.kind(), ReadErrorKind::DataTemplate);
    assert!(
        error.path().contains("/AlphanumericalInformation[")
            && error.path().contains("/Property[1]"),
        "{}",
        error.path()
    );
}

#[test]
fn nilled_per_object_is_refused_not_dropped() {
    let xml = mutated(
        "<loin:LevelOfInformationNeed ",
        "<loin:LevelOfInformationNeed xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ",
    );
    let start = xml.find("<SpecificationPerObjectType").unwrap();
    let end =
        xml.find("</SpecificationPerObjectType>").unwrap() + "</SpecificationPerObjectType>".len();
    let nilled = format!(
        "{}<SpecificationPerObjectType xsi:nil=\"true\"/>{}",
        &xml[..start],
        &xml[end..]
    );
    assert_eq!(refusal(&nilled).kind(), ReadErrorKind::NilledElement);
}
