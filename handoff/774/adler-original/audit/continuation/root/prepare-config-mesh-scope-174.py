import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
C = Path(__file__).resolve().parents[1]
NOW = datetime.now(timezone.utc).isoformat()
def read(p): return json.loads(p.read_text(encoding="utf-8"))
def pin(p): return {"record":"continuation/"+p.relative_to(C).as_posix(),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
def write_new(p,o):
    assert not p.exists(),str(p)
    p.write_text(json.dumps(o,indent=2)+"\n",encoding="utf-8")
qual={"id":"ROOT-PARALLEL-SCOPE-QUALIFICATION-173","at":NOW,"source_id":"pr-working:crates/fabelgeist-mesh/src/data.rs","registration":"RIG-INITIAL06-REG-035","effective_material_ranges":[[1,11],[41,68]],"excluded":[[12,40],[69,293]],"qualification":"Rig QUAL036 excludes unrelated native upload/read helper display12-39 and blank40/69 from owner declaration role. Raw display retained historically, no helper/native semantic credit and no graph expansion. Fresh Root actual admission reads only1-11+41-68. Fields/default scope only, not MeshData methods70+ or tests.","status":"mandatory_material_range_qualification"}
write_new(C/"root/parallel-scope-qualification-173.json",qual)
sels=[("rig","rig/initial06-type-ownership/source-range-registration.jsonl",["RIG-INITIAL06-REG-034","RIG-INITIAL06-REG-035"]),("native","native/initial05-aggregate-states/source-range-registration.jsonl",["NATIVE-INITIAL05-REG030","NATIVE-INITIAL05-REG031","NATIVE-INITIAL05-REG032"])]
roles=[]; paths={}
for supplier, rel, ids in sels:
    rows=[json.loads(l) for l in (C/rel).read_text(encoding="utf-8").splitlines()]
    for ident in ids:
        r=next(x for x in rows if x["id"]==ident)
        snap={"physical-p16":"p16","pr-working":"pr-working"}[r["source_id"].split(":",1)[0]]
        p=Path(r.get("resolved_path",str(C.parents[1]/"774-snapshots"/snap/r["source_id"].split(":",1)[1])))
        assert hashlib.sha256(p.read_bytes()).hexdigest()==r["sha256"]
        assert len(p.read_text(encoding="utf-8").splitlines())==r["actual_EOF"]
        ranges=[[1,11],[41,68]] if ident=="RIG-INITIAL06-REG-035" else r["material_ranges"]
        roles.append({"source_id":r["source_id"],"sha256":r["sha256"],"registration_id":ident,"actual_compared_ranges":ranges,"source_evidence":"Fresh exact registered effective ranges personally read; native config full union of existing material plus new bounded completion independently compared, no semantic carry."})
        paths[r["source_id"]]=str(p)
groups={
"mesh-export":"Actual lib1-22 keeps GPU implementation/exports feature-gated and exposes CPU MeshData, FrontFace, PrimitiveTopology. Only declarations/reexports, no transitive GPU/helper/algorithm credit.",
"mesh-product":"Actual imports1-11 and fields/default41-68 retain optional indices, UVs, four-wide joints/weights, topology/winding and optional-empty normal arrays. No required per-vertex color field. This contract differs from colored unskinned PlantMesh output and must not be mechanically substituted based on similar vector fields. Data methods12-40/70+ excluded; public fields still require whole-product admission at actual operation/native receiver.",
"procedural-tail":"Actual61-66 completes same scoped validate error construction and Ok return: zero presentation sample gap or too-small response-angle rejected. Whole source1-66 read for context, new role only61-66. No next module/body-response runtime credit.",
"playback-tail":"Actual26-48 completes playback declaration and validator, requiring nonzero sample gap and deactivation<=initial<=activation. Canonical signed visual offset distinct from nonnegative movement/stance lengths and positive authored cadence product; default/consumer behavior not inferred. Whole1-48 context read, newrole26-48.",
"ragdoll-gap":"Actual60-80 adds nine-body-part capsule product with PositiveLength radius/length. Existing1-59/81-112 independently read: explicit Never versus positive After muscle release, TryFrom<Option<f32>> delegates positive duration admission; no zero-as-absence compatibility path. Default delegates canonical authored config, not a globally validated runtime ragdoll."
}
write_new(C/"root/admission03/new-parallel-admission-174.json",{"id":"ROOT-ADM03-PARALLEL-174","at":NOW,"source_roles":roles,"groups":groups,"mandatory_qualification173":pin(C/"root/parallel-scope-qualification-173.json"),"status":"exact_registered_source_semantic_admission_compared","source_edits":0,"project_execution":0,"other_lens_credit":False})
qp=C/"bounded01-02-newmaterial-queue-087.json";q=read(qp);q["mandatory_scope_qualifications"].append(pin(C/"root/parallel-scope-qualification-173.json"));qp.write_text(json.dumps(q,indent=2)+"\n",encoding="utf-8")
p=read(C/"root/scope-publication-plan-172.json")
p.update(previous_scope="cumulative-source-scope-parallel-050.json",next_scope="cumulative-source-scope-parallel-051.json",reconciliation_record="root/parallel-scope-reconciliation-175.json",reconciliation_id="ROOT-PARALLEL-SCOPE-RECONCILIATION-175",admission_record="root/admission03/new-parallel-admission-174.json",expected_previous_overlays=233,overlay_backup="owner-range-extensions-through233-before051.jsonl",scope_increment_key="parallel_initial_scope_increment175",ledger_increment_key="parallel_scope175",queue_count_key="051_new_roles")
p["registries"]=[]
for supplier,rel,ids in sels:
    selected=[r for r in roles if r["registration_id"] in ids]
    p["registries"].append({"path":rel,"ids":ids,"supplier":supplier,"effective_ranges":{r["registration_id"]:r["actual_compared_ranges"] for r in selected},"qualifications":{r["registration_id"]:"Exact fresh03 admission174 only; MeshData declaration excludes12-40/69+ under qualification173. Native config gaps complete prior bounded roles, no foreign/native/runtime or other-lens verdict." for r in selected}})
p["source_paths"]=paths
p["limits"]="Two new variants2818 and three overlays236, five exact registered roles; all earlier qualifications plus173 retained. All13 initial audits then handover under163, no implementation."
p["queue_instructions"]="After07 independently compare267 exact pending new-material roles for01/02; retain historical040/042 and mandatory qualifications170/173 plus prior. No semantic carry; all13 initial audits then handover163."
p["progress_updates"]={"current03":"0512818+236 published roles accounted WITH_FINDINGS; prior050 plus actual MeshData declaration/export and config-gap completion174. No other-lens verdict transfer.","historical01_02":"Historical040/042 retained; Camera after07 bounded267-role queue087.","new_scope_frontier":"Through Camera031/Rig035/Native032/Root017 captured in051. MeshData helper display uncredited173; frozen6533+material only, remote6223 later reconciliation. User163 audit then handover."}
write_new(C/"root/scope-publication-plan-175.json",p)
print(json.dumps({"qualification":pin(C/"root/parallel-scope-qualification-173.json"),"admission":pin(C/"root/admission03/new-parallel-admission-174.json"),"registered_roles":len(roles)}))

