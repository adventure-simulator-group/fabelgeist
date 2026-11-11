#!/usr/bin/env python3
"""Verify evidence provenance and static source contracts, never product behavior."""
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
name='ASSESSMENT-GROUND-CARDINALITY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1910-1913.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1910:173,1911:38,1912:136,1913:88};ranges={1910:[],1911:[],1912:[],1913:[[1,88]]};current={}
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
ground=current[1913];velocity=current[1911];window=current[1912]
assert b'let sample = &mut samples[sample_z * width + sample_x]' in ground
assert b'SceneGround::from_samples(width, depth, spacing, samples)' in ground
assert b'SignedSpeed::from_metres_per_second(value.x)?' in velocity and b'SignedSpeed::from_metres_per_second(value.z)?' in velocity
assert b'pub struct SceneWindowPose' in window and b'RigidRotation::from_quaternion' in window
source=sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'grid.heights_metres.len() != expected || grid.environment.len() != expected' in source
assert b'source.heights_metres.clone()' in source and b'source.environment.clone()' in source
assert b'heights.push(lerp(north, south, tz))' in source and b'environment.push(source.environment[nearest_z * source_width + nearest_x])' in source
geo=sources['crates/adventuresim-tactical-core/src/scene_input/generation/geographic.rs']
assert geo.index(b'self.validate_source()?')<geo.index(b'upsample_playable_grid(&self.playable)')<geo.index(b'build_scene_ground(')
assert b'environment: &mut [EnvironmentalSample]' in sources['crates/adventuresim-tactical-core/src/scene_input/generation.rs']
assert b'terrain.grid_depth_checked()? != grid_depth' in sources['crates/adventuresim-tactical-core/src/scene.rs']
assert b'SceneVelocity::from_metres_per_second' in sources['crates/adventuresim-tactical-client/src/camera/smoothing.rs']
assert b'Some(history) if !reset => history.clone()' in sources['crates/adventuresim-tactical-client/src/camera/rig_history.rs']
assert b'state.commit_observation(next)' in sources['crates/adventuresim-tactical-client/src/camera/rig.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(636,[1910,1911,1912]),(637,[1913])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
assert 'missingenvironmentcardinality couldpanic' in json.loads(records['segovb/READ-L12-637.json'])['rows'][0]['function_contract']
prior=load('ASSESSMENT-ROUNDED-POLYGON-TOPOLOGY-95E.json');rec=next(x for x in prior['recovered_historical_records'] if x['original_path']=='adler/READ-L11-291.json');assert sha(contents(rec['preserved_bytes']))==rec['published_sha256'] and rec['sha256']==lookup['adler/READ-L11-291.json']['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1570,1576,1577,1578):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert a['conditional_findings']==[]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(ledger)>=346
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, prior reviewer attribution and selected cardinality/pose source evidence; no product execution.')
