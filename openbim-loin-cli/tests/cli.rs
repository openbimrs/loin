//! End-to-end tests of the `openbim-loin` binary against the shipped examples.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const EXAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../openbim-loin/examples");

fn run(args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_openbim-loin"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let mut input = child.stdin.take().expect("stdin");
    if let Some(bytes) = stdin {
        input.write_all(bytes).expect("write stdin");
    }
    drop(input);
    child.wait_with_output().expect("wait")
}

fn example(name: &str) -> String {
    format!("{EXAMPLES}/{name}.loin.xml")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exit code")
}

#[test]
fn validate_accepts_the_examples() {
    for name in [
        "office-fit-out",
        "site-georeferencing",
        "deferred-object-type",
    ] {
        let output = run(&["validate", &example(name)], None);
        assert_eq!(code(&output), 0, "{name}: {}", stdout(&output));
        assert_eq!(stdout(&output), "no findings\n");
    }
}

#[test]
fn validate_reports_errors_in_text_and_json() {
    let bad = br#"<l:LevelOfInformationNeed xmlns:l="https://iso.org/2024/LOIN"/>"#;
    let text = run(&["validate", "-"], Some(bad));
    assert_eq!(code(&text), 1);
    assert!(
        stdout(&text).contains("MissingRequiredChild"),
        "{}",
        stdout(&text)
    );
    let json = run(&["validate", "--format", "json"], Some(bad));
    assert_eq!(code(&json), 1);
    let body = stdout(&json);
    assert!(
        body.starts_with("[{\"severity\":\"Error\",\"code\":\"MissingRequiredChild\""),
        "{body}"
    );
}

#[test]
fn read_succeeds_on_a_readable_example_and_refuses_the_nilled_one() {
    let ok = run(&["read", &example("office-fit-out")], None);
    assert_eq!(code(&ok), 0);
    assert!(stdout(&ok).starts_with("read OK: "));
    let refused = run(
        &["read", "--format", "json", &example("deferred-object-type")],
        None,
    );
    assert_eq!(code(&refused), 1);
    assert!(
        stdout(&refused).contains("\"kind\":\"NilledElement\""),
        "{}",
        stdout(&refused)
    );
}

#[test]
fn lint_is_clean_on_examples_and_flags_a_nil_guid() {
    assert_eq!(code(&run(&["lint", &example("office-fit-out")], None)), 0);
    let source = std::fs::read_to_string(example("site-georeferencing")).unwrap();
    let guid = source
        .split("dt:GUID=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_owned();
    let nilled = source.replacen(&guid, "00000000-0000-0000-0000-000000000000", 1);
    let output = run(&["lint"], Some(nilled.as_bytes()));
    assert_eq!(code(&output), 1);
    assert!(stdout(&output).contains("NilGuid"), "{}", stdout(&output));
}

#[test]
fn rewrite_is_byte_identical_for_the_examples() {
    let path = example("office-fit-out");
    let output = run(&["rewrite", &path], None);
    assert_eq!(code(&output), 0);
    assert_eq!(output.stdout, std::fs::read(path).unwrap());
}

#[test]
fn migrate_changes_namespace_and_reports_on_stderr() {
    let output = run(
        &["migrate", "--to", "2022", &example("site-georeferencing")],
        None,
    );
    assert_eq!(code(&output), 0);
    assert!(stdout(&output).contains("https://iso.org/2022/LOIN"));
    assert!(!stdout(&output).contains("https://iso.org/2024/LOIN"));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("migrated Draft2024 -> Draft2022"));
}

#[test]
fn utf16_input_is_accepted_and_written_as_utf8() {
    let source = std::fs::read_to_string(example("site-georeferencing")).unwrap();
    let encode = |text: &str| {
        let mut wide = vec![0xFF, 0xFE];
        for unit in text.encode_utf16() {
            wide.extend(unit.to_le_bytes());
        }
        wide
    };
    // The example declares UTF-8, which contradicts UTF-16 bytes.
    let mismatched = run(&["rewrite"], Some(&encode(&source)));
    assert_eq!(code(&mismatched), 2);
    let declared16 = source.replacen("encoding=\"UTF-8\"", "encoding=\"UTF-16\"", 1);
    assert_ne!(declared16, source, "example declares an encoding");
    let output = run(&["rewrite"], Some(&encode(&declared16)));
    assert_eq!(
        code(&output),
        0,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout).is_ok());
}

#[test]
fn grammar_prints_json() {
    let output = run(&["grammar"], None);
    assert_eq!(code(&output), 0);
    assert!(stdout(&output).contains("\"openbim-loin-grammar\""));
}

#[test]
fn usage_and_input_errors_exit_two() {
    for args in [
        &["bogus"][..],
        &["validate", "--format", "xml"],
        &["migrate"],
        &["validate", "/nonexistent.xml"],
        &[],
    ] {
        let output = run(args, None);
        assert_eq!(code(&output), 2, "{args:?}");
    }
    let garbage = run(&["validate"], Some(b"not xml"));
    assert_eq!(code(&garbage), 2);
    assert!(String::from_utf8_lossy(&garbage.stderr).starts_with("error: "));
}

#[test]
fn help_and_version_exit_zero() {
    assert_eq!(code(&run(&["--help"], None)), 0);
    let version = run(&["--version"], None);
    assert_eq!(code(&version), 0);
    assert!(stdout(&version).starts_with("openbim-loin "));
}
