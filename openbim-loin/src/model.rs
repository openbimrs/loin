//! ISO 7817-3 contracts that directly consume ISO 23387-owned types.

use openbim_dt::{
    Concept, DateTime, Decimal, Dimension, GroupOfProperties, Guid, Language, MultiLanguageText,
    ObjectType, Property, QuantityKind, Reference, ReferenceDocument, Unit,
};
use std::{fmt, str::FromStr};

/// Schema-restricted actor email address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidEmailAddress;

impl fmt::Display for InvalidEmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid ISO 7817-3 email address")
    }
}

impl std::error::Error for InvalidEmailAddress {}

impl FromStr for EmailAddress {
    type Err = InvalidEmailAddress;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !matches_actor_email_pattern(value) {
            return Err(InvalidEmailAddress);
        }
        Ok(Self(value.to_owned()))
    }
}

impl EmailAddress {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Implements the ISO 7817-3 `EMailAddressType` pattern.
///
/// The schema facet is `[^@]+@[^\.]+\..+`, which is implicitly anchored to the
/// whole value. Read literally that means:
///
/// * at least one non-`@` character before the first `@`,
/// * at least one character that is not `.` immediately after it,
/// * then a `.` with at least one character following.
///
/// Note the pattern permits later `@` characters, because `.+` matches them:
/// `a@b@c.de` is schema-valid. Only a value whose first `@` has nothing
/// before it, or which lacks the trailing dotted part, is rejected.
pub(crate) fn matches_actor_email_pattern(value: &str) -> bool {
    let characters: Vec<char> = value.chars().collect();
    // `[^@]+` is greedy but cannot cross an `@`, so the match is anchored on
    // the FIRST `@`. Scanning for any acceptable `@` accepted a leading one.
    let Some(at) = characters.iter().position(|character| *character == '@') else {
        return false;
    };
    if at == 0 {
        return false;
    }
    // `[^\.]+` requires at least one non-dot character after the `@`.
    let rest = &characters[at + 1..];
    let Some(dot) = rest.iter().position(|character| *character == '.') else {
        return false;
    };
    // `.+` requires at least one character after that dot.
    dot > 0 && dot + 1 < rest.len()
}

/// One ordered branch of the repeating `PurposeType` choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurposeItem {
    Name(MultiLanguageText),
    Definition(MultiLanguageText),
    ReferenceDocument(Reference),
    Description(MultiLanguageText),
    Language(Language),
    Region(String),
    DictionaryRef(Reference),
}

/// Returned when an update would remove the required final purpose choice item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyPurpose;

impl fmt::Display for EmptyPurpose {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Purpose requires at least one choice item")
    }
}

impl std::error::Error for EmptyPurpose {}

/// LOIN `PurposeType`; its identity, text, and references are DT-owned contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purpose {
    guid: Guid,
    items: Vec<PurposeItem>,
}

impl Purpose {
    #[must_use]
    pub fn new(guid: Guid, name: MultiLanguageText) -> Self {
        Self::from_item(guid, PurposeItem::Name(name))
    }

