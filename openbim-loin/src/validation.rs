//! ISO 7817-3 clause-level document validation.
//!
//! This is deliberately described as XSD-derived structural and lexical validation,
//! not as a complete XML Schema processor. It checks the ISO 7817-3 declarations
//! owned by this crate and delegates lexical contracts such as GUID, `xs:language`,
//! `xs:dateTime`, and `xs:decimal` to `openbim-dt`. Imported ISO 23387 complex-type
//! internals are retained but are outside this validator's complete coverage.

use std::str::FromStr;

use crate::{dt, LoinDocument, XmlElement, XmlNode};

const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Validation severity. Parsing and validation are deliberately separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Stable ISO 7817-3 validation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    UnexpectedNamespace,
    UnknownElement,
    UnexpectedAttribute,
    MissingRequiredAttribute,
    MissingRequiredChild,
    TooManyChildren,
    ChildOutOfOrder,
    UnexpectedText,
    InvalidGuid,
    InvalidAnyUri,
    MissingLanguage,
    InvalidLanguage,

    InvalidDateTime,
    InvalidBoolean,
    InvalidInteger,
    InvalidDecimal,
    InvalidDouble,
    InvalidEnumeration,
    NilledContent,
    UnsupportedXsiType,
    CompatibilityProfile,
}

/// One location-aware clause-level validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    severity: Severity,
    code: DiagnosticCode,
    path: String,
    message: String,
}

impl Diagnostic {
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Copy)]
struct ChildRule {
    name: &'static str,
    min: usize,
    max: Option<usize>,
}

impl ChildRule {
    const fn one(name: &'static str) -> Self {
        Self {
            name,
            min: 1,
            max: Some(1),
        }
    }

    const fn optional(name: &'static str) -> Self {
        Self {
            name,
            min: 0,
            max: Some(1),
        }
    }

    const fn many(name: &'static str, min: usize) -> Self {
        Self {
            name,
            min,
            max: None,
        }
    }
}

#[derive(Clone, Copy)]
struct ContentRule {
    children: &'static [ChildRule],
    ordered: bool,
}

