use openbim_loin::{
    DiagnosticCode, LoinDocument, NamespaceVersion, OutputNamespace, Severity, NAMESPACE_2024,
};

const DT: &str = "https://standards.iso.org/iso/23387/ed-2/en/";
const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";

fn prerequisites() -> &'static str {
    r#"<Prerequisites dt:GUID="20000000-0000-0000-0000-000000000000">
      <Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name language="en">Purpose</Name></Purpose>
      <InformationDeliveryMilestone dt:GUID="40000000-0000-0000-0000-000000000000">
        <Name language="en">Gate</Name>
      </InformationDeliveryMilestone>
      <ProvidingActor dt:GUID="50000000-0000-0000-0000-000000000000"><Role language="en">Author</Role></ProvidingActor>
      <ReceivingActor dt:GUID="60000000-0000-0000-0000-000000000000"><Role language="en">Reviewer</Role></ReceivingActor>
    </Prerequisites>"#
}

fn document(extra: &str) -> String {
    format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}" xmlns:dt="{DT}" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Specification name="Synthetic" dt:GUID="10000000-0000-0000-0000-000000000000">
    {}{}
  </Specification>
</l:LevelOfInformationNeed>"#,
        prerequisites(),
        extra
    )
}

fn errors(xml: &str) -> Vec<openbim_loin::Diagnostic> {
    LoinDocument::parse(xml)
        .unwrap()
        .validate()
        .into_iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Error)
        .collect()
}

#[test]
fn validates_georeferencing_clause_shape_and_scalar_lexemes() {
    let xml = document(
        r#"<GeoReferencing><CoordinateReferenceSystem><Type>ProjectedCRS</Type><Datum><Name>Datum</Name><Type><Name language="en">Registry</Name></Type></Datum></CoordinateReferenceSystem><ModelCoordinateSystem><IsProjected>true</IsProjected><FirstCoordinate>1.25</FirstCoordinate><SecondCoordinate>-2</SecondCoordinate><Height>3</Height><XAxisAbscissa>0</XAxisAbscissa><XAxisOrdinate>1</XAxisOrdinate><UnitScale>1</UnitScale><HorizontalScale>1</HorizontalScale></ModelCoordinateSystem></GeoReferencing>"#,
    );
    assert!(
        errors(&xml).is_empty(),
        "unexpected diagnostics: {:#?}",
        errors(&xml)
    );
    let invalid_decimal = xml.replace(
        "<FirstCoordinate>1.25</FirstCoordinate>",
        "<FirstCoordinate>1e2</FirstCoordinate>",
    );
    assert!(errors(&invalid_decimal)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDecimal));
    let invalid_boolean = xml.replace(
        "<IsProjected>true</IsProjected>",
        "<IsProjected>yes</IsProjected>",
    );
    assert!(errors(&invalid_boolean)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidBoolean));
}

#[test]
fn accepts_schema_valid_nilled_per_object_with_required_inherited_attributes() {
    let xml = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000"
      dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true" />"#,
    );
    assert!(
        errors(&xml).is_empty(),
        "unexpected diagnostics: {:#?}",
        errors(&xml)
    );
}

#[test]
fn rejects_nilled_content_and_still_requires_inherited_attributes() {
    let missing = document(r#"<SpecificationPerObjectType xsi:nil="true" />"#);
    let missing_codes: Vec<_> = errors(&missing).into_iter().map(|d| d.code()).collect();
    assert!(missing_codes.contains(&DiagnosticCode::MissingRequiredAttribute));

    let missing_date = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" xsi:nil="true" />"#,
    );
    assert!(errors(&missing_date)
        .iter()
        .any(|diagnostic| diagnostic.message().contains("dateOfCreation")));

    let content = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000"
      dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true"><ObjectType /></SpecificationPerObjectType>"#,
    );
    assert!(errors(&content)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::NilledContent));

    let whitespace = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000"
      dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true"> </SpecificationPerObjectType>"#,
    );
    assert!(errors(&whitespace)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::NilledContent));

    let empty_cdata = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true"><![CDATA[]]></SpecificationPerObjectType>"#,
    );
    assert!(!errors(&empty_cdata)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::NilledContent));

    let whitespace_cdata = empty_cdata.replace("<![CDATA[]]>", "<![CDATA[ ]]>");
    assert!(errors(&whitespace_cdata)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::NilledContent));
}

