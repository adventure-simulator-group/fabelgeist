#!/usr/bin/env python3
"""Verify evidence provenance and exact dyadic geometry, never product execution, never product behavior."""
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
name='ASSESSMENT-ROUNDED-POLYGON-TOPOLOGY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1906-1909.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1906:57,1907:122,1908:191,1909:270};ranges={1906:[],1907:[[1,122]],1908:[],1909:[[1,200]]};current={}
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
point=current[1907];polygon=current[1908];datum=current[1909]
assert b'Self::from_metres(Vec2::deserialize(deserializer)?)' in point and b'Self(Vec2::new(point.x, point.z))' in point
assert b'let polygon = PlanPolygon::from_ordered_vertices(vertices)?' in polygon
assert b'Self::from_rigid_projection(vertices)' in polygon and b'area <= 0.0' in polygon and b'vertices[..vertex].contains(point)' in polygon
assert b'Position::from_metres(' in datum and b'pub struct SceneDoorPose' in datum
p=sources['crates/adventuresim-building-generator/src/plan_geometry.rs'];assert b'cross(a, b, point) < 0.0' in p and b'signed_area(&vertices)' in p
p=sources['crates/adventuresim-building-generator/src/spatial_geometry/coordinates.rs'];assert b'finite(metres, GeometryRole::Position)?' in p and b'Self::from_metres(Vec3::deserialize(deserializer)?)' in p
assert b'metres: self.min.metres().min(other.min.metres())' in sources['crates/adventuresim-building-generator/src/spatial_geometry/bounds.rs']
floor=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/surface/floor/region.rs'];assert b'ScenePlanPolygon::from_ordered_vertices(vertices.clone())?' in floor and b'ScenePlanPolygon::from_architectural(' in floor
assert b'FloorRegion::from_architectural(' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/single_support/bearing.rs']
assert b'.convex_floor(self.bearing_region.outline(), elevation)?' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/single.rs']
assert b'(1..outline.len() - 1).map(|i| [start, start + i as u32 + 1, start + i as u32])' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/mesh.rs']
assert b'rounded_architectural_edges_keep_union_coverage_without_masking_missing_faces' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/surface/floor/tests.rs']
# Rational orientation and nearest-integer bounds only, without executing geometry code.
def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
a0=(Fraction(0),Fraction(0));b0=(Fraction(4),Fraction(0));c0=(Fraction(4),Fraction(2));d0=(Fraction(161,64),Fraction(81,64));points=[a0,b0,c0,d0]
assert [cross(points[i],points[(i+1)%4],points[(i+2)%4]) for i in range(4)]==[8,Fraction(95,32),Fraction(1,32),Fraction(81,16)]
assert Fraction(5,2)<d0[0]<Fraction(7,2) and Fraction(1,2)<d0[1]<Fraction(3,2)
d1=(Fraction(3),Fraction(1));after=[a0,b0,c0,d1];assert cross(c0,d1,a0)==-2
assert sum(a[0]*b[1]-a[1]*b[0] for a,b in zip(after,after[1:]+after[:1]))==6
assert len(set(after))==4 and 0<d1[1]<d1[0]/2 and d1[0]<4
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(634,[1907]),(635,[1908,1909])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
qual=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1907-EOF-001.json']);registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'))
assert qual['actual_EOF']==122 and qual['body_coverage_before_read']==0
for i,x in zip((1,2),qual['exact_metadata_range_qualifications']):
    assert x['relative_json_pointer']==f'/owner_contract_proofs/{i}/read_ranges' and registry['files'][1907]['owner_contract_proofs'][i]['read_ranges']==x['raw_requested_ranges']==[[1,155]]
    assert x['effective_existing_requested_body']==[[1,122]] and x['nonexistent_tail']==[[123,155]] and x['nonexistent_tail_coverage']==0
assert h['historical_source_roles'][1]['historical_scope_qualification']['nonexistent_tail_coverage']==0
z=json.loads(records['adler/READ-L11-291.json']);check=json.loads(records['adler/VERIFY-L11-291.json']);assert [r['original_index'] for r in z['sources']]==list(range(1906,1914)) and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup['adler/READ-L11-291.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-291.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
assert 'adler/REGISTRATION-ADLER-748.json' in records
q=load('RECONCILIATION-059.json')['entries']
for i in (1573,1574,1575,1576):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='REVIEW-ROUNDED-POLYGON-TOPOLOGY-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, exact EOF qualifications, and dyadic orientation reasoning; no product execution.')
