#!/usr/bin/env python3
"""Verify evidence provenance and selected source safeguards, never product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
    result,pos={},0
    while pos<len(body):
        end=body.index(b'\0',pos);mode,name=body[pos:end].split(b' ',1);result[name.decode()]=(mode.decode(),body[end+1:end+21].hex());pos=end+21
    assert pos==len(body);return result
def check_chain(path,proof,body,start,nodes):
    assert [x['name'] for x in proof['chain']]==path.split('/')
    current=start
    for part in proof['chain']:
        info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==part['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child']);current=part['child']
    assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e'
name='ASSESSMENT-GARDEN-CLEARANCE-SAFEGUARDS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1940-1948.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1940:152,1941:208,1942:96,1943:330,1944:204,1945:96,1946:39,1947:217,1948:87};ranges={i:([[11,76]] if i==1944 else [[20,45]] if i==1945 else []) for i in lengths};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']=='unassessed' and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==ranges[i] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
report=current[1940];garden=current[1941];support=current[1942];generation=current[1944];geo=current[1945];clearance=current[1948]
assert b'fn report_terrain_delta' in report and b'let (Some(fine), Some(coarse))' in report and b'println!' in report
assert b'pub fn from_support(' in garden and b'plant_support.len() != garden.plants.len()' in garden
assert b'GardenSupportError::MissingSoil' in support and b'.collect::<Result<_>>()?' in support
assert b'self.prepare_supported_terrain(&mut building_recipes)?' in generation and b'self.validate_source()?' in geo
assert b'CharacterController::default()' in clearance and b'.skin_width' in clearance
controller=sources['crates/adventuresim-tactical-core/src/physics.rs'];assert b'pub fn tactical_character_controller() -> CharacterController' in controller and b'..default()' in controller
motor=sources['crates/adventuresim-tactical-core/src/physics/controller_config.rs'];assert b'controller.gravity = motor.gravity_metres_per_second_squared' in motor and b'skin_width' not in motor
assert b'Collider::cylinder(HUMANOID_COLLISION_RADIUS_METRES, 1.9)' in sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'Collider::cylinder(HUMANOID_COLLISION_RADIUS_METRES, 1.9)' in sources['crates/adventuresim-tactical-server/src/iteration.rs']
assert b'self.validate()?' in sources['crates/adventuresim-tactical-core/src/scene_input/generation/supported.rs']
assert b'grid.heights_metres.len() != expected || grid.environment.len() != expected' in sources['crates/adventuresim-tactical-core/src/scene_input.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(647,[1941,1942]),(648,[1943,1944]),(649,[1945,1946,1947,1948])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
z=json.loads(records['adler/READ-L11-297.json']);check=json.loads(records['adler/VERIFY-L11-297.json']);assert [r['original_index'] for r in z['sources']]==list(range(1945,1949)) and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup['adler/READ-L11-297.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-297.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
prior=load('ASSESSMENT-INTERIOR-INSTALL-95E.json')
for path in ('segovb/READ-L12-646.json','adler/READ-L11-296.json'):
    rec=next(x for x in prior['recovered_historical_records'] if x['original_path']==path);assert sha(contents(rec['preserved_bytes']))==rec['published_sha256'] and rec['sha256']==lookup[path]['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1590,1591,1592,1593,1595,1596,1601):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert a['conditional_findings']==[]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert name in next(x for x in ledger if x['id']=='CONT08-SAMPLED-SUPPORT-FAILURE-CLASSIFICATION')['additional_current_evidence']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified nine historic roles and bounded garden/controller source safeguards; no product execution.')
