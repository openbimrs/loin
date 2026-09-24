//! Reading the typed model from a parsed LOIN document.
//!
//! [`LevelOfInformationNeed::from_document`] turns a [`LoinDocument`] into the
//! typed model. It is the inverse of [`LoinDocument::from_model`] for every
//! value the writer can produce.
//!
//! # Contract
//!
//! * **Strict.** Anything the model cannot hold is refused with a
//!   [`ReadError`] naming the element, never dropped: unknown or misplaced
//!   elements, unknown attributes, character data in element-only content,
//!   `xsi:nil`, and values outside their lexical space.
//! * **Not a validator.** Structural rules the model does not encode (for
//!   example the order of `Purpose` items beyond the one retained) are left to
//!   [`LoinDocument::validate`], which reports every problem instead of stopping
//!   at the first.
//! * **Semantic projection.** Comments, processing instructions, namespace
//!   prefixes and whitespace are not retained. Edit the [`LoinDocument`] tree
//!   when those must survive.
//!
//! ISO 23387-owned subtrees (`ObjectType`, `Property`, `Unit`, references,
//! ...) are converted to `openbim_dt` elements and decoded by `openbim-dt`
//! itself, so this crate never re-implements the ISO 23387 grammar.

use core::fmt;
use std::{error::Error, str::FromStr};

use openbim_dt as dt;
use openbim_dt::{Concept, DateTime, Decimal, Guid, Language, MultiLanguageText, Reference};

use crate::{
    document::{LoinDocument, XmlElement, XmlNode},
    model::{
        Actor, AlphanumericalInformation, Appearance, Connections, CoordinateReferenceSystem,
        CoordinateReferenceSystemKind, Datum, DatumRegistryReference, Detail, Dimensionality,
        Document, DocumentFormat, Documentation, EmailAddress, Features, GeoReferencing,
        GeometricalInformation, GroupsOfProperties, InformationDeliveryMilestone, InsideGeometry,
        LevelOfInformationNeed, Location, ModelCoordinateSystem, Openings,
        OperatingAndClearanceZones, ParametricBehaviour, Prerequisites, Purpose, PurposeItem,
        RelativeOrAbsolute, ShapeAssembly, ShapeInfluence, ShapeRepresentation, Specification,
        SpecificationPerObjectType, ThresholdDimension,
    },
    NamespaceVersion,
};

const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Why a document could not be read into the typed model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReadErrorKind {
    /// The root is not a `LevelOfInformationNeed` in a known LOIN namespace.
    UnexpectedRoot,
    /// A required child element is absent.
    MissingElement,
    /// A required attribute is absent.
    MissingAttribute,
    /// An element the model does not represent, or one in the wrong place.
    UnexpectedElement,
    /// An attribute the model does not represent.
    UnexpectedAttribute,
    /// Character data where the schema declares element-only content.
    UnexpectedText,
    /// A value outside its lexical space (GUID, date, decimal, enumeration...).
    InvalidValue,
    /// `xsi:nil`: a nilled `SpecificationPerObjectType` has no typed form.
    NilledElement,
    /// ISO 23387 content that `openbim-dt` refused; see the detail.
    DataTemplate,
}

/// A read failure: what went wrong and where.
///
/// `path` uses the format of [`crate::Diagnostic::path`]:
/// `/LevelOfInformationNeed/Specification[1]/Prerequisites[1]`, where each
/// index is the element's 1-based position among all its sibling elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadError {
    kind: ReadErrorKind,
    path: String,
    detail: String,
}

impl ReadError {
    fn new(kind: ReadErrorKind, path: &str, detail: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.to_owned(),
            detail: detail.into(),
        }
    }

    /// The stable failure category.
    #[must_use]
    pub const fn kind(&self) -> ReadErrorKind {
        self.kind
    }

    /// Location of the offending element (attributes as `/@name`).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Human-readable explanation.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at {}: {}",
            self.kind, self.path, self.detail
        )
    }
}

impl Error for ReadError {}

type Result<T> = core::result::Result<T, ReadError>;

