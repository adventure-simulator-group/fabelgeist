#!/usr/bin/env python3
"""Verify marching evidence, historical identity and native sources only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree',raw)==current==part['tree']
        assert sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child'])
        current=part['child']
    assert current==proof['git_blob']==oid('blob',body)
    assert sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009'; old='6533e444fa37552927070cc8d8190907fd3b413e'
name='ASSESSMENT-MARCHING-ADMISSION-95E.json'; hist='ASSESSMENT-L01-HISTORICAL-1882-1885.json'
a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1882:159,1883:278,1884:107,1885:90};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']=='unassessed' and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes'])
    assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
march=current[1883];height=current[1884];tests=current[1885]
assert b'(position.is_finite() && value.is_finite())' in march
assert b'let denominator = start.value - end.value' in march and b'start.value / denominator' in march
assert b'if normal.length_squared() == 0.0' in march and b'TerrainRecipeError::EmptySurface' in march
assert march.count(b'[0, ')>=6 and b'vertex(inside[0], outside[0])?' in march
assert b'dimensions.iter().any(|&side| side < 2)' in height
assert b'!(elevations[0] - height).is_finite()' in height and b'!(elevations[vertical - 1] - height).is_finite()' in height
assert b'elevations.windows(2).any(|pair| pair[0] >= pair[1])' in height
assert b'for fixture in 0..6' in tests and b'assert_eq!(trimmed, dense' in tests
fault=sources['crates/adventuresim-tactical-core/src/volumetric_terrain.rs']
assert b'marching_height_surface(' in fault and b'bottom + index[1] as f32 * spacing' in fault
assert b'origin.x + index[0] as f32 * spacing - radius' in fault and b'origin.y + index[2] as f32 * spacing - radius' in fault
nonfault=sources['crates/adventuresim-tactical-core/src/erosional_terrain.rs']
assert b'marching_tetrahedra(' in nonfault and b'if side > MAX_GRID_SIDE || vertical > MAX_GRID_HEIGHT' in nonfault
patch=sources['crates/adventuresim-tactical-core/src/volumetric_terrain/patch.rs']
assert b'#[serde(try_from = "PatchWire")]' in patch and b'Self::from_mesh_buffers(' in patch
assert b'positions.len() != normals.len()' in patch and b'!indices.len().is_multiple_of(3)' in patch
assert b'.any(|&index| index as usize >= positions.len())' in patch and b'.any(|value| !value.is_finite())' in patch
assert b'Collider::trimesh(' in fault
assert b'!(-12_000.0..=12_000.0).contains(height)' in sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'self.heightmap.iter().any(|h| !h.is_finite())' in sources['crates/adventuresim-tactical-core/src/scene/admission.rs']
native={}
for item in a['native_evidence']:
    m=load(item['manifest']);info=next(x for x in m['inspected_members'] if x['path']==item['member'])
    with tarfile.open(root/Path(item['manifest']).parent/m['archive'],'r:gz') as package:
        body=package.extractfile(item['member']).read()
    assert sha(body)==info['sha256'] and len(body)==info['bytes']
    assert all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in item['ranges'])
    assert all(any(slo<=lo and shi>=hi for slo,shi in m['personally_inspected_ranges'][item['member']]) for lo,hi in item['ranges'])
    native[item['member']]=body
assert b'self * (1.0 - s) + rhs * s' in native['glam-0.32.1/src/f32/vec3.rs']
assert b'Self::try_trimesh(vertices, indices)' in native['avian3d-0.7.0/src/collision/collider/parry/mod.rs']
assert b'TriMesh::new(vertices, indices)?' in native['parry3d-0.27.0/src/shape/shared_shape.rs']
assert b'Self::with_flags(vertices, indices, TriMeshFlags::empty())' in native['parry3d-0.27.0/src/shape/trimesh.rs']
# Exact integer facts about binary32 range; these do not execute product arithmetic.
fmax=(2**24-1)*2**104
assert 2*fmax > fmax and 2*fmax > 2**128
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
seg=json.loads(records['segovb/READ-L12-621.json']);assert [x['original_index'] for x in seg['rows']]==[1884,1885] and seg['current_source_reconciled'] is False
assert seg['plan'] in records and seg['plan_sha256']==lookup[seg['plan']]['sha256']
verify=json.loads(records['segovb/VERIFY-L12-621.json']);assert verify['receipt_sha256']==lookup['segovb/READ-L12-621.json']['sha256'] and verify['state_sha256']==lookup['segovb/STATE-L12-621.json']['sha256']
for row in seg['rows']:
    assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
    assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]] and row['same_lens_reuse']==[]
for n,indices in [(286,[1882,1883,1884,1885]),(301,[1965,1966,1967,1968,1969,1970])]:
    path=f'adler/READ-L11-{n}.json';z=json.loads(records[path]);assert [x['original_index'] for x in z['sources']]==indices
    verify=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert verify['receipt_sha256']==lookup[path]['sha256'] and verify['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z.get('connected_reinspection',[]):
        registration='adler/'+row['registration'].replace('\\','/').split('/')[-1]
        assert registration in records and row['registration_sha256']==lookup[registration]['sha256']
prior=load('ASSESSMENT-INVENTORY-CACHE-IDENTITY-95E.json');rec=next(x for x in prior['recovered_historical_records'] if x['original_path']=='segovb/READ-L12-620.json')
assert sha(contents(rec['preserved_bytes']))==rec['published_sha256'] and rec['sha256']==lookup[rec['original_path']]['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (107,1558,1559):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings=a['conditional_findings'];assert [x['id'] for x in findings]==['ADLER-L11-MARCHING-EDGE-INTERPOLATION','REVIEW-TERRAIN-PATCH-GEOMETRY-ADMISSION-001']
assert all(x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed' for x in findings)
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert all(sum(x['id']==f['id'] and x['record']==name for x in ledger)==1 for f in findings)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, prior finding, native source and bounded patch admission; no product execution.')
