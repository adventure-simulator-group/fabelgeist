"""Preserve exact scope and boundary qualifications, never award coverage."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
qp=C/'native/initial05-aggregate-states/source-qualifications.jsonl'
qs=[json.loads(l) for l in qp.read_text(encoding='utf-8').splitlines()]
q=next(x for x in qs if x.get('id')=='NATIVE-INITIAL05-QUAL073')
timep=B.parent/'774-snapshots/pr-working/crates/adventuresim-stdb-module/src/time/mod.rs'
assert hashlib.sha256(timep.read_bytes()).hexdigest()=='00f1fa799283c10da8bc5b3310a04c43d117753258629c791c379b6485f25f3f'
assert len(timep.read_text(encoding='utf-8').splitlines())==107
orgp=B.parent/'774-snapshots/p16/crates/adventuresim-core/src/organization.rs'
assert hashlib.sha256(orgp.read_bytes()).hexdigest()=='cb410372fbb5a1f8b61fad2bf447bcd93512725560fb69338ec024ed3135b0e1'
dest=C/'root/parallel-scope-qualification-208.json';assert not dest.exists()
record={
    'id':'ROOT-PARALLEL-SCOPE-QUALIFICATION-208','at':NOW,
    'source_boundaries':[
        {'source_id':'physical-p16:crates/adventuresim-core/src/organization.rs','source_sha256':'cb410372fbb5a1f8b61fad2bf447bcd93512725560fb69338ec024ed3135b0e1','actual_EOF':1251,'original_scoped_union_retained':[[1,36],[88,246],[316,645]],'material_operation_ranges':[[1,35],[89,246],[317,632]],'context_only_ranges':[[36,36],[88,88],[316,316],[633,645]],'facts':'Fresh actual Root04 literal exact original scoped union. line36 is derive attribute for excluded SecondPersonRegister declaration, not body. 88/316/633 blanks;634cfg(test)/635module/636import then partial original assertions through645. Original supplier exclusions saying tests646+ do not mean tests start646: actual tests start634. No37-87/247-315/646+ implementation or whole1251/complete fixture credit. Raw original ranges retained, context cannot award excluded body.'},
        {'source_id':'pr-working:crates/adventuresim-stdb-module/src/time/mod.rs','source_sha256':'00f1fa799283c10da8bc5b3310a04c43d117753258629c791c379b6485f25f3f','actual_EOF':107,'inherited_requested_endpoint':108,'qualification':'PR line108 does not exist; max actual body107. This says nothing about P16time/mod.rs which separately has real108. Source length/hash independently checked here; Native073 records actual source semantics, no fresh Root03/04 body coverage inferred.'}
    ],
    'source_qualification_provenance':dict(pin(qp),id=q['id'],row_1based=qs.index(q)+1),
    'raw_native_qualification':q,
    'qualification':'All original requested whole-source scopes and raw reached-range provenance retained. Only actual body/material semantics count; context/nonexistent endpoints give no body/test/function/wholefile credit. No new scope, semantic coverage or previous-lens clearance. All prior mandatory qualifications includingRoot200 remain.',
    'all_lenses_mandatory':True,'source_edits':0,'project_execution':0}
dest.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
queuep=C/'bounded01-02-newmaterial-queue-087.json';queue=read(queuep)
queue['mandatory_scope_qualifications'].append(pin(dest));queue['updated_at']=NOW
queuep.write_text(json.dumps(queue,indent=2)+'\n',encoding='utf-8')
ap=C/'root/initial04-invariant-operations/scope-authority-001.json';a=read(ap)
a['mandatory_boundary_qualification208']=pin(dest)
ap.write_text(json.dumps(a,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp)
p.update(at=NOW,bounded01_02_queue=pin(queuep),mandatory_scope_qualifications=queue['mandatory_scope_qualifications'],latest_scope_qualification=pin(dest))
p['new_scope_frontier']='061 remains2836variants+244overlays, throughCamera040/Rig036/Native045/Root020. Mandatory208 qualifies P16organization partialtest/header context and PRtime actualEOF107, without altering original requested scopes or awarding new body coverage. All previous qualifications includingRoot200 retained. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW
    l['continuation']['current_progress']=pin(pp);l['continuation']['scope_qualification208']=pin(dest)
    if 'current_cumulative_scope' in l:l['current_cumulative_scope']['mandatory_scope_qualifications']=queue['mandatory_scope_qualifications']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    assert read(lp)['continuation']['current_progress']==pin(pp)
print(json.dumps({'qualification':pin(dest),'queue':pin(queuep),'progress':pin(pp),'new_source_coverage':0,'ledger_pointers_verified':True}))
