import collections
import datetime
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
c = root / "continuation"
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
def read(path):
    return json.loads((root / path).read_text(encoding="utf-8"))
def rows(path):
    return [json.loads(line) for line in (root / path).read_text(encoding="utf-8").splitlines()]
def pin(path):
    return {"record": path, "sha256": hashlib.sha256((root / path).read_bytes()).hexdigest()}
destination = c / "root/producer-followup-acceptance-028.json"
assert not destination.exists()
scope = read("continuation/cumulative-source-scope-producer-followup-008.json")
old = read("continuation/cumulative-source-scope-after-producer.json")
current = {x["source_id"]: x for x in scope["files"]}
new_ids = set(current) - {x["source_id"] for x in old["files"]}
assert len(new_ids) == 79
proofs = collections.defaultdict(list)
obligations = []
def add(sid, record, ref, qualification):
    proofs[sid].append({"record": record, "trace_ref": ref, "qualification": qualification})
path = "continuation/root/owner-reached-producer-followup/direct-five-traces-001.json"
for x in read(path)["traces"]:
    obligations.append(x["obligation_id"])
    add(x["source_id"], path, x["obligation_id"], "Actual direct source/receiving contract plus completion-direct-five-002 roof receiver join; no prior incomplete layout disposition carried.")
path = "continuation/camera/owner-reached-producer-followup/traces.jsonl"
for x in rows(path):
    for src in x["actual_source_proofs"]:
        add(src["source_id"], path, x["id"], src["qualification"])
obligations += [x["assigned_obligation"] for x in rows("continuation/camera/owner-reached-producer-followup/coverage.jsonl")]
path = "continuation/native/root-roof-producer-followup/traces.jsonl"
for x in rows(path):
    add(x["source_id"], path, x["id"], x["scope_limit"])
rig = "continuation/rig/owner-reached-producer-followup/"
registrations = {x["id"]: x for x in rows(rig + "additional-sources.jsonl")}
for x in rows(rig + "traces.jsonl"):
    for key in x.get("fresh_scope_registration_refs", []):
        if key.endswith("-004") or key.endswith("-005"):
            continue
        add(registrations[key]["source_id"], rig + "traces.jsonl", x["id"], "Exact per-registration material qualifications; TRACE010 overrides TRACE004 unchanged glazing-size waiver. Requests004/005 are context only; actual Mesh proof uses006/014.")
for x in rows(rig + "coverage.jsonl"):
    obligations.append(x["obligation_id"])
    for ref in x["trace_refs"]:
        add(x["source_id"], rig + "traces.jsonl", ref, x["scope_limits_as_assigned"])
