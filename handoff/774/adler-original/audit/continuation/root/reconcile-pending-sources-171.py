import copy
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

C = Path(__file__).resolve().parents[1]
A = C / "root/initial04-invariant-operations"
NOW = datetime.now(timezone.utc).isoformat()
def read(p): return json.loads(p.read_text(encoding="utf-8"))
def write_new(p, o):
    assert not p.exists(), str(p)
    p.write_text(json.dumps(o, indent=2) + "\n", encoding="utf-8")
def pin(p): return {"record": "continuation/" + p.relative_to(C).as_posix(), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
root_registry = C / "root/initial04-invariant-operations/source-range-registration.jsonl"
native_registry = C / "native/initial05-aggregate-states/source-range-registration.jsonl"
registrations = []
for path, ids in [(root_registry, ["ROOT-INITIAL04-REG016", "ROOT-INITIAL04-REG017"]), (native_registry, ["NATIVE-INITIAL05-REG025", "NATIVE-INITIAL05-REG026", "NATIVE-INITIAL05-REG027", "NATIVE-INITIAL05-REG028", "NATIVE-INITIAL05-REG029"])]:
    rows = [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines()]
    for ident in ids:
        reg = next(r for r in rows if r["id"] == ident)
        source = Path(reg.get("resolved_path", str(C.parents[1] / "774-snapshots/pr-working" / reg["source_id"].split(":", 1)[1])))
        assert source.exists(), str(source)
        assert hashlib.sha256(source.read_bytes()).hexdigest() == reg["sha256"]
        assert len(source.read_text(encoding="utf-8").splitlines()) == reg["actual_EOF"]
        registrations.append((reg, source))
groups = {
"current-pr-mass": "Fresh literal PR mass1-41 after Root016 registration: fireplace exclusion, whole food-lot nonnegative mass, measured fraction or quantity, parametric recipe expect, fitted-holder expect/optional catalogue fallback, raw product and sum. P16 prepares fallible recipe causes instead. F035 mass arithmetic remains separate from absent/error recipe policy. No prior premature103 credit reused.",
"current-pr-armor": "Fresh literal PR armor1-103 after Root017 registration: DB item and equipped placement lookup, body-part membership, max occupancy ordering/default, conditioned item stat projection and canonical material/coverage extraction. Same bytes as P16 were independently compared with actual same Core/DB import meanings; no hash-only semantic transfer. No armor wear/whole capability coverage.",
"inventory-query": "Actual inventory/query1-21 iterates only holder Inventory members through q_item.iter_many; slot lookup projects first matching slot and optional inventory ID. Missing holder means empty iterator. This helper is not proof that every member's armor metadata or ownership product was admitted; exact lower recipient role only.",
"armor-fold": "Actual inventory_armor1-78 plus79blank performs covered-part indexing, layer sorting, selected authored geometry expects, per-layer contact projection and layered numeric fold. covered_parts does not itself prove optional geometry presence; admitted collection/coherent ArmorItem product is required. Raw resistance/padding sums and weighted flexibility require operation-lens numerical comparison; this03 proof identifies graph, does not award04/05 verdicts.80+tests excluded.",
"mission-terminal": "Actual new159-213 completes timeout tick and whole CombatOutcomeHold::advance plus held_duration. It resets incomplete/unsealed/empty terminal observations, compares raw population counters, accumulates enemy/party holds saturating Duration and prioritizes party defeat on simultaneous thresholds. Saturation is appropriate for duration threshold comparison; count/roster correlation needs actual producers and recipients. Existing1-158 proof separate; no214+tests.",
"mission-enrollment": "Actual1-124 plus125blank: Awaiting/Enrolling/Sealed sum type, NonZeroU32 expected, seen CharacterId set, begin/admit/sealed rejoin policy, no-loading seal threshold and reconnect empty-duration threshold. This local type admits expected count but does not bind exact strategic roster or receipt capacity by itself; preserve explicit upstream binding obligation.126+tests excluded.",
"mission-receipt": "Actual1-75 plus76blank copies injury/blood/ammunition from accumulated party, adds equipment-contact characters, sorts and truncates party to canonical max while retaining all equipment contacts. Membership/capacity correlation requires normal roster/accumulator/DB recipient proof; no normal truncation failure or executed run asserted. Direct enum projection exhaustive.77+tests excluded."
}
roles = [{"source_id": r["source_id"], "sha256": r["sha256"], "registration_id": r["id"], "actual_compared_ranges": r["material_ranges"], "source_evidence": "Fresh complete literal registered ranges personally read under exact frozen source before this admission record."} for r, p in registrations]
admission = {"id": "ROOT-ADM03-PARALLEL-171", "at": NOW, "source_roles": roles, "groups": groups, "mandatory_qualification": pin(C / "root/parallel-scope-qualification-170.json"), "status": "exact_registered_source_semantic_admission_compared", "source_edits": 0, "project_execution": 0, "other_lens_credit": False}
write_new(C / "root/admission03/new-parallel-admission-171.json", admission)
qualification = pin(C / "root/parallel-scope-qualification-170.json")
qpath = C / "bounded01-02-newmaterial-queue-087.json"
queue = read(qpath)
assert not any(q.get("record") == qualification["record"] for q in queue["mandatory_scope_qualifications"])
queue["mandatory_scope_qualifications"].append(qualification)
qpath.write_text(json.dumps(queue, indent=2) + "\n", encoding="utf-8")
raw = read(A / "contract-groups-103.json")
excluded = set(read(C / "root/parallel-scope-qualification-170.json")["affected_source_ids"])
valid = [r for r in raw["source_roles"] if r["source_id"] not in excluded]
assert len(valid) == 6
recovery = {"id": "ROOT-INITIAL04-CONTRACT-104", "at": NOW, "source_roles": valid, "source_proof": pin(A / "contract-groups-103.json"), "mandatory_qualification": qualification, "groups": {"recovery": "Credit only six already-scoped actual source roles from103; partial writer saved findings/raw coverage but not authority. Preserve all raw receipts, do not append findings or six coverage rows again. Two PR counterparts remain uncredited103 permanently; fresh registered comparison171 will receive later independent04 reconciliation after global publication."}, "findings": [], "repair": "frozen_initial_audits_then_handover", "validation": "not_run", "source_edits": 0, "project_execution": 0, "other_lens_credit": False}
write_new(A / "contract-groups-104.json", recovery)
apath = A / "scope-authority-001.json"
authority = read(apath)
for r in valid:
    row = next(x for x in authority["rows"] if x["source_id"] == r["source_id"])
    assert row["source_sha256"] == r["sha256"]
    assert row["status"] != "exact_scoped_current04_operations_manually_compared"
    row["status"] = "exact_scoped_current04_operations_manually_compared"
    row["actual_compared_ranges"] = r["actual_compared_ranges"]
    row["evidence"].extend([pin(A / "contract-groups-103.json"), pin(A / "contract-groups-104.json")])
authority["mandatory_premature_scope_qualification170"] = qualification
apath.write_text(json.dumps(authority, indent=2) + "\n", encoding="utf-8")
plan = read(C / "root/scope-publication-plan-169.json")
plan.update(previous_scope="cumulative-source-scope-parallel-049.json", next_scope="cumulative-source-scope-parallel-050.json", reconciliation_record="root/parallel-scope-reconciliation-172.json", reconciliation_id="ROOT-PARALLEL-SCOPE-RECONCILIATION-172", admission_record="root/admission03/new-parallel-admission-171.json", expected_previous_overlays=232, overlay_backup="owner-range-extensions-through232-before050.jsonl", scope_increment_key="parallel_initial_scope_increment172", ledger_increment_key="parallel_scope172", queue_count_key="050_new_roles")
plan["registries"] = []
for supplier, path, selected in [("root", root_registry, roles[:2]), ("native", native_registry, roles[2:])]:
    ids = [r["registration_id"] for r in selected]
    plan["registries"].append({"path": path.relative_to(C).as_posix(), "ids": ids, "supplier": supplier, "effective_ranges": {r["registration_id"]: r["actual_compared_ranges"] for r in selected}, "qualifications": {r["registration_id"]: "Only exact prospectively registered range; fresh03 comparison171. Root170 permanently excludes premature103 credit for PR mass/armor. No other-lens clearance or adjacent test coverage." for r in selected}})
plan["source_paths"] = {r["source_id"]: str(p) for r, p in registrations}
plan["limits"] = "Six new variants2816 and one mandatory overlay233, seven exact registered roles; all previous qualifications plus170 preserved. All initial audits then handover under163; no source edits/project execution."
plan["queue_instructions"] = "After07 independently compare262 pending exact new-material roles for01/02; preserve historical040/042 and all scope qualifications including170. No verdict carry or full replay. All13 initial audits then handover163."
plan["progress_updates"] = {"current03": "0502816+233 published roles accounted WITH_FINDINGS; prior049 plus actual fresh mass/armor, inventory query/fold and mission lifecycle/receipt171. No other-lens verdict transfer.", "historical01_02": "Historical040/042 retained; Camera after07 bounded262-role queue087.", "new_scope_frontier": "Through Camera031/Rig033/Native029/Root017 captured in050. Initial audit frozen6533+recorded material; live6223320 requires explicit later reconciliation. User163 audit then handover, no implementation."}
plan["next_action"] = "Finish all13 exhaustive initial audits then handover per163; no implementation. Retain original774/P16/P17 integration and same-head validation future obligations."
write_new(C / "root/scope-publication-plan-172.json", plan)
print(json.dumps({"admission": pin(C / "root/admission03/new-parallel-admission-171.json"), "recovery": pin(A / "contract-groups-104.json"), "completed04": sum(r["status"] == "exact_scoped_current04_operations_manually_compared" for r in authority["rows"]), "next": "Publish050 once, then fresh04 root2 reconciliation; no duplicate103 appends."}))

