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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-KERNEL-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2243-2245.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2243,2246))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==(i!=2243) and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert len(bodies[2243].splitlines())==375 and len(bodies[2244].splitlines())==406
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
fast=bodies[2243];kernel=bodies[2244];shader=sources['crates/fabelgeist-gpu/src/data/gpu/shader.rs'];reflection=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs'];standard=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs'];buffer=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs'];extent=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs'];scope=sources['crates/fabelgeist-gpu/src/globals/validation_scope.rs'];pipeline=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs'];rules=sources['crates/AGENTS.md'];lock=sources['Cargo.lock']
assert b'context.validation_observation.start_scope(&context.device)' in fast and b'let _ = context.device.poll(wgpu::PollType::wait_indefinitely())' in fast and b'Kernel Pipeline Creation Error: {error}' in fast
assert b'ProgressWithNative' in scope and b'wgpu::PollError' in scope and b'wgpu::Error' in scope and b'let progress = self.device.poll' in scope
assert b'pub struct KernelCache' in kernel and b'self.kernels.read().unwrap().get(code)' in kernel and b'self.kernels.write().unwrap()' in kernel and b'self.kernels.read().unwrap().len()' in kernel and b'Never use `unwrap` or `expect` outside tests' in rules and b'last().expect("allocated above")' in fast
assert b'UniformMember {' in reflection and b'PassParameter::Unsigned(u)' in fast and b'PassParameter::Vec2(v)' in fast and b'if end <= out.len()' in fast and b'let mut put = |values: &[f32]|' in fast and b'was not supplied' in fast and b'let mut buffer_data = vec![0u8;' in standard
# A scalar at byte 12 of a 16-byte block has only four remaining bytes.
# Integer and float representations of one differ as IEEE-754 bit patterns.
assert 12+2*4>16 and 1!=0x3f800000
assert b'size: std::num::NonZeroU64::new(u64::from(buffer.size))' in fast and b'Some(std::num::NonZeroU64::new(u64::from(gpu_buf.size)).unwrap())' in standard and b'pub size: BufferByteLength' in buffer and b'pub struct BufferByteLength' in extent
assert b'let _ = device.poll(wgpu::PollType::wait_indefinitely())' in pipeline and b'*guard = Some(e.to_string())' in pipeline and b'parse_naga' in shader
assert b'name = "wgpu"\nversion = "29.0.4"' in lock and b'76e8840e1ba2881d4cbb18d2147627a56af426ff064c0401eb0c8410c6325d07' in lock
native=load('native-evidence/wgpu-29.0.4/MANIFEST.json');crate=(root/'native-evidence/wgpu-29.0.4'/native['archive']).read_bytes();assert len(crate)==native['bytes'] and sha(crate)==native['sha256']=='76e8840e1ba2881d4cbb18d2147627a56af426ff064c0401eb0c8410c6325d07'
with tarfile.open(root/'native-evidence/wgpu-29.0.4'/native['archive'],'r:gz') as archive:
 for item in native['inspected_members']:
  data=archive.extractfile(item['path']).read();assert len(data)==item['bytes'] and sha(data)==item['sha256']
  if item['path'].endswith('/src/api/bind_group.rs'):assert b'Size of the binding in bytes, or `None` for using the rest of the buffer.' in data
 assert {'wgpu-29.0.4/LICENSE.APACHE','wgpu-29.0.4/LICENSE.MIT'}<=set(archive.getnames())
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==61
adler=json.loads(records['adler/READ-L11-356.json']);assert adler['accepted_unique_count_change']==0
for i in (2243,2244,2245):
 row=next(x for x in adler['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2243]['lines']]] and row['sha256']==roles[i-2243]['sha256']
for batch,indices in ((740,[2243]),(741,[2244,2245]),(749,[])):
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2243]['lines'];assert x['source_sha256']==roles[i-2243]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
correction=json.loads(records['parallel-l12-20261029/QUALIFICATION-PAR-L12-741-STAGING-TRACE-REFERENCE.json']);assert not correction['source_body_fact_changed'] and correction['new_read_or_population_credit']==0 and correction['historical_receipt_sha256']==lookup[correction['historical_receipt']]['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (149,150,151,293,294,295,296,297,298,299,352,353,360,361,1825,1826):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for key in ('connected_historical_reason_family','connected_historical_shader_family'):
 family=a[key];row=next(x for x in ledger if x['id']==family['ledger_id']);assert [x['queue'] for x in row['named_subcontracts']]==family['queue_indices'] and all(x['id']==old[x['queue']]['contract_id'] for x in row['named_subcontracts']) and row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified kernel historical attribution, source contracts and native archive; no product execution.')
