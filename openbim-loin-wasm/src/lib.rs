//! Browser bindings for `openbim-loin`.
//!
//! The core crate carries no wasm or JS dependency; this crate is the only
//! place `wasm-bindgen` appears. Diagnostics cross the boundary as
//! structured JS objects, not formatted strings, so callers branch on
//! `code` and `severity` instead of parsing text.

use openbim_loin::{LoinDocument, OutputNamespace};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// One validation finding, shaped for JS consumption.
#[derive(Serialize)]
pub struct JsDiagnostic {
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

fn to_js(d: &openbim_loin::Diagnostic) -> JsDiagnostic {
    JsDiagnostic {
        severity: format!("{:?}", d.severity()),
        code: format!("{:?}", d.code()),
        path: d.path().to_owned(),
        message: d.message().to_owned(),
    }
}

/// Validates a LOIN document, returning an array of diagnostics.
///
/// Throws a JS `Error` with the parse failure when the input is not a
/// well-formed LOIN document.
#[wasm_bindgen]
pub fn validate(xml: &str) -> Result<JsValue, JsValue> {
    let doc =
        LoinDocument::parse(xml).map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
    let out: Vec<JsDiagnostic> = doc.validate().iter().map(to_js).collect();
    serde_wasm_bindgen::to_value(&out).map_err(JsValue::from)
}

/// Parses and re-serialises a document, preserving its observed namespace.
///
/// Use this to confirm an editor round-trip is lossless.
#[wasm_bindgen]
pub fn rewrite(xml: &str) -> Result<String, JsValue> {
    let doc =
        LoinDocument::parse(xml).map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
    doc.to_xml_string(OutputNamespace::Preserve)
        .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))
}

/// True when the input parses as a LOIN document at all.
#[wasm_bindgen(js_name = isWellFormed)]
pub fn is_well_formed(xml: &str) -> bool {
    LoinDocument::parse(xml).is_ok()
}