const ROOT: &[ChildRule] = &[ChildRule::many("Specification", 1)];
const SPECIFICATION: &[ChildRule] = &[
    ChildRule::one("Prerequisites"),
    ChildRule::many("SpecificationPerObjectType", 0),
    ChildRule::optional("GeoReferencing"),
];
const PREREQUISITES: &[ChildRule] = &[
    ChildRule::one("Purpose"),
    ChildRule::one("InformationDeliveryMilestone"),
    ChildRule::one("ProvidingActor"),
    ChildRule::one("ReceivingActor"),
];
// The XSD repeats a choice one or more times. Branches may interleave freely,
// but the aggregate choice must contain at least one item.
const PURPOSE: &[ChildRule] = &[
    ChildRule::many("Name", 0),
    ChildRule::many("Definition", 0),
    ChildRule::many("ReferenceDocument", 0),
    ChildRule::many("Description", 0),
    ChildRule::many("Language", 0),
    ChildRule::many("Region", 0),
    ChildRule::many("DictionaryRef", 0),
];
const MILESTONE: &[ChildRule] = &[
    ChildRule::one("Name"),
    ChildRule::many("Description", 0),
    ChildRule::many("ReferenceDocument", 0),
];
const ACTOR_CHILDREN: &[ChildRule] = &[
    ChildRule::one("Role"),
    ChildRule::optional("Description"),
    ChildRule::optional("EMailAddress"),
];
const SPEC_PER_OBJECT: &[ChildRule] = &[
    ChildRule::one("ObjectType"),
    ChildRule::optional("AlphanumericalInformation"),
    ChildRule::optional("Documentation"),
    ChildRule::optional("GeometricalInformation"),
];
const INHERITED_DT_CONCEPT_CHILDREN: &[&str] = &[
    "Name",
    "Definition",
    "ReferenceDocumentRef",
    "Description",
    "Example",
    "SimilarToRef",
    "LanguageOfCreator",
    "CountryOfOrigin",
    "VisualRepresentation",
    "MajorVersion",
    "MinorVersion",
    "Status",
    "ReplacedObjectsRef",
    "DeprecationExplanation",
    "DictionaryRef",
];
const ALPHANUMERICAL: &[ChildRule] = &[
    ChildRule::many("Property", 0),
    ChildRule::many("QuantityKind", 0),
    ChildRule::optional("GroupsOfProperties"),
    ChildRule::many("ReferenceDocument", 0),
    ChildRule::many("Dimension", 0),
    ChildRule::many("Unit", 0),
];
const GROUPS: &[ChildRule] = &[
    ChildRule::many("GroupOfProperties", 0),
    ChildRule::many("GroupOfPropertiesRef", 0),
];
const FORMAT: &[ChildRule] = &[
    ChildRule::many("FormatName", 1),
    ChildRule::many("FormatVersion", 1),
    ChildRule::many("FormatSpecification", 0),
];
const DOCUMENT: &[ChildRule] = &[
    ChildRule::one("Name"),
    ChildRule::many("ReferenceDocument", 0),
    ChildRule::many("Description", 0),
    ChildRule::one("Format"),
];
const DOCUMENTATION: &[ChildRule] = &[ChildRule::many("Document", 0)];
const SHAPE_INFLUENCE: &[ChildRule] = &[
    ChildRule::optional("InsideGeometry"),
    ChildRule::optional("Connections"),
    ChildRule::optional("Openings"),
    ChildRule::optional("OperatingAndClearanceZones"),
    ChildRule::optional("Features"),
    ChildRule::optional("ThresholdDimension"),
];
const DETAIL: &[ChildRule] = &[
    ChildRule::optional("Dictionary"),
    ChildRule::optional("ShapeAssembly"),
    ChildRule::optional("ShapeRepresentation"),
    ChildRule::optional("ShapeInfluence"),
];
const CRS: &[ChildRule] = &[
    ChildRule::one("Type"),
    ChildRule::one("Datum"),
    ChildRule::optional("VerticalDatum"),
];
const MODEL_COORDINATES: &[ChildRule] = &[
    ChildRule::one("IsProjected"),
    ChildRule::one("FirstCoordinate"),
    ChildRule::one("SecondCoordinate"),
    ChildRule::one("Height"),
    ChildRule::optional("XAxisAbscissa"),
    ChildRule::optional("XAxisOrdinate"),
    ChildRule::optional("UnitScale"),
    ChildRule::optional("HorizontalScale"),
];
const POSITIONING: &[ChildRule] = &[
    ChildRule::one("RelativeOrAbsolute"),
    ChildRule::optional("ReferenceObject"),
];
const DATUM: &[ChildRule] = &[ChildRule::one("Name"), ChildRule::one("Type")];
const DATUM_REGISTRY: &[ChildRule] = &[
    ChildRule::one("Name"),
    ChildRule::many("Description", 0),
    ChildRule::optional("RegistryReference"),
];
const GEO_REFERENCE: &[ChildRule] = &[
    ChildRule::optional("CoordinateReferenceSystem"),
    ChildRule::many("ModelCoordinateSystem", 0),
];
const GEOMETRICAL: &[ChildRule] = &[
    ChildRule::optional("Detail"),
    ChildRule::optional("Dimensionality"),
    ChildRule::optional("Appearance"),
    ChildRule::optional("ParametricBehaviour"),
    ChildRule::optional("Location"),
];
const THRESHOLD_DIMENSION: &[ChildRule] = &[
    ChildRule::one("Threshold"),
    ChildRule::one("Unit"),
    ChildRule::one("Definition"),
];
const NO_CHILDREN: &[ChildRule] = &[];

