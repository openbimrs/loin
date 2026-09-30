# 0005 — Publish the grammar as a read-only view of the validator tables

- **Status:** Accepted
- **Date:** 2026-09-30
- **Deciders:** Friedrich Schrödter
- **Supersedes:** —

## Context

Issue #10: the only machine-readable description of the LOIN grammar was the
private `const` tables and `match` arms in `openbim-loin/src/validation.rs`.
A browser authoring tool had to reverse-engineer qualification rules, child
order and attribute asymmetries (`dateOfCreation` unqualified next to
`dt:GUID`; the milestone's PascalCase `Date`) from validator source.

Forces:

- Any published description must not drift from what `validate()` enforces.
  A hand-written schema next to the validator would.
- The ISO XSD is not redistributable (see `docs/architecture.md`), so we cannot
  simply ship it.
- `openbim-loin` must not gain dependencies (the gate pins its set), so no
  `serde` in the core.
- The validator is heavily mutation-tested by textual anchors
  (`scripts/test-xml-capability.py`). Rewriting it to be table-driven end to
  end would move ~30 anchors in the same change.
- Non-Rust consumers (JS editors, Python tooling) need a file, not an API.

## Decision

We will expose the grammar in three forms, all produced from one source:

1. **Rust API** `openbim_loin::grammar`: `elements()`, `element(parent, name)`,
   `global_attributes()`, `to_json()`, and read-only types (`Element`,
   `ChildRule`, `Attribute`, `Content`, `ValueType`, `AttributeNamespace`).
   Enums are `#[non_exhaustive]`; nothing is constructible outside the crate.
2. **JSON artifact** `openbim-loin/loin-grammar.json` (`formatVersion` 1),
   rendered by `grammar::to_json()`, committed, and shipped in the crate.
   `tests/grammar.rs` fails when the committed file differs from the render.
3. **WASM export** `grammar()` in `@openbim/loin`, returning the same bytes;
   the Node smoke test compares it to the committed file.

How each part stays tied to the validator:

| Grammar part | Tie to the validator |
| --- | --- |
| Element set, `(parent, name)` keys, children, cardinalities, sequence vs. choice, imported ISO 23387 elements | **Derived**: walked from the root over `content_rule`, `is_imported_dt_complex` and the `ChildRule` tables. `ChildRule` is the validator's own type, re-exported. |
| Enumeration values | **Derived**: `enumeration_values`, which the validator itself calls; the sets live on the model enums. |
| Inherited ISO 23387 Concept children | **Derived**: `INHERITED_DT_CONCEPT_CHILDREN`. |
| Attributes (namespace, required, value type) and simple-content value types | **Declared in `grammar.rs` and checked**: exhaustive unit tests call the validator's `validate_attributes` / `validate_lexical_content` for every element × every attribute in the universe, and require accepted ⇔ declared, missing required ⇒ diagnostic, invalid lexeme ⇒ the type's `DiagnosticCode`, and plain strings unchecked. |

The gate additionally runs `scripts/check-grammar-artifact.py`, an independent
Python walker that reads only the JSON and checks every shipped example, then
rejects seeded mutants. Two new mutations in `test-xml-capability.py` prove a
validator-table change makes the artifact stale.

## Alternatives considered

| Option | Why not |
| --- | --- |
| Hand-written JSON Schema / RELAX NG next to the validator | Two sources that drift; nothing forces agreement. |
| Ship the ISO XSD | Not redistributable. |
| Make the validator fully table-driven now (attributes and scalar types too) | Right end state, but moves most mutation anchors in the same change as a new public API; the exhaustive agreement tests give the same no-drift guarantee today. Recorded as a follow-up. |
| JSON only, no Rust API | Rust consumers would parse our own JSON; the API is also what generates the file. |
| Rust API only | Leaves non-Rust tools reading source, which is the issue. |
| `serde` derive on grammar types | Adds a dependency to the pinned core set for ~100 lines of deterministic writer. |
| Include the crate version in the JSON | Every release would make the artifact stale without a grammar change. `formatVersion` versions the description; the grammar travels with the crate version that ships it. |

## Consequences

**Positive**

- Third-party tools can target the format from a file; the issue's
  asymmetries are explicit (`namespace`, exact-case `name`, `required`).
- A validator change that alters the grammar cannot land without regenerating
  and reviewing `loin-grammar.json` (`LOIN_BLESS_GRAMMAR=1`).
- The grammar itself is test data: a test generates a minimal document from the
  grammar alone and requires `validate()` to accept it.

**Negative / costs**

- New public API (semver-relevant). Kept minimal and read-only; enums are
  non-exhaustive so the description can grow.
- Attribute and scalar-type facts live in two places (validator code and
  `grammar.rs`), bound by tests rather than by construction.
- The grammar is one level context-sensitive, exactly like the validator: it
  keys on the parent's local name, not a full path. Consumers must do the same.
- The artifact lists element and attribute names, cardinalities and
  enumeration tokens derived from the draft schema. That is the same
  information the MIT-licensed validator source already publishes; it carries
  no schema text, annotations or documentation.

**Follow-ups / risks to watch**

- Move `validate_attributes` and `validate_lexical_content` onto the grammar
  tables so attributes are derived too; update the mutation anchors with it.
- ISO 23387 content (`Content::Imported`) is out of scope; `openbim-dt` owns it
  and could publish its own grammar the same way.
- If the XSD is revised, the artifact diff is the reviewable changelog of the
  grammar.

## Relation to existing code

`openbim-loin/src/grammar.rs`, `openbim-loin/src/validation.rs` (`ChildRule`,
`content_rule`, `is_imported_dt_complex`, `enumeration_values`),
`openbim-loin/loin-grammar.json`, `openbim-loin/tests/grammar.rs`,
`openbim-loin-wasm/src/lib.rs` (`grammar`), `openbim-loin-wasm/tests/node_smoke.js`,
`scripts/check-grammar-artifact.py`, `scripts/test-xml-capability.py`,
`scripts/gate.sh`, `scripts/check-evidence.py`.