    #[must_use]
    pub fn from_item(guid: Guid, first_item: PurposeItem) -> Self {
        Self {
            guid,
            items: vec![first_item],
        }
    }

    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub fn name(&self) -> Option<&MultiLanguageText> {
        self.items.iter().find_map(|item| match item {
            PurposeItem::Name(value) => Some(value),
            _ => None,
        })
    }
    #[must_use]
    pub fn items(&self) -> &[PurposeItem] {
        &self.items
    }
    /// Every `Definition` branch, in document order.
    #[must_use]
    pub fn definitions(&self) -> Vec<&MultiLanguageText> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::Definition(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `Description` branch, in document order.
    #[must_use]
    pub fn purpose_descriptions(&self) -> Vec<&MultiLanguageText> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::Description(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `ReferenceDocument` branch, in document order.
    #[must_use]
    pub fn reference_documents(&self) -> Vec<&Reference> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::ReferenceDocument(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `Language` branch, in document order.
    #[must_use]
    pub fn languages(&self) -> Vec<&Language> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::Language(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `Region` branch, in document order.
    #[must_use]
    pub fn regions(&self) -> Vec<&String> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::Region(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `DictionaryRef` branch, in document order.
    #[must_use]
    pub fn dictionary_refs(&self) -> Vec<&Reference> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::DictionaryRef(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    /// Every `Name` branch, in document order.
    ///
    /// The XSD choice is `maxOccurs="unbounded"`, so any branch may repeat;
    /// ISO `split_example.xml` carries two `Name` branches.
    #[must_use]
    pub fn names(&self) -> Vec<&MultiLanguageText> {
        self.items
            .iter()
            .filter_map(|item| match item {
                PurposeItem::Name(value) => Some(value),
                _ => None,
            })
            .collect()
    }
    pub fn add_item(&mut self, value: PurposeItem) {
        self.items.push(value);
    }

    /// Collapses every matching branch to the single supplied value.
    ///
    /// Use this when the caller means this Purpose has exactly one of a
    /// branch. The first occurrence keeps its document position; later
    /// duplicates are removed. Returns how many were removed, so a caller
    /// can tell a no-op from real data loss.
    fn set_all_items(
        &mut self,
        matches: impl Fn(&PurposeItem) -> bool,
        replacement: Option<PurposeItem>,
    ) -> Result<usize, EmptyPurpose> {
        let before = self.items.iter().filter(|i| matches(i)).count();
        self.replace_optional_item(&matches, replacement)?;
        let mut seen = false;
        self.items.retain(|item| {
            if matches(item) {
                let first = !seen;
                seen = true;
                return first;
            }
            true
        });
        let after = self.items.iter().filter(|i| matches(i)).count();
        Ok(before.saturating_sub(after))
    }

    /// Collapses every `Definition` branch to one value.
    ///
    /// Returns the number of duplicate branches removed.
    /// # Errors
    /// Returns `EmptyPurpose` if this would empty the choice.
    pub fn set_all_definitions(
        &mut self,
        value: Option<MultiLanguageText>,
    ) -> Result<usize, EmptyPurpose> {
        self.set_all_items(
            |item| matches!(item, PurposeItem::Definition(_)),
            value.map(PurposeItem::Definition),
        )
    }

    /// Collapses every `Language` branch to one value.
    ///
    /// Returns the number of duplicate branches removed.
    /// # Errors
    /// Returns `EmptyPurpose` if this would empty the choice.
    pub fn set_all_languages(&mut self, value: Option<Language>) -> Result<usize, EmptyPurpose> {
        self.set_all_items(
            |item| matches!(item, PurposeItem::Language(_)),
            value.map(PurposeItem::Language),
        )
    }

    /// Collapses every `Region` branch to one value.
    ///
    /// Returns the number of duplicate branches removed.
    /// # Errors
    /// Returns `EmptyPurpose` if this would empty the choice.
    pub fn set_all_regions(&mut self, value: Option<String>) -> Result<usize, EmptyPurpose> {
        self.set_all_items(
            |item| matches!(item, PurposeItem::Region(_)),
            value.map(PurposeItem::Region),
        )
    }

    /// Collapses every `DictionaryRef` branch to one value.
    ///
    /// Returns the number of duplicate branches removed.
    /// # Errors
    /// Returns `EmptyPurpose` if this would empty the choice.
    pub fn set_all_dictionary_refs(
        &mut self,
        value: Option<Reference>,
    ) -> Result<usize, EmptyPurpose> {
        self.set_all_items(
            |item| matches!(item, PurposeItem::DictionaryRef(_)),
            value.map(PurposeItem::DictionaryRef),
        )
    }

    /// Replaces the FIRST matching branch in place, leaving any later
    /// occurrences untouched.
    ///
    /// The XSD choice is `maxOccurs="unbounded"`, so a branch may legitimately
    /// repeat. Deleting the extras here would destroy valid authored content,
    /// so callers that mean "there is exactly one" should use the
    /// `set_all_*` family instead.
    fn replace_optional_item(
        &mut self,
        matches: impl Fn(&PurposeItem) -> bool,
        replacement: Option<PurposeItem>,
    ) -> Result<(), EmptyPurpose> {
        if replacement.is_none() && self.items.iter().all(&matches) {
            return Err(EmptyPurpose);
        }
        // Replace the first match where it already sits. Appending instead
        // would relocate the item within this ordered choice and silently
        // rewrite document order (issue #5).
        match self.items.iter().position(&matches) {
            Some(index) => match replacement {
                Some(replacement) => self.items[index] = replacement,
                None => {
                    self.items.remove(index);
                }
            },
            // Nothing to replace: a new value is appended, which is the only
            // position the schema can justify for an item that was absent.
            None => {
                if let Some(replacement) = replacement {
                    self.items.push(replacement);
                }
            }
        }
        Ok(())
    }
    pub fn set_definition(&mut self, value: Option<MultiLanguageText>) -> Result<(), EmptyPurpose> {
        self.replace_optional_item(
            |item| matches!(item, PurposeItem::Definition(_)),
            value.map(PurposeItem::Definition),
        )
    }
    pub fn add_reference_document(&mut self, value: Reference) {
        self.items.push(PurposeItem::ReferenceDocument(value));
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.items.push(PurposeItem::Description(value));
    }
    pub fn set_language(&mut self, value: Option<Language>) -> Result<(), EmptyPurpose> {
        self.replace_optional_item(
            |item| matches!(item, PurposeItem::Language(_)),
            value.map(PurposeItem::Language),
        )
    }
    #[must_use]
    pub fn language(&self) -> Option<&Language> {
        self.items.iter().find_map(|item| match item {
            PurposeItem::Language(value) => Some(value),
            _ => None,
        })
    }
    pub fn set_region(&mut self, value: Option<String>) -> Result<(), EmptyPurpose> {
        self.replace_optional_item(
            |item| matches!(item, PurposeItem::Region(_)),
            value.map(PurposeItem::Region),
        )
    }
    pub fn set_dictionary_ref(&mut self, value: Option<Reference>) -> Result<(), EmptyPurpose> {
        self.replace_optional_item(
            |item| matches!(item, PurposeItem::DictionaryRef(_)),
            value.map(PurposeItem::DictionaryRef),
        )
    }
}

/// LOIN `ActorType`, using DT identity and multilingual text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    guid: Guid,
    role: MultiLanguageText,
    description: Option<MultiLanguageText>,
    email_address: Option<EmailAddress>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    affiliation: Option<String>,
}

impl Actor {
    #[must_use]
    pub const fn new(guid: Guid, role: MultiLanguageText) -> Self {
        Self {
            guid,
            role,
            description: None,
            email_address: None,

            first_name: None,
            middle_name: None,
            last_name: None,
            affiliation: None,
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
    #[must_use]
    pub const fn description(&self) -> Option<&MultiLanguageText> {
        self.description.as_ref()
    }
    pub fn set_description(&mut self, value: Option<MultiLanguageText>) {
        self.description = value;
    }
    #[must_use]
    pub const fn email_address(&self) -> Option<&EmailAddress> {
        self.email_address.as_ref()
    }
    pub fn set_email_address(&mut self, value: Option<EmailAddress>) {
        self.email_address = value;
    }
    #[must_use]
    pub fn first_name(&self) -> Option<&str> {
        self.first_name.as_deref()
    }
    pub fn set_first_name(&mut self, value: Option<String>) {
        self.first_name = value;
    }
    #[must_use]
    pub fn middle_name(&self) -> Option<&str> {
        self.middle_name.as_deref()
    }
    pub fn set_middle_name(&mut self, value: Option<String>) {
        self.middle_name = value;
    }
    #[must_use]
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_deref()
    }
    pub fn set_last_name(&mut self, value: Option<String>) {
        self.last_name = value;
    }
    #[must_use]
    pub fn affiliation(&self) -> Option<&str> {
        self.affiliation.as_deref()
    }
    pub fn set_affiliation(&mut self, value: Option<String>) {
        self.affiliation = value;
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
    #[must_use]
    pub fn descriptions(&self) -> &[MultiLanguageText] {
        &self.descriptions
    }
    #[must_use]
    pub fn reference_documents(&self) -> &[Reference] {
        &self.reference_documents
    }
    #[must_use]
    pub const fn date(&self) -> Option<&DateTime> {
        self.date.as_ref()
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
#[derive(Debug, Clone, PartialEq)]
pub struct Specification {
    guid: Guid,
    name: String,
    prerequisites: Prerequisites,
    per_object: Vec<SpecificationPerObjectType>,
    geo_referencing: Option<GeoReferencing>,
}

impl Specification {
    #[must_use]
    pub fn new(guid: Guid, name: impl Into<String>, prerequisites: Prerequisites) -> Self {
        Self {
            guid,
            name: name.into(),
            prerequisites,
            per_object: Vec::new(),
            geo_referencing: None,
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
    #[must_use]
    pub const fn geo_referencing(&self) -> Option<&GeoReferencing> {
        self.geo_referencing.as_ref()
    }
    pub fn set_geo_referencing(&mut self, value: Option<GeoReferencing>) {
        self.geo_referencing = value;
    }
}

/// Global `LevelOfInformationNeed` document content with at least one specification.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelOfInformationNeed {
    specifications: Vec<Specification>,
}

impl LevelOfInformationNeed {
    #[must_use]
    pub fn new(specification: Specification) -> Self {
        Self {
            specifications: vec![specification],
        }
    }

    pub fn add_specification(&mut self, specification: Specification) {
        self.specifications.push(specification);
    }

    #[must_use]
    pub fn specifications(&self) -> &[Specification] {
        &self.specifications
    }
}

/// Optional groups container; the current schema also permits an empty container.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupsOfProperties {
    groups: Vec<GroupOfProperties>,
    references: Vec<Reference>,
}

impl GroupsOfProperties {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            groups: Vec::new(),
            references: Vec::new(),
        }
    }
    pub fn add_group(&mut self, value: GroupOfProperties) {
        self.groups.push(value);
    }
    pub fn add_reference(&mut self, value: Reference) {
        self.references.push(value);
    }
    #[must_use]
    pub fn groups(&self) -> &[GroupOfProperties] {
        &self.groups
    }
    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.references
    }
}

/// LOIN alphanumerical-information identity and imported DT content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphanumericalInformation {
    guid: Guid,
    properties: Vec<Property>,
    quantity_kinds: Vec<QuantityKind>,
    groups_of_properties: Option<GroupsOfProperties>,
    reference_documents: Vec<ReferenceDocument>,
    dimensions: Vec<Dimension>,
    units: Vec<Unit>,
}

impl AlphanumericalInformation {
    #[must_use]
    pub const fn new(guid: Guid) -> Self {
        Self {
            guid,
            properties: Vec::new(),
            quantity_kinds: Vec::new(),
            groups_of_properties: None,
            reference_documents: Vec::new(),
            dimensions: Vec::new(),
            units: Vec::new(),
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    pub fn add_property(&mut self, value: Property) {
        self.properties.push(value);
    }
    pub fn add_quantity_kind(&mut self, value: QuantityKind) {
        self.quantity_kinds.push(value);
    }
    pub fn add_group_of_properties(&mut self, value: GroupOfProperties) {
        self.groups_of_properties
            .get_or_insert_default()
            .add_group(value);
    }
    pub fn add_group_ref(&mut self, value: Reference) {
        self.groups_of_properties
            .get_or_insert_default()
            .add_reference(value);
    }
    pub fn set_groups_of_properties(&mut self, value: Option<GroupsOfProperties>) {
        self.groups_of_properties = value;
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
        self.groups_of_properties
            .as_ref()
            .map_or(&[], |groups| groups.groups())
    }
    #[must_use]
    pub fn groups_container(&self) -> Option<&GroupsOfProperties> {
        self.groups_of_properties.as_ref()
    }
    #[must_use]
    pub fn group_refs(&self) -> &[Reference] {
        self.groups_of_properties
            .as_ref()
            .map_or(&[], |groups| groups.references())
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

/// One populated LOIN requirement per object type; its base and imported content are DT-owned.
///
/// The nillable XML branch is represented by `LoinDocument`, which retains
/// `xsi:nil` and rejects content; this typed value models the populated branch.
#[derive(Debug, Clone, PartialEq)]
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
    #[must_use]
    pub fn names(&self) -> &[MultiLanguageText] {
        &self.names
    }
    #[must_use]
    pub fn versions(&self) -> &[MultiLanguageText] {
        &self.versions
    }
    #[must_use]
    pub fn specifications(&self) -> &[Reference] {
        &self.specifications
    }
}

/// LOIN `DocumentationType`, whose identity attribute is DT-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documentation {
    guid: Guid,
    documents: Vec<Document>,
}

impl Documentation {
    #[must_use]
    pub const fn new(guid: Guid) -> Self {
        Self {
            guid,
            documents: Vec::new(),
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    pub fn add_document(&mut self, value: Document) {
        self.documents.push(value);
    }
    #[must_use]
    pub fn documents(&self) -> &[Document] {
        &self.documents
    }
}

/// A value outside an ISO 7817-3 enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidEnumerationValue {
    enumeration: &'static str,
    value: String,
}

impl InvalidEnumerationValue {
    fn new(enumeration: &'static str, value: &str) -> Self {
        Self {
            enumeration,
            value: value.to_owned(),
        }
    }
    /// Rust name of the enumeration that rejected the value.
    #[must_use]
    pub const fn enumeration(&self) -> &'static str {
        self.enumeration
    }
    /// The rejected value, exactly as given.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for InvalidEnumerationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not a {} value", self.value, self.enumeration)
    }
}

impl std::error::Error for InvalidEnumerationValue {}

macro_rules! loin_enumeration {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $xml:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            /// Every schema lexical value, in declaration order.
            pub const VALUES: &'static [&'static str] = &[$($xml),+];

            /// The schema lexical value this variant is written as.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $xml),+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = InvalidEnumerationValue;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($xml => Ok(Self::$variant),)+
                    _ => Err(InvalidEnumerationValue::new(stringify!($name), value)),
                }
            }
        }
    };
}

loin_enumeration!(ShapeAssembly {
    NotRequired => "NotRequired",
    SingleObjectSingularShape => "SingleObjectSingularShape",
    SingleObjectMultipleShapes => "SingleObjectMultipleShapes",
    MultipleObjects => "MultipleObjects",
});
loin_enumeration!(ShapeRepresentation {
    NotRequired => "NotRequired",
    SingleBoundingPrimitive => "SingleBoundingPrimitive",
    OuterShellAsSingularShape => "OuterShellAsSingularShape",
    OuterShellAsSeparateShapes => "OuterShellAsSeparateShapes",
});
loin_enumeration!(InsideGeometry {
    NotRequired => "NotRequired",
    NoInsideGeometry => "NoInsideGeometry",
    InsideGeometryAsPartOfShape => "InsideGeometryAsPartOfShape",
    SeparateShapes => "SeparateShapes",
});
loin_enumeration!(Connections {
    NotRequired => "NotRequired",
    NoConnections => "NoConnections",
    ConnectionsAsPartOfShape => "ConnectionsAsPartOfShape",
    SeparateShapes => "SeparateShapes",
});
loin_enumeration!(Openings {
    NotRequired => "NotRequired",
    NoOpenings => "NoOpenings",
    OpeningsAsPartOfShape => "OpeningsAsPartOfShape",
    SeparateShapes => "SeparateShapes",
});
loin_enumeration!(OperatingAndClearanceZones {
    NotRequired => "NotRequired",
    NoZones => "NoZones",
    ZonesAsPartOfShape => "ZonesAsPartOfShape",
    SeparateShapes => "SeparateShapes",
});
loin_enumeration!(Features {
    NotRequired => "NotRequired",
    NoFeatures => "NoFeatures",
    FeaturesAsPartOfShape => "FeaturesAsPartOfShape",
    SeparateShapes => "SeparateShapes",
});
loin_enumeration!(Dimensionality {
    NotRequired => "NotRequired",
    ZeroD => "0D",
    OneD => "1D",
    TwoD => "2D",
    ThreeD => "3D",
});
loin_enumeration!(Appearance {
    NotRequired => "NotRequired",
    NoAppearanceInformation => "NoAppearanceInformation",
    SymbolicByMapping => "SymbolicByMapping",
    SingularMaterial => "SingularMaterial",
    MultipleMaterials => "MultipleMaterials",
    ConceptualAppearance => "ConceptualAppearance",
    RealisticAppearance => "RealisticAppearance",
});
loin_enumeration!(ParametricBehaviour {
    NotRequested => "NotRequested",
    Requested => "Requested",
});
loin_enumeration!(RelativeOrAbsolute {
    NotDefined => "NotDefined",
    Absolute => "Absolute",
    Relative => "Relative",
});

/// LOIN geometrical-information identity using the imported DT GUID attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometricalInformation {
    guid: Guid,
    placeholder: Option<bool>,
    detail: Option<Detail>,
    dimensionality: Option<Dimensionality>,
    appearance: Option<Appearance>,
    parametric_behaviour: Option<ParametricBehaviour>,
    location: Option<Location>,
}

