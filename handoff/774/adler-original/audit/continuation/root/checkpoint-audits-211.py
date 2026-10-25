"""Save actual audit evidence and remaining work; no project execution."""
import hashlib,json,re
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def last(p):return json.loads(p.read_text(encoding='utf-8').splitlines()[-1])
ap=C/'root/initial04-invariant-operations/scope-authority-001.json';a=read(ap)
scope=read(B/a['scope']['record'])
count=sum(r['status']=='exact_scoped_current04_operations_manually_compared' for r in a['rows'])
fp=C/'root/initial04-invariant-operations/findings.jsonl';fc=len(fp.read_text(encoding='utf-8').splitlines())
np=sorted((C/'native/initial05-aggregate-states').glob('progress-*.json'))[-1];n=read(np)
rp=C/'rig/initial06-type-ownership/progress.jsonl';r=last(rp)
cmp=C/'camera/initial07-redundant-authority/progress.jsonl';cm=last(cmp)
queuep=C/'bounded01-02-newmaterial-queue-087.json';queue=read(queuep)
pending=[]
for rel,through in [('camera/initial07-redundant-authority/source-range-registration.jsonl',40),('rig/initial06-type-ownership/source-range-registration.jsonl',36),('native/initial05-aggregate-states/source-range-registration.jsonl',45),('root/initial04-invariant-operations/source-range-registration.jsonl',20)]:
    for line in (C/rel).read_text(encoding='utf-8').splitlines():
        row=json.loads(line);ident=row.get('id',row.get('registration_id',''));match=re.search(r'(\d+)$',ident)
        if match and int(match.group(1))>through:pending.append({'registration':ident,'source_id':row['source_id'],'ranges':row.get('material_ranges',row.get('ranges')),'status':'prospective_pending_actual_root03_global_reconciliation'})
oldcp=C/'root/audit-checkpoint-209.json';cp=C/'root/audit-checkpoint-211.json';assert not cp.exists()
out=read(oldcp)
out.update(id='ROOT-AUDIT-CHECKPOINT-211',at=NOW,previous_checkpoint=pin(oldcp),previous_goal_turn_classification='progress',previous_goal_turn_evidence='Immutable209 actual Root03proof206/scope061 and Root04packets129-132/F054-F055, mandatory208, no implementation.',current_progress='Actual Root04packets133-136 complete full P16/PRitemcatalogue+schema comparison with fresh factory/codec support, housing/age roles, exact storefront roles and full affinity/P16stub/exact PRstubprefix. Fourteen additional completed variants475->489; no new unique findings, raw stock-id/kind coherence observation kept qualified rather than declaring procedural items invalid. Mandatory210 corrects P16economy195-200 partialnexthelpercontext. Existing primary GPU edits left untouched; no source implementation/project execution.',scope=a['scope'],variants=len(scope['files']),mandatory_overlays=a['mandatory_overlays'],root04={'completed':count,'findings':fc,'authority':pin(ap),'latest_packet':pin(C/'root/initial04-invariant-operations/contract-groups-136.json'),'findings_record':pin(fp),'finding_qualifications':pin(C/'root/initial04-invariant-operations/finding-qualifications.jsonl'),'status':'incomplete'},peer_progress={'native':{'record':pin(np),'data':n},'rig':{'record':pin(rp),'data':r},'camera':{'record':pin(cmp),'data':cm}},pending_registered_material=pending,latest_scope_reconciliation=pin(C/'root/parallel-scope-reconciliation-207.json'),mandatory_scope_qualifications=queue['mandatory_scope_qualifications'],bounded01_02_queue=pin(queuep),next='Continue exhaustive Root04 exact remaining source/overlay roles then08. First inventory frontier P16physiology; whole PRorganization1251/disease3842 and remaining physiology material still independently incomplete. Keep new Native05 normal time/weather/training propagation findings distinct from API-only extremes and source-specific independent lens comparisons. Peers finish05/10/13,06/09/12,07/01-02bounded293roles/11. New registrations require actual Root03/global reconciliation. Preserve210 economycontext/208 orgcontext+timeEOF/200physiologycontext and earlier mandatory qualifications. All13 initial audits then another handover163; original774/P16/P17 repairs, final13, independent reviews and hosted same-head validation remain future obligations.')
cp.write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp)
p.update(at=NOW,mode='parallel_exhaustive_initial_audits_then_handover',latest_audit_checkpoint=pin(cp),pending_registered_material=pending,next=out['next'])
p['parallel_assignments']={'root':f'04active{count} exact sources through136,{fc} findings; then08','native':f"05active{n.get('global_source_variants_compared')} sources/{n.get('qualified_new_findings')} findings at{np.name}; then10/13",'rig':f"06active{r.get('completed_global_source_variants')} distinct/{r.get('coverage_observations')} observations; then09/12",'camera':f"07active{cm.get('inventoried_variants')} sources; then01/02 bounded293roles,then11"}
p['goal_turn_progress']={'previous_turn':out['previous_goal_turn_evidence'],'current_turn':out['current_progress'],'continuation':out['next']}
p['new_scope_frontier']='Through Camera040/Rig036/Native045/Root020 captured061;2836variants+244overlays. Mandatory210 P16economy partialnexthelpercontext,208 orgcontext+PRtimeEOF107,200 physiologycontext and all prior qualifiers retained; no source/semanticcredit fromqualification. Frozen6533+recordedmaterial; lastobservedremote6223 requires future reconciliation. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW;l['continuation']['current_progress']=pin(pp);l['continuation']['audit_checkpoint211']=pin(cp)
    if 'next_action' in l:l['next_action']=out['delivery']+' '+out['next']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
    assert read(lp)['continuation']['audit_checkpoint211']==pin(cp)
assert len(scope['files'])==2836 and a['mandatory_overlays']['through']==244
print(json.dumps({'checkpoint':pin(cp),'progress':pin(pp),'root04':count,'findings':fc,'native':n.get('global_source_variants_compared'),'rig':r.get('completed_global_source_variants'),'camera':cm.get('inventoried_variants'),'pending':pending,'ledger_pointers_verified':True,'completion':False}))