#[test]
fn parent_sensitive_grammar_rejects_known_elements_in_forbidden_locations() {
    let source = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}" xmlns:dt="{DT}">
  <Specification name="Synthetic" dt:GUID="10000000-0000-0000-0000-000000000000">{}</Specification>
  <ModelCoordinateSystem />
</l:LevelOfInformationNeed>"#,
        prerequisites()
    );
    assert!(errors(&source)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));
}

#[test]
fn purpose_repeating_choice_requires_content_and_accepts_reordered_repeated_branches() {
    let empty = document("").replace(
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name language="en">Purpose</Name></Purpose>"#,
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000"/>"#,
    );
    assert!(errors(&empty)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::MissingRequiredChild));

    let foreign_only = empty.replace(
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000"/>"#,
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><x:Extension xmlns:x="urn:example"/></Purpose>"#,
    );
    let foreign_errors = errors(&foreign_only);
    assert!(foreign_errors
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));
    assert!(foreign_errors
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::MissingRequiredChild));

    let source = document("").replace(
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000"><Name language="en">Purpose</Name></Purpose>"#,
        r#"<Purpose dt:GUID="30000000-0000-0000-0000-000000000000">
        <Region>DE</Region><Name language="en">Coordination</Name>
        <Definition language="en">First</Definition><Name language="de">Koordination</Name>
      </Purpose>"#,
    );
    assert!(
        errors(&source).is_empty(),
        "unexpected diagnostics: {:#?}",
        errors(&source)
    );
}

#[test]
fn alphanumerical_choice_accepts_empty_content_but_requires_identity() {
    let valid = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z">
      <dt:Name/><ObjectType /><AlphanumericalInformation dt:GUID="80000000-0000-0000-0000-000000000000" />
    </SpecificationPerObjectType>"#,
    );
    assert!(
        errors(&valid).is_empty(),
        "unexpected diagnostics: {:#?}",
        errors(&valid)
    );

    let invalid = valid.replace(
        "<AlphanumericalInformation dt:GUID=\"80000000-0000-0000-0000-000000000000\" />",
        "<AlphanumericalInformation />",
    );
    assert!(errors(&invalid)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::MissingRequiredAttribute));
}

#[test]
fn required_attributes_language_and_exact_double_lexemes_are_checked() {
    let missing_name = document("").replace(" name=\"Synthetic\"", "");
    assert!(errors(&missing_name)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::MissingRequiredAttribute));

    let language = document("").replace(
        r#"<Name language="en">Purpose</Name>"#,
        r#"<Language>not_a_language!</Language>"#,
    );
    assert!(errors(&language)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidLanguage));

    let double = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/><GeometricalInformation dt:GUID="80000000-0000-0000-0000-000000000000"><Detail><ShapeInfluence><ThresholdDimension><Threshold>inf</Threshold><Unit/><Definition language="en">Tolerance</Definition></ThresholdDimension></ShapeInfluence></Detail></GeometricalInformation></SpecificationPerObjectType>"#,
    );
    assert!(errors(&double)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDouble));
}

