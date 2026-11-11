#!/usr/bin/env python3
"""Verify recovered source and arithmetic evidence without executing product code."""
import gzip,hashlib,json,tarfile
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-CALENDAR-ECONOMY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2158-2162.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2158:698,2159:512,2160:49,2161:83,2162:358};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={};registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']==('bounded-current-trace-only' if i==2158 else 'unassessed') and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes)
  assert [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in registry['files'][i].get('owner_contract_proofs',[])];current[i]=body
  if i==2158:assert item['repeated_prior_current_ranges']==[[1,112],[245,260],[320,330]] and len(item['prior_current_observations'])==2
  if i==2162:
   assert item['original_supplier_scopes'][0]['read_ranges']==[[1,145]];reg=item['connected_range_registration'];assert reg['ranges']==[[146,358]] and sha((root/reg['path']).read_bytes())==reg['sha256'];assert 'preceded' in reg['qualification']
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
calendar=current[2158];coords=current[2159];bounds=current[2160];economy=current[2162];schedule=sources['crates/adventuresim-core/src/strategic_schedule.rs'];allocation=sources['crates/adventuresim-core/src/strategic_schedule/allocation.rs'];native_import=sources['crates/adventuresim-stdb-module/src/strategic/world_import.rs'];validation=sources['crates/adventuresim-world-import/src/validation.rs'];world=sources['crates/adventuresim-world-schema/src/lib.rs'];compiler=sources['crates/adventuresim-world-import/src/sources/viabundus/mod.rs']
assert b'self.minutes.saturating_mul(share_minutes) / MINUTES_PER_DAY' in calendar and b'cumulative_daily_share_minutes(leisure)' in schedule and b'MINUTES_PER_DAY.saturating_sub(schedule.allocated_minutes())' in schedule and b'pub struct DailySchedule {' in allocation and b'restorative_leisure_minutes(' in sources['crates/adventuresim-stdb-module/src/time/settlement_rest.rs']
maximum=2**64-1;start=maximum//1440+1;assert start==12810238940076078 and start+1<=maximum and start*1440>maximum and (start+1)*1440>maximum and (start+1)-start==1
assert b'u16::try_from(population_level.max(1)).unwrap_or(1) * 85' in economy and 772*85==65620>65535 and b'pub population_level: i32' in world and b'population_level: settlement.population_level' in native_import and b'validate_final_settlement_economies(ctx)?' in native_import and b'if !(1..=5).contains(&settlement.population_level)' in validation and b'0..=1 => 1' in compiler and b'_ => 5' in compiler
assert validation.index(b'validate_world_semantics(world)?')<validation.index(b'if !(1..=5).contains(&settlement.population_level)')
assert economy.index(b'add(StockCategory::Meat, 1, gap)')<economy.index(b'add(StockCategory::Meat,3,ProfileFactProvenance::DerivedFromCanonicalEvidence)') and b'.and_modify(|v| v.abundance = v.abundance.max(abundance))' in economy and b'AgriculturalCommodity::Dairy => StockCategory::Dairy' in economy and b'AgriculturalCommodity::Hides => StockCategory::Hides' in economy and b'"provenance":enum_unit(match v.provenance' in sources['crates/adventuresim-world-import/src/main.rs']
assert b'pub fn validate(&self) -> Result<(), String>' in economy and b'Error::Validation(format!' in sources['crates/adventuresim-world-import/src/sources/economies.rs'] and b'pub fn finish_world_data_import' in native_import
assert b'#[serde(try_from = "i32", into = "i32")]' in calendar and b'impl<\'de> serde::Deserialize<\'de> for LatitudeE7' in coords and b'Antimeridian-crossing' in bounds
native=a['native_semantics_evidence'];arc=(root/native[0]['archive']).read_bytes();assert sha(arc)==native[0]['archive_sha256']
with tarfile.open(root/native[0]['archive'],'r:gz') as ar:member=ar.extractfile(native[0]['member']).read()
assert sha(member)==native[0]['member_sha256'] and b'cont.attrs.type_try_from()' in member and b'TryFrom::try_from(v).map_err' in member and native[0]['personally_inspected_ranges']==[[306,312],[434,440]]
ref=contents(native[1]['path']);assert sha(ref)==native[1]['sha256_uncompressed'] and b'Integer operators will panic when they overflow' in ref
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for batch,indices in [(720,[2158,2159,2160,2161]),(721,[2162])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  r=next(x for x in data['rows'] if x['original_index']==i);assert r['fresh_personal_body_reads']==[[1,lengths[i]]] and r['source_sha256']==sha(current[i])
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
 if batch==721:
  r=next(x for x in data['rows'] if x['original_index']==2163);assert r['fresh_personal_body_reads']==[[120,308],[315,530]]
data=json.loads(records['adler/READ-L11-338.json']);assert data['accepted_unique_count_change']==0
for i in lengths:
 r=next(x for x in data['sources'] if x['original_index']==i);assert r['fresh_reads']==[[1,lengths[i]]] and r['sha256']==sha(current[i])
assert b'allDerived-beforegap' in records['adler/READ-L11-338.json'] and b'line289' in json.dumps(a['observations']).encode()
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (132,603,1082,1804,1805,1806):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert old[132]['contract_id']=='ADLER-L11-CALENDAR-DAILY-SHARE-ARITHMETIC'
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, scoped economy extension, original attributions, pinned source and symbolic arithmetic; no product execution.')
