"""Publish manually reviewed initial-audit scope bookkeeping, never analysis.

Input is an ignored JSON plan authored after actual source comparison. This
utility only preserves registration provenance, immutable scope snapshots,
pending lens obligations, and pointers. It runs no project commands.
"""
import copy
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

C = Path(__file__).resolve().parents[1]
P = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
NOW = datetime.now(timezone.utc).isoformat()


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write(path, obj):
    path.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")


def pin(path):
    return {
        "record": "continuation/" + path.relative_to(C).as_posix(),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


previous = C / P["previous_scope"]
destination = C / P["next_scope"]
reconciliation = C / P["reconciliation_record"]
admission = C / P["admission_record"]
assert admission.exists(), "Actual manual admission proof must already exist"
assert not destination.exists() and not reconciliation.exists()
scope = read(previous)
overlay_path = C / "owner-range-extensions.jsonl"
old_overlay = overlay_path.read_bytes()
old_count = len(old_overlay.splitlines())
assert old_count == P["expected_previous_overlays"]
mandatory_qualifications = copy.deepcopy(read(
    C / "bounded01-02-newmaterial-queue-087.json"
).get("mandatory_scope_qualifications", []))
backup = C / P["overlay_backup"]
assert not backup.exists()
selected = []
inputs = []
for selection in P["registries"]:
    path = C / selection["path"]
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    captured = []
    for number, row in enumerate(rows, 1):
        registration_id = row.get("id", row.get("registration_id"))
        if registration_id not in selection["ids"]:
            continue
        captured.append(registration_id)
        ranges = selection.get("effective_ranges", {}).get(
            registration_id, row.get("material_ranges", row.get("ranges"))
        )
        assert ranges
        selected.append({
            "supplier": selection["supplier"],
            "registration": dict(pin(path), row_1based=number,
                                 id=registration_id, original_record=row),
            "ranges": ranges,
            "qualification": selection["qualifications"][registration_id],
        })
    assert set(captured) == set(selection["ids"])
    inputs.append(dict(pin(path), captured_rows=len(rows), selected_ids=captured))

new_variants = []
new_overlays = []
for selected_role in selected:
    registration = selected_role["registration"]
    row = registration["original_record"]
    sid = row["source_id"]
    existing = next((x for x in scope["files"] if x["source_id"] == sid), None)
    if existing is None:
        path = Path(row.get("resolved_path", P.get("source_paths", {}).get(sid, "")))
        assert path.is_absolute() and path.exists(), "Supply exact frozen source path"
        assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
        existing = {
            "source_id": sid, "sha256": row["sha256"],
            "path": str(path), "snapshot": row.get("snapshot"),
            "actual_line_count": row.get("actual_EOF"),
            "scope_kind": "materially_reached_initial_audit_contract",
            "material_ranges": selected_role["ranges"],
            "audit_owner": selected_role["supplier"],
            "scope_registration": [registration],
            "scope_limits": selected_role["qualification"],
            "source_lens_credit": "none from scope metadata",
        }
        scope["files"].append(existing)
        new_variants.append(existing)
    else:
        assert existing["sha256"] == row["sha256"]
        new_overlays.append({
            "id": f"CONT-OWNER-RANGE-EXTENSION-{old_count + len(new_overlays) + 1:03}",
            "at": NOW, "source_id": sid, "sha256": row["sha256"],
            "material_ranges": selected_role["ranges"],
            "audit_owner": selected_role["supplier"],
            "reason": row.get("reason", row.get("material_caller", row.get("direct_role"))),
            "scope_registration": [registration],
            "limits": row.get("exclusions", row.get("scope_limits")),
            "source_qualifications": selected_role["qualification"],
        })
    selected_role["source_scope_pointer"] = "/files/" + str(
        next(i for i, x in enumerate(scope["files"]) if x["source_id"] == sid)
    )

backup.write_bytes(old_overlay)
with overlay_path.open("a", encoding="utf-8") as stream:
    for row in new_overlays:
        stream.write(json.dumps(row) + "\n")
total_overlays = old_count + len(new_overlays)
information = {
    "previous_scope": pin(previous), "new_source_ids": [x["source_id"] for x in new_variants],
    "new_variants": len(new_variants), "new_overlays": len(new_overlays),
    "total_overlays": total_overlays, "registered_roles": len(selected),
    "input_pins": inputs,
    "mandatory_scope_qualifications": mandatory_qualifications,
    "preserved_old_overlay_prefix": dict(pin(backup), through=old_count),
    "limits": P["limits"],
}
scope["generated_at"] = NOW
scope[P["scope_increment_key"]] = information
write(destination, scope)
write(reconciliation, dict(
    id=P["reconciliation_id"], at=NOW, scope=pin(destination),
    previous_scope=pin(previous), new_variants=new_variants, new_overlays=new_overlays,
    total_variants=len(scope["files"]), total_overlays=total_overlays,
    mandatory_scope_qualifications=mandatory_qualifications,
    registered_rows=len(selected), input_pins=inputs,
    preserved_old_overlay_prefix=information["preserved_old_overlay_prefix"], limits=P["limits"],
))

queue_path = C / "bounded01-02-newmaterial-queue-087.json"
queue = read(queue_path)
for row in selected:
    registration = row["registration"]
    matching = next((x for x in new_overlays if x["scope_registration"][0]["id"] == registration["id"]), None)
    queue["queue"].append({
        "delta_index": len(queue["queue"]), "supplier": row["supplier"],
        "registration_ref": registration, "original_registration": registration["original_record"],
        "effective_material_ranges": row["ranges"],
        "mandatory_role_qualifications": [row["qualification"]],
        "source_scope_pointer": row["source_scope_pointer"],
        "mandatory_overlay": matching["id"] if matching else None,
        "producer01": "INCOMPLETE_NO_SEMANTIC_CREDIT", "owners02": "INCOMPLETE_NO_SEMANTIC_CREDIT",
    })
queue["counts"][P["queue_count_key"]] = len(selected)
queue["counts"]["total"] = len(queue["queue"])
queue["scope"] = pin(destination)
queue["updated_at"] = NOW
queue["instructions"] = P["queue_instructions"]
write(queue_path, queue)

authority_path = C / "root/initial04-invariant-operations/scope-authority-001.json"
authority = read(authority_path)
authority["scope"] = pin(destination)
authority["mandatory_overlays"] = dict(pin(overlay_path), through=total_overlays)
authority[P["scope_increment_key"]] = pin(reconciliation)
for row in selected:
    original = row["registration"]["original_record"]
    sid = original["source_id"]
    existing = next((x for x in authority["rows"] if x["source_id"] == sid), None)
    if existing is None:
        existing = {"source_id": sid, "source_sha256": original["sha256"],
                    "scope_pointer": row["source_scope_pointer"], "evidence": []}
        authority["rows"].append(existing)
    existing["status"] = "INCOMPLETE_NEW_MATERIAL_ROLE_REQUIRES_ACTUAL04_RECONCILIATION"
write(authority_path, authority)

progress_path = C / "root/admission03-progress-067.json"
progress = read(progress_path)
progress.update(P["progress_updates"])
progress.update(at=NOW, source_scope=pin(destination), mandatory_overlays=total_overlays,
                bounded01_02_queue=pin(queue_path))
progress["semantic_parallel_admission_acceptances"].append(pin(admission))
write(progress_path, progress)
for path in [C.parent / "records/execution-ledger.json",
             C.parent / "records/final-acceptance-20261021/audit-ledger.json"]:
    ledger = read(path)
    ledger["updated_at"] = NOW
    ledger["continuation"]["current_progress"] = pin(progress_path)
    ledger["continuation"][P["ledger_increment_key"]] = pin(reconciliation)
    if "current_cumulative_scope" in ledger:
        ledger["current_cumulative_scope"] = dict(
            pin(destination), variants=len(scope["files"]),
            range_extensions=dict(pin(overlay_path), mandatory_overlay=True, through=total_overlays),
            registry_reconciliation=pin(reconciliation), limits=P["limits"],
            mandatory_scope_qualifications=mandatory_qualifications,
        )
    if "next_action" in ledger:
        ledger["next_action"] = P["next_action"]
    write(path, ledger)
print(json.dumps({"scope": pin(destination), "overlay": pin(overlay_path),
                  "reconcile": pin(reconciliation), "queue": pin(queue_path),
                  "counts": [len(scope["files"]), total_overlays, len(queue["queue"])]}))
