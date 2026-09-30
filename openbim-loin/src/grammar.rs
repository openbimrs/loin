//! Read-only, machine-readable view of the LOIN grammar this crate enforces.
//!
//! Third-party tools (editors, generators, other-language validators) can
//! target the format from here instead of reading validator source. The same
//! data is published as JSON by [`to_json`], committed as
//! `openbim-loin/loin-grammar.json`, and exposed to JavaScript by the
//! `@openbim/loin` package.
//!
//! # Single source of truth
//!
//! The element set, child lists, cardinalities, ordering and imported ISO
//! 23387 elements are **derived** from the validator's own tables by walking
//! them from the root, so they cannot drift from [`LoinDocument::validate`].
//! Enumeration value sets come from the model enums the validator also uses.
//! Attribute declarations and scalar value types are declared here and
//! **checked** against the validator by exhaustive unit tests: every declared
//! attribute is accepted, every undeclared one is rejected, every required one
//! is enforced, and every typed value rejects an invalid lexeme with the
//! matching [`DiagnosticCode`](crate::DiagnosticCode).
//!
//! # Scope
//!
//! This describes what the validator checks, which is XSD-derived but not a
//! complete XML Schema (see the crate-level status). The grammar is
//! context-sensitive by one level: an element is identified by its local name
//! *and* its parent's local name, exactly as the validator keys its rules
//! (`Name` under `Purpose` is multilingual text; `Name` under `Datum` is not).
//! Content of imported ISO 23387 types ([`Content::Imported`]) is owned by
//! `openbim-dt` and is not described here.
//!
//! Every element below the root is unqualified (`elementFormDefault` is
//! `unqualified`); only the root is in a LOIN namespace. Attributes are
//! unqualified unless [`Attribute::namespace`] says otherwise.
//!
//! ```
//! use openbim_loin::grammar::{self, Content};
//!
//! let purpose = grammar::element(Some("Prerequisites"), "Purpose").unwrap();
//! assert!(matches!(purpose.content(), Content::Choice(_)));
//! assert!(purpose.attributes().iter().any(|a| a.name() == "GUID" && a.is_required()));
//!
//! let json = grammar::to_json();
//! assert!(json.contains("\"formatVersion\": 1"));
//! ```
//!
//! [`LoinDocument::validate`]: crate::LoinDocument::validate

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::validation;
use crate::{dt, KNOWN_NAMESPACES};

pub use crate::validation::ChildRule;

/// Version of the grammar description format (the Rust shape and the JSON
/// document). Incremented only for incompatible changes to the description
/// itself, not when the described grammar gains elements.
pub const FORMAT_VERSION: u32 = 1;

/// XML Schema instance namespace, used by `xsi:nil` and schema-location hints.
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Local name of the document root.
pub const ROOT: &str = "LevelOfInformationNeed";

/// Namespace of an attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttributeNamespace {
    /// No namespace: written without a prefix (`dateOfCreation`, `language`).
    Unqualified,
    /// The ISO 23387 namespace [`dt::NAMESPACE`] (`dt:GUID`, `dt:about`).
    Dt,
    /// The XML Schema instance namespace [`XSI_NAMESPACE`] (`xsi:nil`).
    Xsi,
}

impl AttributeNamespace {
    /// Namespace URI, or `None` for an unqualified attribute.
    #[must_use]
    pub const fn uri(self) -> Option<&'static str> {
        match self {
            Self::Unqualified => None,
            Self::Dt => Some(dt::NAMESPACE),
            Self::Xsi => Some(XSI_NAMESPACE),
        }
    }

    const fn json_name(self) -> Option<&'static str> {
        match self {
            Self::Unqualified => None,
            Self::Dt => Some("dt"),
            Self::Xsi => Some("xsi"),
        }
    }
}

/// Lexical type of an attribute value or of simple element content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueType {
    /// Any string; the validator applies no lexical check.
    String,
    /// `xs:language`.
    Language,
    /// `xs:boolean` (`true`, `false`, `1`, `0`).
    Boolean,
    /// `xs:decimal`.
    Decimal,
    /// `xs:double`.
    Double,
    /// `xs:dateTime`.
    DateTime,
    /// ISO 23387 GUID.
    Guid,
    /// `xs:anyURI`.
    AnyUri,
    /// The ISO 7817-3 e-mail restriction `[^@]+@[^\.]+\..+` (see ADR 0003).
    EmailAddress,
    /// One of a closed set of values, compared exactly (no whitespace collapse).
    Enumeration(&'static [&'static str]),
}

