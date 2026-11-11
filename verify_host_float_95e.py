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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HOST-FLOAT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2238-2242.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2238,2243))
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
arithmetic=bodies[2239];double=bodies[2240];module=bodies[2241];tests=bodies[2242];metal=sources['crates/fabelgeist-armor/src/gpu/metal/wgsl.rs'];bake=sources['crates/fabelgeist-armor/src/gpu/metal/bake.rs'];shape=sources['crates/fabelgeist-armor/src/gpu/breastplate/shape_wgsl.rs'];fit=sources['crates/fabelgeist-armor/src/gpu/breastplate/fit_wgsl.rs'];design=sources['crates/fabelgeist-armor/src/breastplate_design.rs'];record=sources['crates/fabelgeist-armor/src/gpu/breastplate/mod.rs']
assert b'bitcast<u32>(x) | host_zero()' in arithmetic and b'fma(-q, b, a)' in arithmetic and b'Devices flush subnormals' in module and b'quarter-turn reduction' in module and b'host_float::PARAMS_ZERO_HOOK' in metal and b'host_float::wgsl()' in metal and b'sample.insert(host_float::ZERO_FIELD, 0u32)' in bake and b'cut.insert(host_float::ZERO_FIELD, 0u32)' in bake
begin=double.index(b'fn host_pow(base: f32, exponent: f32)');end=double.index(b'\n}',begin);pow_source=double[begin:end];assert pow_source.index(b'exponent == 0.0 || base == 1.0')<pow_source.index(b'if (!(base > 0.0))') and b'return 0.0' in pow_source and b'host_pow` takes a positive base, or zero' in module and b'words.uniform(0.01, 4.0)' in tests
assert b'pub const FULLNESS_RANGE: RangeInclusive<u16> = 350..=1000' in design and b'!Self::FULLNESS_RANGE.contains(&self.fullness.0)' in design and b'validate_breastplate(design)?' in record and b'profile.fullness.unit()' in shape and b'fn fullness() -> f32 { return plate[14]; }' in shape and b'power = host_div(2.0, fullness())' in shape and b'host_pow(max(state.y, 0.0), host_div(2.0, fullness()))' in fit
oldproof=load(a['older_l11_git_membership_evidence']);sharedold=load(oldproof['shared_commit_and_tree_objects']);oldrev=oldproof['revision'];assert oldrev=='e2755a732c84e67296b5e3441a4259a84f461b2f' and oldproof['root_tree']==sharedold['root_tree'];commit=contents(sharedold['commit']['preserved_bytes']);assert oid('commit',commit)==oldrev and commit.splitlines()[0]==b'tree '+oldproof['root_tree'].encode();oldnodes=dict(sharedold['tree_objects']);oldnodes.update(oldproof['additional_tree_objects'])
oldbodies={'crates/fabelgeist-compute/src/host_float/arithmetic.rs':arithmetic,'crates/fabelgeist-compute/src/host_float/double_float.rs':double,'crates/fabelgeist-compute/src/host_float/mod.rs':module,'crates/fabelgeist-armor/src/gpu/metal/wgsl.rs':metal}
for path,body in oldbodies.items():chain(path,oldproof['paths'][path],body,oldproof['root_tree'],oldnodes)
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
path='segovb/READ-L12-740.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-740.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for i in range(2238,2243):
 x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2238]['lines'];assert x['source_sha256']==roles[i-2238]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]]
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
adler=json.loads(records['adler/READ-L11-356.json']);assert adler['accepted_unique_count_change']==0
for i in range(2238,2243):
 x=next(z for z in adler['sources'] if z['original_index']==i);assert x['fresh_reads']==[[1,roles[i-2238]['lines']]] and x['sha256']==roles[i-2238]['sha256']
prior=json.loads(records['parallel-l11-20261029/READ-163.json']);rootread=json.loads(records['segovb/READ-L11-699.json']);assert rootread['proposal']=='parallel-l11-20261029/READ-163.json' and rootread['proposal_sha256']==lookup[rootread['proposal']]['sha256'] and rootread['agent_verification'] in records and rootread['agent_verification_sha256']==lookup[rootread['agent_verification']]['sha256']
for path in list(oldbodies)[:3]:
 x=next(z for z in prior['rows'] if path in z['source_id']);assert x['sha256']==sha(oldbodies[path]) and x['fresh_personal_body_ranges']==[] and x['qualified_same_lens_reuse_ranges']==x['assessed_ranges'] and 'INHERITED ownLens11 Adler' in x['attribution'] and 'No agent/root personal reread' in x['attribution'] and 'no inherited current/native/runtime verdict' in x['attribution']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (1825,2280,2281):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified inherited host-float attribution, Git proofs and selected source domains; no product execution.')
