# openbim-loin implementation plan

Status: DT-backed domain boundary implemented; codec not started.
Last updated: 2026-08-25

This is task state, not ambient context. Follow `AGENTS.md`; claim one task ID,
record blockers/decisions under it, and check it off only with evidence.

## Established boundary

May consume openBIM core, ISO 23387 data-template, and XML codec contracts.
Namespace version handling is first-class and source namespaces remain observable.

## Work queue

- [x] `LOI-DT` — consume the released `openbim-dt` value and domain types at
  every ISO 7817-3 boundary that is typed by ISO 23387: GUIDs, multilingual
  text, references, concept inheritance, object/property/group/unit/dimension/
  quantity-kind/reference-document content. Prove this with compile-time public
  API tests and behavior tests; do not add a dependency that only supplies a
  namespace constant.
- [ ] `LOI-PORT` — port the lossless LOIN codec from the private implementation;
  verify provenance and keep standards schemas out of the published repository.
- [ ] `LOI-MIGRATE` — design explicit, lossless namespace migration after codec
  contracts exist.
- [ ] `LOI-CONFORMANCE` — add only redistributable fixtures and executable
  validation evidence before claiming standard conformance.

## Completion log

- Namespace constants and recognition helper are exercised by unit and doc tests.
- `openbim-dt 0.2` owns every ISO 23387 value and complex-type contract exposed by
  the LOIN domain subset; compile-time tests and a mutation probe reject
  lookalike replacement types.