const SHAPE_ASSEMBLY: &[&str] = &[
    "NotRequired",
    "SingleObjectSingularShape",
    "SingleObjectMultipleShapes",
    "MultipleObjects",
];
const SHAPE_REPRESENTATION: &[&str] = &[
    "NotRequired",
    "SingleBoundingPrimitive",
    "OuterShellAsSingularShape",
    "OuterShellAsSeparateShapes",
];
const INSIDE_GEOMETRY: &[&str] = &[
    "NotRequired",
    "NoInsideGeometry",
    "InsideGeometryAsPartOfShape",
    "SeparateShapes",
];
const CONNECTIONS: &[&str] = &[
    "NotRequired",
    "NoConnections",
    "ConnectionsAsPartOfShape",
    "SeparateShapes",
];
const OPENINGS: &[&str] = &[
    "NotRequired",
    "NoOpenings",
    "OpeningsAsPartOfShape",
    "SeparateShapes",
];
const ZONES: &[&str] = &[
    "NotRequired",
    "NoZones",
    "ZonesAsPartOfShape",
    "SeparateShapes",
];
const FEATURES: &[&str] = &[
    "NotRequired",
    "NoFeatures",
    "FeaturesAsPartOfShape",
    "SeparateShapes",
];
const DIMENSIONALITY: &[&str] = &["NotRequired", "0D", "1D", "2D", "3D"];
const APPEARANCE: &[&str] = &[
    "NotRequired",
    "NoAppearanceInformation",
    "SymbolicByMapping",
    "SingularMaterial",
    "MultipleMaterials",
    "ConceptualAppearance",
    "RealisticAppearance",
];
const PARAMETRIC: &[&str] = &["NotRequested", "Requested"];
const POSITION: &[&str] = &["NotDefined", "Absolute", "Relative"];
const CRS_TYPE: &[&str] = &[
    "NotRequired",
    "ProjectedCRS",
    "EngineeringCRS",
    "GeographicCRS",
];

pub(crate) fn validate_document(document: &LoinDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.current_namespace() == crate::NamespaceVersion::Draft2022 {
        push(
            &mut diagnostics,
            Severity::Warning,
            DiagnosticCode::CompatibilityProfile,
            "/LevelOfInformationNeed",
            "2022 namespace validation uses the documented compatibility profile, not the audited 2024 schema",
        );
    }
    visit(
        document.root(),
        None,
        "/LevelOfInformationNeed",
        &mut diagnostics,
    );
    diagnostics
}

fn visit(
    element: &XmlElement,
    parent: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if parent.is_none() {
        if element
            .namespace_uri()
            .and_then(crate::NamespaceVersion::from_uri)
            .is_none()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnexpectedNamespace,
                path,
                "root is not bound to a supported ISO 7817-3 namespace",
            );
        }
    } else if element.namespace_uri() == Some(dt::NAMESPACE) {
        if is_imported_dt_complex(element.local_name(), parent)
            || content_rule(element.local_name(), parent).is_some()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnexpectedNamespace,
                path,
                "schema-local elements using imported ISO 23387 types remain unqualified",
            );
            return;
        }
        validate_dt_element(element, path, diagnostics);
        return;
    } else if let Some(namespace) = element.namespace_uri() {
        if crate::NamespaceVersion::from_uri(namespace).is_some() {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnexpectedNamespace,
                path,
                "ISO 7817-3 local elements are unqualified by the published schema",
            );
        } else {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnknownElement,
                path,
                format!("extension element in undeclared namespace {namespace:?} is outside the audited grammar"),
            );
            return;
        }
    }

    if is_imported_dt_complex(element.local_name(), parent) {
        validate_imported_dt_subtree(element, path, diagnostics);
        return;
    }

    validate_attributes(element, parent, path, diagnostics);
    if element.local_name() == "SpecificationPerObjectType" && is_xsi_nilled(element) {
        return;
    }

    let Some(content) = content_rule(element.local_name(), parent) else {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::UnknownElement,
            path,
            format!("unknown ISO 7817-3 element {}", element.qname()),
        );
        return;
    };

    validate_lexical_content(element, parent, path, diagnostics);
    validate_children(element, content, path, diagnostics);
}

