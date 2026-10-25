import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1]
NOW=datetime.now(timezone.utc).isoformat()
def read(p): return json.loads(p.read_text(encoding="utf-8"))
def pin(p): return {"record":"continuation/"+p.relative_to(C).as_posix(),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
def write_new(p,o):
    assert not p.exists(),str(p)
    p.write_text(json.dumps(o,indent=2)+"\n",encoding="utf-8")
sels=[("camera","camera/initial07-redundant-authority/source-range-registration.jsonl",["CAMERA-INITIAL07-REG032"]),("native","native/initial05-aggregate-states/source-range-registration.jsonl",["NATIVE-INITIAL05-REG033"])]
roles=[];paths={}
for supplier,rel,ids in sels:
    rr=[json.loads(l) for l in (C/rel).read_text(encoding="utf-8").splitlines()]
    for ident in ids:
        r=next(x for x in rr if x.get("id",x.get("registration_id"))==ident)
        snap={"physical-p16":"p16","pr-working":"pr-working"}[r["source_id"].split(":",1)[0]]
        p=Path(r.get("resolved_path",str(C.parents[1]/"774-snapshots"/snap/r["source_id"].split(":",1)[1])))
        assert hashlib.sha256(p.read_bytes()).hexdigest()==r["sha256"]
        assert len(p.read_text(encoding="utf-8").splitlines())==r["actual_EOF"]
        ranges=[[1,78]] if supplier=="camera" else [[1,75]]
        roles.append({"source_id":r["source_id"],"sha256":r["sha256"],"registration_id":ident,"actual_compared_ranges":ranges,"displayed_blank_context":79 if supplier=="camera" else 76,"source_evidence":"Fresh complete literal production body after prospective registration; trailing blank context excluded from effective role. Exact frozen revision/import context, no other-lens credit."})
        paths[r["source_id"]]=str(p)
admission={"id":"ROOT-ADM03-PARALLEL-177","at":NOW,"source_roles":roles,"groups":{
"p16-armor-receiver":"Actual P16 armor_layer_chain/fold1-78 independently compared: typed BodyPart maps index, covered mask+scalar coverage choose layers, authored geometry used by contains/expect, descending order selects outermost contact, distinct surface projection. Fold combines coverage, raw nonnegative resistance/padding, resistance-weighted flexibility and minimum ROM. Scalar coverage, per-part mask and authored geometry have different roles; neither same fields nor identical PR bytes proves coherent whole ArmorItem or finite computed aggregate. No tests79+blank/80+body or whole inventory credit.",
"current-window-decoder":"Actual current PR SceneWindow1-75 has typed LeafDimensions, scene Position/normalized SpatialDirection, canonical building/opening/leaf/bar vocabulary and immutable Bevy component designation. ManualDeserialize gathers native leaves then admits local Self, preserving IDs and concrete GeometryError in SceneWindowError before serde error formatting at actual decode boundary. Admission failure cannot publish a partially constructed component. Independent normalized directions do not establish paired orthogonal opening-frame/product identity; exact producer/receiver coherence remains an obligation, no new normal failure inferred. OpeningAssemblyId tuple wrapping does not prove nonzero is a required policy; no arbitrary ID restriction.",
"scope":"Only two direct counterpart/receiver roles; trailing blank armor79/window76 context uncredited. P16 window whole366 and PR armor prior role are separate evidence, no counterpart verdict transfer."
},"status":"exact_registered_source_semantic_admission_compared","source_edits":0,"project_execution":0,"other_lens_credit":False}
write_new(C/"root/admission03/new-parallel-admission-177.json",admission)
plan=read(C/"root/scope-publication-plan-175.json")
plan.update(previous_scope="cumulative-source-scope-parallel-051.json",next_scope="cumulative-source-scope-parallel-052.json",reconciliation_record="root/parallel-scope-reconciliation-178.json",reconciliation_id="ROOT-PARALLEL-SCOPE-RECONCILIATION-178",admission_record="root/admission03/new-parallel-admission-177.json",expected_previous_overlays=236,overlay_backup="owner-range-extensions-through236-before052.jsonl",scope_increment_key="parallel_initial_scope_increment178",ledger_increment_key="parallel_scope178",queue_count_key="052_new_roles")
plan["registries"]=[]
for supplier,rel,ids in sels:
    chosen=[r for r in roles if r["registration_id"] in ids]
    plan["registries"].append({"path":rel,"ids":ids,"supplier":supplier,"effective_ranges":{r["registration_id"]:r["actual_compared_ranges"] for r in chosen},"qualifications":{r["registration_id"]:"Only exact production role after fresh Root03 proof177; armor79/window76 trailing blank context excluded, tests/whole parent/native graph uncredited. No other-lens verdict transfer." for r in chosen}})
plan["source_paths"]=paths
plan["limits"]="Two new variants2820; mandatory overlays unchanged236. Armor exact1-78/window1-75 excludes trailing blanks and tests; all prior qualifications retained. Exhaustive initial audits then handover163, no implementation."
plan["queue_instructions"]="After07 independently compare269 exact pending new-material roles for01/02; preserve historical040/042, all mandatory qualifications and effective production endpoints. All13 initial audits then handover163."
plan["progress_updates"]={"current03":"0522820+236 published roles accounted WITH_FINDINGS; prior051 plus fresh P16 armor and current PR window receiver177. No other-lens verdict transfer.","historical01_02":"Historical040/042 retained; Camera after07 bounded269-role queue087.","new_scope_frontier":"Through Camera032/Rig035/Native033/Root017 captured052. Frozen6533+recorded material; remote6223 future reconciliation. No implementation under163."}
write_new(C/"root/scope-publication-plan-178.json",plan)
print(json.dumps({"admission":pin(C/"root/admission03/new-parallel-admission-177.json"),"plan":pin(C/"root/scope-publication-plan-178.json")}))

