//! Writing LOIN documents from the typed model.
//!
//! [`LoinDocument`] is otherwise obtainable only by parsing. This module adds
//! the missing direction: a [`LevelOfInformationNeed`] becomes a document that
//! serializes with [`LoinDocument::to_xml_string`] and passes `validate()`.
//!
//! # Scope
//!
//! Every value the typed model holds is written. LOIN-owned content is
//! written here; ISO 23387-owned subtrees (`ObjectType`, `Property`,
//! `QuantityKind`, `Dimension`, `Unit`, `ReferenceDocument`,
//! `GroupOfProperties`, and the concept a `SpecificationPerObjectType`
//! extends) are encoded by `openbim-dt`'s own codec and re-rooted under the
//! LOIN element name, so this crate never re-implements the ISO 23387 grammar.
//! That mirrors [`LevelOfInformationNeed::from_document`], which decodes the
//! same subtrees through `openbim-dt`.
//!
//! The writer is a semantic projection like the reader: comments, prefixes
//! and whitespace from a parsed source are not carried by the model. Edit the
//! tree through [`XmlElement::nodes_mut`] when those must survive.

use std::sync::Arc;

use openbim_dt as dt;
use openbim_dt::{Decimal, Guid, MultiLanguageText, Reference};

use crate::{
    document::{LoinDocument, XmlAttribute, XmlElement, XmlNode},
    model::{
        Actor, AlphanumericalInformation, CoordinateReferenceSystem, CoordinateReferenceSystemKind,
        Datum, Detail, Document, DocumentFormat, Documentation, GeoReferencing,
        GeometricalInformation, GroupsOfProperties, InformationDeliveryMilestone,
        LevelOfInformationNeed, Location, ModelCoordinateSystem, Prerequisites, Purpose,
        PurposeItem, ShapeInfluence, Specification, SpecificationPerObjectType, ThresholdDimension,
    },
};

/// Why a typed value could not be written as XML.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthoringError {
    /// The value embeds ISO 23387-owned content this crate cannot serialize.
    ///
    /// No longer returned: since `openbim-dt` 0.3 every owned type has a
    /// codec, and [`LoinDocument::from_model`] writes all of them. Kept so
    /// existing `match` arms compile; the error type stays for future refusals.
    #[deprecated(
        since = "0.4.0",
        note = "never returned: ISO 23387 content is written through openbim-dt's codec"
    )]
    UnwritableDtContent {
        /// Local name of the element that could not be produced.
        element: &'static str,
        /// The upstream limitation preventing serialization.
        reason: &'static str,
    },
}

impl core::fmt::Display for AuthoringError {
    #[allow(deprecated)]
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnwritableDtContent { element, reason } => write!(
                formatter,
                "cannot write <{element}>: {reason}; parse and edit via XmlElement::nodes_mut instead"
            ),
        }
    }
}

impl std::error::Error for AuthoringError {}

impl LoinDocument {
    /// Builds a document from the typed model.
    ///
    /// The result carries no retained source, declares the LOIN and ISO 23387
    /// namespaces on the root, and writes children in `xs:sequence` order.
    ///
    /// ISO 23387-owned subtrees are encoded by `openbim-dt` and embedded
    /// under their LOIN element names, so
    /// `LevelOfInformationNeed::from_document(&LoinDocument::from_model(m)?)`
    /// returns `m` for every model.
    ///
    /// # Errors
    ///
    /// Every current model value is writable, so this returns `Ok`. The
    /// `Result` is kept so a future refusal is not a breaking change.
    pub fn from_model(model: &LevelOfInformationNeed) -> Result<Self, AuthoringError> {
        let mut root = XmlElement::new_root("LevelOfInformationNeed");
        for specification in model.specifications() {
            root = root.with_child(specification_element(specification)?);
        }
        Ok(Self::authored(root))
    }
}

/// Writes `<Specification>`: Prerequisites, then per-object types, then
/// GeoReferencing, matching the declared sequence.
fn specification_element(value: &Specification) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Specification")
        .with_attribute(XmlAttribute::new("name", value.name()))
        .with_attribute(guid_attribute(value.guid()))
        .with_child(prerequisites_element(value.prerequisites())?);
    for per_object in value.per_object() {
        element = element.with_child(per_object_element(per_object)?);
    }
    if let Some(geo) = value.geo_referencing() {
        element = element.with_child(geo_referencing_element(geo)?);
    }
    Ok(element)
}

