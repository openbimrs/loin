//! ISO 7817-3 contracts that directly consume ISO 23387-owned types.

use openbim_dt::{
    Concept, DateTime, Dimension, GroupOfProperties, Guid, Language, MultiLanguageText, ObjectType,
    Property, QuantityKind, Reference, ReferenceDocument, Unit,
};

/// LOIN `PurposeType`; its identity, text, and references are DT-owned contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purpose {
    guid: Guid,
    name: MultiLanguageText,
    definition: Option<MultiLanguageText>,
    reference_documents: Vec<Reference>,
    descriptions: Vec<MultiLanguageText>,
    language: Option<Language>,
    region: Option<String>,
    dictionary_ref: Option<Reference>,
}

impl Purpose {
    #[must_use]
    pub const fn new(guid: Guid, name: MultiLanguageText) -> Self {
        Self {
            guid,
            name,
            definition: None,
            reference_documents: Vec::new(),
            descriptions: Vec::new(),
            language: None,
            region: None,
            dictionary_ref: None,
        }
    }

    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub const fn name(&self) -> &MultiLanguageText {
        &self.name
    }
    pub fn set_definition(&mut self, value: Option<MultiLanguageText>) {
        self.definition = value;
    }
    pub fn add_reference_document(&mut self, value: Reference) {
        self.reference_documents.push(value);
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
    pub fn set_language(&mut self, value: Option<Language>) {
        self.language = value;
    }
    #[must_use]
    pub const fn language(&self) -> Option<&Language> {
        self.language.as_ref()
    }
    pub fn set_region(&mut self, value: Option<String>) {
        self.region = value;
    }
    pub fn set_dictionary_ref(&mut self, value: Option<Reference>) {
        self.dictionary_ref = value;
    }
}

/// LOIN `ActorType`, using DT identity and multilingual text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    guid: Guid,
    role: MultiLanguageText,
    description: Option<MultiLanguageText>,
}

