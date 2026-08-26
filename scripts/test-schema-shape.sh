#!/usr/bin/env bash
# Prove that typed ISO 7817-3 state is exercised by the DT contract tests.
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
model_rel=openbim-loin/src/model.rs
log=$(mktemp)
mutation_root="${CARGO_TARGET_DIR:-target}/loin-schema-mutations"
baseline=$(mktemp -d "${TMPDIR:-/tmp}/loin-schema-baseline-XXXXXX")
active_sandbox=""
python3 - "$repo_root" "$baseline/repo" <<'PY'
from pathlib import Path
import shutil, sys
shutil.copytree(Path(sys.argv[1]), Path(sys.argv[2]), ignore=shutil.ignore_patterns(".git", "target", ".hermes", "references", "*.xsd", "*.xsd.xml"))
PY
chmod -R u+w "$baseline/repo"
cleanup() {
    rm -rf "$baseline"
    if [[ -n "$active_sandbox" ]]; then
        rm -rf "$active_sandbox"
    fi
    rm -f "$log"
}
trap cleanup EXIT INT TERM

run_mutation() {
    local name=$1 old=$2 new=$3 test_name=$4
    local target="$mutation_root/$name"
    active_sandbox=$(mktemp -d "${TMPDIR:-/tmp}/loin-schema-mutant-XXXXXX")
    cp -a "$baseline/repo" "$active_sandbox/repo"
    local model="$active_sandbox/repo/$model_rel"
    python3 - "$model" "$old" "$new" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
old, new = sys.argv[2], sys.argv[3]
text = path.read_text(encoding="utf-8")
count = text.count(old)
if count != 1:
    raise SystemExit(f"schema mutation fixture count for {old!r}: {count}")
path.write_text(text.replace(old, new), encoding="utf-8")
PY
    if ! (cd "$active_sandbox/repo" && CARGO_TARGET_DIR="$target" cargo check -p openbim-loin --locked >"$log" 2>&1); then
        printf 'typed-model mutation did not compile: %s\n' "$name" >&2
        cat "$log" >&2
        exit 1
    fi
    printf 'mutation compile passed: %s (CARGO_TARGET_DIR=%s)\n' "$name" "$target"
    if (cd "$active_sandbox/repo" && CARGO_TARGET_DIR="$target" cargo test -p openbim-loin --test dt_contracts --locked "$test_name" >"$log" 2>&1); then
        printf 'typed-model mutation escaped tests: %s\n' "$name" >&2
        cat "$log" >&2
        exit 1
    fi
    printf 'mutation killed: %s\n' "$name"
    rm -rf "$active_sandbox"
    active_sandbox=""
}

run_mutation purpose-items 'self.items.push(value);' 'let _ = value;' typed_contract_represents_current_purpose_actor_and_alphanumerical_state
run_mutation purpose-non-empty '        if replacement.is_none() && self.items.iter().all(&matches) {' '        if false {' typed_contract_represents_current_purpose_actor_and_alphanumerical_state
run_mutation purpose-reference-document 'self.items.push(PurposeItem::ReferenceDocument(value));' 'let _ = value;' purpose_exposes_actual_dt_value_contracts
run_mutation actor-email 'self.email_address = value;' 'let _ = value;' typed_contract_represents_current_purpose_actor_and_alphanumerical_state
run_mutation alphanumerical-groups 'self.groups_of_properties = value;' 'let _ = value;' typed_contract_represents_current_purpose_actor_and_alphanumerical_state
run_mutation alphanumerical-group-member 'self.groups.push(value);' 'let _ = value;' typed_contract_represents_current_purpose_actor_and_alphanumerical_state
run_mutation specification-georeferencing 'self.geo_referencing = value;' 'let _ = value;' remaining_iso_7817_touchpoints_keep_dt_value_identity
run_mutation root-specifications 'self.specifications.push(specification);' 'let _ = specification;' remaining_iso_7817_touchpoints_keep_dt_value_identity
run_mutation document-form 'self.form = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation document-content 'self.content = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation document-reference 'pub fn add_reference_document(&mut self, value: Reference) {
        self.reference_documents.push(value);
    }
    #[must_use]
    pub fn reference_documents' 'pub fn add_reference_document(&mut self, value: Reference) {
        let _ = value;
    }
    #[must_use]
    pub fn reference_documents' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-placeholder 'self.placeholder = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-detail 'self.detail = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-dimensionality 'self.dimensionality = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-appearance 'self.appearance = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-parametric 'self.parametric_behaviour = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation geometry-location 'self.location = value;' 'let _ = value;' typed_contract_represents_current_document_and_geometry_state
run_mutation detail-dictionary 'self.dictionary = value;' 'let _ = value;' remaining_iso_7817_touchpoints_keep_dt_value_identity
run_mutation datum-registry-reference 'self.registry_reference = value;' 'let _ = value;' typed_contract_represents_current_georeferencing_state
run_mutation crs-kind '            crs_type,' '            crs_type: CoordinateReferenceSystemKind::NotRequired,' typed_contract_represents_current_georeferencing_state
run_mutation model-coordinate-projection '            is_projected,' '            is_projected: false,' typed_contract_represents_current_georeferencing_state
run_mutation georeferencing-crs 'self.coordinate_reference_system = value;' 'let _ = value;' typed_contract_represents_current_georeferencing_state
run_mutation georeferencing-model-coordinate 'self.model_coordinate_systems.push(value);' 'let _ = value;' typed_contract_represents_current_georeferencing_state

printf 'all typed-model mutations killed\n'