impl ValueType {
    /// Stable lower-camel-case name used in the JSON artifact.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Language => "language",
            Self::Boolean => "boolean",
            Self::Decimal => "decimal",
            Self::Double => "double",
            Self::DateTime => "dateTime",
            Self::Guid => "guid",
            Self::AnyUri => "anyUri",
            Self::EmailAddress => "emailAddress",
            Self::Enumeration(_) => "enumeration",
        }
    }
}

/// One declared attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute {
    namespace: AttributeNamespace,
    name: &'static str,
    required: bool,
    value_type: ValueType,
}

impl Attribute {
    const fn local(name: &'static str, required: bool, value_type: ValueType) -> Self {
        Self {
            namespace: AttributeNamespace::Unqualified,
            name,
            required,
            value_type,
        }
    }

    const fn dt(name: &'static str, required: bool, value_type: ValueType) -> Self {
        Self {
            namespace: AttributeNamespace::Dt,
            name,
            required,
            value_type,
        }
    }

    const fn xsi(name: &'static str, value_type: ValueType) -> Self {
        Self {
            namespace: AttributeNamespace::Xsi,
            name,
            required: false,
            value_type,
        }
    }

    /// Namespace of the attribute. Note the asymmetry the schema declares:
    /// `dt:GUID` is qualified while `dateOfCreation` on the same element is not.
    #[must_use]
    pub const fn namespace(&self) -> AttributeNamespace {
        self.namespace
    }

    /// Local name, with the exact case the schema declares (`Date`, not `date`).
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Whether the attribute is `use="required"`.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    /// Lexical type of the value.
    #[must_use]
    pub const fn value_type(&self) -> ValueType {
        self.value_type
    }
}

/// Content model of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Content {
    /// Child elements in the declared order, each within its cardinality.
    Sequence(&'static [ChildRule]),
    /// A repeating choice: branches may interleave in any order, each within
    /// its cardinality, and at least one item must be present overall.
    Choice(&'static [ChildRule]),
    /// Character data only, of the given type.
    Simple(ValueType),
    /// Content of an imported ISO 23387 complex type. The element itself is
    /// unqualified; its content is owned by `openbim-dt`.
    Imported,
}

/// One element declaration, identified by its parent's local name and its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    parent: Option<&'static str>,
    name: &'static str,
    content: Content,
    attributes: &'static [Attribute],
    inherited_dt_children: &'static [&'static str],
    nillable: bool,
}

impl Element {
    /// Local name of the parent element; `None` for the root.
    #[must_use]
    pub const fn parent(&self) -> Option<&'static str> {
        self.parent
    }

    /// Local name of the element.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Content model.
    #[must_use]
    pub const fn content(&self) -> Content {
        self.content
    }

    /// Declared child elements, in declared order; empty for simple and
    /// imported content.
    #[must_use]
    pub const fn children(&self) -> &'static [ChildRule] {
        match self.content {
            Content::Sequence(children) | Content::Choice(children) => children,
            Content::Simple(_) | Content::Imported => &[],
        }
    }

    /// Declared attributes. Order carries no meaning. Namespace declarations
    /// and [`global_attributes`] are allowed in addition. Empty for
    /// [`Content::Imported`], whose attributes belong to ISO 23387.
    #[must_use]
    pub const fn attributes(&self) -> &'static [Attribute] {
        self.attributes
    }

    /// ISO 23387 `ConceptType` children inherited through the element's base
    /// type. They are `dt`-qualified, precede every declared child, may appear
    /// in any order among themselves, and at least one is required. Only
    /// `SpecificationPerObjectType` has any.
    #[must_use]
    pub const fn inherited_dt_children(&self) -> &'static [&'static str] {
        self.inherited_dt_children
    }

    /// Whether `xsi:nil="true"` may replace the element's content.
    #[must_use]
    pub const fn is_nillable(&self) -> bool {
        self.nillable
    }
}

