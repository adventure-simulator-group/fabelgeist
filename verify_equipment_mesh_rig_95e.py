#!/usr/bin/env python3
"""Verify bounded mesh/rig evidence and prior attribution; never execute product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-EQUIPMENT-MESH-RIG-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1488-1494.json');w=load('COVERAGE-L01-ROLES.json');proof=load('MESH-RIG-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);fit=load(proof['additional_shared_tree_objects']);tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
nodes={**shared['tree_objects'],**fit['additional_tree_objects'],**proof['additional_tree_objects']}
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1488,1495)) and [x['lines'] for x in h['historical_source_roles']]==[137,160,77,190,161,89,111]
current={};historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1488-1494.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historical[i]=raw
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert (raw==cur)==c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  if raw!=cur:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile=f'historical-{i}/'+c['path'],tofile='95e/'+c['path'])).encode();assert d==(r/c['diff']).read_bytes() and sha(d)==c['diff_sha256']
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
assert [i for i in current if current[i]!=historical[i]]==[1488,1489,1494]
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
skin=s['crates/fabelgeist-rig/src/skin.rs'];body=s['crates/adventuresim-character-creator/src/runtime_equipment.rs'];clothing=s['crates/adventuresim-character-creator/src/runtime_equipment/clothing.rs'];generator=s['crates/adventuresim-character-creator/src/clothing.rs'];validation=s['crates/adventuresim-character-creator/src/clothing/validation.rs'];seams=s['crates/fabelgeist-armor/src/textile_seams.rs'];textile=s['crates/adventuresim-character-creator/src/device_textile_materials.rs'];mesh=current[1489];rig=current[1491]
assert b'pub weights: [f32; WIDTH]' in skin and b'let sum: f32 = influences.iter().map(|(_, weight)| weight).sum();' in skin and b'result.weights[slot] = weight / sum;' in skin and b'weights,\n        }' in skin
assert b'.weighted_vertex(vertex, armor.joint_weights[vertex])' in mesh and b'.strongest_four()' in mesh and b'Mesh::ATTRIBUTE_JOINT_WEIGHT' in mesh
validate=body.split(b'fn validate(&self)')[1].split(b'fn fit_body(&self)')[0];assert b'self.joint_weights.len() != vertices' in validate and b'is_finite' not in validate
assert b'joint_weights: body.joint_weights.clone()' in clothing and b'joint_indices: body.joint_indices.clone()' in clothing and b'.flat_map(|face| face.iter().copied())' in clothing
assert b'weight >= specification.weight_threshold' in generator and b'>= 2' in generator and b'validation::expand_selected_faces(' in generator and b'validated_placeholder_faces(' in generator and b'Ok(faces.to_vec())' in validation
assert b'.find_map(|component|' in historical[1489] and b'.find(|component|' in mesh and b'component.textile.as_ref()' in mesh
assert b'if !self.components.iter().any(|c| c.textile.is_some())' in seams and b'self.validate_textile_attributes()?' in seams and b'component.vertices.start != vertex_end' in seams and b'component.indices.start != index_end' in seams and b'armor.split_textile_seams()?' in textile
assert b'rig node has multiple parents' in rig and b'invalid skeletal basis' in rig and b'inconsistent skeletal references' in rig and b'for _ in 0..self.nodes.len()' in rig and b'cyclic fitting rig' in rig
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(448,[1488,1489,1490]),(449,[1491,1492,1493]),(450,[1494,1495])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
for who,batch in [('adler',374),('segovb',409),('segovb',540),('segovb',544),('segovb',547)]:
 receipt=json.loads(records[f'{who}/READ-L11-{batch}.json']);check=json.loads(records[f'{who}/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and check['receipt_sha256']==m[f'{who}/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'{who}/STATE-L11-{batch}.json']['sha256']
 for field in ('proposal','agent_verification'):
  if field in receipt:assert receipt[field] in records and sha(records[receipt[field]])==receipt[field+'_sha256']
 for plan in receipt.get('plans',[]):assert plan['path'] in records and sha(records[plan['path']])==plan['sha256']
assert json.loads(records['segovb/READ-L11-540.json'])['source_revision']=='7a9d92ad7357779e9280146c521bd8588b86fcdb'
assert all(json.loads(records[f'segovb/READ-L11-{n}.json'])['source_revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' for n in (544,547))
assert b'rejects zero/nonfinite row sum before strongest_four' in records['segovb/READ-L11-409.json']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (175,1282,1283,1284));old=load('baseline-059/queue.json')['entries'];assert old[175]['contract_id']=='ADLER-L11-RIG-INFLUENCE-NORMALIZATION' and [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1282,1283,1284)]==[1490,1493,1494]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']=='ADLER-L11-RIG-INFLUENCE-NORMALIZATION' for x in ledger)==1 and a['conditional_findings'][0]['accepted_historical_unique_count_change']==0
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified seven historical mesh/rig bodies, Git proof, earlier attribution and bounded source connections; no product execution.')
