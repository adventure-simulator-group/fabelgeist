#!/usr/bin/env python3
"""Verify recovery provenance and symbolic source evidence; never run product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';older='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
name='ASSESSMENT-FOUNDATION-VOLUME-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1811-1815.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1811:209,1812:272,1813:208,1814:390,1815:146};ranges={1811:[[1,209]],1812:[[1,230],[238,245]],1813:[],1814:[],1815:[]}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=w['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]]
        assert len(before.splitlines())==n and sha(before)==row['historical_sha256']
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
native={}
for item in a['native_archive_members']:
    raw=(root/item['archive']).read_bytes();assert sha(raw)==item['archive_sha256']
    with tarfile.open(root/item['archive'],'r:gz') as archive:body=archive.extractfile(item['member']).read()
    assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['personal_ranges'])
    assert item['archive_sha256'].encode() in sources['Cargo.lock'];native[item['member']]=body
vec=native['glam-0.32.1/src/f32/vec3.rs'];bevy=native['bevy_math-0.19.1/src/lib.rs'];cfg=native['bevy_math-0.19.1/Cargo.toml']
assert b'iter.fold(Self::ZERO, Self::add)' in vec and b'y: self.y.add(rhs.y)' in vec
assert b'pub use glam::*;' in bevy and b'[dependencies.glam]\nversion = "0.32.0"' in cfg
assert b'"glam 0.32.1"' in sources['Cargo.lock']
foundation=current[1814];admission=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/admission.rs']
assert b'f64::from(cell[i].y - cell[i + 3].y)' in foundation and b'super::admission::prism(&points)?;' in foundation
assert b'points.iter().copied().sum::<Vec3>().is_finite()' in admission and b'points[i].xz() != points[i + 3].xz()' in admission
assert b'points[i].y < points[i + 3].y' in admission and b'points[i].y == points[i + 3].y' in admission
assert b'PropertyMembers::validate(&self.member_building_ids)?;' in foundation
assert b'if self.support_triangles.len() != cells' in foundation
assert b'if parts.is_empty() {\n            Self::ContactOnly' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/collider.rs']
assert b'BoundarySupportConstraint::MissingGateDatum' in current[1812]
# Exact integer/rational reasoning only; this evaluates no f32 operation or product routine.
max32=(2**24-1)*2**104; accum=0
for y in (max32,0,0,-max32,0,0):
    accum+=y;assert abs(accum)<=max32
assert accum==0 and 2*max32>max32
volume=Fraction(1,2)*Fraction(2*max32,3)
assert volume==Fraction(max32,3) and volume<2**1023
oldproof=load(a['older_locator_evidence']);oldmanifest=load(oldproof['shared_objects'])
assert oldproof['revision']==oldmanifest['commit']==older
oldnodes={z['git_object']:z for z in oldmanifest['objects'] if z['type']=='tree'}
for path in oldproof['additional_shared_objects']:oldnodes.update(load(path)['additional_tree_objects'])
oldnodes.update(oldproof['additional_tree_objects']);oldcommit=next(x for x in oldmanifest['objects'] if x['type']=='commit');raw=contents(oldcommit['preserved_bytes'])
assert oid('commit',raw)==older and raw.splitlines()[0]==b'tree '+oldproof['root_tree'].encode()
oldfiles={}
for item in oldproof['paths']:
    body=contents(item['preserved_bytes']);check_chain(item['path'],item['path_proof'],body,oldproof['root_tree'],oldnodes)
    assert sha(body)==item['exported_sha256'];oldfiles[item['label']]=body
manifest=json.loads(contents(oldproof['evidence_manifest']))
for item in oldproof['paths']:
    path=item['path'].split('adler-original/',1)[1];entry=next(x for x in manifest['audit_files'] if x['path']==path)
    assert entry['original_sha256']==item['original_sha256_as_manifested'] and entry['exported_sha256']==item['exported_sha256'] and entry['bytes']==len(oldfiles[item['label']])
assert sha(oldfiles['root-traces'])=='6cfd484d22fe2d5972a1de6a3a2f0ca7b54c89b2b459ccba7efd61c0193ae583'
assert sha(oldfiles['registration-export'])=='620a2b12f1a2138b2d87600fdde8c069520d97f50042e87c37dca0f305bf00cc'
assert oldproof['paths'][1]['original_sha256_as_manifested']=='42db551c1765fc0c264eb972d5c17b6f69671f476ee3aab27cc97a0e5bac76f4'
locator=json.loads(contents(oldproof['locator_export']))['new_variant_identity_and_range_map'][106]
assert locator['classification']=='genuinely-new-reached-source-variant' and locator['material_owner_fit_status']=='pending; no coverage awarded'
assert locator['recorded_sha256']==w['roles'][1811]['historical_sha256'] and locator['resolved_candidate']['actual_line_count']==209
assert all(x['range_metadata']['status']=='qualified-endpoints-only' and x['source_context']['read_ranges']==[[1,199]] for x in locator['observations'])
rr=oldfiles['root-traces'].splitlines();sr=oldfiles['registration-export'].splitlines();nr=contents(oldproof['native_trace']).splitlines()
assert len(rr)==54 and len(sr)==52 and sha(rr[36])==oldproof['root_trace_line_sha256'] and sha(sr[21])==oldproof['registration_export_line_sha256']
assert sha(nr[14])==oldproof['native_trace_line_sha256']
rootrow=json.loads(rr[36]);regrow=json.loads(sr[21]);nativerow=json.loads(nr[14])
assert rootrow['id']=='ROOT-PPS-TERRAIN-DISPATCH-WEB-037' and rootrow['source_proofs'][12]['read_ranges']==[[1,199]]
assert nativerow['id']=='NATIVE-PS-ROOT-TERRAIN-SOURCE-PACKET-015' and nativerow['related_current_reads'][12]['read_ranges']==[[1,199]]
assert regrow['id']=='ROOT-ADM-REG-022' and regrow['material_ranges']==[[200,209]] and regrow['status']=='prospectively_registered_before_body_analysis'
assert oldproof['bounded_original_search']['matching_basename_files_hashed']==20 and oldproof['bounded_original_search']['exact_hash_matches']==0
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
qual=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1811-LOCATOR.json'])
assert qual['historical_locator_proof_body_not_read'] is True and qual['inherited_requested_ranges']==[[1,199],[200,209]]
assert regrow==qual['additional_material_role_observations'][0]['original_record']
for n,indices in [(592,[1811]),(593,[1812,1813]),(594,[1814,1815])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']]==indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256']==lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
reuse=json.loads(records['segovb/READ-L12-593.json'])['qualified_supporting_same_lens_reuse'][0]
assert reuse['original_index']==1811 and reuse['fresh_personal_body_reads_this_batch']==[] and reuse['receipt_sha256']==lookup[reuse['receipt']]['sha256']
for n in (270,271,280):
    path=f'adler/READ-L11-{n}.json';check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (102,1491,1492,1493,1494,1495,1496,1497,1498,1499,1500,1501):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for i in (1491,1492,1493,1494):assert 'ASSESSMENT-GROUNDING-BOUNDARY-95E.json' in q[i]['current_reconciliation']
findings=a['conditional_findings'];assert {x['id'] for x in findings}=={'ADLER-L11-FOUNDATION-VOLUME-ARITHMETIC','Q-L12-f9b80782d86542a3c6de'}
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for f in findings:
    assert f['accepted_historical_unique_count_change']==0 and sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical roles, older locator proof and symbolic volume relation; no product execution.')
