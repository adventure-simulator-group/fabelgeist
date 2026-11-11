#!/usr/bin/env python3
"""Verify evidence identities and elementary recorded ratios; no product execution."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-TRANSITION-RECTANGULAR-VISTA-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1959-1964.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1959:20,1960:64,1961:82,1962:200,1963:41,1964:480};ranges={1959:[],1960:[[1,64]],1961:[],1962:[],1963:[],1964:[]};current={}
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
urban=current[1959];validation=current[1960];transition=current[1961];collar=current[1962];vista=current[1964]
assert b'buildings::validate_building_placements(&input.buildings)' in urban and b'buildings::validate_distant_building_placements(&input.distant_buildings)' in urban
assert b'validate_grid(&self.playable' in validation and b'validate_grid(&grid' in validation and b'lod.spacing_metres <= previous_spacing' in validation
assert b'.as_chunks::<3>()' in transition and b'mesh_components_with_stride_filtered' in transition
assert b'indices.extend_from_slice(&triangle)' in sources['crates/adventuresim-tactical-core/src/scene.rs'] and b'indices.extend([start, start + 1, start + 2])' in sources['crates/adventuresim-tactical-core/src/scene/property_support.rs']
assert b'self.half_width_metres <= self.width_metres' in collar and b'self.width_variation_bps > 5_000' in collar
recipe=sources['crates/adventuresim-tactical-core/src/volumetric_terrain/recipe.rs'];volume=sources['crates/adventuresim-tactical-core/src/volumetric_terrain.rs'];assert b'u32::from(self.collar_cm) * 2 >= u32::from(self.half_width_cm)' in recipe and b'SCARP_WIDTH_VARIATION_BPS: u16 = 1_200' in volume
assert Fraction(10000-1200,10000)>Fraction(1,2)
assert b'f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres * 0.5' in vista and b'(world - center).abs().max_element()' in vista
assert b'width < 2 || depth < 2 || width > max_side || depth > max_side' in sources['crates/adventuresim-tactical-core/src/scene_input.rs']
presented=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/source/presented.rs'];assert b'half_extent.cmple(inner_half_extent).any()' in presented and b'f32::from(lod.depth.checked_sub(1)?)' in presented
natural=sources['crates/adventuresim-tactical-client/src/presentation/vista/natural.rs'];assert b'cell_rectangles_outside_inner_rectangle' in natural and b'VistaVertexGeometry::sample' in natural
prepared=sources['crates/adventuresim-tactical-client/src/presentation/vista/prepared.rs'];assert b'f32::from(lod.depth.saturating_sub(1)) * lod.spacing_metres * 0.5' in prepared
assert b'GridDimensions::square(spec.side)' in sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs']
# Exact arithmetic substantiates only the recorded symbolic example, not a runtime result.
play_half=Fraction(1,4);width_half=Fraction(9-1,2);depth_half=Fraction(5-1,2);coarse_half=Fraction((9-1)*2,2);transition_width=max(min(Fraction(2),width_half),Fraction(1));radius=depth_half
weight=(radius-(width_half-transition_width))/transition_width
assert (play_half,width_half,depth_half,coarse_half,transition_width,weight)==(Fraction(1,4),4,2,8,2,0)
assert depth_half-play_half>1 and Fraction(0)*(1-weight)+Fraction(10)*weight==0 and Fraction(10)-0==10
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(653,[1959,1960]),(654,[1961,1962,1963]),(655,[1964])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        if n!=655:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
        else:
            assert row['fresh_personal_body_reads']==[] and len(row['same_lens_reuse'])==1
            reuse=row['same_lens_reuse'][0];assert reuse['fresh_personal_body_reads_this_batch']==[] and reuse['assessed_ranges']==[[1,480]] and reuse['frozen_git_revision']==old and reuse['receipt'] in records and reuse['registration'] in records
            earlier=json.loads(records[reuse['receipt']]);assert earlier['fresh_personal_body_reads']==[[1,480]] and earlier['source_sha256']==w['roles'][1964]['historical_sha256']
traceinfo=a['older_root_receipt'];blob=contents(traceinfo['preserved_bytes']);assert sha(blob)==traceinfo['sha256'];producer=next(json.loads(x) for x in blob.splitlines() if json.loads(x).get('id')==traceinfo['receipt_id'])
assert load(traceinfo['git_membership_evidence'])['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
for j,i in enumerate((1962,1963)):
    path=f'segovb/QUALIFICATION-L12-ORIGINAL{i}-LOCATOR.json';z=json.loads(records[path]);assert z['canonical_git_revision']==old and z['body_coverage']==0 and z['historical_locator_proof_body_not_read'] and z['exact_source_sha256']==w['roles'][i]['historical_sha256'] and z['raw_producer_observations'][0]['source_context']==producer['source_proofs'][j]
z=json.loads(records['adler/READ-L11-300.json']);check=json.loads(records['adler/VERIFY-L11-300.json']);assert [r['original_index'] for r in z['sources']]==[1961,1962,1963,1964] and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup['adler/READ-L11-300.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-300.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
prior=load('ASSESSMENT-RECIPE-ROAD-95E.json');rec=next(x for x in prior['recovered_historical_records'] if x['original_path']=='adler/READ-L11-299.json');assert sha(contents(rec['preserved_bytes']))==rec['published_sha256'] and rec['sha256']==lookup[rec['original_path']]['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1512,1588,1604,1605,1606,1607,1608,1615):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings'])==1 and a['conditional_findings'][0]['id']=='REVIEW-RECTANGULAR-VISTA-LOD-MORPH-001'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert any(x['id']=='REVIEW-RECTANGULAR-VISTA-LOD-MORPH-001' and x['record']==name and x['validation']=='not performed' for x in ledger)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical bodies, preserved attributions and symbolic rectangular-ring arithmetic; no product execution.')
