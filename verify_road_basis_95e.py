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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-ROAD-BASIS-POINTS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2153-2156.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2153:64,2154:299,2155:121,2156:151};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
roads=current[2154];bps=current[2156];terrain=sources['crates/adventuresim-terrain/src/lib.rs'];core=sources['crates/adventuresim-core/src/local_problem.rs'];invest=sources['crates/adventuresim-core/src/investigation.rs'];lock=sources['Cargo.lock']
assert b'let Ok(plan) = terrain.plan(a, b) else {' in roads and b'*evaluations.entry(candidate.from).or_default() += 1' in roads and b'world.report.inferred_road_edges = accepted.len()' in roads
assert b'pub fn plan(&self, start: (f64, f64), goal: (f64, f64))' in terrain and b'RoutingWeather::default(), None' in terrain and b'Err(Error::NoRoute) => continue' in terrain and b'Err(error) => return Err(error)' in terrain and b'self.cell(latitude - y * step_lat, longitude + x * step_lon)?' in terrain
assert b'#[serde(transparent)]' in bps and b'pub struct UnitBasisPoints {' in bps and b'pub struct SignedUnitBasisPoints {' in bps and b'if value <= BASIS_POINTS_PER_WHOLE' in bps and b'basis_points: BASIS_POINTS_PER_WHOLE - self.basis_points' in bps and b'pub struct BasisPoints(UnitBasisPoints)' in invest and b'mitigation.complement().as_unit_f32()' in core
assert b'name = "serde_derive"\nversion = "1.0.229"' in lock and b'e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348' in lock
native=a['native_semantics_evidence'];arc=(root/native[0]['archive']).read_bytes();assert sha(arc)==native[0]['archive_sha256']
with tarfile.open(root/native[0]['archive'],'r:gz') as t:member=t.extractfile(native[0]['member']).read()
assert sha(member)==native[0]['member_sha256'] and b'if cont.attrs.transparent()' in member and b'quote!(#member: __transparent)' in member and b'|__transparent| #this_value { #(#assign),* }' in member and native[0]['personally_inspected_ranges']==[[306,323],[382,423]]
ref=contents(native[1]['path']);assert sha(ref)==native[1]['sha256_uncompressed'] and b'Integer operators will panic when they overflow' in ref and native[1]['personally_inspected_ranges']==[[335,349]] and 10001>10000
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed' and not a['new_current_candidates']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
path='segovb/READ-L12-718.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-718.json']);assert [x['original_index'] for x in data['rows']]==list(lengths) and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for row in data['rows']:
 i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
 if 'exact_locator_qualification' in row:assert row['exact_locator_qualification'] in records
assert all('exact_locator_qualification' in next(x for x in data['rows'] if x['original_index']==i) for i in (2153,2155))
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-337.json']);assert data['accepted_unique_count_change']==0
for i in lengths:
 row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (130,131,1802,1803):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==old[i]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, original findings, pinned Serde source and terrain error trace; no product execution.')