fn content_rule(name: &str, parent: Option<&str>) -> Option<ContentRule> {
    let (children, ordered) = match (parent, name) {
        (None, "LevelOfInformationNeed") => (ROOT, true),
        (Some("LevelOfInformationNeed"), "Specification") => (SPECIFICATION, true),
        (Some("Specification"), "Prerequisites") => (PREREQUISITES, true),
        (Some("Specification"), "SpecificationPerObjectType") => (SPEC_PER_OBJECT, true),
        (Some("Specification"), "GeoReferencing") => (GEO_REFERENCE, true),
        (Some("Prerequisites"), "Purpose") => (PURPOSE, false),
        (Some("Prerequisites"), "InformationDeliveryMilestone") => (MILESTONE, true),
        (Some("Prerequisites"), "ProvidingActor" | "ReceivingActor") => (ACTOR_CHILDREN, true),
        (Some("SpecificationPerObjectType"), "AlphanumericalInformation") => (ALPHANUMERICAL, true),
        (Some("SpecificationPerObjectType"), "Documentation") => (DOCUMENTATION, true),
        (Some("SpecificationPerObjectType"), "GeometricalInformation") => (GEOMETRICAL, true),

        (Some("AlphanumericalInformation"), "GroupsOfProperties") => (GROUPS, true),
        (Some("Documentation"), "Document") => (DOCUMENT, true),
        (Some("Document"), "Format") => (FORMAT, true),
        (Some("GeometricalInformation"), "Detail") => (DETAIL, true),
        (Some("Detail"), "ShapeInfluence") => (SHAPE_INFLUENCE, true),
        (Some("GeoReferencing"), "CoordinateReferenceSystem") => (CRS, true),
        (Some("GeoReferencing"), "ModelCoordinateSystem") => (MODEL_COORDINATES, true),
        (Some("GeometricalInformation"), "Location") => (POSITIONING, true),
        (Some("CoordinateReferenceSystem"), "Datum" | "VerticalDatum") => (DATUM, true),
        (Some("Datum" | "VerticalDatum"), "Type") => (DATUM_REGISTRY, true),
        (Some("ShapeInfluence"), "ThresholdDimension") => (THRESHOLD_DIMENSION, true),
        // Simple-content LOIN elements. Parent matching keeps names out of forbidden locations.
        (Some("Purpose"), "Name" | "Definition" | "Description" | "Language" | "Region")
        | (Some("InformationDeliveryMilestone"), "Name" | "Description")
        | (Some("ProvidingActor" | "ReceivingActor"), "Role" | "EMailAddress" | "Description")
        | (Some("Document"), "Name" | "Description")
        | (Some("Format"), "FormatName" | "FormatVersion")
        | (Some("Detail"), "ShapeAssembly" | "ShapeRepresentation")
        | (
            Some("ShapeInfluence"),
            "InsideGeometry"
            | "Connections"
            | "Openings"
            | "OperatingAndClearanceZones"
            | "Features",
        )
        | (
            Some("GeometricalInformation"),
            "Dimensionality" | "Appearance" | "ParametricBehaviour",
        )
        | (Some("CoordinateReferenceSystem"), "Type")
        | (
            Some("ModelCoordinateSystem"),
            "IsProjected" | "FirstCoordinate" | "SecondCoordinate" | "Height" | "XAxisAbscissa"
            | "XAxisOrdinate" | "UnitScale" | "HorizontalScale",
        )
        | (Some("Location"), "RelativeOrAbsolute" | "ReferenceObject")
        | (Some("Datum" | "VerticalDatum"), "Name")
        | (Some("Type"), "Name" | "Description")
        | (Some("ThresholdDimension"), "Threshold" | "Definition") => (NO_CHILDREN, true),
        _ => return None,
    };
    Some(ContentRule { children, ordered })
}

fn is_imported_dt_complex(name: &str, parent: Option<&str>) -> bool {
    matches!(
        (parent, name),
        (Some("SpecificationPerObjectType"), "ObjectType")
            | (Some("Purpose"), "ReferenceDocument" | "DictionaryRef")
            | (Some("InformationDeliveryMilestone"), "ReferenceDocument")
            | (
                Some("AlphanumericalInformation"),
                "Property" | "QuantityKind" | "ReferenceDocument" | "Dimension" | "Unit"
            )
            | (
                Some("GroupsOfProperties"),
                "GroupOfProperties" | "GroupOfPropertiesRef"
            )
            | (Some("Document"), "ReferenceDocument")
            | (Some("Format"), "FormatSpecification")
            | (Some("Detail"), "Dictionary")
            | (Some("Type"), "RegistryReference")
            | (Some("ThresholdDimension"), "Unit")
    )
}

