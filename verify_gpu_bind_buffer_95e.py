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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GPU-BIND-BUFFER-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2273-2278.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2273,2279))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==(i!=2273) and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in range(2273,2279)]==[44,303,53,178,89,58] and roles[0]['lines']==43
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
manifest=bodies[2273];bind=bodies[2274];extent=bodies[2275];buffer=bodies[2276];readback=bodies[2277];upload=bodies[2278];shader=sources['crates/fabelgeist-gpu/src/data/gpu/shader.rs'];reflection=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs'];definition=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/definition.rs'];compute=sources['crates/fabelgeist-compute/src/readback.rs'];lock=sources['Cargo.lock'];view_error=sources['crates/fabelgeist-gpu/src/data/gpu/texture/view_error.rs'];validation=sources['crates/fabelgeist-gpu/src/globals/validation_scope.rs']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 oldmanifest=archive.extractfile(roles[0]['sha256']).read();oldlock=archive.extractfile(work['roles'][497]['historical_sha256']).read()
assert b'derive_more' not in oldmanifest and b'derive_more = ' in manifest and b'version = "2.1.1"' in oldlock and b'"derive_more"' in oldlock and b'use derive_more::{Display, Error};' in view_error and b'use derive_more::{Display, Error};' in validation
assert b'pub struct UniformMember' in shader and b'pub offset: u32' in shader and b'pub size: u32' in shader and b'pub uniform_members: Vec<UniformMember>' in shader and b'size: module.types[member.ty].inner.size(module.to_ctx())' in reflection
assert b'PassParameter::Mat3(v)' in bind and b'let col_stride = member.size / 3;' in bind and b'(member.offset as usize + member.size as usize) <= buffer_data.len()' in bind and b'buffer_data[offset..offset + 4]' in bind
# Symbolic bounds only; the product packer and slice are not executed.
size=12;stride=size//3;start=0;assert start+size<=12 and stride==4 and start+1*stride+2*4==12 and 12+4>12
assert b'pub buffer: Arc<wgpu::Buffer>' in buffer and b'pub size: BufferByteLength' in buffer and b'pub usage: wgpu::BufferUsages' in buffer and b'let usage = buffer.usage();' in buffer and b'self.usage.contains(wgpu::BufferUsages::MAP_READ)' in buffer
assert b'data.write_to(context, &self.buffer, BufferByteOffset::START)' in buffer and b'.checked_after(data.length())' in buffer and b'if u64::from(end) > u64::from(self.size)' in buffer and b'context.queue.write_buffer(buffer, u64::from(at), self.0)' in upload
assert b'res.map_err(|e| anyhow!("GPU Mapping error: {:?}", e))?' in buffer and b'.map_err(|_| anyhow!("GPU Mapping error"))?' in buffer and b'let _ = context.device.poll' in buffer and b'Mapping(#[source] wgpu::BufferAsyncError)' in compute and b'Poll(#[source] wgpu::PollError)' in compute
assert b'element_size != 0' in readback and b'byte_len.is_multiple_of(element_size)' in readback and b'bytes.len() >= self.byte_len' in readback and b'copy_nonoverlapping' in readback
native={}
for item in a['native_reference_sources']:
 m=load(item['manifest']);raw=(root/item['archive']).read_bytes();assert sha(raw)==item['archive_sha256']==m['sha256'] and m['sha256'].encode() in lock
 with tarfile.open(root/item['archive'],'r:gz') as tar:body=tar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in item['personally_inspected_ranges']);native[item['member']]=body
wgpu=native['wgpu-29.0.4/src/api/buffer.rs'];queue=native['wgpu-29.0.4/src/api/queue.rs'];assert b'pub fn size(&self) -> BufferAddress' in wgpu and b'pub fn usage(&self) -> BufferUsages' in wgpu and b'pub struct BufferAsyncError;' in wgpu and b'pub fn write_buffer(&self, buffer: &Buffer' in queue
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==79
for batch,indices in ((361,(2273,)),(362,(2274,2275)),(363,(2276,2277,2278))):
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  x=next(r for r in data['sources'] if r['original_index']==i);n=roles[i-2273]['lines'];assert x['fresh_reads']==x['scope']==[[1,n]] and x['sha256']==roles[i-2273]['sha256']
for batch,indices in ((747,(2273,)),(748,(2274,2275,2276,2277,2278))):
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256']
 for i in indices:
  x=next(r for r in data['rows'] if r['original_index']==i);n=roles[i-2273]['lines'];assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]] and x['source_sha256']==roles[i-2273]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in [151,157,158,*range(317,339),352,2331]:assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
parent=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'][old[158]['contract_id']];aliases={x['source_existing_finding_id'] for x in parent['exact_existing_finding_subcontract_alias_applications']};assert {old[334]['contract_id'],old[335]['contract_id']}<=aliases and len(a['explicit_historical_subcontract_aliases'])==2
uniform=next(x for x in ledger if x['id']==old[151]['contract_id']);assert uniform['record']=='ASSESSMENT-KERNEL-95E.json' and name in uniform['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified changed GPU manifest, original error aliases and conditional bind/buffer contracts; no product execution.')
