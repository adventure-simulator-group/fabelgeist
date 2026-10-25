"""Refresh persisted audit checkpoint and ledger pointers; no project execution."""
import hashlib,json,re
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def last(p):return json.loads(p.read_text(encoding='utf-8').splitlines()[-1])
ap=C/'root/initial04-invariant-operations/scope-authority-001.json';a=read(ap);scope=read(B/a['scope']['record']);count=sum(r['status']=='exact_scoped_current04_operations_manually_compared' for r in a['rows']);fp=C/'root/initial04-invariant-operations/findings.jsonl';fc=len(fp.read_text(encoding='utf-8').splitlines())
np=sorted((C/'native/initial05-aggregate-states').glob('progress-*.json'))[-1];n=read(np);rp=C/'rig/initial06-type-ownership/progress.jsonl';r=last(rp);cmp=C/'camera/initial07-redundant-authority/progress.jsonl';cm=last(cmp)
pending=[]
for rel,through in [('camera/initial07-redundant-authority/source-range-registration.jsonl',39),('rig/initial06-type-ownership/source-range-registration.jsonl',36),('native/initial05-aggregate-states/source-range-registration.jsonl',36),('root/initial04-invariant-operations/source-range-registration.jsonl',19)]:
    for line in (C/rel).read_text(encoding='utf-8').splitlines():
        row=json.loads(line);ident=row.get('id',row.get('registration_id',''));m=re.search(r'(\d+)$',ident)
        if m and int(m.group(1))>through:pending.append({'registration':ident,'source_id':row['source_id'],'ranges':row.get('material_ranges')})
cp=C/'root/audit-checkpoint-199.json';assert not cp.exists();oldcp=C/'root/audit-checkpoint-196.json';out=read(oldcp)
out.update(id='ROOT-AUDIT-CHECKPOINT-199',at=NOW,previous_checkpoint=pin(oldcp),previous_goal_turn_classification='progress',current_progress='Actual Root04 scoped disease/clinical interpolation122, full NPC/encounter123 and exact settlement import124; new qualified F048 pre-widening interpolation and existing F046/F020/NativeF039 support. Fresh prospective Root018/019 actual Root03proof197 and scope058. No implementation or project execution.',scope=a['scope'],variants=len(scope['files']),mandatory_overlays=a['mandatory_overlays'],root04={'completed':count,'findings':fc,'authority':pin(ap),'latest_packet':pin(C/'root/initial04-invariant-operations/contract-groups-124.json'),'findings_record':pin(fp),'status':'incomplete'},peer_progress={'native':{'record':pin(np),'data':n},'rig':{'record':pin(rp),'data':r},'camera':{'record':pin(cmp),'data':cm}},pending_registered_material=pending,latest_scope_reconciliation=pin(C/'root/parallel-scope-reconciliation-198.json'),next='Root04 remaining exact sources and overlays then08. First current incomplete inventory cell P16Core errantry; whole PR disease3842 and whole physiology roles remain separate incomplete cells despite bounded P16 disease122/helper support. Peers finish05/10/13,06/09/12,07/01-02bounded282roles/11 in order. Reconcile any new registrations prospectively. All13 initial audits then handover163; original774/P16/P17 repairs, final thirteen lenses, independent reviews and hosted same-head validation remain future obligations.',remote_drift='Last observed remote7746223320d09f4691b6de54382d9709b233274d435 is not audited by frozen6533 plus recorded integration material; reconcile actual current remote delta before future acceptance.')
cp.write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp);p.update(at=NOW,mode='parallel_exhaustive_initial_audits_then_handover',latest_audit_checkpoint=pin(cp),pending_registered_material=pending,next=out['next'])
p['parallel_assignments']={'root':f'04active{count} exact sources through124, {fc} findings; then08','native':f"05active{n.get('global_source_variants_compared')} sources/{n.get('qualified_new_findings')} findings at{np.name}; then10/13",'rig':f"06active{r.get('completed_global_source_variants')} distinct/{r.get('coverage_observations')} observations; then09/12",'camera':f"07active{cm.get('inventoried_variants')} sources; then01/02 bounded282roles,then11"}
p['goal_turn_progress']={'previous_turn':'Progress: actual Root04 packets117-121/F043-F047 and source scopes055-057 with mandatory qualifications188/193; no source implementation.','current_turn':out['current_progress'],'continuation':out['next']}
p['new_scope_frontier']='Through Camera039/Rig036/Native036/Root019 captured058;2826variants+243overlays. All mandatory qualifications retained, no cross-lens verdict transfer. Last observed remote6223 requires future delta reconciliation. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW;l['continuation']['current_progress']=pin(pp);l['continuation']['audit_checkpoint199']=pin(cp)
    if 'next_action' in l:l['next_action']=out['delivery']+' '+out['next']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
    assert read(lp)['continuation']['audit_checkpoint199']==pin(cp)
assert len(scope['files'])==2826;assert a['mandatory_overlays']['through']==243
print(json.dumps({'checkpoint':pin(cp),'progress':pin(pp),'root04':count,'findings':fc,'native':n.get('global_source_variants_compared'),'rig':r.get('completed_global_source_variants'),'camera':cm.get('inventoried_variants'),'pending':pending,'ledger_pointers_verified':True,'completion':False}))
