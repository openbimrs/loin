# loin

Short-name package for ISO 7817-3 / EN 17412-3 Level of Information Need.

This crate is a **pure re-export** of
[`openbim-loin`](https://crates.io/crates/openbim-loin). It defines nothing of
its own, so both package names expose one canonical API and one type identity.
The dependency is pinned to the exact canonical version.

```toml
loin = "0.3"
# equivalent API to:
openbim-loin = "0.3"
```

Version `0.3` exposes DT-backed LOIN domain contracts together with the canonical
crate's strict, bounded semantic XML reader and writer, explicit 2022/2024
namespace migration, and clause-level validation. Imported ISO 23387 complex
internals remain owned by `openbim-dt` and are not claimed as recursively
validated.

## Documentation

- [Repository README](https://github.com/openbimrs/loin#readme)
- [API documentation](https://docs.rs/openbim-loin)

## License

AGPL-3.0-or-later
