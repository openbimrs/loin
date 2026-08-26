# loin

Short-name package for ISO 7817-3 / EN 17412-3 Level of Information Need.

This crate is a **pure re-export** of
[`openbim-loin`](https://crates.io/crates/openbim-loin). It defines nothing of
its own, so both package names expose one canonical API and one type identity.
The dependency is pinned to the exact canonical version.

```toml
loin = "0.2"
# equivalent API to:
openbim-loin = "0.2"
```

Version `0.2` exposes DT-backed LOIN domain contracts. It does not yet expose a
complete LOIN XML reader, writer, namespace migrator, or XSD validator.

## Documentation

- [Repository README](https://github.com/openbimrs/loin#readme)
- [API documentation](https://docs.rs/openbim-loin)

## License

MIT