#[test]
fn xsd_whitespace_does_not_treat_unicode_separators_as_whitespace() {
    let scalar_source = document(
        r#"<GeoReferencing><CoordinateReferenceSystem><Type>ProjectedCRS</Type><Datum><Name>Datum</Name><Type><Name language="en">Registry</Name></Type></Datum></CoordinateReferenceSystem><ModelCoordinateSystem><IsProjected>true</IsProjected><FirstCoordinate>1.25</FirstCoordinate><SecondCoordinate>-2</SecondCoordinate><Height>3</Height><XAxisAbscissa>0</XAxisAbscissa><XAxisOrdinate>1</XAxisOrdinate><UnitScale>1</UnitScale><HorizontalScale>1</HorizontalScale></ModelCoordinateSystem></GeoReferencing>"#,
    );
    let double_source = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/><GeometricalInformation dt:GUID="80000000-0000-0000-0000-000000000000"><Detail><ShapeInfluence><ThresholdDimension><Threshold>1.5</Threshold><Unit/><Definition language="en">Tolerance</Definition></ThresholdDimension></ShapeInfluence></Detail></GeometricalInformation></SpecificationPerObjectType>"#,
    );

    let xsd_whitespace = "\t\n\r ";
    let valid_boolean = scalar_source.replace(
        "<IsProjected>true</IsProjected>",
        &format!("<IsProjected>{xsd_whitespace}true{xsd_whitespace}</IsProjected>"),
    );
    assert!(!errors(&valid_boolean)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidBoolean));

    let valid_decimal = scalar_source.replace(
        "<FirstCoordinate>1.25</FirstCoordinate>",
        &format!("<FirstCoordinate>{xsd_whitespace}1.25{xsd_whitespace}</FirstCoordinate>"),
    );
    assert!(!errors(&valid_decimal)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDecimal));

    let valid_double = double_source.replace(
        "<Threshold>1.5</Threshold>",
        &format!("<Threshold>{xsd_whitespace}1.5{xsd_whitespace}</Threshold>"),
    );
    assert!(!errors(&valid_double)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDouble));

    let valid_language = document("").replace(
        "language=\"en\"",
        &format!("language=\"{xsd_whitespace}en{xsd_whitespace}\""),
    );
    assert!(!errors(&valid_language)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidLanguage));

    let valid_nil = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true" />"#,
    )
    .replace(
        "xsi:nil=\"true\"",
        &format!("xsi:nil=\"{xsd_whitespace}true{xsd_whitespace}\""),
    );
    assert!(errors(&valid_nil).is_empty());

    let valid_complex_whitespace = document("").replace(
        "name=\"Synthetic\" dt:GUID=\"10000000-0000-0000-0000-000000000000\">",
        &format!(
            "name=\"Synthetic\" dt:GUID=\"10000000-0000-0000-0000-000000000000\">{xsd_whitespace}"
        ),
    );
    assert!(!errors(&valid_complex_whitespace)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedText));

    for separator in ['\u{00a0}', '\u{2003}'] {
        let invalid_boolean = scalar_source.replace(
            "<IsProjected>true</IsProjected>",
            &format!("<IsProjected>{separator}true{separator}</IsProjected>"),
        );
        assert!(errors(&invalid_boolean)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidBoolean));

        let invalid_decimal = scalar_source.replace(
            "<FirstCoordinate>1.25</FirstCoordinate>",
            &format!("<FirstCoordinate>{separator}1.25{separator}</FirstCoordinate>"),
        );
        assert!(errors(&invalid_decimal)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDecimal));

        let invalid_double = double_source.replace(
            "<Threshold>1.5</Threshold>",
            &format!("<Threshold>{separator}1.5{separator}</Threshold>"),
        );
        assert!(errors(&invalid_double)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidDouble));

        let invalid_language = document("").replace(
            "language=\"en\"",
            &format!("language=\"{separator}en{separator}\""),
        );
        assert!(errors(&invalid_language)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidLanguage));

        let invalid_nil = document(
            r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z" xsi:nil="true" />"#,
        )
        .replace(
            "xsi:nil=\"true\"",
            &format!("xsi:nil=\"{separator}true{separator}\""),
        );
        assert!(errors(&invalid_nil)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidBoolean));

        let unexpected_text = document("").replace(
            "name=\"Synthetic\" dt:GUID=\"10000000-0000-0000-0000-000000000000\">",
            &format!(
                "name=\"Synthetic\" dt:GUID=\"10000000-0000-0000-0000-000000000000\">{separator}"
            ),
        );
        assert!(errors(&unexpected_text)
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedText));
    }
}

#[test]
fn rejects_invalid_geometrical_enumeration() {
    let source = document(
        r#"<SpecificationPerObjectType
      dt:GUID="70000000-0000-0000-0000-000000000000"
      dateOfCreation="2026-08-26T00:00:00Z">
      <ObjectType />
      <GeometricalInformation dt:GUID="80000000-0000-0000-0000-000000000000">
        <Dimensionality>4D</Dimensionality>
      </GeometricalInformation>
    </SpecificationPerObjectType>"#,
    );
    assert!(errors(&source)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidEnumeration));
}

#[test]
fn imported_dt_complex_content_is_retained_without_false_complete_xsd_claims() {
    let source = document(
        r#"<SpecificationPerObjectType dt:GUID="70000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z">
      <dt:Name/><ObjectType vendorAttribute="retained"><VendorSpecific><Nested /></VendorSpecific></ObjectType>
      <AlphanumericalInformation dt:GUID="80000000-0000-0000-0000-000000000000"><Property><UnknownDtInternal /></Property></AlphanumericalInformation>
    </SpecificationPerObjectType>"#,
    );
    assert!(
        errors(&source).is_empty(),
        "imported internals are outside complete coverage: {:#?}",
        errors(&source)
    );
    let parsed = LoinDocument::parse(&source).unwrap();
    let encoded = parsed.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert_eq!(LoinDocument::parse(&encoded).unwrap().root(), parsed.root());
}