/// Attributes allowed on every LOIN element besides namespace declarations.
#[must_use]
pub const fn global_attributes() -> &'static [Attribute] {
    GLOBAL_ATTRIBUTES
}

const GLOBAL_ATTRIBUTES: &[Attribute] = &[
    Attribute::xsi("schemaLocation", ValueType::String),
    Attribute::xsi("noNamespaceSchemaLocation", ValueType::String),
];

/// Every element declaration reachable from the root, in document pre-order.
///
/// Derived once from the validator's tables. Each `(parent, name)` pair
/// appears once.
#[must_use]
pub fn elements() -> &'static [Element] {
    static ELEMENTS: OnceLock<Vec<Element>> = OnceLock::new();
    ELEMENTS.get_or_init(derive_elements)
}

/// The declaration of `name` below a parent with local name `parent` (`None`
/// for the root).
#[must_use]
pub fn element(parent: Option<&str>, name: &str) -> Option<&'static Element> {
    elements()
        .iter()
        .find(|element| element.parent == parent && element.name == name)
}

/// Walks the validator's tables from the root, depth first, in declared child
/// order. An explicit stack keeps the walk independent of recursion depth.
fn derive_elements() -> Vec<Element> {
    let mut out: Vec<Element> = Vec::new();
    let mut pending: Vec<(Option<&'static str>, &'static str)> = vec![(None, ROOT)];
    while let Some((parent, name)) = pending.pop() {
        if out
            .iter()
            .any(|element| element.parent == parent && element.name == name)
        {
            continue;
        }
        let element = describe(parent, name);
        // Reverse so the first declared child is visited first.
        for child in element.children().iter().rev() {
            pending.push((Some(element.name), child.name()));
        }
        out.push(element);
    }
    out
}

fn describe(parent: Option<&'static str>, name: &'static str) -> Element {
    let content = if validation::is_imported_dt_complex(name, parent) {
        Content::Imported
    } else {
        let rule = validation::content_rule(name, parent).unwrap_or_else(|| {
            panic!("validator declares child {name} below {parent:?} without a rule")
        });
        match (rule.children.is_empty(), rule.ordered) {
            (true, _) => Content::Simple(simple_value_type(parent, name)),
            (false, true) => Content::Sequence(rule.children),
            (false, false) => Content::Choice(rule.children),
        }
    };
    let imported = content == Content::Imported;
    let is_per_object = name == "SpecificationPerObjectType";
    Element {
        parent,
        name,
        content,
        attributes: if imported {
            &[]
        } else {
            declared_attributes(parent, name)
        },
        inherited_dt_children: if is_per_object {
            validation::INHERITED_DT_CONCEPT_CHILDREN
        } else {
            &[]
        },
        nillable: is_per_object,
    }
}

/// Scalar type of simple content. Checked against `validate_lexical_content`
/// by the unit tests below; enumerations are read from the validator.
fn simple_value_type(parent: Option<&str>, name: &str) -> ValueType {
    if let Some(values) = validation::enumeration_values(parent, name) {
        return ValueType::Enumeration(values);
    }
    match (parent, name) {
        (Some("Purpose"), "Language") => ValueType::Language,
        (Some("ThresholdDimension"), "Threshold") => ValueType::Double,
        (Some("ProvidingActor" | "ReceivingActor"), "EMailAddress") => ValueType::EmailAddress,
        (Some("ModelCoordinateSystem"), "IsProjected") => ValueType::Boolean,
        (
            Some("ModelCoordinateSystem"),
            "FirstCoordinate" | "SecondCoordinate" | "Height" | "XAxisAbscissa" | "XAxisOrdinate"
            | "UnitScale" | "HorizontalScale",
        ) => ValueType::Decimal,
        _ => ValueType::String,
    }
}