impl Actor {
    #[must_use]
    pub const fn new(guid: Guid, role: MultiLanguageText) -> Self {
        Self {
            guid,
            role,
            description: None,
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub const fn role(&self) -> &MultiLanguageText {
        &self.role
    }
    pub fn set_role(&mut self, value: MultiLanguageText) {
        self.role = value;
    }
    pub fn set_description(&mut self, value: Option<MultiLanguageText>) {
        self.description = value;
    }
}

/// LOIN `InformationDeliveryMilestoneType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformationDeliveryMilestone {
    guid: Guid,
    name: MultiLanguageText,
    descriptions: Vec<MultiLanguageText>,
    reference_documents: Vec<Reference>,
    date: Option<DateTime>,
}

impl InformationDeliveryMilestone {
    #[must_use]
    pub const fn new(guid: Guid, name: MultiLanguageText) -> Self {
        Self {
            guid,
            name,
            descriptions: Vec::new(),
            reference_documents: Vec::new(),
            date: None,
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub const fn name(&self) -> &MultiLanguageText {
        &self.name
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
    pub fn add_reference_document(&mut self, value: Reference) {
        self.reference_documents.push(value);
    }
    pub fn set_date(&mut self, value: Option<DateTime>) {
        self.date = value;
    }
}

/// LOIN prerequisites; all referenced identities are DT-owned GUID contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prerequisites {
    guid: Guid,
    purpose: Purpose,
    milestone: InformationDeliveryMilestone,
    providing_actor: Actor,
    receiving_actor: Actor,
}

impl Prerequisites {
    #[must_use]
    pub const fn new(
        guid: Guid,
        purpose: Purpose,
        milestone: InformationDeliveryMilestone,
        providing_actor: Actor,
        receiving_actor: Actor,
    ) -> Self {
        Self {
            guid,
            purpose,
            milestone,
            providing_actor,
            receiving_actor,
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub const fn purpose(&self) -> &Purpose {
        &self.purpose
    }
    #[must_use]
    pub const fn milestone(&self) -> &InformationDeliveryMilestone {
        &self.milestone
    }
    #[must_use]
    pub const fn providing_actor(&self) -> &Actor {
        &self.providing_actor
    }
    #[must_use]
    pub const fn receiving_actor(&self) -> &Actor {
        &self.receiving_actor
    }
}

/// Root LOIN specification contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    guid: Guid,
    name: String,
    prerequisites: Prerequisites,
    per_object: Vec<SpecificationPerObjectType>,
}

impl Specification {
    #[must_use]
    pub fn new(guid: Guid, name: impl Into<String>, prerequisites: Prerequisites) -> Self {
        Self {
            guid,
            name: name.into(),
            prerequisites,
            per_object: Vec::new(),
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn prerequisites(&self) -> &Prerequisites {
        &self.prerequisites
    }
    pub fn add_per_object(&mut self, value: SpecificationPerObjectType) {
        self.per_object.push(value);
    }
    #[must_use]
    pub fn per_object(&self) -> &[SpecificationPerObjectType] {
        &self.per_object
    }
}

/// Anonymous LOIN alphanumerical-information content using imported DT types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphanumericalInformation {
    properties: Vec<Property>,
    quantity_kinds: Vec<QuantityKind>,
    groups_of_properties: Vec<GroupOfProperties>,
    group_refs: Vec<Reference>,
    reference_documents: Vec<ReferenceDocument>,
    dimensions: Vec<Dimension>,
    units: Vec<Unit>,
}

impl Default for AlphanumericalInformation {
    fn default() -> Self {
        Self::new()
    }
}

impl AlphanumericalInformation {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            properties: Vec::new(),
            quantity_kinds: Vec::new(),
            groups_of_properties: Vec::new(),
            group_refs: Vec::new(),
            reference_documents: Vec::new(),
            dimensions: Vec::new(),
            units: Vec::new(),
        }
    }
    pub fn add_property(&mut self, value: Property) {
        self.properties.push(value);
    }
    pub fn add_quantity_kind(&mut self, value: QuantityKind) {
        self.quantity_kinds.push(value);
    }
    pub fn add_group_of_properties(&mut self, value: GroupOfProperties) {
        self.groups_of_properties.push(value);
    }
    pub fn add_group_ref(&mut self, value: Reference) {
        self.group_refs.push(value);
    }
    pub fn add_reference_document(&mut self, value: ReferenceDocument) {
        self.reference_documents.push(value);
    }
    pub fn add_dimension(&mut self, value: Dimension) {
        self.dimensions.push(value);
    }
    pub fn add_unit(&mut self, value: Unit) {
        self.units.push(value);
    }
    #[must_use]
    pub fn properties(&self) -> &[Property] {
        &self.properties
    }
    #[must_use]
    pub fn quantity_kinds(&self) -> &[QuantityKind] {
        &self.quantity_kinds
    }
    #[must_use]
    pub fn groups_of_properties(&self) -> &[GroupOfProperties] {
        &self.groups_of_properties
    }
    #[must_use]
    pub fn group_refs(&self) -> &[Reference] {
        &self.group_refs
    }
    #[must_use]
    pub fn reference_documents(&self) -> &[ReferenceDocument] {
        &self.reference_documents
    }
    #[must_use]
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }
    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }
}

/// One LOIN requirement per object type; its base and imported content are DT-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationPerObjectType {
    concept: Concept,
    object_type: ObjectType,
    alphanumerical_information: Option<AlphanumericalInformation>,
    documentation: Option<Documentation>,
    geometrical_information: Option<GeometricalInformation>,
}

impl SpecificationPerObjectType {
    #[must_use]
    pub const fn new(concept: Concept, object_type: ObjectType) -> Self {
        Self {
            concept,
            object_type,
            alphanumerical_information: None,
            documentation: None,
            geometrical_information: None,
        }
    }
    #[must_use]
    pub const fn concept(&self) -> &Concept {
        &self.concept
    }
    #[must_use]
    pub const fn object_type(&self) -> &ObjectType {
        &self.object_type
    }
    pub fn set_alphanumerical_information(&mut self, value: Option<AlphanumericalInformation>) {
        self.alphanumerical_information = value;
    }
    #[must_use]
    pub const fn alphanumerical_information(&self) -> Option<&AlphanumericalInformation> {
        self.alphanumerical_information.as_ref()
    }
    pub fn set_documentation(&mut self, value: Option<Documentation>) {
        self.documentation = value;
    }
    #[must_use]
    pub const fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
    pub fn set_geometrical_information(&mut self, value: Option<GeometricalInformation>) {
        self.geometrical_information = value;
    }
    #[must_use]
    pub const fn geometrical_information(&self) -> Option<&GeometricalInformation> {
        self.geometrical_information.as_ref()
    }
}

