"""Metadata-only receipt inventory. No source reads, project execution or verdicts."""
import pathlib,json,hashlib,datetime,collections
A=pathlib.Path(__file__).resolve().parent;S=A/'wave-01/producer-supplement-after-owner';frozen=json.loads((A/'wave-01/source-scope-after-owner.json').read_text());snapshots=frozen['snapshots'];roots={x['id']:pathlib.Path(x['root']) for x in snapshots};old=collections.defaultdict(list);byid=collections.defaultdict(list)
def absolute_source(x):
 p=pathlib.Path(x['path']);return str(p if p.is_absolute() else roots[x['snapshot']]/p)
for x in frozen['files']:
 old[(absolute_source(x),x['sha256'])].append(x);byid[x['source_id']].append(x)
groups={};unresolved=[];sources=[]
def walk(x,trail=''):
 if isinstance(x,dict):
  sha=x.get('sha256',x.get('source_sha256'));p=x.get('resolved_path',x.get('source_path',x.get('path')));sid=x.get('source_id');ranges=x.get('read_ranges',x.get('ranges',x.get('contract_ranges')))
  if isinstance(sha,str) and len(sha)==64 and isinstance(ranges,list):
   if p is None and sid:
    candidates=[q for q in byid.get(sid,[]) if q['sha256']==sha]
    if len({absolute_source(q) for q in candidates})==1:p=absolute_source(candidates[0])
    elif ':' in sid and sid.split(':',1)[0] in roots:p=str(roots[sid.split(':',1)[0]]/sid.split(':',1)[1])
   if isinstance(p,str) and p.startswith('/'):
    if 'frozen-inputs.zip!/' in p:
     suffix=p.split('frozen-inputs.zip!/',1)[1];context,rel=suffix.split('/files/',1)
     if context in roots:p=str(roots[context]/rel)
    yield x,trail,p,sha,ranges
   elif sid:unresolved.append({'source_id':sid,'sha256':sha,'json_pointer':trail,'reason':'No unique actual absolute source path in receipt/frozen manifest; manual material-role resolution required'})
  for k,v in x.items():
   if isinstance(v,(dict,list)):yield from walk(v,trail+'/'+str(k))
 elif isinstance(x,list):
  for i,v in enumerate(x):yield from walk(v,trail+'/'+str(i))
for owner in ['root','camera','rig','native']:
 p=S/owner/'traces.jsonl';b=p.read_bytes();sources.append({'record':str(p.relative_to(A)),'sha256':hashlib.sha256(b).hexdigest()})
 for row,l in enumerate(b.decode().splitlines(),1):
  j=json.loads(l);tag=j.get('id',j.get('trace_id'))
  for x,trail,p,sha,ranges in walk(j):
   known=old.get((p,sha),[]);snap=next((z['id'] for z in snapshots if p.startswith(z['root']+'/')),None);sid=x.get('source_id');g=groups.setdefault((p,sha),{'path':p,'sha256':sha,'snapshot':snap,'source_id_aliases':[],'frozen_source_ids':[z['source_id'] for z in known],'in_frozen_by_path_and_fingerprint':bool(known),'proofs':[],'discovery':'pending_personal_material_role_and_range_reconciliation_no_audit_credit'})
   if sid and sid not in g['source_id_aliases']:g['source_id_aliases'].append(sid)
   g['proofs'].append({'supplier':owner,'record':str((S/owner/'traces.jsonl').relative_to(A)),'row':row,'trace':tag,'json_pointer':trail,'ranges':ranges,'material_contract_ranges':x.get('material_contract_ranges',ranges),'role':x.get('role',x.get('scope',x.get('contract_trace'))),'read_status':x.get('mode',x.get('method',x.get('read_kind',x.get('read_status'))))})
rows=[groups[k] for k in sorted(groups)];p=A/'wave-01/producer-role-receipts-expanded-pending.jsonl';p.write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in rows));q=A/'wave-01/producer-role-receipts-expanded-observation.json';report={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'authority':'Metadata-only complete recognized ranges/read_ranges/contract_ranges role index. No source scan, coverage, owner-fit or acceptance credit. Frozen context variants preserved independently, aliases by actual path+fingerprint only.','record':str(p.relative_to(A)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'source_variants':len(rows),'new_actual_path_fingerprints':sum(not x['in_frozen_by_path_and_fingerprint'] for x in rows),'unresolved_path_observations':unresolved,'source_receipts':sources,'required_next':'Reconcile each material role/range and alias personally after actual producer queues close; preserve frozen original changed context variants, extend scope and owner-fit for new roles before admission. Earlier named865/139 index was deliberately incomplete schema and remains historic navigation only.'};q.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['record','sha256','source_variants','new_actual_path_fingerprints']},indent=2));print('unresolved_path_observations',len(unresolved))
