# openbim-loin instructions

Purpose: canonical ISO 7817-3 / EN 17412-3 Level of Information Need package.

Follow `../AGENTS.md`. Read `PLAN.md` only for assigned implementation or roadmap
work; keep progress, blockers, and evidence there.

## Boundary

May consume released openBIM core, ISO 23387 data-template, and XML contracts.
Namespace handling must preserve observed input and make output choice explicit.
All LOIN implementation and public items live here.

## Status

Namespace constants and recognition are implemented and tested. XML codec,
migration, validation, and lossless round-trip are not implemented.
