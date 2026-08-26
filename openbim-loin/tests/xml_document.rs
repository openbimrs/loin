use openbim_loin::{
    DiagnosticCode, LoinDocument, NamespaceVersion, OutputNamespace, ParseErrorKind, ParseOptions,
    Severity, NAMESPACE_2022, NAMESPACE_2024,
};

fn valid_xml(namespace: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<?review synthetic?>
<l:LevelOfInformationNeed xmlns:l="{namespace}"
    xmlns:dt="https://standards.iso.org/iso/23387/ed-2/en/"
    xmlns:ext="urn:example:loin-extension">
  <!-- retained comment -->
  <Specification name="Coordination" dt:GUID="10000000-0000-0000-0000-000000000000">
    <Prerequisites dt:GUID="20000000-0000-0000-0000-000000000000">
      <Purpose dt:GUID="30000000-0000-0000-0000-000000000000">
        <Name language="en">Coordinate design</Name>
        <ReferenceDocument dt:GUID="40000000-0000-0000-0000-000000000000" />
        <Description language="en"><![CDATA[Resolve <hard> interfaces]]></Description>
      </Purpose>
      <InformationDeliveryMilestone dt:GUID="50000000-0000-0000-0000-000000000000" Date="2026-08-26T00:00:00Z">
        <Name language="en">Design freeze</Name>
      </InformationDeliveryMilestone>
      <ProvidingActor dt:GUID="60000000-0000-0000-0000-000000000000">
        <Role language="en">Designer</Role>
        <EMailAddress>designer@example.invalid</EMailAddress>
      </ProvidingActor>
      <ReceivingActor dt:GUID="70000000-0000-0000-0000-000000000000">
        <Role language="en">Coordinator</Role>
      </ReceivingActor>
    </Prerequisites>
    <SpecificationPerObjectType dt:GUID="80000000-0000-0000-0000-000000000000">
      <dt:Name language="en">Wall requirements</dt:Name>
      <dt:Definition language="en">Synthetic wall requirements</dt:Definition>
      <ObjectType dt:GUID="90000000-0000-0000-0000-000000000000">
        <dt:Name language="en">Wall</dt:Name>
        <dt:Definition language="en">Synthetic wall type</dt:Definition>
      </ObjectType>
      <GeometricalInformation dt:GUID="a0000000-0000-0000-0000-000000000000">
        <Detail>
          <ShapeAssembly>SingleObjectSingularShape</ShapeAssembly>
          <ShapeRepresentation>OuterShellAsSingularShape</ShapeRepresentation>
        </Detail>
        <Dimensionality>3D</Dimensionality>
        <Appearance>SymbolicByMapping</Appearance>
        <ParametricBehaviour>Requested</ParametricBehaviour>
        <Location><RelativeOrAbsolute>Relative</RelativeOrAbsolute><ReferenceObject>site</ReferenceObject></Location>
      </GeometricalInformation>
      <ext:Future ext:flag="kept">future content</ext:Future>
    </SpecificationPerObjectType>
  </Specification>
</l:LevelOfInformationNeed>
"#
    )
}

fn schema_valid_xml(namespace: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<l:LevelOfInformationNeed xmlns:l="{namespace}" xmlns:dt="https://standards.iso.org/iso/23387/ed-2/en/">
  <Specification name="Synthetic validation control" dt:GUID="10000000-0000-0000-0000-000000000000">
    <Prerequisites dt:GUID="20000000-0000-0000-0000-000000000000">
      <Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name language="en">Purpose</Name></Purpose>
      <InformationDeliveryMilestone dt:GUID="50000000-0000-0000-0000-000000000000" Date="2026-08-26T00:00:00Z">
        <Name language="en">Review gate</Name>
      </InformationDeliveryMilestone>
      <ProvidingActor dt:GUID="60000000-0000-0000-0000-000000000000">
        <Role language="en">Author</Role>
      </ProvidingActor>
      <ReceivingActor dt:GUID="70000000-0000-0000-0000-000000000000">
        <Role language="en">Reviewer</Role>
      </ReceivingActor>
    </Prerequisites>
  </Specification>
</l:LevelOfInformationNeed>"#,
    )
}

