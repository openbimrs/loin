# openbim-loin instructions

Purpose: canonical ISO 7817-3 / EN 17412-3 Level of Information Need package.

Follow `../AGENTS.md`. Read `PLAN.md` only for assigned implementation or roadmap
work; keep progress, blockers, and evidence there.

## Boundary

May consume released openBIM core, ISO 23387 data-template, and XML contracts.
Namespace handling must preserve observed input and make output choice explicit.
All LOIN implementation and public items live here.

## Status

The DT-backed domain boundary and strict LOIN XML document codec are implemented.
Entry points:

- `src/document.rs`: owned lossless-semantic syntax tree, explicit writing, and
  migration with collision refusal;
- `src/parser.rs`: bounded XML 1.0 parsing and namespace/QName enforcement;
- `src/validation.rs`: ISO 7817-3 XSD-derived clause-level diagnostics;
- `src/model.rs`: owned domain contracts backed by `openbim-dt`.

Validation is intentionally not advertised as complete XSD validation: imported
ISO 23387 complex content is retained and receives DT scalar checks, but complete
validation of that imported grammar remains the DT layer's responsibility.
