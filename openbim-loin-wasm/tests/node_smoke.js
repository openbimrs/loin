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