const GUID: Attribute = Attribute::dt("GUID", true, ValueType::Guid);
const LANGUAGE: Attribute = Attribute::local("language", true, ValueType::Language);
const IDENTIFIED: &[Attribute] = &[GUID];
const MULTILINGUAL: &[Attribute] = &[LANGUAGE];
const SPECIFICATION: &[Attribute] = &[Attribute::local("name", true, ValueType::String), GUID];
const PER_OBJECT: &[Attribute] = &[
    GUID,
    Attribute::dt("about", false, ValueType::AnyUri),
    Attribute::local("dateOfCreation", true, ValueType::DateTime),
    Attribute::xsi("nil", ValueType::Boolean),
];
const MILESTONE: &[Attribute] = &[GUID, Attribute::local("Date", false, ValueType::DateTime)];
const ACTOR: &[Attribute] = &[
    GUID,
    Attribute::local("firstName", false, ValueType::String),
    Attribute::local("middleName", false, ValueType::String),
    Attribute::local("lastName", false, ValueType::String),
    Attribute::local("affiliation", false, ValueType::String),
];
const DOCUMENT: &[Attribute] = &[
    GUID,
    Attribute::local("type", false, ValueType::String),
    Attribute::local("form", false, ValueType::String),
    Attribute::local("content", false, ValueType::String),
];
const GEOMETRICAL: &[Attribute] = &[
    GUID,
    Attribute::local("placeholder", false, ValueType::Boolean),
];

/// Attribute declarations. Checked against `validate_attributes` by the unit
/// tests below.
fn declared_attributes(parent: Option<&str>, name: &str) -> &'static [Attribute] {
    match (parent, name) {
        (Some("LevelOfInformationNeed"), "Specification") => SPECIFICATION,
        (Some("Specification"), "SpecificationPerObjectType") => PER_OBJECT,
        (Some("Prerequisites"), "InformationDeliveryMilestone") => MILESTONE,
        (Some("Prerequisites"), "ProvidingActor" | "ReceivingActor") => ACTOR,
        (Some("Documentation"), "Document") => DOCUMENT,
        (Some("SpecificationPerObjectType"), "GeometricalInformation") => GEOMETRICAL,
        (Some("Specification"), "Prerequisites")
        | (Some("Prerequisites"), "Purpose")
        | (Some("SpecificationPerObjectType"), "AlphanumericalInformation" | "Documentation") => {
            IDENTIFIED
        }
        (Some("Purpose"), "Name" | "Definition" | "Description")
        | (Some("InformationDeliveryMilestone"), "Name" | "Description")
        | (Some("ProvidingActor" | "ReceivingActor"), "Role" | "Description")
        | (Some("Document"), "Name" | "Description")
        | (Some("Format"), "FormatName" | "FormatVersion")
        | (Some("ThresholdDimension"), "Definition")
        | (Some("Type"), "Name" | "Description") => MULTILINGUAL,
        _ => &[],
    }
}

/// Renders the grammar as the published JSON document.
///
/// The output is deterministic and is committed as
/// `openbim-loin/loin-grammar.json`; a test fails when the two differ.
#[must_use]
pub fn to_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"openbim-loin-grammar\",\n");
    let _ = writeln!(out, "  \"formatVersion\": {FORMAT_VERSION},");
    out.push_str("  \"description\": ");
    push_json_string(
        &mut out,
        "ISO 7817-3 LOIN grammar enforced by openbim-loin validate(). \
         Elements are keyed by (parent, name); every element below the root is unqualified. \
         Content of 'imported' elements is ISO 23387 and is not described here.",
    );
    out.push_str(",\n  \"namespaces\": {\n    \"loin\": [");
    for (index, namespace) in KNOWN_NAMESPACES.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(&mut out, namespace);
    }
    out.push_str("],\n    \"dt\": ");
    push_json_string(&mut out, dt::NAMESPACE);
    out.push_str(",\n    \"xsi\": ");
    push_json_string(&mut out, XSI_NAMESPACE);
    out.push_str("\n  },\n  \"root\": ");
    push_json_string(&mut out, ROOT);
    out.push_str(",\n  \"globalAttributes\": ");
    push_attributes(&mut out, GLOBAL_ATTRIBUTES, "  ");
    out.push_str(",\n  \"elements\": [");
    for (index, element) in elements().iter().enumerate() {
        out.push_str(if index == 0 { "\n" } else { ",\n" });
        push_element_json(&mut out, element);
    }
    out.push_str("\n  ]\n}\n");
    out
}

