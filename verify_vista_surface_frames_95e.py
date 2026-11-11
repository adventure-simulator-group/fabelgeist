#!/usr/bin/env python3
"""Verify vista source identity, historical attribution and symbolic frame evidence."""
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
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-VISTA-SURFACE-FRAMES-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1642-1646.json');w=load('COVERAGE-L01-ROLES.json');proof=load('VISTA-SURFACE-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1642,1647))
lengths={1642:119,1643:130,1644:300,1645:163,1646:149};ranges={1642:[[1,119]],1643:[[1,130]],1644:[[1,150],[151,300]],1645:[[1,163]],1646:[[1,149]]};current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];b=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges[i] and len(old.splitlines())==lengths[i] and sha(old)==r['historical_sha256']
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1642-1646.json'
        assert sha(b)==c['sha256'] and len(b)==c['bytes'] and len(b.splitlines())==lengths[i] and c['current_personal_ranges']==ranges[i] and oid('blob',b)==c['git_blob']==tree[c['path']] and c['byte_identical'] and old==b
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);current[i]=b
    for e in a['earlier_historical_source_identities']:
        i=e['original_index'];r=w['roles'][i];body=ar.extractfile(e['sha256']).read();path=e['source_id'].split(':',1)[1]
        assert i in (1647,1648) and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0 and sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']]==[1647,1648]
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
for n,indices in [(521,[1643,1644,1645]),(522,[1646,1647,1648])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
p='adler/READ-L11-236.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-236.json'])
assert [r['original_index'] for r in z['sources']]==list(range(1642,1648)) and z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-236.json']['sha256']
for r in z['sources']+z['connected_reinspection']:
    if r.get('registration_sha256'):
        reg='adler/'+r['registration'].replace('\\','/').split('/adler/')[-1];assert reg in records and r['registration_sha256']==lookup[reg]['sha256']
for r in z.get('same_lens_context',[]):
    rp='adler/'+r['receipt'].replace('\\','/').split('/adler/')[-1];assert rp in records and r['sha256']==lookup[rp]['sha256']
assert any('owned centred clipping' in x for x in z['pending_contracts'])
prev=load('ASSESSMENT-VISTA-CARD-BATCHING-95E.json');e,=prev['earlier_historical_source_identities'];assert e['original_index']==1642 and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0
prefix='crates/adventuresim-tactical-client/src/presentation/vista/'
details=sources[prefix+'details.rs'];clip=sources[prefix+'owned/clip.rs'];exterior=sources[prefix+'owned/exterior.rs'];surface=sources[prefix+'surface.rs'];core=sources['crates/adventuresim-tactical-core/src/vista_surface.rs'];settlement=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/settlement.rs'];dispatcher=sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs']
assert b'Native scene-metre triangles' in settlement and b'Position<' in settlement and b'natural_triangles' in settlement
assert b'cell_rectangles_outside_inner_rectangle(minimum, maximum, inner)' in current[1646] and b'GroundPresentation::in_rectangles(surface, &regions)' in current[1646] and b'.map(|p| p.to_array())' in current[1646]
assert b'regions.overlaps_bounds(low, high)' in exterior and b'ClippedTriangles::Interior(Some(triangle))' in clip and b'Transform::from_translation(origin)' in surface
assert b'presented_height_at(lod, world, coarser)' in current[1643] and b'Some(playable_terrain)' in current[1643] and b'VistaVertexGeometry::sample(' in current[1645]
assert b'native_stitch_vista_height_to_playable_edge' in core and b'outside_distance / transition_width' in core and b'local.x.abs() <= playable_half_extent.x + 7.0' in details
assert b'VistaLodSpec::new(0, 50.0, 41)' in dispatcher and b'origin_east_metres: 0.0' in dispatcher
# Symbolic arithmetic of stated source conditions only; no product execution.
assert 20>=1 and 21<=250 and 20>=0 and 21<=250 and 20+100==120 and 21+100==121
assert 9>1+7 and Fraction(1,2)<1
edge=Fraction(100)*Fraction(50-1,50);height=edge*Fraction(9-1,50-1)
assert edge==98 and height==16 and Fraction(100)-height==84
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in list(range(1364,1369))+[1604,1609])
expected=['REVIEW-VISTA-OWNED-SURFACE-ORIGIN-FRAME-001','REVIEW-VISTA-TREE-SAMPLED-SEAM-HEIGHT-001'];assert [x['id'] for x in a['conditional_findings']]==expected
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries']
for x in a['conditional_findings']:
    assert x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed'
    assert sum(r['id']==x['id'] and r['record']=='ASSESSMENT-VISTA-SURFACE-FRAMES-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified five historical roles, original reviewer records and symbolic vista frame/seam evidence; no product execution.')
