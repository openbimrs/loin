//! Bounded XML parsing with explicit namespace resolution.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::Arc,
};

use quick_xml::{escape::unescape, events::BytesStart, events::Event, Reader, XmlVersion};
use roxmltree::{Document as StrictDocument, Error as StrictError, ParsingOptions};

use crate::document::{
    push_coalescing_text, LoinDocument, XmlAttribute, XmlDeclaration, XmlElement, XmlNode,
};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

type NamespaceScope = HashMap<String, Arc<str>>;
type NamespaceUndo = Vec<(String, Option<Arc<str>>)>;

/// Resource limits applied before or during XML parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseOptions {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_attributes_per_element: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_depth: 128,
            max_nodes: 1_000_000,
            max_attributes_per_element: 1_024,
        }
    }
}

/// Stable category for a parse failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    InputTooLarge,
    DepthLimit,
    NodeLimit,
    AttributeLimit,
    DoctypeForbidden,
    UnsupportedXmlVersion,
    DuplicateExpandedAttribute,
    UnknownEntity,
    UndeclaredPrefix,
    MalformedQName,
    MissingRoot,
    MultipleRoots,
    UnexpectedRoot,
    UnsupportedNamespace,
    MalformedXml,
    InvalidEncoding,
}

/// XML parse failure with a stable category and byte position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    position: u64,
    message: String,
}

impl ParseError {
    fn new(kind: ParseErrorKind, position: u64, message: impl Into<String>) -> Self {
        Self {
            kind,
            position,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "XML parse error {:?} at byte {}: {}",
            self.kind, self.position, self.message
        )
    }
}

impl Error for ParseError {}