fn prerequisites_element(value: &Prerequisites) -> Result<XmlElement, AuthoringError> {
    Ok(XmlElement::new("Prerequisites")
        .with_attribute(guid_attribute(value.guid()))
        .with_child(purpose_element(value.purpose())?)
        .with_child(milestone_element(value.milestone())?)
        .with_child(actor_element("ProvidingActor", value.providing_actor()))
        .with_child(actor_element("ReceivingActor", value.receiving_actor())))
}

/// Writes `<Purpose>`. Its content model is a repeating choice, so items are
/// emitted in the order the model holds them rather than a fixed order.
fn purpose_element(value: &Purpose) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Purpose").with_attribute(guid_attribute(value.guid()));
    for item in value.items() {
        element = element.with_child(match item {
            PurposeItem::Name(text) => multilingual_element("Name", text),
            PurposeItem::Definition(text) => multilingual_element("Definition", text),
            PurposeItem::Description(text) => multilingual_element("Description", text),
            PurposeItem::Language(language) => {
                XmlElement::new("Language").with_text(language.as_str())
            }
            PurposeItem::Region(region) => XmlElement::new("Region").with_text(region),
            PurposeItem::ReferenceDocument(reference) => {
                reference_element("ReferenceDocument", reference)
            }
            PurposeItem::DictionaryRef(reference) => reference_element("DictionaryRef", reference),
        });
    }
    Ok(element)
}

fn milestone_element(value: &InformationDeliveryMilestone) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("InformationDeliveryMilestone")
        .with_attribute(guid_attribute(value.guid()));
    if let Some(date) = value.date() {
        element = element.with_attribute(XmlAttribute::new("Date", date.as_str()));
    }
    element = element.with_child(multilingual_element("Name", value.name()));
    for description in value.descriptions() {
        element = element.with_child(multilingual_element("Description", description));
    }
    for reference in value.reference_documents() {
        element = element.with_child(reference_element("ReferenceDocument", reference));
    }
    Ok(element)
}

/// Writes an actor under the caller-supplied element name; the schema declares
/// `ProvidingActor` and `ReceivingActor` with one shared content model.
fn actor_element(name: &'static str, value: &Actor) -> XmlElement {
    let mut element = XmlElement::new(name);
    for (attribute, text) in [
        ("firstName", value.first_name()),
        ("middleName", value.middle_name()),
        ("lastName", value.last_name()),
        ("affiliation", value.affiliation()),
    ] {
        if let Some(text) = text {
            element = element.with_attribute(XmlAttribute::new(attribute, text));
        }
    }
    element = element
        .with_attribute(guid_attribute(value.guid()))
        .with_child(multilingual_element("Role", value.role()));
    if let Some(description) = value.description() {
        element = element.with_child(multilingual_element("Description", description));
    }
    if let Some(email) = value.email_address() {
        element = element.with_child(XmlElement::new("EMailAddress").with_text(email.as_str()));
    }
    element
}

/// Writes `<SpecificationPerObjectType>`, which extends `dt:ConceptType`: the
/// concept's attributes and `dt:` children come first (encoded by
/// `openbim-dt`), then `ObjectType` and the optional LOIN extension.
fn per_object_element(value: &SpecificationPerObjectType) -> Result<XmlElement, AuthoringError> {
    let mut element = embed_dt(
        "SpecificationPerObjectType",
        &value
            .concept()
            .to_element_named("SpecificationPerObjectType"),
    )
    .with_child(embed_dt("ObjectType", &value.object_type().to_element()));
    if let Some(alpha) = value.alphanumerical_information() {
        element = element.with_child(alphanumerical_element(alpha));
    }
    if let Some(documentation) = value.documentation() {
        element = element.with_child(documentation_element(documentation)?);
    }
    if let Some(geometry) = value.geometrical_information() {
        element = element.with_child(geometrical_element(geometry));
    }
    Ok(element)
}

