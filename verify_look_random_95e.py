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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-LOOK-RANDOM-CONTRACTS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2294-2295.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2294,2296))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2294,2295)]==[193,50] and [x['lines'] for x in roles]==[193,50]
support=a['current_supporting_sources'][0];rng=contents(support['preserved_bytes']);raw=(root/support['registration']).read_bytes();registered=json.loads(raw)
assert sha(raw)==support['registration_sha256'] and registered['support'][0]['sha256']==sha(rng)==support['sha256'] and registered['support'][0]['ranges']==support['ranges']==[[1,168]] and len(rng.splitlines())==168 and oid('blob',rng)==support['git_blob']==tree[support['path']]
groom=bodies[2294];follicle=bodies[2295]
for token in (b'self.guide_operations(head, &mut ops)',b'self.coarse_texture(&mut ops)',b'self.fine_texture(&mut ops)',b'seed: seed + 9',b'seed: seed + 7'):assert token in groom
for token in (b'let mut pick = rng.index(100) as i32',b'(Growth::Whorl, 55)',b'(Growth::Cowlick, 12)',b'(Growth::DoubleWhorl, 8)',b'(Growth::Radial, 5)',b'(Growth::Forward, 8)',b'(Growth::Backward, 7)',b'(Growth::Down, 5)',b'h.recession += rng.range(0.0, 0.02)'):assert token in follicle
for token in (b'pub fn range(&mut self, low: f32, high: f32) -> f32',b'low + (high - low) * self.next_f32()',b'pub fn index(&mut self, n: usize) -> usize',b'.min(n.saturating_sub(1))',b'(self.next_u64() >> 40) as f32 / (1u64 << 24) as f32'):assert token in rng
assert sum([55,12,8,5,8,7,5])==100 and (2**24)%100!=0
# Rational proof of the stated rounding midpoint; no product RNG is run.
u=Fraction(2**24-1,2**24);value=Fraction(1,2)+Fraction(1,2)*u;assert value==1-Fraction(1,2**25) and (value-(1-Fraction(1,2**24)))==(1-value)==Fraction(1,2**25)
import math
assert math.gcd(0xBF58476D1CE4E5B9,2**64)==math.gcd(0x94D049BB133111EB,2**64)==1
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['reused_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if not item['redacted_local_path_occurrences']:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
assert len(records)==9
adler=json.loads(records['adler/READ-L11-366.json']);delegate=json.loads(records['segovb/READ-L12-755.json']);assert adler['accepted_unique_count_change']==delegate['accepted_unique_count_change']==0 and delegate['root_personal_rereads']==[] and delegate['delegated_attribution_preserved']
for i,n in ((2294,193),(2295,50)):
 x=next(x for x in adler['sources'] if x['original_index']==i);y=next(x for x in delegate['rows'] if x['original_index']==i)
 assert x['scope']==x['fresh_reads']==y['assessed_ranges']==y['fresh_personal_body_reads']==[[1,n]] and x['sha256']==y['source_sha256']==roles[i-2294]['sha256']
 assert roles[i-2294]['original_supplier_scopes'][0]['read_ranges']==roles[i-2294]['original_producer_scopes'][0]['read_ranges_as_recorded']==[[1,n]]
inherited=a['inherited_supplier_record'];raw=contents(inherited['preserved_bytes']);entry=lookup[inherited['original_path']];assert sha(raw)==inherited['sha256']==entry['sha256'] and len(raw)==inherited['bytes']==entry['bytes'] and oid('blob',raw)==inherited['git_blob']==entry['git_blob'];lines=raw.splitlines()
for idx,i,n in ((51,2294,193),(52,2295,50)):
 x=json.loads(lines[idx]);assert x['source_id']==roles[i-2294]['source_id'] and x['observed_sha256']==roles[i-2294]['sha256'] and x['read_ranges']==[[1,n]]
row=work['roles'][2307]
if row['status']!='unassessed':
 assert row['status']=='full-historical-body-trace-open';later=load(row['historical_assessment']);item=next(x for x in later['historical_source_roles'] if x['original_index']==2307);assert item['prior_role_status']=='unassessed' and item['local_registration']['path']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (161,377,1840):assert q[i]['contract_id']==old[i]['contract_id'] and name in q[i]['further_current_evidence'] and q[i]['current_reconciliation'].startswith('partial ')
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert name in row['further_current_evidence'] and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed';p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified look groom, historical supplier copy and symbolic RNG boundaries; no product execution.')