#[test]
fn inherited_concept_children_are_dt_qualified_ordered_and_retained() {
    let base = document(
        r#"<SpecificationPerObjectType dt:GUID="79000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><ObjectType/></SpecificationPerObjectType>"#,
    );
    assert!(errors(&base)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::MissingRequiredChild));
    let inherited = base.replace(
        "<ObjectType/>",
        "<dt:Name/><dt:Definition/><dt:ReferenceDocumentRef/><dt:Description/><dt:Example/><dt:SimilarToRef/><dt:LanguageOfCreator/><dt:CountryOfOrigin/><dt:VisualRepresentation/><dt:MajorVersion/><dt:MinorVersion/><dt:Status/><dt:ReplacedObjectsRef/><dt:DeprecationExplanation/><dt:DictionaryRef/><ObjectType/>",
    );
    assert!(errors(&inherited).is_empty());

    let invented = inherited.replace("<dt:Name/>", "<dt:Invented/><dt:Name/>");
    assert!(errors(&invented)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));

    let local_inherited = base.replace("<ObjectType/>", "<Name/><ObjectType/>");
    assert!(errors(&local_inherited)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));

    let wrong_order = base.replace("<ObjectType/>", "<ObjectType/><dt:Name/>");
    let wrong_order_errors = errors(&wrong_order);
    assert!(
        wrong_order_errors
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::ChildOutOfOrder),
        "{wrong_order_errors:#?}"
    );
}

#[test]
fn qualified_local_loin_elements_are_retained_but_diagnosed() {
    let source = format!(
        r#"<l:LevelOfInformationNeed xmlns:l="{NAMESPACE_2024}"><l:Specification /></l:LevelOfInformationNeed>"#
    );
    let document = LoinDocument::parse(&source).unwrap();
    assert!(document
        .validate()
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedNamespace));
    let encoded = document.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert_eq!(
        LoinDocument::parse(&encoded).unwrap().root(),
        document.root()
    );
}

#[test]
fn xsi_and_expanded_attribute_rules_match_the_declared_boundary() {
    let schema_location = document("").replace(
        &format!(r#"xmlns:xsi="{XSI}""#),
        &format!(r#"xmlns:xsi="{XSI}" xsi:schemaLocation="{NAMESPACE_2024} synthetic.xsd""#),
    );
    assert!(errors(&schema_location).is_empty());

    let bare_guid = document("").replace(
        "name=\"Synthetic\" dt:GUID=\"10000000-0000-0000-0000-000000000000\"",
        "name=\"Synthetic\" GUID=\"10000000-0000-0000-0000-000000000000\"",
    );
    let bare_codes: Vec<_> = errors(&bare_guid).into_iter().map(|d| d.code()).collect();
    assert!(bare_codes.contains(&DiagnosticCode::UnexpectedAttribute));
    assert!(bare_codes.contains(&DiagnosticCode::MissingRequiredAttribute));

    let xsi_type = document("").replace(
        "name=\"Synthetic\"",
        "name=\"Synthetic\" xsi:type=\"l:SpecificationType\"",
    );
    assert!(errors(&xsi_type)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnsupportedXsiType));

    let misplaced_nil =
        document("").replace("name=\"Synthetic\"", "name=\"Synthetic\" xsi:nil=\"false\"");
    assert!(errors(&misplaced_nil)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedAttribute));
}

#[test]
fn actor_sequence_and_cardinality_match_the_current_schema() {
    let valid = document("").replace(
        "</ProvidingActor>",
        r#"<Description language="en">Author</Description><EMailAddress>actor@example.invalid</EMailAddress></ProvidingActor>"#,
    );
    assert!(errors(&valid).is_empty());

    let invalid_email = valid.replace("actor@example.invalid", "not-an-email");
    assert!(errors(&invalid_email)
        .iter()
        .any(|diagnostic| diagnostic.message().contains("email restriction")));

    let organization = valid.replace(
        "</Description>",
        r#"</Description><Organization language="en">OpenBIM</Organization>"#,
    );
    assert!(errors(&organization)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));

    let repeated = valid.replace(
        "</Description>",
        r#"</Description><Description language="en">Duplicate</Description>"#,
    );
    assert!(errors(&repeated)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::TooManyChildren));
}

