# 0001 — Browser bindings live in a separate crate, not behind a feature

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Friedrich Schrödter
- **Supersedes:** —

## Context

`openbim-loin` must be usable from JavaScript (browser editors, Node tooling).
`wasm-bindgen` output requires `crate-type = ["cdylib"]`, and the bindings pull
in `wasm-bindgen`, `js-sys`, `serde` and `serde-wasm-bindgen`.

## Decision

We will ship the bindings as `openbim-loin-wasm`, a separate workspace package
that depends on `openbim-loin` at an exact version. The core crate stays free of
any wasm or JS dependency, and the gate asserts its exact dependency set.

## Alternatives considered

| Option | Why not |
| --- | --- |
| `wasm` feature on `openbim-loin` | `crate-type` is per-package, not per-feature: every native build would emit a cdylib. Feature unification would force the JS dependencies on any workspace where one member enabled it. |
| Hand-written `extern "C"` exports in the core | Workable (a probe ran in Node) but every consumer re-implements marshalling, and diagnostics cross as pointers rather than typed objects. |

## Consequences

**Positive**

- Native consumers never compile or audit wasm-bindgen.
- The JS surface (structured errors, typed `.d.ts`) evolves without touching the core API.

**Negative / costs**

- A third package to version and publish; mitigated by lockstep releases (ADR 0002).
- Stable JS spellings of codes need an explicit mapping in the bindings.

**Follow-ups / risks to watch**

- A `LoinDoc` handle to avoid re-parsing is parked in issue #12.

## Relation to existing code

`openbim-loin-wasm/`, `scripts/test-wasm-package.sh`, the dependency-set assertion in `scripts/gate.sh`.
