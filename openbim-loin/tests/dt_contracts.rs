use std::str::FromStr;

use openbim_dt::{
    Base, Concept, DataType, DataTypeName, DateTime, Decimal, Dimension, GroupOfProperties, Guid,
    Language, MultiLanguageText, ObjectType, Property, QuantityKind, Rational, Reference,
    ReferenceDocument, Scale, Subject, Unit,
};
use openbim_loin::*;

fn guid(value: &str) -> Guid {
    Guid::from_str(value).unwrap()
}
fn text(value: &str) -> MultiLanguageText {
    MultiLanguageText::new("en", value).unwrap()
}
fn concept(value: &str) -> Concept {
    Concept::new(
        guid(value),
        DateTime::from_str("2026-08-25T00:00:00Z").unwrap(),
        text("Synthetic"),
        text("Synthetic definition"),
    )
}
fn reference(value: &str) -> Reference {
    Reference::new(Some(guid(value)), None)
}
fn unit(value: &str) -> Unit {
    Unit::new(
        concept(value),
        reference("14000000-0000-0000-0000-000000000000"),
        Scale::Linear,
        Base::Ten,
        Rational::from_str("1").unwrap(),
        Rational::from_str("0").unwrap(),
    )
}

#[test]
fn purpose_exposes_actual_dt_value_contracts() {
    let id = guid("10000000-0000-0000-0000-000000000000");
    let mut purpose = Purpose::new(id.clone(), text("Coordination"));
    purpose.add_reference_document(reference("20000000-0000-0000-0000-000000000000"));
    assert_eq!(purpose.guid(), &id);
    assert_eq!(purpose.name().unwrap().text(), "Coordination");
    assert!(matches!(
        purpose.items()[1],
        PurposeItem::ReferenceDocument(_)
    ));
}

#[test]
fn specification_uses_dt_owned_iso_23387_types_at_every_imported_boundary() {
    let object_type = ObjectType::new(Subject::new(concept(
        "30000000-0000-0000-0000-000000000000",
    )));
    let mut info = AlphanumericalInformation::new(guid("31000000-0000-0000-0000-000000000000"));
    info.add_property(Property::new(
        concept("40000000-0000-0000-0000-000000000000"),
        DataType::new(Some(DataTypeName::String)),
    ));
    info.add_quantity_kind(QuantityKind::new(
        concept("50000000-0000-0000-0000-000000000000"),
        reference("60000000-0000-0000-0000-000000000000"),
    ));
    info.add_group_of_properties(GroupOfProperties::new(
        Subject::new(concept("70000000-0000-0000-0000-000000000000")),
        reference("80000000-0000-0000-0000-000000000000"),
    ));
    let zero = Decimal::from_str("0").unwrap();
    info.add_dimension(Dimension::new(
        concept("90000000-0000-0000-0000-000000000000"),
        [
            zero.clone(),
            zero.clone(),
            zero.clone(),
            zero.clone(),
            zero.clone(),
            zero.clone(),
            zero,
        ],
    ));
    info.add_unit(Unit::new(
        concept("a0000000-0000-0000-0000-000000000000"),
        reference("b0000000-0000-0000-0000-000000000000"),
        Scale::Linear,
        Base::Ten,
        Rational::from_str("1").unwrap(),
        Rational::from_str("0").unwrap(),
    ));
    info.add_reference_document(ReferenceDocument::new(
        concept("c0000000-0000-0000-0000-000000000000"),
        Language::from_str("en").unwrap(),
    ));
    let mut spec = SpecificationPerObjectType::new(
        concept("d0000000-0000-0000-0000-000000000000"),
        object_type,
    );
    spec.set_alphanumerical_information(Some(info));
    let info = spec.alphanumerical_information().unwrap();
    assert_eq!(info.properties().len(), 1);
    assert_eq!(info.quantity_kinds().len(), 1);
    assert_eq!(info.groups_of_properties().len(), 1);
    assert_eq!(info.dimensions().len(), 1);
    assert_eq!(info.units().len(), 1);
    assert_eq!(info.reference_documents().len(), 1);
}

