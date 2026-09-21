//! Lossless, namespace-aware XML tree used by ISO 7817-3 documents.

use std::{borrow::Cow, collections::HashSet, error::Error, fmt, io::Cursor, sync::Arc};

use quick_xml::{
    events::{
        attributes::Attribute as QuickXmlAttribute, BytesCData, BytesDecl, BytesEnd, BytesPI,
        BytesStart, BytesText, Event,
    },
    name::QName,
    Writer,
};

use openbim_dt as dt;

/// Prefix bound to the LOIN namespace on authored roots.
const LOIN_PREFIX: &str = "loin";

/// The reserved namespace that `xmlns` declarations themselves belong to.
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

use crate::{
    parser::{parse_document, ParseError, ParseOptions},
    validation, NAMESPACE_2022, NAMESPACE_2024,
};

/// Known ISO 7817-3 namespace editions accepted by the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceVersion {
    Draft2022,
    Draft2024,
}

impl NamespaceVersion {
    #[must_use]
    pub const fn uri(self) -> &'static str {
        match self {
            Self::Draft2022 => NAMESPACE_2022,
            Self::Draft2024 => NAMESPACE_2024,
        }
    }

    pub(crate) fn from_uri(uri: &str) -> Option<Self> {
        match uri {
            NAMESPACE_2022 => Some(Self::Draft2022),
            NAMESPACE_2024 => Some(Self::Draft2024),
            _ => None,
        }
    }
}

/// Explicit namespace policy required by every write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputNamespace {
    Preserve,
    Version(NamespaceVersion),
}

/// Counts produced by a namespace migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationReport {
    source: NamespaceVersion,
    target: NamespaceVersion,
    changed_names: usize,
    changed_declarations: usize,
}

impl MigrationReport {
    #[must_use]
    pub const fn source(&self) -> NamespaceVersion {
        self.source
    }

    #[must_use]
    pub const fn target(&self) -> NamespaceVersion {
        self.target
    }

    #[must_use]
    pub const fn changed_names(&self) -> usize {
        self.changed_names
    }

    #[must_use]
    pub const fn changed_declarations(&self) -> usize {
        self.changed_declarations
    }
}

/// A migration would make two attributes share one expanded name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationError {
    element: String,
    local_name: String,
}

impl MigrationError {
    #[must_use]
    pub fn element(&self) -> &str {
        &self.element
    }

    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "namespace migration would duplicate expanded attribute {:?} on {}",
            self.local_name, self.element
        )
    }
}

impl Error for MigrationError {}

/// XML declaration values retained by the document model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDeclaration {
    pub version: String,
    pub encoding: Option<String>,
    pub standalone: Option<String>,
}

/// One XML attribute with both lexical and resolved namespace identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlAttribute {
    qname: String,
    prefix: Option<String>,
    local_name: String,
    namespace_uri: Option<Arc<str>>,
    value: String,
}

impl XmlAttribute {
    pub(crate) fn parsed(
        qname: String,
        prefix: Option<String>,
        local_name: String,
        namespace_uri: Option<Arc<str>>,
        value: String,
    ) -> Self {
        Self {
            qname,
            prefix,
            local_name,
            namespace_uri,
            value,
        }
    }

