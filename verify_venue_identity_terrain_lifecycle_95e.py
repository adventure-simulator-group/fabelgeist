#!/usr/bin/env python3
"""Verify source identities, prior attribution and bounded native evidence only."""
import difflib
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
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-VENUE-IDENTITY-TERRAIN-LIFECYCLE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1632-1635.json');w=load('COVERAGE-L01-ROLES.json');proof=load('VENUE-TERRAIN-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1632,1633,1634,1635]
lengths={1632:336,1633:259,1634:338,1635:100};clengths={1632:330,1633:253,1634:336,1635:100};ranges={1632:[[1,180],[181,336]],1633:[[1,140],[141,259]],1634:[[1,180],[181,338]],1635:[[1,100]]};cranges={1632:[[1,180],[181,330]],1633:[[1,130],[131,253]],1634:[[1,175],[176,336]],1635:[[1,100]]};current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];body=contents(c['preserved_bytes'])
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges[i] and len(old.splitlines())==lengths[i]
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1632-1635.json' and sha(old)==r['historical_sha256']
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==clengths[i] and c['current_personal_ranges']==cranges[i]
        assert oid('blob',body)==c['git_blob']==tree[c['path']] and c['byte_identical']==(i==1635)
        if i!=1635:
            if i in (1632,1634):assert old.split(b'#[cfg(test)]\nmod tests {',1)[0]==body.split(b'#[cfg(test)]\nmod tests {',1)[0]
            diff=''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),body.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode()
            assert diff==contents(c['exact_diff']['preserved_bytes']) and sha(diff)==c['exact_diff']['sha256']
        else:assert old==body
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);current[i]=body
sources={}
for s in a['current_supporting_sources']:
    body=contents(s['preserved_bytes']);assert s['revision']==revision and sha(body)==s['sha256'] and len(body)==s['bytes'] and len(body.splitlines())==s['lines']
    assert oid('blob',body)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=body
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']}
def prior(assessment,path):
    record=next(x for x in load(assessment)['recovered_historical_records'] if x['original_path']==path);body=contents(record['preserved_bytes']);original=lookup[path]
    assert record['sha256']==original['sha256'] and record['git_blob']==original['git_blob'] and sha(body)==record['published_sha256']
    return json.loads(body)
adler=prior('ASSESSMENT-TERRAIN-LANDFORM-95E.json','adler/READ-L11-234.json')
assert [x['original_index'] for x in adler['sources']]==[1632,1633,1634,1635] and adler['accepted_unique_count_change']==0 and adler['current_reconciled'] is False
seg516=prior('ASSESSMENT-TERRAIN-CONTACT-ENCODING-95E.json','segovb/READ-L12-516.json');seg517=prior('ASSESSMENT-TERRAIN-LANDFORM-95E.json','segovb/READ-L12-517.json')
assert [x['original_index'] for x in seg516['rows']]==[1630,1631,1632] and [x['original_index'] for x in seg517['rows']]==[1633,1634,1635]
prior82=prior('ASSESSMENT-TERRAIN-LANDFORM-95E.json','root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-082.json')['rows'][4]
assert prior82['fresh_personal_body_ranges']==[[1,217]] and prior82['actual_EOF']==336
native=load(a['native_source_evidence']['manifest']);archive_path=root/'native-evidence/bevy-replicon-0.41.1-archive/bevy_replicon-0.41.1.crate';assert sha(archive_path.read_bytes())==native['sha256']=='a25105650eb1b27c6ada888e06bb38ff7e326a94c7c5b4d75fb5aee50c5ae45e'
assert native['sha256'].encode() in contents(native['current_lock_evidence']);members={}
with tarfile.open(archive_path,'r:gz') as ar:
    for ref in a['native_source_evidence']['members']:
        body=ar.extractfile(ref['member']).read();assert sha(body)==ref['sha256'] and len(body)==ref['bytes']
        entry=next(x for x in native['inspected_members'] if x['path']==ref['member']);assert entry['sha256']==ref['sha256']
        assert native['personally_inspected_ranges'][ref['member']]==ref['personally_inspected_ranges'];members[ref['member']]=body
