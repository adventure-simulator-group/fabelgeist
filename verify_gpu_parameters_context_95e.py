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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GPU-PARAMETERS-CONTEXT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2281-2284.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2281,2285))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==(i!=2284) and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2281,2282,2283,2284)]==[42,129,73,323] and [x['lines'] for x in roles]==[42,129,73,314]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
parameters=bodies[2281];parameter=bodies[2282];shader=bodies[2283];context=bodies[2284]
passbody=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs'];fast=sources['crates/fabelgeist-compute/src/kernel/fast.rs'];helper=sources['crates/fabelgeist-gpu/src/data/gpu/bind_group.rs'];kernel=sources['crates/fabelgeist-compute/src/kernel/mod.rs'];lock=sources['Cargo.lock']
assert b'pub parameters: IndexMap<String, PassParameter>' in parameters and b'pub fn insert(&mut self, key: impl Into<String>' in parameters and b'impl From<IndexMap<String, PassParameter>>' in parameters
assert b'impl From<i32> for PassParameter' in parameter and b'Self::Unsigned(value as u32)' in parameter
assert b'PassParameter::Unsigned(u)' in passbody and b'u.to_le_bytes()' in passbody and b'PassParameter::Unsigned(u)' in fast and b'u.to_le_bytes()' in fast and b'_ => {}' in helper and b'PassParameter::Unsigned(u)' not in helper
# The two inspected 32-bit representations have identical bytes, by integer arithmetic.
assert ((-1) % (1 << 32)).to_bytes(4,'little')==bytes([255,255,255,255])
assert b'code.contains("#version")' in shader and b'return "glsl".to_string()' in shader and b'"wgsl".to_string()' in shader and b'if lang == "glsl"' in shader and b'e.emit_to_string(code)' in shader
assert b'/// Compile WGSL into a kernel' in kernel and b'code: impl Into<String>' in kernel and b'parse_naga(&code' in kernel
assert b'.map_err(|_| anyhow!("Failed to create OffscreenCanvas"))?' in context and b'.map_err(|_| anyhow!("Failed to create surface"))?' in context and b'.map_err(|_| anyhow!("Failed to find an appropriate adapter"))?' in context and b'.map_err(|_| anyhow!("Failed to create device"))?' in context and b'let _ = self.device.poll(wgpu::PollType::Wait' in context
assert b'pub fn from_parts(' in context and b'pub device: Arc<Device>' in context and b'pub queue: Arc<Queue>' in context
native={}
for item in a['native_reference_sources']:
 m=load(item['manifest']);archive=(root/item['archive']).read_bytes();assert sha(archive)==item['archive_sha256']==m['sha256'] and m['sha256'].encode() in lock
 with tarfile.open(root/item['archive'],'r:gz') as tar:body=tar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in item['personally_inspected_ranges']);native[item['member']]=body
assert b'pub fn new(width: u32, height: u32) -> Result<OffscreenCanvas, JsValue>' in native['web-sys-0.3.103/src/features/gen_OffscreenCanvas.rs']
assert b'Result<Adapter, RequestAdapterError>' in native['wgpu-29.0.4/src/api/instance.rs'] and b'Result<Surface<\'window>, CreateSurfaceError>' in native['wgpu-29.0.4/src/api/instance.rs']
assert b'Result<(Device, Queue), RequestDeviceError>' in native['wgpu-29.0.4/src/api/adapter.rs'] and b'Result<crate::PollStatus, crate::PollError>' in native['wgpu-29.0.4/src/api/device.rs']
assert b'pub struct ParseErrors' in native['naga-29.0.4/src/front/glsl/error.rs'] and b'pub kind: ErrorKind' in native['naga-29.0.4/src/front/glsl/error.rs'] and b'pub fn labels(' in native['naga-29.0.4/src/front/wgsl/error.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==77
adler=json.loads(records['adler/READ-L11-363.json']);assert adler['accepted_unique_count_change']==0
for i,n in ((2281,42),(2282,129),(2283,73),(2284,314)):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==[[1,n]] and x['sha256']==roles[i-2281]['sha256']
 batch=749 if i<2284 else 750;delegate=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert delegate['root_personal_rereads']==[] and delegate['delegated_attribution_preserved'] and delegate['accepted_unique_count_change']==0
 x=next(r for r in delegate['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]] and x['source_sha256']==roles[i-2281]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
alias=json.loads(records['segovb/ALIAS-L11-GPU-CONTEXT-SUBCONTRACTS-006.json']);assert len(alias['aliases'])==4 and not alias['global_exact_alias_inventory_complete'] and alias['body_population_unique_credit']==0 and not alias['product_execution']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (151,159,317,*range(359,366),2269,2270,2271,2272,2331):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for i,j in ((362,2271),(363,2272),(364,2269),(365,2270)):
 x=old[i]['exact_alias_applications'][0];assert x['existing_canonical_id']==old[159]['contract_id'] and old[j]['contract_id'].endswith('#'+x['subcontract']) and x['accepted_unique_count_change']==0
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for i in (151,317):
 row=next(x for x in ledger if x['id']==old[i]['contract_id']);assert name in row['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified parameter/context provenance, native signatures and exact historical subcontracts; no product execution.')
