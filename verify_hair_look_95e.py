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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HAIR-LOOK-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2292-2293.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2292,2294))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2292,2293)]==[187,469] and [x['lines'] for x in roles]==[187,469]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
import math
facade=bodies[2292];look=bodies[2293];groom=sources['crates/fabelgeist-hair/src/look/groom.rs'];parameters=sources['crates/fabelgeist-hair/src/parameters.rs'];rng=sources['crates/fabelgeist-hair/src/rng.rs'];seed=sources['crates/fabelgeist-determinism/src/seed.rs'];units=sources['crates/fabelgeist-math/src/units.rs']
for literal in (b'pub mod follicles;',b'#[cfg(feature = "gpu")]',b'pub mod gpu;',b'pub use look::Look;'):assert literal in facade
for literal in (b'pub preset: Preset',b'pub seed: HairSeed',b'rng.next_u32() as i32 & 0x7FFF_FFFF',b'let kind = match preset',b'match kind',b'((turns * 8.0) as usize).clamp(look.points.count(), 256)'):assert literal in look
assert b'let seed = self.seed.word()' in groom and b'seed: seed + 9' in groom and b'Self(Seed::from_u64(word as u32 as u64))' in parameters and b'self.0.to_u64() as u32 as i32' in parameters and b'Self { word: value }' in seed and b'self.word' in seed
assert b'seed ^ 0x9E37_79B9_7F4A_7C15' in rng and b'.wrapping_add(0x9E37_79B9_7F4A_7C15)' in rng and b'z ^ (z >> 30)' in rng and b'z ^ (z >> 27)' in rng and b'z ^ (z >> 31)' in rng and b'(self.next_u64() >> 32) as u32' in rng
# Symbolic bijection prerequisites; this does not run the product PRNG.
assert math.gcd(0xBF58476D1CE4E5B9,2**64)==math.gcd(0x94D049BB133111EB,2**64)==1 and all(0<x<64 for x in (30,27,31)) and (0x7fffffff & 0x7fffffff)==2**31-1 and (2**31-1)+9>2**31-1
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==47
adler=json.loads(records['adler/READ-L11-366.json']);assert adler['accepted_unique_count_change']==0 and not adler['current_reconciled']
for i,n in ((2292,187),(2293,469)):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==[[1,n]] and x['sha256']==roles[i-2292]['sha256']
for i,n,batch in ((2292,90,754),(2293,469,755)):
 d=json.loads(records[f'segovb/READ-L12-{batch}.json']);x=next(r for r in d['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]] and d['root_personal_rereads']==[] and d['accepted_unique_count_change']==0
for p in ('root-l2-extension-20261030/READ-ROOT-L2-EXT-034-044.json','root-l3-extension-20261030/READ-ROOT-L3-EXT-035-042.json'):
 d=json.loads(records[p]);x=next(r for r in d['rows'] if r.get('source_id','').endswith('crates/fabelgeist-hair/src/lib.rs'));assert x['fresh_personal_ranges']==x['assessed_ranges']==[[1,187]] and x['revision']==pr and x['body_credit_before_read']==0
future=next(x for x in adler['connected_reinspection'] if x['original_index']==2306);assert future['fresh_reads']==[[1,140]] and future['additional_original_frontier_credit']==0 and future['source_sha256']==sha(parameters)==work['roles'][2306]['historical_sha256']
for later_index in (2294,2306):
 row=work['roles'][later_index]
 if row['status']!='unassessed':
  assert row['status']=='full-historical-body-trace-open';later=load(row['historical_assessment']);item=next(x for x in later['historical_source_roles'] if x['original_index']==later_index);assert item['prior_role_status']=='unassessed' and item['local_registration']['path']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (161,377,1840,2407,3144):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed';p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified hair facade historical scopes, seed source algebra and numeric choice; no product execution.')
