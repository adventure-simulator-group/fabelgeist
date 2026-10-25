"""Publish an existing-range context qualification; no source coverage inferred."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
qp=C/'native/initial05-aggregate-states/source-qualifications.jsonl';qs=[json.loads(l) for l in qp.read_text(encoding='utf-8').splitlines()];q=next(x for x in qs if x.get('id')=='NATIVE-INITIAL05-QUAL060')
source=B.parent/'774-snapshots/p16/crates/adventuresim-core/src/physiology.rs';assert hashlib.sha256(source.read_bytes()).hexdigest()==q['sha256'];assert len(source.read_text(encoding='utf-8').splitlines())==1169
dest=C/'root/parallel-scope-qualification-200.json';assert not dest.exists()
record={'id':'ROOT-PARALLEL-SCOPE-QUALIFICATION-200','at':NOW,'source_id':q['source_id'],'source_sha256':q['sha256'],'source_qualification':dict(pin(qp),id=q['id'],row_1based=qs.index(q)+1),'raw_source_qualification':q,'fresh_actual_boundary_comparison':{'ranges':[[789,790],[850,935]],'facts':'789-790 is disease_meter_state signature only.851-928 actual PresenceSpan/ChartEntry/cadence/quantization bodies.850/929 blanks and930-935 observer-noise docs/attrs; no791-849 or936+ body displayed/credited.'},'qualification':'Original raw scope ranges retained. Actual material1-787/851-929 only; context headers/docs do not award called body or whole function coverage. No source scope expansion/shrinking, new semantic coverage or whole-physiology clearance. Root122 support724-790 includes only header context after787, already no whole-source credit.','all_lenses_mandatory':True,'source_edits':0,'project_execution':0}
dest.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
queuep=C/'bounded01-02-newmaterial-queue-087.json';queue=read(queuep);queue['mandatory_scope_qualifications'].append(pin(dest));queue['updated_at']=NOW;queuep.write_text(json.dumps(queue,indent=2)+'\n',encoding='utf-8')
ap=C/'root/initial04-invariant-operations/scope-authority-001.json';a=read(ap);a['mandatory_physiology_context_qualification200']=pin(dest);ap.write_text(json.dumps(a,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp);p.update(at=NOW,bounded01_02_queue=pin(queuep),mandatory_scope_qualifications=queue['mandatory_scope_qualifications'],latest_scope_qualification=pin(dest));p['new_scope_frontier']='058 remains2826variants+243overlays, through Camera039/Rig036/Native036/Root019. Mandatory200 additionally qualifies existing P16physiology789-790/930-935 as context only, no later body credit. All previous mandatory qualifications retained; auditthenhandover163.';pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW;l['continuation']['current_progress']=pin(pp);l['continuation']['scope_qualification200']=pin(dest)
    if 'current_cumulative_scope' in l:l['current_cumulative_scope']['mandatory_scope_qualifications']=queue['mandatory_scope_qualifications']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:assert read(lp)['continuation']['current_progress']==pin(pp)
print(json.dumps({'qualification':pin(dest),'queue':pin(queuep),'progress':pin(pp),'new_source_coverage':0,'ledger_pointers_verified':True}))
