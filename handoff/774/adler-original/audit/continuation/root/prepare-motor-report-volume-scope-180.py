import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1]
NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding="utf-8"))
def pin(p):return {"record":"continuation/"+p.relative_to(C).as_posix(),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
def new(p,o):
    assert not p.exists(),str(p)
    p.write_text(json.dumps(o,indent=2)+"\n",encoding="utf-8")
sels=[("camera","camera/initial07-redundant-authority/source-range-registration.jsonl",["CAMERA-INITIAL07-REG033","CAMERA-INITIAL07-REG034","CAMERA-INITIAL07-REG035","CAMERA-INITIAL07-REG036","CAMERA-INITIAL07-REG037"]),("native","native/initial05-aggregate-states/source-range-registration.jsonl",["NATIVE-INITIAL05-REG034"])]
roles=[];paths={}
for supplier,rel,ids in sels:
    rr=[json.loads(l) for l in (C/rel).read_text(encoding="utf-8").splitlines()]
    for ident in ids:
        r=next(x for x in rr if x.get("id",x.get("registration_id"))==ident)
        snap={"physical-p16":"p16","pr-working":"pr-working"}[r["source_id"].split(":",1)[0]]
        p=Path(r.get("resolved_path",str(C.parents[1]/"774-snapshots"/snap/r["source_id"].split(":",1)[1])))
        assert hashlib.sha256(p.read_bytes()).hexdigest()==r["sha256"]
        assert len(p.read_text(encoding="utf-8").splitlines())==r["actual_EOF"]
        roles.append({"source_id":r["source_id"],"sha256":r["sha256"],"registration_id":ident,"actual_compared_ranges":r["material_ranges"],"source_evidence":"Fresh literal exact complete registered source or74-106 material role; actual PR/P16 plugin each independently read, no body verdict from hash/names/other lenses."})
        paths[r["source_id"]]=str(p)
groups={
"motor-config-writer":"P16 direct helper1-18 zeroes native acceleration/air/water acceleration/friction/stop speed, writes gravity/step height, derives walk cosine from configured angle. Native startup defaults differ from per-step runtime policy and must not be merged as duplicate authorities. No foreign Ahoy implementation or actual execution inferred.",
"physics-schedule":"Both actual88-line plugin variants separately read: simulation branch schedules own motor before Ahoy MoveCharacters; presentation branch enables solver and disables ordinary replicated rigidbodies by required component; tool branch registers collision/hierarchy phases without solver and explicitly end-optimize in finalize. Public bool combinations choose simulation first. CPU debug config/gizmos presentation separate from authoritative gameplay. No foreign schedules/global ordering/replay guarantee from local wiring alone.",
"causal-report":"Full P16report269 retains unit-labelled measurements, counters and nested per-combatant summaries. finish_means mutates stored sum-named mean fields by sample counts and can partially update before later failure. record_final checks native measurement leaves/addition sequentially, then raw wound/run/map counters. Correct caller must discard/reject failed product; actual whole report typestate/count capacity and publish flow need operation/aggregate lenses, no ordinary partial report claimed. Preserve distinct tactical/autoresolve/wall/energy/movement units, history/canceled/transformed counts; canonical getters/output adapters may remove true duplicate projections without collapsing distinct populations.",
"decisions-callbacks":"Full P16decisions89 projects committed-defense/transformed/continuation events into log with exact tick/elapsed/name/optional target/attack-key/cause. Missing player query returns before mutation. Canceled versus transformed and withdraw-started versus yield-accepted are distinct events, not duplicate current authority. Clock/attack-key identity integration remains lower suppliers and current receivers, notably RootF027, not cleared by a log append.",
"milliliters-kernel":"Actual P16material74-106 privateu64 Milliliters, zero-valid default/new/get/iszero and checked_add/sub returnNone for native overflow/underflow. This exact unit role preserves checked accumulation in containment graph; no Microliters/conversion/other material methods or tests107+ credit. Source metadata does not transfer Native05 graph verdict."
}
new(C/"root/admission03/new-parallel-admission-180.json",{"id":"ROOT-ADM03-PARALLEL-180","at":NOW,"source_roles":roles,"groups":groups,"status":"exact_registered_source_semantic_admission_compared","source_edits":0,"project_execution":0,"other_lens_credit":False})
plan=read(C/"root/scope-publication-plan-178.json")
plan.update(previous_scope="cumulative-source-scope-parallel-052.json",next_scope="cumulative-source-scope-parallel-053.json",reconciliation_record="root/parallel-scope-reconciliation-181.json",reconciliation_id="ROOT-PARALLEL-SCOPE-RECONCILIATION-181",admission_record="root/admission03/new-parallel-admission-180.json",expected_previous_overlays=236,overlay_backup="owner-range-extensions-through236-before053.jsonl",scope_increment_key="parallel_initial_scope_increment181",ledger_increment_key="parallel_scope181",queue_count_key="053_new_roles")
plan["registries"]=[]
for supplier,rel,ids in sels:
    chosen=[r for r in roles if r["registration_id"] in ids]
    plan["registries"].append({"path":rel,"ids":ids,"supplier":supplier,"effective_ranges":{r["registration_id"]:r["actual_compared_ranges"] for r in chosen},"qualifications":{r["registration_id"]:"Exact prospectively registered source role after fresh Root03 proof180; no native library, whole simulation/CLI execution or adjacent material107+ credit. All prior mandatory qualifications retained." for r in chosen}})
plan["source_paths"]=paths
plan["limits"]="Five new variants2825 and one overlay237, six exact registered roles. Only actual registered controller/scheduler/report/decision/Milliliters roles, no foreign-native/test expansion. Exhaustive all13 initial audits then handover163."
plan["queue_instructions"]="After07 independently compare275 exact pending new-material roles for01/02; retain historical040/042 and all mandatory qualifications. No other-lens verdict transfer. All13 initial audits then handover163."
plan["progress_updates"]={"current03":"0532825+237 published roles accounted WITH_FINDINGS; prior052 plus exact motor/schedule/report/decision/Milliliters180. No other-lens verdict transfer.","historical01_02":"Historical040/042 retained; Camera after07 bounded275-role queue087.","new_scope_frontier":"Through Camera037/Rig035/Native034/Root017 captured053. Frozen6533+recorded material; remote6223 future reconciliation. No implementation under163."}
new(C/"root/scope-publication-plan-181.json",plan)
print(json.dumps({"admission":pin(C/"root/admission03/new-parallel-admission-180.json"),"plan":pin(C/"root/scope-publication-plan-181.json")}))

