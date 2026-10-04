//! Parses, validates and round-trips LOIN documents.
//!
//! ```text
//! cargo run -p openbim-loin --example check_examples            # every examples/*.xml
//! cargo run -p openbim-loin --example check_examples -- a.xml   # your own files
//! ```
//!
//! For each document it checks, in order:
//!
//! 1. it parses;
//! 2. `validate()` reports no diagnostics;
//! 3. writing with `OutputNamespace::Preserve` reproduces the input byte for byte;
//! 4. it reads into the typed model (a documented refusal is reported, not failed);
//! 5. writing it back with `LoinDocument::from_model` validates and reads
//!    back to the same model.
//!
//! Exits non-zero when a check fails.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use openbim_loin::{LevelOfInformationNeed, LoinDocument, OutputNamespace};

fn main() -> ExitCode {
    let mut paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let Ok(entries) = std::fs::read_dir(&directory) else {
            eprintln!("cannot read {}", directory.display());
            return ExitCode::FAILURE;
        };
        paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "xml"))
            .collect();
        paths.sort();
    }
    if paths.is_empty() {
        eprintln!("no LOIN documents to check");
        return ExitCode::FAILURE;
    }

    let mut failures = 0;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in &paths {
        println!("{}", path.strip_prefix(root).unwrap_or(path).display());
        if let Err(reason) = check(path) {
            println!("  FAILED: {reason}");
            failures += 1;
        }
    }
    if failures > 0 {
        eprintln!("{failures} of {} documents failed", paths.len());
        return ExitCode::FAILURE;
    }
    println!(
        "examples OK: {} documents validated clean and round-tripped",
        paths.len()
    );
    ExitCode::SUCCESS
}

fn check(path: &Path) -> Result<(), String> {
    let xml = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let document = LoinDocument::parse(&xml).map_err(|error| format!("parse: {error}"))?;
    println!("  parsed");

    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        for diagnostic in &diagnostics {
            println!("    {diagnostic}");
        }
        return Err(format!("{} validation diagnostics", diagnostics.len()));
    }
    println!("  validate(): no diagnostics");

    let written = document
        .to_xml_string(OutputNamespace::Preserve)
        .map_err(|error| format!("write: {error}"))?;
    if written != xml {
        return Err("Preserve output differs from the input bytes".to_owned());
    }
    println!("  byte-for-byte round trip ({} bytes)", xml.len());

    let model = match LevelOfInformationNeed::from_document(&document) {
        Ok(model) => model,
        Err(refusal) => {
            println!(
                "  typed model: refused ({:?} at {})",
                refusal.kind(),
                refusal.path()
            );
            return Ok(());
        }
    };
    println!(
        "  typed model: {} specification(s)",
        model.specifications().len()
    );

    let authored =
        LoinDocument::from_model(&model).map_err(|error| format!("from_model: {error}"))?;
    let diagnostics = authored.validate();
    if !diagnostics.is_empty() {
        return Err(format!(
            "document written from the model has {} diagnostics",
            diagnostics.len()
        ));
    }
    let reread = LevelOfInformationNeed::from_document(&authored)
        .map_err(|error| format!("written document does not read back: {error}"))?;
    if reread != model {
        return Err("model -> document -> model is not the identity".to_owned());
    }
    println!("  model -> document -> model: identical");
    Ok(())
}
