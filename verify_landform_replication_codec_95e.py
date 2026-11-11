#!/usr/bin/env python3
"""Verify pinned source and historic provenance; do not run codec or product."""
import gzip,hashlib,json,tarfile,tomllib
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-LANDFORM-REPLICATION-CODEC-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1971-1972.json'
a,h,w=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1971:490,1972:514};ranges={1971:[[9,104]],1972:[]};current={}
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
surface=current[1971];assert b'#[serde(rename_all = "snake_case", tag = "kind")]' in surface and b'pub enum TerrainGeologicStructure' in surface and b'    Massive,' in surface and b'    Bedded {' in surface and b'    Foliated {' in surface
assert b'#[serde(try_from = "SurfaceWire")]' in surface and b'structure: TerrainGeologicStructure' in surface and b'Self::from_structure(wire.lithology, wire.source, wire.structure)' in surface
prior=load('ASSESSMENT-L01-HISTORICAL-1965-1970.json');item=next(x for x in prior['historical_source_roles'] if x['original_index']==1968);recipe=contents(item['current_comparison']['preserved_bytes']);assert b'#[serde(try_from = "QuantizedLandformRecipe", into = "QuantizedLandformRecipe")]' in recipe and b'pub surface: TerrainSurfaceRecipe' in recipe
registration=sources['crates/adventuresim-tactical-netcode/src/replication.rs'];assert b'.replicate_once::<TerrainLandformRecipe>()' in registration and b'serialize_scene_building' in registration and b'deserialize_scene_building' in registration
server=sources['crates/adventuresim-tactical-server/src/terrain_collision.rs'];assert b'commands.spawn((Replicated, SceneId(scene_id), ground, Transform::default()))' in server and b'if let Some(recipe) = landform' in server and b'scene.insert(recipe)' in server
client=sources['crates/adventuresim-tactical-client/src/presentation/terrain/volumetric.rs'];assert b'Option<&\'static TerrainLandformRecipe>' in client and b'landform.and_then(|recipe| terrain_landform_patch(terrain, *recipe).ok())' in client
cargo_record=next(x for x in load('ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json')['current_supporting_sources'] if x['path']=='Cargo.lock');cargo=contents(cargo_record['preserved_bytes']);assert oid('blob',cargo)==tree['Cargo.lock'] and sha(cargo)==cargo_record['sha256'];packages=tomllib.loads(cargo.decode())['package'];native={}
for item in a['native_source_members']:
    manifest=load(item['manifest']);archive=(root/item['archive']).read_bytes();assert len(archive)==manifest['bytes'] and sha(archive)==manifest['sha256']==item['archive_sha256']
    with tarfile.open(root/item['archive'],'r:gz') as pack:body=pack.extractfile(item['member']).read()
    assert sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['personally_inspected_ranges']);native[item['member']]=body
for pkg,version in [('bevy_replicon','0.41.1'),('postcard','1.1.3'),('serde_derive','1.0.229')]:
    row=next(x for x in packages if x['name']==pkg and x['version']==version);match=next(x for x in a['native_source_members'] if x['package']==pkg+'-'+version);assert row['checksum']==match['archive_sha256']
rules=native['bevy_replicon-0.41.1/src/shared/replication/rules.rs'];fns=native['bevy_replicon-0.41.1/src/shared/replication/registry/rule_fns.rs'];utils=native['bevy_replicon-0.41.1/src/postcard_utils.rs']
assert b'self.replicate_once_filtered::<C, ()>()' in rules and b'(RuleFns::<C>::default(), ReplicationMode::Once)' in rules
assert b'Self::new(default_serialize::<C>, default_deserialize::<C>)' in fns and b'let mut component: C = postcard_utils::from_buf(message)?' in fns
assert b'Deserializer::from_flavor(BufFlavor::new(buf))' in utils and b'T::deserialize(&mut deserializer)' in utils
serde=native['serde_derive-1.0.229/src/de.rs'];enum=native['serde_derive-1.0.229/src/de/enum_.rs'];tagged=native['serde_derive-1.0.229/src/de/enum_internally.rs'];postcard=native['postcard-1.1.3/src/de/deserializer.rs']
assert b'<#type_try_from as _serde::Deserialize>::deserialize(__deserializer)' in serde and b'attr::TagType::Internal { tag }' in enum and b'enum_internally::deserialize(params, variants, cattrs, tag)' in enum
assert b'_serde::Deserializer::deserialize_any(' in tagged and b'TaggedContentVisitor::<__Field>::new' in tagged
assert b'fn deserialize_any<V>' in postcard and b'Err(Error::WontImplement)' in postcard
assert b'Serializer::serialize_struct(' in native['serde_derive-1.0.229/src/ser.rs'] and b'fn serialize_struct(self' in native['postcard-1.1.3/src/ser/serializer.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,index in [(658,1971),(659,1972)]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==[index] and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    row=z['rows'][0];assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[index]]]
for n,indices in [(302,[1971,1972]),(304,[1975,1976])]:
    path=f'adler/READ-L11-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert [r['original_index'] for r in z['sources']]==indices and z['current_reconciled'] is False
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z.get('connected_reinspection',[]):
        p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
q=load('RECONCILIATION-059.json')['entries'];original=load('baseline-059/queue.json')['entries'];assert original[113]['contract_id']=='ADLER-L11-LANDFORM-REPLICATION-CODEC' and 'ADLER-L11-LANDFORM-REPLICATION-CODEC' in original[1629]['explicit_existing_contract_references']
for i in (113,1616,1617,1618,1619,1620,1629):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings'])==1 and a['conditional_findings'][0]['id']==original[113]['contract_id']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==original[113]['contract_id'] for x in ledger)==1 and next(x for x in ledger if x['id']==original[113]['contract_id'])['record']==name
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified two historical bodies and original landform codec finding against pinned library source; no codec or product execution.')
