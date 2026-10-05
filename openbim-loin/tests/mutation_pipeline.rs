//! Seeded byte-level mutation of the shipped documents through the whole
//! pipeline. Deterministic (fixed seed, bounded count) so it runs in the gate;
//! a panic or abort anywhere fails the test, and every accepted mutant must
//! keep the cross-stage invariants below.

use openbim_loin::{
    LevelOfInformationNeed, LoinDocument, NamespaceVersion, OutputNamespace, Severity,
};

const SOURCES: [(&str, &str); 4] = [
    (
        "office-fit-out",
        include_str!("../examples/office-fit-out.loin.xml"),
    ),
    (
        "site-georeferencing",
        include_str!("../examples/site-georeferencing.loin.xml"),
    ),
    (
        "deferred-object-type",
        include_str!("../examples/deferred-object-type.loin.xml"),
    ),
    (
        "reader-maximal",
        include_str!("fixtures/reader-maximal.xml"),
    ),
];
const MUTANTS_PER_SOURCE: usize = 1_500;
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

struct Rng(u64);

impl Rng {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

fn mutate(rng: &mut Rng, original: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();
    for _ in 0..=rng.below(4) {
        match rng.below(5) {
            0 => {
                let at = rng.below(bytes.len());
                bytes[at] = b"<>&\"'= /x0\n"[rng.below(11)];
            }
            1 => {
                bytes.remove(rng.below(bytes.len()));
            }
            2 => {
                let from = rng.below(bytes.len());
                let to = (from + rng.below(200)).min(bytes.len());
                let segment = bytes[from..to].to_vec();
                let at = rng.below(bytes.len());
                bytes.splice(at..at, segment);
            }
            3 => {
                let from = rng.below(bytes.len());
                let to = (from + rng.below(300)).min(bytes.len());
                bytes.drain(from..to);
            }
            _ => {
                let at = rng.below(bytes.len());
                bytes[at] = rng.below(256) as u8;
            }
        }
        if bytes.is_empty() {
            bytes.push(b'<');
        }
    }
    bytes
}

fn error_count(document: &LoinDocument) -> usize {
    document
        .validate()
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .count()
}

#[test]
fn mutants_never_break_pipeline_invariants() {
    let mut rng = Rng(SEED);
    let (mut parsed, mut read, mut total) = (0usize, 0usize, 0usize);
    for (name, source) in SOURCES {
        for _ in 0..MUTANTS_PER_SOURCE {
            let bytes = mutate(&mut rng, source.as_bytes());
            // parse_bytes must agree with parse on valid UTF-8 and never panic otherwise.
            let from_bytes = LoinDocument::parse_bytes(&bytes);
            let Ok(text) = String::from_utf8(bytes) else {
                assert!(from_bytes.is_err(), "{name}: invalid UTF-8 accepted");
                continue;
            };
            total += 1;
            let Ok(document) = LoinDocument::parse(&text) else {
                continue;
            };
            parsed += 1;
            if let Ok(via_bytes) = &from_bytes {
                assert_eq!(via_bytes, &document, "{name}: parse_bytes disagrees");
            }

            let errors = error_count(&document);

            // Every writer and migrator must stay total on any parsed tree.
            // (Parsed trees are written back; only edited trees can hold
            // content XML cannot represent.)
            let written = document
                .to_xml_string(OutputNamespace::Preserve)
                .unwrap_or_else(|e| panic!("{name}: parsed tree not writable: {e}"));
            LoinDocument::parse(&written)
                .unwrap_or_else(|e| panic!("{name}: rewrite unparsable: {e}"));
            let _ = document.migrated(match document.observed_namespace() {
                NamespaceVersion::Draft2022 => NamespaceVersion::Draft2024,
                NamespaceVersion::Draft2024 => NamespaceVersion::Draft2022,
            });

            let Ok(model) = LevelOfInformationNeed::from_document(&document) else {
                continue;
            };
            read += 1;
            assert_eq!(
                errors, 0,
                "{name}: reader accepted a document the validator rejects"
            );

            let rebuilt = LoinDocument::from_model(&model).expect("model converts");
            assert_eq!(
                error_count(&rebuilt),
                0,
                "{name}: written model does not validate"
            );
            let xml = rebuilt
                .to_xml_string(OutputNamespace::Preserve)
                .unwrap_or_else(|e| panic!("{name}: read model not writable: {e}"));
            let reparsed = LoinDocument::parse(&xml).expect("written model reparses");
            let again = LevelOfInformationNeed::from_document(&reparsed).expect("reads back");
            assert_eq!(
                again, model,
                "{name}: model -> document -> model is not identity"
            );
        }
    }
    // Guard against a vacuous run (for example every mutant failing to parse).
    assert!(parsed > total / 20, "only {parsed}/{total} mutants parsed");
    assert!(read > 20, "only {read} mutants reached the typed model");
}
