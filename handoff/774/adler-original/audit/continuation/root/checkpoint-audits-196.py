"""Snapshot current persisted audit evidence and update pointers; metadata only."""
import hashlib,json,re
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def last(p):return json.loads(p.read_text(encoding='utf-8').splitlines()[-1])
authorityp=C/'root/initial04-invariant-operations/scope-authority-001.json';authority=read(authorityp);scope=read(B/authority['scope']['record'])
root_count=sum(r['status']=='exact_scoped_current04_operations_manually_compared' for r in authority['rows'])
fp=C/'root/initial04-invariant-operations/findings.jsonl';finding_count=len(fp.read_text(encoding='utf-8').splitlines())
nativep=sorted((C/'native/initial05-aggregate-states').glob('progress-*.json'))[-1];native=read(nativep)
rigp=C/'rig/initial06-type-ownership/progress.jsonl';rig=last(rigp)
camerap=C/'camera/initial07-redundant-authority/progress.jsonl';camera=last(camerap)
pending=[]
for rel,through in [('camera/initial07-redundant-authority/source-range-registration.jsonl',39),('rig/initial06-type-ownership/source-range-registration.jsonl',36),('native/initial05-aggregate-states/source-range-registration.jsonl',36),('root/initial04-invariant-operations/source-range-registration.jsonl',17)]:
    for line in (C/rel).read_text(encoding='utf-8').splitlines():
        r=json.loads(line);ident=r.get('id',r.get('registration_id',''));m=re.search(r'(\d+)$',ident)
        if m and int(m.group(1))>through:pending.append({'registration':ident,'source_id':r['source_id'],'ranges':r.get('material_ranges')})
cp=C/'root/audit-checkpoint-196.json';assert not cp.exists()
checkpoint={'id':'ROOT-AUDIT-CHECKPOINT-196','at':NOW,'previous_goal_turn_classification':'progress','current_progress':'Actual source comparisons117-121, canonical applied damage findingF043 and qualified API arithmetic/capacity findingsF044-F047, fresh Root03 new-role proofs189/191/194 and scopes055-057. No implementation or project execution.','scope':authority['scope'],'variants':len(scope['files']),'mandatory_overlays':authority['mandatory_overlays'],'root04':{'completed':root_count,'findings':finding_count,'authority':pin(authorityp),'latest_packet':pin(C/'root/initial04-invariant-operations/contract-groups-121.json'),'findings_record':pin(fp),'partial_packet117':pin(C/'root/initial04-invariant-operations/contract-groups-117.json'),'status':'incomplete'},'peer_progress':{'native':{'record':pin(nativep),'data':native},'rig':{'record':pin(rigp),'data':rig},'camera':{'record':pin(camerap),'data':camera}},'pending_registered_material':pending,'latest_scope_reconciliation':pin(C/'root/parallel-scope-reconciliation-195.json'),'mandatory_qualification188':pin(C/'root/parallel-scope-qualification-188.json'),'mandatory_qualification193':pin(C/'root/parallel-scope-qualification-193.json'),'delivery':'Finish all13 exhaustive initial audits then another handover per direct user163; no implementation in this continuation.','next':'Root04 remaining exact sources and overlays then08. P16courtship/personality and main autoresolve/combat are complete operations comparisons; next inventory cell is P16Core disease, with fresh global057 world_import681-717 and prior pending overlays also required. Peers finish05/10/13,06/09/12,07/01-02bounded280roles/11 in order. Original774/P16/P17 repairs, final thirteen lenses, independent reviews and hosted same-head validation remain future obligations.','source_edits':0,'project_execution':0,'repairs_allowed':False,'remote_drift':'Live7746223320d09f4691b6de54382d9709b233274d435 not audited by frozen6533 plus recorded integration material; explicit future reconciliation required.'}
cp.write_text(json.dumps(checkpoint,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';progress=read(pp);progress.update(at=NOW,mode='parallel_exhaustive_initial_audits_then_handover',latest_audit_checkpoint=pin(cp))
progress['parallel_assignments']={'root':f'04active{root_count} exact sources through121, {finding_count} findings; then08','native':f"05active{native.get('global_source_variants_compared')} sources/{native.get('qualified_new_findings')} findings at{nativep.name}; then10/13",'rig':f"06active{rig.get('completed_global_source_variants')} distinct/{rig.get('coverage_observations')} observations; then09/12",'camera':f"07active{camera.get('inventoried_variants')} sources; then01/02 bounded280roles,then11"}
progress['goal_turn_progress']={'previous_turn':'Progress: verified and delivered both authorized GitHub coordination replies with explicit agent identity; preserved pending partial-main evidence187.','current_turn':checkpoint['current_progress'],'continuation':checkpoint['next']}
progress['next']=checkpoint['next'];progress['pending_registered_material']=pending
progress['new_scope_frontier']='Through Camera039/Rig036/Native036/Root017 captured057;2826variants+241overlays. Mandatory188 EOF331 and193 premature read exclusion preserved; all prior mandatory qualifications retained. Remote6223 requires future reconciliation. Auditthenhandover163.'
pp.write_text(json.dumps(progress,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    ledger=read(lp);ledger['updated_at']=NOW;ledger['continuation']['current_progress']=pin(pp);ledger['continuation']['audit_checkpoint196']=pin(cp)
    if 'next_action' in ledger:ledger['next_action']=checkpoint['delivery']+' '+checkpoint['next']
    lp.write_text(json.dumps(ledger,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
    assert read(lp)['continuation']['audit_checkpoint196']==pin(cp)
assert len(scope['files'])==2826;assert authority['mandatory_overlays']['through']==241
print(json.dumps({'checkpoint':pin(cp),'progress':pin(pp),'root04':root_count,'findings':finding_count,'native':native.get('global_source_variants_compared'),'rig':rig.get('completed_global_source_variants'),'camera':camera.get('inventoried_variants'),'pending':pending,'ledger_pointers_verified':True,'completion':False}))