pub(crate) fn parse_document(xml: &str, options: ParseOptions) -> Result<LoinDocument, ParseError> {
    if xml.len() > options.max_bytes {
        return Err(ParseError::new(
            ParseErrorKind::InputTooLarge,
            0,
            format!("{} bytes exceeds limit {}", xml.len(), options.max_bytes),
        ));
    }

    strict_well_formedness_check(xml, options)?;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_end_names = true;
    reader.config_mut().expand_empty_elements = false;

    let mut declaration = None;
    let mut prolog = Vec::new();
    let mut epilog = Vec::new();
    let mut root = None;
    let mut elements = Vec::<XmlElement>::new();
    let mut scope = base_scope();
    let mut scope_undos = Vec::<NamespaceUndo>::new();
    let mut node_count = 0usize;
    let mut xml_version = XmlVersion::Implicit1_0;

    loop {
        let position = reader.buffer_position();
        let event = reader.read_event().map_err(|error| {
            ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
        })?;
        match event {
            Event::Decl(value) => {
                if declaration.is_some() || root.is_some() || !elements.is_empty() {
                    return Err(ParseError::new(
                        ParseErrorKind::MalformedXml,
                        position,
                        "XML declaration is not first",
                    ));
                }
                let parsed = parse_declaration(&reader, &value, position)?;
                xml_version = match parsed.version.as_str() {
                    "1.0" => XmlVersion::Explicit1_0,
                    "1.1" => {
                        return Err(ParseError::new(
                            ParseErrorKind::UnsupportedXmlVersion,
                            position,
                            "XML 1.1 is not supported; convert the document to XML 1.0 first",
                        ));
                    }
                    other => {
                        return Err(ParseError::new(
                            ParseErrorKind::UnsupportedXmlVersion,
                            position,
                            format!("unsupported XML version `{other}`"),
                        ));
                    }
                };
                declaration = Some(parsed);
            }
            Event::Start(value) => {
                increment_nodes(&mut node_count, options, position)?;
                if elements.len() + 1 > options.max_depth {
                    return Err(ParseError::new(
                        ParseErrorKind::DepthLimit,
                        position,
                        format!("depth exceeds limit {}", options.max_depth),
                    ));
                }
                if elements.is_empty() && root.is_some() {
                    return Err(ParseError::new(
                        ParseErrorKind::MultipleRoots,
                        position,
                        "document contains more than one root element",
                    ));
                }
                let (element, undo) = parse_element(
                    &reader,
                    &value,
                    false,
                    &mut scope,
                    options,
                    position,
                    xml_version,
                )?;
                elements.push(element);
                scope_undos.push(undo);
            }
            Event::Empty(value) => {
                increment_nodes(&mut node_count, options, position)?;
                if elements.len() + 1 > options.max_depth {
                    return Err(ParseError::new(
                        ParseErrorKind::DepthLimit,
                        position,
                        format!("depth exceeds limit {}", options.max_depth),
                    ));
                }
                if elements.is_empty() && root.is_some() {
                    return Err(ParseError::new(
                        ParseErrorKind::MultipleRoots,
                        position,
                        "document contains more than one root element",
                    ));
                }
                let (element, undo) = parse_element(
                    &reader,
                    &value,
                    true,
                    &mut scope,
                    options,
                    position,
                    xml_version,
                )?;
                append_element(element, &mut elements, &mut root)?;
                restore_scope(&mut scope, undo);
            }
            Event::End(_) => {
                let element = elements.pop().ok_or_else(|| {
                    ParseError::new(
                        ParseErrorKind::MalformedXml,
                        position,
                        "closing element has no matching start",
                    )
                })?;
                let undo = scope_undos.pop().ok_or_else(|| {
                    ParseError::new(
                        ParseErrorKind::MalformedXml,
                        position,
                        "namespace scope underflow",
                    )
                })?;
                restore_scope(&mut scope, undo);
                append_element(element, &mut elements, &mut root)?;
            }
            Event::Text(value) => {
                increment_nodes(&mut node_count, options, position)?;
                let decoded = value.xml_content(xml_version).map_err(|error| {
                    ParseError::new(ParseErrorKind::InvalidEncoding, position, error.to_string())
                })?;
                let text = unescape(&decoded).map_err(|error| {
                    ParseError::new(ParseErrorKind::UnknownEntity, position, error.to_string())
                })?;
                append_node(
                    XmlNode::Text(text.into_owned()),
                    &mut elements,
                    &mut prolog,
                    &mut epilog,
                    root.is_some(),
                );
            }
            Event::CData(value) => {
                increment_nodes(&mut node_count, options, position)?;
                let text = value.xml_content(xml_version).map_err(|error| {
                    ParseError::new(ParseErrorKind::InvalidEncoding, position, error.to_string())
                })?;
                append_node(
                    XmlNode::CData(text.into_owned()),
                    &mut elements,
                    &mut prolog,
                    &mut epilog,
                    root.is_some(),
                );
            }
            Event::Comment(value) => {
                increment_nodes(&mut node_count, options, position)?;
                let text = decode(&reader, value.as_ref(), position)?;
                append_node(
                    XmlNode::Comment(text),
                    &mut elements,
                    &mut prolog,
                    &mut epilog,
                    root.is_some(),
                );
            }
            Event::PI(value) => {
                increment_nodes(&mut node_count, options, position)?;
                let text = decode(&reader, value.as_ref(), position)?;
                append_node(
                    XmlNode::ProcessingInstruction(text),
                    &mut elements,
                    &mut prolog,
                    &mut epilog,
                    root.is_some(),
                );
            }
            Event::GeneralRef(value) => {
                increment_nodes(&mut node_count, options, position)?;
                let reference = decode(&reader, value.as_ref(), position)?;
                let resolved = resolve_reference(&reference).ok_or_else(|| {
                    ParseError::new(
                        ParseErrorKind::UnknownEntity,
                        position,
                        format!("entity &{reference}; is not predefined or numeric"),
                    )
                })?;
                append_node(
                    XmlNode::Text(resolved),
                    &mut elements,
                    &mut prolog,
                    &mut epilog,
                    root.is_some(),
                );
            }
            Event::DocType(_) => {
                return Err(ParseError::new(
                    ParseErrorKind::DoctypeForbidden,
                    position,
                    "DOCTYPE declarations are disabled",
                ));
            }
            Event::Eof => break,
        }
    }

    if !elements.is_empty() {
        return Err(ParseError::new(
            ParseErrorKind::MalformedXml,
            reader.buffer_position(),
            "document ended inside an element",
        ));
    }
    let root = root.ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::MissingRoot,
            reader.buffer_position(),
            "document has no root element",
        )
    })?;
    if root.local_name() != "LevelOfInformationNeed" {
        return Err(ParseError::new(
            ParseErrorKind::UnexpectedRoot,
            0,
            format!(
                "expected LevelOfInformationNeed root, found {}",
                root.qname()
            ),
        ));
    }
    let namespace = root
        .namespace_uri()
        .and_then(crate::document::NamespaceVersion::from_uri)
        .ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnsupportedNamespace,
                0,
                format!("unsupported LOIN root namespace {:?}", root.namespace_uri()),
            )
        })?;
    Ok(LoinDocument::parsed(
        declaration,
        prolog,
        root,
        epilog,
        namespace,
    ))
}

