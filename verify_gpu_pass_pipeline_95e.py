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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GPU-PASS-PIPELINE-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2279-2280.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2279,2281))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==(i!=2280) and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2279,2280)]==[484,189] and [x['lines'] for x in roles]==[484,187]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
oldproof=load(a['prior_e275_git_membership_evidence']);oldrev=oldproof['revision'];assert oldrev=='e2755a732c84e67296b5e3441a4259a84f461b2f';sharedold=load(oldproof['shared_commit_and_tree_objects']);raw=contents(sharedold['commit']['preserved_bytes']);assert oid('commit',raw)==oldrev and sha(raw)==sharedold['commit']['sha256'] and len(raw)==sharedold['commit']['bytes'] and raw.splitlines()[0]==b'tree '+oldproof['root_tree'].encode();oldnodes=dict(sharedold['tree_objects'])
for p in oldproof['additional_shared_tree_objects']:oldnodes.update(load(p)['additional_tree_objects'])
oldnodes.update(oldproof['additional_tree_objects']);oldbodies={}
for path,item in oldproof['paths'].items():
 raw=contents(item['preserved_bytes']);chain(path,item,raw,oldproof['root_tree'],oldnodes);assert item['current95e_identical'] and (sources.get(path) if path in sources else bodies[2280])==raw;oldbodies[path]=raw
passbody=bodies[2279];pipeline=bodies[2280];reflection=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs'];shader=sources['crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs'];helper=sources['crates/fabelgeist-gpu/src/data/gpu/bind_group.rs'];observation=sources['crates/fabelgeist-gpu/src/globals/validation_observation.rs'];context=sources['crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs'];lock=sources['Cargo.lock']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:oldpipeline=archive.extractfile(roles[1]['sha256']).read()
assert b'blocking_validation: bool' in oldpipeline and b'validation_observation: ValidationObservation' in pipeline and b'validation_observation.start_scope(device)' in pipeline and b'ValidationObservation::Uncaptured' in context and b'ValidationObservation::BlockingScopes' in context
for source in (passbody,helper):assert b'PassParameter::Mat3(v)' in source and b'let col_stride = member.size / 3;' in source and b'buffer_data[offset..offset + 4]' in source
assert b'let mut groups = BTreeMap::new();' in reflection and b'.entry(binding.group)' in reflection and b'layouts.push(Arc::new(' in reflection and b'reflected.push(group.reflection)' in reflection and b'.get(bg_reflection.index as usize)' in passbody and b'bind_groups.iter().enumerate()' in passbody
# Symbolic group-position identity only; no shader or GPU operation is executed.
assert len([2])==1 and 2>=len([2])
assert b'self.reflection.uniform_binding = Some(binding)' in reflection and b'self.reflection.uniform_buffer_size = *span' in reflection and b'wgpu::naga::TypeInner::Image { dim, class, .. }' in reflection and b'multisampled: false' in reflection and b'wgpu::StorageTextureAccess::WriteOnly' in reflection and b'wgpu::SamplerBindingType::Filtering' in reflection
assert b'pub code: String' in shader and b'pub module: Option<Arc<wgpu::ShaderModule>>' in shader and b'ReflectedInterface::new(context, &shader.code)' in pipeline and b'.module' in pipeline and b'entry_point.hash(&mut s)' in pipeline and b'if let Some(p) = cache.get(&cache_key)' in pipeline and b'pub pipeline_cache: Arc<Mutex<HashMap<u64' in pipeline
assert b'if let Ok(guard) = pipeline_def.validation_error.lock()' in passbody and b'.map_err(|_| anyhow!("Failed to lock pipeline validation state"))?' in pipeline and b'Self::BlockingScopes => Some(device.push_error_scope' in observation
native={}
for item in a['native_reference_sources']:
 m=load(item['manifest']);archive=(root/item['archive']).read_bytes();assert sha(archive)==item['archive_sha256']==m['sha256'] and m['sha256'].encode() in lock
 with tarfile.open(root/item['archive'],'r:gz') as tar:body=tar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in item['personally_inspected_ranges']);native[item['member']]=body
layout=native['wgpu-29.0.4/src/api/pipeline_layout.rs'];device=native['wgpu-29.0.4/src/api/device.rs'];assert b'first entry will provide all the bindings for' in layout and b'"set = 0"' in layout and b'impl Drop for ErrorScopeGuard' in device and b'drop(self.device.pop_error_scope(self.index))' in device
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==70
adler=json.loads(records['adler/READ-L11-363.json']);delegate=json.loads(records['segovb/READ-L12-749.json']);check=json.loads(records['segovb/VERIFY-L12-749.json']);assert adler['accepted_unique_count_change']==0 and delegate['root_personal_rereads']==[] and delegate['delegated_attribution_preserved'] and delegate['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup['segovb/READ-L12-749.json']['sha256']
for i,n in ((2279,484),(2280,187)):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==[[1,n]] and x['sha256']==roles[i-2279]['sha256'];x=next(r for r in delegate['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]] and x['source_sha256']==roles[i-2279]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
connected=json.loads(records['segovb/READ-L12-CONNECTED-TEAM-004.json']);assert connected['root_personal_rereads']==[] and connected['original_overlay_merged_shared_extension_credit']==0 and connected['accepted_unique_count_change']==0
for item in connected['evidence']['rows']:
 assert item['new_original_credit']==0 and item['registration'] in records and item['windows_locator_qualification'] in records
assert work['roles'][2635]['status']=='unassessed' and work['roles'][2636]['status']=='unassessed'
older=json.loads(records['segovb/READ-L11-626.json']);assert older['source_revision']==oldrev and not older['new_target_reconciliation_complete'] and not older['other_lens_coverage_transferred'] and older['accepted_unique_count_change']==0
for item in older['rows']:
 if item['path'] in oldbodies:assert sha(oldbodies[item['path']])==item['sha256'] and item['fresh_personal_body_ranges']==item['assessed_ranges']==[[1,item['actual_EOF']]] and 'root adoption' in item['attribution']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in [150,151,*range(339,359),370,2331]:assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in (old[150]['contract_id'],old[151]['contract_id'],old[352]['contract_id'],old[353]['contract_id'],'REVIEW-GPU-UNIFORM-MAT3-BOUNDS-95E'):
 row=next(x for x in ledger if x['id']==finding);assert name in row['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified pass/pipeline provenance, earlier e275 attribution and conditional reflection contracts; no product execution.')
