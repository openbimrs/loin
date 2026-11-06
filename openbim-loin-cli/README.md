# openbim-loin-cli

Command-line front end for [`openbim-loin`](../openbim-loin/): validate, lint,
read-check, migrate and rewrite ISO 7817-3 / EN 17412-3 LOIN documents.

```bash
cargo install openbim-loin-cli          # installs the `openbim-loin` binary
openbim-loin validate model.loin.xml    # diagnostics; exit 1 on errors
openbim-loin lint model.loin.xml        # opt-in warnings; exit 1 on findings
openbim-loin read model.loin.xml        # typed-model read check; exit 1 if refused
openbim-loin migrate --to 2024 old.xml  # migrated XML on stdout, report on stderr
openbim-loin rewrite model.loin.xml     # parse and write back
openbim-loin grammar                    # the published grammar as JSON
```

`validate`, `lint` and `read` accept `--format json`. Input is a path or `-`
for standard input, as UTF-8 or UTF-16.

Exit codes: `0` success, `1` the document has findings or was refused, `2`
usage, I/O or parse error.

Validation is the same XSD-derived clause-level validation as the library; it is
not complete XML Schema validation.
