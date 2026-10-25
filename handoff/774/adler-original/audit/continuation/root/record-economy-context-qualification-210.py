"""Preserve a reached source's original bounds while identifying helper context."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];B=C.parent;NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
src=B.parent/'774-snapshots/p16/crates/adventuresim-core/src/settlement_economy.rs'
assert hashlib.sha256(src.read_bytes()).hexdigest()=='5ebaaf60d766322d2dc8bbd0cf1a54b0c790793a11d1d3f7b98e139861a3c54e'
assert len(src.read_text(encoding='utf-8').splitlines())==552
dest=C/'root/parallel-scope-qualification-210.json';assert not dest.exists()
record={'id':'ROOT-PARALLEL-SCOPE-QUALIFICATION-210','at':NOW,'source_id':'physical-p16:crates/adventuresim-core/src/settlement_economy.rs','source_sha256':'5ebaaf60d766322d2dc8bbd0cf1a54b0c790793a11d1d3f7b98e139861a3c54e','actual_EOF':552,'original_scoped_union_retained':[[1,200],[233,250]],'actual_material_operation_ranges':[[1,194],[233,250]],'context_only_ranges':[[195,200]],'fresh_actual_source_facts':'Fresh literal exactP16 originalunion after originalscope metadata inspection. Storefront/service/tab/navigation operations end181, CatalogKind declaration183-194.195blank and196-200 only start next item_stock_category helper through a partialmatch; no wholehelper201-232 or storefront_stocks251+ body credit. PRcounterpart originalfull552 remainsfull, separately actuallycompared. Empty full sourcediff doesnot enlarge P16role.','qualification':'All original raw reachedranges/provenance retained, context cannot clear a whole partially displayed nextfunction. No original requested whole source scope narrowed, new source/semanticcoverage or other-lens verdict. Prior mandatory qualifications including200/208 retained.','all_lenses_mandatory':True,'source_edits':0,'project_execution':0}
dest.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
queuep=C/'bounded01-02-newmaterial-queue-087.json';queue=read(queuep)
queue['mandatory_scope_qualifications'].append(pin(dest));queue['updated_at']=NOW
queuep.write_text(json.dumps(queue,indent=2)+'\n',encoding='utf-8')
ap=C/'root/initial04-invariant-operations/scope-authority-001.json';a=read(ap);a['mandatory_economy_context_qualification210']=pin(dest)
ap.write_text(json.dumps(a,indent=2)+'\n',encoding='utf-8')
pp=C/'root/admission03-progress-067.json';p=read(pp)
p.update(at=NOW,bounded01_02_queue=pin(queuep),mandatory_scope_qualifications=queue['mandatory_scope_qualifications'],latest_scope_qualification=pin(dest))
p['new_scope_frontier']='061 remains2836variants+244overlays, throughCamera040/Rig036/Native045/Root020. Mandatory210 P16economy195-200 partialnexthelpercontext,208orgcontext/timeEOF,200physiologycontext and all earlier qualifications retained. No newsemanticcredit fromqualification. Auditthenhandover163.'
pp.write_text(json.dumps(p,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:
    l=read(lp);l['updated_at']=NOW
    l['continuation']['current_progress']=pin(pp);l['continuation']['scope_qualification210']=pin(dest)
    if 'current_cumulative_scope' in l:l['current_cumulative_scope']['mandatory_scope_qualifications']=queue['mandatory_scope_qualifications']
    lp.write_text(json.dumps(l,indent=2)+'\n',encoding='utf-8')
for lp in [B/'records/execution-ledger.json',B/'records/final-acceptance-20261021/audit-ledger.json']:assert read(lp)['continuation']['current_progress']==pin(pp)
print(json.dumps({'qualification':pin(dest),'queue':pin(queuep),'progress':pin(pp),'new_source_coverage':0,'ledger_pointers_verified':True}))
