//! Browser bindings for `openbim-loin`.
//!
//! The core crate carries no wasm or JS dependency; this crate is the only
//! place `wasm-bindgen` appears. Diagnostics *and* errors cross the boundary
//! as structured JS objects rather than formatted strings, so callers branch
//! on `code`, `severity` and `kind` instead of parsing message text.

use openbim_loin::{
    Diagnostic, DiagnosticCode, LoinDocument, OutputNamespace, ParseError, ParseErrorKind, Severity,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Installs a panic hook so a Rust panic reaches the JS console with a stack
/// trace instead of surfacing as a bare `unreachable executed`.
///
/// `skip_typescript` keeps this internal detail out of the published `.d.ts`;
/// wasm-bindgen still invokes it automatically on module init.
#[wasm_bindgen(start, skip_typescript)]
fn start() {
    console_error_panic_hook::set_once();
}

/// Stable JS-facing spelling of a diagnostic code.
///
/// Written out rather than derived from `Debug`: the strings are public API
/// for every JS consumer, and deriving them would let a Rust-side rename
/// silently break them. The match is exhaustive on purpose, so adding a
/// variant upstream fails this crate's build and forces a decision here.
const fn code_str(code: DiagnosticCode) -> &'static str {
    match code {
        DiagnosticCode::UnexpectedNamespace => "UnexpectedNamespace",
        DiagnosticCode::UnknownElement => "UnknownElement",
        DiagnosticCode::UnexpectedAttribute => "UnexpectedAttribute",
        DiagnosticCode::MissingRequiredAttribute => "MissingRequiredAttribute",
        DiagnosticCode::MissingRequiredChild => "MissingRequiredChild",
        DiagnosticCode::TooManyChildren => "TooManyChildren",
        DiagnosticCode::ChildOutOfOrder => "ChildOutOfOrder",
        DiagnosticCode::UnexpectedText => "UnexpectedText",
        DiagnosticCode::InvalidGuid => "InvalidGuid",
        DiagnosticCode::InvalidAnyUri => "InvalidAnyUri",
        DiagnosticCode::MissingLanguage => "MissingLanguage",
        DiagnosticCode::InvalidLanguage => "InvalidLanguage",
        DiagnosticCode::InvalidDateTime => "InvalidDateTime",
        DiagnosticCode::InvalidBoolean => "InvalidBoolean",
        DiagnosticCode::InvalidDecimal => "InvalidDecimal",
        DiagnosticCode::InvalidDouble => "InvalidDouble",
        DiagnosticCode::InvalidEnumeration => "InvalidEnumeration",
        DiagnosticCode::NilledContent => "NilledContent",
        DiagnosticCode::UnsupportedXsiType => "UnsupportedXsiType",
        DiagnosticCode::CompatibilityProfile => "CompatibilityProfile",
    }
}

/// Stable JS-facing spelling of a severity. See [`code_str`].
const fn severity_str(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
    }
}

/// Stable JS-facing spelling of a parse-failure category. See [`code_str`].
const fn kind_str(kind: ParseErrorKind) -> &'static str {
    match kind {
        ParseErrorKind::InputTooLarge => "InputTooLarge",
        ParseErrorKind::DepthLimit => "DepthLimit",
        ParseErrorKind::NodeLimit => "NodeLimit",
        ParseErrorKind::AttributeLimit => "AttributeLimit",
        ParseErrorKind::DoctypeForbidden => "DoctypeForbidden",
        ParseErrorKind::UnsupportedXmlVersion => "UnsupportedXmlVersion",
        ParseErrorKind::DuplicateExpandedAttribute => "DuplicateExpandedAttribute",
        ParseErrorKind::UnknownEntity => "UnknownEntity",
        ParseErrorKind::UndeclaredPrefix => "UndeclaredPrefix",
        ParseErrorKind::MalformedQName => "MalformedQName",
        ParseErrorKind::MissingRoot => "MissingRoot",
        ParseErrorKind::MultipleRoots => "MultipleRoots",
        ParseErrorKind::UnexpectedRoot => "UnexpectedRoot",
        ParseErrorKind::UnsupportedNamespace => "UnsupportedNamespace",
        ParseErrorKind::MalformedXml => "MalformedXml",
        ParseErrorKind::InvalidEncoding => "InvalidEncoding",
    }
}