#[test]
fn decodes_and_semantically_round_trips_complete_syntax() {
    let source = valid_xml(NAMESPACE_2024);
    let document = LoinDocument::parse(&source).expect("synthetic document parses");

    assert_eq!(document.observed_namespace(), NamespaceVersion::Draft2024);
    assert_eq!(document.current_namespace(), NamespaceVersion::Draft2024);
    assert_eq!(document.root().local_name(), "LevelOfInformationNeed");
    assert_eq!(document.root().namespace_uri(), Some(NAMESPACE_2024));

    let encoded = document
        .to_xml_string(OutputNamespace::Preserve)
        .expect("document encodes");
    assert!(encoded.contains("retained comment"));
    assert!(encoded.contains("<![CDATA[Resolve <hard> interfaces]]>"));
    assert!(encoded.contains("ext:Future"));
    assert!(encoded.contains("<?review synthetic?>"));

    let reparsed = LoinDocument::parse(&encoded).expect("encoded document reparses");
    assert_eq!(reparsed, document);
    assert_eq!(reparsed.root(), document.root());
    assert_eq!(reparsed.prolog(), document.prolog());
    assert_eq!(reparsed.epilog(), document.epilog());
}

#[test]
fn preserves_literal_comment_content_without_entity_escaping() {
    let source = format!(
        r#"<!--&<--><l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><!--&<--></l:LevelOfInformationNeed><!--&<-->"#
    );
    let document = LoinDocument::parse(&source).unwrap();
    let encoded = document.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert_eq!(encoded, source);
    assert_eq!(LoinDocument::parse(&encoded).unwrap(), document);
}

#[test]
fn preserves_represented_attribute_whitespace_and_text_carriage_return() {
    let source = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{}" xmlns:dt="{}" xmlns:ext="urn:ext" ext:data="a&#x9;b&#xA;c&#xD;d"><Specification name="x&#xD;y" dt:GUID="10000000-0000-0000-0000-000000000000"><Prerequisites dt:GUID="20000000-0000-0000-0000-000000000000"><Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name language="en">x&#xD;y</Name></Purpose><InformationDeliveryMilestone dt:GUID="40000000-0000-0000-0000-000000000000"><Name language="en">m</Name></InformationDeliveryMilestone><ProvidingActor dt:GUID="50000000-0000-0000-0000-000000000000"><Role language="en">p</Role></ProvidingActor><ReceivingActor dt:GUID="60000000-0000-0000-0000-000000000000"><Role language="en">r</Role></ReceivingActor></Prerequisites></Specification></l:LevelOfInformationNeed>"#,
        NAMESPACE_2024,
        openbim_loin::dt::NAMESPACE,
    );
    let parsed = LoinDocument::parse(&source).unwrap();
    let encoded = parsed.to_xml_string(OutputNamespace::Preserve).unwrap();
    let reparsed = LoinDocument::parse(&encoded).unwrap();
    assert_eq!(reparsed.root(), parsed.root());
    assert!(encoded.contains("&#x9;"));
    assert!(encoded.contains("&#xA;"));
    assert!(encoded.contains("&#xD;"));
}

#[test]
fn migration_rewrites_only_the_observed_loin_namespace() {
    let document = LoinDocument::parse(&valid_xml(NAMESPACE_2024)).unwrap();
    let (migrated, report) = document.migrated(NamespaceVersion::Draft2022).unwrap();

    assert_eq!(report.source(), NamespaceVersion::Draft2024);
    assert_eq!(report.target(), NamespaceVersion::Draft2022);
    assert_eq!(report.changed_names(), 1);
    assert_eq!(report.changed_declarations(), 1);
    assert_eq!(migrated.observed_namespace(), NamespaceVersion::Draft2024);
    assert_eq!(migrated.current_namespace(), NamespaceVersion::Draft2022);
    assert_eq!(migrated.root().namespace_uri(), Some(NAMESPACE_2022));

    let xml = migrated.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert!(xml.contains(NAMESPACE_2022));
    assert!(!xml.contains(NAMESPACE_2024));
    assert!(xml.contains(openbim_loin::dt::NAMESPACE));
    assert!(xml.contains("urn:example:loin-extension"));
    assert!(xml.contains("future content"));
    assert_eq!(LoinDocument::parse(&xml).unwrap().root(), migrated.root());
}

