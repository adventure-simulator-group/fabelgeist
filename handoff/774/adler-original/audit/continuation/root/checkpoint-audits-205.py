"""Record actual initial audit progress and outstanding work; metadata only."""
import hashlib,json,re
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent
NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def last(p):return json.loads(p.read_text(encoding='utf-8').splitlines()[-1])
ap=C/'root/initial04-invariant-operations/scope-authority-001.json'
a=read(ap);scope=read(B/a['scope']['record'])
count=sum(r['status']=='exact_scoped_current04_operations_manually_compared' for r in a['rows'])
fp=C/'root/initial04-invariant-operations/findings.jsonl'
fc=len(fp.read_text(encoding='utf-8').splitlines())
np=sorted((C/'native/initial05-aggregate-states').glob('progress-*.json'))[-1]
n=read(np)
rp=C/'rig/initial06-type-ownership/progress.jsonl';r=last(rp)
cmp=C/'camera/initial07-redundant-authority/progress.jsonl';cm=last(cmp)
pending=[]
for rel,through in [
    ('camera/initial07-redundant-authority/source-range-registration.jsonl',40),
    ('rig/initial06-type-ownership/source-range-registration.jsonl',36),
    ('native/initial05-aggregate-states/source-range-registration.jsonl',36),
    ('root/initial04-invariant-operations/source-range-registration.jsonl',20)]:
    for line in (C/rel).read_text(encoding='utf-8').splitlines():
        row=json.loads(line);ident=row.get('id',row.get('registration_id',''))
        match=re.search(r'(\d+)$',ident)
        if match and int(match.group(1))>through:
            pending.append({'registration':ident,'source_id':row['source_id'],'ranges':row.get('material_ranges',row.get('ranges')),'status':'registered_not_yet_globally_reconciled_no_other_lens_credit'})
oldcp=C/'root/audit-checkpoint-199.json'
cp=C/'root/audit-checkpoint-205.json';assert not cp.exists()
out=read(oldcp)
out.update(
    id='ROOT-AUDIT-CHECKPOINT-205',at=NOW,previous_checkpoint=pin(oldcp),
    previous_goal_turn_classification='progress',
    previous_goal_turn_evidence='Direct user GitHub coordination request completed by two posted/readback-verified replies explicitly identified as Adler Codex agent. Receipt github-coordination-20261022/receipt.json retained; no duplicate comments posted in current continuation.',
    current_progress='Actual Root04 packets125-128 independently compare errantry, exact P16clue owner, full filth pair, full PRordered generator/validator/vocabulary and native/application culling contracts. F049 rawclue, F050 terminal inclusive blood window, F051 rawfilth total, F052 duplicate taggedsoap budget, F053 fully folded floatceil dispatch gap qualified as API/terminal or native supplier obligations; normal-path safeguards preserved. Fresh native Root03proof203 and scope060 published. No implementation or project execution.',
    scope=a['scope'],variants=len(scope['files']),mandatory_overlays=a['mandatory_overlays'],
    root04={'completed':count,'findings':fc,'authority':pin(ap),'latest_packet':pin(C/'root/initial04-invariant-operations/contract-groups-128.json'),'findings_record':pin(fp),'status':'incomplete'},
    peer_progress={'native':{'record':pin(np),'data':n},'rig':{'record':pin(rp),'data':r},'camera':{'record':pin(cmp),'data':cm}},
    pending_registered_material=pending,
    latest_scope_reconciliation=pin(C/'root/parallel-scope-reconciliation-204.json'),
    mandatory_qualification200=pin(C/'root/parallel-scope-qualification-200.json') if (C/'root/parallel-scope-qualification-200.json').exists() else pin(C/'root/physiology-scope-qualification-200.json'),
    next='Root04 remaining exact sources/overlays then08; next inventory frontier P16Core item_catalog/item_catalog_schema. Reconcile Native037-045 direct P16puzzle registrations with actual Root03 source proof before globally crediting; do not extrapolate PR bodies or wholeP16fixtures. Whole PRdisease3842 and physiology scoped roles remain independently incomplete. Peers finish05/10/13,06/09/12,07/01-02bounded284roles/11 in order. All13 initial audits then handover163. Original774/P16/P17 repairs, final13, independent reviews and hosted same-head validation remain future obligations.',
    remote_drift='Last observed remote7746223320d09f4691b6de54382d9709b233274d435 is not audited by frozen6533 plus recorded integration material; actual future remote delta needs reconciliation, no current remote-head claim.')
cp.write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp)
p.update(at=NOW,mode='parallel_exhaustive_initial_audits_then_handover',latest_audit_checkpoint=pin(cp),pending_registered_material=pending,next=out['next'])
p['parallel_assignments']={
    'root':f'04active{count} exact sources through128,{fc} findings; then08',
    'native':f"05active{n.get('global_source_variants_compared')} sources/{n.get('qualified_new_findings')} findings at{np.name}; then10/13",
    'rig':f"06active{r.get('completed_global_source_variants')} distinct/{r.get('coverage_observations')} observations; then09/12",
    'camera':f"07active{cm.get('inventoried_variants')} sources; then01/02 bounded284roles,then11"}
p['goal_turn_progress']={'previous_turn':out['previous_goal_turn_evidence'],'current_turn':out['current_progress'],'continuation':out['next']}
p['new_scope_frontier']='Through Camera040/Rig036/Native036/Root020 captured060;2828variants+243overlays. Native037-045 prospectively registered pending Root03/global reconciliation. All mandatory qualifications includingRoot200 retained; no cross-lens verdict transfer. Frozen6533+recordedmaterial, lastobservedremote6223 requires future delta reconciliation. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW
    l['continuation']['current_progress']=pin(pp)
    l['continuation']['audit_checkpoint205']=pin(cp)
    if 'next_action' in l:l['next_action']=out['delivery']+' '+out['next']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
    assert read(lp)['continuation']['audit_checkpoint205']==pin(cp)
assert len(scope['files'])==2828 and a['mandatory_overlays']['through']==243
print(json.dumps({'checkpoint':pin(cp),'progress':pin(pp),'root04':count,'findings':fc,'native':n.get('global_source_variants_compared'),'rig':r.get('completed_global_source_variants'),'camera':cm.get('inventoried_variants'),'pending':pending,'ledger_pointers_verified':True,'completion':False}))
