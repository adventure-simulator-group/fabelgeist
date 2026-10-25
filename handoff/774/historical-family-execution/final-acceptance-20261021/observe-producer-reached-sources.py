"""Index explicit source receipts for later scope reconciliation; no audit verdict."""
import datetime,hashlib,json,pathlib
A=pathlib.Path(__file__).resolve().parent;S=A/'wave-01/producer-supplement-after-owner'
b=json.loads((A/'wave-01/source-scope-after-owner.json').read_text());known={(x['source_id'],x['sha256']) for x in b['files']};groups={};sources=[]
def visit(v,trail=''):
 if isinstance(v,dict):
  sid=v.get('source_id');sha=v.get('sha256',v.get('source_sha256'));ranges=v.get('read_ranges');path=v.get('resolved_path',v.get('path',v.get('source_path')))
  if sid and isinstance(sha,str) and len(sha)==64 and isinstance(ranges,list) and path:
   yield v,trail
  for k,z in v.items():
   if isinstance(z,(dict,list)):yield from visit(z,trail+'/'+str(k))
 elif isinstance(v,list):
  for i,z in enumerate(v):yield from visit(z,trail+'/'+str(i))
for owner in ('root','camera','rig','native'):
 f=S/owner/'traces.jsonl'
 if not f.exists():continue
 data=f.read_bytes();sources.append({'record':str(f.relative_to(A)),'sha256':hashlib.sha256(data).hexdigest()})
 for row,line in enumerate(data.decode().splitlines(),1):
  x=json.loads(line);trace=x.get('id',x.get('trace_id'))
  for p,trail in visit(x):
   sid=p['source_id'];sha=p.get('sha256',p.get('source_sha256'));key=(sid,sha)
   g=groups.setdefault(key,{'source_id':sid,'sha256':sha,'path':p.get('resolved_path',p.get('path',p.get('source_path'))),'in_frozen2279_scope':key in known,'source_role_receipts':[],'status':'Explicit source proof observed; material scope/alias/owner-fit reconciliation pending. No discovery, repair, family or validation credit.'})
   g['source_role_receipts'].append({'supplier':owner,'record':str(f.relative_to(A)),'row':row,'trace':trace,'json_pointer':trail,'read_ranges':p['read_ranges'],'material_contract_ranges':p.get('material_contract_ranges',p['read_ranges']),'role':p.get('role',p.get('contract_trace',p.get('scope'))),'limits':'These are recorded source facts, not all-lens coverage. Whole-body, newly reached role and native boundary dispositions require personal reconciliation. Preserve distinct snapshot contexts even where bytes match.'})
rows=[groups[k] for k in sorted(groups)];f=A/'wave-01/producer-reached-scope-pending.jsonl';f.write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in rows))
m={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'authority':'Derived explicit-receipt index only; audit-ledger sole discovery/repair/validation authority, exact claims sole repair owner. Does not scan project code or add audit clearance.','frozen_scope':'wave-01/source-scope-after-owner.json','frozen_scope_sha256':hashlib.sha256((A/'wave-01/source-scope-after-owner.json').read_bytes()).hexdigest(),'record':str(f.relative_to(A)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'observed_named_source_variants':len(rows),'named_variants_not_in_frozen_scope':sum(not x['in_frozen2279_scope'] for x in rows),'source_receipts':sources,'required_next':'After all producer shard actual queues close, reconcile aliases/fingerprints/material ranges and producer roles; extend authoritative scope and perform owner-fit supplement for newly reached roles before admission lens. A prior samefile/hash source read alone is not owner-fit coverage.','qualifications':['wave-01/source-range-errata.json','wave-01/producer-supplement-after-owner/root/receipt-qualifications.json'],'gate':'incomplete; source repairs frozen until exhaustive initial13 discovery'}
f=A/'wave-01/producer-reached-scope-observation.json';f.write_text(json.dumps(m,indent=2)+'\n')
print(json.dumps({k:m[k] for k in ('record','sha256','observed_named_source_variants','named_variants_not_in_frozen_scope','gate')},indent=2))
