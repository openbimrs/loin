# loin maintenance plan

Status: exact-version pure alias implemented and gated.
Last updated: 2026-08-25

This is task state, not ambient context. Follow `AGENTS.md`; claim one task ID,
record blockers/decisions under it, and check it off only with evidence.

## Established boundary

Exactly one meaningful source line: `pub use openbim_loin::*;`. The semantic and
mutation gates enforce package-version lockstep, target shape, dependency shape,
and absence of alias-owned implementation files.

## Work queue

- [ ] `ALI-LOIN` — keep release metadata synchronized with `openbim-loin`; add no
  independent API or behavior.

## Completion log

- Alias purity is part of the standalone CI gate.