    /// Builds an unprefixed attribute, used by LOIN's own local attributes.
    #[must_use]
    pub fn new(local_name: impl Into<String>, value: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: local_name.clone(),
            prefix: None,
            local_name,
            namespace_uri: None,
            value: value.into(),
        }
    }

    /// Builds an `xmlns` / `xmlns:prefix` declaration.
    ///
    /// Namespace declarations live in the reserved xmlns namespace; they are
    /// attributes syntactically but bind prefixes rather than carry data.
    #[must_use]
    pub fn namespace_declaration(prefix: Option<&str>, uri: impl Into<String>) -> Self {
        match prefix {
            None => Self {
                qname: "xmlns".to_owned(),
                prefix: None,
                local_name: "xmlns".to_owned(),
                namespace_uri: Some(Arc::from(XMLNS_NAMESPACE)),
                value: uri.into(),
            },
            Some(prefix) => Self {
                qname: format!("xmlns:{prefix}"),
                prefix: Some("xmlns".to_owned()),
                local_name: prefix.to_owned(),
                namespace_uri: Some(Arc::from(XMLNS_NAMESPACE)),
                value: uri.into(),
            },
        }
    }

    /// Builds an attribute in the ISO 23387 namespace, such as `dt:GUID`.
    #[must_use]
    pub fn new_dt(local_name: impl Into<String>, value: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: format!("dt:{local_name}"),
            prefix: Some("dt".to_owned()),
            local_name,
            namespace_uri: Some(Arc::from(dt::NAMESPACE)),
            value: value.into(),
        }
    }

    #[must_use]
    pub fn qname(&self) -> &str {
        &self.qname
    }

    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    #[must_use]
    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace_uri.as_deref()
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// An XML element retaining names, namespace resolution, attributes, and order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement {
    qname: String,
    prefix: Option<String>,
    local_name: String,
    namespace_uri: Option<Arc<str>>,
    attributes: Vec<XmlAttribute>,
    nodes: Vec<XmlNode>,
    empty_style: bool,
}

impl XmlElement {
    pub(crate) fn parsed(
        qname: String,
        prefix: Option<String>,
        local_name: String,
        namespace_uri: Option<Arc<str>>,
        attributes: Vec<XmlAttribute>,
        empty_style: bool,
    ) -> Self {
        Self {
            qname,
            prefix,
            local_name,
            namespace_uri,
            attributes,
            nodes: Vec::new(),
            empty_style,
        }
    }

    /// Builds a schema-local LOIN element.
    ///
    /// ISO 7817-3 declares `elementFormDefault="unqualified"`, so every element
    /// below the root is unprefixed *and* in no namespace. Only the root
    /// element carries the LOIN namespace.
    #[must_use]
    pub fn new(local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: local_name.clone(),
            prefix: None,
            local_name,
            namespace_uri: None,
            attributes: Vec::new(),
            nodes: Vec::new(),
            empty_style: false,
        }
    }

    /// Builds the document root, which carries the LOIN namespace itself.
    ///
    /// Unlike [`Self::new`], the root is namespace-qualified: it is the element
    /// that declares the LOIN namespace for the whole document.
    #[must_use]
    pub fn new_root(local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: format!("{LOIN_PREFIX}:{local_name}"),
            prefix: Some(LOIN_PREFIX.to_owned()),
            local_name,
            namespace_uri: Some(Arc::from(NAMESPACE_2024)),
            attributes: Vec::new(),
            nodes: Vec::new(),
            empty_style: false,
        }
    }

    /// Builds an element in the ISO 23387 namespace, carrying the `dt` prefix.
    #[must_use]
    pub fn new_dt(local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            qname: format!("dt:{local_name}"),
            prefix: Some("dt".to_owned()),
            local_name,
            namespace_uri: Some(Arc::from(dt::NAMESPACE)),
            attributes: Vec::new(),
            nodes: Vec::new(),
            empty_style: false,
        }
    }

    /// Appends an attribute, keeping declaration order.
    #[must_use]
    pub fn with_attribute(mut self, attribute: XmlAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Appends a child element, preserving `xs:sequence` document order.
    #[must_use]
    pub fn with_child(mut self, child: Self) -> Self {
        self.nodes.push(XmlNode::Element(child));
        self
    }

    /// Appends character data, coalescing with any adjacent text node.
    #[must_use]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.push(XmlNode::Text(text.into()));
        self
    }

    /// Serializes as `<name/>` rather than `<name></name>`.
    ///
    /// Purely a serialization style; the two forms are semantically equal.
    #[must_use]
    pub const fn as_empty_element(mut self) -> Self {
        self.empty_style = true;
        self
    }

    /// Appends any node, for callers assembling mixed content directly.
    pub fn push_node(&mut self, node: XmlNode) {
        self.push(node);
    }

    /// Mutable access to child nodes, for editing a parsed document in place.
    pub fn nodes_mut(&mut self) -> &mut Vec<XmlNode> {
        &mut self.nodes
    }

    /// Mutable access to attributes, for editing a parsed document in place.
    pub fn attributes_mut(&mut self) -> &mut Vec<XmlAttribute> {
        &mut self.attributes
    }

    #[must_use]
    pub fn qname(&self) -> &str {
        &self.qname
    }

    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    #[must_use]
    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace_uri.as_deref()
    }

    #[must_use]
    pub fn attributes(&self) -> &[XmlAttribute] {
        &self.attributes
    }

    #[must_use]
    pub fn nodes(&self) -> &[XmlNode] {
        &self.nodes
    }

    /// Whether the source used an empty-element tag such as `<dt:Property/>`.
    #[must_use]
    pub const fn was_empty_element(&self) -> bool {
        self.empty_style
    }

    /// Direct child elements in document order.
    pub fn children(&self) -> impl Iterator<Item = &XmlElement> {
        self.nodes.iter().filter_map(XmlNode::as_element)
    }

    /// Finds an attribute by resolved namespace URI and local name.
    #[must_use]
    pub fn attribute_ns(&self, namespace_uri: Option<&str>, local_name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| {
                attribute.namespace_uri() == namespace_uri && attribute.local_name() == local_name
            })
            .map(XmlAttribute::value)
    }

    /// Direct semantic text and CDATA content, preserving order.
    #[must_use]
    pub fn direct_text(&self) -> String {
        let mut result = String::new();
        for node in &self.nodes {
            match node {
                XmlNode::Text(value) | XmlNode::CData(value) => result.push_str(value),
                _ => {}
            }
        }
        result
    }

    pub(crate) fn push(&mut self, node: XmlNode) {
        push_coalescing_text(&mut self.nodes, node);
    }
}

