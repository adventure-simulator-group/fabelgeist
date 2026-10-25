"""Save previously compared partial evidence; no source analysis or full credit."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];A=C/'root/initial04-invariant-operations'
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
recovery=read(C/'root/main-autoresolve-partial-recovery-187.json');authority=read(A/'scope-authority-001.json')
p=A/'contract-groups-117.json';assert not p.exists()
f=recovery['pending_finding'];finding={'id':'ROOT-INITIAL04-F043','kind':'applied_health_channels_repartitioned_with_different_blunt_weight_in_battle_log','source_id':'physical-p16:crates/adventuresim-core/src/autoresolve.rs','locations':[[542,558],[1125,1135]],'case':f['example'],'operation':f['problem'],'qualification':f['qualification'],'connected_repair':f['repair'],'acceptance':'Logged applied cut/blunt and resulting durable injury composition agree with the actual applied canonical channels, including mixed positive inputs; existing fixtures only, no added cases/assertions/helpers.','same_graph_numeric_support':f['same_graph_numeric_support'],'deduplication':f['dedup'],'contract_ref':'ROOT-INITIAL04-CONTRACT-117'}
assert not any(json.loads(l)['id']==finding['id'] for l in (A/'findings.jsonl').read_text(encoding='utf-8').splitlines())
packet={'id':'ROOT-INITIAL04-CONTRACT-117','at':datetime.now(timezone.utc).isoformat(),'lens':'audit-invariant-operations','scope':authority['scope'],'mandatory_overlays':authority['mandatory_overlays'],'partial_source_roles':[recovery['source']],'support_only_source_roles':recovery['support_only_literal_ranges_already_compared'],'counterpart_diff_only':recovery['counterpart'],'groups':recovery['preserved_prefix_qualifications'],'findings':[finding],'source_edits':0,'project_execution':0,'validation':'not_run','other_lens_credit':False,'full_source_credit':False,'recovered_evidence':pin(C/'root/main-autoresolve-partial-recovery-187.json')}
p.write_text(json.dumps(packet,indent=2)+'\n',encoding='utf-8')
with (A/'findings.jsonl').open('a',encoding='utf-8') as out:out.write(json.dumps(finding)+'\n')
with (A/'coverage.jsonl').open('a',encoding='utf-8') as out:out.write(json.dumps({'source_id':finding['source_id'],'actual_compared_ranges':[[1,1410]],'contract_ref':packet['id'],'status':'partial_actual04_comparison_no_whole_source_credit','sha256':recovery['source']['sha256']})+'\n')
row=next(r for r in authority['rows'] if r['source_id']==finding['source_id']);assert row['status']!='exact_scoped_current04_operations_manually_compared'
row.setdefault('evidence',[]).append(pin(p));row['partial_compared_ranges']=[[1,1410]]
(A/'scope-authority-001.json').write_text(json.dumps(authority,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'packet':pin(p),'complete_exact_sources':sum(r['status']=='exact_scoped_current04_operations_manually_compared' for r in authority['rows']),'findings':43,'main_complete':False}))
