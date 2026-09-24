# openbim-loin implementation plan

Status: `LOI-PORT`, `LOI-MIGRATE`, and `LOI-CONFORMANCE` implemented; final gate/review in progress.
Last updated: 2026-08-26

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
- [x] `LOI-PORT` — implement a strict bounded namespace-aware LOIN XML syntax
  tree with semantic-lossless reading/writing. Preserve qualified names,
  namespace declarations, attribute/child order, text, CDATA, comments,
  processing instructions, empty-element style, and unknown content. Parsing and
  conformance validation remain separate.
- [x] `LOI-MIGRATE` — expose explicit 2022/2024 namespace targets. Migration
  rewrites only names whose resolved namespace is the observed LOIN namespace,
  preserves DT and foreign namespaces, retains the originally observed version,
  and returns a change report. Writing always requires an explicit namespace
  policy, including explicit preservation of the current syntax tree.
- [x] `LOI-CONFORMANCE` — implement typed clause-level structural diagnostics
  derived from the locally audited ISO 7817-3 XSD: exact root, declared LOIN
  children, sequence/cardinality, required/declared attributes, LOIN enums, and
  XML Schema lexical values at LOIN-owned boundaries. Imported ISO 23387 content
  remains owned by `openbim-dt`; do not claim complete XSD 1.1 validation.
  Synthetic fixtures only; restricted schemas and examples remain ignored and
  excluded from packages.

## Completion log

- Namespace constants and recognition helper are exercised by unit and doc tests.
- `openbim-dt 0.3` owns every ISO 23387 value and complex-type contract exposed by
  the LOIN domain subset; public API tests pin the DT type identities and
  compile-clean behavior mutations verify that imported values are retained.
- Strict XML tests cover XML 1.1/DTD/entity/prefix/QName/duplicate-attribute
  rejection, parser budgets, semantic controls, unknown syntax, and repeated
  parse/write/parse stability.
- Migration tests cover both draft namespaces, observed/current identity,
  LOIN-only rewrites, non-mutating targeted writes, reports, and collision
  refusal.
- Synthetic clause-level tests cover ISO 7817-3 sequences, cardinalities,
  georeferencing, geometry enumerations, XML Schema scalar lexemes, unknown
  extension retention, and the documented DT imported-type boundary.
- Independent XML and typed-model mutation matrices prove the safety, migration,
  collision, namespace-form, required-cardinality, sequence, enumeration, scalar,
  control-character, and typed-state gates.