fn validate_imported_dt_subtree(
    element: &XmlElement,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for attribute in element.attributes() {
        if attribute.namespace_uri() == Some(dt::NAMESPACE)
            && attribute.local_name() == "GUID"
            && dt::Guid::from_str(attribute.value()).is_err()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::InvalidGuid,
                path,
                "invalid GUID in imported ISO 23387 content",
            );
        }
        if attribute.namespace_uri() == Some(dt::NAMESPACE)
            && attribute.local_name() == "referenceURI"
            && dt::AnyUri::from_str(attribute.value()).is_err()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::InvalidAnyUri,
                path,
                "invalid XML Schema anyURI in imported ISO 23387 content",
            );
        }
    }
    for (index, child) in element.children().enumerate() {
        let child_path = format!("{path}/{}[{}]", child.local_name(), index + 1);
        validate_imported_dt_subtree(child, &child_path, diagnostics);
    }
}

fn validate_children(
    element: &XmlElement,
    rule: ContentRule,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let children: Vec<&XmlElement> = element.children().collect();
    let mut counts = vec![0usize; rule.children.len()];
    let mut last_rule = 0usize;
    let mut seen_known = false;
    let mut inherited_dt_count = 0usize;

    for (position, child) in children.iter().enumerate() {
        let child_path = format!("{path}/{}[{}]", child.local_name(), position + 1);

        // The imported DT base contributes namespaced children before the LOIN extension.
        if element.local_name() == "SpecificationPerObjectType"
            && child.namespace_uri() == Some(dt::NAMESPACE)
            && !rule
                .children
                .iter()
                .any(|expected| expected.name == child.local_name())
        {
            if !INHERITED_DT_CONCEPT_CHILDREN.contains(&child.local_name()) {
                push(
                    diagnostics,
                    Severity::Error,
                    DiagnosticCode::UnknownElement,
                    &child_path,
                    "undeclared ISO 23387 child in the inherited Concept content",
                );
                continue;
            }
            inherited_dt_count += 1;
            if seen_known {
                push(
                    diagnostics,
                    Severity::Error,
                    DiagnosticCode::ChildOutOfOrder,
                    &child_path,
                    "imported ISO 23387 base content must precede the LOIN extension",
                );
            }
            validate_imported_dt_subtree(child, &child_path, diagnostics);
            continue;
        }

        let Some(index) = rule
            .children
            .iter()
            .position(|expected| expected.name == child.local_name())
        else {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnknownElement,
                &child_path,
                format!("{} is not allowed below {}", child.qname(), element.qname()),
            );
            continue;
        };

        if child.namespace_uri().is_some() {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnexpectedNamespace,
                &child_path,
                format!("schema-local element {} must be unqualified", child.qname()),
            );
            continue;
        }

        counts[index] += 1;
        if rule.ordered && seen_known && index < last_rule {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::ChildOutOfOrder,
                path,
                format!("{} occurs outside its XSD sequence position", child.qname()),
            );
        }
        last_rule = last_rule.max(index);
        seen_known = true;
        let child_path = format!("{path}/{}[{}]", child.local_name(), counts[index]);
        visit(child, Some(element.local_name()), &child_path, diagnostics);
    }

    let declared_count = counts.iter().sum::<usize>();
    if element.local_name() == "SpecificationPerObjectType" && inherited_dt_count == 0 {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingRequiredChild,
            path,
            "SpecificationPerObjectType requires inherited ISO 23387 Concept content",
        );
    }

    for (expected, count) in rule.children.iter().zip(counts) {
        if count < expected.min {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::MissingRequiredChild,
                path,
                format!(
                    "{} requires at least {} occurrence(s) of {}",
                    element.qname(),
                    expected.min,
                    expected.name
                ),
            );
        }
        if expected.max.is_some_and(|max| count > max) {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::TooManyChildren,
                path,
                format!("{} occurs too many times", expected.name),
            );
        }
    }

    if element.local_name() == "Purpose" && declared_count == 0 {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingRequiredChild,
            path,
            "Purpose requires at least one declared choice item",
        );
    }

    if !rule.children.is_empty() && has_non_whitespace_text(element) {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::UnexpectedText,
            path,
            "complex content contains non-whitespace character data",
        );
    }
}