impl LevelOfInformationNeed {
    /// Reads the typed model from a parsed document.
    ///
    /// ```
    /// use openbim_loin::{LevelOfInformationNeed, LoinDocument};
    /// # let xml = r#"<loin:LevelOfInformationNeed xmlns:loin="https://iso.org/2024/LOIN" xmlns:dt="https://standards.iso.org/iso/23387/ed-2/en/"><Specification name="S" dt:GUID="00000000-0000-4000-8000-000000000001"><Prerequisites dt:GUID="00000000-0000-4000-8000-000000000002"><Purpose dt:GUID="00000000-0000-4000-8000-000000000003"><Name language="en">P</Name></Purpose><InformationDeliveryMilestone dt:GUID="00000000-0000-4000-8000-000000000004"><Name language="en">M</Name></InformationDeliveryMilestone><ProvidingActor dt:GUID="00000000-0000-4000-8000-000000000005"><Role language="en">A</Role></ProvidingActor><ReceivingActor dt:GUID="00000000-0000-4000-8000-000000000006"><Role language="en">B</Role></ReceivingActor></Prerequisites></Specification></loin:LevelOfInformationNeed>"#;
    /// let document = LoinDocument::parse(xml)?;
    /// let model = LevelOfInformationNeed::from_document(&document)?;
    /// assert_eq!(model.specifications()[0].name(), "S");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError`] for the first element or value the model cannot
    /// represent. See the module documentation for the exact contract.
    pub fn from_document(document: &LoinDocument) -> Result<Self> {
        let root = document.root();
        let path = "/LevelOfInformationNeed";
        let known_namespace = root
            .namespace_uri()
            .and_then(NamespaceVersion::from_uri)
            .is_some();
        if !known_namespace || root.local_name() != "LevelOfInformationNeed" {
            return Err(ReadError::new(
                ReadErrorKind::UnexpectedRoot,
                path,
                format!(
                    "expected LevelOfInformationNeed in a LOIN namespace, found {} ({})",
                    root.local_name(),
                    root.namespace_uri().unwrap_or("no namespace")
                ),
            ));
        }
        let node = Node::new(root, path.to_owned())?;
        node.attributes(&[])?;
        let mut children = node.children();
        let mut specifications = children
            .take_all("Specification")
            .into_iter()
            .map(|child| read_specification(&child));
        let first = specifications.next().ok_or_else(|| {
            ReadError::new(
                ReadErrorKind::MissingElement,
                path,
                "at least one Specification is required",
            )
        })??;
        let mut model = Self::new(first);
        for specification in specifications {
            model.add_specification(specification?);
        }
        children.finish()?;
        Ok(model)
    }
}

// ---------------------------------------------------------------------------
// Tree walking
// ---------------------------------------------------------------------------

/// One element plus its diagnostic path.
struct Node<'a> {
    element: &'a XmlElement,
    path: String,
}

impl<'a> Node<'a> {
    /// Wraps `element`, refusing non-whitespace character data (every LOIN
    /// complex type is element-only).
    fn new(element: &'a XmlElement, path: String) -> Result<Self> {
        let node = Self { element, path };
        if node.has_text() {
            return Err(node.error(
                ReadErrorKind::UnexpectedText,
                "character data in element-only content",
            ));
        }
        Ok(node)
    }

    /// Wraps a simple-content element; its text is read with [`Node::text`].
    fn simple(element: &'a XmlElement, path: String) -> Result<Self> {
        let node = Self { element, path };
        if let Some(child) = node.element.children().next() {
            return Err(node.error(
                ReadErrorKind::UnexpectedElement,
                format!(
                    "{} has simple content; found element {}",
                    node.name(),
                    child.qname()
                ),
            ));
        }
        Ok(node)
    }

    fn name(&self) -> &str {
        self.element.local_name()
    }

    fn error(&self, kind: ReadErrorKind, detail: impl Into<String>) -> ReadError {
        ReadError::new(kind, &self.path, detail)
    }

    fn attribute_error(
        &self,
        name: &str,
        kind: ReadErrorKind,
        detail: impl Into<String>,
    ) -> ReadError {
        ReadError::new(kind, &format!("{}/@{name}", self.path), detail)
    }

    fn has_text(&self) -> bool {
        self.element.nodes().iter().any(|node| {
            matches!(node, XmlNode::Text(text) | XmlNode::CData(text)
                if text.chars().any(|c| !is_xsd_whitespace(c)))
        })
    }

    /// Refuses any attribute outside `allowed` (namespace, local name).
    /// Namespace declarations and `xsi:schemaLocation` hints are syntax.
    fn attributes(&self, allowed: &[(Option<&str>, &str)]) -> Result<()> {
        for attribute in self.element.attributes() {
            let namespace = attribute.namespace_uri();
            let name = attribute.local_name();
            if namespace == Some(XMLNS_NAMESPACE) {
                continue;
            }
            if namespace == Some(XSI_NAMESPACE) {
                match name {
                    "schemaLocation" | "noNamespaceSchemaLocation" => continue,
                    "nil" => {
                        return Err(self.error(
                            ReadErrorKind::NilledElement,
                            "xsi:nil has no typed representation; read the document tree instead",
                        ))
                    }
                    _ => {}
                }
            }
            if !allowed
                .iter()
                .any(|(ns, local)| *ns == namespace && *local == name)
            {
                return Err(self.attribute_error(
                    attribute.qname(),
                    ReadErrorKind::UnexpectedAttribute,
                    format!(
                        "attribute {} ({}) is not represented by the model",
                        attribute.qname(),
                        namespace.unwrap_or("no namespace")
                    ),
                ));
            }
        }
        Ok(())
    }

