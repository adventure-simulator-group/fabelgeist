#!/usr/bin/env python3
"""Verify recovered gate source and qualified source-only trace; no product runs."""
import gzip,hashlib,json,tarfile,tomllib
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
 assert [x['name'] for x in proof['chain']]==path.split('/')
 current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes'])
  assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
  assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';physical='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
name='ASSESSMENT-GATE-HANDEDNESS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2046-2047.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
oldproof=load(proof['additional_shared_physical_tree_objects']);nodes.update(oldproof['tree_objects']);nodes.update(proof['additional_tree_objects'])
commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={2046:672,2047:100};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
  chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
p=proof['physical_2529_comparison'];commit=contents(oldproof['revisions'][physical]['commit']['preserved_bytes'])
assert p['commit']==physical and oid('commit',commit)==physical and commit.splitlines()[0]==b'tree '+p['root_tree'].encode();chain(p['path'],p['proof'],current[2046],p['root_tree'],nodes)
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
 assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
server=current[2046];gate=sources['crates/adventuresim-tactical-core/src/city_layout/compound/boundary.rs'];sides=sources['crates/adventuresim-tactical-core/src/city_layout/compound.rs'];property=sources['crates/adventuresim-tactical-core/src/city_layout/compiled/property.rs'];grabbing=sources['crates/adventuresim-tactical-core/src/doors.rs'];spatial=sources['crates/adventuresim-tactical-core/src/scene_coordinates/spatial.rs'];opening=sources['crates/adventuresim-tactical-server/src/openings.rs'];windows=sources['crates/adventuresim-tactical-server/src/windows.rs']
assert b'Self::Right => 1.0' in sides and b'Self::Left => -1.0' in sides
assert b'closed_centre\n                        + Vec3::new(tangent.x, 0.0, tangent.y)' in gate and b'* self.hinge.sign()' in gate
assert b'let tangent = self.orientation.local_to_world(Vec2::X)' in gate and b'let outward = self.orientation.local_to_world(-Vec2::Y)' in gate
assert b'const GATE_WIDTH_METRES: f32 = 1.6;' in property and b'hinge: lot.passage_side.opposite()' in property
assert b'hinge_centre: self.point(leaf.hinge_centre)?' in spatial and b'closed_centre: self.point(leaf.closed_centre)?' in spatial
assert b'hinge_centre + tangent.vector() * size.x * 0.5' in server and b'doorway_centre_metres: doorway_centre' in server and b'doorway_centre,' in server
assert b'DOOR_GRAB_LATERAL_MARGIN_METRES: f32 = 0.6;' in grabbing and b'(-DOOR_GRAB_DEPTH_METRES..=-DOOR_INTERIOR_SIDE_EPSILON_METRES).contains(&signed_depth)' in grabbing
assert b'lateral_distance <= half_width_metres + DOOR_GRAB_LATERAL_MARGIN_METRES' in grabbing and b'passage_decision(signed_depth, within_passage, was_exempt)' in server
assert b'position.0 + rotation.0 * Vec3::new(-door.size_metres.metres().x * 0.5, 0.0, 0.0)' in server and b'MAX_HINGE_SEPARATION_METRES: f32 = 0.03' in server
# Exact rational algebra explains the symbolic, axis-aligned source example; no product math runs.
w=Fraction(8,5);half=w/2;assert (half+half)==w and w>half+Fraction(3,5) and (-half-half).__abs__()>Fraction(3,100)
assert opening.index(b'doors::spawn_building_doors(')<opening.index(b'windows::spawn_building_windows(')<opening.index(b'compile_tactical_building_collider(&collision)?')
assert b'for window in compile_operable_windows(plan)?' in windows and b'spawn_window(commands, building, datum.window(window)?);' in windows
# Native archived source only: error handler runs before deferred commands are queued.
native_refs=a['native_source_evidence'];native=load(native_refs[0]['manifest']);archive=root/native_refs[0]['archive'];raw=archive.read_bytes();assert sha(raw)==native['sha256'] and len(raw)==native['bytes']
lock=contents('main-advance-95e/tree-stream-evidence/current-cargo-lock.source.gz');assert oid('blob',lock)==tree['Cargo.lock'];bevy=next(x for x in tomllib.loads(lock.decode())['package'] if x['name']=='bevy_ecs' and x['version']=='0.19.1');assert bevy['checksum']==native['sha256'];nativebody={}
with tarfile.open(archive,'r:gz') as package:
 for ref in native_refs:
  body=package.extractfile(ref['member']).read();assert sha(body)==ref['sha256'] and len(body)==ref['bytes'] and all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in ref['personally_inspected_ranges'])
  item=next((x for x in native['inspected_members'] if x['path']==ref['member']),None);assert ref['previously_indexed_member']==(item is not None)
  if item:assert sha(body)==item['sha256'] and len(body)==item['bytes']
  nativebody[ref['member']]=body
runner=nativebody['bevy_ecs-0.19.1/src/observer/runner.rs'];handler=nativebody['bevy_ecs-0.19.1/src/error/handler.rs'];err=nativebody['bevy_ecs-0.19.1/src/error/bevy_error.rs'];assert runner.index(b'handler(')<runner.index(b'(*system).queue_deferred') and b'Self(match_severity)' in handler and b'Severity::Panic => panic(err, ctx)' in handler and b'severity: Severity::Panic' in err
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
z=json.loads(records['segovb/READ-L12-688.json']);assert [x['original_index'] for x in z['rows']]==[2046,2047] and z['current_source_reconciled'] is False
for row in z['rows']:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]] and row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
z=json.loads(records['adler/READ-L11-319.json']);assert [x['original_index'] for x in z['sources']]==list(range(2046,2054)) and z['current_reconciled'] is False
z=json.loads(records['segovb/READ-L12-858.json']);row=next(x for x in z['rows'] if x['original_index']==2529);assert z['root_personal_rereads']==[] and z['delegated_attribution_preserved'] is True and row['source_sha256']==sha(current[2046]) and row['frozen_git_revision']==physical and 'transport only' in row['revision_qualification']
assert row['fresh_personal_body_reads']==[[1,36],[49,246],[248,389]] and 'Subagent' in row['attribution'] and z['proposal'] in records and z['validated_bundle']['path'] in records
for item in z['adopted_registration_mapping']:assert item['path'] in records and item['sha256']==lookup[item['path']]['sha256']
q=load('RECONCILIATION-059.json')['entries'];original=load('baseline-059/queue.json')['entries']
for i in (1739,1740,1741):assert q[i]['contract_id']==original[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];finding=a['conditional_findings'][0];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified two historical gate roles, dirty physical receipt attribution, pinned Bevy error source and bounded gate-handedness reasoning; no product execution.')
