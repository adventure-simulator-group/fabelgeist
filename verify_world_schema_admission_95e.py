#!/usr/bin/env python3
"""Verify separately scoped source evidence, not product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-WORLD-SCHEMA-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2164.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
item=h['historical_source_roles'][0];assert len(h['historical_source_roles'])==1 and item['original_index']==2164 and item['historical_requested_union']==[[1,330],[1633,1690],[1768,1909],[2670,3510]] and item['separately_registered_historical_enum_tail']==[[331,333]] and item['new_personal_read_ranges']==[[1,4796]]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));assert [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in registry['files'][2164]['owner_contract_proofs']]
for reg in item['local_registrations']:assert sha((root/reg['path']).read_bytes())==reg['sha256']
registered=item['historical_requested_union']+item['separately_registered_historical_enum_tail']+item['new_connected_import_range']+item['new_connected_remainder_ranges'];cursor=1
for lo,hi in sorted(registered):assert lo==cursor and hi>=lo;cursor=hi+1
assert cursor==4797
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:before=archive.extractfile(item['sha256']).read()
c=item['current_comparison'];body=contents(c['preserved_bytes']);row=work['roles'][2164]
assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and len(before.splitlines())==4796 and sha(before)==row['historical_sha256'] and before==body
assert c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,4796]] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes)
sources={}
for item in a['current_supporting_sources']:
 source=contents(item['preserved_bytes']);assert item['revision']==rev and sha(source)==item['sha256'] and len(source)==item['bytes'] and len(source.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',source)==item['git_blob']==tree[item['path']];sources[item['path']]=source
coords=sources['crates/adventuresim-world-schema/src/coordinates.rs'];validator=sources['crates/adventuresim-world-import/src/validation.rs'];main=sources['crates/adventuresim-world-import/src/main.rs'];native_import=sources['crates/adventuresim-stdb-module/src/strategic/world_import.rs'];simulation=sources['crates/adventuresim-stdb-module/src/simulation.rs'];catalog=sources['crates/adventuresim-core/src/organization_catalog_validation.rs'];schedule=sources['crates/adventuresim-core/src/strategic_schedule.rs'];org=sources['crates/adventuresim-core/src/organization.rs'];lock=sources['Cargo.lock']
assert b'pub longitude_e7: i32' in body and b'pub latitude_e7: i32' in body and b'LongitudeE7::new(self.longitude_e7)' in body and b'.expect("travel geometry longitude was validated at construction")' in body and b'pub const MAX: Self = Self(180 * Self::UNITS_PER_DEGREE)' in coords and 180*10000000<2000000000<=2**31-1
assert b'let (lat, lon) = (point.latitude(), point.longitude());' in validator and b'validate_inferred_geometry(edge, &node_coordinates)?' in validator and b'let world: CompiledWorld = serde_json::from_slice(&artifact)?;' in main and b'validate_world(&world)?;' in main
edge=body.split(b'pub struct TravelEdgeLoad {',1)[1].split(b'\n}',1)[0];assert b'geometry:' not in edge and b'pub geometry: Vec<TravelGeometryPoint>' in body
assert b'if self.direct(religion) <= 0.0 {' in body and b'if direct.is_finite()' in body and b'[1.0, 0.80, 0.75' in body and b'simulation_religion_hours_valid(skills.religion_hours)' in simulation and 1000*80//100==800
assert b'value.validate().map_err(serde::de::Error::custom)?;' in body and b'pub fn validate_context(&self, route: &TravelRoute, length_m: u32)' in body and b'validate_context(&edge.route, edge.length_m)' in validator and b'validate_context(&edge.route, edge.length_m)' in native_import
religions=re.findall(rb'Self::[A-Za-z]+ => "([a-z_]+)"',body.split(b'pub const fn religion_id(self)')[1].split(b'pub const fn label(self)')[0]);catalogue=re.findall(rb'"([a-z_]+)"',catalog.split(b'const RELIGIONS: &[&str] = &[')[1].split(b'];')[0]);assert religions==catalogue and len(religions)==7 and b'Religion {\n        religion: String,' in org and b'OfficialReligion::from_id(religion)' in schedule
assert b'candidates: Vec<ModeledTreeSpecies>' in body and b'species: Vec<TreeSpeciesId>' in body and b'const MAX_MODELED_TREE_SPECIES: usize = 12' in body and b'const MAX_INFERRED_TREE_SPECIES: usize = 4' in body and b'let wire = Wire::deserialize(deserializer)?;\n        Self::new(wire.candidates)' in body
native=a['native_semantics_evidence'];members=[]
for x in native[:3]:
 arc=(root/x['archive']).read_bytes();assert sha(arc)==x['archive_sha256'] and x['archive_sha256'].encode() in lock
 with tarfile.open(root/x['archive'],'r:gz') as ar:raw=ar.extractfile(x['member']).read()
 assert sha(raw)==x['member_sha256'];members.append(raw)
assert b'MapAccess::next_value::<#field_ty>' in members[0] and b'Self::Value::try_from(v as u64)' in members[1] and b'while let Some(value) = tri!(seq.next_element())' in members[1] and b'values.push(value)' in members[1] and b'ParserNumber::U64(significand)' in members[2] and b'deserialize_number!(deserialize_i32)' in members[2]
ref=contents(native[3]['path']);assert sha(ref)==native[3]['sha256_uncompressed'] and b'It is also neither smaller nor greater than any float' in ref
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
path='segovb/READ-L12-722.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-722.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
r=next(x for x in data['rows'] if x['original_index']==2164);assert r['fresh_personal_body_reads']==[[1,330],[1633,1690],[1768,1909],[2670,3510]] and r['whole_file_coverage'] is False and r['source_sha256']==sha(body)
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-340.json']);r=next(x for x in data['sources'] if x['original_index']==2164);assert data['accepted_unique_count_change']==0 and r['fresh_reads']==[[1,333],[1633,1690],[1768,1909],[2670,3510]] and r['sha256']==sha(body) and b'globallytruncated receivedzero credit' in records['adler/READ-L11-340.json']
l3=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-035-042.json'])['rows'][0];assert l3['fresh_personal_ranges']==[[1,333],[1633,1690],[1768,1909],[2670,3510]] and l3['qualified_inherited_same_lens_ranges']==[]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (133,3139,3140,2493,1807,1806):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original scoped world-schema evidence, separately registered full body, original attributions and pinned decoding sources; no product execution.')
