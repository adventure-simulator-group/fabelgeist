"""Pin manually checked fragment boundaries; no project analysis/execution."""
import hashlib
import json
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
    return {"record": "continuation/" + path.relative_to(C).as_posix(),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


destination = C / "root/parallel-scope-qualification-230.json"
assert not destination.exists()
supplier_path = C / "native/initial05-aggregate-states/source-qualifications.jsonl"
supplier = next(json.loads(line) for line in supplier_path.read_text(encoding="utf-8").splitlines()
                if json.loads(line).get("id") == "NATIVE-INITIAL05-QUAL088")
record = {
    "id": "ROOT-PARALLEL-SCOPE-QUALIFICATION-230", "at": NOW,
    "supplier": {"record": pin(supplier_path), "original_record": supplier},
    "fresh_Root_boundary_evidence": [
        {"source_id": "pr-working:crates/adventuresim-building-generator/src/generator/room_allocation.rs",
         "sha256": "0af46aaa9d2e8406c02e95802701cab06cb2c9e496138d7589c422b4d53d48c5",
         "actual_EOF": 249, "actual_read": [[140, 153], [169, 181]],
         "raw_scoped_role_retained": [[140, 249]],
         "qualified_complete_material_ranges": [[148, 175], [177, 249]],
         "context_only_ranges": [[140, 147], [176, 176]],
         "qualification": "140-146 is seeded-origin helper tail;148 starts room_counts.176blank;177 starts GrowthCandidate docs/declaration. No original1-139 allocator/rank-prefix credit from this fragment role. Boundary evidence here does not clear remaining148-249 bodies or any full Root03/04 source."},
        {"source_id": "pr-working:crates/adventuresim-building-generator/src/generator/occupied_storeys.rs",
         "sha256": "e45cc55e23a73758b66304cd2fa518b1cb291eb214df0e66787f9c9f13b58341",
         "actual_EOF": 157, "actual_read": [[100, 113], [137, 145]],
         "raw_scoped_role_retained": [[100, 145]],
         "qualified_material_fragment": [[108, 143]],
         "context_only_ranges": [[100, 107], [144, 145]],
         "qualification": "100-107 continues earlier StairHall search and conversion, not complete helper.108-143 is actual reservation/derivation fragment in existing caller, not a standalone complete function.144-145 begins apply_opening_edits call only;146-157 tail excluded. No1-99 or whole157 credit from this role."},
        {"source_id": "pr-working:crates/adventuresim-building-generator/src/generator/heated_rooms.rs",
         "sha256": "443063c283413c94c1d5b856ab84ff71fe275f9f039e1427384f13a395671879",
         "actual_EOF": 274, "actual_read": [[115, 136]],
         "raw_scoped_role_retained": [[1, 74], [115, 274]],
         "qualified_material_ranges": [[1, 74], [131, 274]],
         "context_only_ranges": [[115, 130]],
         "qualification": "115-130 is prior reservation closure tail; no75-114 predicate/input construction credit from that tail.131 resumes complete bay iteration but does not prove excluded closure origin. Native reports original231+ fixtures actually read; this Root boundary read grants none of those bodies or whole274."}
    ],
    "qualification": "Preserve original raw scopes/provenance and any independently broader original full role. Context qualification cannot narrow a truly requested whole file or enlarge a partial one. No new scope, overlay, bounded queue role, whole-source semantic clearance, other-lens verdict or ownership transfer. All mandatory226/224/220/210/208/200 and older qualifications retained.",
    "all_lenses_mandatory": True,
    "counts_unchanged": {"variants": 2846, "overlays": 245, "bounded01_02_roles": 304},
    "source_edits": 0, "project_execution": 0
}
write(destination, record)
queue_path = C / "bounded01-02-newmaterial-queue-087.json"
queue = read(queue_path)
queue["mandatory_scope_qualifications"].append(pin(destination))
queue["updated_at"] = NOW
write(queue_path, queue)
authority_path = C / "root/initial04-invariant-operations/scope-authority-001.json"
authority = read(authority_path)
authority["mandatory_generator_fragment_qualification230"] = pin(destination)
write(authority_path, authority)
progress_path = C / "root/admission03-progress-067.json"
progress = read(progress_path)
progress.update(at=NOW, bounded01_02_queue=pin(queue_path),
                mandatory_scope_qualifications=queue["mandatory_scope_qualifications"],
                latest_scope_qualification=pin(destination))
write(progress_path, progress)
for ledger_path in [B / "records/execution-ledger.json", B / "records/final-acceptance-20261021/audit-ledger.json"]:
    ledger = read(ledger_path)
    ledger["updated_at"] = NOW
    ledger["continuation"]["current_progress"] = pin(progress_path)
    ledger["continuation"]["scope_qualification230"] = pin(destination)
    if "current_cumulative_scope" in ledger:
        ledger["current_cumulative_scope"]["mandatory_scope_qualifications"] = queue["mandatory_scope_qualifications"]
    write(ledger_path, ledger)
    assert read(ledger_path)["continuation"]["current_progress"] == pin(progress_path)
print(json.dumps({"qualification": pin(destination), "queue": pin(queue_path),
                  "progress": pin(progress_path), "new_source_coverage": 0,
                  "ledger_pointers_verified": True}))