impl GeometricalInformation {
    #[must_use]
    pub const fn new(guid: Guid) -> Self {
        Self {
            guid,
            placeholder: None,
            detail: None,
            dimensionality: None,
            appearance: None,
            parametric_behaviour: None,
            location: None,
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    #[must_use]
    pub const fn placeholder(&self) -> Option<bool> {
        self.placeholder
    }
    pub fn set_placeholder(&mut self, value: Option<bool>) {
        self.placeholder = value;
    }
    #[must_use]
    pub const fn detail(&self) -> Option<&Detail> {
        self.detail.as_ref()
    }
    pub fn set_detail(&mut self, value: Option<Detail>) {
        self.detail = value;
    }
    pub fn set_dimensionality(&mut self, value: Option<Dimensionality>) {
        self.dimensionality = value;
    }
    #[must_use]
    pub const fn dimensionality(&self) -> Option<Dimensionality> {
        self.dimensionality
    }
    pub fn set_appearance(&mut self, value: Option<Appearance>) {
        self.appearance = value;
    }
    #[must_use]
    pub const fn appearance(&self) -> Option<Appearance> {
        self.appearance
    }
    pub fn set_parametric_behaviour(&mut self, value: Option<ParametricBehaviour>) {
        self.parametric_behaviour = value;
    }
    #[must_use]
    pub const fn parametric_behaviour(&self) -> Option<ParametricBehaviour> {
        self.parametric_behaviour
    }
    pub fn set_location(&mut self, value: Option<Location>) {
        self.location = value;
    }
    #[must_use]
    pub const fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

/// LOIN `RequiredDocumentType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    guid: Guid,
    name: MultiLanguageText,
    format: DocumentFormat,
    reference_documents: Vec<Reference>,
    descriptions: Vec<MultiLanguageText>,
    document_type: Option<String>,
    form: Option<String>,
    content: Option<String>,
}

impl Document {
    #[must_use]
    pub const fn new(guid: Guid, name: MultiLanguageText, format: DocumentFormat) -> Self {
        Self {
            guid,
            name,
            format,
            reference_documents: Vec::new(),
            descriptions: Vec::new(),
            document_type: None,
            form: None,
            content: None,
        }
    }
    #[must_use]
    pub const fn guid(&self) -> &Guid {
        &self.guid
    }
    pub fn add_reference_document(&mut self, value: Reference) {
        self.reference_documents.push(value);
    }
    #[must_use]
    pub fn reference_documents(&self) -> &[Reference] {
        &self.reference_documents
    }
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
    #[must_use]
    pub const fn name(&self) -> &MultiLanguageText {
        &self.name
    }
    #[must_use]
    pub fn descriptions(&self) -> &[MultiLanguageText] {
        &self.descriptions
    }
    #[must_use]
    pub const fn format(&self) -> &DocumentFormat {
        &self.format
    }
    pub fn set_document_type(&mut self, value: Option<String>) {
        self.document_type = value;
    }
    pub fn set_form(&mut self, value: Option<String>) {
        self.form = value;
    }
    pub fn set_content(&mut self, value: Option<String>) {
        self.content = value;
    }
    #[must_use]
    pub fn document_type(&self) -> Option<&str> {
        self.document_type.as_deref()
    }
    #[must_use]
    pub fn form(&self) -> Option<&str> {
        self.form.as_deref()
    }
    #[must_use]
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
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
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }
    #[must_use]
    pub const fn unit(&self) -> &Unit {
        &self.unit
    }
    #[must_use]
    pub const fn definition(&self) -> &MultiLanguageText {
        &self.definition
    }
}

/// Optional `ShapeInfluenceType` branches.
///
/// Field order matches the XSD `xs:sequence`: a serializer walking these
/// fields in declaration order emits valid document order.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeInfluence {
    pub inside_geometry: Option<InsideGeometry>,
    pub connections: Option<Connections>,
    pub openings: Option<Openings>,
    pub operating_and_clearance_zones: Option<OperatingAndClearanceZones>,
    pub features: Option<Features>,
    pub threshold_dimension: Option<ThresholdDimension>,
}