/// Writes `<AlphanumericalInformation>` in its declared sequence. Every child
/// apart from the `GroupsOfProperties` container is ISO 23387-owned.
fn alphanumerical_element(value: &AlphanumericalInformation) -> XmlElement {
    let mut element =
        XmlElement::new("AlphanumericalInformation").with_attribute(guid_attribute(value.guid()));
    for property in value.properties() {
        element = element.with_child(embed_dt("Property", &property.to_element()));
    }
    for kind in value.quantity_kinds() {
        element = element.with_child(embed_dt("QuantityKind", &kind.to_element()));
    }
    if let Some(groups) = value.groups_container() {
        element = element.with_child(groups_element(groups));
    }
    for document in value.reference_documents() {
        element = element.with_child(embed_dt("ReferenceDocument", &document.to_element()));
    }
    for dimension in value.dimensions() {
        element = element.with_child(embed_dt("Dimension", &dimension.to_element()));
    }
    for unit in value.units() {
        element = element.with_child(embed_dt("Unit", &unit.to_element()));
    }
    element
}

fn groups_element(value: &GroupsOfProperties) -> XmlElement {
    let mut element = XmlElement::new("GroupsOfProperties");
    for group in value.groups() {
        element = element.with_child(embed_dt("GroupOfProperties", &group.to_element()));
    }
    for reference in value.references() {
        element = element.with_child(reference_element("GroupOfPropertiesRef", reference));
    }
    element
}

fn documentation_element(value: &Documentation) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Documentation").with_attribute(guid_attribute(value.guid()));
    for document in value.documents() {
        element = element.with_child(document_element(document)?);
    }
    Ok(element)
}

fn document_element(value: &Document) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Document");
    for (attribute, text) in [
        ("type", value.document_type()),
        ("form", value.form()),
        ("content", value.content()),
    ] {
        if let Some(text) = text {
            element = element.with_attribute(XmlAttribute::new(attribute, text));
        }
    }
    element = element
        .with_attribute(guid_attribute(value.guid()))
        .with_child(multilingual_element("Name", value.name()));
    for reference in value.reference_documents() {
        element = element.with_child(reference_element("ReferenceDocument", reference));
    }
    for description in value.descriptions() {
        element = element.with_child(multilingual_element("Description", description));
    }
    Ok(element.with_child(format_element(value.format())?))
}

fn format_element(value: &DocumentFormat) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Format");
    for name in value.names() {
        element = element.with_child(multilingual_element("FormatName", name));
    }
    for version in value.versions() {
        element = element.with_child(multilingual_element("FormatVersion", version));
    }
    for specification in value.specifications() {
        element = element.with_child(reference_element("FormatSpecification", specification));
    }
    Ok(element)
}

/// Writes `<GeometricalInformation>`: the optional `placeholder` attribute,
/// then Detail, Dimensionality, Appearance, ParametricBehaviour and Location
/// in their declared sequence.
fn geometrical_element(value: &GeometricalInformation) -> XmlElement {
    let mut element =
        XmlElement::new("GeometricalInformation").with_attribute(guid_attribute(value.guid()));
    if let Some(placeholder) = value.placeholder() {
        element = element.with_attribute(XmlAttribute::new("placeholder", boolean(placeholder)));
    }
    if let Some(detail) = value.detail() {
        element = element.with_child(detail_element(detail));
    }
    element = with_token(
        element,
        "Dimensionality",
        value.dimensionality().map(|v| v.as_str()),
    );
    element = with_token(
        element,
        "Appearance",
        value.appearance().map(|v| v.as_str()),
    );
    element = with_token(
        element,
        "ParametricBehaviour",
        value.parametric_behaviour().map(|v| v.as_str()),
    );
    if let Some(location) = value.location() {
        element = element.with_child(location_element(location));
    }
    element
}

fn detail_element(value: &Detail) -> XmlElement {
    let mut element = XmlElement::new("Detail");
    if let Some(dictionary) = value.dictionary() {
        element = element.with_child(reference_element("Dictionary", dictionary));
    }
    element = with_token(
        element,
        "ShapeAssembly",
        value.shape_assembly().map(|v| v.as_str()),
    );
    element = with_token(
        element,
        "ShapeRepresentation",
        value.shape_representation().map(|v| v.as_str()),
    );
    if let Some(influence) = value.shape_influence() {
        element = element.with_child(shape_influence_element(influence));
    }
    element
}

