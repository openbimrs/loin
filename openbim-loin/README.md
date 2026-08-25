# openbim-loin

ISO 7817-3 / EN 17412-3 Level of Information Need (LOIN) contracts for Rust.

LOIN expresses how much geometric information, alphanumeric information, and
documentation is required for a purpose and milestone. EN 17412-1 defines the
concepts; part 3 defines the machine-readable exchange format.

## Status

This `0.1.0` release is a **reserved scaffold with namespace contracts**. It
exports the 2024 and 2022 draft namespace URIs and a tested recognition helper.
It does not parse, write, migrate, or validate LOIN documents.

The namespace is not treated as final. A future codec must preserve the source
namespace and require an explicit target namespace when writing.

No ISO/CEN schema is vendored. Standards and public committee drafts do not
implicitly grant redistribution rights; local references stay out of tree.

## Documentation

- [Repository README](https://github.com/openbimrs/loin#readme)
- [Architecture](https://github.com/openbimrs/loin/blob/main/docs/architecture.md)
- [API documentation](https://docs.rs/openbim-loin)

## License

MIT
