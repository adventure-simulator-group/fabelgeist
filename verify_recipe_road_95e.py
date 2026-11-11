#!/usr/bin/env python3
"""Verify recovered source identities and recorded bounded traces; never execute product code."""
import gzip,hashlib,json,tarfile
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
def chain(path,steps,start,nodes):
    assert [x['name'] for x in steps]==path.split('/')
    current=start
    for part in steps:
        info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==part['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child']);current=part['child']
    return current
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';base='3a50b708381ad00c116445c2c999a49c3943088b'
name='ASSESSMENT-RECIPE-ROAD-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1949-1957.json';deleted='ASSESSMENT-L01-DELETED-1958.json'
a,h,d,w=load(name),load(hist),load(deleted),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1949:148,1950:112,1951:346,1952:60,1953:32,1954:39,1955:7,1956:109,1957:179};ranges={1949:[[1,148]],1950:[[1,112]],1951:[[1,150]],1952:[],1953:[],1954:[],1955:[],1956:[],1957:[]};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=w['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);membership=proof['paths'][c['path']]
        assert item['prior_role_status']=='unassessed' and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==ranges[i] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256']==membership['sha256'] and len(body)==c['bytes']==membership['bytes'] and oid('blob',body)==c['git_blob']==membership['git_blob']==tree[c['path']]
        assert chain(c['path'],membership['chain'],proof['root_tree'],nodes)==oid('blob',body);current[i]=body
assert d['original_index']==1958 and d['registry_state']=='deleted' and d['snapshot_sha256'] is None and not d['historical_object_available'] and d['base_revision']==base
assert w['roles'][1958]['status']=='deleted-base-trace-open' and w['roles'][1958]['historical_assessment']==deleted
raw=(root/d['preserved_base_bytes']).read_bytes();assert sha(raw)==d['base_sha256']=='d35e66e32b787fd9b115f5aead54a0f66ea2c69a2a150e3a2816293600f7ce60' and len(raw)==d['base_bytes']==1035 and len(raw.splitlines())==d['base_lines']==27 and d['new_personal_base_read_ranges']==[[1,27]]
basecommit=contents(d['commit_proof']['preserved_bytes']);assert oid('commit',basecommit)==base and sha(basecommit)==d['commit_proof']['sha256'] and basecommit.splitlines()[0]==b'tree '+d['base_root_tree'].encode()
path=d['source_id'].split(':',1)[1];assert chain(path,d['base_chain'],d['base_root_tree'],d['base_tree_objects'])==oid('blob',raw)==d['base_git_blob']
absent=proof['absence'];assert absent['path']==path and absent['absent_name']==path.split('/')[-1] and path not in tree
parent=chain('/'.join(path.split('/')[:-1]),absent['chain'],proof['root_tree'],nodes);assert parent==absent['parent_tree'];parentbytes=contents(nodes[parent]['preserved_bytes']);assert oid('tree',parentbytes)==parent and absent['absent_name'] not in entries(parentbytes)
assert d['older_collection_crosscheck']['sha256']==sha(raw) and d['older_collection_crosscheck']['bytes']==len(raw) and 'base' in d['older_collection_crosscheck']['qualification']
assert b'Vec2::new(-43.5, -39.5)' in raw and b'7.023_172_4_f32.to_bits()' in raw
assert b'ScenePlanPoint::from_metres(Vec2::new(-43.5, -39.5))' in current[1953] and b'7.023_172_4_f32.to_bits()' in current[1953] and b'point: ScenePlanPoint' in current[1952]
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
recipe=current[1950];assert b'pub program: BuildingProgram' in recipe and b'pub plan: BuildingPlan' in recipe and b'pub collision: BuildingCollision' in recipe and b'compile_building_collision(&plan)' in recipe and b'pub fn insert(&mut self, recipe: GeneratedBuildingRecipe)' in recipe and b'.position(|recipe| recipe.program == *program)' in recipe
building=sources['crates/adventuresim-tactical-core/src/scene_input/buildings.rs'];assert b'.take(&placement.program)' in building and b'plan: recipe.plan' in building and b'collision: recipe.collision' in building and b'building.collision.bounds.plan_half_extents()' in building
site=sources['crates/adventuresim-tactical-core/src/scene_input/furniture/sites.rs'];assert b'site.program == program' in site and b'FurnitureSiteRecipe::new(&generated.plan, generated.collision.bounds)' in site
worker=sources['crates/adventuresim-tactical-client/src/presentation/generation/worker_products.rs'];assert b'job.require_owner(ticket)?' in worker and b'*program == venue.recipe.program' in worker and b'*program == facade.program' in worker
venue=sources['crates/adventuresim-tactical-client/src/presentation/generation/venue.rs'];assert b'Some(recipe) if recipe.program == program' in venue and b'furnish(&recipe.plan, &recipe.program)' in venue
assert b'scene.building_recipes.insert(venue.recipe.clone())' in sources['crates/adventuresim-tactical-client/src/presentation/generation/scene.rs']
assert b'.validate(&catalog.settlement_id, catalog.population)' in current[1949] and b'SceneSource::SyntheticFixture(value) => value.as_str()' in sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'let process_relief = shape.map_or(0.0' in current[1951]
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(650,[1949,1950,1951]),(651,[1952,1953,1954,1955,1956,1957]),(652,[1958])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        if n!=652:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
        else:assert row['fresh_personal_body_reads']==[[1,27]] and row['current_body_credit']==0 and row['source_sha256']==sha(raw) and row['deletion_qualification'] in records
z=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1958-DELETION.json']);assert z['raw_current_sha256'] is None and z['raw_state']=='deleted' and z['exact_base_sha256']==sha(raw) and z['canonical_base_git_revision']==base and z['frozen_absence_revision']==old
catalog_record=next(x for x in load('ASSESSMENT-EQUIPMENT-PRESENTATION-95E.json')['recovered_historical_records'] if x['original_path']=='segovb/FROZEN774-CATALOG.json');catalog=contents(catalog_record['preserved_bytes']);assert catalog_record['sha256']==z['catalog_sha256']==lookup['segovb/FROZEN774-CATALOG.json']['sha256'] and sha(catalog)==catalog_record['published_sha256'] and catalog_record['redacted_local_path_occurrences']>0 and 'pinned 059 transport' in catalog_record['original_recovery'];assert any(row==z['catalog_row'] for row in json.loads(catalog)['rows'])
for n,indices in [(298,list(range(1949,1958))),(299,[1958,1959,1960])]:
    path=f'adler/READ-L11-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert [r['original_index'] for r in z['sources']]==indices and z['current_reconciled'] is False
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z.get('connected_reinspection',[]):
        p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
    if n==299:assert z['sources'][0]['sha256'] is None and z['sources'][0]['fresh_reads']==[] and z['sources'][0]['base_source']['sha256']==sha(raw)
q=load('RECONCILIATION-059.json')['entries']
for i in (1290,1580,1597,1598,1599,1600,1602,1603):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings'])==1 and a['conditional_findings'][0]['id']=='REVIEW-BUILDING-RECIPE-COHERENCE-001'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert any(x['id']=='REVIEW-BUILDING-RECIPE-COHERENCE-001' and x['record']==name and x['validation']=='not performed' for x in ledger)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified nine historical bodies, one base-only deleted role, original receipts and bounded recipe source evidence; no product execution.')