expected = read("continuation/owner-reached-producer-followup-queue-002.json")["items"]
assert len(obligations) == 27 and len(set(obligations)) == 27
expected_ids = {x.get("id", x.get("obligation_id")) for x in expected}
assert set(obligations) == expected_ids, (set(obligations) - expected_ids, expected_ids - set(obligations))
assert new_ids <= set(proofs), sorted(new_ids - set(proofs))
overlays = rows("continuation/owner-range-extensions.jsonl")
assert len(overlays) == 36
assert all(x["source_id"] in proofs for x in overlays)
completion_paths = [
    "continuation/root/owner-reached-producer-followup/completion-direct-five-002.json",
    "continuation/camera/owner-reached-producer-followup/completion-001.json",
    "continuation/native/root-roof-producer-followup/progress.json",
    rig + "completion-001.json",
    "continuation/root/producer-scope-reconciliation-024.json",
    "continuation/root/producer-mesh-boundary-comparison-025.json",
    "continuation/root/rig-producer-claim-reconciliation-027.json",
]
receipt = {
    "id": "CONT-ROOT-CURRENT-PRODUCER-FOLLOWUP-ACCEPTANCE-028",
    "at": now,
    "lens": "audit-producer-propagation",
    "discovery": "Current exact producer-role inventory exhausted with actionable findings; not clear, no repair or validation credit.",
    "prior_scope": pin("continuation/cumulative-source-scope-after-producer.json"),
    "scope": pin("continuation/cumulative-source-scope-producer-followup-008.json"),
    "overlays": pin("continuation/owner-range-extensions.jsonl"),
    "counts": {"current_variants": 2552, "prior_variants": 2473, "new_variants": 79, "original_bounded_obligations": 27, "overlays": 36},
    "actual_semantic_reconciliation": [
        "Root personally consumed all seven Camera narratives and exact original ten obligations, including native response equations/config retention, Threat syntax/member/lexical-order distinctions, SDK volume/age and true Maud ports.",
        "Root personally consumed all26 roof-specific Native producer narratives, exact Root direct-five and RIG027 receiver joins, scaled plane/concave-hole and checked-point-discard distinctions.",
        "Root personally consumed all ten Rig narratives, nine findings, original12 source obligations, three coverage errata and finding erratum001. TRACE010/F009 explicitly overrides TRACE004's broad numerical-pipeline waiver. Root also freshly read actual glazing/Builder/corner/UV/Lod/finish path in025.",
        "All94 prospective registry rows and56 local qualifications reconciled in024; every source/role remains qualified independently of overlapping bytes. Actual source-role evidence supplies acceptance; source/hash/count checks supply identity only.",
        "Original2473 exact accepted producer proofs remain pinned. Every new79 source variant and all36 range/role overlays join actual current narratives; no unread actual receiver reported or discovered in current bounded graphs.",
    ],
    "new_source_proof_map": [{"source_id": sid, "sha256": current[sid]["sha256"], "source_role_traces": proofs[sid]} for sid in sorted(new_ids)],
    "overlay_role_proof_map": [{"overlay": x["id"], "source_id": x["source_id"], "actual_traces": proofs[x["source_id"]]} for x in overlays],
    "pins": [pin(path) for path in completion_paths],
    "claims": "Root023/027 and existing exact whole-function claims govern repairs; audit-source assignments transfer no repair authority.",
    "caller_limit": "Shared API implementation must cover all materially affected callers and extend source/audit scope before new analysis. Current checked-value traces do not clear unrelated raw authoring functions; full post-repair waves remain mandatory.",
    "next": "Only bounded owner81 (Camera16/Rig39/Native26) against accepted owner2483+17; newly reached definitions can reopen producer coverage. Admission03 remains closed until fixed point.",
    "source_edits": 0,
    "project_execution": 0,
    "new_human_decisions": [],
}
destination.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
audit_path = root / "records/final-acceptance-20261021/audit-ledger.json"
audit = json.loads(audit_path.read_text(encoding="utf-8"))
for lens in audit["lenses"]:
    present = {x["source_id"] for x in lens["cells"]}
    if lens["lens"] == "audit-producer-propagation":
        lens["discovery"] = "initial_current_exact_scope_exhausted_with_actionable_findings; owner81_can_reopen_new_roles"
        lens["current_followup_accounting"] = pin("continuation/root/producer-followup-acceptance-028.json")
        for sid in sorted(new_ids - present):
            lens["cells"].append({"source_id": sid, "owner": current[sid].get("audit_owner", current[sid].get("owner")), "observed_sha256": current[sid]["sha256"], "discovery": "accounted_with_findings_for_exact_current_roles", "actual_trace_refs": proofs[sid], "evidence": "../../continuation/root/producer-followup-acceptance-028.json"})
    else:
        for sid in sorted(set(current) - present):
            lens["cells"].append({"source_id": sid, "owner": current[sid].get("audit_owner", current[sid].get("owner")), "discovery": "incomplete", "source_read": "unread_for_this_lens", "evidence": []})
    lens["scoped_variants"] = 2552
audit["updated_at"] = now
audit["current_lens"] = "audit-existing-type-owners"
audit["first_unfinished_cell"] = "producer-reached-owner-queue-026.json first obligation; exact81 not_started; initial admission03-13 still incomplete"
audit["current_inventory_step"] = "bounded_owner81_after_producer2552_exact_roles_exhausted_with_findings"
audit["continuation"]["current_producer_followup_acceptance"] = pin("continuation/root/producer-followup-acceptance-028.json")
audit_path.write_text(json.dumps(audit, indent=2) + "\n")
execution_path = root / "records/execution-ledger.json"
execution = json.loads(execution_path.read_text())
execution["updated_at"] = now
execution["next_action"] = receipt["next"] + " Then initial lenses03-13, mandatory P16/P17 and connected repairs, complete repeat waves/independent reviews/hosted finalhead acceptance. Source freeze remains."
execution["events"].append({"at": now, "kind": "current_producer_followup_exhausted_with_findings", "record": "continuation/root/producer-followup-acceptance-028.json", "source_edits": 0, "family_status": "unchanged"})
execution_path.write_text(json.dumps(execution, indent=2) + "\n")
print(json.dumps({"producer": "exhausted_with_findings", "current_lens": audit["current_lens"], "owner_delta": 81, "source_edits": 0}))