impl ShapeInfluence {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inside_geometry: None,
            connections: None,
            openings: None,
            operating_and_clearance_zones: None,
            features: None,
            threshold_dimension: None,
        }
    }
}

impl Default for ShapeInfluence {
    fn default() -> Self {
        Self::new()
    }
}

/// LOIN geometry detail with an optional DT dictionary reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Detail {
    dictionary: Option<Reference>,
    shape_assembly: Option<ShapeAssembly>,
    shape_representation: Option<ShapeRepresentation>,
    shape_influence: Option<ShapeInfluence>,
}

impl Detail {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dictionary: None,
            shape_assembly: None,
            shape_representation: None,
            shape_influence: None,
        }
    }
    #[must_use]
    pub const fn dictionary(&self) -> Option<&Reference> {
        self.dictionary.as_ref()
    }
    pub fn set_dictionary(&mut self, value: Option<Reference>) {
        self.dictionary = value;
    }
    pub fn set_shape_assembly(&mut self, value: Option<ShapeAssembly>) {
        self.shape_assembly = value;
    }
    pub fn set_shape_representation(&mut self, value: Option<ShapeRepresentation>) {
        self.shape_representation = value;
    }
    pub fn set_shape_influence(&mut self, value: Option<ShapeInfluence>) {
        self.shape_influence = value;
    }
    #[must_use]
    pub const fn shape_assembly(&self) -> Option<ShapeAssembly> {
        self.shape_assembly
    }
    #[must_use]
    pub const fn shape_representation(&self) -> Option<ShapeRepresentation> {
        self.shape_representation
    }
    #[must_use]
    pub const fn shape_influence(&self) -> Option<&ShapeInfluence> {
        self.shape_influence.as_ref()
    }
}

