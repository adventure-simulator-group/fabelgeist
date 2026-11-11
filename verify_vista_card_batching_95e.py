#!/usr/bin/env python3
"""Verify source provenance and symbolic coordinate/distance gates only."""
import difflib
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
    out,pos={},0
    while pos<len(b):
        end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
    assert pos==len(b);return out
def chain(path,r,body,root_tree,nodes):
    assert [s['name'] for s in r['chain']]==path.split('/')
    current=root_tree
    for s in r['chain']:
        info=nodes[current];data=contents(info['preserved_bytes']);assert oid('tree',data)==current==s['tree'] and sha(data)==info['sha256'] and len(data)==info['bytes']
        assert entries(data)[s['name']]==(s['mode'],s['child']);current=s['child']
    assert current==r['git_blob']==oid('blob',body) and sha(body)==r['sha256'] and len(body)==r['bytes'] and len(body.splitlines())==r['lines']
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-VISTA-CARD-BATCHING-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1637-1641.json');w=load('COVERAGE-L01-ROLES.json');proof=load('VISTA-CARD-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1637,1642))
lengths={1637:362,1638:21,1639:144,1640:144,1641:253};clengths={1637:345,1638:21,1639:144,1640:144,1641:247};ranges={1637:[[1,190],[191,362]],1638:[[1,21]],1639:[[1,144]],1640:[[1,144]],1641:[[1,253]]};cranges={1637:[[1,175],[176,345]],1638:[[1,21]],1639:[[1,144]],1640:[[1,144]],1641:[[1,125],[126,247]]};current={};historical={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];b=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges[i] and len(old.splitlines())==lengths[i] and sha(old)==r['historical_sha256']
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1637-1641.json'
        assert sha(b)==c['sha256'] and len(b)==c['bytes'] and len(b.splitlines())==clengths[i] and c['current_personal_ranges']==cranges[i] and oid('blob',b)==c['git_blob']==tree[c['path']]
        assert c['byte_identical']==(i not in (1637,1641))
        if i in (1637,1641):
            diff=''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),b.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode();assert diff==contents(c['exact_diff']['preserved_bytes']) and sha(diff)==c['exact_diff']['sha256']
        else:assert old==b
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);historical[i]=old;current[i]=b
    e,=a['earlier_historical_source_identities'];r=w['roles'][1642];body=ar.extractfile(e['sha256']).read();path=e['source_id'].split(':',1)[1]
    assert e['original_index']==1642 and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0 and sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
    lr=a['historical_lock_excerpt'];lock=ar.extractfile(lr['sha256']).read();assert lr['original_index']==497 and lr['personally_inspected_ranges']==[[1649,1652]] and sha(lock)==w['roles'][497]['historical_sha256'];chain('Cargo.lock',proof['paths']['Cargo.lock'],lock,proof['root_tree'],nodes)
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
native=load(a['native_source_evidence']['manifest']);archive=root/'native-evidence/bevy_camera-0.19.1/bevy_camera-0.19.1.crate';assert sha(archive.read_bytes())==native['sha256']=='96de20d0babfb51e3907eb4eb56f1e36096a6f8c77acd997f5de611df4ee067d'
assert native['sha256'].encode() in lock and native['sha256'].encode() in sources['Cargo.lock']
member=a['native_source_evidence']['member'];assert all(r in native['personally_inspected_ranges'][member] for r in [[1,170],[250,275]])
with tarfile.open(archive,'r:gz') as ar:
    b=ar.extractfile(member).read();assert sha(b)==next(x['sha256'] for x in native['inspected_members'] if x['path']==member)
    assert b'(view_position - model_position).length()' in b and b'camera_distance >= self.start_margin.start' in b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
for n,indices in [(519,[1637,1638,1639]),(520,[1640,1641,1642])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
prev=load('ASSESSMENT-VISTA-SELECTED-LOD-95E.json');r=next(x for x in prev['recovered_historical_records'] if x['original_path']=='adler/READ-L11-235.json');z=json.loads(contents(r['preserved_bytes']));assert r['sha256']==lookup[r['original_path']]['sha256'] and [x['original_index'] for x in z['sources']]==list(range(1636,1642))
cards=sources['crates/adventuresim-tactical-client/src/presentation/vista/tree_cards.rs'];chunk=sources['crates/adventuresim-tactical-client/src/presentation/chunk_instancing/spawn.rs'];abi=sources['crates/adventuresim-tactical-client/src/presentation/chunk_instancing/chunk_instancing.wgsl'];shader=sources['assets/shaders/tactical_vista_tree_instanced.wgsl'];surface=sources['crates/adventuresim-tactical-client/src/presentation/vista/surface.rs'];batches=sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass/batches.rs'];dispatcher=sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs']
assert b'lod.origin_east_metres as f32' in current[1637] and b'position: Vec3::new(local.x, height.metres(), local.y)' in current[1637]
assert b'instances: group.instances' in cards and b'.spawn(commands, |chunk|' in cards and b'Instances are already in world space' in abi and b'vec4<f32>(i_pos_scale.xyz, 1.0)' in abi
assert b'Transform::from_translation(origin)' in surface and b'origin_east_metres: 0.0' in dispatcher
assert b'Transform::from_xyz(local.x, height.metres(), local.y)' in historical[1637] and b'use_aabb: false' in historical[1637]
assert b'const VISTA_TREE_HALF_ANGLE_TANGENT: f32 = 0.14054084' in shader and b'distance(view.world_position.xz, in.world_position.xz)' in shader and b'if camera_distance < in.near_cutoff' in shader
assert b'NotShadowCaster' in historical[1641] and b'InstanceBatch {' in batches and b'NotShadowCaster,' in chunk
# Symbolic predicate arithmetic, not a rendering test.
cutoff=Fraction(10,1)/(2*Fraction(14054084,100000000));assert 35<cutoff<36 and 20<cutoff and 20**2+50**2>(cutoff*Fraction(112,100))**2
assert 20+100==120
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1359,1364))
expected=['REVIEW-VISTA-TREE-ORIGIN-FRAME-001','REVIEW-VISTA-CARD-NEAR-DISTANCE-BEHAVIOR-001'];assert [x['id'] for x in a['conditional_findings']]==expected
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries']
for x in a['conditional_findings']:
    assert x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed'
    assert sum(r['id']==x['id'] and r['record']=='ASSESSMENT-VISTA-CARD-BATCHING-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified five historical roles, native distance provenance and symbolic card gates; no product execution.')
