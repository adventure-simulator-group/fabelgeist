"""Observe explicit owner-lens evidence; no source inspection or execution."""
from pathlib import Path
import json,hashlib,datetime,collections
A=Path(__file__).resolve().parent;mp=A/'audit-ledger.json';m=json.loads(mp.read_text());lens=next(r for r in m['lenses'] if r['lens']=='audit-existing-type-owners');D=A/'wave-01/audit-existing-type-owners';evidence={};observed={};candidates=[]
def read(p,lines=False):
 b=p.read_bytes();evidence[str(p.relative_to(A))]={'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
 return [json.loads(x) for x in b.splitlines() if x.strip()] if lines else json.loads(b)
for owner in ['root','camera','rig','native']:
 p=D/owner/'coverage.jsonl'
 if p.exists():
  for r in read(p,True):
   status=r.get('status',r.get('discovery_status',r.get('lens_status',r.get('discovery','incomplete'))))
   # Only explicit current-lens inventory completion counts, never a full read.
   complete=(owner=='root' and r.get('discovery_status')=='complete') or (owner=='camera' and r.get('status') in ['inventory_accounted','inventory_accounted_with_findings']) or (owner in ['rig','native'] and r.get('inventory_status')=='accounted-current-lens')
   if not r.get('read_ranges') and r.get('read_status') not in ['native_png_header_and_full_blob_fingerprint']:
    continue
   observed[r['source_id']]={'discovery':'complete_inventory_with_dispositions' if complete else 'incomplete_owner_fit','source_read':r.get('read_status',r.get('discovery')),'read_ranges':r.get('read_ranges',[]),'observed_sha256':r.get('observed_sha256'),'observed_baseline_sha256':r.get('observed_baseline_sha256'),'evidence':str(p.relative_to(A)),'contract_refs':r.get('contract_refs',[]),'explicit_shard_status':status}
 p=D/owner/'findings.jsonl'
 if p.exists():
  for r in read(p,True):
   candidates.append({'id':r.get('id',r.get('finding_id')),'evidence_owner':owner,'source_evidence':str(p.relative_to(A)),'disposition':r.get('disposition',r.get('status')),'dedup':r.get('dedup'),'repair':'frozen_until_all13_initial_inventories','validation':'not_established'})
 p=D/owner/'progress.json'
 if p.exists():read(p)
for cell in lens['cells']:
 if cell['source_id'] in observed:cell.update(observed[cell['source_id']])
 elif cell.get('source_read') in ['unread',None] and not cell.get('read_ranges'):
  # Remove only stale unread metadata from the earlier observer; never erase proof.
  cell['discovery']='incomplete';cell['source_read']='unread'
lens['evidence_shard_observation']=evidence;lens['discovery']='incomplete';lens['owner_fit_findings']=candidates;lens['progress_counts']={'recorded_source_variants':len(observed),'explicit_inventory_accounted':sum(c['discovery']=='complete_inventory_with_dispositions' for c in lens['cells']),'total_scoped_variants':len(lens['cells']),'coverage_status':'incomplete; reads and candidate matches alone do not complete connected owner fit','per_owner_recorded':dict(collections.Counter(c['owner'] for c in lens['cells'] if c['source_id'] in observed))}
first=next((c for c in lens['cells'] if c['discovery']!='complete_inventory_with_dispositions'),None);m['first_unfinished_cell']=('wave-01/audit-existing-type-owners/'+first['owner']+'/'+first['source_id']) if first else 'wave-01/audit-existing-type-owners/reached-contract-provenance-reconciliation';m['updated_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();mp.write_text(json.dumps(m,indent=2)+'\n');print(json.dumps({'lens':lens['lens'],'progress':lens['progress_counts'],'candidate_records':len(candidates),'unique_candidate_ids':len({(x['evidence_owner'],x['id']) for x in candidates}),'repairs_allowed':m['discovery_gate']['repairs_allowed'],'project_commands':False}))