#[test]
fn remaining_iso_7817_touchpoints_keep_dt_value_identity() {
    let mut purpose = Purpose::new(
        guid("f0000000-0000-0000-0000-000000000000"),
        text("Purpose"),
    );
    purpose
        .set_language(Some(Language::from_str("de-CH").unwrap()))
        .unwrap();
    let actor = Actor::new(guid("f1000000-0000-0000-0000-000000000000"), text("Author"));
    let mut milestone = InformationDeliveryMilestone::new(
        guid("f2000000-0000-0000-0000-000000000000"),
        text("Handover"),
    );
    milestone.set_date(Some(DateTime::from_str("2026-08-25T00:00:00Z").unwrap()));
    let format = DocumentFormat::new(text("PDF"), text("2.0"));
    let required = Document::new(
        guid("f3000000-0000-0000-0000-000000000000"),
        text("Manual"),
        format,
    );
    let mut documentation = Documentation::new(guid("f4000000-0000-0000-0000-000000000000"));
    documentation.add_document(required);
    let unit = Unit::new(
        concept("f5000000-0000-0000-0000-000000000000"),
        reference("f6000000-0000-0000-0000-000000000000"),
        Scale::Linear,
        Base::Ten,
        Rational::from_str("1").unwrap(),
        Rational::from_str("0").unwrap(),
    );
    let threshold = ThresholdDimension::new(1.0, unit, text("Minimum"));
    let mut detail = Detail::new();
    detail.set_dictionary(Some(reference("f7000000-0000-0000-0000-000000000000")));
    let geometry = GeometricalInformation::new(guid("f9000000-0000-0000-0000-000000000000"));
    assert_eq!(purpose.language().unwrap().as_str(), "de-CH");
    assert_eq!(actor.role().text(), "Author");
    assert_eq!(milestone.name().text(), "Handover");
    assert_eq!(documentation.documents().len(), 1);
    assert_eq!(threshold.unit().concept().names()[0].text(), "Synthetic");
    assert!(detail.dictionary().is_some());
    assert_eq!(
        geometry.guid().as_str(),
        "f9000000-0000-0000-0000-000000000000"
    );

    let receiving_actor = Actor::new(
        guid("fa000000-0000-0000-0000-000000000000"),
        text("Receiver"),
    );
    let prerequisites = Prerequisites::new(
        guid("fb000000-0000-0000-0000-000000000000"),
        purpose,
        milestone,
        actor,
        receiving_actor,
    );
    let per_object = SpecificationPerObjectType::new(
        concept("fc000000-0000-0000-0000-000000000000"),
        ObjectType::new(Subject::new(concept(
            "fd000000-0000-0000-0000-000000000000",
        ))),
    );
    let mut specification = Specification::new(
        guid("fe000000-0000-0000-0000-000000000000"),
        "Synthetic specification",
        prerequisites,
    );
    assert!(specification.per_object().is_empty());
    specification.add_per_object(per_object);
    assert_eq!(specification.per_object().len(), 1);
    specification.set_geo_referencing(Some(GeoReferencing::new()));
    assert!(specification.geo_referencing().is_some());
    let mut root = LevelOfInformationNeed::new(specification);
    assert_eq!(root.specifications().len(), 1);
    root.add_specification(root.specifications()[0].clone());
    assert_eq!(root.specifications().len(), 2);
}

#[test]
fn typed_contract_represents_current_purpose_actor_and_alphanumerical_state() {
    let mut purpose = Purpose::from_item(
        guid("11000000-0000-0000-0000-000000000000"),
        PurposeItem::Region("DE".into()),
    );
    assert_eq!(purpose.set_region(None), Err(EmptyPurpose));
    assert_eq!(purpose.items().len(), 1);
    purpose.add_item(PurposeItem::Name(text("First")));
    purpose.add_item(PurposeItem::Name(text("Second")));
    assert_eq!(purpose.items().len(), 3);

    let mut actor = Actor::new(guid("12000000-0000-0000-0000-000000000000"), text("Role"));
    actor.set_email_address(Some("actor@example.invalid".parse().unwrap()));
    actor.set_description(Some(text("Author")));
    actor.set_first_name(Some("Ada".into()));
    actor.set_middle_name(Some("M".into()));
    actor.set_last_name(Some("Lovelace".into()));
    actor.set_affiliation(Some("Synthetic".into()));
    assert_eq!(
        actor.email_address().map(EmailAddress::as_str),
        Some("actor@example.invalid")
    );
    assert!("prefix@@a@b.c".parse::<EmailAddress>().is_ok());
    assert!("not-an-email".parse::<EmailAddress>().is_err());
    assert!("a@.invalid".parse::<EmailAddress>().is_err());
    assert_eq!(actor.description().unwrap().text(), "Author");
    assert_eq!(actor.affiliation(), Some("Synthetic"));

    let id = guid("13000000-0000-0000-0000-000000000000");
    let mut info = AlphanumericalInformation::new(id.clone());
    assert_eq!(info.guid(), &id);
    assert!(info.groups_container().is_none());
    info.set_groups_of_properties(Some(GroupsOfProperties::new()));
    assert!(info.groups_container().is_some());
    assert!(info.groups_of_properties().is_empty());
    info.add_group_of_properties(GroupOfProperties::new(
        Subject::new(concept("14000000-0000-0000-0000-000000000000")),
        reference("14100000-0000-0000-0000-000000000000"),
    ));
    assert_eq!(info.groups_of_properties().len(), 1);
}