fn validate_attributes(
    element: &XmlElement,
    parent: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = element.local_name();
    let required_guid = matches!(
        (parent, name),
        (Some("LevelOfInformationNeed"), "Specification")
            | (
                Some("Specification"),
                "Prerequisites" | "SpecificationPerObjectType"
            )
            | (
                Some("Prerequisites"),
                "Purpose" | "InformationDeliveryMilestone" | "ProvidingActor" | "ReceivingActor"
            )
            | (
                Some("SpecificationPerObjectType"),
                "AlphanumericalInformation"
            )
            | (Some("SpecificationPerObjectType"), "Documentation")
            | (Some("Documentation"), "Document")
            | (Some("SpecificationPerObjectType"), "GeometricalInformation")
    );
    let multilingual = is_multilingual(name, parent);
    let mut guid_seen = false;
    let mut language_seen = false;
    let mut name_seen = false;
    let mut creation_seen = false;
    let mut nilled = false;

    for attribute in element.attributes() {
        if attribute.namespace_uri() == Some(XMLNS_NAMESPACE) {
            continue;
        }
        let allowed = match (attribute.namespace_uri(), attribute.local_name()) {
            (Some(namespace), "schemaLocation" | "noNamespaceSchemaLocation")
                if namespace == XSI_NAMESPACE =>
            {
                true
            }
            (Some(namespace), "type") if namespace == XSI_NAMESPACE => {
                push(
                    diagnostics,
                    Severity::Error,
                    DiagnosticCode::UnsupportedXsiType,
                    path,
                    "xsi:type derivation is outside this clause validator",
                );
                true
            }
            (Some(namespace), "nil")
                if namespace == XSI_NAMESPACE && name == "SpecificationPerObjectType" =>
            {
                validate_boolean(attribute.value(), path, diagnostics);
                nilled = matches!(
                    collapse_whitespace(attribute.value()).as_str(),
                    "true" | "1"
                );
                true
            }
            (Some(namespace), "GUID") if namespace == dt::NAMESPACE && required_guid => {
                guid_seen = true;
                if dt::Guid::from_str(attribute.value()).is_err() {
                    push(
                        diagnostics,
                        Severity::Error,
                        DiagnosticCode::InvalidGuid,
                        path,
                        format!("invalid ISO 23387 GUID on {}", element.qname()),
                    );
                }
                true
            }
            (Some(namespace), "about")
                if namespace == dt::NAMESPACE && name == "SpecificationPerObjectType" =>
            {
                if dt::AnyUri::from_str(attribute.value()).is_err() {
                    push(
                        diagnostics,
                        Severity::Error,
                        DiagnosticCode::InvalidAnyUri,
                        path,
                        "dt:about is not an xs:anyURI lexical value",
                    );
                }
                true
            }
            (None, "language") if multilingual => {
                language_seen = true;
                if dt::Language::from_str(&collapse_whitespace(attribute.value())).is_err() {
                    push(
                        diagnostics,
                        Severity::Error,
                        DiagnosticCode::InvalidLanguage,
                        path,
                        "invalid xs:language lexical value",
                    );
                }
                true
            }
            (None, "name") if name == "Specification" => {
                name_seen = true;
                true
            }
            (None, "dateOfCreation") if name == "SpecificationPerObjectType" => {
                creation_seen = true;
                if dt::DateTime::from_str(attribute.value()).is_err() {
                    push(
                        diagnostics,
                        Severity::Error,
                        DiagnosticCode::InvalidDateTime,
                        path,
                        "dateOfCreation is not an xs:dateTime lexical value",
                    );
                }
                true
            }
            (None, "Date") if name == "InformationDeliveryMilestone" => {
                if dt::DateTime::from_str(attribute.value()).is_err() {
                    push(
                        diagnostics,
                        Severity::Error,
                        DiagnosticCode::InvalidDateTime,
                        path,
                        "Date is not an xs:dateTime lexical value",
                    );
                }
                true
            }
            (None, "firstName" | "middleName" | "lastName" | "affiliation")
                if matches!(name, "ProvidingActor" | "ReceivingActor") =>
            {
                true
            }
            (None, "type" | "form" | "content") if name == "Document" => true,
            (None, "placeholder") if name == "GeometricalInformation" => {
                validate_boolean(attribute.value(), path, diagnostics);
                true
            }
            _ => false,
        };
        if !allowed {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnexpectedAttribute,
                path,
                format!(
                    "attribute {} is not declared for {}",
                    attribute.qname(),
                    name
                ),
            );
        }
    }

    if required_guid && !guid_seen {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingRequiredAttribute,
            path,
            format!("{name} requires dt:GUID"),
        );
    }
    if multilingual && !language_seen {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingLanguage,
            path,
            format!("{name} requires an xs:language attribute"),
        );
    }
    if name == "Specification" && !name_seen {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingRequiredAttribute,
            path,
            "Specification requires @name",
        );
    }
    if name == "SpecificationPerObjectType" && !creation_seen {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::MissingRequiredAttribute,
            path,
            "SpecificationPerObjectType requires @dateOfCreation",
        );
    }
    if nilled && (element.children().next().is_some() || has_text_content(element)) {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::NilledContent,
            path,
            "xsi:nil=true forbids child elements and character content",
        );
    }
}

