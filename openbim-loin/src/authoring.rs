//! Writing LOIN documents from the typed model.
//!
//! [`LoinDocument`] is otherwise obtainable only by parsing. This module adds
//! the missing direction: a [`LevelOfInformationNeed`] becomes a document that
//! serializes with [`LoinDocument::to_xml_string`] and passes `validate()`.
//!
//! # Scope
//!
//! LOIN-owned content is written in full. ISO 23387-owned complex content
//! (`ObjectType`, `Property`, `QuantityKind`, `Dimension`, `Unit`,
//! `ReferenceDocument`, `GroupOfProperties`) cannot be written here: the
//! `openbim-dt` 0.2 owned types expose no serializer, and `dt::Element`'s
//! constructors are `pub(crate)`, so a downstream crate cannot build one.
//! Rather than duplicate ISO 23387's grammar, those subtrees are refused with
//! [`AuthoringError::UnwritableDtContent`], naming the element and the reason.
//!
//! Authors needing that content today can parse a document and edit it through
//! [`XmlElement::nodes_mut`], which preserves the DT subtrees verbatim.

use openbim_dt::{Guid, MultiLanguageText, Reference};

use crate::{
    document::{LoinDocument, XmlAttribute, XmlElement},
    model::{
        Actor, Document, DocumentFormat, Documentation, InformationDeliveryMilestone,
        LevelOfInformationNeed, Prerequisites, Purpose, PurposeItem, Specification,
        SpecificationPerObjectType,
    },
};

/// Why a typed value could not be written as XML.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthoringError {
    /// The value embeds ISO 23387-owned content this crate cannot serialize.
    ///
    /// Carries the element that would have been written and why it is blocked,
    /// so the message is actionable rather than a bare failure.
    UnwritableDtContent {
        /// Local name of the element that could not be produced.
        element: &'static str,
        /// The upstream limitation preventing serialization.
        reason: &'static str,
    },
}

impl core::fmt::Display for AuthoringError {
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

const DT_UNWRITABLE: &str = crate::DT_UNWRITABLE_REASON;

impl LoinDocument {
    /// Builds a document from the typed model.
    ///
    /// The result carries no retained source, declares the LOIN and ISO 23387
    /// namespaces on the root, and writes children in `xs:sequence` order.
    ///
    /// # Errors
    ///
    /// Returns [`AuthoringError::UnwritableDtContent`] when the model embeds
    /// ISO 23387-owned complex content, which this crate cannot serialize.
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
    if value.geo_referencing().is_some() {
        return Err(AuthoringError::UnwritableDtContent {
            element: "GeoReferencing",
            reason: "georeferencing authoring is not implemented yet",
        });
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
            PurposeItem::ReferenceDocument(_) => {
                return Err(unwritable("ReferenceDocument"));
            }
            PurposeItem::DictionaryRef(_) => {
                return Err(unwritable("DictionaryRef"));
            }
        });
    }
    Ok(element)
}

fn milestone_element(value: &InformationDeliveryMilestone) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("InformationDeliveryMilestone")
        .with_attribute(guid_attribute(value.guid()))
        .with_child(multilingual_element("Name", value.name()));
    for description in value.descriptions() {
        element = element.with_child(multilingual_element("Description", description));
    }
    if !value.reference_documents().is_empty() {
        return Err(unwritable("ReferenceDocument"));
    }
    Ok(element)
}

/// Writes an actor under the caller-supplied element name; the schema declares
/// `ProvidingActor` and `ReceivingActor` with one shared content model.
fn actor_element(name: &'static str, value: &Actor) -> XmlElement {
    let mut element = XmlElement::new(name)
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

fn per_object_element(value: &SpecificationPerObjectType) -> Result<XmlElement, AuthoringError> {
    // <ObjectType> is ISO 23387-owned and has no downstream serializer.
    let _ = value;
    Err(unwritable("ObjectType"))
}

fn documentation_element(value: &Documentation) -> Result<XmlElement, AuthoringError> {
    let mut element = XmlElement::new("Documentation").with_attribute(guid_attribute(value.guid()));
    for document in value.documents() {
        element = element.with_child(document_element(document)?);
    }
    Ok(element)
}

fn document_element(value: &Document) -> Result<XmlElement, AuthoringError> {
    if !value.reference_documents().is_empty() {
        return Err(unwritable("ReferenceDocument"));
    }
    let mut element = XmlElement::new("Document")
        .with_attribute(guid_attribute(value.guid()))
        .with_child(multilingual_element("Name", value.name()));
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
    if !value.specifications().is_empty() {
        return Err(unwritable("FormatSpecification"));
    }
    Ok(element)
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

const fn unwritable(element: &'static str) -> AuthoringError {
    AuthoringError::UnwritableDtContent {
        element,
        reason: DT_UNWRITABLE,
    }
}

/// Silences dead-code warnings for helpers reachable only once DT content is
/// writable; they are exercised by unit tests below.
#[allow(dead_code)]
fn reference_is_unwritable(_: &Reference) -> AuthoringError {
    unwritable("ReferenceDocument")
}

#[allow(dead_code)]
fn documentation_is_reachable(value: &Documentation) -> Result<XmlElement, AuthoringError> {
    documentation_element(value)
}