    fn optional_attribute(&self, namespace: Option<&str>, name: &str) -> Option<&'a str> {
        self.element.attribute_ns(namespace, name)
    }

    fn required_attribute(&self, namespace: Option<&str>, name: &str) -> Result<&'a str> {
        self.optional_attribute(namespace, name).ok_or_else(|| {
            self.attribute_error(
                name,
                ReadErrorKind::MissingAttribute,
                format!("required attribute {name} is missing"),
            )
        })
    }

    /// The required `dt:GUID` attribute.
    fn guid(&self) -> Result<Guid> {
        let value = self.required_attribute(Some(dt::NAMESPACE), "GUID")?;
        Guid::from_str(value).map_err(|error| {
            self.attribute_error("GUID", ReadErrorKind::InvalidValue, error.to_string())
        })
    }

    /// Text content with XML Schema whitespace collapsed (for tokens:
    /// enumerations, numbers, booleans, languages, dates).
    fn token(&self) -> String {
        collapse_whitespace(&self.text())
    }

    /// Raw text content (for `xs:string` values, which preserve whitespace).
    fn text(&self) -> String {
        let mut value = String::new();
        for node in self.element.nodes() {
            if let XmlNode::Text(text) | XmlNode::CData(text) = node {
                value.push_str(text);
            }
        }
        value
    }

    fn parse<T, E: fmt::Display>(
        &self,
        parse: impl FnOnce(&str) -> core::result::Result<T, E>,
    ) -> Result<T> {
        let token = self.token();
        parse(&token).map_err(|error| {
            self.error(
                ReadErrorKind::InvalidValue,
                format!("{token:?} is not a valid {}: {error}", self.name()),
            )
        })
    }

    fn children(&self) -> Children<'a> {
        Children::new(self.element, &self.path)
    }
}

/// The element children of one element, consumed in declared order.
///
/// `take` only moves forward, so a child that appears after a later-declared
/// sibling is left unconsumed and reported by `finish` as out of place.
struct Children<'a> {
    parent: String,
    elements: Vec<(&'a XmlElement, String)>,
    cursor: usize,
}

impl<'a> Children<'a> {
    fn new(element: &'a XmlElement, path: &str) -> Self {
        let elements = element
            .children()
            .enumerate()
            .map(|(index, child)| {
                (
                    child,
                    format!("{path}/{}[{}]", child.local_name(), index + 1),
                )
            })
            .collect();
        Self {
            parent: path.to_owned(),
            elements,
            cursor: 0,
        }
    }

    /// Whether the child at the cursor is the unqualified element `name`.
    /// Every element a LOIN type declares locally is unqualified.
    fn peek_is(&self, name: &str) -> bool {
        self.elements.get(self.cursor).is_some_and(|(element, _)| {
            element.namespace_uri().is_none() && element.local_name() == name
        })
    }

    fn next_matching(&mut self, name: &str) -> Option<(&'a XmlElement, String)> {
        if self.peek_is(name) {
            let (element, path) = &self.elements[self.cursor];
            self.cursor += 1;
            Some((element, path.clone()))
        } else {
            None
        }
    }

    /// Every consecutive `name` child at the cursor.
    fn take_all(&mut self, name: &str) -> Vec<(&'a XmlElement, String)> {
        let mut found = Vec::new();
        while let Some(child) = self.next_matching(name) {
            found.push(child);
        }
        found
    }

    /// Consumes the leading run of ISO 23387 (`dt:`) children: the content a
    /// LOIN type inherits from `dt:ConceptType` before its own extension.
    fn take_dt_prefix(&mut self) -> Vec<&'a XmlElement> {
        let mut found = Vec::new();
        while let Some((element, _)) = self.elements.get(self.cursor) {
            if element.namespace_uri() != Some(dt::NAMESPACE) {
                break;
            }
            found.push(*element);
            self.cursor += 1;
        }
        found
    }

    fn optional(&mut self, name: &str) -> Option<(&'a XmlElement, String)> {
        self.next_matching(name)
    }

    fn required(&mut self, name: &str) -> Result<(&'a XmlElement, String)> {
        self.optional(name).ok_or_else(|| self.missing(name))
    }

    fn missing(&self, name: &str) -> ReadError {
        let detail = match self.elements.get(self.cursor) {
            Some((element, _)) => format!(
                "required {name} is missing (found {} instead)",
                element.qname()
            ),
            None => format!("required {name} is missing"),
        };
        ReadError::new(ReadErrorKind::MissingElement, &self.parent, detail)
    }

    /// Fails on the first element nothing consumed.
    fn finish(self) -> Result<()> {
        match self.elements.get(self.cursor) {
            None => Ok(()),
            Some((element, path)) => Err(ReadError::new(
                ReadErrorKind::UnexpectedElement,
                path,
                format!(
                    "{} ({}) is not allowed here, or is out of the declared order",
                    element.qname(),
                    element.namespace_uri().unwrap_or("no namespace")
                ),
            )),
        }
    }
}

