//! `LoinDocument::lint`: opt-in warnings that never change `validate()`.

use openbim_loin::{DiagnosticCode, LoinDocument, Severity};

const EXAMPLES: [&str; 4] = [
    include_str!("../examples/office-fit-out.loin.xml"),
    include_str!("../examples/site-georeferencing.loin.xml"),
    include_str!("../examples/deferred-object-type.loin.xml"),
    include_str!("fixtures/reader-maximal.xml"),
];

const HEADER: &str = "xmlns:l=\"https://iso.org/2024/LOIN\" xmlns:dt=\"https://standards.iso.org/iso/23387/ed-2/en/\"";

fn g(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

fn document(name: &str, spec_guid: &str, purpose_guid: &str, date: &str) -> String {
    format!(
        r#"<l:LevelOfInformationNeed {HEADER}><Specification name="{name}" dt:GUID="{spec_guid}"><Prerequisites dt:GUID="{p}"><Purpose dt:GUID="{purpose_guid}"><Name language="en">P</Name></Purpose><InformationDeliveryMilestone dt:GUID="{m}" Date="{date}"><Name language="en">M</Name></InformationDeliveryMilestone><ProvidingActor dt:GUID="{a}"><Role language="en">A</Role></ProvidingActor><ReceivingActor dt:GUID="{b}"><Role language="en">B</Role></ReceivingActor></Prerequisites></Specification></l:LevelOfInformationNeed>"#,
        p = g(2),
        m = g(4),
        a = g(5),
        b = g(6),
    )
}

fn lint(xml: &str) -> Vec<(DiagnosticCode, String)> {
    let document = LoinDocument::parse(xml).unwrap();
    let findings = document.lint();
    assert!(findings.iter().all(|d| d.severity() == Severity::Warning));
    findings
        .iter()
        .map(|d| (d.code(), d.path().to_owned()))
        .collect()
}

#[test]
fn shipped_documents_have_no_lint_findings() {
    for xml in EXAMPLES {
        assert_eq!(lint(xml), vec![]);
    }
}

#[test]
fn clean_document_has_no_findings() {
    assert_eq!(
        lint(&document("S", &g(1), &g(3), "2024-03-04T05:06:07Z")),
        vec![]
    );
}

#[test]
fn duplicate_guid_is_reported_at_the_second_use() {
    let findings = lint(&document("S", &g(1), &g(1), "2024-03-04T05:06:07Z"));
    assert_eq!(
        findings,
        vec![(
            DiagnosticCode::DuplicateGuid,
            "/LevelOfInformationNeed/Specification[1]/Prerequisites[1]/Purpose[1]".to_owned()
        )]
    );
}

#[test]
fn nil_guid_blank_name_and_padded_date_are_reported() {
    let nil = "00000000-0000-0000-0000-000000000000";
    let xml = document("  ", nil, &g(3), " 2024-03-04T05:06:07Z ");
    let codes: Vec<_> = lint(&xml).into_iter().map(|(code, _)| code).collect();
    assert!(codes.contains(&DiagnosticCode::NilGuid));
    assert!(codes.contains(&DiagnosticCode::EmptyName));
    assert!(codes.contains(&DiagnosticCode::PaddedDateTime));
    assert_eq!(codes.len(), 3);
}

#[test]
fn lint_findings_do_not_make_a_document_invalid() {
    let xml = document("", &g(1), &g(1), " 2024-03-04T05:06:07Z ");
    let document = LoinDocument::parse(&xml).unwrap();
    assert!(!document.lint().is_empty());
    assert!(document
        .validate()
        .iter()
        .all(|d| d.severity() != Severity::Error));
}