impl Default for Detail {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry-backed datum type declared by the LOIN schema.
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
    pub fn add_description(&mut self, value: MultiLanguageText) {
        self.descriptions.push(value);
    }
    #[must_use]
    pub fn descriptions(&self) -> &[MultiLanguageText] {
        &self.descriptions
    }
    pub fn set_registry_reference(&mut self, value: Option<Reference>) {
        self.registry_reference = value;
    }
    #[must_use]
    pub const fn registry_reference(&self) -> Option<&Reference> {
        self.registry_reference.as_ref()
    }
}

/// Datum name and required registry-reference type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Datum {
    name: String,
    datum_type: DatumRegistryReference,
}

impl Datum {
    #[must_use]
    pub fn new(name: impl Into<String>, datum_type: DatumRegistryReference) -> Self {
        Self {
            name: name.into(),
            datum_type,
        }
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn datum_type(&self) -> &DatumRegistryReference {
        &self.datum_type
    }
}

loin_enumeration!(
    /// Declared coordinate-reference-system kind.
    CoordinateReferenceSystemKind {
    NotRequired => "NotRequired",
    ProjectedCrs => "ProjectedCRS",
    EngineeringCrs => "EngineeringCRS",
    GeographicCrs => "GeographicCRS",
});

/// Coordinate reference system with required kind and datum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateReferenceSystem {
    pub crs_type: CoordinateReferenceSystemKind,
    pub datum: Datum,
    pub vertical_datum: Option<Datum>,
}

