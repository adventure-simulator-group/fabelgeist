"""Record authored scope qualifications and finding support; no project analysis."""
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


destination = C / "root/parallel-scope-qualification-226.json"
assert not destination.exists()
native_path = C / "native/initial05-aggregate-states/source-qualifications.jsonl"
supplier = next(json.loads(line) for line in native_path.read_text(encoding="utf-8").splitlines()
                if json.loads(line).get("id") == "NATIVE-INITIAL05-QUAL086")
record = {
    "id": "ROOT-PARALLEL-SCOPE-QUALIFICATION-226", "at": NOW,
    "supplier": {"record": pin(native_path), "original_record": supplier},
    "skill": {
        "source_sha256": "8e728cfbf635da12652b9efd7017da057525b5f483756483a21b85a5aa700f9c",
        "actual_EOF": 1165,
        "pr_original_scoped_union_retained": [[1, 187], [690, 1165]],
        "pr_fragment_qualifications": "119-187 includes test stubs and partial first fixture, no188-689 or whole test-module coverage.690-703 is only label match tail, not complete label function or bijection.704-1165 contains the relevant complete production operations, compared in Root148.",
        "p16_original_scoped_union_retained": [[9, 117], [720, 777], [892, 981], [1010, 1105]],
        "p16_complete_operations_and_declarations": [[9, 117], [720, 777], [892, 981], [1015, 1102]],
        "p16_context_only_ranges": [[1010, 1014], [1103, 1105]],
        "p16_qualification": "1010-1013 description tail and impl closure,1014blank.1103blank/1104-1105 transfer docs only, no1106+ effective_skill_hours or final skill_check bodies.983-1009 description prefix excluded. Empty full counterpart diff never enlarges scoped union or grants test bodies. Root148 exact comparison and fresh boundary reread support this qualification."
    },
    "interior": {
        "source_id": "pr-working:crates/adventuresim-building-generator/src/interior.rs",
        "source_sha256": "6dfb61f15cc323392577e8773083c09ea549a3efaace1d3c92802e113ba026cd",
        "actual_EOF": 129,
        "original_scoped_union_retained": [[1, 90]],
        "qualified_material_ranges": [[1, 79]],
        "context_only_ranges": [[80, 90]],
        "fresh_Root_boundary_read": [[72, 90]],
        "qualification": "80-88 error alias and first enum variants are adapter context;90 opens MissingRoom fields not shown. No91-129 fields/whole error enum, included modules or whole129 credit from this partial role. Fresh EOF/SHA/boundary verification grants no complete Root03/04 semantic coverage. Any independently broader original scope stays broader."
    },
    "qualification": "Original raw scopes and provenance retained. No new variants, overlays, bounded queue roles, source semantic clearance or other-lens credit. All prior qualifications including224/220/210/208/200 retained.",
    "all_lenses_mandatory": True,
    "counts_unchanged": {"variants": 2844, "overlays": 245, "bounded01_02_roles": 302},
    "source_edits": 0, "project_execution": 0
}
write(destination, record)
A = C / "root/initial04-invariant-operations"
qualification = {
    "id": "ROOT-INITIAL04-F016-ADDITIONAL-SKILL-148", "at": NOW,
    "finding_id": "ROOT-INITIAL04-F016", "contract_ref": "ROOT-INITIAL04-CONTRACT-148",
    "source_id": "pr-working:crates/adventuresim-core/src/skill.rs",
    "ranges": [[1106, 1125], [707, 714]],
    "qualification": "Additional independently compared ordinary-skill aggregate witness: finite Dodge direct MAX plus finite Balance MAX*.20 overflows effective hours to infinity, then training_rank converts nonfinite to0. This is existing broad study-hours propagation graph, not a new unique finding. Ordinary calibration up to30000 is not language/family5000 cap; preserve asymptotic capacity and latent hours. Actual normal producer/persisted bounds not traced; no runtime or ordinary gameplay failure claimed. P16 scope ends1105 context, receives none of this body credit.",
    "independent_existing_proof_dedup": {
        "record": "continuation/native/initial05-aggregate-states/contract-groups-227.json",
        "sha256": "9ff92c280232c45d9c0f6032d67f8a4f9db1d4ebde75ad56c2d55a865a06faa7",
        "qualification": "Native reported prior exact one-pass transfer witness; no cross-lens semantic credit."
    },
    "finding_count_delta": 0, "source_edits": 0, "project_execution": 0
}
with (A / "finding-qualifications.jsonl").open("a", encoding="utf-8") as stream:
    stream.write(json.dumps(qualification) + "\n")
queue_path = C / "bounded01-02-newmaterial-queue-087.json"
queue = read(queue_path)
queue["mandatory_scope_qualifications"].append(pin(destination))
queue["updated_at"] = NOW
write(queue_path, queue)
authority_path = A / "scope-authority-001.json"
authority = read(authority_path)
authority["mandatory_skill_interior_qualification226"] = pin(destination)
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
    ledger["continuation"]["scope_qualification226"] = pin(destination)
    if "current_cumulative_scope" in ledger:
        ledger["current_cumulative_scope"]["mandatory_scope_qualifications"] = queue["mandatory_scope_qualifications"]
    write(ledger_path, ledger)
    assert read(ledger_path)["continuation"]["current_progress"] == pin(progress_path)
print(json.dumps({"qualification": pin(destination), "queue": pin(queue_path),
                  "progress": pin(progress_path), "new_source_coverage": 0,
                  "finding_count_delta": 0, "ledger_pointers_verified": True}))
