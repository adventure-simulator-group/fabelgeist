"""Capture actual audit work and remaining obligations; no project execution."""
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
for rel,through in [
    ('camera/initial07-redundant-authority/source-range-registration.jsonl',40),
    ('rig/initial06-type-ownership/source-range-registration.jsonl',36),
    ('native/initial05-aggregate-states/source-range-registration.jsonl',45),
    ('root/initial04-invariant-operations/source-range-registration.jsonl',20)]:
    for line in (C/rel).read_text(encoding='utf-8').splitlines():
        row=json.loads(line);ident=row.get('id',row.get('registration_id',''))
        match=re.search(r'(\d+)$',ident)
        if match and int(match.group(1))>through:
            pending.append({'registration':ident,'source_id':row['source_id'],'ranges':row.get('material_ranges',row.get('ranges')),'status':'prospective_pending_actual_root03_global_reconciliation'})
fq=C/'root/initial04-invariant-operations/finding-qualifications.jsonl'
qid='ROOT-INITIAL04-F055-DEDUP-QUAL209'
assert not any(json.loads(l).get('id')==qid for l in fq.read_text(encoding='utf-8').splitlines())
qualification={'id':qid,'at':NOW,'finding_id':'ROOT-INITIAL04-F055','contract_ref':'ROOT-INITIAL04-CONTRACT-132','qualification':'Native05 existing-proof dedup reply confirms no exact prior Nativefinding for unframed representative pair preimage. Resolves pending peerreply in immutable132 without new source coverage or changed counterexample. API-only: actual normal IDgrammar/caller/DBcollision unproved. All original scope/context qualification208 retained.','new_semantic_source_coverage':0,'finding_count_delta':0}
with fq.open('a',encoding='utf-8') as stream:stream.write(json.dumps(qualification)+'\n')
oldcp=C/'root/audit-checkpoint-205.json'
cp=C/'root/audit-checkpoint-209.json';assert not cp.exists()
out=read(oldcp)
out.update(
    id='ROOT-AUDIT-CHECKPOINT-209',at=NOW,previous_checkpoint=pin(oldcp),
    previous_goal_turn_classification='progress',
    previous_goal_turn_evidence='Immutable205 records actual Root04 packets125-128/F049-F053 plus Root03 nativeproof203/scope060, preserving auditthenhandover stage; no implementation.',
    current_progress='Actual Root03 literal nine P16puzzle production roles and direct DBreceiver snippets proof206, global061 eight newvariants/oneoverlay with exact production qualifications. Root04 independently operation-compared new puzzle/vocabulary roles and original PRfixtures in129/130, full P16morale + exact narrowPRmorale/security/demographics131, exact P16organization132. New API-only F054 hunger intermediate denominator overflow and F055 unframed identity pair; normalfactory/DBreplay and caller grammar qualifications preserved. Mandatory208 corrects P16org partialtest/header context and PRtime EOF107. No implementation or project execution.',
    scope=a['scope'],variants=len(scope['files']),mandatory_overlays=a['mandatory_overlays'],
    root04={'completed':count,'findings':fc,'authority':pin(ap),'latest_packet':pin(C/'root/initial04-invariant-operations/contract-groups-132.json'),'findings_record':pin(fp),'finding_qualifications':pin(fq),'status':'incomplete'},
    peer_progress={'native':{'record':pin(np),'data':n},'rig':{'record':pin(rp),'data':r},'camera':{'record':pin(cmp),'data':cm}},
    pending_registered_material=pending,
    latest_scope_reconciliation=pin(C/'root/parallel-scope-reconciliation-207.json'),
    mandatory_scope_qualifications=queue['mandatory_scope_qualifications'],
    bounded01_02_queue=pin(queuep),
    next='Continue exhaustive Root04 remaining exact sources/overlays then08; item_catalog/item_catalog_schema remain first inventory frontier, whole PRorganization1251/disease3842 and physiology roles stay independently incomplete. Peers finish05/10/13,06/09/12,07/01-02bounded293roles/11 in order. Any new registrations require prospective source-based Root03/global reconciliation. Preserve208 orgcontext/timeEOF and all previous qualifications including200. All13 initial audits then another handover163; original774/P16/P17 repairs, final13, independent reviews and hosted same-head validation remain future obligations.',
    remote_drift='Last observed remote7746223320d09f4691b6de54382d9709b233274d435 is not audited by frozen6533 plus recorded integration material; actual future remote delta needs reconciliation, no current remote-head claim.')
cp.write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp)
p.update(at=NOW,mode='parallel_exhaustive_initial_audits_then_handover',latest_audit_checkpoint=pin(cp),pending_registered_material=pending,next=out['next'])
p['parallel_assignments']={
    'root':f'04active{count} exact sources through132,{fc} findings; then08',
    'native':f"05active{n.get('global_source_variants_compared')} sources/{n.get('qualified_new_findings')} findings at{np.name}; then10/13",
    'rig':f"06active{r.get('completed_global_source_variants')} distinct/{r.get('coverage_observations')} observations; then09/12",
    'camera':f"07active{cm.get('inventoried_variants')} sources; then01/02 bounded293roles,then11"}
p['goal_turn_progress']={'previous_turn':out['previous_goal_turn_evidence'],'current_turn':out['current_progress'],'continuation':out['next']}
p['new_scope_frontier']='Through Camera040/Rig036/Native045/Root020 captured061;2836variants+244overlays. Mandatory208 orgpartialtest/headercontext+PRtimeEOF107 and200physiologycontext retained, no newsemanticcredit fromqualifications. No cross-lens verdict transfer; frozen6533+recordedmaterial, lastobservedremote6223 needs future reconciliation. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW
    l['continuation']['current_progress']=pin(pp);l['continuation']['audit_checkpoint209']=pin(cp)
    if 'next_action' in l:l['next_action']=out['delivery']+' '+out['next']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
    assert read(lp)['continuation']['audit_checkpoint209']==pin(cp)
assert len(scope['files'])==2836 and a['mandatory_overlays']['through']==244
print(json.dumps({'checkpoint':pin(cp),'progress':pin(pp),'root04':count,'findings':fc,'native':n.get('global_source_variants_compared'),'rig':r.get('completed_global_source_variants'),'camera':cm.get('inventoried_variants'),'pending':pending,'ledger_pointers_verified':True,'completion':False}))
