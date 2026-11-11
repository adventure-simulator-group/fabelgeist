#!/usr/bin/env python3
"""Verify recovered and source evidence without executing product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-WEAPON-ORNAMENT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2118-2127.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2118:393,2119:211,2120:223,2121:128,2122:158,2123:116,2124:113,2125:94,2126:265,2127:98};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
base='crates/adventuresim-weapon-model/src/';pommel=sources[base+'model/pommels.rs'];graph=sources[base+'recipe/validation/graphs.rs'];hilt=sources[base+'recipe/hilt.rs'];melee=sources[base+'recipe/validation/melee.rs'];validator=sources[base+'recipe/validation.rs'];extents=sources[base+'model/extents.rs'];guard=sources[base+'model/guard_nodes.rs'];assembly=current[2119];shape=current[2118];clearance=current[2120];profile=current[2122];quantity=current[2127]
assert b'pub struct Ornament {' in assembly and b'Authored {' in assembly and b'pub sockets: Option<BTreeMap<String, [Metres; 3]>>' in hilt and b'pub ornaments: Option<Vec<Ornament>>' in hilt
assert b'!positions.is_empty()' in graph and b'positions.len() % 3 == 0' in graph and b'indices.len() % 3 == 0' in graph and b'indices.iter().all(|&i|' in graph and b'!indices.is_empty()' not in graph
assert b'if let Some(points) = &p.profile' in melee and b'profile(points, ProfileEnds::Poles)?' in melee and b'points.windows(2).all(|p| p[1][0].get() > p[0][0].get())' in validator
assert b'PommelConstruction::Composite => composite(r, p, detail)' in pommel and b'PommelBaseConstruction::Lathed => PommelConstruction::Lathed' in pommel and b'let additions = ornament_solids(ornament, detail)?' in pommel and b'for face in indices.as_chunks::<3>().0' in pommel and b'additions.push((solid.positive(), "authored ornament"))' in pommel and b'Shape::Pommel(p) => pommels::pommel(resolved, p, detail)' in shape
assert b'pub to: String' in assembly and b'pub enum AttachmentAnchor' in assembly and b'proportion((0.0..=1.0).contains(&t.get()))?' in graph and b'lerp(a, b, t.map_or(0.5, Ratio::get))' in guard
assert b'(self.section == BladeCrossSection::Recessed) == self.fuller.is_some()' in profile and b'MAX_CLEARANCE_INTERVALS: usize = 4096' in clearance and b'MAX_CLEARANCE_DEPTH: usize = 18' in clearance and b'if value.is_finite()' in quantity and b'if value == 0.0 { 0.0 }' in quantity
prior=load(a['prior_current_evidence'][0]);outsource=next(x for x in prior['current_supporting_sources'] if x['path']==base+'model/output.rs');out=contents(outsource['preserved_bytes']);assert oid('blob',out)==outsource['git_blob']==tree[outsource['path']] and b'center_of_mass: mul(moment, 1.0 / mass)' in out
r=next(x for x in prior['recovered_historical_records'] if x['original_path']=='adler/READ-L11-335.json');old335=json.loads(contents(r['preserved_bytes']));assert any('AuthoredPommelornamentempty/openindexgeometry' in s for s in old335['pending_contracts']) and old335['accepted_unique_count_change']==0
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((710,[2118,2119]),(711,[2120,2121,2122]),(712,[2123,2124,2125,2126,2127])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
  if 'exact_locator_qualification' in row:assert row['exact_locator_qualification'] in records
 if n==710:assert data['new_findings']==['SEGOVB-L12-WEAPON-FRAME-IDENTITY-TEXT-PROTOCOL']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((331,range(2118,2125)),(332,range(2125,2128))):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
assert [x['original_index'] for x in json.loads(records['adler/READ-L11-332.json'])['sources'] if 2128<=x['original_index']<=2132]==list(range(2128,2133))
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (126,127,282,1794,1795,1796,1797):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==old[i]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert next(x for x in ledger if x['id']=='ADLER-L11-GRIP-SCALE-ADMISSION')['ornament_followup']==name and next(x for x in ledger if x['id']=='ADLER-L11-ATTACHMENT-FRAME-OWNER')['frame_text_followup']==name
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified ten historical roles, historical frame attribution and bounded ornament admission chain; no product execution.')
