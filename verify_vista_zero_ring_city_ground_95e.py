#!/usr/bin/env python3
"""Verify original vista roles, attribution, and zero-ring source branching."""
import gzip
import hashlib
import json
import tarfile
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
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-VISTA-ZERO-RING-CITY-GROUND-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1652-1656.json');w=load('COVERAGE-L01-ROLES.json');proof=load('VISTA-ZERO-RING-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1652,1657))
lengths={1652:350,1653:83,1654:192,1655:236,1656:62};ranges={1652:[[1,180],[181,350]],1653:[[1,83]],1654:[[1,192]],1655:[[1,125],[126,236]],1656:[[1,62]]};current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];b=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges[i] and len(old.splitlines())==lengths[i] and sha(old)==r['historical_sha256']
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1652-1656.json'
        assert sha(b)==c['sha256'] and len(b)==c['bytes'] and len(b.splitlines())==lengths[i] and c['current_personal_ranges']==ranges[i] and oid('blob',b)==c['git_blob']==tree[c['path']] and c['byte_identical'] and old==b
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);current[i]=b
    for e in a['earlier_historical_source_identities']:
        i=e['original_index'];r=w['roles'][i];body=ar.extractfile(e['sha256']).read();path=e['source_id'].split(':',1)[1]
        assert i in (1657,1658,1659) and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0 and sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']]==[1657,1658,1659]
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
for n,indices in [(524,[1652,1653,1654]),(525,[1655,1656,1657,1658])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
    for r in z.get('qualified_supporting_same_lens_reuse',[]):assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256'] and r['receipt'] in records and r['receipt_sha256']==lookup[r['receipt']]['sha256'] and r['fresh_personal_body_reads_this_batch']==[]
p='adler/READ-L11-238.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-238.json'])
assert [r['original_index'] for r in z['sources']]==list(range(1654,1660)) and z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-238.json']['sha256']
for r in z['sources']+z['connected_reinspection']:
    if r.get('registration_sha256'):
        reg='adler/'+r['registration'].replace('\\','/').split('/adler/')[-1];assert reg in records and r['registration_sha256']==lookup[reg]['sha256']
for r in z.get('same_lens_context',[]):
    rp='adler/'+r['receipt'].replace('\\','/').split('/adler/')[-1];assert rp in records and r['sha256']==lookup[rp]['sha256']
assert any('nonzero east125' in x for x in z['pending_contracts'])
prev=load('ASSESSMENT-OWNED-GROUND-SUPPORT-95E.json');assert [e['original_index'] for e in prev['earlier_historical_source_identities']]==[1652,1653] and all(e['additional_original_frontier_credit']==0 for e in prev['earlier_historical_source_identities'])
prefix='crates/adventuresim-tactical-client/src/'
vista=sources[prefix+'presentation/vista.rs'];surface=sources[prefix+'presentation/vista/surface.rs'];streets=sources[prefix+'presentation/vista/streets.rs'];prepared=sources[prefix+'presentation/vista/streets/prepared.rs'];focused=sources[prefix+'presentation/vista/focused.rs'];cfg=sources[prefix+'presentation/config.rs'];sky=sources[prefix+'tactical_sky_viewer.rs'];shader=sources['assets/shaders/tactical_vista.wgsl'];validation=sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs'];net=sources['crates/adventuresim-tactical-netcode/src/replication.rs']
assert b'.take(settings.config.rendering.vista.maximum_lods)' in vista and b'if let Some(lod) = visible_lods.first().copied()' in vista and b'city_ground.spawn(' in vista and b'commands.entity(entity).despawn()' in vista
assert b'for (source_index, lod) in lods.iter().copied().enumerate()' in current[1656] and b'Ok(prepared)' in current[1656]
assert b'self.urban = !bundle.streets.is_empty() || !bundle.yards.is_empty()' in surface and b'if self.urban' in surface and b'UrbanGround' not in current[1656]
assert b'VistaTerrain(adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0))' in streets and b'PreparedCityGround::new(' in streets and b'support.add_owned_region(' in prepared and b'Self::new(&input.streets, &input.yards, groups, &support)' in prepared and b'paving.spawn_focused(' in focused
assert b'plugin.config.rendering.vista.maximum_lods = 0' in sky and b'maximum_lods: 3' in sources['assets/config/tactical-graphics.yaml'] and b'self.rendering.vista.maximum_lods = 1' in cfg
assert b'self.vista.lods.len() > MAX_VISTA_LEVELS' in validation and b'.add_server_event::<SceneVistaBundle>(Channel::Ordered)' in net
assert b'origin_east_metres: 125.0' in current[1655] and b'origin_north_metres: -275.0' in current[1655] and b'stitched.w = vista_color.w' in current[1654] and b'let sward_coverage = clamp(pbr_input.material.base_color.a' in shader
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1374,1379))
f,=a['conditional_findings'];assert f['id']=='REVIEW-VISTA-ZERO-RING-CITY-GROUND-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries'];assert sum(r['id']==f['id'] and r['record']=='ASSESSMENT-VISTA-ZERO-RING-CITY-GROUND-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified five historical roles, reviewer reuse and conditional zero-ring source path; no product execution.')
