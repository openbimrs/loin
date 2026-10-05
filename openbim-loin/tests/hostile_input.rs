//! Hostile inputs must be refused with an error, never abort or emit XML that
//! the parser rejects.

use std::str::FromStr;

use openbim_loin::{
    dt, Actor, InformationDeliveryMilestone, LevelOfInformationNeed, LoinDocument, OutputNamespace,
    ParseErrorKind, ParseOptions, Prerequisites, Purpose, Specification, WriteError, XmlAttribute,
    XmlElement, XmlNode,
};

const LOIN: &str = "xmlns:l=\"https://iso.org/2024/LOIN\"";

fn nested(depth: usize) -> String {
    format!(
        "<l:LevelOfInformationNeed {LOIN}>{}{}</l:LevelOfInformationNeed>",
        "<a>".repeat(depth),
        "</a>".repeat(depth)
    )
}

/// Runs `f` on a thread with a small stack, as in wasm (about 1 MiB by
/// default). A stack overflow aborts the whole test process.
fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(f)
        .expect("spawn")
        .join()
        .expect("no panic")
}

#[test]
fn nesting_beyond_the_budget_is_refused_not_a_stack_overflow() {
    for depth in [129, 1_000, 10_000, 100_000] {
        let xml = nested(depth);
        let error = on_small_stack(move || LoinDocument::parse(&xml).expect_err("too deep"));
        assert_eq!(error.kind(), ParseErrorKind::DepthLimit, "depth {depth}");
    }
}

#[test]
fn unbalanced_deep_nesting_is_refused_without_overflow() {
    let xml = format!("<l:LevelOfInformationNeed {LOIN}>{}", "<a>".repeat(100_000));
    let error = on_small_stack(move || LoinDocument::parse(&xml).expect_err("too deep"));
    assert_eq!(error.kind(), ParseErrorKind::DepthLimit);
}

#[test]
fn nesting_at_the_budget_parses_validates_and_writes() {
    // The root is level 1, so 127 more levels is exactly max_depth (128).
    let xml = nested(127);
    on_small_stack(move || {
        let document = LoinDocument::parse(&xml).expect("at the budget");
        let _ = document.validate();
        let written = document.to_xml_string(OutputNamespace::Preserve).unwrap();
        assert_eq!(written, xml);
    });
}

#[test]
fn a_custom_depth_budget_is_honoured() {
    let options = ParseOptions {
        max_depth: 4,
        ..ParseOptions::default()
    };
    assert!(LoinDocument::parse_with_options(&nested(3), options).is_ok());
    let error = LoinDocument::parse_with_options(&nested(4), options).expect_err("over");
    assert_eq!(error.kind(), ParseErrorKind::DepthLimit);
}

fn guid(n: u32) -> dt::Guid {
    dt::Guid::from_str(&format!("00000000-0000-4000-8000-{n:012}")).unwrap()
}

fn text(value: &str) -> dt::MultiLanguageText {
    dt::MultiLanguageText::new("en", value).unwrap()
}

fn model(name: &str) -> LevelOfInformationNeed {
    let prerequisites = Prerequisites::new(
        guid(2),
        Purpose::new(guid(3), text("P")),
        InformationDeliveryMilestone::new(guid(4), text("M")),
        Actor::new(guid(5), text("A")),
        Actor::new(guid(6), text("B")),
    );
    LevelOfInformationNeed::new(Specification::new(guid(1), name, prerequisites))
}

fn write(model: &LevelOfInformationNeed) -> Result<String, WriteError> {
    LoinDocument::from_model(model)
        .expect("model converts")
        .to_xml_string(OutputNamespace::Preserve)
}

#[test]
fn characters_xml_forbids_are_refused_at_write_time() {
    for bad in [
        "a\u{0}b",
        "a\u{1}b",
        "a\u{8}b",
        "a\u{b}b",
        "a\u{fffe}b",
        "a\u{ffff}b",
    ] {
        let error = write(&model(bad)).expect_err("must refuse");
        assert!(
            matches!(error, WriteError::InvalidContent { .. }),
            "{bad:?}"
        );
    }
}

#[test]
fn characters_xml_allows_still_round_trip() {
    for good in [
        "",
        "  x  ",
        "a\r\n\tb",
        "<&\"'>",
        "\u{d7ff}\u{e000}\u{fffd}\u{10000}\u{10ffff}",
    ] {
        let original = model(good);
        let xml = write(&original).expect("legal content writes");
        let document = LoinDocument::parse(&xml).expect("output reparses");
        let back = LevelOfInformationNeed::from_document(&document).expect("rereads");
        assert_eq!(back, original, "{good:?}");
    }
}

fn root_with(node: XmlNode) -> LoinDocument {
    let xml = format!("<l:LevelOfInformationNeed {LOIN}/>");
    let mut document = LoinDocument::parse(&xml).unwrap();
    document.root_mut().push_node(node);
    document
}

#[test]
fn edited_trees_are_checked_too() {
    let cases = [
        XmlNode::Text("a\u{0}b".into()),
        XmlNode::CData("a]]>b".into()),
        XmlNode::CData("a\u{1}b".into()),
        XmlNode::Comment("a--b".into()),
        XmlNode::Comment("trailing-".into()),
        XmlNode::Comment("a\u{2}b".into()),
        XmlNode::ProcessingInstruction("t a?>b".into()),
    ];
    for node in cases {
        let document = root_with(node.clone());
        let error = document
            .to_xml_string(OutputNamespace::Preserve)
            .expect_err(&format!("{node:?}"));
        assert!(
            matches!(error, WriteError::InvalidContent { .. }),
            "{node:?}"
        );
    }
    let mut document = LoinDocument::parse(&format!("<l:LevelOfInformationNeed {LOIN}/>")).unwrap();
    document
        .root_mut()
        .attributes_mut()
        .push(XmlAttribute::new("note", "a\u{0}b"));
    assert!(matches!(
        document.to_xml_string(OutputNamespace::Preserve),
        Err(WriteError::InvalidContent { .. })
    ));
    let _ = XmlElement::new("unused");
}

#[test]
fn a_successful_write_always_reparses() {
    for value in ["plain", "a\u{0}b", "x\u{1f}y", "ok\u{85}ok", "tab\tok"] {
        if let Ok(xml) = write(&model(value)) {
            LoinDocument::parse(&xml)
                .unwrap_or_else(|e| panic!("{value:?} wrote unparsable XML: {e}"));
        }
    }
}
