import argparse
import datetime
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("candidate")
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
continuation = root / "continuation"
candidate_path = root / args.candidate
candidate = json.loads(candidate_path.read_text(encoding="utf-8"))
assert not candidate["issues"]
assert not candidate["registries_changed_during_snapshot"]
for pin in candidate["input_pins"].values():
    assert hashlib.sha256((root / pin["record"]).read_bytes()).hexdigest() == pin["sha256"], pin
base_pin = candidate["input_pins"]["base"]
base = json.loads((root / base_pin["record"]).read_text(encoding="utf-8"))
by_id = {row["source_id"]: row for row in base["files"]}
overlay_path = continuation / "owner-range-extensions.jsonl"
overlay_bytes = overlay_path.read_bytes()
overlays = [json.loads(line) for line in overlay_bytes.decode("utf-8").splitlines()]
assert len(overlays) == 22
destination = continuation / "cumulative-source-scope-producer-followup-008.json"
receipt_path = continuation / "root/producer-scope-reconciliation-024.json"
assert not destination.exists() and not receipt_path.exists()
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
new_sources = []
new_overlays = []
for row in candidate["source_variant_dispositions"]:
    assert row["identity_validation"] == "verified_frozen_bytes"
    role = {
        "authority": "continuation/root/producer-scope-extension-authority-022.json",
        "candidate_record": args.candidate,
        "registered_before_body_analysis": row["registration_observations"],
        "effective_local_material_ranges": row["effective_local_material_ranges"],
        "qualification_refs": row["qualification_refs"],
        "scope_limits": row["role_preservation"] + " " + row["scope_removal"],
        "producer_discovery": "separate actual source/recipient packet acceptance required",
        "owner_fit": "pending exact new-role delta against owner019",
        "source_or_other_lens_credit": "none from scope bookkeeping",
    }
    if not row["baseline_source_exists"]:
        assert row["source_id"] not in by_id
        source = {
            "source_id": row["source_id"],
            "snapshot": row["snapshot"],
            "path": row["resolved_path"],
            "sha256": row["sha256"],
            "bytes": row["bytes"],
            "actual_line_count": row["actual_EOF"],
            "material_ranges": row["effective_local_material_ranges"],
            "scope_kind": "materially_reached_producer_followup_contract",
            "audit_owner": row["proposed_audit_owner"],
            "readonly_suppliers": row["read_only_suppliers"],
            "producer_followup_scope_additions": [role],
            "owner_fit_delta_status": "pending exact new-role comparison",
        }
        base["files"].append(source)
        by_id[row["source_id"]] = source
        new_sources.append(row["source_id"])
    else:
        source = by_id[row["source_id"]]
        assert source["sha256"] == row["sha256"]
        source.setdefault("producer_followup_scope_additions", []).append(role)
        extension = {
            "id": f"CONT-OWNER-RANGE-EXTENSION-{23 + len(new_overlays):03}",
            "at": now,
            "source_id": row["source_id"],
            "sha256": row["sha256"],
            "material_ranges": row["effective_local_material_ranges"],
            "audit_owner": row["proposed_audit_owner"],
            "reason": "Exact prospectively registered material range/role reconciliation; byte overlap never substitutes semantic role acceptance.",
            "new_byte_ranges": row["new_byte_ranges_vs_scope007_plus_overlay022"],
            "scope_registration": role,
            "producer_followup": "separate actual source/recipient packet acceptance required",
            "owner_fit": "pending exact new-role delta against owner019",
        }
        new_overlays.append(extension)
assert len(base["files"]) == candidate["counts"]["candidate_total_source_variants"]
assert len(new_sources) == candidate["counts"]["new_source_variants"]
base["generated_at"] = now
base["status"] = "Global exact producer followup scope reconciled; semantic producer acceptance and new owner delta remain separate audit-ledger gates."
base["previous_pinned_scope"] = base_pin
base["current_producer_registry_reconciliation"] = {
    "candidate_record": args.candidate,
    "candidate_sha256": hashlib.sha256(candidate_path.read_bytes()).hexdigest(),
    "input_pins": candidate["input_pins"],
    "all_original_registration_rows": candidate["original_registry_rows"],
    "local_qualifications": candidate["qualifications_preserved"],
    "counts": candidate["counts"],
    "limits": "Local qualifications narrow only local supplier claims, never original global scope. Every role retained independently; no project execution or semantic credit from bytes. Source freeze remains.",
}
new_overlay_bytes = overlay_bytes + b"".join(
    (json.dumps(row, ensure_ascii=False) + "\n").encode("utf-8") for row in new_overlays
)
receipt = {
    "id": "CONT-ROOT-PRODUCER-SCOPE-RECONCILIATION-024",
    "at": now,
    "candidate": args.candidate,
    "candidate_sha256": hashlib.sha256(candidate_path.read_bytes()).hexdigest(),
    "base_scope": base_pin,
    "new_source_ids": new_sources,
    "new_overlay_ids": [row["id"] for row in new_overlays],
    "total_variants": len(base["files"]),
    "total_overlays": 22 + len(new_overlays),
    "counts": candidate["counts"],
    "semantic_coverage": "none from this reconciliation; exact actual packets and owner delta required",
    "source_edits": 0,
    "project_execution": 0,
}
destination.write_text(json.dumps(base, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
overlay_path.write_bytes(new_overlay_bytes)
receipt["scope_sha256"] = hashlib.sha256(destination.read_bytes()).hexdigest()
receipt["overlay_sha256"] = hashlib.sha256(new_overlay_bytes).hexdigest()
receipt_path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"scope": str(destination), "variants": len(base["files"]), "overlays": 22 + len(new_overlays), "semantic_credit": False}))
