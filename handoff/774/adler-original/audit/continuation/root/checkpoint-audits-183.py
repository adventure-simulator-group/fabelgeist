import hashlib,json,re
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding="utf-8"))
def pin(p):return {"record":"continuation/"+p.relative_to(C).as_posix(),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
def last_jsonl(p):return json.loads(p.read_text(encoding="utf-8").splitlines()[-1])
authorityp=C/"root/initial04-invariant-operations/scope-authority-001.json";authority=read(authorityp)
scope=read(B/authority["scope"]["record"])
root_count=sum(r["status"]=="exact_scoped_current04_operations_manually_compared" for r in authority["rows"])
findingsp=C/"root/initial04-invariant-operations/findings.jsonl";finding_count=len(findingsp.read_text(encoding="utf-8").splitlines())
nativep=sorted((C/"native/initial05-aggregate-states").glob("progress-*.json"))[-1];native=read(nativep)
rigp=C/"rig/initial06-type-ownership/progress.jsonl";rig=last_jsonl(rigp)
camerap=C/"camera/initial07-redundant-authority/progress.jsonl";camera=last_jsonl(camerap)
pending=[]
for rel,through in [("camera/initial07-redundant-authority/source-range-registration.jsonl",38),("rig/initial06-type-ownership/source-range-registration.jsonl",35),("native/initial05-aggregate-states/source-range-registration.jsonl",34),("root/initial04-invariant-operations/source-range-registration.jsonl",17)]:
    for line in (C/rel).read_text(encoding="utf-8").splitlines():
        r=json.loads(line);ident=r.get("id",r.get("registration_id",""))
        number=int(re.search(r"(\d+)$",ident).group(1))
        if number>through:pending.append({"registration":ident,"source_id":r["source_id"],"ranges":r.get("material_ranges")})
checkpointp=C/"root/audit-checkpoint-183.json";assert not checkpointp.exists()
checkpoint={"id":"ROOT-AUDIT-CHECKPOINT-183","at":NOW,"previous_goal_turn_classification":"progress","current_progress":"Actual source audit, qualified source repairs recorded, no implementation or project execution.","scope":authority["scope"],"variants":len(scope["files"]),"mandatory_overlays":authority["mandatory_overlays"],"root04":{"completed":root_count,"findings":finding_count,"authority":pin(authorityp),"latest_packet":pin(C/"root/initial04-invariant-operations/contract-groups-114.json"),"findings_record":pin(findingsp),"status":"incomplete"},"peer_progress":{"native":{"record":pin(nativep),"data":native},"rig":{"record":pin(rigp),"data":rig},"camera":{"record":pin(camerap),"data":camera}},"pending_registered_material":pending,"latest_scope_reconciliation":pin(C/"root/parallel-scope-reconciliation-186.json"),"mandatory_qualification184":pin(C/"root/parallel-scope-qualification-184.json"),"delivery":"Finish all13 exhaustive initial audits, then another handover per direct user163; no implementation in this continuation.","next":"Root04 remaining exact sources and overlays then08; peers finish05/10/13,06/09/12,07/01-02newmaterial/11 in order. Reconcile any later prospective source roles before claiming coverage. Original774/P16/P17 implementation and final reviews/hosted validation remain future obligations.","source_edits":0,"project_execution":0,"repairs_allowed":False}
checkpointp.write_text(json.dumps(checkpoint,indent=2)+"\n",encoding="utf-8")
progressp=C/"root/admission03-progress-067.json";progress=read(progressp)
progress.update(at=NOW,mode="parallel_exhaustive_initial_audits_then_handover",latest_audit_checkpoint=pin(checkpointp))
progress["parallel_assignments"]={"root":f"04active{root_count} exact sources through114 plusG083, {finding_count} qualified findings; then08","native":f"05active{native.get('global_source_variants_compared')} persisted sources, {native.get('qualified_new_findings')} findings at {nativep.name}; then10/13","rig":f"06active{rig.get('completed_global_source_variants')} distinct/{rig.get('coverage_observations')} observations; then09/12","camera":f"07active{camera.get('inventoried_variants')} persisted sources; then01/02 bounded276roles,then11"}
progress["goal_turn_progress"]={"previous_turn":"Progress: verified external ownership replies and actual Root04 source records101-113, new03 proofs/scopes050-053; no source changes/project execution.","current_turn":"Progress: full tactical equipment parent/armor receivers114 plus F041 computed balance route; existingF011 armor aggregation independently propagated rather than duplicated. Source scope054/qualification184 correct actual terrain enum gap and tacticsEOF48. Root checkpoint183 keeps all initial lenses incomplete.","continuation":checkpoint["next"]}
progress["next"]=checkpoint["next"];progress["pending_registered_material"]=pending
progress["new_scope_frontier"]="Through Camera038/Rig035/Native034/Root017 captured054 after actual Root03 proof185. Later pending registrations explicitly listed; current source remains frozen6533+recorded material, remote6223 requires future reconciliation."
progressp.write_text(json.dumps(progress,indent=2)+"\n",encoding="utf-8")
for lp in [B/"records/execution-ledger.json",B/"records/final-acceptance-20261021/audit-ledger.json"]:
    ledger=read(lp);ledger["updated_at"]=NOW;ledger["continuation"]["current_progress"]=pin(progressp);ledger["continuation"]["audit_checkpoint183"]=pin(checkpointp)
    if "next_action" in ledger:ledger["next_action"]=checkpoint["delivery"]+" "+checkpoint["next"]
    lp.write_text(json.dumps(ledger,indent=2)+"\n",encoding="utf-8")
for lp in [B/"records/execution-ledger.json",B/"records/final-acceptance-20261021/audit-ledger.json"]:
    assert read(lp)["continuation"]["current_progress"]==pin(progressp)
assert checkpoint["variants"]==2825
assert authority["mandatory_overlays"]["through"]==238
print(json.dumps({"checkpoint":pin(checkpointp),"progress":pin(progressp),"root04":root_count,"findings":finding_count,"native":native.get("global_source_variants_compared"),"rig":rig.get("completed_global_source_variants"),"camera":camera.get("inventoried_variants"),"pending":pending,"ledger_pointers_verified":True,"completion":False}))

