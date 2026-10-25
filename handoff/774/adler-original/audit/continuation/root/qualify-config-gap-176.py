import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1]
def read(p): return json.loads(p.read_text(encoding="utf-8"))
def pin(p): return {"record":"continuation/"+p.relative_to(C).as_posix(),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
p=C/"root/parallel-scope-qualification-176.json"
assert not p.exists()
q={"id":"ROOT-PARALLEL-SCOPE-QUALIFICATION-176","at":datetime.now(timezone.utc).isoformat(),"registration":"NATIVE-INITIAL05-REG032","source_id":"physical-p16:crates/adventuresim-tactical-core/src/combat_config/ragdoll.rs","raw_material_ranges":[[60,80]],"effective_material_ranges":[[61,80]],"qualification":"Native fresh qualification excludes blank60; Root actual displayed60 remains context, no semantic credit. Source174 raw actual range preserves display; effective role61-80 only. Playback26-48 contains declaration and validate, no Default implementation; do not infer one from registration wording.","status":"mandatory_effective_role_qualification"}
p.write_text(json.dumps(q,indent=2)+"\n",encoding="utf-8")
qp=C/"bounded01-02-newmaterial-queue-087.json";queue=read(qp);queue["mandatory_scope_qualifications"].append(pin(p));qp.write_text(json.dumps(queue,indent=2)+"\n",encoding="utf-8")
pp=C/"root/scope-publication-plan-175.json";plan=read(pp)
native=next(r for r in plan["registries"] if r["supplier"]=="native")
native["effective_ranges"]["NATIVE-INITIAL05-REG032"]=[[61,80]]
native["qualifications"]["NATIVE-INITIAL05-REG032"]="Only declaration61-80; blank60 displayed context uncredited under Root176, old1-59/81-112 separate. Playback has no Default implementation. Actual03 proof174 with effective qualification176, no other-lens credit."
plan["limits"]+=" Native ragdoll effective61-80 excludes blank60 under176; playback Default not inferred."
pp.write_text(json.dumps(plan,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"qualification":pin(p),"plan":pin(pp)}))