/// One validation finding, shaped for JS consumption.
#[derive(Serialize)]
pub struct JsDiagnostic {
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

fn to_js(d: &Diagnostic) -> JsDiagnostic {
    JsDiagnostic {
        severity: severity_str(d.severity()).to_owned(),
        code: code_str(d.code()).to_owned(),
        path: d.path().to_owned(),
        message: d.message().to_owned(),
    }
}

/// Builds a JS `Error` that keeps the parse failure machine-readable.
///
/// `kind` and `position` are attached as own properties, so a browser editor
/// can place a marker at the byte offset instead of regexing the message.
fn parse_error(error: &ParseError) -> JsValue {
    let js = js_sys::Error::new(&error.to_string());
    js.set_name("LoinParseError");
    let value: JsValue = js.into();
    // Reflect::set only fails for a non-object target; `value` is an Error.
    let _ = js_sys::Reflect::set(
        &value,
        &JsValue::from_str("kind"),
        &JsValue::from_str(kind_str(error.kind())),
    );
    let _ = js_sys::Reflect::set(
        &value,
        &JsValue::from_str("position"),
        &JsValue::from_f64(error.position() as f64),
    );
    value
}

/// Builds a JS `Error` for a write failure.
fn write_error(message: &str) -> JsValue {
    let js = js_sys::Error::new(message);
    js.set_name("LoinWriteError");
    js.into()
}

// Hand-written declarations: wasm-bindgen types a `JsValue` return as `any`,
// which would leave the whole point of structured diagnostics untyped.
#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &'static str = r#"
/** Severity of a validation finding. */
export type Severity = "Error" | "Warning";

/** One validation finding. `code` is stable across releases. */
export interface Diagnostic {
    severity: Severity;
    code: string;
    /** Element path, e.g. `/LevelOfInformationNeed/Specification[1]`. */
    path: string;
    message: string;
}

/** Error thrown when input is not a well-formed LOIN document. */
export interface LoinParseError extends Error {
    name: "LoinParseError";
    /** Stable failure category, e.g. `MalformedXml`. */
    kind: string;
    /** Byte offset into the input. */
    position: number;
}

/**
 * Validates a LOIN document.
 *
 * @throws {LoinParseError} when the input is not a well-formed LOIN document.
 */
export function validate(xml: string): Diagnostic[];
"#;

/// Validates a LOIN document, returning an array of diagnostics.
///
/// Throws a JS `LoinParseError` carrying `kind` and `position` when the input
/// is not a well-formed LOIN document.
#[wasm_bindgen(skip_typescript)]
pub fn validate(xml: &str) -> Result<JsValue, JsValue> {
    let doc = LoinDocument::parse(xml).map_err(|e| parse_error(&e))?;
    let out: Vec<JsDiagnostic> = doc.validate().iter().map(to_js).collect();
    serde_wasm_bindgen::to_value(&out).map_err(JsValue::from)
}

/// Parses and re-serialises a document, preserving its observed namespace.
///
/// Use this to confirm an editor round-trip is lossless.
#[wasm_bindgen]
pub fn rewrite(xml: &str) -> Result<String, JsValue> {
    let doc = LoinDocument::parse(xml).map_err(|e| parse_error(&e))?;
    doc.to_xml_string(OutputNamespace::Preserve)
        .map_err(|e| write_error(&e.to_string()))
}

/// True when the input parses as a LOIN document at all.
#[wasm_bindgen(js_name = isWellFormed)]
pub fn is_well_formed(xml: &str) -> bool {
    LoinDocument::parse(xml).is_ok()
}
