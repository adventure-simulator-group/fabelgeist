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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-DETERMINISM-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2248-2250.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2248,2251))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='bounded-current-trace-only' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2248,2249,2250)]==[58,213,169]
oldproof=load(a['older_own_lens_git_membership_evidence']);oldrev=oldproof['revision'];assert oldrev=='e2755a732c84e67296b5e3441a4259a84f461b2f';sharedold=load(oldproof['shared_commit_and_tree_objects']);raw=contents(sharedold['commit']['preserved_bytes']);assert oid('commit',raw)==oldrev and sha(raw)==sharedold['commit']['sha256'] and len(raw)==sharedold['commit']['bytes'] and raw.splitlines()[0]==b'tree '+oldproof['root_tree'].encode();oldnodes=dict(sharedold['tree_objects'])
for p in oldproof['additional_shared_tree_objects']:oldnodes.update(load(p)['additional_tree_objects'])
oldnodes.update(oldproof['additional_tree_objects']);oldbodies={}
for p,x in oldproof['paths'].items():
 raw=contents(x['preserved_bytes']);chain(p,x,raw,oldproof['root_tree'],oldnodes);oldbodies[p]=raw
for i in (2248,2249,2250):assert oldbodies[roles[i-2248]['current_comparison']['path']]==bodies[i] and roles[i-2248]['prior_current_observations'] and roles[i-2248]['prior_role_qualification']
oldlock=oldbodies['Cargo.lock'];assert sha(oldlock)=='58118b1b138af4f9feea993ea990dc334f63d30c4aa66f8b5597a0f1dfecfcf6'
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
lib=bodies[2248];seed=bodies[2249];stream=bodies[2250];lock=sources['Cargo.lock'];rules=sources['crates/AGENTS.md'];audio=sources['crates/adventuresim-tactical-client/src/movement_audio.rs'];sound=sources['crates/adventuresim-tactical-client/src/sound_output.rs'];spawn=sources['crates/adventuresim-tactical-server/src/player_projection/spawn_admission.rs'];profile=sources['crates/adventuresim-strategic-sim/src/profile.rs']
assert b'fabelgeist.determinism.seed.v1' in seed and b'for field in [root, stream.0.as_bytes()]' in seed and b'field.len() as u64' in seed and b'root.to_le_bytes()' in seed and b'value.to_le_bytes()' in seed and b'Hash::hash(&self.to_le_bytes(), state)' in seed and b'self.to_u64().cmp(&other.to_u64())' in seed
assert b'assert!(minimum.is_finite() && maximum.is_finite() && minimum <= maximum)' in stream and b'minimum + self.unit_f32() * (maximum - minimum)' in stream and b'(value >> 40) as f32 / UNIT_F32_SCALE' in lib
assert b'NonZeroU64::new(length as u64).expect(' in stream and b'NonZeroU64::new(2).expect(' in stream and b'.expect("nonzero bound")' in stream and b'Never use `unwrap` or `expect` outside tests' in rules and b'.index(SoundFamily::TAKES)' in audio and b'const TAKES: usize = 3' in sound
assert b'random.range_f32(-5.0, 5.0)' in spawn and b'rng.range_f32(-spread, spread)' in profile and b'0.35, &mut rng' in profile
# Finite binary32 maximum M=(2^24-1)*2^104. 2*M exceeds M;
# this is exact integer arithmetic, not executing f32 or an RNG.
M=((1<<24)-1)*(1<<104);assert 2*M>M
native_members={}
for project,ver,checksum in [('rand','0.9.5','b9ef1d0d795eb7d84685bca4f72f3649f064e6641543d3a8c415898726a57b41'),('rand_xoshiro','0.7.0','f703f4665700daf5512dcca5f43afa6af89f09db47fb56be587f80636bda2d41')]:
 directory=root/f'native-evidence/{project}-{ver}';m=json.loads((directory/'MANIFEST.json').read_text());raw=(directory/m['archive']).read_bytes();assert sha(raw)==checksum==m['sha256'] and len(raw)==m['bytes'] and checksum.encode() in lock and checksum.encode() in oldlock
 with tarfile.open(directory/m['archive'],'r:gz') as tar:
  for x in m['inspected_members']:
   d=tar.extractfile(x['path']).read();assert len(d)==x['bytes'] and sha(d)==x['sha256'];native_members[sha(d)]=d
   if x['path'].endswith('/src/distr/weighted/mod.rs'):assert b'impl_weight_int!(u8, u16, u32, u64' in d and b'InvalidWeight' in d and b'Overflow' in d
   if x['path'].endswith('/src/distr/weighted/weighted_index.rs'):assert b'iter.next().ok_or(Error::InvalidInput)' in d and b'total_weight.checked_add_assign(w.borrow())' in d and b'Error::InsufficientNonZero' in d
   if x['path'].endswith('/src/distr/uniform_int.rs'):assert b'if !(low < high)' in d and b'Sample from distribution, Lemire\'s method, unbiased' in d and b'Sample single value, Canon\'s method, biased' in d
  assert {f'{project}-{ver}/LICENSE-APACHE',f'{project}-{ver}/LICENSE-MIT'}<=set(tar.getnames())
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==74
adler=json.loads(records['adler/READ-L11-357.json']);first=json.loads(records['adler/READ-L11-220.json']);assert adler['accepted_unique_count_change']==0
for i in (2248,2249,2250):
 x=next(r for r in adler['sources'] if r['original_index']==i);prev=next(r for r in first['connected_reinspection'] if r['original_index']==i);assert x['fresh_reads']==[] and x['sha256']==roles[i-2248]['sha256'] and prev['source_sha256']==x['sha256'] and prev['additional_original_frontier_credit']==0
 assert prev['fresh_reads']==([[1,131],[132,213]] if i==2249 else [[1,roles[i-2248]['lines']]])
