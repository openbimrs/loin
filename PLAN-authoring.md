# Authoring layer for openbim-loin (issue #1)

Status: writer landed; papercuts in progress

## Goal

Make LOIN documents *writable*, not just readable. Issue #1 asks for a public
conversion `LevelOfInformationNeed -> LoinDocument`, with a round trip that
re-validates clean and preserves `xs:sequence` child order.

## Constraints discovered by inspection

1. `XmlElement::parsed` / `XmlElement::push` / `LoinDocument::parsed` are all
   `pub(crate)`; `XmlElement` has no `nodes_mut`/`attributes_mut`. So today no
   downstream serializer is possible. (Confirmed: `document.rs`.)
2. `model.rs` has zero references to `LoinDocument`/`XmlElement`. No bridge.
3. **Blocking discovery:** `openbim-dt 0.2.0` cannot serialize its owned domain
   types. `dt::Element`'s constructors are all `pub(crate)`, and
   `Property`/`ObjectType`/`GroupOfProperties`/`ReferenceDocument`/`Unit`/
   `Dimension`/`QuantityKind` expose no `to_element`/`to_xml`.
   `TypedElement::into_element` only unwraps an element that was *parsed*.
   => LOIN cannot emit DT-owned subtrees such as the `<ObjectType>` that
   `SpecificationPerObjectType` requires (`SPEC_PER_OBJECT` = one ObjectType).

## Decision

Do **not** duplicate ISO 23387 element shapes inside LOIN to work around (3) —
that would fork DT's schema into this repo and violate the boundary in
`AGENTS.md` ("IFC, core, codec, and data-template crates must never depend on
LOIN"; DT owns its own contracts).

Split the work:

- **This repo, now:** author everything LOIN *owns* — the grammar-ordered
  LOIN element tree, attributes (dt:GUID, name, language, dateOfCreation),
  and simple-content children. Emit DT-owned subtrees only where the typed
  model already carries a parsed `XmlElement` we can clone, otherwise report
  a precise, typed refusal rather than inventing DT syntax.
- **openbimrs/dt, next:** add owned-type serialization (`to_element`) so the
  remaining DT subtrees become writable. Then LOIN's refusal cases disappear.

Ordering is derived from the existing `ChildRule` tables in `validation.rs`
(single source of truth) — the writer must *consume* those tables, never
restate the order, or the two drift.

## Work queue

- [x] Expose a minimal public construction API on `XmlElement`/`LoinDocument`
- [x] `src/authoring.rs`: `LevelOfInformationNeed -> LoinDocument`
- [x] Typed `AuthoringError` enum naming the refused element and reason
- [x] Round-trip test: model -> document -> xml -> reparse -> `validate()` empty
- [x] Order test: Prerequisites children follow the declared `xs:sequence`
- [x] Getters for write-only types (closes #8 for the types the writer needs)
- [ ] Fix `Purpose::replace_optional_item` reordering (#5)
- [ ] Mutation probes for each refusal + the ordering step
- [ ] README/rustdoc/CHANGELOG updated together
- [ ] `./scripts/gate.sh` exit 0

## Verified findings

1. **DT owned types cannot be serialized downstream.** Proved by compiling
   `object_type.to_element()`: `no method named 'to_element' found for struct
   'openbim_dt::ObjectType'`. `dt::Element`'s constructors are `pub(crate)`,
   and DT has no `from_element` either — its owned types are XML-isolated in
   both directions. So `<ObjectType>` and friends are refused, not faked.

2. **ISO 7817-3 is `elementFormDefault="unqualified"`.** Only the root carries
   the LOIN namespace; every schema-local child is in *no* namespace
   (`validation.rs`: "schema-local element {} must be unqualified").

3. **The LOIN namespace must be bound to a prefix, never declared as the
   default.** A default `xmlns=` declaration is inherited by unprefixed
   children on reparse, which silently invalidates the document. The
   in-memory tree validated but the reparsed one did not — caught only by the
   serialize->reparse->validate test, which is why that test matters.

## Verification

```bash
./scripts/gate.sh          # decides on exit code
```