fn node(child: (&XmlElement, String)) -> Result<Node<'_>> {
    Node::new(child.0, child.1)
}

fn simple(child: (&XmlElement, String)) -> Result<Node<'_>> {
    Node::simple(child.0, child.1)
}

// ---------------------------------------------------------------------------
// ISO 23387 bridge
// ---------------------------------------------------------------------------

/// Converts a LOIN tree element into an `openbim_dt` element, keeping names,
/// namespaces, attributes and text; comments and PIs are not content.
fn to_dt(element: &XmlElement) -> dt::Element {
    let mut converted = dt::Element::new(
        element.namespace_uri(),
        element.prefix(),
        element.local_name(),
    );
    for attribute in element.attributes() {
        converted = converted.with_attribute(dt::Attribute::new(
            attribute.prefix(),
            attribute.local_name(),
            attribute.namespace_uri(),
            attribute.value(),
        ));
    }
    for child in element.nodes() {
        converted = match child {
            XmlNode::Element(child) => converted.with_child(to_dt(child)),
            XmlNode::Text(text) | XmlNode::CData(text) => converted.with_text(text.clone()),
            _ => converted,
        };
    }
    converted
}

/// Decodes one ISO 23387 subtree with `openbim-dt`, re-rooting its error path
/// under the LOIN path of the element.
fn decode_dt<T>(
    child: (&XmlElement, String),
    decode: impl FnOnce(&dt::Element) -> core::result::Result<T, dt::CodecError>,
) -> Result<T> {
    let (element, path) = child;
    decode(&to_dt(element)).map_err(|error| dt_error(&path, &error))
}

/// Re-roots a `openbim-dt` codec error under the LOIN path of its element.
fn dt_error(path: &str, error: &dt::CodecError) -> ReadError {
    let inner = error.path();
    let location = if inner.is_empty() {
        path.to_owned()
    } else {
        format!("{path}/{inner}")
    };
    ReadError::new(
        ReadErrorKind::DataTemplate,
        &location,
        format!("{:?}: {}", error.kind(), error.detail()),
    )
}

/// A `dt:ReferenceType` element under a LOIN name.
fn reference(child: (&XmlElement, String)) -> Result<Reference> {
    let (element, path) = child;
    let node = Node::new(element, path)?;
    node.attributes(&[
        (Some(dt::NAMESPACE), "GUID"),
        (Some(dt::NAMESPACE), "referenceURI"),
    ])?;
    node.children().finish()?;
    let guid = node
        .optional_attribute(Some(dt::NAMESPACE), "GUID")
        .map(|value| {
            Guid::from_str(value).map_err(|error| {
                node.attribute_error("GUID", ReadErrorKind::InvalidValue, error.to_string())
            })
        })
        .transpose()?;
    let uri = node
        .optional_attribute(Some(dt::NAMESPACE), "referenceURI")
        .map(|value| {
            dt::AnyUri::from_str(value).map_err(|error| {
                node.attribute_error(
                    "referenceURI",
                    ReadErrorKind::InvalidValue,
                    error.to_string(),
                )
            })
        })
        .transpose()?;
    Ok(Reference::new(guid, uri))
}

/// A `dt:MultiLanguageTextType` element under a LOIN name.
fn multilingual(child: (&XmlElement, String)) -> Result<MultiLanguageText> {
    let node = simple(child)?;
    node.attributes(&[(None, "language")])?;
    let language = node.required_attribute(None, "language")?;
    MultiLanguageText::new(collapse_whitespace(language), node.text()).map_err(|error| {
        node.attribute_error("language", ReadErrorKind::InvalidValue, error.to_string())
    })
}

// ---------------------------------------------------------------------------
// LOIN types, in schema order
// ---------------------------------------------------------------------------

fn read_specification(child: &(&XmlElement, String)) -> Result<Specification> {
    let node = Node::new(child.0, child.1.clone())?;
    node.attributes(&[(None, "name"), (Some(dt::NAMESPACE), "GUID")])?;
    let guid = node.guid()?;
    let name = node.required_attribute(None, "name")?;
    let mut children = node.children();
    let prerequisites = read_prerequisites(children.required("Prerequisites")?)?;
    let mut specification = Specification::new(guid, name, prerequisites);
    for per_object in children.take_all("SpecificationPerObjectType") {
        specification.add_per_object(read_per_object(per_object)?);
    }
    if let Some(geo) = children.optional("GeoReferencing") {
        specification.set_geo_referencing(Some(read_geo_referencing(geo)?));
    }
    children.finish()?;
    Ok(specification)
}