/// Writes `<ShapeInfluence>`, walking the fields in their declared order
/// (see the field-order note on [`ShapeInfluence`]).
fn shape_influence_element(value: &ShapeInfluence) -> XmlElement {
    let mut element = XmlElement::new("ShapeInfluence");
    for (name, token) in [
        ("InsideGeometry", value.inside_geometry.map(|v| v.as_str())),
        ("Connections", value.connections.map(|v| v.as_str())),
        ("Openings", value.openings.map(|v| v.as_str())),
        (
            "OperatingAndClearanceZones",
            value.operating_and_clearance_zones.map(|v| v.as_str()),
        ),
        ("Features", value.features.map(|v| v.as_str())),
    ] {
        element = with_token(element, name, token);
    }
    if let Some(threshold) = &value.threshold_dimension {
        element = element.with_child(threshold_element(threshold));
    }
    element
}

/// Writes `<ThresholdDimension>`: Threshold, the DT-owned Unit, Definition.
fn threshold_element(value: &ThresholdDimension) -> XmlElement {
    XmlElement::new("ThresholdDimension")
        .with_child(XmlElement::new("Threshold").with_text(double(value.threshold())))
        .with_child(embed_dt("Unit", &value.unit().to_element()))
        .with_child(multilingual_element("Definition", value.definition()))
}

fn location_element(value: &Location) -> XmlElement {
    let element = XmlElement::new("Location").with_child(
        XmlElement::new("RelativeOrAbsolute").with_text(value.relative_or_absolute.as_str()),
    );
    match &value.reference_object {
        Some(object) => element.with_child(XmlElement::new("ReferenceObject").with_text(object)),
        None => element,
    }
}

/// Appends `<name>token</name>` when `token` is present.
fn with_token(element: XmlElement, name: &'static str, token: Option<&str>) -> XmlElement {
    match token {
        Some(token) => element.with_child(XmlElement::new(name).with_text(token)),
        None => element,
    }
}

const fn boolean(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// `xs:double` lexical form. Rust spells the specials `inf` and `NaN`; XML
/// Schema requires `INF`, `-INF` and `NaN`. Finite values use Rust's shortest
/// round-trip form, which is inside the `xs:double` lexical space.
fn double(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value.is_infinite() {
        if value > 0.0 { "INF" } else { "-INF" }.to_owned()
    } else {
        value.to_string()
    }
}

/// Writes `<GeoReferencing>`: the optional CRS, then every model coordinate
/// system, matching the declared sequence.
fn geo_referencing_element(value: &GeoReferencing) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("GeoReferencing");
    if let Some(crs) = value.coordinate_reference_system() {
        element = element.with_child(crs_element(crs)?);
    }
    for system in value.model_coordinate_systems() {
        element = element.with_child(model_coordinate_system_element(system));
    }
    Ok(element)
}

fn crs_element(value: &CoordinateReferenceSystem) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("CoordinateReferenceSystem")
        .with_child(XmlElement::new("Type").with_text(crs_kind_str(value.crs_type())))
        .with_child(datum_element("Datum", &value.datum)?);
    if let Some(vertical) = &value.vertical_datum {
        element = element.with_child(datum_element("VerticalDatum", vertical)?);
    }
    Ok(element)
}

fn datum_element(name: &'static str, value: &Datum) -> Result<XmlElement, AuthoringError> {
    let registry = value.datum_type();
    let mut datum_type =
        XmlElement::new("Type").with_child(multilingual_element("Name", registry.name()));
    for description in registry.descriptions() {
        datum_type = datum_type.with_child(multilingual_element("Description", description));
    }
    if let Some(reference) = registry.registry_reference() {
        datum_type = datum_type.with_child(reference_element("RegistryReference", reference));
    }
    Ok(XmlElement::new(name)
        .with_child(XmlElement::new("Name").with_text(value.name()))
        .with_child(datum_type))
}