pub(crate) fn push_coalescing_text(nodes: &mut Vec<XmlNode>, node: XmlNode) {
    if let XmlNode::Text(value) = node {
        if let Some(XmlNode::Text(existing)) = nodes.last_mut() {
            existing.push_str(&value);
        } else {
            nodes.push(XmlNode::Text(value));
        }
    } else {
        nodes.push(node);
    }
}

/// XML node kinds retained by the document model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlNode {
    Element(XmlElement),
    Text(String),
    CData(String),
    Comment(String),
    /// Complete processing-instruction content: target plus optional data.
    ProcessingInstruction(String),
}

impl XmlNode {
    #[must_use]
    pub fn as_element(&self) -> Option<&XmlElement> {
        match self {
            Self::Element(element) => Some(element),
            _ => None,
        }
    }
}

/// A well-formed XML document with one root element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoinDocument {
    declaration: Option<XmlDeclaration>,
    prolog: Vec<XmlNode>,
    root: XmlElement,
    epilog: Vec<XmlNode>,
    observed_namespace: NamespaceVersion,
    current_namespace: NamespaceVersion,
}

impl LoinDocument {
    /// Parses with conservative production defaults.
    pub fn parse(xml: &str) -> Result<Self, ParseError> {
        Self::parse_with_options(xml, ParseOptions::default())
    }

    /// Parses with explicit resource limits.
    pub fn parse_with_options(xml: &str, options: ParseOptions) -> Result<Self, ParseError> {
        parse_document(xml, options)
    }

    /// Builds a document around an authored root element.
    ///
    /// Declares the LOIN namespace as the default and binds the `dt` prefix for
    /// ISO 23387 content, so serialized output reparses with the same resolved
    /// namespaces it was built with.
    pub(crate) fn authored(root: XmlElement) -> Self {
        // The LOIN namespace is bound to a PREFIX, never declared as the
        // default. ISO 7817-3 is elementFormDefault="unqualified", so a default
        // declaration would silently pull every schema-local child into the
        // namespace on reparse and make the document invalid.
        let root = root
            .with_attribute(XmlAttribute::namespace_declaration(
                Some(LOIN_PREFIX),
                NAMESPACE_2024,
            ))
            .with_attribute(XmlAttribute::namespace_declaration(
                Some("dt"),
                dt::NAMESPACE,
            ));
        Self {
            declaration: None,
            prolog: Vec::new(),
            root,
            epilog: Vec::new(),
            observed_namespace: NamespaceVersion::Draft2024,
            current_namespace: NamespaceVersion::Draft2024,
        }
    }

