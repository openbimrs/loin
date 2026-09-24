//! Writing LOIN documents from the typed model (issue #1).

use openbim_loin::{
    dt::{Concept, DateTime, Guid, MultiLanguageText},
    Actor, AuthoringError, InformationDeliveryMilestone, LevelOfInformationNeed, LoinDocument,
    OutputNamespace, Prerequisites, Purpose, PurposeItem, Severity, Specification,
};

fn guid(seed: u8) -> Guid {
    format!("{seed}0000000-0000-0000-0000-000000000000")
        .parse()
        .expect("valid GUID")
}

fn created() -> DateTime {
    "2026-01-01T00:00:00Z".parse().expect("valid dateTime")
}

fn text(value: &str) -> MultiLanguageText {
    MultiLanguageText::new("en", value).expect("valid multilingual text")
}

/// The smallest model the schema accepts: one specification with prerequisites.
fn minimal_model() -> LevelOfInformationNeed {
    let purpose = Purpose::new(guid(3), text("Coordination"));
    let milestone = InformationDeliveryMilestone::new(guid(4), text("Gate"));
    let providing = Actor::new(guid(5), text("Author"));
    let receiving = Actor::new(guid(6), text("Reviewer"));
    let prerequisites = Prerequisites::new(guid(2), purpose, milestone, providing, receiving);
    let specification = Specification::new(guid(1), "Synthetic", prerequisites);
    LevelOfInformationNeed::new(specification)
}

/// `minimal_model` with GeoReferencing on its only specification.
fn model_with_geo(geo: openbim_loin::GeoReferencing) -> LevelOfInformationNeed {
    let purpose = Purpose::new(guid(3), text("Coordination"));
    let milestone = InformationDeliveryMilestone::new(guid(4), text("Gate"));
    let providing = Actor::new(guid(5), text("Author"));
    let receiving = Actor::new(guid(6), text("Reviewer"));
    let prerequisites = Prerequisites::new(guid(2), purpose, milestone, providing, receiving);
    let mut specification = Specification::new(guid(1), "Synthetic", prerequisites);
    specification.set_geo_referencing(Some(geo));
    LevelOfInformationNeed::new(specification)
}

fn errors(document: &LoinDocument) -> Vec<openbim_loin::Diagnostic> {
    document
        .validate()
        .into_iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Error)
        .collect()
}

#[test]
fn authored_document_serializes_and_validates() {
    let document = LoinDocument::from_model(&minimal_model()).expect("model is writable");
    assert!(
        errors(&document).is_empty(),
        "authored document must validate: {:#?}",
        errors(&document)
    );
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    // The root is namespace-prefixed; schema-local children are not.
    assert!(xml.contains("<loin:LevelOfInformationNeed"), "{xml}");
    assert!(xml.contains("<Specification"), "{xml}");
}

/// Definition of done: parse -> model -> document -> validate with no errors.
#[test]
fn authored_output_reparses_and_still_validates() {
    let document = LoinDocument::from_model(&minimal_model()).expect("model is writable");
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let reparsed = LoinDocument::parse(&xml).expect("authored output must reparse");
    assert!(
        errors(&reparsed).is_empty(),
        "reparsed document must validate: {:#?}",
        errors(&reparsed)
    );
    // Writing the reparsed document again must be byte-stable.
    assert_eq!(
        xml,
        reparsed
            .to_xml_string(OutputNamespace::Preserve)
            .expect("serializes")
    );
}

/// Child order must follow the declared `xs:sequence`, not insertion order.
#[test]
fn prerequisites_children_follow_schema_sequence() {
    let document = LoinDocument::from_model(&minimal_model()).expect("model is writable");
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let order: Vec<usize> = [
        "<Purpose",
        "<InformationDeliveryMilestone",
        "<ProvidingActor",
        "<ReceivingActor",
    ]
    .iter()
    .map(|needle| {
        xml.find(needle)
            .unwrap_or_else(|| panic!("missing {needle} in {xml}"))
    })
    .collect();
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(order, sorted, "children out of schema order: {xml}");
}