fn read_prerequisites(child: (&XmlElement, String)) -> Result<Prerequisites> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID")])?;
    let guid = node.guid()?;
    let mut children = node.children();
    let purpose = read_purpose(children.required("Purpose")?)?;
    let milestone = read_milestone(children.required("InformationDeliveryMilestone")?)?;
    let providing = read_actor(children.required("ProvidingActor")?)?;
    let receiving = read_actor(children.required("ReceivingActor")?)?;
    children.finish()?;
    Ok(Prerequisites::new(
        guid, purpose, milestone, providing, receiving,
    ))
}

/// `PurposeType` is one repeating choice, so items are read in document order.
fn read_purpose(child: (&XmlElement, String)) -> Result<Purpose> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID")])?;
    let guid = node.guid()?;
    let mut items = Vec::new();
    for (index, element) in node.element.children().enumerate() {
        let path = format!("{}/{}[{}]", node.path, element.local_name(), index + 1);
        let entry = (element, path);
        if element.namespace_uri().is_some() {
            return Err(unexpected(&entry));
        }
        let item = match element.local_name() {
            "Name" => PurposeItem::Name(multilingual(entry)?),
            "Definition" => PurposeItem::Definition(multilingual(entry)?),
            "ReferenceDocument" => PurposeItem::ReferenceDocument(reference(entry)?),
            "Description" => PurposeItem::Description(multilingual(entry)?),
            "Language" => {
                let language = simple(entry)?;
                language.attributes(&[])?;
                PurposeItem::Language(language.parse(Language::from_str)?)
            }
            "Region" => {
                let region = simple(entry)?;
                region.attributes(&[])?;
                PurposeItem::Region(region.text())
            }
            "DictionaryRef" => PurposeItem::DictionaryRef(reference(entry)?),
            _ => return Err(unexpected(&entry)),
        };
        items.push(item);
    }
    let mut items = items.into_iter();
    let first = items.next().ok_or_else(|| {
        node.error(
            ReadErrorKind::MissingElement,
            "a Purpose requires at least one item",
        )
    })?;
    let mut purpose = Purpose::from_item(guid, first);
    for item in items {
        purpose.add_item(item);
    }
    Ok(purpose)
}

fn unexpected(entry: &(&XmlElement, String)) -> ReadError {
    ReadError::new(
        ReadErrorKind::UnexpectedElement,
        &entry.1,
        format!(
            "{} ({}) is not allowed here",
            entry.0.qname(),
            entry.0.namespace_uri().unwrap_or("no namespace")
        ),
    )
}

fn read_milestone(child: (&XmlElement, String)) -> Result<InformationDeliveryMilestone> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID"), (None, "Date")])?;
    let guid = node.guid()?;
    let mut children = node.children();
    let name = multilingual(children.required("Name")?)?;
    let mut milestone = InformationDeliveryMilestone::new(guid, name);
    for description in children.take_all("Description") {
        milestone.add_description(multilingual(description)?);
    }
    for document in children.take_all("ReferenceDocument") {
        milestone.add_reference_document(reference(document)?);
    }
    children.finish()?;
    if let Some(date) = node.optional_attribute(None, "Date") {
        let date = DateTime::from_str(&collapse_whitespace(date)).map_err(|error| {
            node.attribute_error("Date", ReadErrorKind::InvalidValue, error.to_string())
        })?;
        milestone.set_date(Some(date));
    }
    Ok(milestone)
}

fn read_actor(child: (&XmlElement, String)) -> Result<Actor> {
    let node = node(child)?;
    node.attributes(&[
        (Some(dt::NAMESPACE), "GUID"),
        (None, "firstName"),
        (None, "middleName"),
        (None, "lastName"),
        (None, "affiliation"),
    ])?;
    let guid = node.guid()?;
    let mut children = node.children();
    let role = multilingual(children.required("Role")?)?;
    let mut actor = Actor::new(guid, role);
    if let Some(description) = children.optional("Description") {
        actor.set_description(Some(multilingual(description)?));
    }
    if let Some(email) = children.optional("EMailAddress") {
        let email = simple(email)?;
        email.attributes(&[])?;
        let value = email.text();
        let address = EmailAddress::from_str(&value).map_err(|error| {
            email.error(ReadErrorKind::InvalidValue, format!("{value:?}: {error}"))
        })?;
        actor.set_email_address(Some(address));
    }
    children.finish()?;
    let text = |name: &str| node.optional_attribute(None, name).map(str::to_owned);
    actor.set_first_name(text("firstName"));
    actor.set_middle_name(text("middleName"));
    actor.set_last_name(text("lastName"));
    actor.set_affiliation(text("affiliation"));
    Ok(actor)
}

