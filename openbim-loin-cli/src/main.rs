//! `openbim-loin`: validate, lint, read-check, migrate and rewrite LOIN documents.

use std::io::{Read, Write};
use std::process::ExitCode;

use openbim_loin::{
    Diagnostic, LevelOfInformationNeed, LoinDocument, NamespaceVersion, OutputNamespace, Severity,
};

const USAGE: &str = "\
openbim-loin: ISO 7817-3 / EN 17412-3 LOIN documents

USAGE:
    openbim-loin <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    validate   Report schema-derived diagnostics (exit 1 if any error)
    lint       Report opt-in semantic warnings (exit 1 if any finding)
    read       Check the document reads into the typed model (exit 1 if refused)
    migrate    Rewrite into another namespace edition: --to 2022|2024
    rewrite    Parse and write back, preserving the observed namespace
    grammar    Print the published grammar as JSON (no FILE)

OPTIONS:
    --format text|json   Output format for validate, lint and read (default text)
    --to 2022|2024       Target edition for migrate
    -h, --help           Print this help
    -V, --version        Print the version

FILE is a path, or `-` (or omitted) for standard input. Input may be UTF-8 or
UTF-16 bytes.

EXIT CODES:
    0  success    1  the document has findings or was refused
    2  usage, I/O or parse error
";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = run(
        &args,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    ExitCode::from(code)
}

fn run(args: &[String], stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match execute(args, stdin, out, err) {
        Ok(code) => code,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            2
        }
    }
}

fn execute(
    args: &[String],
    stdin: &mut dyn Read,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<u8, String> {
    let mut command = None;
    let mut file = None;
    let mut format = Format::Text;
    let mut target = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => {
                write!(out, "{USAGE}").map_err(io)?;
                return Ok(0);
            }
            "-V" | "--version" => {
                writeln!(out, "openbim-loin {}", env!("CARGO_PKG_VERSION")).map_err(io)?;
                return Ok(0);
            }
            "--format" => {
                format = match iter.next().map(String::as_str) {
                    Some("text") => Format::Text,
                    Some("json") => Format::Json,
                    other => return Err(format!("--format expects text or json, got {other:?}")),
                };
            }
            "--to" => {
                target = Some(match iter.next().map(String::as_str) {
                    Some("2022") => NamespaceVersion::Draft2022,
                    Some("2024") => NamespaceVersion::Draft2024,
                    other => return Err(format!("--to expects 2022 or 2024, got {other:?}")),
                });
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option `{flag}`")),
            positional if command.is_none() => command = Some(positional.to_owned()),
            positional if file.is_none() => file = Some(positional.to_owned()),
            extra => return Err(format!("unexpected argument `{extra}`")),
        }
    }
    let Some(command) = command else {
        write!(err, "{USAGE}").map_err(io)?;
        return Ok(2);
    };
    if command == "grammar" {
        if file.is_some() {
            return Err("grammar takes no FILE".to_owned());
        }
        writeln!(out, "{}", openbim_loin::grammar::to_json().trim_end()).map_err(io)?;
        return Ok(0);
    }
    if !matches!(
        command.as_str(),
        "validate" | "lint" | "read" | "migrate" | "rewrite"
    ) {
        return Err(format!("unknown command `{command}` (try --help)"));
    }
    if command == "migrate" && target.is_none() {
        return Err("migrate requires --to 2022|2024".to_owned());
    }

    let bytes = read_input(file.as_deref(), stdin)?;
    let document = LoinDocument::parse_bytes(&bytes).map_err(|e| e.to_string())?;

    match command.as_str() {
        "validate" => {
            let diagnostics = document.validate();
            report(out, format, &diagnostics)?;
            Ok(u8::from(
                diagnostics.iter().any(|d| d.severity() == Severity::Error),
            ))
        }
        "lint" => {
            let findings = document.lint();
            report(out, format, &findings)?;
            Ok(u8::from(!findings.is_empty()))
        }
        "read" => match LevelOfInformationNeed::from_document(&document) {
            Ok(model) => {
                let count = model.specifications().len();
                match format {
                    Format::Text => writeln!(out, "read OK: {count} specification(s)"),
                    Format::Json => writeln!(out, "{{\"ok\":true,\"specifications\":{count}}}"),
                }
                .map_err(io)?;
                Ok(0)
            }
            Err(error) => {
                match format {
                    Format::Text => writeln!(out, "{error}"),
                    Format::Json => writeln!(
                        out,
                        "{{\"ok\":false,\"kind\":{},\"path\":{},\"detail\":{}}}",
                        json(&format!("{:?}", error.kind())),
                        json(error.path()),
                        json(error.detail())
                    ),
                }
                .map_err(io)?;
                Ok(1)
            }
        },
        "rewrite" => {
            let xml = document
                .to_xml_string(OutputNamespace::Preserve)
                .map_err(|e| e.to_string())?;
            out.write_all(xml.as_bytes()).map_err(io)?;
            Ok(0)
        }
        _ => {
            let target = target.expect("checked above");
            let (migrated, report) = document.migrated(target).map_err(|e| e.to_string())?;
            let xml = migrated
                .to_xml_string(OutputNamespace::Preserve)
                .map_err(|e| e.to_string())?;
            out.write_all(xml.as_bytes()).map_err(io)?;
            writeln!(
                err,
                "migrated {:?} -> {:?}: {} name(s), {} declaration(s)",
                report.source(),
                report.target(),
                report.changed_names(),
                report.changed_declarations()
            )
            .map_err(io)?;
            Ok(0)
        }
    }
}

fn read_input(file: Option<&str>, stdin: &mut dyn Read) -> Result<Vec<u8>, String> {
    match file {
        None | Some("-") => {
            let mut bytes = Vec::new();
            stdin
                .read_to_end(&mut bytes)
                .map_err(|e| format!("cannot read standard input: {e}"))?;
            Ok(bytes)
        }
        Some(path) => std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}")),
    }
}

fn report(out: &mut dyn Write, format: Format, diagnostics: &[Diagnostic]) -> Result<(), String> {
    match format {
        Format::Text => {
            for diagnostic in diagnostics {
                writeln!(out, "{diagnostic}").map_err(io)?;
            }
            if diagnostics.is_empty() {
                writeln!(out, "no findings").map_err(io)?;
            }
        }
        Format::Json => {
            let items: Vec<String> = diagnostics
                .iter()
                .map(|d| {
                    format!(
                        "{{\"severity\":{},\"code\":{},\"path\":{},\"message\":{}}}",
                        json(&format!("{:?}", d.severity())),
                        json(&format!("{:?}", d.code())),
                        json(d.path()),
                        json(d.message())
                    )
                })
                .collect();
            writeln!(out, "[{}]", items.join(",")).map_err(io)?;
        }
    }
    Ok(())
}

fn io(error: std::io::Error) -> String {
    format!("write failed: {error}")
}

/// A JSON string literal for `value`.
fn json(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::json;

    #[test]
    fn json_escapes_quotes_backslashes_and_controls() {
        assert_eq!(json("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
    }
}
