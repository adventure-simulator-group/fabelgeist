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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-READBACK-SORT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2246-2247.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2246,2248))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert len(bodies[2246].splitlines())==294 and len(bodies[2247].splitlines())==327
oldproof=load(a['older_own_lens_git_membership_evidence']);oldrev=oldproof['revision'];assert oldrev=='e2755a732c84e67296b5e3441a4259a84f461b2f';oldshared=load(oldproof['shared_commit_and_tree_objects']);raw=contents(oldshared['commit']['preserved_bytes']);assert oid('commit',raw)==oldrev and sha(raw)==oldshared['commit']['sha256'] and len(raw)==oldshared['commit']['bytes'] and raw.splitlines()[0]==b'tree '+oldproof['root_tree'].encode();oldnodes=dict(oldshared['tree_objects'])
for p in oldproof['additional_shared_tree_objects']:oldnodes.update(load(p)['additional_tree_objects'])
oldnodes.update(oldproof['additional_tree_objects']);path='crates/fabelgeist-compute/src/sort/mod.rs';chain(path,oldproof['paths'][path],bodies[2247],oldproof['root_tree'],oldnodes)
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
rb=bodies[2246];sort=bodies[2247];buf=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs'];extent=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs'];definition=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/definition.rs'];context=sources['crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs'];prefix=sources['crates/fabelgeist-compute/src/prefix_sum.rs'];staging=sources['crates/fabelgeist-armor/src/gpu/staging.rs'];lock=sources['Cargo.lock'];kernel=contents(load('ASSESSMENT-L01-HISTORICAL-2243-2245.json')['historical_source_roles'][1]['current_comparison']['preserved_bytes'])
assert b'length: buffer.size' in rb and b'.div_ceil(wgpu::COPY_BUFFER_ALIGNMENT)' in rb and b'u64::from(range.length)' in rb and b'batch.encoder().copy_buffer_to_buffer(' in rb and b'if u64::from(bytes) == 0' in buf and b'pub struct BufferByteLength(u64)' in extent and b'pub fn copy_src()' in definition
assert b'let state = Arc::new(Mutex::new(ReadbackState::Pending))' in rb and b'batch.after_submit(move ||' in rb and b'ReadbackState::Pending => return Ok(None)' in rb and b'ReadbackState::Pending | ReadbackState::Taken => Ok(false)' in rb and b'Dropped unrun if the batch never is' in kernel and b'for work in self.after_submit' in kernel
assert b'pub keys: Buffer' in sort and b'pub values: Buffer' in sort and b'pub histogram: Buffer' in sort and b'capacity: u32' in sort and b'if scratch.capacity < count' in sort and b'if u64::from(keys.size) < needed || u64::from(values.size) < needed' in sort and b'ScanElementCount::from_elements(RADIX * tiles)' in sort
assert b'const TILE: u32 = 256' in sort and b'const RADIX: u32 = 256' in sort and b'let tiles = count.div_ceil(TILE)' in sort and b'&histogram_parameters, [tiles, 1, 1]' in sort and b'&scatter_parameters, [tiles, 1, 1]' in sort and b'global_invocation_id' in sort and b'workgroup_id' in sort and b'let index = global_id.x' in sort and b'let block = group_id.y * groups.x + group_id.x' in prefix
assert b'adapter_limits.max_compute_workgroups_per_dimension' in context and b'Readback::record(self.context(), &mut batch, &staging.buffers)' in staging
# Symbolic integers only: no native buffer, multiplication in the product or
# dispatch runs. The exact sort count exceeds the illustrative native limit;
# the later extreme threshold overflows a u32 histogram count.
assert 65535*256+1==16776961 and (16776961+255)//256==65536 and 256*((4294967041+255)//256)==1<<32 and 4294967041<=2**32-1 and 1%4!=0
for project,checksum in [('wgpu','76e8840e1ba2881d4cbb18d2147627a56af426ff064c0401eb0c8410c6325d07'),('wgpu-types','84bf84cd9ca8ca45e2b223a3868f1adf9bfc0c66aeac212e76ee7e40fdadf8f5')]:
 directory=root/f'native-evidence/{project}-29.0.4';m=json.loads((directory/'MANIFEST.json').read_text());raw=(directory/m['archive']).read_bytes();assert sha(raw)==checksum==m['sha256'] and len(raw)==m['bytes'] and checksum.encode() in lock and (f'name = "{project}"\nversion = "29.0.4"').encode() in lock
 with tarfile.open(directory/m['archive'],'r:gz') as tar:
  for x in m['inspected_members']:
   d=tar.extractfile(x['path']).read();assert len(d)==x['bytes'] and sha(d)==x['sha256']
   if x['path'].endswith('/src/api/command_encoder.rs'):assert b'Buffer offsets or copy size not a multiple of [`COPY_BUFFER_ALIGNMENT`]' in d
   if x['path'].endswith('/src/lib.rs') and project=='wgpu-types':assert b'pub const COPY_BUFFER_ALIGNMENT: BufferAddress = 4;' in d
   if x['path'].endswith('/src/limits.rs'):assert b'The maximum value for each dimension of a `ComputePass::dispatch(x, y, z)` operation.' in d and b'max_compute_workgroups_per_dimension: 65535' in d
  assert {f'{project}-29.0.4/LICENSE.APACHE',f'{project}-29.0.4/LICENSE.MIT'}<=set(tar.getnames())
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==71
adler=json.loads(records['adler/READ-L11-356.json']);assert adler['accepted_unique_count_change']==0
for i in (2246,2247):
 row=next(x for x in adler['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2246]['lines']]] and row['sha256']==roles[i-2246]['sha256']
for batch,i,scope in ((741,2246,[[1,294]]),(742,2247,[[1,305]])):
 path=f'segovb/READ-L12-{batch}.json';d=json.loads(records[path]);v=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert d['root_personal_rereads']==[] and d['delegated_attribution_preserved'] and d['accepted_unique_count_change']==0 and v['receipt_sha256']==lookup[path]['sha256'] and d['proposal'] in records
 row=next(x for x in d['rows'] if x['original_index']==i);assert row['source_sha256']==roles[i-2246]['sha256'] and row['fresh_personal_body_reads']==row['assessed_ranges']==scope and 'Subagent /root/lens12_frontier' in row['attribution'] and row['windows_locator_qualification'] in records
 for reg in d['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
 for x in d['additional_connected_body_receipts']:assert x['path'] in records and sha(records[x['path']])==x['sha256']
tail=json.loads(records['parallel-l12-20261029/READ-CONNECTED-PAR-L12-742-ORIGINAL2247-TAIL.json']);t=tail['rows'][0];assert t['original_index']==2247 and t['fresh_personal_body_reads']==[[306,327]] and t['combined_L12_coverage']==[[1,327]] and t['new_original_credit']==0 and t['registration'] in records and t['windows_tail_locator_qualification'] in records
for lens in (2,3):
 frozen=json.loads(records[f'root-l{lens}-extension-20261030/READ-ROOT-L{lens}-EXT-0'+('34-044' if lens==2 else '35-042')+'.json']);f=next(x for x in frozen['rows'] if x['source_id'].endswith('crates/fabelgeist-compute/src/sort/mod.rs'));assert f['fresh_personal_ranges']==[[1,327]] and f['source_sha256']==sha(bodies[2247]) and f['revision']==pr
 current=json.loads(records[f'root-l{lens}-extension-20261030/READ-ROOT-L{lens}-CURRENT-00'+('9' if lens==2 else '5')+'.json']);c=next(x for x in current['rows'] if x['source_id'].endswith('crates/fabelgeist-compute/src/sort/mod.rs'));assert c['revision']==oldrev and c['source_sha256']==sha(bodies[2247]) and c['fresh_personal_ranges']==[] and c['assessed_ranges']==[[1,327]]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (140,299,301,302,1827,2406,3142):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
family=a['connected_historical_reason_family'];row=next(x for x in ledger if x['id']==family['ledger_id']);assert [x['queue'] for x in row['named_subcontracts']]==family['queue_indices'] and all(x['id']==old[x['queue']]['contract_id'] for x in row['named_subcontracts']) and row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified readback and sort historical attribution, native contracts and symbolic limits; no product execution.')