fn model_coordinate_system_element(value: &ModelCoordinateSystem) -> XmlElement {
    let decimal = |name: &'static str, v: &Decimal| XmlElement::new(name).with_text(v.as_str());
    let mut element = XmlElement::new("ModelCoordinateSystem")
        .with_child(XmlElement::new("IsProjected").with_text(boolean(value.is_projected)))
        .with_child(decimal("FirstCoordinate", &value.first_coordinate))
        .with_child(decimal("SecondCoordinate", &value.second_coordinate))
        .with_child(decimal("Height", &value.height));
    for (name, optional) in [
        ("XAxisAbscissa", &value.x_axis_abscissa),
        ("XAxisOrdinate", &value.x_axis_ordinate),
        ("UnitScale", &value.unit_scale),
        ("HorizontalScale", &value.horizontal_scale),
    ] {
        if let Some(v) = optional {
            element = element.with_child(decimal(name, v));
        }
    }
    element
}

/// XSD spelling of the CRS kind. Exhaustive so a new variant fails to compile
/// here rather than writing a value the schema rejects.
const fn crs_kind_str(value: CoordinateReferenceSystemKind) -> &'static str {
    match value {
        CoordinateReferenceSystemKind::NotRequired => "NotRequired",
        CoordinateReferenceSystemKind::ProjectedCrs => "ProjectedCRS",
        CoordinateReferenceSystemKind::EngineeringCrs => "EngineeringCRS",
        CoordinateReferenceSystemKind::GeographicCrs => "GeographicCRS",
    }
}

/// Writes a LOIN multilingual element: text content plus `@language`.
fn multilingual_element(name: &'static str, value: &MultiLanguageText) -> XmlElement {
    XmlElement::new(name)
        .with_attribute(XmlAttribute::new("language", value.language()))
        .with_text(value.text())
}

fn guid_attribute(value: &Guid) -> XmlAttribute {
    XmlAttribute::new_dt("GUID", value.as_str())
}

/// Writes a `dt:ReferenceType` value under a LOIN element name: the
/// reference is carried by `dt:`-qualified attributes on an empty element.
fn reference_element(name: &'static str, value: &Reference) -> XmlElement {
    let mut element = XmlElement::new(name);
    if let Some(guid) = value.guid() {
        element = element.with_attribute(guid_attribute(guid));
    }
    if let Some(uri) = value.uri() {
        element = element.with_attribute(XmlAttribute::new_dt("referenceURI", uri));
    }
    element.as_empty_element()
}

/// Embeds an element encoded by `openbim-dt` under the LOIN-local, unqualified
/// name `local_name`. ISO 7817-3 declares these elements locally with a DT
/// type, so only the outer name changes; attributes and content are DT's.
fn embed_dt(local_name: &'static str, element: &dt::Element) -> XmlElement {
    let mut embedded = XmlElement::new(local_name);
    embedded
        .attributes_mut()
        .extend(element.attributes().iter().map(from_dt_attribute));
    push_dt_content(&mut embedded, element);
    embedded
}

/// Converts an `openbim-dt` element verbatim: names, namespaces, attributes,
/// text and children.
fn from_dt(element: &dt::Element) -> XmlElement {
    let mut converted = XmlElement::parsed(
        element.qname().to_owned(),
        element.prefix().map(str::to_owned),
        element.local_name().to_owned(),
        element.namespace_uri().map(Arc::from),
        element.attributes().iter().map(from_dt_attribute).collect(),
        false,
    );
    push_dt_content(&mut converted, element);
    converted
}

fn push_dt_content(target: &mut XmlElement, element: &dt::Element) {
    for node in element.nodes() {
        target.push_node(match node {
            dt::Node::Element(child) => XmlNode::Element(from_dt(child)),
            dt::Node::Text(text) => XmlNode::Text(text.clone()),
            dt::Node::CData(text) => XmlNode::CData(text.clone()),
            dt::Node::Comment(text) => XmlNode::Comment(text.clone()),
            dt::Node::ProcessingInstruction(text) => XmlNode::ProcessingInstruction(text.clone()),
        });
    }
}

fn from_dt_attribute(attribute: &dt::Attribute) -> XmlAttribute {
    XmlAttribute::parsed(
        attribute.qname().to_owned(),
        attribute.prefix().map(str::to_owned),
        attribute.local_name().to_owned(),
        attribute.namespace_uri().map(Arc::from),
        attribute.value().to_owned(),
    )
}
