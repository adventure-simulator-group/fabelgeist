#!/usr/bin/env python3
"""Verify recovered and source evidence without executing product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-SHAFT-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2144-2151.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2144:240,2145:75,2146:137,2147:126,2148:174,2149:42,2150:101,2151:72};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
base='crates/adventuresim-weapon-model/src/';shaft=sources[base+'recipe/shaft.rs'];recipe=sources[base+'recipe/mod.rs'];validation=sources[base+'recipe/validation.rs'];grip=sources[base+'model/grip.rs'];placement=sources[base+'model/placement.rs'];model=sources[base+'model/mod.rs'];solid=sources[base+'construction/solids.rs'];construction=sources[base+'construction/mod.rs'];output=sources[base+'model/output.rs'];wrapping=sources[base+'model/shaft_wrapping.rs'];spear=current[2148];holder=current[2151]
assert b'pub shaft: Option<Shaft>' in recipe and b'(!self.components.is_empty() || self.shaft.is_some())' in validation and b'positive(p.length.get())?' in validation and b'positive(p.radius.get())?' in validation and b'.max(p.top_scale.map_or(0.92, Ratio::get))' in validation and b'p.radius.get() * scale <= MAX_ROUND_GRIP_RADIUS' in validation
assert b'if let Some(wrappings) = &p.wrappings' in spear and b'if let Some(tenon) = &p.tenon' in spear and b'self.radius.get() * self.bottom_scale.map_or(1.0, Ratio::get)' in shaft and b'self.radius.get() * self.top_scale.map_or(0.92, Ratio::get)' in shaft
assert b'let profile = shaft.profile()' in model and b'Solid::lathe(' in model and b'parts.extend(shaft_wrapping::parts' in model and b'for (index, wrapping) in shaft.wrappings.iter().flatten()' in wrapping
assert b'profile.len() < 2 || requested < 3 || radial_scale <= 0.0' in solid and b'if profile[ring][1] > 0.0' in solid and b'if profile[ring + 1][1] > 0.0' in solid and b'if profile[row][1] == 0.0' in solid and b'Ok(solid.positive())' in solid and b'if self.volume() < -1e-10' in construction
assert b'center_of_mass: mul(moment, 1.0 / mass)' in output and b'let center_of_mass = mul(first, 1.0 / mass_kg)' in output
assert b'graph.frames.insert(SHAFT_BOTTOM_FRAME.into(), [0.0; 3])' in placement and b'.insert(SHAFT_TOP_FRAME.into(), [0.0, shaft.length.get(), 0.0])' in placement and b'(length * 0.2).clamp(0.18, 0.45) / length' in grip and 18>10
assert b'Holder("kind")' in holder and b'Holder("parameters")' in holder and b'Holder("source geometry")' in holder and b'ValidationError::Construction(error.to_string())' in holder
ref=a['native_semantics_evidence'][0];reference=contents(ref['path']);assert sha(reference)==ref['sha256_uncompressed'] and ref['personally_inspected_lines']==[1912,1917] and b'NaN</code> will return <code>0</code>' in reference
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
path='segovb/READ-L12-716.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-716.json']);assert [x['original_index'] for x in data['rows']]==list(lengths) and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for row in data['rows']:
 i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-335.json']);assert data['accepted_unique_count_change']==0
for i in range(2144,2151):
 row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
row=next(x for x in data['sources'] if x['original_index']==2151);assert row['fresh_reads']==[] and row['sha256']==sha(current[2151])
disposition=next(x for x in data['role_dispositions'] if 2151 in x.get('source_indices',[]));assert disposition['reuse_proof']['read_ranges']==[[1,72]] and disposition['reuse_proof']['receipt_sha256']==lookup['adler/READ-L11-328.json']['sha256']
old328=json.loads(records['adler/READ-L11-328.json']);prior=next(x for x in old328['connected_reinspection'] if x['original_index']==2151);assert prior['fresh_reads']==[[1,72]] and prior['additional_original_frontier_credit']==0 and prior['source_sha256']==sha(current[2151])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (122,126,1798,1801):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified eight historical roles, prior role2151 reuse and conditional shaft source traces; no product execution.')
