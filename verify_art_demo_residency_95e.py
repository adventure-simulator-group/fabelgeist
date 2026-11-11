#!/usr/bin/env python3
"""Verify historical body identities and the static art-demo failure connection."""
import difflib,gzip,hashlib,json,math,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-ART-DEMO-RESIDENCY-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1449-1456.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1449,1457))
assert [x['lines'] for x in h['historical_source_roles']]==[343,179,136,275,45,336,365,41]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1449-1456.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1454:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-1454/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256'] and len(cur.splitlines())==348
   assert raw.split(b'#[cfg(test)]',1)[0]==cur.split(b'#[cfg(test)]',1)[0]
  else: assert raw==cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
assert b'.add_plugins(presentation)' in current[1449] and b'.add_plugins(materials::TacticalMaterialsPlugin)' in s['crates/adventuresim-tactical-client/src/presentation/mod.rs']
assert b'.add_plugins(adventuresim_procedural_textures::BakedTexturesPlugin)' in s['crates/adventuresim-tactical-client/src/presentation/materials.rs']
texture=s['crates/adventuresim-procedural-textures/src/runtime.rs'];assert b'.init_resource::<ProceduralTextureResidency>()' in texture and b'.or_insert_with(|| server.load(recipe.runtime_asset_path()))' in texture
assert b'.all(|recipe| self.installed.contains(&recipe))' in texture and b'recipes.get(handle)' in texture and b'residency.installed.insert(id)' in texture and b'reader.read_to_end(&mut bytes).await?' in texture
res=current[1454];assert b'if !textures_ready(world, exhibit)' in res and b'return Ok(false);' in res and b'world.insert_resource(PendingScenery(id))' in res
assert b'Ok(false) => return,' in res and b'get_load_state(handle.id())' in res and b'is_loaded_with_dependencies(handle.id())' in res
assert b'if scenery.is_some()' in current[1449] and b'DemoStatus::Loading' in current[1449]
js=s['crates/strategic-web/static/art-demo/viewer.mjs'];assert b'loading.hidden = status.state === "ready"' in js and b'"Loading exhibit\xe2\x80\xa6"' in js
impostor=s['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/impostor_assets.rs'];assert b'_context: &mut LoadContext' in impostor and b'PreparedTreeImpostorAsset::from_compressed_bytes(&compressed)' in impostor
city=s['crates/adventuresim-tactical-client/src/presentation/buildings/streaming.rs'];production=city[city.index(b'impl CityBuildingAssets'):city.index(b'pub(super) fn present')];assert b'Ok(true)' in production and b'Ok(false)' not in production
cat=json.loads(s['assets/art-demo/catalog.json']);game=json.loads(s['crates/adventuresim-weapon-model/catalog/gameplay.json']);obs=a['current_catalogue_static_observation'];assert [x['id'] for x in cat]==obs['ids'] and len(set(obs['ids']))==10
assert all(len(x['focus'])==3 and all(math.isfinite(v) for v in x['focus']) and math.isfinite(x['distance']) and x['distance']>0 for x in cat)
assert [x['asset'] for x in cat if x['kind']=='weapon']==obs['weapon_catalogue_ids'] and all(x in {y['catalog_id'] for y in game} for x in obs['weapon_catalogue_ids'])
lock=s['Cargo.lock'];assert b'name = "bevy_asset"\nversion = "0.19.1"' in lock and b'1624eddbf2293441b2158506aba75e523432b898d9da26eb6ed3e20af4c7fd38' in lock
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {423:[1449],424:[1450,1451],425:[1452,1453],426:[1454],427:[1455,1456]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
for n in (205,206):
 receipt=json.loads(records[f'adler/READ-L11-{n}.json']);state=json.loads(records[f'adler/STATE-L11-{n}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert state['receipt_sha256']==m[f'adler/READ-L11-{n}.json']['sha256']
check=json.loads(records['adler/VERIFY-L11-206.json']);assert [x['receipt_sha256'] for x in check['state_receipt_pairs']]==[m[f'adler/READ-L11-{n}.json']['sha256'] for n in (205,206)]
assert 'adler/VERIFY-L11-205.json' not in m and all(not x['matching_relative_paths'] and x['error_count']==0 for x in a['targeted_local_evidence_search']['collections'])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=list(range(1247,1255));assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==list(range(1449,1457));assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
ext=a['historical_catalogue_extension'];eread=json.loads(records[ext['receipt']]);ereg=json.loads(records[ext['registration']]);esel=json.loads(records[ext['selector']]);original=records[ext['exact_original_source']]
assert sha(original)==ext['source_sha256']==eread['source_sha256']==ereg['source_sha256'] and blob(original)==ext['source_git_blob'] and original==s['assets/art-demo/catalog.json']
assert eread['fresh_personal_body_reads']==[[1,154]] and eread['current_source_reconciled'] is False and eread['accepted_unique_count_change']==0 and eread['new_original_overlay_merged_shared_population_credit']==0 and len(eread['other_lenses_pending'])==12
assert any(x['receipt']==ext['receipt'] and x['receipt_sha256']==m[ext['receipt']]['sha256'] and x['registration_sha256']==m[ext['registration']]['sha256'] for x in esel['own_extensions'])
assert old[1255]['first_failed_contract_or_unresolved_question']['source_id']=='pr-working:assets/art-demo/catalog.json' and q[1255]['current_reconciliation'].startswith('partial ')
ident='ADLER-L11-TEXTURE-RESIDENCY-FAILURE-STATE';assert ident in load('baseline-059/ledger.json')['discovery_findings_and_dependencies'] and a['conditional_findings'][0]['id']==ident
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==ident for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified eight historical bodies and one bounded current reassessment of a prior finding; no product execution.')