fn strict_well_formedness_check(xml: &str, options: ParseOptions) -> Result<(), ParseError> {
    let strict_options = ParsingOptions {
        allow_dtd: false,
        nodes_limit: u32::try_from(options.max_nodes).unwrap_or(u32::MAX),
        entity_resolver: None,
    };
    StrictDocument::parse_with_options(xml, strict_options)
        .map(|_| ())
        .map_err(|error| {
            let kind = match error {
                StrictError::DtdDetected => ParseErrorKind::DoctypeForbidden,
                StrictError::NodesLimitReached => ParseErrorKind::NodeLimit,
                StrictError::UnknownNamespace(_, _) => ParseErrorKind::UndeclaredPrefix,
                StrictError::UnknownEntityReference(_, _)
                | StrictError::MalformedEntityReference(_) => ParseErrorKind::UnknownEntity,
                StrictError::DuplicatedAttribute(_, _) => {
                    ParseErrorKind::DuplicateExpandedAttribute
                }
                StrictError::InvalidName(_) => ParseErrorKind::MalformedQName,
                _ => ParseErrorKind::MalformedXml,
            };
            ParseError::new(kind, 0, error.to_string())
        })
}

fn base_scope() -> NamespaceScope {
    HashMap::from([("xml".to_owned(), Arc::<str>::from(XML_NAMESPACE))])
}

fn restore_scope(scope: &mut NamespaceScope, undo: NamespaceUndo) {
    for (prefix, previous) in undo.into_iter().rev() {
        if let Some(uri) = previous {
            scope.insert(prefix, uri);
        } else {
            scope.remove(&prefix);
        }
    }
}

fn parse_declaration(
    reader: &Reader<&[u8]>,
    value: &quick_xml::events::BytesDecl<'_>,
    position: u64,
) -> Result<XmlDeclaration, ParseError> {
    let version = value.version().map_err(|error| {
        ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
    })?;
    let encoding = value.encoding().transpose().map_err(|error| {
        ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
    })?;
    let standalone = value.standalone().transpose().map_err(|error| {
        ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
    })?;
    let version = decode(reader, version.as_ref(), position)?;
    let encoding = encoding
        .map(|raw| decode(reader, raw.as_ref(), position))
        .transpose()?;
    let standalone = standalone
        .map(|raw| decode(reader, raw.as_ref(), position))
        .transpose()?;

    if let Some(value) = &encoding {
        if !value.eq_ignore_ascii_case("UTF-8") && !value.eq_ignore_ascii_case("UTF8") {
            return Err(ParseError::new(
                ParseErrorKind::InvalidEncoding,
                position,
                format!("input is a UTF-8 Rust string but declares encoding `{value}`"),
            ));
        }
    }
    if let Some(value) = &standalone {
        if value != "yes" && value != "no" {
            return Err(ParseError::new(
                ParseErrorKind::MalformedXml,
                position,
                format!("standalone must be `yes` or `no`, not `{value}`"),
            ));
        }
    }

    Ok(XmlDeclaration {
        version,
        encoding,
        standalone,
    })
}

