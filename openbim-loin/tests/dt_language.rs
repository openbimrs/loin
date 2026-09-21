//! Regression tests for issues #2, #6 and #7.
#![allow(clippy::uninlined_format_args)]
use openbim_loin::{EmailAddress, LoinDocument, Severity};

const DT: &str = "https://standards.iso.org/iso/23387/ed-2/en/";
const LOIN: &str = "https://iso.org/2024/LOIN";
const G: &str = "70000000-0000-0000-0000-000000000000";

fn doc(inner: &str) -> String {
    format!(
        concat!(
            r#"<loin:LevelOfInformationNeed xmlns:loin="{LOIN}" xmlns:dt="{DT}">"#,
            r#"<Specification name="S" dt:GUID="{G}">{inner}</Specification>"#,
            r#"</loin:LevelOfInformationNeed>"#,
        ),
        LOIN = LOIN,
        DT = DT,
        G = G,
        inner = inner,
    )
}

#[test]
fn issue2_dt_multilingual_text_requires_language() {
    let inner = format!(
        r#"<SpecificationPerObjectType dt:GUID="{G}" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name/><ObjectType/></SpecificationPerObjectType>"#,
        G = G,
    );
    let d = LoinDocument::parse(&doc(&inner)).expect("parses");
    let errs: Vec<_> = d
        .validate()
        .into_iter()
        .filter(|x| x.severity() == Severity::Error)
        .collect();
    assert!(
        errs.iter()
            .any(|e| format!("{:?}", e.code()) == "MissingLanguage"),
        "{errs:#?}"
    );
}

/// Issue #2: a malformed `xs:language` value is rejected, not ignored.
#[test]
fn issue2_dt_multilingual_text_rejects_invalid_language() {
    let inner = format!(
        r#"<SpecificationPerObjectType dt:GUID="{G}" dateOfCreation="2026-08-26T00:00:00Z"><dt:Name language="!!not-a-language!!"/><ObjectType/></SpecificationPerObjectType>"#,
        G = G,
    );
    let d = LoinDocument::parse(&doc(&inner)).expect("parses");
    let errs: Vec<_> = d.validate().into_iter().collect();
    assert!(
        errs.iter()
            .any(|e| format!("{:?}", e.code()) == "InvalidLanguage"),
        "{errs:#?}"
    );
}

/// Issue #6: ChildOutOfOrder names the offending child, not its parent.
#[test]
fn issue6_child_out_of_order_reports_the_child_path() {
    let body = format!(
        concat!(
            r#"<Prerequisites dt:GUID="{G}">"#,
            r#"<InformationDeliveryMilestone dt:GUID="{G}">"#,
            r#"<Name language="en">M</Name></InformationDeliveryMilestone>"#,
            r#"<Purpose dt:GUID="{G}"><Name language="en">P</Name></Purpose>"#,
            r#"</Prerequisites>"#,
        ),
        G = G
    );
    let d = LoinDocument::parse(&doc(&body)).expect("parses");
    let out: Vec<_> = d
        .validate()
        .into_iter()
        .filter(|e| format!("{:?}", e.code()) == "ChildOutOfOrder")
        .collect();
    assert!(!out.is_empty(), "expected an ordering diagnostic");
    for e in &out {
        assert!(
            e.path().contains("/Purpose"),
            "path must name the child, got {}",
            e.path()
        );
    }
}

/// Issue #7: Diagnostic and EmailAddress render without hand-written code.
#[test]
fn issue7_diagnostic_and_email_address_display() {
    let d = LoinDocument::parse(&doc("")).expect("parses");
    let first = d.validate().into_iter().next().expect("a diagnostic");
    let text = first.to_string();
    assert!(text.contains(first.path()), "{text}");
    assert!(text.contains(first.message()), "{text}");
    let _: &dyn std::error::Error = &first;
    let email: EmailAddress = "a@b.example".parse().expect("valid");
    assert_eq!(email.to_string(), email.as_str());
}
