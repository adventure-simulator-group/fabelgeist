import json
from pathlib import Path
O=Path(__file__).resolve().parent
R=O.parent.parent
cat=json.loads((R/'recovered-sources/catalog.json').read_text())['rows']
cm={(x['source_id'],x['expected_sha256']):x for x in cat}
proofs=[]
for fname in ['coverage.jsonl','inherited-coverage.jsonl','source-extensions.jsonl','mandatory-overlay-coverage.jsonl']:
 p=O/fname
 if not p.exists():continue
 for line in p.read_text().splitlines():
  q=json.loads(line); contract=q.get('contract',q.get('contract_ref'))
  if not q.get('source_id') or not q.get('sha256') or not contract:continue
  rs=q.get('ranges',q.get('actual_ranges',q.get('actual_fresh_or_compared_ranges',[])))
  proofs.append({'source_id':q['source_id'],'sha256':q['sha256'],'ranges':rs,'contract':contract,'ledger':fname,'status':q.get('status',q.get('disposition',q.get('limits')) )})
def merge(rs):
 out=[]
 for a,b in sorted(rs):
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
def sub(need,have):
 out=[]
 for a,b in need:
  pos=a
  for l,u in have:
   if u<pos:continue
   if l>b:break
   if l>pos:out.append([pos,min(b,l-1)])
   pos=max(pos,u+1)
   if pos>b:break
  if pos<=b:out.append([pos,b])
 return out
rows=[]
for line in (R/'audit/continuation/owner-range-extensions.jsonl').read_text().splitlines():
 q=json.loads(line); row=cm.get((q['source_id'],q['sha256']));eof=row['line_count'] if row else None
 p=[x for x in proofs if x['sha256']==q['sha256']];same=[x for x in p if x['source_id']==q['source_id']]
 have=merge([z for x in p for z in x['ranges']]); samehave=merge([z for x in same for z in x['ranges']])
 needed=[[a,min(b,eof)] for a,b in q['material_ranges'] if a<=eof] if eof is not None else q['material_ranges']
 exceeded=[[max(a,eof+1),b] for a,b in q['material_ranges'] if b>eof] if eof is not None else []
 rows.append({'registry':q,'scope_index':row['scope_index'] if row else None,'actual_eof':eof,'needed_present_ranges':needed,'nonexistent_registered_ranges':exceeded,'own09_proofs':p,'same_variant_unread_ranges':sub(needed,samehave),'unread_present_ranges':sub(needed,have),'status':'pending explicit metadata and semantic-context reconciliation; provenance/range inventory only'})
s={'id':'SEGO-INITIAL09-OVERLAY-INVENTORY-710','lens':'audit-enum-shapes','mandatory_records':len(rows),'same_variant_present_range_available':sum(not x['same_variant_unread_ranges'] for x in rows),'exact_immutable_body_present_range_available':sum(not x['unread_present_ranges'] for x in rows),'records':rows,'limits':'Own09 evidence only. Samebyte counterpart proof never imports consumer context or another-lens conclusions. Scope EOF qualifications remain; no nonexistent-line credit. This inventory does not award overlay semantic completion.'}
(O/'overlay-inventory-710.json').write_text(json.dumps(s,indent=2)+'\n')
print({k:v for k,v in s.items() if k!='records'})
print('PENDING',[(x['registry']['id'],x['scope_index'],x['unread_present_ranges']) for x in rows if x['unread_present_ranges']])