/// A Purpose's repeating choice keeps the order the model holds.
#[test]
fn purpose_choice_items_keep_model_order() {
    let mut purpose = Purpose::new(guid(3), text("Coordination"));
    purpose.add_item(PurposeItem::Definition(text("Definition")));
    purpose.add_item(PurposeItem::Region("DE".to_owned()));
    let milestone = InformationDeliveryMilestone::new(guid(4), text("Gate"));
    let prerequisites = Prerequisites::new(
        guid(2),
        purpose,
        milestone,
        Actor::new(guid(5), text("Author")),
        Actor::new(guid(6), text("Reviewer")),
    );
    let model =
        LevelOfInformationNeed::new(Specification::new(guid(1), "Synthetic", prerequisites));
    let xml = LoinDocument::from_model(&model)
        .expect("model is writable")
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let name = xml.find("<Name").expect("Name present");
    let definition = xml.find("<Definition").expect("Definition present");
    let region = xml.find("<Region").expect("Region present");
    assert!(name < definition && definition < region, "{xml}");
}

/// Optional actor content is written when present.
#[test]
fn optional_actor_children_are_written_in_order() {
    let mut providing = Actor::new(guid(5), text("Author"));
    providing.set_description(Some(text("Prepares the model")));
    providing.set_email_address(Some("author@example.com".parse().expect("valid email")));
    let prerequisites = Prerequisites::new(
        guid(2),
        Purpose::new(guid(3), text("Coordination")),
        InformationDeliveryMilestone::new(guid(4), text("Gate")),
        providing,
        Actor::new(guid(6), text("Reviewer")),
    );
    let model =
        LevelOfInformationNeed::new(Specification::new(guid(1), "Synthetic", prerequisites));
    let document = LoinDocument::from_model(&model).expect("model is writable");
    assert!(errors(&document).is_empty(), "{:#?}", errors(&document));
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let role = xml.find("<Role").expect("Role present");
    let description = xml.find("<Description").expect("Description present");
    let email = xml.find("<EMailAddress").expect("EMailAddress present");
    assert!(role < description && description < email, "{xml}");
}

/// ISO 23387-owned content is refused with a named, actionable error rather
/// than silently dropped.
#[test]
fn dt_owned_content_is_refused_by_name() {
    let mut specification = Specification::new(
        guid(1),
        "Synthetic",
        Prerequisites::new(
            guid(2),
            Purpose::new(guid(3), text("Coordination")),
            InformationDeliveryMilestone::new(guid(4), text("Gate")),
            Actor::new(guid(5), text("Author")),
            Actor::new(guid(6), text("Reviewer")),
        ),
    );
    specification.add_per_object(openbim_loin::SpecificationPerObjectType::new(
        Concept::new(guid(7), created(), text("Wall"), text("A wall")),
        openbim_loin::dt::ObjectType::new(openbim_loin::dt::Subject::new(Concept::new(
            guid(8),
            created(),
            text("Wall"),
            text("A wall"),
        ))),
    ));
    let model = LevelOfInformationNeed::new(specification);
    let error = LoinDocument::from_model(&model).expect_err("DT content is not writable");
    assert_eq!(
        error,
        AuthoringError::UnwritableDtContent {
            element: "ObjectType",
            reason: openbim_loin::DT_UNWRITABLE_REASON,
        }
    );
    // The message must name the element and point at the workaround.
    let message = error.to_string();
    assert!(message.contains("ObjectType"), "{message}");
    assert!(message.contains("nodes_mut"), "{message}");
}

/// The escape hatch issue #1 asks for: edit a parsed document in place.
#[test]
fn parsed_documents_can_be_edited_through_public_mutators() {
    let document = LoinDocument::from_model(&minimal_model()).expect("model is writable");
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let mut parsed = LoinDocument::parse(&xml).expect("reparses");
    let specification = parsed
        .root_mut()
        .nodes_mut()
        .iter_mut()
        .find_map(|node| match node {
            openbim_loin::XmlNode::Element(element) => Some(element),
            _ => None,
        })
        .expect("Specification present");
    specification
        .attributes_mut()
        .retain(|attribute| attribute.local_name() != "name");
    // @name is required, so removing it must be observable as a diagnostic.
    assert!(
        !errors(&parsed).is_empty(),
        "removing a required attribute must produce a diagnostic"
    );
}

