#!/usr/bin/env python3
"""Verify grounding provenance and source-level conditional evidence only."""
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
name='ASSESSMENT-GROUNDING-BOUNDARY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1803-1810.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1803:407,1804:115,1805:92,1806:106,1807:304,1808:171,1809:233,1810:263}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=w['roles'][i];before=archive.extractfile(item['sha256']).read()
        c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']==('bounded-current-trace-only' if i in (1803,1807,1808) else 'unassessed')
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert item['prior_current_observations']==row.get('current_observations',[])
        assert before==body and c['byte_identical'] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==n
        assert oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes'])
    assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes']
    assert len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
native={}
for item in a['native_archive_members']:
    raw=(root/item['archive']).read_bytes();assert sha(raw)==item['archive_sha256']
    with tarfile.open(root/item['archive'],'r:gz') as archive:body=archive.extractfile(item['member']).read()
    assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['personal_ranges'])
    assert item['archive_sha256'].encode() in sources['Cargo.lock'];native[item['member']]=body
macro=native['serde_derive-1.0.229/src/de.rs']
assert b'cont.attrs.type_try_from()' in macro and b'<#type_try_from as _serde::Deserialize>::deserialize(__deserializer)' in macro
assert b'|v| _serde::#private::TryFrom::try_from(v).map_err(_serde::de::Error::custom)' in macro
compound=current[1807]
assert b'delta.y.abs() <= route.half_width_metres() * 2.0' in compound
assert b'route.half_width_metres() <= 0.0' in compound and b'validate_bearings(&plan)?' in compound
access=current[1804];apron=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/single/apron.rs']
assert b'if maximum.is_finite()' in access and b'f32::INFINITY,' in apron
assert b'.flat_map(|polygon| crate::city_layout::grounding::planar::subtract(polygon, region))' in access
cell=current[1810];admission=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/admission.rs']
assert b'#[serde(try_from = "BoundaryCellWire")]' in cell and b'super::admission::prism(&positions.map(Position::metres))?' in cell
assert b'#[serde(try_from = "BoundaryMeshWire")]' in cell and b'wire.binding.validate_cells(&wire.cells)?' in cell
assert b'if area == 0.0 {\n        None' in admission
assert b'let Some(faces) = super::admission::solid_faces(&points, ENCLOSURE_CELL_FACES) else {' in cell
assert b'continue;' in cell
binding=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/enclosure/binding.rs']
assert b'.any(|cell| !self.elements.contains(&cell.element))' in binding
assert b'.any(|element| !cells.iter().any(|cell| cell.element == *element))' in binding
assert b'BoundaryDescriptorDigest::of(boundary) != self.descriptor_digest' in binding
scene=sources['crates/adventuresim-tactical-core/src/scene_input/compounds.rs']
assert b'#[serde(try_from = "SceneBoundaryWire")]' in scene and b'wire.fixed_support.binding().validate_scene(' in scene
collider=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/collider.rs']
assert b'if parts.is_empty() {\n            Self::ContactOnly' in collider
assert b'if let Some(collider) = collider.into_solid()' in sources['crates/adventuresim-tactical-server/src/boundaries.rs']
projection=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/enclosure/projection.rs']
assert b'let discrepancy = (covered - expected).abs();' in projection and b'if discrepancy > permitted' in projection
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(588,[1803]),(589,[1804,1805,1806]),(590,[1807,1808]),(591,[1809,1810])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']]==indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256']==lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
reuse=json.loads(records['segovb/READ-L12-590.json'])['qualified_supporting_same_lens_reuse'][0]
assert reuse['original_index']==1803 and reuse['fresh_personal_body_reads_this_batch']==[] and reuse['receipt_sha256']==lookup[reuse['receipt']]['sha256']
frozen=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-020-027.json'])['rows'][1]
revisit=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-CURRENT-004.json'])['rows'][2]
assert frozen['revision']==old and frozen['fresh_personal_ranges']==[[1,58]]
assert revisit['revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' and revisit['fresh_personal_ranges']==[]
assert revisit['qualified_reuse_ranges']==[[1,58]] and revisit['prior_own_lens_receipt']['sha256']==lookup['root-l3-extension-20261030/READ-ROOT-L3-EXT-020-027.json']['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1474,1477,1482,1483,1484,1485,1486,1487,1488,1489,1490,1491,1492,1493,1494,3125):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for i in (1474,1477):assert 'ASSESSMENT-GATE-AND-SUPPORT-95E.json' in q[i]['current_reconciliation']
finding=a['conditional_findings'][0]
assert finding['id']=='REVIEW-BOUNDARY-WIRE-CONTACT-ONLY-001' and finding['accepted_historical_unique_count_change']==0
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==finding['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified eight historical roles, separate lens receipts and source-only boundary case; no product execution.')