    pub(crate) fn parsed(
        declaration: Option<XmlDeclaration>,
        prolog: Vec<XmlNode>,
        root: XmlElement,
        epilog: Vec<XmlNode>,
        namespace: NamespaceVersion,
    ) -> Self {
        Self {
            declaration,
            prolog,
            root,
            epilog,
            observed_namespace: namespace,
            current_namespace: namespace,
        }
    }

    #[must_use]
    pub const fn declaration(&self) -> Option<&XmlDeclaration> {
        self.declaration.as_ref()
    }

    #[must_use]
    pub fn prolog(&self) -> &[XmlNode] {
        &self.prolog
    }

    #[must_use]
    pub const fn root(&self) -> &XmlElement {
        &self.root
    }

    /// Mutable access to the root element, for editing a parsed document.
    ///
    /// Edits bypass validation, so call [`Self::validate`] afterwards.
    pub fn root_mut(&mut self) -> &mut XmlElement {
        &mut self.root
    }

    #[must_use]
    pub fn epilog(&self) -> &[XmlNode] {
        &self.epilog
    }

    /// Namespace observed on the parsed root, retained across migrations.
    #[must_use]
    pub const fn observed_namespace(&self) -> NamespaceVersion {
        self.observed_namespace
    }

    /// Namespace currently represented by the root syntax tree.
    #[must_use]
    pub const fn current_namespace(&self) -> NamespaceVersion {
        self.current_namespace
    }

    /// Returns a migrated clone and a precise change count.
    pub fn migrated(
        &self,
        target: NamespaceVersion,
    ) -> Result<(Self, MigrationReport), MigrationError> {
        let source = self.current_namespace;
        if source != target {
            ensure_migration_safe(&self.root, source.uri(), target.uri())?;
        }
        let mut migrated = self.clone();
        let mut report = MigrationReport {
            source,
            target,
            changed_names: 0,
            changed_declarations: 0,
        };
        if source != target {
            migrate_element(&mut migrated.root, source.uri(), target.uri(), &mut report);
            migrated.current_namespace = target;
        }
        Ok((migrated, report))
    }

    /// Runs the crate's ISO 7817-3 clause-level structural validator.
    #[must_use]
    pub fn validate(&self) -> Vec<validation::Diagnostic> {
        validation::validate_document(self)
    }

    /// Serializes without dropping retained syntax. The namespace policy is mandatory.
    pub fn to_xml_string(&self, target: OutputNamespace) -> Result<String, WriteError> {
        match target {
            OutputNamespace::Preserve => self.write_xml(),
            OutputNamespace::Version(version) if version == self.current_namespace => {
                self.write_xml()
            }
            OutputNamespace::Version(version) => self.migrated(version)?.0.write_xml(),
        }
    }

    fn write_xml(&self) -> Result<String, WriteError> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        if let Some(declaration) = &self.declaration {
            writer.write_event(Event::Decl(BytesDecl::new(
                &declaration.version,
                declaration.encoding.as_deref(),
                declaration.standalone.as_deref(),
            )))?;
        }
        for node in &self.prolog {
            write_node(&mut writer, node)?;
        }
        write_element(&mut writer, &self.root)?;
        for node in &self.epilog {
            write_node(&mut writer, node)?;
        }
        String::from_utf8(writer.into_inner().into_inner()).map_err(WriteError::Utf8)
    }
}

fn ensure_migration_safe(
    element: &XmlElement,
    source: &str,
    target: &str,
) -> Result<(), MigrationError> {
    let mut expanded = HashSet::new();
    for attribute in &element.attributes {
        if attribute.namespace_uri.as_deref() == Some("http://www.w3.org/2000/xmlns/") {
            continue;
        }
        let namespace = if attribute.namespace_uri.as_deref() == Some(source) {
            Some(target)
        } else {
            attribute.namespace_uri.as_deref()
        };
        if !expanded.insert((namespace.map(str::to_owned), attribute.local_name.clone())) {
            return Err(MigrationError {
                element: element.qname.clone(),
                local_name: attribute.local_name.clone(),
            });
        }
    }
    for node in &element.nodes {
        if let XmlNode::Element(child) = node {
            ensure_migration_safe(child, source, target)?;
        }
    }
    Ok(())
}

