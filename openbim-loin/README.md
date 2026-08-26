# openbim-loin

ISO 7817-3 / EN 17412-3 Level of Information Need contracts for Rust.

Version `0.2.0` implements the domain boundary that ISO 7817-3 imports from ISO
23387. Purpose and related LOIN items use DT-owned GUID, language, multilingual
text, and reference values. Per-object specifications use the exact DT concept,
object-type, property, quantity-kind, group, reference-document, unit, and
dimension contracts.

```rust
use openbim_loin::{Purpose, dt};

let guid: dt::Guid = "11111111-1111-1111-1111-111111111111".parse()?;
let purpose = Purpose::new(guid, dt::MultiLanguageText::new("en", "Coordination")?);
assert_eq!(purpose.name().text(), "Coordination");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`openbim-loin::dt` re-exports the exact `openbim-dt` version used by the public
LOIN model.

This release does **not** parse, write, migrate, or XSD-validate complete LOIN
XML documents. The 2024 and 2022 namespaces remain draft contracts; a future
codec must preserve the source namespace and require an explicit output target.

No ISO/CEN schema is vendored. Standards and public committee drafts do not
implicitly grant redistribution rights; local references stay out of tree.

## Documentation

- [Repository README](https://github.com/openbimrs/loin#readme)
- [Architecture](https://github.com/openbimrs/loin/blob/main/docs/architecture.md)
- [API documentation](https://docs.rs/openbim-loin)

## License

MIT
