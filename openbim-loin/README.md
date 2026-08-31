# openbim-loin

ISO 7817-3 / EN 17412-3 Level of Information Need contracts and a
lossless-semantic XML document codec for Rust.

The crate provides:

- owned LOIN domain contracts using `openbim-dt` for every imported ISO 23387
  GUID, language, text, reference, concept, property, quantity, group, document,
  unit, and dimension boundary;
- bounded XML 1.0 decoding and encoding that retains namespace syntax, unknown
  content, comments, processing instructions, CDATA, and represented controls;
- explicit migration between the known 2022 and 2024 draft namespaces;
- XSD-derived ISO 7817-3 structural and lexical diagnostics;
- semantic parse/write/parse stability without claiming byte-for-byte identity.

```rust
use openbim_loin::{LoinDocument, NamespaceVersion, OutputNamespace};

let xml = r#"<l:LevelOfInformationNeed
    xmlns:l="https://iso.org/2024/LOIN" />"#;
let document = LoinDocument::parse(xml)?;
let (migrated, report) = document.migrated(NamespaceVersion::Draft2022)?;
assert_eq!(report.changed_names(), 1);
let encoded = migrated.to_xml_string(OutputNamespace::Preserve)?;
assert_eq!(LoinDocument::parse(&encoded)?.root(), migrated.root());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`validate()` is clause-level validation of ISO 7817-3-owned structures,
cardinalities, enumerations, and scalar lexemes. It is **not** a claim of complete
W3C XML Schema validation for imported ISO 23387 complex types. Those contracts
remain owned by `openbim-dt`, which this crate re-exports as `openbim_loin::dt`.

No ISO/CEN schema or annex material is vendored. Standards references stay out
of the published package; public tests use synthetic documents.

## Documentation

- [Repository README](https://github.com/openbimrs/loin#readme)
- [XML codec and validation](https://github.com/openbimrs/loin/blob/main/docs/xml.md)
- [Architecture](https://github.com/openbimrs/loin/blob/main/docs/architecture.md)
- [API documentation](https://docs.rs/openbim-loin)

## License

AGPL-3.0-or-later