fn parse_element(
    reader: &Reader<&[u8]>,
    start: &BytesStart<'_>,
    empty_style: bool,
    scope: &mut NamespaceScope,
    options: ParseOptions,
    position: u64,
    xml_version: XmlVersion,
) -> Result<(XmlElement, NamespaceUndo), ParseError> {
    let qname = decode(reader, start.name().as_ref(), position)?;
    let (prefix, local_name) = split_qname(&qname, position)?;
    let raw_attributes: Vec<(String, Option<String>, String, String)> = start
        .attributes()
        .with_checks(true)
        .map(|attribute| {
            let attribute = attribute.map_err(|error| {
                ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
            })?;
            let qname = decode(reader, attribute.key.as_ref(), position)?;
            let (prefix, local_name) = split_qname(&qname, position)?;
            let value = attribute
                .decoded_and_normalized_value(xml_version, reader.decoder())
                .map_err(|error| {
                    ParseError::new(ParseErrorKind::MalformedXml, position, error.to_string())
                })?
                .into_owned();
            Ok((qname, prefix, local_name, value))
        })
        .collect::<Result<_, ParseError>>()?;

    if raw_attributes.len() > options.max_attributes_per_element {
        return Err(ParseError::new(
            ParseErrorKind::AttributeLimit,
            position,
            format!(
                "{} attributes exceeds limit {}",
                raw_attributes.len(),
                options.max_attributes_per_element
            ),
        ));
    }

    let mut undo = NamespaceUndo::new();
    for (qname, prefix, local_name, value) in &raw_attributes {
        if qname == "xmlns" {
            validate_namespace_binding(None, value, position)?;
            let previous = if value.is_empty() {
                scope.remove("")
            } else {
                scope.insert(String::new(), Arc::<str>::from(value.as_str()))
            };
            undo.push((String::new(), previous));
        } else if prefix.as_deref() == Some("xmlns") {
            validate_namespace_binding(Some(local_name.as_str()), value, position)?;
            let previous = scope.insert(local_name.clone(), Arc::<str>::from(value.as_str()));
            undo.push((local_name.clone(), previous));
        }
    }
    if let Some(prefix) = &prefix {
        if !scope.contains_key(prefix) {
            return Err(ParseError::new(
                ParseErrorKind::UndeclaredPrefix,
                position,
                format!("element prefix `{prefix}` is not declared"),
            ));
        }
    }
    for (_, prefix, _, _) in &raw_attributes {
        if let Some(prefix) = prefix {
            if prefix != "xmlns" && !scope.contains_key(prefix) {
                return Err(ParseError::new(
                    ParseErrorKind::UndeclaredPrefix,
                    position,
                    format!("attribute prefix `{prefix}` is not declared"),
                ));
            }
        }
    }
    let namespace_uri = scope.get(prefix.as_deref().unwrap_or_default()).cloned();
    let attributes: Vec<XmlAttribute> = raw_attributes
        .into_iter()
        .map(|(qname, prefix, local_name, value)| {
            let namespace_uri = if qname == "xmlns" || prefix.as_deref() == Some("xmlns") {
                Some(Arc::<str>::from(XMLNS_NAMESPACE))
            } else {
                prefix
                    .as_ref()
                    .and_then(|prefix| scope.get(prefix).cloned())
            };
            XmlAttribute::parsed(qname, prefix, local_name, namespace_uri, value)
        })
        .collect();
    let mut expanded_names = HashSet::new();
    for attribute in &attributes {
        if attribute.namespace_uri() == Some(XMLNS_NAMESPACE) {
            continue;
        }
        let expanded = (
            attribute.namespace_uri().map(str::to_owned),
            attribute.local_name().to_owned(),
        );
        if !expanded_names.insert(expanded) {
            return Err(ParseError::new(
                ParseErrorKind::DuplicateExpandedAttribute,
                position,
                format!(
                    "attribute {} duplicates an expanded attribute name",
                    attribute.qname()
                ),
            ));
        }
    }
    Ok((
        XmlElement::parsed(
            qname,
            prefix,
            local_name,
            namespace_uri,
            attributes,
            empty_style,
        ),
        undo,
    ))
}