#[test]
fn typed_contract_represents_current_document_and_geometry_state() {
    let mut document = Document::new(
        guid("15000000-0000-0000-0000-000000000000"),
        text("Manual"),
        DocumentFormat::new(text("PDF"), text("2.0")),
    );
    document.set_document_type(Some("manual".into()));
    document.set_form(Some("digital".into()));
    document.set_content(Some("synthetic".into()));
    document.add_reference_document(reference("15100000-0000-0000-0000-000000000000"));
    assert_eq!(document.form(), Some("digital"));
    assert_eq!(document.content(), Some("synthetic"));
    assert_eq!(document.reference_documents().len(), 1);

    let threshold = ThresholdDimension::new(
        0.01,
        unit("16000000-0000-0000-0000-000000000000"),
        text("Tolerance"),
    );
    let mut influence = ShapeInfluence::new();
    influence.inside_geometry = Some(InsideGeometry::SeparateShapes);
    influence.threshold_dimension = Some(threshold);
    let mut detail = Detail::new();
    detail.set_shape_assembly(Some(ShapeAssembly::MultipleObjects));
    detail.set_shape_representation(Some(ShapeRepresentation::OuterShellAsSeparateShapes));
    detail.set_shape_influence(Some(influence));
    let mut geometry = GeometricalInformation::new(guid("17000000-0000-0000-0000-000000000000"));
    geometry.set_placeholder(Some(false));
    geometry.set_detail(Some(detail));
    geometry.set_dimensionality(Some(Dimensionality::ThreeD));
    geometry.set_appearance(Some(Appearance::RealisticAppearance));
    geometry.set_parametric_behaviour(Some(ParametricBehaviour::Requested));
    let location = Location::new(RelativeOrAbsolute::Absolute, Some("site-grid".into()));
    geometry.set_location(Some(location));
    assert!(geometry.detail().unwrap().shape_influence().is_some());
    assert_eq!(geometry.placeholder(), Some(false));
    assert_eq!(geometry.dimensionality(), Some(Dimensionality::ThreeD));
    assert_eq!(geometry.appearance(), Some(Appearance::RealisticAppearance));
    assert_eq!(
        geometry.parametric_behaviour(),
        Some(ParametricBehaviour::Requested)
    );
    assert_eq!(
        geometry.location().unwrap().reference_object.as_deref(),
        Some("site-grid")
    );
}

#[test]
fn typed_contract_represents_current_georeferencing_state() {
    let mut datum_type = DatumRegistryReference::new(text("Synthetic datum type"));
    datum_type.set_registry_reference(Some(reference("18100000-0000-0000-0000-000000000000")));
    assert!(datum_type.registry_reference().is_some());
    let datum = Datum::new("Synthetic datum", datum_type);
    let crs = CoordinateReferenceSystem::new(CoordinateReferenceSystemKind::ProjectedCrs, datum);
    let coordinates = ModelCoordinateSystem::new(
        true,
        Decimal::from_str("1.25").unwrap(),
        Decimal::from_str("-2").unwrap(),
        Decimal::from_str("3").unwrap(),
    );
    assert!(coordinates.is_projected);
    assert_eq!(coordinates.first_coordinate.as_str(), "1.25");
    let mut geo = GeoReferencing::new();
    geo.set_coordinate_reference_system(Some(crs));
    geo.add_model_coordinate_system(coordinates);
    assert_eq!(geo.model_coordinate_systems().len(), 1);
    assert_eq!(
        geo.coordinate_reference_system().unwrap().crs_type(),
        CoordinateReferenceSystemKind::ProjectedCrs
    );
}
