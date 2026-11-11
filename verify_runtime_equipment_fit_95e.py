#!/usr/bin/env python3
"""Verify runtime fit source, historical identities and no product execution."""
import difflib,gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-RUNTIME-EQUIPMENT-FIT-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1482-1487.json');w=load('COVERAGE-L01-ROLES.json');proof=load('RUNTIME-FIT-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
nodes={**shared['tree_objects'],**proof['additional_tree_objects']}
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1482,1488)) and [x['lines'] for x in h['historical_source_roles']]==[143,229,467,394,193,62]
current={};historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1482-1487.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historical[i]=raw
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert (raw==cur)==c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  if raw!=cur:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile=f'historical-{i}/'+c['path'],tofile='95e/'+c['path'])).encode();assert d==(r/c['diff']).read_bytes() and sha(d)==c['diff_sha256']
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
layers=current[1486];selection=s['crates/adventuresim-core/src/equipment_assets/selection.rs'];plan=s['crates/adventuresim-character-creator/src/equipment_layers.rs'];body=current[1483];morph=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/morphs.rs'];generation=current[1484]
assert b'fn key(&self) -> (&\'static str, &\'static str)' in layers and b'self.key().hash(state)' in layers and b'self.key().cmp(&other.key())' in layers
assert b'std::ptr::eq(self.placement, other.placement)' in selection and b'selections.sort();\n        selections.dedup();' in layers
assert b'equipment layer dependencies contain a cycle' in plan and b'equipment placements declare contradictory layer precedence' in plan and b'layers: plan.ancestors(&support)?' in layers
assert b'targets.chunks(count)' in morph and b'if !height.is_finite() || height <= 0.0' in body and b'let texcoords = body_surface_coordinates(&positions)?' in body
assert b'.is_some_and(|pending| &pending.key.shape == shape)' in generation and b'self.models.keys().any(|key| &key.shape == shape)' in generation
assert b'support.selection.item().id' in historical[1486] and b'support.selection.id' in historical[1486] and b'support.selection.selection.item().id' in current[1486] and b'support.selection.selection.definition().id' in current[1486]
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(446,[1483,1484]),(447,[1485,1486,1487])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-211.json']);assert all(next(x for x in receipt['sources'] if x['original_index']==i)['fresh_reads']==[] for i in (1493,1495))
check=json.loads(records['adler/VERIFY-L11-211.json']);assert check['receipt_sha256']==m['adler/READ-L11-211.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-211.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (61,1279,1280,1281));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1279,1280,1281)]==[1482,1483,1486]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']=='ADLER-L11-RUNTIME-EQUIPMENT-FAILURE-OWNERSHIP' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical runtime-fit bodies, Git proof, qualifications and bounded source connections; no product execution.')