impl CoordinateReferenceSystem {
    #[must_use]
    pub const fn new(crs_type: CoordinateReferenceSystemKind, datum: Datum) -> Self {
        Self {
            crs_type,
            datum,
            vertical_datum: None,
        }
    }
    #[must_use]
    pub const fn crs_type(&self) -> CoordinateReferenceSystemKind {
        self.crs_type
    }
    pub fn set_vertical_datum(&mut self, value: Option<Datum>) {
        self.vertical_datum = value;
    }
}

/// One required model-coordinate-system state in schema sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCoordinateSystem {
    pub is_projected: bool,
    pub first_coordinate: Decimal,
    pub second_coordinate: Decimal,
    pub height: Decimal,
    pub x_axis_abscissa: Option<Decimal>,
    pub x_axis_ordinate: Option<Decimal>,
    pub unit_scale: Option<Decimal>,
    pub horizontal_scale: Option<Decimal>,
}

impl ModelCoordinateSystem {
    #[must_use]
    pub const fn new(
        is_projected: bool,
        first_coordinate: Decimal,
        second_coordinate: Decimal,
        height: Decimal,
    ) -> Self {
        Self {
            is_projected,
            first_coordinate,
            second_coordinate,
            height,
            x_axis_abscissa: None,
            x_axis_ordinate: None,
            unit_scale: None,
            horizontal_scale: None,
        }
    }
}

