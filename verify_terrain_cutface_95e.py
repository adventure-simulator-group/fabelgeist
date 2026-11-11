#!/usr/bin/env python3
"""Verify source provenance and conditional cut-face reasoning only."""
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
        end=body.index(b'\0',pos);mode,name=body[pos:end].split(b' ',1)
        result[name.decode()]=(mode.decode(),body[end+1:end+21].hex());pos=end+21
    assert pos==len(body);return result
def check_chain(path,proof,body,start,nodes):
    assert [x['name'] for x in proof['chain']]==path.split('/')
    current=start
    for part in proof['chain']:
        info=nodes[current];raw=contents(info['preserved_bytes'])
        assert oid('tree',raw)==current==part['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child']);current=part['child']
    assert current==proof['git_blob']==oid('blob',body)
    assert sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e'
name='ASSESSMENT-TERRAIN-CUTFACE-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1816-1822.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1816:141,1817:242,1818:152,1819:420,1820:322,1821:83,1822:392};ranges={1816:[],1817:[],1818:[],1819:[[1,162],[170,222]],1820:[],1821:[[1,83]],1822:[]}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=w['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']==('bounded-current-trace-only' if i==1819 else 'unassessed') and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==ranges[i] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==n
        assert oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes'])
    assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes']
    assert len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
# The prior checkpoint contains the exact current foundation bytes and source ranges.
prior=load('ASSESSMENT-L01-HISTORICAL-1811-1815.json');fitem=next(x for x in prior['historical_source_roles'] if x['original_index']==1814)
foundation=contents(fitem['current_comparison']['preserved_bytes']);assert oid('blob',foundation)==tree[fitem['current_comparison']['path']]
settlement=current[1819];query=current[1820];prepared=current[1821];property_terrain=current[1818]
assert b'pub fn from_prisms(' in foundation and b'if self.cut_faces.iter().flatten().any(|p| !p.is_finite())' in foundation
assert b'if self.support_triangles.len() != cells' in foundation and b'Ok(SupportCollision::compound(parts))' in foundation
assert b'pub fn from_foundations(' in settlement and b'surface.validate_source_geometry()?;' in settlement
assert b'surface.query = SupportQueryIndex::compile(&surface);' in settlement and b'surface.query.validate(&surface)?;' in settlement
assert b'if !triangles.is_empty()' in query and b'visited.iter().any(|v| !v)' in query and b'references.len()' in query
assert b'if !self.natural_triangles.is_empty()' in settlement and b'.chain(self.foundations.iter().flat_map(|f| f.cut_faces.iter()))' in settlement
assert b'.flat_map(|f| f.cut_faces.iter().copied())' in settlement
assert b'if self.natural_triangles.is_empty()' in property_terrain
assert b'let geometry = GroundTriangle::new(points)?;' in prepared and b'let Some(triangle) = prepared.triangle(reference) else' in query
scene=sources['crates/adventuresim-tactical-core/src/scene.rs'];support=sources['crates/adventuresim-tactical-core/src/scene/property_support.rs'];server=sources['crates/adventuresim-tactical-server/src/terrain_collision.rs']
assert b'TerrainGeometry::Owned(surface) => surface.colliders()' in scene
assert b'pub fn with_property_surface(mut self, surface: BoundedSettlementTerrain)' in support
assert b'let Some(recipe) = recipe else {\n        return Ok(terrain.colliders()?);' in server
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(595,[1816,1817,1818]),(596,[1819,1820]),(597,[1821,1822])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']]==indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256']==lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
reuse=json.loads(records['segovb/READ-L12-597.json'])['qualified_supporting_same_lens_reuse'][0]
assert reuse['original_index']==1820 and reuse['fresh_personal_body_reads_this_batch']==[] and reuse['receipt_sha256']==lookup[reuse['receipt']]['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1499,1502,1503,1504,1505,1506,1507,1508,1509,1510,1511):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-FOUNDATION-VOLUME-95E.json' in q[1499]['current_reconciliation']
finding=a['conditional_findings'][0];assert finding['id']=='Q-L12-2675d00ebe7ed868d3ad' and finding['accepted_historical_unique_count_change']==0
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==finding['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified seven historical roles, scoped receipts and conditional cut-face chain; no product execution.')