fn is_xsi_nilled(element: &XmlElement) -> bool {
    element.attributes().iter().any(|attribute| {
        attribute.namespace_uri() == Some(XSI_NAMESPACE)
            && attribute.local_name() == "nil"
            && matches!(
                collapse_whitespace(attribute.value()).as_str(),
                "true" | "1"
            )
    })
}

fn validate_dt_element(element: &XmlElement, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if matches!(element.local_name(), "Name" | "Definition" | "Description") {
        validate_attributes(
            element,
            Some("SpecificationPerObjectType"),
            path,
            diagnostics,
        );
        if element.children().next().is_some() {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::UnknownElement,
                path,
                "ISO 23387 multilingual text cannot contain child elements",
            );
        }
    }
    for attribute in element.attributes() {
        if attribute.namespace_uri() == Some(dt::NAMESPACE)
            && attribute.local_name() == "GUID"
            && dt::Guid::from_str(attribute.value()).is_err()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::InvalidGuid,
                path,
                "invalid GUID in retained ISO 23387 content",
            );
        }
        if attribute.namespace_uri() == Some(dt::NAMESPACE)
            && attribute.local_name() == "referenceURI"
            && dt::AnyUri::from_str(attribute.value()).is_err()
        {
            push(
                diagnostics,
                Severity::Error,
                DiagnosticCode::InvalidAnyUri,
                path,
                "invalid XML Schema anyURI in retained ISO 23387 content",
            );
        }
    }
}