fn split_qname(qname: &str, position: u64) -> Result<(Option<String>, String), ParseError> {
    match qname.split_once(':') {
        None if !qname.is_empty() => Ok((None, qname.to_owned())),
        Some((prefix, local))
            if !prefix.is_empty() && !local.is_empty() && !local.contains(':') =>
        {
            Ok((Some(prefix.to_owned()), local.to_owned()))
        }
        _ => Err(ParseError::new(
            ParseErrorKind::MalformedQName,
            position,
            format!("`{qname}` is not a namespace-qualified XML name"),
        )),
    }
}

fn validate_namespace_binding(
    prefix: Option<&str>,
    uri: &str,
    position: u64,
) -> Result<(), ParseError> {
    let valid = match prefix {
        Some("xmlns") => false,
        Some("xml") => uri == XML_NAMESPACE,
        Some(_) => !uri.is_empty() && uri != XML_NAMESPACE && uri != XMLNS_NAMESPACE,
        None => uri != XML_NAMESPACE && uri != XMLNS_NAMESPACE,
    };
    if valid {
        Ok(())
    } else {
        Err(ParseError::new(
            ParseErrorKind::MalformedXml,
            position,
            format!("invalid reserved namespace binding for prefix {prefix:?}"),
        ))
    }
}

fn append_element(
    element: XmlElement,
    parents: &mut [XmlElement],
    root: &mut Option<XmlElement>,
) -> Result<(), ParseError> {
    if let Some(parent) = parents.last_mut() {
        parent.push(XmlNode::Element(element));
        return Ok(());
    }
    if root.replace(element).is_some() {
        return Err(ParseError::new(
            ParseErrorKind::MultipleRoots,
            0,
            "document contains more than one root element",
        ));
    }
    Ok(())
}

fn append_node(
    node: XmlNode,
    parents: &mut [XmlElement],
    prolog: &mut Vec<XmlNode>,
    epilog: &mut Vec<XmlNode>,
    root_exists: bool,
) {
    if let Some(parent) = parents.last_mut() {
        parent.push(node);
    } else if root_exists {
        push_coalescing_text(epilog, node);
    } else {
        push_coalescing_text(prolog, node);
    }
}

fn increment_nodes(
    count: &mut usize,
    options: ParseOptions,
    position: u64,
) -> Result<(), ParseError> {
    *count += 1;
    if *count > options.max_nodes {
        return Err(ParseError::new(
            ParseErrorKind::NodeLimit,
            position,
            format!("node count exceeds limit {}", options.max_nodes),
        ));
    }
    Ok(())
}

fn decode(reader: &Reader<&[u8]>, bytes: &[u8], position: u64) -> Result<String, ParseError> {
    reader
        .decoder()
        .decode(bytes)
        .map(|value| value.into_owned())
        .map_err(|error| {
            ParseError::new(ParseErrorKind::InvalidEncoding, position, error.to_string())
        })
}

fn resolve_reference(reference: &str) -> Option<String> {
    let character = match reference {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "apos" => '\'',
        "quot" => '"',
        value if value.starts_with("#x") => {
            char::from_u32(u32::from_str_radix(&value[2..], 16).ok()?)?
        }
        value if value.starts_with('#') => char::from_u32(value[1..].parse().ok()?)?,
        _ => return None,
    };
    Some(character.to_string())
}
