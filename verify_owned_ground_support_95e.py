#!/usr/bin/env python3
"""Verify owned-support source identity, attribution and symbolic threshold evidence."""
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
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-OWNED-GROUND-SUPPORT-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1647-1651.json');w=load('COVERAGE-L01-ROLES.json');proof=load('OWNED-SUPPORT-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1647,1652))
lengths={1647:173,1648:205,1649:278,1650:24,1651:118};ranges={1647:[[1,173]],1648:[[1,205]],1649:[[1,150],[151,278]],1650:[[1,24]],1651:[[1,118]]};current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];b=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges[i] and len(old.splitlines())==lengths[i] and sha(old)==r['historical_sha256']
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1647-1651.json'
        assert sha(b)==c['sha256'] and len(b)==c['bytes'] and len(b.splitlines())==lengths[i] and c['current_personal_ranges']==ranges[i] and oid('blob',b)==c['git_blob']==tree[c['path']] and c['byte_identical'] and old==b
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);current[i]=b
    for e in a['earlier_historical_source_identities']:
        i=e['original_index'];r=w['roles'][i];body=ar.extractfile(e['sha256']).read();path=e['source_id'].split(':',1)[1]
        assert i in (1652,1653) and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0 and sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']]==[1652,1653]
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
native=load(a['native_source_evidence']['manifest']);archive=root/'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate';assert sha(archive.read_bytes())==native['sha256']=='98ead494dcfaa9b4594eed7f09a27d868b7cc7fc9ebd655c5643322778f4a8a2'
member=a['native_source_evidence']['member'];assert all(r in native['personally_inspected_ranges'][member] for r in [[563,564],[582,593]])
with tarfile.open(archive,'r:gz') as ar:
    b=ar.extractfile(member).read();assert sha(b)==next(x['sha256'] for x in native['inspected_members'] if x['path']==member) and b'length.is_finite() && length > 0.0' in b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
p='segovb/READ-L12-523.json';z=json.loads(records[p]);check=json.loads(records['segovb/VERIFY-L12-523.json'])
assert [r['original_index'] for r in z['rows']]==[1649,1650,1651] and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['segovb/STATE-L12-523.json']['sha256']
for r in z['rows']:assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
p='adler/READ-L11-237.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-237.json'])
assert [r['original_index'] for r in z['sources']]==list(range(1648,1654)) and z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-237.json']['sha256']
for r in z['sources']+z['connected_reinspection']:
    if r.get('registration_sha256'):
        reg='adler/'+r['registration'].replace('\\','/').split('/adler/')[-1];assert reg in records and r['registration_sha256']==lookup[reg]['sha256']
for r in z.get('same_lens_context',[]):
    rp='adler/'+r['receipt'].replace('\\','/').split('/adler/')[-1];assert rp in records and r['sha256']==lookup[rp]['sha256']
assert any('roundoff filtering' in x for x in z['pending_contracts'])
prev=load('ASSESSMENT-VISTA-SURFACE-FRAMES-95E.json');assert [e['original_index'] for e in prev['earlier_historical_source_identities']]==[1647,1648] and all(e['additional_original_frontier_credit']==0 for e in prev['earlier_historical_source_identities'])
prefix='crates/adventuresim-tactical-client/src/presentation/vista/'
support=sources[prefix+'streets/support.rs'];prepared=sources[prefix+'streets/prepared.rs'];focused=sources[prefix+'focused/meshes.rs'];mesh=sources[prefix+'streets/mesh.rs'];geom=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/geometry.rs'];settlement=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/settlement.rs'];prop=sources['crates/adventuresim-tactical-core/src/scene/property_support.rs']
assert b'const CLIP_EPSILON: f32 = 0.00001' in support and b'<= CLIP_EPSILON' in support and b'self.add_triangles(triangles, origin)' in support
assert b'GroundPresentation::in_rectangles(surface, &regions)' in current[1650] and b'Vec3::ZERO' in current[1650] and b'ClippedTriangles::Interior(Some(triangle))' in current[1647]
assert b'.map(|[a, b, c]| [*a, *c, *b])' in current[1648] and b'!transition_collar.is_some_and' in current[1648]
assert b'8.0 * f64::EPSILON' in geom and b'signed_area.abs() <= determinant_roundoff' in geom and b'pub fn from_foundations(' in settlement and b'!p.is_finite()' in settlement
assert b'support.add_owned_region(' in prepared and b'support.add_mesh(&chunk.mesh, chunk.origin)' in focused and b'support.clip(patch.corners' in mesh
assert b'natural_collision_mesh_with_transition' in prop and b'collar.cuts_out(point)' in prop
# Exact rational comparisons of a hypothetical source triangle; no product execution.
side=Fraction(1,512);det=side*side;assert det==Fraction(1,262144) and det<Fraction(1,100000)
assert det>8*Fraction(1,2**52)*det and side*Fraction(1,4)+side*Fraction(1,4)<side
assert det*det<Fraction(1,100000)**2
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in list(range(1369,1374))+[1604,1609])
f,=a['conditional_findings'];assert f['id']=='REVIEW-GROUND-SUPPORT-SMALL-TRIANGLE-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries'];assert sum(r['id']==f['id'] and r['record']=='ASSESSMENT-OWNED-GROUND-SUPPORT-95E.json' for r in ledger)==1
owned=next(x for x in ledger if x['id']=='REVIEW-VISTA-OWNED-SURFACE-ORIGIN-FRAME-001');assert 'ASSESSMENT-OWNED-GROUND-SUPPORT-95E.json' in owned['additional_assessments']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified five historical roles, original reviewer records and symbolic support threshold; no product execution.')