#[test]
fn migration_fails_closed_on_expanded_attribute_collisions() {
    let source = format!(
        r#"<old:LevelOfInformationNeed xmlns:old="{NAMESPACE_2022}" xmlns:new="{NAMESPACE_2024}" old:key="one" new:key="two" />"#
    );
    let document = LoinDocument::parse(&source).unwrap();
    let error = document.migrated(NamespaceVersion::Draft2024).unwrap_err();
    assert_eq!(error.element(), "old:LevelOfInformationNeed");
    assert_eq!(error.local_name(), "key");
    assert!(document
        .to_xml_string(OutputNamespace::Version(NamespaceVersion::Draft2024))
        .is_err());
}

#[test]
fn direct_encoding_to_a_target_namespace_is_explicit_and_non_mutating() {
    let document = LoinDocument::parse(&valid_xml(NAMESPACE_2022)).unwrap();
    let encoded = document
        .to_xml_string(OutputNamespace::Version(NamespaceVersion::Draft2024))
        .unwrap();
    assert!(encoded.contains(NAMESPACE_2024));
    assert!(!encoded.contains(NAMESPACE_2022));
    assert_eq!(document.current_namespace(), NamespaceVersion::Draft2022);
}

#[test]
fn migration_handles_default_namespaces_and_nested_prefix_shadowing() {
    let source = format!(
        r#"<LevelOfInformationNeed xmlns="{NAMESPACE_2022}" xmlns:l="{NAMESPACE_2022}" xmlns:ext="urn:extension"><l:Before/><ext:Box xmlns:l="{NAMESPACE_2024}"><l:AlreadyCurrent/></ext:Box><l:After/></LevelOfInformationNeed>"#
    );
    let document = LoinDocument::parse(&source).unwrap();
    let (migrated, report) = document.migrated(NamespaceVersion::Draft2024).unwrap();

    assert_eq!(report.changed_names(), 3);
    assert_eq!(report.changed_declarations(), 2);
    let children: Vec<_> = migrated.root().children().collect();
    assert_eq!(children[0].namespace_uri(), Some(NAMESPACE_2024));
    assert_eq!(children[1].namespace_uri(), Some("urn:extension"));
    assert_eq!(
        children[1].children().next().unwrap().namespace_uri(),
        Some(NAMESPACE_2024)
    );
    assert_eq!(children[2].namespace_uri(), Some(NAMESPACE_2024));

    let encoded = migrated.to_xml_string(OutputNamespace::Preserve).unwrap();
    let reparsed = LoinDocument::parse(&encoded).unwrap();
    assert_eq!(reparsed.root(), migrated.root());
}

#[test]
fn strict_decoder_rejects_unsafe_or_ambiguous_xml() {
    let dtd = format!(
        r#"<!DOCTYPE x [<!ENTITY e "boom">]><l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}">&e;</l:LevelOfInformationNeed>"#
    );
    assert_eq!(
        LoinDocument::parse(&dtd).unwrap_err().kind(),
        ParseErrorKind::DoctypeForbidden
    );

    let xml11 =
        format!(r#"<?xml version="1.1"?><l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}" />"#);
    assert_eq!(
        LoinDocument::parse(&xml11).unwrap_err().kind(),
        ParseErrorKind::UnsupportedXmlVersion
    );

    let undeclared = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><missing:Thing /></l:LevelOfInformationNeed>"#
    );
    assert_eq!(
        LoinDocument::parse(&undeclared).unwrap_err().kind(),
        ParseErrorKind::UndeclaredPrefix
    );

    let duplicate = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}" xmlns:a="urn:x" xmlns:b="urn:x" a:k="1" b:k="2" />"#
    );
    assert_eq!(
        LoinDocument::parse(&duplicate).unwrap_err().kind(),
        ParseErrorKind::DuplicateExpandedAttribute
    );

    let bad_binding = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}" xmlns:xml="urn:not-xml" />"#
    );
    assert_eq!(
        LoinDocument::parse(&bad_binding).unwrap_err().kind(),
        ParseErrorKind::MalformedXml
    );

    let bad_qname = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><a:b:c /></l:LevelOfInformationNeed>"#
    );
    assert!(matches!(
        LoinDocument::parse(&bad_qname).unwrap_err().kind(),
        ParseErrorKind::MalformedQName | ParseErrorKind::MalformedXml
    ));

    let unknown_entity = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}">&notDeclared;</l:LevelOfInformationNeed>"#
    );
    assert_eq!(
        LoinDocument::parse(&unknown_entity).unwrap_err().kind(),
        ParseErrorKind::UnknownEntity
    );

    let malformed = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><Thing></l:LevelOfInformationNeed>"#
    );
    assert_eq!(
        LoinDocument::parse(&malformed).unwrap_err().kind(),
        ParseErrorKind::MalformedXml
    );
}