fn push_element_json(out: &mut String, element: &Element) {
    out.push_str("    {\n      \"parent\": ");
    match element.parent {
        Some(parent) => push_json_string(out, parent),
        None => out.push_str("null"),
    }
    out.push_str(",\n      \"name\": ");
    push_json_string(out, element.name);
    out.push_str(",\n      \"content\": ");
    let kind = match element.content {
        Content::Sequence(_) => "sequence",
        Content::Choice(_) => "choice",
        Content::Simple(_) => "simple",
        Content::Imported => "imported",
    };
    push_json_string(out, kind);
    if let Content::Simple(value_type) = element.content {
        push_value_type(out, value_type, "      ");
    }
    if !element.children().is_empty() {
        out.push_str(",\n      \"children\": [");
        for (index, child) in element.children().iter().enumerate() {
            out.push_str(if index == 0 { "\n" } else { ",\n" });
            out.push_str("        { \"name\": ");
            push_json_string(out, child.name());
            let _ = write!(out, ", \"min\": {}, \"max\": ", child.min());
            match child.max() {
                Some(max) => {
                    let _ = write!(out, "{max}");
                }
                None => out.push_str("null"),
            }
            out.push_str(" }");
        }
        out.push_str("\n      ]");
    }
    if !element.inherited_dt_children.is_empty() {
        out.push_str(",\n      \"inheritedDtChildren\": [");
        for (index, name) in element.inherited_dt_children.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            push_json_string(out, name);
        }
        out.push(']');
    }
    if element.nillable {
        out.push_str(",\n      \"nillable\": true");
    }
    if !element.attributes.is_empty() {
        out.push_str(",\n      \"attributes\": ");
        push_attributes(out, element.attributes, "      ");
    }
    out.push_str("\n    }");
}

fn push_attributes(out: &mut String, attributes: &[Attribute], indent: &str) {
    out.push('[');
    for (index, attribute) in attributes.iter().enumerate() {
        out.push_str(if index == 0 { "\n" } else { ",\n" });
        let _ = write!(out, "{indent}  {{ \"namespace\": ");
        match attribute.namespace.json_name() {
            Some(prefix) => push_json_string(out, prefix),
            None => out.push_str("null"),
        }
        out.push_str(", \"name\": ");
        push_json_string(out, attribute.name);
        let _ = write!(out, ", \"required\": {}", attribute.required);
        out.push_str(", \"valueType\": ");
        push_json_string(out, attribute.value_type.as_str());
        out.push_str(" }");
    }
    let _ = write!(out, "\n{indent}]");
}