/// Relative or absolute geometrical location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub relative_or_absolute: RelativeOrAbsolute,
    pub reference_object: Option<String>,
}

impl Location {
    #[must_use]
    pub const fn new(
        relative_or_absolute: RelativeOrAbsolute,
        reference_object: Option<String>,
    ) -> Self {
        Self {
            relative_or_absolute,
            reference_object,
        }
    }
}

/// Optional specification-level georeferencing state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeoReferencing {
    pub coordinate_reference_system: Option<CoordinateReferenceSystem>,
    model_coordinate_systems: Vec<ModelCoordinateSystem>,
}

impl GeoReferencing {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            coordinate_reference_system: None,
            model_coordinate_systems: Vec::new(),
        }
    }
    pub fn set_coordinate_reference_system(&mut self, value: Option<CoordinateReferenceSystem>) {
        self.coordinate_reference_system = value;
    }
    #[must_use]
    pub const fn coordinate_reference_system(&self) -> Option<&CoordinateReferenceSystem> {
        self.coordinate_reference_system.as_ref()
    }
    pub fn add_model_coordinate_system(&mut self, value: ModelCoordinateSystem) {
        self.model_coordinate_systems.push(value);
    }
    #[must_use]
    pub fn model_coordinate_systems(&self) -> &[ModelCoordinateSystem] {
        &self.model_coordinate_systems
    }
}