#[test]
fn decoder_enforces_all_configured_budgets() {
    let source = valid_xml(NAMESPACE_2024);
    let options = ParseOptions {
        max_bytes: source.len() - 1,
        ..ParseOptions::default()
    };
    assert_eq!(
        LoinDocument::parse_with_options(&source, options)
            .unwrap_err()
            .kind(),
        ParseErrorKind::InputTooLarge
    );

    let options = ParseOptions {
        max_depth: 2,
        ..ParseOptions::default()
    };
    assert_eq!(
        LoinDocument::parse_with_options(&source, options)
            .unwrap_err()
            .kind(),
        ParseErrorKind::DepthLimit
    );

    let options = ParseOptions {
        max_nodes: 3,
        ..ParseOptions::default()
    };
    assert_eq!(
        LoinDocument::parse_with_options(&source, options)
            .unwrap_err()
            .kind(),
        ParseErrorKind::NodeLimit
    );

    let options = ParseOptions {
        max_attributes_per_element: 1,
        ..ParseOptions::default()
    };
    assert_eq!(
        LoinDocument::parse_with_options(&source, options)
            .unwrap_err()
            .kind(),
        ParseErrorKind::AttributeLimit
    );
}

#[test]
fn clause_validation_accepts_the_synthetic_iso_shape() {
    let document = LoinDocument::parse(&schema_valid_xml(NAMESPACE_2024)).unwrap();
    let diagnostics = document.validate();
    assert!(
        diagnostics
            .iter()
            .all(|item| item.severity() != Severity::Error),
        "unexpected diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn clause_validation_reports_order_cardinality_lexical_and_unknown_content() {
    let invalid = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{}" xmlns:dt="{}"><Specification name="Broken" dt:GUID="not-a-guid" unexpected="x"><Prerequisites dt:GUID="20000000-0000-0000-0000-000000000000"><Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name>Missing language</Name></Purpose><ProvidingActor dt:GUID="50000000-0000-0000-0000-000000000000"><Role language="en">p</Role></ProvidingActor><InformationDeliveryMilestone dt:GUID="40000000-0000-0000-0000-000000000000" Date="not-a-date"><Name language="en">m</Name></InformationDeliveryMilestone></Prerequisites><GeoReferencing /><GeoReferencing /><UnknownLoinElement /></Specification></l:LevelOfInformationNeed>"#,
        NAMESPACE_2024,
        openbim_loin::dt::NAMESPACE,
    );
    let document = LoinDocument::parse(&invalid).expect("invalid structure still parses");
    let diagnostics = document.validate();
    let has = |code| diagnostics.iter().any(|item| item.code() == code);

    assert!(has(DiagnosticCode::InvalidGuid));
    assert!(has(DiagnosticCode::UnexpectedAttribute));
    assert!(has(DiagnosticCode::MissingLanguage));
    assert!(has(DiagnosticCode::InvalidDateTime));
    assert!(has(DiagnosticCode::ChildOutOfOrder));
    assert!(has(DiagnosticCode::MissingRequiredChild));
    assert!(has(DiagnosticCode::TooManyChildren));
    assert!(has(DiagnosticCode::UnknownElement));
    assert!(diagnostics.iter().all(|item| !item.path().is_empty()));
}

#[test]
fn parsing_and_validation_are_separate_so_invalid_unknown_content_round_trips() {
    let source = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><Foreign /></l:LevelOfInformationNeed>"#
    );
    let document = LoinDocument::parse(&source).unwrap();
    assert!(document
        .validate()
        .iter()
        .any(|item| item.code() == DiagnosticCode::UnknownElement));
    let encoded = document.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert_eq!(
        LoinDocument::parse(&encoded).unwrap().root(),
        document.root()
    );
}
