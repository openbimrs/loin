# openbim-loin implementation plan

Status: namespace contracts implemented; codec not started.
Last updated: 2026-08-25

This is task state, not ambient context. Follow `AGENTS.md`; claim one task ID,
record blockers/decisions under it, and check it off only with evidence.

## Established boundary

May consume openBIM core, ISO 23387 data-template, and XML codec contracts.
Namespace version handling is first-class and source namespaces remain observable.

## Work queue

- [ ] `LOI-PORT` — port the lossless LOIN codec from the private implementation;
  verify provenance and keep standards schemas out of the published repository.
- [ ] `LOI-MIGRATE` — design explicit, lossless namespace migration after codec
  contracts exist.
- [ ] `LOI-CONFORMANCE` — add only redistributable fixtures and executable
  validation evidence before claiming standard conformance.

## Completion log

- Namespace constants and recognition helper are exercised by unit and doc tests.
