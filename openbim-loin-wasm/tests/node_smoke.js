// Exercises the real wasm-bindgen package in Node. Proves the bindings run,
// not merely that they link.
const assert = require("node:assert");
const m = require(process.env.LOIN_WASM_PKG + "/openbim_loin_wasm.js");

const DT = "https://standards.iso.org/iso/23387/ed-2/en/";
const LOIN = "https://iso.org/2024/LOIN";
const G = "70000000-0000-0000-0000-000000000000";

function doc(inner) {
  return (
    `<loin:LevelOfInformationNeed xmlns:loin="${LOIN}" xmlns:dt="${DT}">` +
    `<Specification name="S" dt:GUID="${G}">${inner}</Specification>` +
    `</loin:LevelOfInformationNeed>`
  );
}

// 1. Missing dt:Name/@language must surface MissingLanguage, proving the
//    issue #2 fix is live inside the wasm module.
const bad = doc(
  `<SpecificationPerObjectType dt:GUID="${G}" dateOfCreation="2026-08-26T00:00:00Z">` +
  `<dt:Name/><ObjectType/></SpecificationPerObjectType>`
);
const diags = m.validate(bad);
assert(Array.isArray(diags), "validate returns an array");
const lang = diags.find((d) => d.code === "MissingLanguage");
assert(lang, "MissingLanguage diagnostic present");
assert.strictEqual(lang.severity, "Error");
assert(lang.path.endsWith("/Name[1]"), lang.path);

// 2. Round-trip through rewrite() must be byte-identical.
const good = doc(
  `<Prerequisites dt:GUID="${G}"><Purpose dt:GUID="${G}">` +
  `<Name language="en">P</Name></Purpose></Prerequisites>`
);
assert.strictEqual(m.rewrite(good), good, "round trip is byte-identical");
assert.strictEqual(m.isWellFormed(good), true);
assert.strictEqual(m.isWellFormed("<not-loin/>"), false);

// 3. A parse failure must throw a real JS Error, not return a sentinel.
assert.throws(() => m.validate("this is not xml"), /.*/);

console.log("wasm package OK:", diags.length, "diagnostics;",
  "round-trip", m.rewrite(good).length, "bytes");

// 4. A parse failure must stay MACHINE-READABLE: kind and position are
//    real properties, so an editor can place a marker without regexing
//    the message text.
let caught = null;
try { m.validate("<a><b></a>"); } catch (e) { caught = e; }
assert(caught, "malformed XML must throw");
assert.strictEqual(caught.name, "LoinParseError");
assert.strictEqual(typeof caught.kind, "string", "kind is exposed");
assert.strictEqual(typeof caught.position, "number", "position is exposed");
assert.strictEqual(caught.kind, "MalformedXml", caught.kind);

// 5. Diagnostic codes are an explicit mapping, not Rust Debug output.
//    Guard the exact spelling so a Rust-side rename cannot silently
//    change the JS contract.
assert.strictEqual(lang.code, "MissingLanguage");
assert.deepStrictEqual(
  Object.keys(lang).sort(),
  ["code", "message", "path", "severity"],
  "diagnostic shape is stable"
);

console.log("structured errors OK:", caught.kind, "@", caught.position);

// 6. Namespace migration: Draft2024 -> Draft2022 must rewrite names and
//    survive serialisation, so the result reparses in the NEW namespace.
const OLD_NS = "https://iso.org/2022/LOIN";
const mig = m.migrate(good, "Draft2022");
assert.strictEqual(mig.report.source, "Draft2024", JSON.stringify(mig.report));
assert.strictEqual(mig.report.target, "Draft2022");
assert(mig.report.changedNames > 0, "migration changed element names");
assert(mig.xml.includes(OLD_NS), "output declares the target namespace");
assert(!mig.xml.includes(LOIN), "output drops the source namespace");
assert(m.isWellFormed(mig.xml), "migrated output is still a LOIN document");

// Migrating to the version already in use is a no-op, but still reports.
const same = m.migrate(good, "Draft2024");
assert.strictEqual(same.report.changedNames, 0, "no-op changes nothing");
assert.strictEqual(same.report.changedDeclarations, 0);

// An unknown target must throw TypeError, never silently default.
assert.throws(() => m.migrate(good, "Draft1999"), TypeError);

console.log(
  "migration OK:", mig.report.source, "->", mig.report.target,
  "names=" + mig.report.changedNames,
  "decls=" + mig.report.changedDeclarations
);
