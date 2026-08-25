# loin instructions

Purpose: pure alias so the standard is reachable under its common package name.

Follow `../AGENTS.md`. Read `PLAN.md` only for assigned implementation or roadmap
work; keep progress, blockers, and evidence there.

## Boundary

`src/lib.rs` contains only `pub use openbim_loin::*;`. Defining an item, feature,
extra target, conditional dependency, or implementation file here is a defect.
The canonical dependency remains an exact-version, unconditional sibling path.

## Status

Published to reserve the name and mechanically gated as a pure alias.
