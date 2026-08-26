# LOIN XML codec and validation

`openbim-loin` provides two deliberately separate layers:

1. `LoinDocument` is a strict, bounded, namespace-aware XML syntax tree.
2. `LoinDocument::validate()` produces ISO 7817-3 clause-level diagnostics.

Keeping these layers separate means malformed XML is rejected, while well-formed
but schema-invalid documents can still be inspected, migrated, repaired, and
round-tripped without dropping unknown content.

## Decode and encode

```rust
use openbim_loin::{LoinDocument, OutputNamespace};

let xml = r#"<l:LevelOfInformationNeed
    xmlns:l="https://iso.org/2024/LOIN" />"#;
let document = LoinDocument::parse(xml)?;
let encoded = document.to_xml_string(OutputNamespace::Preserve)?;
assert_eq!(LoinDocument::parse(&encoded)?.root(), document.root());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The decoder rejects XML 1.1, DTDs, undeclared entities and prefixes, malformed
QNames and namespace bindings, duplicate expanded attributes, multiple roots,
and configurable byte, depth, node, and per-element attribute budget overruns.
`ParseOptions` controls those limits.

## Lossless-semantic scope

The syntax tree retains:

- the XML declaration;
- prolog and epilog comments and processing instructions;
- lexical qualified names and resolved namespace URIs;
- namespace declarations and attribute order;
- element/character-data/CDATA/comment/processing-instruction order;
- empty-element style;
- semantically represented TAB/LF/CR attribute values and CR text values;
- unknown elements, attributes, namespaces, and content.

The guarantee is **semantic**, not byte-for-byte. Quote choice, entity spelling,
and equivalent escaping may be normalized by the writer. Repeated
parse/write/parse cycles preserve the owned syntax-tree semantics.

## Namespace migration

Both known draft bindings are explicit values:

```rust
use openbim_loin::{LoinDocument, NamespaceVersion, OutputNamespace};

# let xml = r#"<l:LevelOfInformationNeed xmlns:l="https://iso.org/2024/LOIN" />"#;
let source = LoinDocument::parse(xml)?;
let (migrated, report) = source.migrated(NamespaceVersion::Draft2022)?;
assert_eq!(report.changed_names(), 1);
let xml_2022 = migrated.to_xml_string(OutputNamespace::Preserve)?;
# let _ = xml_2022;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Migration rewrites only expanded names and namespace declarations bound to the
observed/current LOIN URI. Imported ISO 23387 and extension namespaces are not
rewritten. The original observed edition remains available through
`observed_namespace()`, while `current_namespace()` reports the transformed
syntax tree. Migration fails closed if rewriting would create duplicate expanded
attribute names.

Every write requires an `OutputNamespace`: preserve the document's current
bindings or explicitly target a known edition. There is no implicit upgrade.

## Validation coverage

`validate()` is **XSD-derived structural and lexical validation**, not a complete
W3C XML Schema implementation. It currently checks:

- ISO 7817-3 root, ordered sequences, repeating choices, effective
  cardinalities, and required-attribute rules;
- the prerequisite, per-object, alphanumerical, documentation, geometry, and
  georeferencing structures declared by the audited draft schema;
- all ISO 7817-3 enumeration values;
- `xs:boolean`, `xs:decimal`, `xs:double`, `xs:dateTime`, `xs:language`,
  the current actor email restriction, and DT-owned `GUID` lexical values at
  LOIN-owned boundaries;
- the effective inherited `dt:GUID`, `dateOfCreation`, and optional `dt:about`
  attributes on `SpecificationPerObjectType`, plus the declared immediate
  DT-qualified inherited Concept choice and its minimum cardinality;
- `xsi:nil` empty-content semantics and globally permitted schema-location
  attributes; unsupported `xsi:type` derivation is diagnosed rather than guessed;
- unqualified local-element semantics required by the audited schema;
- unknown content as errors, including foreign extensions because the audited
  grammar declares no wildcard, while retaining that syntax in the document tree.

Imported ISO 23387 complex-type elements are retained and treated as structurally
opaque at this validation boundary. DT-namespaced `GUID` and `referenceURI`
attributes encountered inside them are lexically checked, but this crate does not
claim recursive or complete XSD validation of their internals. The 2022 namespace
uses the same explicitly documented compatibility profile; validation is not
claimed as schema-backed for that draft.
`openbim-loin -> openbim-dt` remains the dependency direction and DT continues to
own those contracts.

No ISO/CEN schema, annex example, or other restricted standards artifact is
vendored or packaged. Public tests use synthetic documents.