/// `SpecificationPerObjectTypeType` extends `dt:ConceptType`: the element's
/// own attributes and leading `dt:` children are the concept, decoded by
/// `openbim-dt`; the LOIN extension follows.
fn read_per_object(child: (&XmlElement, String)) -> Result<SpecificationPerObjectType> {
    let node = node(child)?;
    node.attributes(&[
        (Some(dt::NAMESPACE), "GUID"),
        (Some(dt::NAMESPACE), "about"),
        (None, "dateOfCreation"),
    ])?;
    let mut children = node.children();
    // The concept is this element's attributes plus its leading dt: children;
    // only those are converted, never the LOIN extension that follows.
    let mut concept_element = dt::Element::new(None, None, node.name());
    for attribute in node.element.attributes() {
        if attribute.namespace_uri() == Some(XMLNS_NAMESPACE) {
            continue;
        }
        concept_element = concept_element.with_attribute(dt::Attribute::new(
            attribute.prefix(),
            attribute.local_name(),
            attribute.namespace_uri(),
            attribute.value(),
        ));
    }
    for element in children.take_dt_prefix() {
        concept_element = concept_element.with_child(to_dt(element));
    }
    let concept =
        Concept::from_element(&concept_element).map_err(|error| dt_error(&node.path, &error))?;
    let object_type = decode_dt(
        children.required("ObjectType")?,
        dt::ObjectType::from_element,
    )?;
    let mut per_object = SpecificationPerObjectType::new(concept, object_type);
    if let Some(alpha) = children.optional("AlphanumericalInformation") {
        per_object.set_alphanumerical_information(Some(read_alphanumerical(alpha)?));
    }
    if let Some(documentation) = children.optional("Documentation") {
        per_object.set_documentation(Some(read_documentation(documentation)?));
    }
    if let Some(geometry) = children.optional("GeometricalInformation") {
        per_object.set_geometrical_information(Some(read_geometry(geometry)?));
    }
    children.finish()?;
    Ok(per_object)
}

fn read_alphanumerical(child: (&XmlElement, String)) -> Result<AlphanumericalInformation> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID")])?;
    let mut alpha = AlphanumericalInformation::new(node.guid()?);
    let mut children = node.children();
    for property in children.take_all("Property") {
        alpha.add_property(decode_dt(property, dt::Property::from_element)?);
    }
    for kind in children.take_all("QuantityKind") {
        alpha.add_quantity_kind(decode_dt(kind, dt::QuantityKind::from_element)?);
    }
    if let Some(groups) = children.optional("GroupsOfProperties") {
        alpha.set_groups_of_properties(Some(read_groups(groups)?));
    }
    for document in children.take_all("ReferenceDocument") {
        alpha.add_reference_document(decode_dt(document, dt::ReferenceDocument::from_element)?);
    }
    for dimension in children.take_all("Dimension") {
        alpha.add_dimension(decode_dt(dimension, dt::Dimension::from_element)?);
    }
    for unit in children.take_all("Unit") {
        alpha.add_unit(decode_dt(unit, dt::Unit::from_element)?);
    }
    children.finish()?;
    Ok(alpha)
}

fn read_groups(child: (&XmlElement, String)) -> Result<GroupsOfProperties> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut groups = GroupsOfProperties::new();
    let mut children = node.children();
    for group in children.take_all("GroupOfProperties") {
        groups.add_group(decode_dt(group, dt::GroupOfProperties::from_element)?);
    }
    for group_ref in children.take_all("GroupOfPropertiesRef") {
        groups.add_reference(reference(group_ref)?);
    }
    children.finish()?;
    Ok(groups)
}

fn read_documentation(child: (&XmlElement, String)) -> Result<Documentation> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID")])?;
    let mut documentation = Documentation::new(node.guid()?);
    let mut children = node.children();
    for document in children.take_all("Document") {
        documentation.add_document(read_document(document)?);
    }
    children.finish()?;
    Ok(documentation)
}

fn read_document(child: (&XmlElement, String)) -> Result<Document> {
    let node = node(child)?;
    node.attributes(&[
        (Some(dt::NAMESPACE), "GUID"),
        (None, "type"),
        (None, "form"),
        (None, "content"),
    ])?;
    let guid = node.guid()?;
    let mut children = node.children();
    let name = multilingual(children.required("Name")?)?;
    let references = children
        .take_all("ReferenceDocument")
        .into_iter()
        .map(reference)
        .collect::<Result<Vec<_>>>()?;
    let descriptions = children
        .take_all("Description")
        .into_iter()
        .map(multilingual)
        .collect::<Result<Vec<_>>>()?;
    let format = read_format(children.required("Format")?)?;
    children.finish()?;
    let mut document = Document::new(guid, name, format);
    for value in references {
        document.add_reference_document(value);
    }
    for value in descriptions {
        document.add_description(value);
    }
    let text = |name: &str| node.optional_attribute(None, name).map(str::to_owned);
    document.set_document_type(text("type"));
    document.set_form(text("form"));
    document.set_content(text("content"));
    Ok(document)
}

