#!/usr/bin/env python3
"""Verify quickstep evidence and historical provenance, not product behavior."""
import gzip
import hashlib
import json
import tarfile
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
        info=nodes[current];raw=contents(info['preserved_bytes'])
        assert oid('tree',raw)==current==part['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child']);current=part['child']
    assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e'
name='ASSESSMENT-QUICKSTEP-FORCE-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1886-1887.json'
a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1886:1738,1887:18};ranges={1886:[[180,228],[285,330],[530,598],[625,650],[705,720]],1887:[[1,18]]}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
current={}
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
    body=contents(item['preserved_bytes'])
    assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
motor=current[1886];controller=current[1887]
assert b'biological_mass_kg.max(0.0) * motor.gravity_metres_per_second_squared * bodyweights' in motor
assert b'let horizontal_force = maximum_force * force_scale' in motor
assert b'velocity.y += vertical_net_force / mass_kg * time.delta_secs()' in motor
assert b'acceleration_metres_per_second_squared: second_derivative_u' in motor and b'/ (duration * duration)' in motor
assert b'if phase >= 1.0 || duration.is_zero()' in motor
assert motor.index(b'controller_config::apply_motor_config') < motor.index(b'let target = match quickstep_motion_target(')
assert b'let requested_force = -velocity.xz() * mass_kg.max(1.0) / time.delta_secs()' in motor
assert b'controller.acceleration_hz = 0.0' in controller and b'controller.gravity = motor.gravity_metres_per_second_squared' in controller
clock=sources['crates/adventuresim-tactical-core/src/physics/quickstep.rs']
assert b'.saturating_add(AnimationSampleSpan::ONE)' in clock and b'.round()\n                    .max(1.0) as u64' in clock
plugin=sources['crates/adventuresim-tactical-core/src/physics/plugin.rs']
assert b'apply_character_motor' in plugin and b'.before(AhoySystems::MoveCharacters)' in plugin
config=sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'movement.motor.gravity_metres_per_second_squared' in config and b'.all(|value| finite_nonnegative(value) && value > 0.0)' in config
assert b'movement.motor.quickstep_takeoff_angle_degrees >= 45.0' in config
assert b'pub const fn from_seconds(seconds: f32)' in sources['crates/adventuresim-core/src/elapsed.rs']
start=sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'pub(crate) fn begin_authoritative_quickstep(' in start and b'quickstep_push.begin(start, direction, orientation, origin)' in start
assert b'config\n        .validate()' in sources['crates/adventuresim-tactical-server/src/startup.rs']
assert b'gravity_metres_per_second_squared: 9.81' in sources['content/tactical/combat.yaml']
assert b'quickstep_takeoff_angle_degrees: 9.0' in sources['content/tactical/combat.yaml']
assert b'sample_hz: 64.0' in sources['content/tactical/combat.yaml']
# Exact integer facts only: no floating-point motor or product function is executed.
fmax=(2**24-1)*2**104
assert 70*10**37>fmax and 64//2==32
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(622,[1886]),(623,[1887,1888])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);assert [x['original_index'] for x in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
    check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['same_lens_reuse']==[]
        if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
z=json.loads(records['adler/READ-L11-287.json']);assert [x['original_index'] for x in z['sources']]==[1886,1887]
check=json.loads(records['adler/VERIFY-L11-287.json']);assert check['receipt_sha256']==lookup['adler/READ-L11-287.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-287.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    path='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert path in records and row['registration_sha256']==lookup[path]['sha256']
prefix='evidence/continuation-segovb/audit-resume-20261027/lens08/'
batch=json.loads(records[prefix+'batch-015-originals1886-1893.json']);reg=json.loads(records[prefix+'REGISTRATION-033.json'])
assert batch['lens']==8 and batch['qualified_observation']['id']=='CONT08-MOTOR-FAILURE-UPDATE-ORDER'
assert batch['other_lens_credit'] is False and batch['new_unique_count']==0 and batch['project_execution'] is False
assert [x['original'] for x in reg['sources']]==list(range(1886,1894))
for row in reg['sources']:
    source=prefix+row['head_object'].rsplit('/',1)[-1];delta=prefix+row['delta']['object'].rsplit('/',1)[-1]
    assert source in records and delta in records and row['head_sha256']==lookup[source]['sha256'] and row['delta']['sha256']==lookup[delta]['sha256']
    assert len(contents(next(x['preserved_bytes'] for x in a['recovered_historical_records'] if x['original_path']==delta)).splitlines())==row['delta']['lines']
q=load('RECONCILIATION-059.json')['entries']
for i in (4,108,1560):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings=a['conditional_findings'];assert [x['id'] for x in findings]==['ADLER-L11-QUICKSTEP-TARGET-ACCELERATION-ADMISSION','CONT08-MOTOR-FAILURE-UPDATE-ORDER','REVIEW-QUICKSTEP-PEAK-FORCE-OVERFLOW-001']
assert all(x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed' for x in findings)
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert all(sum(x['id']==f['id'] and x['record']==name for x in ledger)==1 for f in findings)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified two historical roles, earlier behavior records and bounded force/clock traces; no product execution.')
