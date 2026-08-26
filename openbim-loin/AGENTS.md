# openbim-loin instructions

Purpose: canonical ISO 7817-3 / EN 17412-3 Level of Information Need package.

Follow `../AGENTS.md`. Read `PLAN.md` only for assigned implementation or roadmap
work; keep progress, blockers, and evidence there.

## Boundary

May consume released openBIM core, ISO 23387 data-template, and XML contracts.
Namespace handling must preserve observed input and make output choice explicit.
All LOIN implementation and public items live here.

## Status

Namespace constants, recognition, and DT-backed value, `ConceptType`, object-type,
alphanumerical-information, documentation, and registry-reference boundaries are
implemented and tested. Complete LOIN XML codec, migration, schema validation,
and lossless LOIN-document round trips are not implemented.