fn validate_lexical_content(
    element: &XmlElement,
    parent: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let value = text_value(element);
    let name = element.local_name();
    let trimmed = collapse_whitespace(&value);

    if parent == Some("Purpose") && name == "Language" && dt::Language::from_str(&trimmed).is_err()
    {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidLanguage,
            path,
            "Language is not an xs:language lexical value",
        );
    }
    if parent == Some("ThresholdDimension") && name == "Threshold" && !valid_xs_double(&trimmed) {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidDouble,
            path,
            "Threshold is not an xs:double lexical value",
        );
    }
    if matches!(parent, Some("ProvidingActor" | "ReceivingActor"))
        && name == "EMailAddress"
        && !crate::model::matches_actor_email_pattern(&value)
    {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidEnumeration,
            path,
            "EMailAddress does not match the ISO 7817-3 email restriction",
        );
    }
    if parent == Some("ModelCoordinateSystem")
        && matches!(
            name,
            "FirstCoordinate"
                | "SecondCoordinate"
                | "Height"
                | "XAxisAbscissa"
                | "XAxisOrdinate"
                | "UnitScale"
                | "HorizontalScale"
        )
        && dt::Decimal::from_str(&trimmed).is_err()
    {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidDecimal,
            path,
            format!("{name} is not an xs:decimal lexical value"),
        );
    }
    if parent == Some("ModelCoordinateSystem") && name == "IsProjected" {
        validate_boolean(&value, path, diagnostics);
    }

    let values = match (parent, name) {
        (Some("Detail"), "ShapeAssembly") => Some(SHAPE_ASSEMBLY),
        (Some("Detail"), "ShapeRepresentation") => Some(SHAPE_REPRESENTATION),
        (Some("ShapeInfluence"), "InsideGeometry") => Some(INSIDE_GEOMETRY),
        (Some("ShapeInfluence"), "Connections") => Some(CONNECTIONS),
        (Some("ShapeInfluence"), "Openings") => Some(OPENINGS),
        (Some("ShapeInfluence"), "OperatingAndClearanceZones") => Some(ZONES),
        (Some("ShapeInfluence"), "Features") => Some(FEATURES),
        (Some("GeometricalInformation"), "Dimensionality") => Some(DIMENSIONALITY),
        (Some("GeometricalInformation"), "Appearance") => Some(APPEARANCE),
        (Some("GeometricalInformation"), "ParametricBehaviour") => Some(PARAMETRIC),
        (Some("Location"), "RelativeOrAbsolute") => Some(POSITION),
        (Some("CoordinateReferenceSystem"), "Type") => Some(CRS_TYPE),
        _ => None,
    };
    if values.is_some_and(|allowed| !allowed.contains(&value.as_str())) {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidEnumeration,
            path,
            format!("{value:?} is not a declared {name} value"),
        );
    }
}

fn is_multilingual(name: &str, parent: Option<&str>) -> bool {
    matches!(
        (parent, name),
        (
            Some("SpecificationPerObjectType"),
            "Name" | "Definition" | "Description"
        ) | (Some("Purpose"), "Name" | "Definition" | "Description")
            | (Some("InformationDeliveryMilestone"), "Name" | "Description")
            | (
                Some("ProvidingActor" | "ReceivingActor"),
                "Role" | "Description"
            )
            | (Some("Document"), "Name" | "Description")
            | (Some("Format"), "FormatName" | "FormatVersion")
            | (Some("ThresholdDimension"), "Definition")
            | (Some("Type"), "Name" | "Description")
    )
}

fn text_value(element: &XmlElement) -> String {
    let mut value = String::new();
    for node in element.nodes() {
        match node {
            XmlNode::Text(text) | XmlNode::CData(text) => value.push_str(text),
            _ => {}
        }
    }
    value
}

fn has_text_content(element: &XmlElement) -> bool {
    element.nodes().iter().any(
        |node| matches!(node, XmlNode::Text(value) | XmlNode::CData(value) if !value.is_empty()),
    )
}

fn is_xsd_whitespace(character: char) -> bool {
    matches!(character, '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}')
}

fn has_non_whitespace_text(element: &XmlElement) -> bool {
    element.nodes().iter().any(|node| {
        matches!(node, XmlNode::Text(text) | XmlNode::CData(text) if text.chars().any(|character| !is_xsd_whitespace(character)))
    })
}

fn validate_boolean(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !matches!(
        collapse_whitespace(value).as_str(),
        "true" | "false" | "1" | "0"
    ) {
        push(
            diagnostics,
            Severity::Error,
            DiagnosticCode::InvalidBoolean,
            path,
            format!("{value:?} is not an xs:boolean lexical value"),
        );
    }
}

fn valid_xs_double(value: &str) -> bool {
    if matches!(value, "INF" | "-INF" | "NaN") {
        return true;
    }
    let bytes = value.as_bytes();
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let integer_digits = index - integer_start;
    let mut fractional_digits = 0;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fractional_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        fractional_digits = index - fractional_start;
    }
    if integer_digits + fractional_digits == 0 {
        return false;
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_start {
            return false;
        }
    }
    index == bytes.len()
}

fn collapse_whitespace(value: &str) -> String {
    value
        .split(is_xsd_whitespace)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn push(
    diagnostics: &mut Vec<Diagnostic>,
    severity: Severity,
    code: DiagnosticCode,
    path: &str,
    message: impl Into<String>,
) {
    diagnostics.push(Diagnostic {
        severity,
        code,
        path: path.to_owned(),
        message: message.into(),
    });
}