fn read_format(child: (&XmlElement, String)) -> Result<DocumentFormat> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let mut names = children
        .take_all("FormatName")
        .into_iter()
        .map(multilingual);
    let first_name = names
        .next()
        .ok_or_else(|| children.missing("FormatName"))??;
    let names = names.collect::<Result<Vec<_>>>()?;
    let mut versions = children
        .take_all("FormatVersion")
        .into_iter()
        .map(multilingual);
    let first_version = versions
        .next()
        .ok_or_else(|| children.missing("FormatVersion"))??;
    let versions = versions.collect::<Result<Vec<_>>>()?;
    let mut format = DocumentFormat::new(first_name, first_version);
    for name in names {
        format.add_name(name);
    }
    for version in versions {
        format.add_version(version);
    }
    for specification in children.take_all("FormatSpecification") {
        format.add_specification(reference(specification)?);
    }
    children.finish()?;
    Ok(format)
}

fn read_geometry(child: (&XmlElement, String)) -> Result<GeometricalInformation> {
    let node = node(child)?;
    node.attributes(&[(Some(dt::NAMESPACE), "GUID"), (None, "placeholder")])?;
    let mut geometry = GeometricalInformation::new(node.guid()?);
    if let Some(placeholder) = node.optional_attribute(None, "placeholder") {
        let value = parse_boolean(placeholder).ok_or_else(|| {
            node.attribute_error(
                "placeholder",
                ReadErrorKind::InvalidValue,
                format!("{placeholder:?} is not an xs:boolean"),
            )
        })?;
        geometry.set_placeholder(Some(value));
    }
    let mut children = node.children();
    if let Some(detail) = children.optional("Detail") {
        geometry.set_detail(Some(read_detail(detail)?));
    }
    geometry.set_dimensionality(enumeration::<Dimensionality>(
        &mut children,
        "Dimensionality",
    )?);
    geometry.set_appearance(enumeration::<Appearance>(&mut children, "Appearance")?);
    geometry.set_parametric_behaviour(enumeration::<ParametricBehaviour>(
        &mut children,
        "ParametricBehaviour",
    )?);
    if let Some(location) = children.optional("Location") {
        geometry.set_location(Some(read_location(location)?));
    }
    children.finish()?;
    Ok(geometry)
}

/// An optional enumeration child, parsed by the model enum's `FromStr`.
fn enumeration<T>(children: &mut Children<'_>, name: &str) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    children
        .optional(name)
        .map(|child| {
            let value = simple(child)?;
            value.attributes(&[])?;
            value.parse(T::from_str)
        })
        .transpose()
}

fn read_detail(child: (&XmlElement, String)) -> Result<Detail> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut detail = Detail::new();
    let mut children = node.children();
    if let Some(dictionary) = children.optional("Dictionary") {
        detail.set_dictionary(Some(reference(dictionary)?));
    }
    detail.set_shape_assembly(enumeration::<ShapeAssembly>(
        &mut children,
        "ShapeAssembly",
    )?);
    detail.set_shape_representation(enumeration::<ShapeRepresentation>(
        &mut children,
        "ShapeRepresentation",
    )?);
    if let Some(influence) = children.optional("ShapeInfluence") {
        detail.set_shape_influence(Some(read_shape_influence(influence)?));
    }
    children.finish()?;
    Ok(detail)
}

fn read_shape_influence(child: (&XmlElement, String)) -> Result<ShapeInfluence> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut influence = ShapeInfluence::new();
    let mut children = node.children();
    influence.inside_geometry = enumeration::<InsideGeometry>(&mut children, "InsideGeometry")?;
    influence.connections = enumeration::<Connections>(&mut children, "Connections")?;
    influence.openings = enumeration::<Openings>(&mut children, "Openings")?;
    influence.operating_and_clearance_zones =
        enumeration::<OperatingAndClearanceZones>(&mut children, "OperatingAndClearanceZones")?;
    influence.features = enumeration::<Features>(&mut children, "Features")?;
    if let Some(threshold) = children.optional("ThresholdDimension") {
        influence.threshold_dimension = Some(read_threshold(threshold)?);
    }
    children.finish()?;
    Ok(influence)
}

fn read_threshold(child: (&XmlElement, String)) -> Result<ThresholdDimension> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let threshold = simple(children.required("Threshold")?)?;
    threshold.attributes(&[])?;
    let value = threshold.parse(parse_double)?;
    let unit = decode_dt(children.required("Unit")?, dt::Unit::from_element)?;
    let definition = multilingual(children.required("Definition")?)?;
    children.finish()?;
    Ok(ThresholdDimension::new(value, unit, definition))
}

fn read_location(child: (&XmlElement, String)) -> Result<Location> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let kind = enumeration::<RelativeOrAbsolute>(&mut children, "RelativeOrAbsolute")?
        .ok_or_else(|| children.missing("RelativeOrAbsolute"))?;
    let reference_object = children
        .optional("ReferenceObject")
        .map(|child| {
            let value = simple(child)?;
            value.attributes(&[])?;
            Ok(value.text())
        })
        .transpose()?;
    children.finish()?;
    Ok(Location::new(kind, reference_object))
}