#[test]
fn documentation_and_alphanumerical_sequences_match_the_current_schema() {
    let valid = document(
        r#"<SpecificationPerObjectType dt:GUID="71000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/><AlphanumericalInformation dt:GUID="72000000-0000-0000-0000-000000000000"><GroupsOfProperties/></AlphanumericalInformation><Documentation dt:GUID="73000000-0000-0000-0000-000000000000"><Document dt:GUID="74000000-0000-0000-0000-000000000000" type="manual" form="digital" content="synthetic"><Name language="en">Manual</Name><ReferenceDocument/><Description language="en">Description</Description><Format><FormatName language="en">PDF</FormatName><FormatVersion language="en">2.0</FormatVersion></Format></Document></Documentation></SpecificationPerObjectType>"#,
    );
    assert!(errors(&valid).is_empty(), "{:?}", errors(&valid));
    let empty_groups = valid.clone();
    assert!(errors(&empty_groups).is_empty());
    let qualified_local_import = valid.replace("<ObjectType", "<dt:ObjectType");
    assert!(errors(&qualified_local_import)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedNamespace));
    let repeated_groups = valid.replace(
        "<GroupsOfProperties/>",
        "<GroupsOfProperties/><GroupsOfProperties/>",
    );
    assert!(errors(&repeated_groups)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::TooManyChildren));
    let old_name = valid
        .replace("<Document ", "<RequiredDocument ")
        .replace("</Document>", "</RequiredDocument>");
    assert!(errors(&old_name)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::UnknownElement));
}

#[test]
fn geometrical_sequence_represents_all_current_owned_branches() {
    let xml = document(
        r#"<SpecificationPerObjectType dt:GUID="75000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/><GeometricalInformation dt:GUID="76000000-0000-0000-0000-000000000000" placeholder="false"><Detail><Dictionary/><ShapeAssembly>MultipleObjects</ShapeAssembly><ShapeRepresentation>OuterShellAsSeparateShapes</ShapeRepresentation><ShapeInfluence><InsideGeometry>SeparateShapes</InsideGeometry><Connections>NoConnections</Connections><Openings>NoOpenings</Openings><OperatingAndClearanceZones>NoZones</OperatingAndClearanceZones><Features>NoFeatures</Features><ThresholdDimension><Threshold>0.01</Threshold><Unit/><Definition language="en">Tolerance</Definition></ThresholdDimension></ShapeInfluence></Detail><Dimensionality>3D</Dimensionality><Appearance>RealisticAppearance</Appearance><ParametricBehaviour>Requested</ParametricBehaviour><Location><RelativeOrAbsolute>Absolute</RelativeOrAbsolute><ReferenceObject>Site</ReferenceObject></Location></GeometricalInformation></SpecificationPerObjectType>"#,
    );
    assert!(errors(&xml).is_empty(), "{:?}", errors(&xml));
    let wrong_order = document(
        r#"<SpecificationPerObjectType dt:GUID="77000000-0000-0000-0000-000000000000" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/><GeometricalInformation dt:GUID="78000000-0000-0000-0000-000000000000"><Location><RelativeOrAbsolute>NotDefined</RelativeOrAbsolute></Location><Detail/></GeometricalInformation></SpecificationPerObjectType>"#,
    );
    assert!(errors(&wrong_order)
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::ChildOutOfOrder));
}

#[test]
fn repeated_parse_write_cycles_are_semantically_stable_after_migration() {
    let original = LoinDocument::parse(&document("")).unwrap();
    let (migrated, report) = original.migrated(NamespaceVersion::Draft2022).unwrap();
    assert_eq!(report.source(), NamespaceVersion::Draft2024);
    assert_eq!(report.target(), NamespaceVersion::Draft2022);
    assert!(migrated
        .validate()
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::CompatibilityProfile));

    let mut current = migrated;
    for _ in 0..4 {
        let encoded = current.to_xml_string(OutputNamespace::Preserve).unwrap();
        let reparsed = LoinDocument::parse(&encoded).unwrap();
        assert_eq!(reparsed.root(), current.root());
        current = reparsed;
    }
    assert_eq!(current.current_namespace(), NamespaceVersion::Draft2022);
    assert!(!current
        .to_xml_string(OutputNamespace::Preserve)
        .unwrap()
        .contains(NAMESPACE_2024));
}
