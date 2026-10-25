import datetime
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
continuation = root / "continuation"
scope_path = continuation / "cumulative-source-scope-producer-followup-008.json"
baseline_path = continuation / "cumulative-source-scope-owner-delta-006.json"
scope = json.loads(scope_path.read_text(encoding="utf-8"))
baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
before = {row["source_id"]: row for row in baseline["files"]}
current = {row["source_id"]: row for row in scope["files"]}
camera_path = continuation / "camera/owner-reached-producer-followup/proposed-owner-fit-delta-queue-001.json"
camera = json.loads(camera_path.read_text(encoding="utf-8"))
camera_ids = {row["source_id"] for row in camera["proposed_sources"]}
rig_path = continuation / "rig/owner-reached-producer-followup/additional-sources.jsonl"
rig_ids = {json.loads(line)["source_id"] for line in rig_path.read_text(encoding="utf-8").splitlines()}
native_path = continuation / "native/root-roof-producer-followup/coverage.jsonl"
native_ids = {json.loads(line)["source_id"] for line in native_path.read_text(encoding="utf-8").splitlines()}
overlays_path = continuation / "owner-range-extensions.jsonl"
overlays = [json.loads(line) for line in overlays_path.read_text(encoding="utf-8").splitlines()]
extensions = [row for row in overlays if int(row["id"].rsplit("-", 1)[1]) >= 18]
new_ids = set(current) - set(before)
ids = new_ids | {row["source_id"] for row in extensions} | camera_ids
queue = []
for sid in sorted(ids):
    source = current[sid]
    eligible = [owner for owner, group in [("camera", camera_ids), ("rig", rig_ids), ("native", native_ids)] if sid in group]
    assert len(eligible) == 1, (sid, eligible)
    owner = eligible[0]
    queue.append({
        "id": f"CONT-PRODUCER-REACHED-OWNER-{len(queue) + 1:03}",
        "source_id": sid,
        "sha256": source["sha256"],
        "readonly_supplier": owner,
        "repair_ownership": "Existing authoritative whole-function claims remain unchanged.",
        "classification": "new_source_variant_since_owner019" if sid in new_ids else "existing_source_new_material_ranges_or_roles",
        "current_source_scope_record": source,
        "new_range_or_role_overlays": [row for row in extensions if row["source_id"] == sid],
        "previous_owner_scope_navigation_only": before.get(sid),
        "current_camera_proposed_roles": [row for row in camera["proposed_sources"] if row["source_id"] == sid],
        "obligation": "Compare exact current meaning, units, frame, provenance/authority, sign/bounds/zero, admission policy and actual producer/recipient with actual existing canonical definitions. Distinguish exact match, reusable numeric leaf with distinct role, missing owner and unresolved contract. Prior verdict, same bytes and metadata ranges alone do not establish fit.",
        "limits": "Only newly reached producer roles and registered qualifications; original accepted owner2483 plus17 overlays remain exact prior evidence. No whole unrelated function/file expansion. Source freeze and one lens remain in force.",
        "discovery": "not_started",
        "repair": "frozen_initial_all13",
        "validation": "not_established",
    })
assert new_ids <= {row["source_id"] for row in queue}
def pin(path):
    return {"record": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
result = {
    "id": "CONT-ROOT-PRODUCER-REACHED-OWNER-QUEUE-026",
    "at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "kind": "metadata_only_exact_delta_queue; activation requires separate producer acceptance and lens switch",
    "baseline": pin(baseline_path),
    "accepted_baseline": pin(continuation / "root/owner-inventory-acceptance-019.json"),
    "current_scope": pin(scope_path),
    "range_overlay": pin(overlays_path),
    "supplier_navigation": [pin(camera_path), pin(rig_path), pin(native_path)],
    "counts": {"new_source_variants": len(new_ids), "existing_source_new_roles": len(ids - new_ids), "total_obligations": len(queue), "by_supplier": {n: sum(row["readonly_supplier"] == n for row in queue) for n in ["camera", "rig", "native"]}},
    "queue": queue,
    "metadata_coverage_credit": False,
    "no_repair_or_function_authority_transferred": True,
    "source_edits": 0,
}
destination = continuation / "producer-reached-owner-queue-026.json"
assert not destination.exists()
destination.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["counts"]))
