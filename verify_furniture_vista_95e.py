#!/usr/bin/env python3
"""Verify evidence provenance and connected source provenance, never product behavior."""
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
name='ASSESSMENT-FURNITURE-VISTA-SUPPORT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1920-1931.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1920:50,1921:86,1922:74,1923:68,1924:145,1925:143,1926:24,1927:71,1928:208,1929:29,1930:122,1931:97};ranges={i:([[1,97]] if i==1931 else []) for i in lengths};current={}
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
environment=current[1921];establishments=current[1923];ground=current[1931]
assert b'pub scene_digest: String' in environment and b'#[serde(deny_unknown_fields)]' in environment
assert b'.collect::<BTreeMap<_, _>>()' in establishments and b'operator_ids.insert' in establishments
assert b'self.terrain.height_at(point).or_else(||' in ground and b'vista_triangle_height(' in ground
assert b'self.input.vista.lods.first()?' in ground
vista=sources['crates/adventuresim-tactical-core/src/vista_surface.rs'];assert b'let local = point.metres()' in vista and b'cell_rectangles_outside_inner_rectangle(minimum, maximum, half)' in vista
assert b'if local.x < x0 || local.x > x1 || local.y < z0 || local.y > z1' in vista
oldvista=a['historical_vista_source']
with tarfile.open(root/oldvista['source_archive'],'r:gz') as archive:before=archive.extractfile(oldvista['sha256']).read()
assert before==vista and oldvista['same_as_current'] and sha(before)==oldvista['sha256'] and len(before)==oldvista['bytes']
assert oldvista['git_blob']==oid('blob',before)==tree['crates/adventuresim-tactical-core/src/vista_surface.rs']
check_chain('crates/adventuresim-tactical-core/src/vista_surface.rs',proof['paths']['crates/adventuresim-tactical-core/src/vista_surface.rs'],before,proof['root_tree'],nodes)
place=sources['crates/adventuresim-tactical-core/src/scene_input/furniture/placement.rs']
assert b'let Some(playable) = support.scope(footprint)' in place and b'support.height_at(point)' in place
assert place.index(b'let Some(instances) = supported_instances')<place.index(b'layout.groups.push(FurnitureGroup')
assert b'.replicate::<SceneEnvironment>()' in sources['crates/adventuresim-tactical-netcode/src/replication.rs']
assert b'UnitBasisPoints::saturating(value)' in sources['crates/adventuresim-tactical-client/src/presentation/procedural.rs']
# The original source-only shifted-centre illustration, using integer geometry.
world=(10,0);origin=(10,0);local=(world[0]-origin[0],world[1]-origin[1]);assert local==(0,0) and abs(world[0])>1
rectangles=[(1,2,0,2),(0,1,1,2)];assert all(not (x0<=local[0]<=x1 and z0<=local[1]<=z1) for x0,x1,z0,z1 in rectangles)
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(641,[1920,1921,1922,1923]),(642,[1924,1925,1926,1927]),(643,[1928,1929,1930,1931])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
p='segovb/READ-CONNECTED-L12-644-1964.json';z=json.loads(records[p]);check=json.loads(records['segovb/VERIFY-CONNECTED-L12-644-1964.json'])
assert z['registration'] in records and z['registration_sha256']==lookup[z['registration']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['new_original_roles']==0
assert z['original_index']==1964 and z['fresh_personal_body_reads']==[[1,480]] and z['source_sha256']==oldvista['sha256']
for n,indices in [(294,list(range(1924,1930))),(295,list(range(1930,1939))),(300,list(range(1961,1965)))]:
    path=f'adler/READ-L11-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert [r['original_index'] for r in z['sources']]==indices and z['current_reconciled'] is False
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z.get('connected_reinspection',[]):
        p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
assert 1964 not in [x['original_index'] for x in h['historical_source_roles']]
original=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']['ADLER-L11-VISTA-PLAYABLE-SEAM-FRAME'];assert 'actualplacementground295' in json.dumps(original).lower()
q=load('RECONCILIATION-059.json')['entries']
for i in (112,1580,1582,1583,1584,1585,1587,1588):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert a['conditional_findings']==[]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];entry=next(x for x in ledger if x['id']=='ADLER-L11-VISTA-PLAYABLE-SEAM-FRAME');assert name in entry['additional_current_evidence']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified twelve historical roles, out-of-order independent evidence and connected existing vista finding; no product execution.')
