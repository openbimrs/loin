//! Grammar facts taken from the official ISO 7817-3 Annex B XSD.
//!
//! The XSD and ISO's Annex C example documents are ISO copyright and are not
//! redistributable, so they live in gitignored
//! `references/specs/` and cannot be committed. These fixtures reproduce the
//! structures those examples exercise, so the grammar stays pinned in CI.
//!
//! Verified against the official schema, sha256
//! b715df44e5aef5541f31fad4d98522fc831a27bc4b12a68842c3666cbfd634de.

use openbim_loin::{LoinDocument, Severity, ShapeInfluence};

const DT: &str = "https://standards.iso.org/iso/23387/ed-2/en/";
const LOIN: &str = "https://iso.org/2024/LOIN";
const G1: &str = "10000000-0000-0000-0000-000000000000";
const G2: &str = "20000000-0000-0000-0000-000000000000";
const G3: &str = "30000000-0000-0000-0000-000000000000";
const G4: &str = "40000000-0000-0000-0000-000000000000";
const G5: &str = "50000000-0000-0000-0000-000000000000";
const G6: &str = "60000000-0000-0000-0000-000000000000";

fn document(specification_body: &str) -> String {
    format!(
        concat!(
            r#"<loin:LevelOfInformationNeed xmlns:loin="{LOIN}" xmlns:dt="{DT}">"#,
            r#"<Specification name="S" dt:GUID="{G1}">{specification_body}</Specification>"#,
            r#"</loin:LevelOfInformationNeed>"#,
        ),
        LOIN = LOIN,
        DT = DT,
        G1 = G1,
        specification_body = specification_body
    )
}

fn prerequisites() -> String {
    format!(
        concat!(
            r#"<Prerequisites dt:GUID="{G2}">"#,
            r#"<Purpose dt:GUID="{G3}"><Name language="en">P</Name></Purpose>"#,
            r#"<InformationDeliveryMilestone dt:GUID="{G4}">"#,
            r#"<Name language="en">M</Name></InformationDeliveryMilestone>"#,
            r#"<ProvidingActor dt:GUID="{G5}"><Role language="en">A</Role></ProvidingActor>"#,
            r#"<ReceivingActor dt:GUID="{G6}"><Role language="en">R</Role></ReceivingActor>"#,
            r#"</Prerequisites>"#,
        ),
        G2 = G2,
        G3 = G3,
        G4 = G4,
        G5 = G5,
        G6 = G6
    )
}

fn errors(xml: &str) -> Vec<openbim_loin::Diagnostic> {
    LoinDocument::parse(xml)
        .expect("parses")
        .validate()
        .into_iter()
        .filter(|d| d.severity() == Severity::Error)
        .collect()
}

/// Issue #5: the XSD choice is maxOccurs unbounded, so every branch may
/// repeat. ISO split_example.xml carries two Name branches in one Purpose.
#[test]
fn purpose_choice_branches_may_repeat() {
    let purpose = format!(
        concat!(
            r#"<Purpose dt:GUID="{G3}">"#,
            r#"<Name language="en">One</Name>"#,
            r#"<Name language="de">Zwei</Name>"#,
            r#"<Description language="en">D1</Description>"#,
            r#"<Description language="de">D2</Description>"#,
            r#"</Purpose>"#,
        ),
        G3 = G3
    );
    let original = format!(r#"<Purpose dt:GUID="{G3}"><Name language="en">P</Name></Purpose>"#);
    let body = prerequisites().replace(&original, &purpose);
    let xml = document(&body);
    assert!(errors(&xml).is_empty(), "{:#?}", errors(&xml));
}

/// Issue #9: a serializer walking `ShapeInfluence` fields in declaration
/// order must emit valid document order. Derived `Debug` lists fields in
/// declaration order, so this drives XML generation from it and validates.
#[test]
fn shape_influence_field_order_matches_xsd_sequence() {
    let debug = format!("{:?}", ShapeInfluence::new());
    let fields = [
        ("inside_geometry", "InsideGeometry"),
        ("connections", "Connections"),
        ("openings", "Openings"),
        (
            "operating_and_clearance_zones",
            "OperatingAndClearanceZones",
        ),
        ("features", "Features"),
        ("threshold_dimension", "ThresholdDimension"),
    ];
    let mut ordered: Vec<(usize, &str)> = fields
        .iter()
        .map(|(rust, xml)| (debug.find(rust).expect("field in Debug"), *xml))
        .collect();
    ordered.sort_by_key(|(at, _)| *at);

    // Emit each child in declaration order. ThresholdDimension needs content,
    // so only the ordering-relevant simple children are emitted plus it last.
    let mut body = String::new();
    for (_, xml) in &ordered {
        if *xml == "ThresholdDimension" {
            body.push_str(r#"<ThresholdDimension><Threshold>1</Threshold></ThresholdDimension>"#);
        } else {
            body.push_str(&format!("<{xml}>NotRequired</{xml}>"));
        }
    }
    let detail = format!("<Detail><ShapeInfluence>{body}</ShapeInfluence></Detail>");
    let geometry = format!(
        r#"<SpecificationPerObjectType dt:GUID="{G1}" dateOfCreation="2026-01-01T00:00:00Z"><ObjectType /><GeometricalInformation dt:GUID="{G2}">{detail}</GeometricalInformation></SpecificationPerObjectType>"#,
    );
    let prerequisites = prerequisites();
    let xml = document(&format!("{prerequisites}{geometry}"));
    let out_of_order: Vec<_> = errors(&xml)
        .into_iter()
        .filter(|d| format!("{:?}", d.code()).contains("ChildOutOfOrder"))
        .collect();
    assert!(
        out_of_order.is_empty(),
        "declaration order must match the XSD sequence: {out_of_order:#?}"
    );
}
