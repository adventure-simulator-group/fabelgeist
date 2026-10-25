"""Save authored audit evidence and actual bookkeeping; no project analysis."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

C = Path(__file__).resolve().parents[1]
B = C.parent
NOW = datetime.now(timezone.utc).isoformat()


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def pin(path):
    return {
        "record": "continuation/" + path.relative_to(C).as_posix(),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def last(path):
    return json.loads(path.read_text(encoding="utf-8").splitlines()[-1])


plan = read(Path(sys.argv[1]))
number = plan["number"]
destination = C / f"root/audit-checkpoint-{number:03}.json"
assert not destination.exists()
previous = C / plan["previous_checkpoint"]
authority_path = C / "root/initial04-invariant-operations/scope-authority-001.json"
authority = read(authority_path)
scope = read(B / authority["scope"]["record"])
completed = sum(
    row["status"] == "exact_scoped_current04_operations_manually_compared"
    for row in authority["rows"]
)
findings_path = C / "root/initial04-invariant-operations/findings.jsonl"
findings = len(findings_path.read_text(encoding="utf-8").splitlines())
native_path = sorted((C / "native/initial05-aggregate-states").glob("progress-*.json"))[-1]
native = read(native_path)
rig_path = C / "rig/initial06-type-ownership/progress.jsonl"
rig = last(rig_path)
camera_path = C / "camera/initial07-redundant-authority/progress.jsonl"
camera = last(camera_path)
queue_path = C / "bounded01-02-newmaterial-queue-087.json"
queue = read(queue_path)
pending = []
for registry in plan["captured_registries"]:
    for line in (C / registry["path"]).read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        identity = row.get("id", row.get("registration_id", ""))
        suffix = re.search(r"(\d+)$", identity)
        if suffix and int(suffix.group(1)) > registry["through"]:
            pending.append({
                "registration": identity,
                "source_id": row["source_id"],
                "ranges": row.get("material_ranges", row.get("ranges")),
                "status": "prospective_pending_actual_root03_global_reconciliation",
            })
assert completed == plan["expected_root_completed"]
assert findings == plan["expected_root_findings"]
assert len(scope["files"]) == plan["expected_variants"]
assert authority["mandatory_overlays"]["through"] == plan["expected_overlays"]
assert len(queue["queue"]) == plan["expected_queue_roles"]
out = read(previous)
out.update(
    id=f"ROOT-AUDIT-CHECKPOINT-{number:03}", at=NOW,
    previous_checkpoint=pin(previous), previous_goal_turn_classification="progress",
    previous_goal_turn_evidence=plan["previous_evidence"],
    current_progress=plan["current_progress"], scope=authority["scope"],
    variants=len(scope["files"]), mandatory_overlays=authority["mandatory_overlays"],
    root04={
        "completed": completed, "findings": findings,
        "authority": pin(authority_path),
        "latest_packet": pin(C / plan["latest_packet"]),
        "findings_record": pin(findings_path),
        "finding_qualifications": pin(C / "root/initial04-invariant-operations/finding-qualifications.jsonl"),
        "status": "incomplete",
    },
    peer_progress={
        "native": {"record": pin(native_path), "data": native},
        "rig": {"record": pin(rig_path), "data": rig},
        "camera": {"record": pin(camera_path), "data": camera},
    },
    pending_registered_material=pending,
    latest_scope_reconciliation=pin(C / plan["latest_scope_reconciliation"]),
    mandatory_scope_qualifications=queue["mandatory_scope_qualifications"],
    bounded01_02_queue=pin(queue_path), next=plan["next"],
)
out.update(plan.get("checkpoint_updates", {}))
write(destination, out)
progress_path = C / "root/admission03-progress-067.json"
progress = read(progress_path)
progress.update(
    at=NOW, mode="parallel_exhaustive_initial_audits_then_handover",
    latest_audit_checkpoint=pin(destination), pending_registered_material=pending,
    next=plan["next"], new_scope_frontier=plan["frontier"],
)
progress["parallel_assignments"] = {
    "root": f"04active{completed} exactsources/{findings} findings; then08",
    "native": f"05active{native.get('global_source_variants_compared')} sources/{native.get('qualified_new_findings')} findings at{native_path.name}; then10/13",
    "rig": f"06active{rig.get('completed_global_source_variants')} distinct/{rig.get('coverage_observations')} observations; then09/12",
    "camera": f"07active{camera.get('inventoried_variants')} sources; then01/02bounded{len(queue['queue'])}roles,then11",
}
progress["parallel_assignments"].update(plan.get("assignment_overrides", {}))
progress["goal_turn_progress"] = {
    "previous_turn": plan["previous_evidence"],
    "current_turn": plan["current_progress"], "continuation": plan["next"],
}
write(progress_path, progress)
for path in [B / "records/execution-ledger.json", B / "records/final-acceptance-20261021/audit-ledger.json"]:
    ledger = read(path)
    ledger["updated_at"] = NOW
    ledger["continuation"]["current_progress"] = pin(progress_path)
    ledger["continuation"][f"audit_checkpoint{number}"] = pin(destination)
    if "next_action" in ledger:
        ledger["next_action"] = out["delivery"] + " " + plan["next"]
    write(path, ledger)
    assert read(path)["continuation"]["current_progress"] == pin(progress_path)
    assert read(path)["continuation"][f"audit_checkpoint{number}"] == pin(destination)
print(json.dumps({
    "checkpoint": pin(destination), "progress": pin(progress_path),
    "root04": completed, "findings": findings,
    "native": native.get("global_source_variants_compared"),
    "rig": rig.get("completed_global_source_variants"),
    "camera": camera.get("inventoried_variants"),
    "pending": pending, "ledger_pointers_verified": True, "completion": False,
}))
