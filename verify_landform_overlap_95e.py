#!/usr/bin/env python3
"""Verify recoverable identities and conditional idealized ratios, never product execution."""
import gzip,hashlib,json,tarfile,tomllib
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-LANDFORM-OVERLAP-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1965-1970.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1965:526,1966:40,1967:99,1968:217,1969:114,1970:17};ranges={1965:[[220,270]],1966:[],1967:[],1968:[[1,161]],1969:[],1970:[]};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=w['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
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
recipe=current[1968];patch=current[1967];stencil=current[1969];support=current[1970];volume=current[1965]
assert b'PlanDirection::from_vector' in recipe and b'let extent = tangent.abs() * half_length + normal.abs() * half_width' in recipe and b'if !(0.98..=1.02).contains(&tangent.length())' in recipe
assert b'u32::from(self.collar_cm) * 2 >= u32::from(self.half_width_cm)' in recipe and b'self.collar_cm >= self.half_length_cm' in recipe
assert b'!indices.len().is_multiple_of(3)' in patch and b'any(|&index| index as usize >= positions.len())' in patch and b'any(|value| !value.is_finite())' in patch
assert b'if nx >= side || nz >= side || masks[source] <= 0.0' in stencil and b'masks[nz * side + nx]' in stencil and stencil.index(b'if nx >= side')<stencil.index(b'masks[nz * side + nx]')
assert b'Ok(terrain.required_height_at(boundary)?)' in support and b'let vertical = ((required_top - bottom) / spacing).ceil() as usize + 1;' in volume
assert b'SceneTerrainPatch::from_mesh_buffers(' in sources['crates/adventuresim-tactical-core/src/marching_tetrahedra.rs']
assert b'.validate(&terrain)' in sources['crates/adventuresim-tactical-core/src/scene_fault.rs'] and b'Self::from_normalized(value.normalize_or_zero())' in sources['crates/adventuresim-building-generator/src/spatial_geometry/orientation.rs']
assert b'if terrain.grid_depth_checked()? != grid_depth' in sources['crates/adventuresim-tactical-core/src/scene.rs'] and b'heightmap.iter().any(|h| !h.is_finite())' in sources['crates/adventuresim-tactical-core/src/scene/admission.rs']
assert b'(tangent[0] / length * 10_000.0).round()' in sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs'] and b'let downhill = -gradient.normalize()' in sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input/geological_landforms.rs']
# Rational idealization only; this does not execute or prove the native floating-point backend.
encoded=Fraction(99,100);origin=Fraction(1095,100);half_x=1;half_z=8;length=10
assert Fraction(98,100)<=encoded<=Fraction(102,100) and half_x+encoded*length==Fraction(109,10)<origin
assert abs(Fraction(1)-origin)/length==Fraction(199,200)<1 and Fraction(5,4)<half_z
native=a['native_source'];manifest=load(native['archive_manifest']);archive=(root/native['archive']).read_bytes();assert sha(archive)==manifest['sha256'] and len(archive)==manifest['bytes']
with tarfile.open(root/native['archive'],'r:gz') as package:
    member=package.extractfile(native['member']).read();assert sha(member)==native['member_sha256'] and b'self.normalize_or(Self::ZERO)' in member and b'self.length().recip()' in member
cargo_record=next(x for x in load('ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json')['current_supporting_sources'] if x['path']=='Cargo.lock');cargo=contents(cargo_record['preserved_bytes']);assert oid('blob',cargo)==tree['Cargo.lock'] and sha(cargo)==cargo_record['sha256']
packages=tomllib.loads(cargo.decode())['package'];bevy=next(x for x in packages if x['name']=='bevy_math' and x['version']=='0.19.1');glam=next(x for x in packages if x['name']=='glam' and x['version']=='0.32.1');assert 'glam 0.32.1' in bevy['dependencies'] and glam['checksum']==manifest['sha256']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(656,[1965,1966]),(657,[1967,1968,1969,1970])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
z=json.loads(records['adler/READ-L11-301.json']);check=json.loads(records['adler/VERIFY-L11-301.json']);assert [r['original_index'] for r in z['sources']]==list(range(1965,1971)) and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup['adler/READ-L11-301.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-301.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
older=load('ASSESSMENT-TRANSITION-RECTANGULAR-VISTA-95E.json')['older_root_receipt'];blob=contents(older['preserved_bytes']);assert sha(blob)==older['sha256'] and a['older_root_receipt']['receipt_id']==older['receipt_id'];producer=next(json.loads(x) for x in blob.splitlines() if json.loads(x).get('id')==older['receipt_id']);assert any('normaliz' in t for t in producer['trace'])
q=load('RECONCILIATION-059.json')['entries']
for i in (1552,1608,1610,1611,1612,1613,1614,1615):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings'])==1 and a['conditional_findings'][0]['id']=='REVIEW-LANDFORM-OVERLAP-DIRECTION-001'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert any(x['id']=='REVIEW-LANDFORM-OVERLAP-DIRECTION-001' and x['record']==name and x['validation']=='not performed' for x in ledger)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical bodies, original attributions and conditional overlap arithmetic; no product execution.')
