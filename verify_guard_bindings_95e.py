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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GUARD-BINDINGS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2139-2143.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2139:217,2140:153,2141:106,2142:65,2143:140};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
base='crates/adventuresim-weapon-model/src/';graph=current[2143];furniture=current[2139];union=current[2142];assembly=sources[base+'recipe/assembly.rs'];nodes=sources[base+'model/guard_nodes.rs'];placement=sources[base+'model/placement.rs'];extents=sources[base+'model/extents.rs'];anchor=sources[base+'model/placement/anchors.rs'];guard=sources[base+'model/guards.rs'];mod=sources[base+'model/mod.rs'];quantity=sources[base+'recipe/quantities.rs'];validation=sources[base+'recipe/validation.rs'];grip=sources[base+'model/grip.rs'];mount=sources[base+'model/mounts.rs'];construction=sources[base+'construction/mod.rs'];hilt=sources[base+'recipe/hilt.rs']
assert b'pub node_bindings: Option<BTreeMap<String, NodeBinding>>' in hilt and b'pub enum NodeBinding' in assembly and b'offset: Option<[Metres; 3]>' in assembly and b'if value.is_finite()' in quantity
assert b'for point in p.nodes.values()' in graph and b'bounded(value.get())?' in graph and b'NodeBinding::Between { between, t, .. }' in graph and b'offset' not in graph and b'p.binding_order().map_err' in graph
assert b'delta.map_or([0.0; 3], |p| p.map(Metres::get))' in nodes and b'Metres::new(point[0])?' in nodes and b'guard_nodes::resolve(p, frames, offset, rotation)?' in placement and b'graph.frames.insert(WEAPON_ROOT_FRAME.into(), [0.0; 3])' in placement
assert b'.map(|p| p[1].get())' in extents and b'add(rotate([0.0, y, 0.0], rotation), offset)' in anchor and b'MODEL_FRAME_EXTENT: f64 = 20.0' in construction and b'.any(|value| value.abs() > MODEL_FRAME_EXTENT)' in mod and b'[0.0; 3]' in grip
assert b'let Some(parent) = parents.iter().find' in mount and b'.or(implicit.as_ref()) else {' in mount and b'.map(|point| point.map(Metres::get))' in guard and b'2.0 * b[axis]' in guard
assert b'while !pending.is_empty()' in assembly and b'.find(|name| match &bindings[*name]' in assembly and b'.all(|dependency| !pending.contains(dependency))' in assembly and b'pending.remove(&next)' in assembly and b'MAX_RECIPE_BYTES: usize = 65536' in validation and 513*514//2==131841
assert b'p.taper.get() > 0.0' in union and b'p.belly.get() > -1.0' in union and b'pub(super) fn grip(p: &GripParameters)' in furniture
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
path='segovb/READ-L12-715.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-715.json']);assert [x['original_index'] for x in data['rows']]==list(lengths) and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for row in data['rows']:
 i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((334,range(2139,2143)),(335,range(2143,2144))):
 data=json.loads(records[f'adler/READ-L11-{n}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
assert [x['original_index'] for x in json.loads(records['adler/READ-L11-335.json'])['sources'] if 2144<=x['original_index']<=2151]==list(range(2144,2152))
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (126,127,1798,1800):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, original attribution and conditional guard binding traces; no product execution.')
