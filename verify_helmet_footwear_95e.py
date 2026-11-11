#!/usr/bin/env python3
"""Verify source and recovery evidence only; do not run product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HELMET-FOOTWEAR-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2184-2188.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2184,2189))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['sha256']==item['sha256'] and regrow['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and old==body and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,n]]
  assert c['byte_identical'] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
foot=bodies[2184];burgonet=bodies[2185];helmet=bodies[2186];parts=bodies[2187];surface=bodies[2188];limb=sources['crates/fabelgeist-armor/src/limb_armor.rs'];extremities=sources['crates/fabelgeist-armor/src/gpu/extremities.rs'];crown=sources['crates/fabelgeist-armor/src/helmet_crown.rs'];design=sources['crates/fabelgeist-armor/src/helmets.rs'];buffe=sources['crates/fabelgeist-armor/src/helmets_buffe.rs'];courses=sources['crates/fabelgeist-armor/src/helmets_buffe_courses.rs'];breaths=sources['crates/fabelgeist-armor/src/visor_breaths.rs'];pierced=sources['crates/fabelgeist-armor/src/pierced_plate_domain.rs'];ui=sources['crates/adventuresim-character-creator/src/buffe_controls.rs'];fluting=sources['crates/fabelgeist-armor/src/plate_fluting.rs'];codes=sources['crates/fabelgeist-armor/src/gpu/helmet/codes.rs']
assert b'design.validate()?' in helmet and b'crown.validate()?' in design and b'pattern.validate()?' in crown and b'COUNT_RANGE: RangeInclusive<u16> = 2..=64' in fluting and b'samples.extend([left, right])' in fluting and b'dedup_by' in fluting and b'const AROUND: usize = 48' in surface and b'const FAN_COLUMNS: usize = 40' in surface
assert b'design.validate()?' in extremities and b'(40..=350).contains(&d.shaft_height.0)' in limb and b'(3..=8).contains(&d.lame_count)' in limb and b'(rise - upper_height) / shaft_height' in foot
assert b'if let Some(buffe) = &d.buffe' in design and b'buffe.validate()?' in design and b'courses.validate()?' in buffe and b'breaths.validate(50..=950)?' in buffe and b'!(2..=3).contains(&self.plate_count)' in courses and b'upper_boundary: Permille(700)' in courses and b'overlap: Millimeters(5)' in courses
assert b'height: Permille(160)' in breaths and b'count_per_row: 4' in breaths and b'length: Millimeters(12)' in breaths and b'width: Millimeters(3)' in breaths and b'height: Permille(820)' in breaths and b'chart_height / 1000.0' in breaths and b'breaths.openings(BUFFE_CHART_HEIGHT_MM' in burgonet
assert b'!inside(*p, outer) || edge_distance(*p, outer) < MINIMUM_WEB_MM' in pierced and b'DesignError::VisorOpeningSpacing.into()' in pierced and b'"Overlapping buffe faceplates"' in ui and b'breaths, 50..=950' in ui
course_height=Fraction(100)*(1-Fraction(700,1000))+5;center=Fraction(820,1000)*100;assert course_height==35 and center-6>course_height and 4*3+3*(4-1)<=54 and 2*36-54>=3
assert b'shared_codes!' in codes and b'$name::$variant as u32' in codes and b'kind as u32 as f32' in surface and b'ChartKernel::new(gpu, &format!("{design}{shape}"))' in extremities
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(727,[2184]),(728,[2185,2186,2187]),(729,[2188])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);assert x['source_sha256']==roles[i-2184]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
  expected=[[120,165],[200,224]] if i==2188 else [[1,roles[i-2184]['lines']]];assert x['fresh_personal_body_reads']==expected and x['assessed_ranges']==expected
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(344,[2184,2185]),(345,[2186,2187,2188])]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2184]['lines']]] and row['sha256']==roles[i-2184]['sha256']
qual=json.loads(records['parallel-l12-20261029/QUALIFICATION-PAR-L12-ORIGINAL2188-WINDOWS.json']);assert qual['body_read_credit']==0 and qual['ranges']==[[120,165],[200,224]] and qual['raw_locator']==registry['files'][2188]['path']
l2=json.loads(records['root-l2-extension-20261030/READ-ROOT-L2-EXT-034-044.json'])['rows'][1];l3=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-035-042.json'])['rows'][1];assert l2['fresh_personal_ranges']==[[1,235]] and l3['fresh_personal_ranges']==[[1,235]] and l3['qualified_inherited_same_lens_ranges']==[]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (2404,2691,3141):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified scoped helmet attribution, independent full reads, selected design predicates and conditional course geometry; no product execution.')
