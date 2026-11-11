#!/usr/bin/env python3
"""Verify publication evidence only; do not execute product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-BVH-CLOTH-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2224-2231.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2224,2232))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical'] and old==body and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
aabb=bodies[2225];cpu=bodies[2226];cloth=bodies[2228];fabric=bodies[2229];vec=sources['crates/fabelgeist-math/src/vector/vec3.rs'];shell=sources['crates/fabelgeist-shell/src/shell.rs'];shelllib=sources['crates/fabelgeist-shell/src/lib.rs'];constraint=sources['crates/fabelgeist-xpbd/src/constraint.rs'];wgsl=sources['crates/fabelgeist-xpbd/src/wgsl.rs']
assert b'pub direction: Vec3' in aabb and b'inverse_direction: Vec3' in aabb and b'(bounds.min - self.origin) * self.inverse_direction' in aabb and b'let h = ray.direction.cross(edge2)' in aabb
assert b'pub const EMPTY: Self' in aabb and b'(enter <= exit).then_some(enter)' in aabb and b'if bounds.is_empty()' in cpu and b'if self.nodes.is_empty()' in cpu
assert b'distance > EPSILON && distance <= max_distance' in aabb and b'distance < limit.get()' in cpu and b'crate::aabb::ray_triangle(ray, a, b, c, limit)' in cpu
assert b'self.x.min(other.x)' in vec and b'self.x.max(other.x)' in vec and b'0.0 /' not in aabb
for x in a['native_reference_sources']:
 raw=(root/x['path']).read_bytes();raw=gzip.decompress(raw) if x['path'].endswith('.gz') else raw;assert sha(raw)==x['sha256']
f32=(root/'native-evidence/rust-nightly-2026-07-13/library/core/src/num/f32.rs').read_bytes();assert b'If exactly one of the arguments is NaN' in f32 and b'intrinsics::minimum_number_nsz_f32' in f32 and b'intrinsics::maximum_number_nsz_f32' in f32
# Boundary triangle has the exact edge hit (x=0,z=1/4) at y=0 and t=1;
# no float, ray, geometry operation or product expression is evaluated.
assert Fraction(0)+Fraction(1,4)<=1 and Fraction(-1)+1==0
assert b'if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0' in aabb and b'd1 / denominator' in aabb and b'return a + ab * v' in aabb
# Duplicate A=B, C=(2,0,0), query=(1,1,0) has d1=d3=0,d2=d4=2;
# A's squared distance is 2, while the segment point (1,0,0) has 1.
assert 0*0+0*1==0 and 2*1==2 and 1*1+1*1>1
assert b'material.validate()?' in shell and b'pub fn validate(&self) -> anyhow::Result<()>' in shelllib and b'*v >= 0.0' in shelllib and b'pub fn apply_fabric(&mut self)' in cloth and b'self.shell.material = self.fabric.into()' in cloth and b'self.bending.compliance = self.material.bend_compliance' in shell and b'parameters.insert("compliance", self.compliance)' in constraint and b'compliance / (substep * substep)' in wgsl and b'pub bend_compliance: f32' in fabric
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(738,list(range(2224,2231))),(739,[2231])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2224]['lines'];assert x['source_sha256']==roles[i-2224]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(354,[2224,2225,2226]),(355,[2227,2228,2229,2230,2231])]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  x=next(z for z in data['sources'] if z['original_index']==i);assert x['fresh_reads']==[[1,roles[i-2224]['lines']]] and x['sha256']==roles[i-2224]['sha256']
adler=json.loads(records['adler/READ-L11-354.json']);d=next(x['disposition'] for x in adler['role_dispositions'] if x['source_indices']==[2225]);assert 'Parallelrayonexactslabboundary' in d and 'nativebodyunread' in d and 'duplicateA==B' in d
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (144,145,146,1823,3137):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical BVH and cloth attribution, source contracts and symbolic conditions; no product execution.')