data=json.loads(records['segovb/READ-L12-742.json']);check=json.loads(records['segovb/VERIFY-L12-742.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup['segovb/READ-L12-742.json']['sha256']
for i in (2248,2249):
 x=next(r for r in data['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,roles[i-2248]['lines']]] and x['source_sha256']==roles[i-2248]['sha256']
x=next(r for r in data['rows'] if r['original_index']==2250);prior=json.loads(records[x['qualified_same_lens_reuse']['receipt']])['rows'][0];assert x['fresh_personal_body_reads']==[] and x['qualified_same_lens_reuse']['new_rereads']==0 and prior['fresh_personal_body_reads']==[[1,169]] and prior['new_ordered_population_credit']==0 and x['registration'] in records
correction=json.loads(records['parallel-l12-20261029/QUALIFICATION-PAR-L12-742-SEED-CONTEXT-LITERAL.json']);assert correction['exact_literal']=='fabelgeist.determinism.seed.v1' and correction['new_read_or_population_credit']==0
r1=json.loads(records['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-001.json']);r4=json.loads(records['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-004.json']);assert r4['prior_partial_preserved']['path'] in records and r4['same_lens_first_party_source_support']['receipt'] in records
for x in r1['rows']:
 if x['git_path'] in oldbodies:assert sha(oldbodies[x['git_path']])==x['sha256'] and x['fresh_personal_body_ranges']==x['assessed_ranges']
for x in r4['rows']:
 p=x['source_object'].removeprefix('target/726-audit-adler-20261027/');assert p in records and sha(records[p])==x['sha256']
 if 'native_package' in x:assert x['sha256'] in native_members and records[p]==native_members[x['sha256']]
 if x['source_id'].endswith('uniform.rs'):assert x['fresh_personal_body_ranges_this_checkpoint']==[[1,488]] and x['excluded_test_prefix_and_tail']==[[489,622]]
 if x['source_id'].endswith('uniform_int.rs'):assert x['fresh_personal_body_ranges_this_checkpoint']==[[1,595]] and x['excluded_test_prefix_and_tail']==[[596,914]]
 if x['source_id'].endswith('splitmix64.rs'):assert x['fresh_personal_body_ranges_this_checkpoint']==[] and x['qualified_same_lens_prior_partial_ranges']==[[1,77]] and x['excluded_test_prefix_and_tail']==[[78,170]]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (134,286,646,662,1553,1616,1827):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['new_current_candidates']==[] and a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified determinism no-reread adoptions, prior mixed native scope and range evidence; no product execution.')
