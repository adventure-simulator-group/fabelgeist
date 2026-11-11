#!/usr/bin/env python3
"""Verify bounded weapon-frame source evidence without running product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-WEAPON-FRAMES-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2105-2117.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2105:161,2106:274,2107:241,2108:90,2109:35,2110:76,2111:109,2112:9,2113:273,2114:365,2115:156,2116:137,2117:141};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
base='crates/adventuresim-weapon-model/src/';profile=sources[base+'recipe/blade_profile.rs'];mounts=sources[base+'model/mounts.rs'];extents=sources[base+'model/extents.rs'];validation=sources[base+'validation.rs'];blade=current[2106];transitions=current[2111];model=current[2113];out=current[2114];placement=current[2115];anchors=current[2116]
assert b'(self.section == BladeCrossSection::Recessed) == self.fuller.is_some()' in profile and b'p.validate().map_err(|e| e.to_string())?' in blade and b'super::groove_sections::section(p.fuller.unwrap()' in blade
assert b'for t in [0.25, 0.5, 0.75]' in blade and b'depth >= f64::MANTISSA_DIGITS as usize' in blade and b'construction_budget(((out.len() + 1) * left.len() * 2) as f64)?' in blade and b'for _ in 0..=cutoffs.len()' in transitions and b'if inside.is_empty()' in transitions
assert b'graph.frames.insert(WEAPON_ROOT_FRAME.into(), [0.0; 3])' in placement and b'(GRIP_TOP_FRAME, range[1])' in anchors and b'rotations.insert(id.to_owned(), rotation)' in anchors and b'*rotations.get(owner).unwrap_or(&[0.0; 3])' in anchors and b'let Some(parent) = parents.iter().find(|p| p.id == owner).or(implicit.as_ref()) else' in mounts and b'Self::Grip(p) => [0.0, p.length.get()]' in extents
assert b'recipe.validate().map_err(|error| error.to_string())?' in model and b'.map_err(|error| vec![ValidationError::Construction(error.to_string())])' in validation and b'smoothing_cosine: 0.25' in out and b'policy.map_or(source.smoothing_cosine, |p| p.cosine)' in out and b'dot(neighbor, face) > cosine.max(limit)' in out
prior1=load(a['prior_current_evidence'][0]);prior2=load(a['prior_current_evidence'][1]);assert prior1['source_revision']==prior2['source_revision']==rev
oldhist=load(prior2['historical_reads']);holderrow=next(x for x in oldhist['historical_source_roles'] if x['original_index']==2099);holder=contents(holderrow['current_comparison']['preserved_bytes']);assert holderrow['current_comparison']['git_blob']==tree[holderrow['current_comparison']['path']] and b'_crease: f64' in holder and b'"scabbard-body"' in holder and b'0.82' in holder and b'0.65' in holder
meshrow=next(x for x in oldhist['historical_source_roles'] if x['original_index']==2104);mesh=contents(meshrow['current_comparison']['preserved_bytes']);assert b'#[error("invalid weapon design")]' in mesh and b'Invalid(Vec<ValidationError>)' in mesh
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((707,list(range(2105,2109))),(708,list(range(2109,2114))),(709,list(range(2114,2118)))):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
  if 'exact_locator_qualification' in row:assert row['exact_locator_qualification'] in records
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((329,range(2105,2115)),(330,range(2115,2118))):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (122,123,125,127,1791,1792,1793,1794):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==old[i]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified thirteen historical roles, original reviewer scopes, frame and error source links; no product execution.')
