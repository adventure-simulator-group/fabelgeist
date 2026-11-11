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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GARMENT-MEASUREMENT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2263-2264.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2263,2265))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2263,2264)]==[791,356]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
measure=bodies[2263];section=bodies[2264];creator=sources['crates/adventuresim-character-creator/src/garment.rs'];drape=sources['crates/adventuresim-character-creator/src/garment/drape.rs'];toolchain=sources['rust-toolchain.toml']
assert b'in **centimetres**' in measure and b'const GRID: f32 = 10.0;' in section and b'tenth of a millimetre' in section and b'(point[0] * GRID).round() as i32' in section
assert b'pub fn measure(mesh: BodyMesh' in measure and b'if mesh.vertices.is_empty() || mesh.faces.is_empty()' in measure and b'if !height.is_finite() || height <= 50.0' in measure and b'marks: *landmarks' in measure
assert b'let hip_joint = (self.marks.hips[0][1] + self.marks.hips[1][1]) / 2.0;' in measure and b'.split_two_legs(hip_joint)' in measure and b'while y > bottom' in measure and b'y -= self.step;' in measure and b'step: (top - floor) * SCAN_STEP' in measure and b'const SCAN_STEP: f32 = 0.002;' in measure
assert b'mesh.vertices[face[0] as usize]' in section and b'mesh.vertices[face[2] as usize]' in section and b'distances.iter().all(|d| *d < 0.0)' in section and b'key(a) != key(b)' in section
assert b'measure::{' not in creator or b'BodyMesh, Landmarks, measure' in creator
assert b'j[0] * 100.0' in creator and b'p.map(|v| v * 100.0)' in creator and b'faces: &input.faces' in creator and b'let body = measured_body(input).context("measuring the character")?;' in drape and b'channel = "nightly-2026-07-13"' in toolchain
native=load('native-evidence/rust-nightly-2026-07-13/MANIFEST.json');entry=next(x for x in native['files'] if x['path']=='reference/expressions/operator-expr.html.gz');reference=contents('native-evidence/rust-nightly-2026-07-13/'+entry['path']);assert entry['sha256']==sha(reference)=='d51cc1160b3f46b63b0189e0bfed536581e30dea52b56b18908ff413b83a758c' and len(reference)==entry['bytes'] and b'Values larger than the maximum integer value' in reference and b'will saturate to the maximum value of the integer type' in reference
for path in ('LICENSE-APACHE','LICENSE-MIT'):
 item=next(x for x in native['files'] if x['path']==path);raw=(root/'native-evidence/rust-nightly-2026-07-13'/path).read_bytes();assert sha(raw)==item['sha256'] and len(raw)==item['bytes']
# Exact integer bounds support the symbolic f32 scenarios; product code is not run.
assert 180*2/1000 < 1 and 1 < 2**35 and 2**60-(2**60-2**36)==2**36
assert 10*(2**28)>2**31-1 and 10*(2**28+32)>2**31-1 and (2**28+32)-(2**28)==32
old_assessment=load('ASSESSMENT-GARMENT-DESIGN-95E.json');assert a['recovered_historical_records']==old_assessment['recovered_historical_records'];transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==31
adler=json.loads(records['adler/READ-L11-360.json']);delegate=json.loads(records['segovb/READ-L12-745.json']);check=json.loads(records['segovb/VERIFY-L12-745.json']);assert adler['accepted_unique_count_change']==0 and delegate['root_personal_rereads']==[] and delegate['delegated_attribution_preserved'] and delegate['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup['segovb/READ-L12-745.json']['sha256']
for i,scope in ((2263,[[1,791]]),(2264,[[1,356]])):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==scope and x['sha256']==roles[i-2263]['sha256'];x=next(r for r in delegate['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==scope and x['source_sha256']==roles[i-2263]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'];assert roles[i-2263]['original_supplier_scopes'][0]['read_ranges']==scope
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (156,315,314,1829):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
key=next(x for x in ledger if x['id']==old[314]['contract_id']);assert key['record']=='ASSESSMENT-GARMENT-DESIGN-95E.json' and name in key['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified measurement provenance, conditional progress and key evidence; no product execution.')
