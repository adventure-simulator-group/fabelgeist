#!/usr/bin/env python3
"""Verify scoped evidence and historical attribution without product execution."""
import gzip,hashlib,json,re,tarfile
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-BESTIARY-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2157.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
item=h['historical_source_roles'][0];assert len(h['historical_source_roles'])==1 and item['original_index']==2157 and item['original_ordered_ranges']==[[1,112],[140,285]] and item['separately_registered_historical_gap_ranges']==[[113,139],[286,330]] and item['new_personal_read_ranges']==[[1,330]]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));assert registry['files'][2157]['owner_contract_proofs'][1]['read_ranges']==item['original_ordered_ranges']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:before=archive.extractfile(item['sha256']).read()
c=item['current_comparison'];body=contents(c['preserved_bytes']);row=work['roles'][2157]
assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and len(before.splitlines())==330 and sha(before)==row['historical_sha256'] and before==body
assert c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,330]] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes)
sources={}
for item in a['current_supporting_sources']:
 source=contents(item['preserved_bytes']);assert item['revision']==rev and sha(source)==item['sha256'] and len(source)==item['bytes'] and len(source.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',source)==item['git_blob']==tree[item['path']];sources[item['path']]=source
bestiary=body;simulation=sources['crates/adventuresim-stdb-module/src/simulation.rs'];org=sources['crates/adventuresim-core/src/organization.rs'];catalog=sources['crates/adventuresim-core/src/organization_catalog_validation.rs'];schedule=sources['crates/adventuresim-core/src/strategic_schedule.rs'];build=sources['crates/adventuresim-core/build.rs']
assert b'pub wildmen: f32' in bestiary and b'pub human: f32' in bestiary and b'hours.is_finite() && (0.0..=maximum).contains(&hours)' in bestiary and b'if self.direct(category) <= 0.0 {' in bestiary and b'if direct.is_finite()' in bestiary and b'0.35, 0.20, 0.65, 0.40' in bestiary and b'pub fn direct_mut(&mut self' in bestiary
assert b'simulation_bestiary_hours_valid(skills.bestiary_hours)' in simulation and b'hours.direct_fields_valid(MAX_SIMULATION_SKILL_HOURS)' in simulation
assert b'Bestiary {\n        category: String,' in org and b'BestiaryCategory::from_id(category)' in schedule and b'if let Some(category)' in schedule and b'organization_catalog_validation::validate_documents(' in build and b'if !BESTIARY.contains(&text(entry, source, "category")?)' in catalog
ids=re.findall(rb'Self::[A-Za-z]+ => "([a-z]+)"',bestiary.split(b'pub const fn id(self)')[1].split(b'pub const fn label(self)')[0]);validator=re.findall(rb'"([a-z]+)"',catalog.split(b'const BESTIARY: &[&str] = &[')[1].split(b'];')[0]);assert ids==validator and len(ids)==13
assert b'category: category.clone()' in sources['crates/adventuresim-core/src/life_simulation.rs'] and b'BookTarget::Bestiary { category }' in sources['crates/adventuresim-stdb-module/src/time/schedule.rs']
native=a['native_semantics_evidence'][0];nativebody=contents(native['path']);assert sha(nativebody)==native['sha256_uncompressed'] and native['personally_inspected_ranges']==[[1193,1208]] and b'It is also neither smaller nor greater than any float' in nativebody and 1000*65//100==650
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
path='segovb/READ-L12-719.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-719.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
r=next(x for x in data['rows'] if x['original_index']==2157);assert r['fresh_personal_body_reads']==[[1,112],[140,285]] and r['connected_gap_personal_reads'][0]['ranges']==[[113,139],[286,330]] and r['combined_actual_body_assessed']==[[1,330]] and r['source_sha256']==sha(body) and r['connected_gap_personal_reads'][0]['registration'] in records
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-338.json']);r=next(x for x in data['sources'] if x['original_index']==2157);assert data['accepted_unique_count_change']==0 and r['fresh_reads']==[[1,330]] and r['sha256']==sha(body)
l2=json.loads(records['root-l2-extension-20261030/READ-ROOT-L2-EXT-021-033.json'])['rows'][6];l3=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-020-027.json'])['rows'][6];cur=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-CURRENT-004.json'])['rows'][5]
assert l2['fresh_personal_ranges']==[[113,139],[286,330]] and l2['qualified_inherited_same_lens_ranges']==[[1,112],[140,285]] and l3['fresh_personal_ranges']==[[1,330]] and l3['qualified_inherited_same_lens_ranges']==[] and cur['fresh_personal_ranges']==[] and cur['qualified_reuse_ranges']==[[1,330]] and cur['revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f'
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (3130,2373,1804):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert old[3130]['contract_id']=='REVIEW-L3-BESTIARY-EXPERIENCE-ADMISSION'
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original scoped role and gaps, distinct reviewer attribution, selected category sources and pinned NaN semantics; no product execution.')