assert b'keep entity updates atomic' in members['bevy_replicon-0.41.1/src/server/replication_messages/updates.rs']
assert b'Message splits only happen per-entity' in members['bevy_replicon-0.41.1/src/server/replication_messages/mutations.rs']
s=a['scoped_source_search'];body=contents(s['preserved_bytes']);assert sha(body)==s['sha256'] and len(body)==s['bytes'] and b'PendingTerrainPresentation' in body and b'TerrainLandformRecipe' in body
venue=current[1635];selection=sources['crates/adventuresim-tactical-client/src/strategic_scene/buildings.rs'];worker=sources['crates/adventuresim-tactical-client/src/presentation/generation/venue.rs'];street=sources['crates/adventuresim-tactical-client/src/strategic_scene/street.rs'];js=sources['crates/strategic-web/static/strategic-scene.js'];layout=sources['crates/strategic-web/src/templates/layout.rs'];org=sources['crates/adventuresim-core/src/organization.rs']
assert b'PlaceKind::Guild => BuildingUse::Guildhall' in venue and b'.find(|e| operators.contains(&e.operator_character_id.0))' in venue and b'.find(|b| b.program.usage == Some(usage))' in venue
assert b'select_building(input, place.kind, operators)' in worker and b'.filter(|p| p.place == place.id)' in worker
assert b'selected.insert(place.id.clone(), id)' in selection and b'for place in places' in street and b'moves.insert(' in street and b'building.placement.id,' in street
assert b'.before\n                .compute_affine()' in street and b'&mut generated.furniture, &moves' in street
assert b'kind: kinds[link.dataset.serviceId] || "guild"' in js and b'data-service-id="organization"' in layout and b'data-building-id=(&chapter.location_id)' in layout
assert b'_ => false' in org and b'chapter_effective_location_id(organization, chapter, profile) == chapter.location_id.as_str()' in org
catalog=json.loads(sources['content/organizations/catalog.yaml']);chapters={}
for o in catalog:
    if o['id'] in ('physicians_college','surgeons_guild'):
        chapters[o['id']]=(o['service_id'],next(c for c in o['chapters'] if c['settlement_id']=='viabundus-0'))
assert chapters['physicians_college'][0]=='physician' and chapters['physicians_college'][1]['location_id']=='organization-physicians-college' and chapters['physicians_college'][1]['building_kind']=='college'
assert chapters['surgeons_guild'][0]=='surgeon' and chapters['surgeons_guild'][1]['location_id']=='organization-surgeons-guild' and chapters['surgeons_guild'][1]['building_kind']=='guildhall'
terrain=sources['crates/adventuresim-tactical-client/src/presentation/terrain.rs'];module=sources['crates/adventuresim-tactical-client/src/presentation/mod.rs'];ready=sources['crates/adventuresim-tactical-client/src/presentation/scene_readiness.rs'];net=sources['crates/adventuresim-tactical-netcode/src/replication.rs'];server=sources['crates/adventuresim-tactical-server/src/terrain_collision.rs']
assert b'volumetric::refresh_presentation(' in terrain and b'volumetric::spawn_presentation(' in terrain and b'.remove::<PendingTerrainPresentation>()' in terrain
assert b'.add_observer(on_game_scene_added)' in module and b'.add_observer(terrain::on_environment_added)' in module and b'.add_observer(terrain::on_ground_added)' in module
assert b'counts.terrain_presented += usize::from(!terrain_pending)' in ready
assert b'.replicate::<SceneTerrain>()' in net and b'.replicate_once::<TerrainLandformRecipe>()' in net
assert server.index(b'scene.insert(recipe)') < server.index(b'scene.insert(terrain)')
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (86,1348,1354,1355,1356,1357))
finding,=a['conditional_findings'];assert finding['id']=='REVIEW-VENUE-PLACE-BUILDING-IDENTITY-001' and finding['accepted_historical_unique_count_change']==0
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries'];assert sum(x['id']==finding['id'] and x['record']=='ASSESSMENT-VENUE-IDENTITY-TERRAIN-LIFECYCLE-95E.json' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified four historical roles, scoped venue source and pinned replication counterevidence; no product execution.')