fn migrate_element(
    element: &mut XmlElement,
    source: &str,
    target: &str,
    report: &mut MigrationReport,
) {
    if element.namespace_uri.as_deref() == Some(source) {
        element.namespace_uri = Some(Arc::from(target));
        report.changed_names += 1;
    }
    for attribute in &mut element.attributes {
        if attribute.namespace_uri.as_deref() == Some(source) {
            attribute.namespace_uri = Some(Arc::from(target));
            report.changed_names += 1;
        }
        if attribute.namespace_uri.as_deref() == Some("http://www.w3.org/2000/xmlns/")
            && attribute.value == source
        {
            attribute.value = target.to_owned();
            report.changed_declarations += 1;
        }
    }
    for node in &mut element.nodes {
        if let XmlNode::Element(child) = node {
            migrate_element(child, source, target, report);
        }
    }
}

fn write_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    element: &XmlElement,
) -> Result<(), WriteError> {
    let mut start = BytesStart::new(element.qname());
    for attribute in element.attributes() {
        start.push_attribute(QuickXmlAttribute {
            key: QName(attribute.qname().as_bytes()),
            value: Cow::Owned(escape_attribute_value(attribute.value())),
        });
    }
    if element.empty_style && element.nodes.is_empty() {
        writer.write_event(Event::Empty(start))?;
        return Ok(());
    }
    writer.write_event(Event::Start(start))?;
    for node in element.nodes() {
        write_node(writer, node)?;
    }
    writer.write_event(Event::End(BytesEnd::new(element.qname())))?;
    Ok(())
}

fn escape_attribute_value(value: &str) -> Vec<u8> {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '"' => output.push_str("&quot;"),
            '\t' => output.push_str("&#x9;"),
            '\n' => output.push_str("&#xA;"),
            '\r' => output.push_str("&#xD;"),
            _ => output.push(character),
        }
    }
    output.into_bytes()
}

fn escape_text_value(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\r' => output.push_str("&#13;"),
            _ => output.push(character),
        }
    }
    output
}

fn write_node(writer: &mut Writer<Cursor<Vec<u8>>>, node: &XmlNode) -> Result<(), WriteError> {
    match node {
        XmlNode::Element(element) => write_element(writer, element),
        XmlNode::Text(value) => writer
            .write_event(Event::Text(BytesText::from_escaped(escape_text_value(
                value,
            ))))
            .map_err(Into::into),
        XmlNode::CData(value) => writer
            .write_event(Event::CData(BytesCData::new(value)))
            .map_err(Into::into),
        XmlNode::Comment(value) => writer
            .write_event(Event::Comment(BytesText::from_escaped(value)))
            .map_err(Into::into),
        XmlNode::ProcessingInstruction(value) => writer
            .write_event(Event::PI(BytesPI::new(value)))
            .map_err(Into::into),
    }
}

/// XML serialization failure.
#[derive(Debug)]
pub enum WriteError {
    Migration(MigrationError),
    Xml(std::io::Error),
    Utf8(std::string::FromUtf8Error),
}

impl fmt::Display for WriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Migration(error) => error.fmt(formatter),
            Self::Xml(error) => write!(formatter, "could not write XML: {error}"),
            Self::Utf8(error) => write!(formatter, "XML writer produced invalid UTF-8: {error}"),
        }
    }
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Migration(error) => Some(error),
            Self::Xml(error) => Some(error),
            Self::Utf8(error) => Some(error),
        }
    }
}

impl From<MigrationError> for WriteError {
    fn from(value: MigrationError) -> Self {
        Self::Migration(value)
    }
}

impl From<std::io::Error> for WriteError {
    fn from(value: std::io::Error) -> Self {
        Self::Xml(value)
    }
}