fn read_geo_referencing(child: (&XmlElement, String)) -> Result<GeoReferencing> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut geo = GeoReferencing::new();
    let mut children = node.children();
    if let Some(crs) = children.optional("CoordinateReferenceSystem") {
        geo.set_coordinate_reference_system(Some(read_crs(crs)?));
    }
    for system in children.take_all("ModelCoordinateSystem") {
        geo.add_model_coordinate_system(read_model_coordinate_system(system)?);
    }
    children.finish()?;
    Ok(geo)
}

fn read_crs(child: (&XmlElement, String)) -> Result<CoordinateReferenceSystem> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let kind = enumeration::<CoordinateReferenceSystemKind>(&mut children, "Type")?
        .ok_or_else(|| children.missing("Type"))?;
    let datum = read_datum(children.required("Datum")?)?;
    let mut crs = CoordinateReferenceSystem::new(kind, datum);
    if let Some(vertical) = children.optional("VerticalDatum") {
        crs.set_vertical_datum(Some(read_datum(vertical)?));
    }
    children.finish()?;
    Ok(crs)
}

fn read_datum(child: (&XmlElement, String)) -> Result<Datum> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let name = simple(children.required("Name")?)?;
    name.attributes(&[])?;
    let name = name.text();
    let registry = read_datum_registry(children.required("Type")?)?;
    children.finish()?;
    Ok(Datum::new(name, registry))
}

fn read_datum_registry(child: (&XmlElement, String)) -> Result<DatumRegistryReference> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let mut registry = DatumRegistryReference::new(multilingual(children.required("Name")?)?);
    for description in children.take_all("Description") {
        registry.add_description(multilingual(description)?);
    }
    if let Some(reference_element) = children.optional("RegistryReference") {
        registry.set_registry_reference(Some(reference(reference_element)?));
    }
    children.finish()?;
    Ok(registry)
}

fn read_model_coordinate_system(child: (&XmlElement, String)) -> Result<ModelCoordinateSystem> {
    let node = node(child)?;
    node.attributes(&[])?;
    let mut children = node.children();
    let is_projected = {
        let value = simple(children.required("IsProjected")?)?;
        value.attributes(&[])?;
        let token = value.token();
        parse_boolean(&token).ok_or_else(|| {
            value.error(
                ReadErrorKind::InvalidValue,
                format!("{token:?} is not an xs:boolean"),
            )
        })?
    };
    let first = decimal(children.required("FirstCoordinate")?)?;
    let second = decimal(children.required("SecondCoordinate")?)?;
    let height = decimal(children.required("Height")?)?;
    let mut system = ModelCoordinateSystem::new(is_projected, first, second, height);
    system.x_axis_abscissa = children
        .optional("XAxisAbscissa")
        .map(decimal)
        .transpose()?;
    system.x_axis_ordinate = children
        .optional("XAxisOrdinate")
        .map(decimal)
        .transpose()?;
    system.unit_scale = children.optional("UnitScale").map(decimal).transpose()?;
    system.horizontal_scale = children
        .optional("HorizontalScale")
        .map(decimal)
        .transpose()?;
    children.finish()?;
    Ok(system)
}

fn decimal(child: (&XmlElement, String)) -> Result<Decimal> {
    let value = simple(child)?;
    value.attributes(&[])?;
    value.parse(Decimal::from_str)
}

// ---------------------------------------------------------------------------
// Lexical helpers (XML Schema 1.0 datatypes)
// ---------------------------------------------------------------------------

fn is_xsd_whitespace(character: char) -> bool {
    matches!(character, '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}')
}

fn collapse_whitespace(value: &str) -> String {
    value
        .split(is_xsd_whitespace)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_boolean(value: &str) -> Option<bool> {
    match collapse_whitespace(value).as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// `xs:double`: Rust's float grammar is wider (`inf`, `infinity`, `nan`, a
/// leading `+` on specials), so the XML Schema lexical space is checked first.
fn parse_double(value: &str) -> core::result::Result<f64, &'static str> {
    match value {
        "INF" => return Ok(f64::INFINITY),
        "-INF" => return Ok(f64::NEG_INFINITY),
        "NaN" => return Ok(f64::NAN),
        _ => {}
    }
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (mantissa, exponent) = match digits.find(['e', 'E']) {
        Some(index) => (&digits[..index], Some(&digits[index + 1..])),
        None => (digits, None),
    };
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    let mantissa_ok =
        all_digits(integer) && all_digits(fraction) && !(integer.is_empty() && fraction.is_empty());
    let exponent_ok = exponent.is_none_or(|exponent| {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        !exponent.is_empty() && all_digits(exponent)
    });
    if !(mantissa_ok && exponent_ok) {
        return Err("not an xs:double lexical value");
    }
    value
        .parse::<f64>()
        .map_err(|_| "not an xs:double lexical value")
}