/// LOIN `FormatType` using DT multilingual text and references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFormat {
    names: Vec<MultiLanguageText>,
    versions: Vec<MultiLanguageText>,
    specifications: Vec<Reference>,
}

impl DocumentFormat {
    #[must_use]
    pub fn new(name: MultiLanguageText, version: MultiLanguageText) -> Self {
        Self {
            names: vec![name],
            versions: vec![version],
            specifications: Vec::new(),
        }
    }
    pub fn add_name(&mut self, value: MultiLanguageText) {
        self.names.push(value);
    }
    pub fn add_version(&mut self, value: MultiLanguageText) {
        self.versions.push(value);
    }
    pub fn add_specification(&mut self, value: Reference) {
        self.specifications.push(value);
    }
}

/// LOIN `DocumentationType`, whose identity attribute is DT-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documentation {
    guid: Guid,
    required_documents: Vec<RequiredDocument>,
}

impl Documentation {
    #[must_use]
    pub const fn new(guid: Guid) -> Self {
        Self {
            guid,
            required_documents: Vec::new(),
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    pub fn add_required_document(&mut self, value: RequiredDocument) {
        self.required_documents.push(value);
    }
    #[must_use]
    pub fn required_documents(&self) -> &[RequiredDocument] {
        &self.required_documents
    }
}

/// LOIN geometrical-information identity using the imported DT GUID attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricalInformation {
    guid: Guid,
}

impl GeometricalInformation {
    #[must_use]
    pub const fn new(guid: Guid) -> Self {
        Self { guid }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
}

/// LOIN `RequiredDocumentType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredDocument {
    guid: Guid,
    name: MultiLanguageText,
    format: DocumentFormat,
    references: Vec<Reference>,
    descriptions: Vec<MultiLanguageText>,
}

impl RequiredDocument {
    #[must_use]
    pub const fn new(guid: Guid, name: MultiLanguageText, format: DocumentFormat) -> Self {
        Self {
            guid,
            name,
            format,
            references: Vec::new(),
            descriptions: Vec::new(),
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    pub fn add_reference(&mut self, value: Reference) {
        self.references.push(value);
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
}

/// LOIN threshold dimension importing DT unit and text contracts.
#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdDimension {
    threshold: f64,
    unit: Unit,
    definition: MultiLanguageText,
}

impl ThresholdDimension {
    #[must_use]
    pub const fn new(threshold: f64, unit: Unit, definition: MultiLanguageText) -> Self {
        Self {
            threshold,
            unit,
            definition,
        }
    }
    #[must_use]
    pub const fn unit(&self) -> &Unit {
        &self.unit
    }
}

/// LOIN geometry detail with an optional DT dictionary reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    dictionary: Option<Reference>,
}

impl Detail {
    #[must_use]
    pub const fn new(dictionary: Option<Reference>) -> Self {
        Self { dictionary }
    }
    #[must_use]
    pub const fn dictionary(&self) -> Option<&Reference> {
        self.dictionary.as_ref()
    }
}

/// LOIN datum-registry reference using DT text and reference types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatumRegistryReference {
    name: MultiLanguageText,
    descriptions: Vec<MultiLanguageText>,
    registry_reference: Option<Reference>,
}

impl DatumRegistryReference {
    #[must_use]
    pub const fn new(name: MultiLanguageText) -> Self {
        Self {
            name,
            descriptions: Vec::new(),
            registry_reference: None,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &MultiLanguageText {
        &self.name
    }
    #[must_use]
    pub fn descriptions(&self) -> &[MultiLanguageText] {
        &self.descriptions
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
    pub fn set_registry_reference(&mut self, value: Option<Reference>) {
        self.registry_reference = value;
    }
}
