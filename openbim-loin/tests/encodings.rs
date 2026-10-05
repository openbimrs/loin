//! `LoinDocument::parse_bytes`: encoding detection and refusal.

use openbim_loin::{LoinDocument, OutputNamespace, ParseErrorKind, ParseOptions};

const BODY: &str =
    "<l:LevelOfInformationNeed xmlns:l=\"https://iso.org/2024/LOIN\" note=\"é€😀\"/>";

fn utf16(text: &str, little: bool, bom: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if bom {
        out.extend(if little { [0xFF, 0xFE] } else { [0xFE, 0xFF] });
    }
    for unit in text.encode_utf16() {
        out.extend(if little {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        });
    }
    out
}

fn kind(bytes: &[u8]) -> ParseErrorKind {
    LoinDocument::parse_bytes(bytes)
        .expect_err("must refuse")
        .kind()
}

#[test]
fn utf8_with_and_without_bom() {
    let plain = LoinDocument::parse_bytes(BODY.as_bytes()).unwrap();
    let mut with_bom = vec![0xEF, 0xBB, 0xBF];
    with_bom.extend(BODY.as_bytes());
    assert_eq!(LoinDocument::parse_bytes(&with_bom).unwrap(), plain);
    assert_eq!(plain, LoinDocument::parse(BODY).unwrap());
}

#[test]
fn utf16_with_bom_and_sniffed_without() {
    let expected = LoinDocument::parse(BODY).unwrap();
    for (little, bom) in [(true, true), (false, true), (true, false), (false, false)] {
        let document = LoinDocument::parse_bytes(&utf16(BODY, little, bom)).unwrap();
        assert_eq!(document, expected, "little={little} bom={bom}");
    }
}

#[test]
fn utf16_declaration_is_rewritten_as_utf8() {
    let xml = format!("<?xml version=\"1.0\" encoding=\"UTF-16\"?>{BODY}");
    let document = LoinDocument::parse_bytes(&utf16(&xml, true, true)).unwrap();
    assert_eq!(
        document.declaration().unwrap().encoding.as_deref(),
        Some("UTF-8")
    );
    let written = document.to_xml_string(OutputNamespace::Preserve).unwrap();
    assert!(written.contains("encoding=\"UTF-8\""));
    assert!(LoinDocument::parse_bytes(written.as_bytes()).is_ok());
}

#[test]
fn declarations_that_contradict_the_bytes_are_refused() {
    for (declared, bytes_are_utf16) in [
        ("UTF-16", false),
        ("ISO-8859-1", false),
        ("windows-1252", false),
        ("UTF-8", true),
        ("UTF-16BE", true), // bytes below are little-endian
    ] {
        let xml = format!("<?xml version=\"1.0\" encoding=\"{declared}\"?>{BODY}");
        let bytes = if bytes_are_utf16 {
            utf16(&xml, true, true)
        } else {
            xml.into_bytes()
        };
        assert_eq!(kind(&bytes), ParseErrorKind::InvalidEncoding, "{declared}");
    }
}

#[test]
fn matching_declarations_are_accepted_case_insensitively() {
    let xml = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>{BODY}");
    assert!(LoinDocument::parse_bytes(xml.as_bytes()).is_ok());
    let xml = format!("<?xml version=\"1.0\" encoding=\"utf-16le\"?>{BODY}");
    assert!(LoinDocument::parse_bytes(&utf16(&xml, true, true)).is_ok());
}

#[test]
fn malformed_byte_sequences_are_refused() {
    assert_eq!(kind(&[0xFF, 0xFF, 0xFF]), ParseErrorKind::InvalidEncoding);
    let mut bad = BODY.as_bytes().to_vec();
    bad[10] = 0xC0; // overlong lead byte
    assert_eq!(kind(&bad), ParseErrorKind::InvalidEncoding);
    let mut odd = utf16(BODY, true, true);
    odd.pop();
    assert_eq!(kind(&odd), ParseErrorKind::InvalidEncoding);
    let mut lone = vec![0xFF, 0xFE];
    lone.extend([0x3C, 0x00, 0x00, 0xD8]); // '<' then a lone high surrogate
    assert_eq!(kind(&lone), ParseErrorKind::InvalidEncoding);
}

#[test]
fn the_byte_budget_applies_to_raw_input() {
    let wide = utf16(BODY, true, true);
    let options = ParseOptions {
        max_bytes: wide.len() - 1,
        ..ParseOptions::default()
    };
    let error = LoinDocument::parse_bytes_with_options(&wide, options).expect_err("over budget");
    assert_eq!(error.kind(), ParseErrorKind::InputTooLarge);
    let roomy = ParseOptions {
        max_bytes: wide.len(),
        ..ParseOptions::default()
    };
    assert!(LoinDocument::parse_bytes_with_options(&wide, roomy).is_ok());
}

#[test]
fn deep_utf16_input_is_still_refused_not_overflowed() {
    let deep = format!(
        "<l:LevelOfInformationNeed xmlns:l=\"https://iso.org/2024/LOIN\">{}</l:LevelOfInformationNeed>",
        "<a>".repeat(50_000)
    );
    assert_eq!(kind(&utf16(&deep, false, true)), ParseErrorKind::DepthLimit);
}