fn push_value_type(out: &mut String, value_type: ValueType, indent: &str) {
    let _ = write!(out, ",\n{indent}\"valueType\": ");
    push_json_string(out, value_type.as_str());
    if let ValueType::Enumeration(values) = value_type {
        let _ = write!(out, ",\n{indent}\"values\": [");
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            push_json_string(out, value);
        }
        out.push(']');
    }
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if u32::from(control) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::validation::{validate_attributes, validate_lexical_content};
    use crate::{Diagnostic, DiagnosticCode, XmlAttribute, XmlElement};

    fn valid_lexeme(value_type: ValueType) -> &'static str {
        match value_type {
            ValueType::String => "any text",
            ValueType::Language => "en",
            ValueType::Boolean => "true",
            ValueType::Decimal => "1.5",
            ValueType::Double => "1.5E3",
            ValueType::DateTime => "2026-01-01T00:00:00Z",
            ValueType::Guid => "10000000-0000-4000-8000-000000000001",
            ValueType::AnyUri => "urn:example:thing",
            ValueType::EmailAddress => "someone@example.org",
            ValueType::Enumeration(values) => values[0],
        }
    }

    /// An invalid lexeme and the code the validator must report for it.
    fn invalid_lexeme(value_type: ValueType) -> Option<(&'static str, DiagnosticCode)> {
        Some(match value_type {
            ValueType::String => return None,
            ValueType::Language => ("not a language", DiagnosticCode::InvalidLanguage),
            ValueType::Boolean => ("yes", DiagnosticCode::InvalidBoolean),
            ValueType::Decimal => ("1,5", DiagnosticCode::InvalidDecimal),
            ValueType::Double => ("1,5", DiagnosticCode::InvalidDouble),
            ValueType::DateTime => ("yesterday", DiagnosticCode::InvalidDateTime),
            ValueType::Guid => ("not-a-guid", DiagnosticCode::InvalidGuid),
            ValueType::AnyUri => ("\u{1}", DiagnosticCode::InvalidAnyUri),
            ValueType::EmailAddress => ("nobody", DiagnosticCode::InvalidEnumeration),
            ValueType::Enumeration(_) => ("NotADeclaredValue", DiagnosticCode::InvalidEnumeration),
        })
    }

    fn build_attribute(attribute: &Attribute, value: &str) -> XmlAttribute {
        match attribute.namespace {
            AttributeNamespace::Unqualified => XmlAttribute::new(attribute.name, value),
            AttributeNamespace::Dt => XmlAttribute::new_dt(attribute.name, value),
            AttributeNamespace::Xsi => XmlAttribute::parsed(
                format!("xsi:{}", attribute.name),
                Some("xsi".to_owned()),
                attribute.name.to_owned(),
                Some(Arc::from(XSI_NAMESPACE)),
                value.to_owned(),
            ),
        }
    }

    fn attribute_diagnostics(element: &Element, attributes: &[XmlAttribute]) -> Vec<Diagnostic> {
        let mut xml = XmlElement::new(element.name);
        for attribute in attributes {
            xml = xml.with_attribute(attribute.clone());
        }
        let mut diagnostics = Vec::new();
        validate_attributes(&xml, element.parent, "/probe", &mut diagnostics);
        diagnostics
    }

    fn required_attributes(element: &Element) -> Vec<XmlAttribute> {
        element
            .attributes
            .iter()
            .filter(|attribute| attribute.required)
            .map(|attribute| build_attribute(attribute, valid_lexeme(attribute.value_type)))
            .collect()
    }

    fn declared() -> impl Iterator<Item = &'static Element> {
        elements()
            .iter()
            .filter(|element| element.content != Content::Imported)
    }

    #[test]
    fn walk_covers_the_whole_validator_grammar() {
        let all = elements();
        assert_eq!(all[0].name, ROOT);
        assert_eq!(all[0].parent, None);
        // A directory-style filter could match nothing; pin real counts.
        assert!(all.len() >= 80, "only {} elements derived", all.len());
        let imported = all
            .iter()
            .filter(|element| element.content == Content::Imported)
            .count();
        assert!(imported >= 14, "only {imported} imported elements");
        for element in all {
            for child in element.children() {
                assert!(
                    super::element(Some(element.name), child.name()).is_some(),
                    "{} below {} is not described",
                    child.name(),
                    element.name
                );
            }
        }
    }

    #[test]
    fn only_purpose_is_an_unordered_choice() {
        let choices: Vec<_> = elements()
            .iter()
            .filter(|element| matches!(element.content, Content::Choice(_)))
            .map(|element| element.name)
            .collect();
        assert_eq!(choices, ["Purpose"]);
    }

    #[test]
    fn declared_attributes_match_the_validator_exactly() {
        let universe: Vec<Attribute> = {
            let mut all: Vec<Attribute> = elements()
                .iter()
                .flat_map(|element| element.attributes.iter().copied())
                .collect();
            all.push(Attribute::local("undeclared", false, ValueType::String));
            all.push(Attribute::dt("undeclared", false, ValueType::String));
            all
        };
        let mut checked = 0;
        for element in declared() {
            let base = required_attributes(element);
            let clean = attribute_diagnostics(element, &base);
            assert!(clean.is_empty(), "{}: {clean:#?}", element.name);

            for candidate in &universe {
                let is_declared = element.attributes.iter().any(|declared| {
                    declared.namespace == candidate.namespace && declared.name == candidate.name
                });
                if is_declared && candidate.required {
                    continue;
                }
                let value = if is_declared {
                    let declared = element
                        .attributes
                        .iter()
                        .find(|d| d.namespace == candidate.namespace && d.name == candidate.name)
                        .unwrap();
                    valid_lexeme(declared.value_type)
                } else {
                    valid_lexeme(candidate.value_type)
                };
                let mut attributes = base.clone();
                attributes.push(build_attribute(candidate, value));
                let diagnostics = attribute_diagnostics(element, &attributes);
                let rejected = diagnostics
                    .iter()
                    .any(|d| d.code() == DiagnosticCode::UnexpectedAttribute);
                assert_eq!(
                    rejected,
                    !is_declared,
                    "{}/{} attribute {:?}:{}: {diagnostics:#?}",
                    element.parent.unwrap_or("-"),
                    element.name,
                    candidate.namespace,
                    candidate.name
                );
                checked += 1;
            }

            for attribute in element.attributes {
                if attribute.required {
                    let without: Vec<_> = base
                        .iter()
                        .filter(|a| {
                            !(a.local_name() == attribute.name
                                && a.namespace_uri() == attribute.namespace.uri())
                        })
                        .cloned()
                        .collect();
                    let diagnostics = attribute_diagnostics(element, &without);
                    assert!(
                        diagnostics.iter().any(|d| matches!(
                            d.code(),
                            DiagnosticCode::MissingRequiredAttribute
                                | DiagnosticCode::MissingLanguage
                        )),
                        "{}: missing {} not reported",
                        element.name,
                        attribute.name
                    );
                }
                if let Some((lexeme, code)) = invalid_lexeme(attribute.value_type) {
                    let mut attributes: Vec<_> = base
                        .iter()
                        .filter(|a| {
                            !(a.local_name() == attribute.name
                                && a.namespace_uri() == attribute.namespace.uri())
                        })
                        .cloned()
                        .collect();
                    attributes.push(build_attribute(attribute, lexeme));
                    let diagnostics = attribute_diagnostics(element, &attributes);
                    assert!(
                        diagnostics.iter().any(|d| d.code() == code),
                        "{}@{}: {lexeme:?} not reported as {code:?}: {diagnostics:#?}",
                        element.name,
                        attribute.name
                    );
                }
            }
        }
        assert!(checked >= 500, "only {checked} attribute probes ran");
    }

    #[test]
    fn global_attributes_are_accepted_everywhere() {
        for element in declared() {
            let mut attributes = required_attributes(element);
            for global in GLOBAL_ATTRIBUTES {
                attributes.push(build_attribute(global, "urn:example:schema"));
            }
            let diagnostics = attribute_diagnostics(element, &attributes);
            assert!(diagnostics.is_empty(), "{}: {diagnostics:#?}", element.name);
        }
    }

    #[test]
    fn simple_value_types_match_the_validator_exactly() {
        let mut typed = 0;
        for element in elements() {
            let Content::Simple(value_type) = element.content else {
                continue;
            };
            let run = |text: &str| {
                let xml = XmlElement::new(element.name).with_text(text);
                let mut diagnostics = Vec::new();
                validate_lexical_content(&xml, element.parent, "/probe", &mut diagnostics);
                diagnostics
            };
            let valid = run(valid_lexeme(value_type));
            assert!(valid.is_empty(), "{}: {valid:#?}", element.name);
            match invalid_lexeme(value_type) {
                Some((lexeme, code)) => {
                    typed += 1;
                    let invalid = run(lexeme);
                    assert!(
                        invalid.iter().any(|d| d.code() == code),
                        "{}: {lexeme:?} not reported as {code:?}",
                        element.name
                    );
                }
                None => {
                    // A plain string must really be unchecked: every other
                    // type's invalid lexeme is accepted.
                    for lexeme in [
                        "not a language",
                        "yes",
                        "1,5",
                        "yesterday",
                        "nobody",
                        "NotADeclaredValue",
                    ] {
                        let diagnostics = run(lexeme);
                        assert!(
                            diagnostics.is_empty(),
                            "{}/{} is checked but declared String: {diagnostics:#?}",
                            element.parent.unwrap_or("-"),
                            element.name
                        );
                    }
                }
            }
        }
        assert!(typed >= 20, "only {typed} typed simple elements");
    }

    #[test]
    fn json_escapes_control_characters() {
        let mut out = String::new();
        push_json_string(&mut out, "a\"b\\c\u{1}");
        assert_eq!(out, "\"a\\\"b\\\\c\\u0001\"");
    }
}