/// Issue #5: setters must replace in place, not relocate to the end.
///
/// The issue's scenario: items `[Name, Definition, Language]`, then
/// `set_definition` -> must stay `[Name, Definition, Language]`.
#[test]
fn purpose_setters_replace_in_place_without_reordering() {
    let mut purpose = Purpose::new(guid(3), text("Coordination"));
    purpose.add_item(PurposeItem::Definition(text("First")));
    purpose.add_item(PurposeItem::Language("en".parse().expect("valid language")));

    purpose
        .set_definition(Some(text("Replaced")))
        .expect("purpose keeps its Name");

    let kinds: Vec<&str> = purpose
        .items()
        .iter()
        .map(|item| match item {
            PurposeItem::Name(_) => "Name",
            PurposeItem::Definition(_) => "Definition",
            PurposeItem::Language(_) => "Language",
            PurposeItem::Region(_) => "Region",
            PurposeItem::Description(_) => "Description",
            PurposeItem::ReferenceDocument(_) => "ReferenceDocument",
            PurposeItem::DictionaryRef(_) => "DictionaryRef",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["Name", "Definition", "Language"],
        "setter must not relocate the replaced item"
    );
    assert!(matches!(
        &purpose.items()[1],
        PurposeItem::Definition(value) if value.text() == "Replaced"
    ));
}

/// Authored XML must reflect the in-place replacement, not a reordered choice.
#[test]
fn replaced_purpose_item_keeps_position_in_authored_xml() {
    let mut purpose = Purpose::new(guid(3), text("Coordination"));
    purpose.add_item(PurposeItem::Definition(text("First")));
    purpose.add_item(PurposeItem::Region("DE".to_owned()));
    purpose
        .set_definition(Some(text("Replaced")))
        .expect("purpose keeps its Name");

    let model = LevelOfInformationNeed::new(Specification::new(
        guid(1),
        "Synthetic",
        Prerequisites::new(
            guid(2),
            purpose,
            InformationDeliveryMilestone::new(guid(4), text("Gate")),
            Actor::new(guid(5), text("Author")),
            Actor::new(guid(6), text("Reviewer")),
        ),
    ));
    let xml = LoinDocument::from_model(&model)
        .expect("model is writable")
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let definition = xml.find("<Definition").expect("Definition present");
    let region = xml.find("<Region").expect("Region present");
    assert!(definition < region, "Definition was relocated: {xml}");
    assert!(xml.contains("Replaced"), "{xml}");
}

/// Setting an optional item to `None` removes it without disturbing the rest.
#[test]
fn clearing_an_optional_purpose_item_preserves_remaining_order() {
    let mut purpose = Purpose::new(guid(3), text("Coordination"));
    purpose.add_item(PurposeItem::Definition(text("Definition")));
    purpose.add_item(PurposeItem::Region("DE".to_owned()));

    purpose
        .set_definition(None)
        .expect("purpose keeps its Name");

    assert!(matches!(purpose.items()[0], PurposeItem::Name(_)));
    assert!(matches!(purpose.items()[1], PurposeItem::Region(_)));
    assert_eq!(purpose.items().len(), 2);
}

/// Removing the only remaining item must be refused, not silently emptied.
#[test]
fn clearing_the_last_purpose_item_is_refused() {
    let mut purpose = Purpose::from_item(guid(3), PurposeItem::Definition(text("Only")));
    assert!(purpose.set_definition(None).is_err());
    assert_eq!(purpose.items().len(), 1);
}

/// GeoReferencing is LOIN-owned, so it must be written, not refused. The
/// fully populated state (CRS with vertical datum, datum descriptions, a model
/// coordinate system with every optional decimal) exercises the whole subtree.
#[test]
fn georeferencing_is_written_and_validates() {
    use openbim_loin::{
        dt::Decimal, CoordinateReferenceSystem, CoordinateReferenceSystemKind, Datum,
        DatumRegistryReference, GeoReferencing, ModelCoordinateSystem,
    };
    let decimal = |v: &str| -> Decimal { v.parse().expect("valid decimal") };
    let mut registry = DatumRegistryReference::new(text("EPSG"));
    registry.add_description(text("European Petroleum Survey Group"));
    let mut crs = CoordinateReferenceSystem::new(
        CoordinateReferenceSystemKind::ProjectedCrs,
        Datum::new("ETRS89", registry.clone()),
    );
    crs.set_vertical_datum(Some(Datum::new("DHHN2016", registry)));
    let mut system = ModelCoordinateSystem::new(
        true,
        decimal("32500000.5"),
        decimal("5650000"),
        decimal("120.25"),
    );
    system.x_axis_abscissa = Some(decimal("1"));
    system.x_axis_ordinate = Some(decimal("0"));
    system.unit_scale = Some(decimal("0.001"));
    system.horizontal_scale = Some(decimal("0.9996"));
    let mut geo = GeoReferencing::new();
    geo.set_coordinate_reference_system(Some(crs));
    geo.add_model_coordinate_system(system);

    let model = model_with_geo(geo);
    let document = LoinDocument::from_model(&model).expect("georeferencing is writable");
    assert!(errors(&document).is_empty(), "{:#?}", errors(&document));
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    for needle in [
        "<GeoReferencing>",
        "<Type>ProjectedCRS</Type>",
        "<VerticalDatum>",
        "<Description language=\"en\">European Petroleum Survey Group</Description>",
        "<IsProjected>true</IsProjected>",
        "<FirstCoordinate>32500000.5</FirstCoordinate>",
        "<HorizontalScale>0.9996</HorizontalScale>",
    ] {
        assert!(xml.contains(needle), "missing {needle} in {xml}");
    }
    let reparsed = LoinDocument::parse(&xml).expect("authored georeferencing reparses");
    assert!(errors(&reparsed).is_empty(), "{:#?}", errors(&reparsed));
    assert_eq!(
        xml,
        reparsed
            .to_xml_string(OutputNamespace::Preserve)
            .expect("serializes")
    );
}

/// The one DT-owned leaf inside GeoReferencing is still refused by name,
/// rather than the whole subtree being blamed on openbim-dt.
#[test]
fn georeferencing_registry_reference_is_refused_as_dt_content() {
    use openbim_loin::{
        CoordinateReferenceSystem, CoordinateReferenceSystemKind, Datum, DatumRegistryReference,
        GeoReferencing,
    };
    let reference: openbim_loin::dt::Reference =
        openbim_loin::dt::Reference::new(None, Some("https://epsg.io/4258".parse().expect("uri")));
    let mut registry = DatumRegistryReference::new(text("EPSG"));
    registry.set_registry_reference(Some(reference));
    let crs = CoordinateReferenceSystem::new(
        CoordinateReferenceSystemKind::GeographicCrs,
        Datum::new("ETRS89", registry),
    );
    let mut geo = GeoReferencing::new();
    geo.set_coordinate_reference_system(Some(crs));
    let model = model_with_geo(geo);
    let error = LoinDocument::from_model(&model).expect_err("registry reference is DT-owned");
    assert!(
        matches!(
            error,
            AuthoringError::UnwritableDtContent {
                element: "RegistryReference",
                ..
            }
        ),
        "{error:?}"
    );
}

/// The writer used to drop the milestone `Date` and the actor name
/// attributes silently. Found by the phase-4 reader: a read -> write -> read
/// round trip of an official example lost the milestone date.
#[test]
fn optional_attributes_survive_write_then_read() {
    let mut milestone = InformationDeliveryMilestone::new(guid(4), text("Gate"));
    milestone.set_date(Some(created()));
    let mut providing = Actor::new(guid(5), text("Author"));
    providing.set_first_name(Some("Ada".into()));
    providing.set_middle_name(Some("B.".into()));
    providing.set_last_name(Some("Author".into()));
    providing.set_affiliation(Some("Synthetic Works".into()));
    let prerequisites = Prerequisites::new(
        guid(2),
        Purpose::new(guid(3), text("Coordination")),
        milestone,
        providing,
        Actor::new(guid(6), text("Reviewer")),
    );
    let model =
        LevelOfInformationNeed::new(Specification::new(guid(1), "Synthetic", prerequisites));

    let document = LoinDocument::from_model(&model).expect("model is writable");
    assert!(errors(&document).is_empty(), "{:#?}", errors(&document));
    let xml = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("serializes");
    let reread =
        LevelOfInformationNeed::from_document(&LoinDocument::parse(&xml).expect("reparses"))
            .expect("reads back");
    assert_eq!(reread, model, "{xml}");
}
