use std::str::FromStr;

use openbim_dt::{
    Base, Concept, DataType, DataTypeName, DateTime, Decimal, Dimension, GroupOfProperties, Guid,
    Language, MultiLanguageText, ObjectType, Property, QuantityKind, Rational, Reference,
    ReferenceDocument, Scale, Subject, Unit,
};
use openbim_loin::{
    Actor, AlphanumericalInformation, DatumRegistryReference, Detail, DocumentFormat,
    Documentation, GeometricalInformation, InformationDeliveryMilestone, Prerequisites, Purpose,
    RequiredDocument, Specification, SpecificationPerObjectType, ThresholdDimension,
};

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

#[test]
fn purpose_exposes_actual_dt_value_contracts() {
    let id = guid("10000000-0000-0000-0000-000000000000");
    let mut purpose = Purpose::new(id.clone(), text("Coordination"));
    purpose.add_reference_document(reference("20000000-0000-0000-0000-000000000000"));
    assert_eq!(purpose.guid(), &id);
    assert_eq!(purpose.name().text(), "Coordination");
}

#[test]
fn specification_uses_dt_owned_iso_23387_types_at_every_imported_boundary() {
    let object_type = ObjectType::new(Subject::new(concept(
        "30000000-0000-0000-0000-000000000000",
    )));
    let mut info = AlphanumericalInformation::new();
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
    purpose.set_language(Some(Language::from_str("de-CH").unwrap()));
    let actor = Actor::new(guid("f1000000-0000-0000-0000-000000000000"), text("Author"));
    let mut milestone = InformationDeliveryMilestone::new(
        guid("f2000000-0000-0000-0000-000000000000"),
        text("Handover"),
    );
    milestone.set_date(Some(DateTime::from_str("2026-08-25T00:00:00Z").unwrap()));
    let format = DocumentFormat::new(text("PDF"), text("2.0"));
    let required = RequiredDocument::new(
        guid("f3000000-0000-0000-0000-000000000000"),
        text("Manual"),
        format,
    );
    let mut documentation = Documentation::new(guid("f4000000-0000-0000-0000-000000000000"));
    documentation.add_required_document(required);
    let unit = Unit::new(
        concept("f5000000-0000-0000-0000-000000000000"),
        reference("f6000000-0000-0000-0000-000000000000"),
        Scale::Linear,
        Base::Ten,
        Rational::from_str("1").unwrap(),
        Rational::from_str("0").unwrap(),
    );
    let threshold = ThresholdDimension::new(1.0, unit, text("Minimum"));
    let detail = Detail::new(Some(reference("f8000000-0000-0000-0000-000000000000")));
    let mut datum = DatumRegistryReference::new(text("Registry"));
    datum.add_description(text("Description"));
    let geometry = GeometricalInformation::new(guid("f9000000-0000-0000-0000-000000000000"));
    assert_eq!(purpose.language().unwrap().as_str(), "de-CH");
    assert_eq!(actor.role().text(), "Author");
    assert_eq!(milestone.name().text(), "Handover");
    assert_eq!(documentation.required_documents().len(), 1);
    assert_eq!(threshold.unit().concept().names()[0].text(), "Synthetic");
    assert!(detail.dictionary().is_some());
    assert_eq!(datum.name().text(), "Registry");
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
}
