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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HAIR-GPU-MODULE-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2289.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2289,2290))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert len(bodies[2289].splitlines())==1562 and [x['lines'] for x in roles]==[1562]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
module=bodies[2289];buffers=sources['crates/fabelgeist-hair/src/gpu/buffers.rs'];surface=sources['crates/fabelgeist-hair/src/surface.rs'];strands=sources['crates/fabelgeist-hair/src/strands.rs'];lods=sources['crates/fabelgeist-hair/src/gpu/lods.rs'];context=sources['crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs'];observation=sources['crates/fabelgeist-gpu/src/globals/validation_observation.rs'];error=sources['crates/fabelgeist-hair/src/gpu/error.rs'];lock=sources['Cargo.lock']
for literal in (b'pub struct GpuGroom',b'pub strands: u32',b'pub points: u32',b'let q = positions[s * points + j]',b'let a = attributes[s]',b'if values.len() < words.len()',b'Ok(values[..words.len()].to_vec())',b'neighbors[a as usize].contains(&b)',b'let n = strands.len()',b'strands: n as u32',b'points: strands.points as u32',b'pub fn headless_context()',b'Ok(WgpuContext::from_parts(',b'.poll(wgpu::PollType::wait_indefinitely())'):assert literal in module
assert b'pub positions: Vec<Vec3>' in surface and b'pub triangles: Vec<[u32; 3]>' in surface and b'if data.is_empty()' in buffers and b'BufferUpload::from_elements(&[T::default(); 4])' in buffers
assert b'let mut lo = Vec3::splat(f32::INFINITY)' in strands and b'let mut hi = Vec3::splat(f32::NEG_INFINITY)' in strands and b'Some(bounds) => bounds' in lods and b'Grid::over(lo, hi' in lods
assert b'validation_observation: ValidationObservation::Uncaptured' in context and b'Self::Uncaptured => None' in observation and b'Adapter(#[from] wgpu::RequestAdapterError)' in error and b'Device(#[from] wgpu::RequestDeviceError)' in error
for item in a['native_reference_sources']:
 m=load(item['manifest']);raw=(root/item['archive']).read_bytes();assert sha(raw)==m['sha256']==item['archive_sha256'] and m['sha256'].encode() in lock
 with tarfile.open(root/item['archive'],'r:gz') as tar:body=tar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and b'Result<crate::PollStatus, crate::PollError>' in body
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==22
adler=json.loads(records['adler/READ-L11-365.json']);x=next(r for r in adler['sources'] if r['original_index']==2289);assert x['fresh_reads']==x['scope']==[[1,1562]] and x['sha256']==roles[0]['sha256'] and adler['accepted_unique_count_change']==0 and not adler['current_reconciled']
delegate=json.loads(records['segovb/READ-L12-753.json']);x=next(r for r in delegate['rows'] if r['original_index']==2289);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,1562]] and x['source_sha256']==roles[0]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and delegate['root_personal_rereads']==[] and delegate['accepted_unique_count_change']==0
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (159,160,379,1836,1838):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for ident in (old[159]['contract_id'],old[379]['contract_id'],'REVIEW-HAIR-GPU-COUNT-EXTENT-95E'):
 row=next(x for x in ledger if x['id']==ident);assert name in row['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed';p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified hair GPU module provenance, readback and policy source contracts; no product execution.')
