#!/usr/bin/env python3
"""Verify recorded source identities, historical attribution and symbolic arithmetic."""
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
def oid(kind,b):
    return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
    out,pos={},0
    while pos<len(b):
        end=b.index(b'\0',pos); mode,name=b[pos:end].split(b' ',1)
        out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
    assert pos==len(b);return out
def chain(path,r,body,root_tree,nodes):
    assert [s['name'] for s in r['chain']]==path.split('/')
    current=root_tree
    for s in r['chain']:
        info=nodes[current];data=contents(info['preserved_bytes'])
        assert oid('tree',data)==current==s['tree'] and sha(data)==info['sha256'] and len(data)==info['bytes']
        assert entries(data)[s['name']]==(s['mode'],s['child']);current=s['child']
    assert current==r['git_blob']==oid('blob',body) and sha(body)==r['sha256'] and len(body)==r['bytes'] and len(body.splitlines())==r['lines']
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-TERRAIN-CONTACT-ENCODING-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1627-1631.json');w=load('COVERAGE-L01-ROLES.json');proof=load('TERRAIN-CONTACT-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1627,1632))
lengths={1627:216,1628:258,1629:102,1630:96,1631:103};current_lengths={1627:219,1628:258,1629:102,1630:96,1631:103};currents={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for x in h['historical_source_roles']:
        i=x['original_index'];role=w['roles'][i];original=archive.extractfile(x['sha256']).read();c=x['current_comparison'];body=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==[[1,lengths[i]]] and len(original.splitlines())==lengths[i]
        assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1627-1631.json' and sha(original)==role['historical_sha256']
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==current_lengths[i]
        assert oid('blob',body)==c['git_blob']==tree[c['path']] and c['current_personal_ranges']==[[1,current_lengths[i]]]
        assert c['byte_identical']==(i!=1627)
        if i==1627:
            assert original.split(b'#[cfg(test)]\nmod tests {',1)[0]==body.split(b'#[cfg(test)]\nmod tests {',1)[0]
            diff=''.join(difflib.unified_diff(original.decode().splitlines(keepends=True),body.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode()
            assert contents(c['exact_diff']['preserved_bytes'])==diff and sha(diff)==c['exact_diff']['sha256']
        else: assert original==body
        chain(c['path'],proof['paths'][c['path']],original,proof['root_tree'],nodes);currents[i]=body
    extra,=a['earlier_historical_source_identities'];r=w['roles'][1632];body=archive.extractfile(extra['sha256']).read();path=extra['source_id'].split(':',1)[1]
    assert extra['original_index']==1632 and extra['new_personal_historical_full_read'] is False and extra['additional_original_frontier_credit']==0
    assert sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
sources={}
for s in a['current_supporting_sources']:
    body=contents(s['preserved_bytes'])
    assert s['revision']==revision and sha(body)==s['sha256'] and len(body)==s['bytes'] and len(body.splitlines())==s['lines']
    assert oid('blob',body)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=body
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];body=contents(r['preserved_bytes'])
    assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==r['published_sha256'] and len(body)==r['published_bytes']
    assert r['redacted_local_path_occurrences'] or (sha(body)==r['sha256'] and oid('blob',body)==r['git_blob']);records[r['original_path']]=body
for n, indices in [(515,[1627,1628,1629]),(516,[1630,1631,1632])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
    assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:
        assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
        assert r['fresh_personal_body_reads']==r['assessed_ranges']
prev=load('ASSESSMENT-TERRAIN-LANDFORM-95E.json');r=next(r for r in prev['recovered_historical_records'] if r['original_path']=='adler/READ-L11-233.json');earlier=json.loads(contents(r['preserved_bytes']))
assert r['sha256']==lookup[r['original_path']]['sha256'] and [x['original_index'] for x in earlier['sources']]==list(range(1626,1632))
scene=sources['crates/adventuresim-tactical-core/src/scene.rs'];query=sources['crates/adventuresim-tactical-core/src/scene/queries.rs'];admit=sources['crates/adventuresim-tactical-core/src/scene/admission.rs'];hit=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/surface_hit.rs'];height=currents[1630]
weather=sources['crates/adventuresim-tactical-client/src/presentation/weather.rs'];weather_shader=sources['assets/shaders/tactical_weather.wgsl'];bark=sources['crates/adventuresim-procedural-materials/src/shaders/tactical_tree_bark.wgsl'];materials=sources['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/materials.rs'];cache=sources['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/presentation/residency.rs']
assert b'pub fn from_heightmap(' in scene and b'width < 2' in admit and b'depth < 2' in admit and b'h.is_finite()' in admit
assert b'if fraction.x + fraction.y <= 1.0' in query and b'x0y0 + (x1y0 - x0y0) * fraction.x + (x0y1 - x0y0) * fraction.y' in query
assert b'SupportElevation::from_metres(elevation_metres)?' in hit and b'Dir3::new(normal).ok()?' in hit
assert b'(bounds.maximum().metres() - minimum).max(0.001)' in height and b'Ok(Self { image, range })' in height and b'(normalized * 65_535.0).round() as u16' in height
assert b'if fraction.x >= fraction.y' in bark and b'+ (x1y1 - x1y0) * fraction.y' in bark
assert b'mix(bark.terrain_surface.z, bark.terrain_surface.w, normalized)' in bark
assert b'return mix(mix(h00, h10, blend.x), mix(h01, h11, blend.x), blend.y)' in weather_shader
assert b'mix(weather_terrain.z, weather_terrain.w, encoded / 65535.0)' in weather_shader
assert b'range.minimum().metres()' in materials and b'range.maximum().metres()' in materials
assert b'TerrainContactHeightmap::from_terrain(terrain)?' in cache and b'TerrainContactHeightmap::from_terrain(terrain)' in weather
# Integer/rational reasoning only: corner values (0,0,0,1), fractions (1/4,1/4).
x=y=Fraction(1,4); cpu=0; tree_height=(1-0)*y; weather_height=x*y
assert (cpu,tree_height,weather_height)==(0,Fraction(1,4),Fraction(1,16))
small=Fraction(1,2048);nominal_denominator=Fraction(1,1000)
assert 0<small<nominal_denominator and Fraction(48,100)<small/nominal_denominator<Fraction(49,100)
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1349,1354))
expected=['REVIEW-TERRAIN-CONTACT-INTERPOLATION-001','REVIEW-TERRAIN-CONTACT-ENCODING-SPAN-001'];assert [x['id'] for x in a['conditional_findings']]==expected
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries']
for x in a['conditional_findings']:
    assert x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed'
    assert sum(r['id']==x['id'] and r['record']=='ASSESSMENT-TERRAIN-CONTACT-ENCODING-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified five historical roles, bounded source, prior attributions and rational interpolation; no product execution.')
