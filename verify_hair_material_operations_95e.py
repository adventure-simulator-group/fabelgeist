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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HAIR-MATERIAL-OPERATIONS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2296-2298.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2296,2299))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2296,2297,2298)]==[225,233,141] and [x['lines'] for x in roles]==[225,233,141]
sources={}
for x in a['current_supporting_sources']:
 body=contents(x['preserved_bytes']);reg=(root/x['registration']).read_bytes();rows=json.loads(reg)['support'];r=next(r for r in rows if r['path']==x['path']);assert sha(reg)==x['registration_sha256'] and r['sha256']==sha(body)==x['sha256'] and r['ranges']==x['ranges']==[[1,x['lines']]] and len(body)==x['bytes'] and len(body.splitlines())==x['lines'] and oid('blob',body)==x['git_blob']==tree[x['path']];chain(x['path'],proof['paths'][x['path']],body,proof['root_tree'],nodes);sources[x['path']]=body
material=bodies[2296];clump=bodies[2297];curl=bodies[2298];guide=sources['crates/fabelgeist-hair/src/ops/mod.rs'];bridge=sources['crates/adventuresim-character-creator/src/hair.rs'];ui=sources['crates/adventuresim-character-creator/src/hair_color_controls.rs']
for label in (b'black',b'dark brown',b'brown',b'auburn',b'red',b'blond',b'platinum',b'grey'):assert b'"'+label+b'"' in ui and b'"'+label+b'"' in material
for token in (b'for name in NATURAL',b'HairPigment::named(name)',b'design.pigment = pigment',b'&capitalized(name)'):assert token in ui
assert b'HairMaterial::named(name)' in bridge and b'pub fn named(name: &str) -> Option<Self>' in material
assert b'.nearest(root, distance * 8.0, |g| g == group)' in guide and b'.or_else(|| grid.nearest(root, f32::INFINITY, |_| true))' in guide and b'let map = GuideMap::get(' in clump and b'let map = GuideMap::get(' in curl
assert b'reuse_guide_map: false' in clump and b'reuse_guide_map: true' in curl and b'let turns_per_metre = self.frequency * 3.0' in curl
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for x in a['recovered_historical_records']:
 original=lookup[x['original_path']];raw=contents(x['preserved_bytes']);assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes']
 if x['redacted_local_path_occurrences']:assert raw.count(b'<LOCAL_PATH_')==x['redacted_local_path_occurrences'] and 'pinned complete059 transport' in x['original_recovery']
 else:assert sha(raw)==x['sha256'] and oid('blob',raw)==x['git_blob']
 records[x['original_path']]=raw
adler=json.loads(records['adler/READ-L11-367.json']);delegate=json.loads(records['segovb/READ-L12-756.json']);connected=json.loads(records['segovb/READ-L12-CONNECTED-TEAM-005.json']);assert adler['accepted_unique_count_change']==delegate['accepted_unique_count_change']==connected['accepted_unique_count_change']==0 and delegate['root_personal_rereads']==connected['root_personal_rereads']==[] and delegate['delegated_attribution_preserved']
for i,n in ((2296,225),(2297,233),(2298,141)):
 x=next(x for x in adler['sources'] if x['original_index']==i);y=next(x for x in delegate['rows'] if x['original_index']==i);assert x['scope']==x['fresh_reads']==y['assessed_ranges']==y['fresh_personal_body_reads']==[[1,n]] and x['sha256']==y['source_sha256']==roles[i-2296]['sha256']
for suffix,n in (('/hair.rs',177),('/hair_color_controls.rs',196)):
 x=next(x for x in connected['checked_proofs'] if x['source_id'].endswith(suffix));assert x['assessed_ranges']==[[1,n]] and x['root_personal_rereads']==[] and x['registration'] in records
row=work['roles'][2301]
if row['status']!='unassessed':
 assert row['status']=='full-historical-body-trace-open';later=load(row['historical_assessment']);item=next(x for x in later['historical_source_roles'] if x['original_index']==2301);assert item['prior_role_status']=='unassessed' and item['local_registration']['path']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (164,403,1841,1842):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
row=next(x for x in ledger if x['id']==a['related_existing_current_finding']);assert name in row['further_current_evidence']
assert a['repair']=='not started' and a['validation']=='not performed';p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical material and operator evidence, current color and guide traces; no product execution.')
