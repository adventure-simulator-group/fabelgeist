#!/usr/bin/env python3
"""Verify scoped scene, preparation and historical reviewer evidence only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda value: hashlib.sha256(value).hexdigest()

def oid(kind, value):
    return hashlib.sha1(kind.encode() + b' ' + str(len(value)).encode() + b'\0' + value).hexdigest()

def entries(value):
    out, pos = {}, 0
    while pos < len(value):
        end = value.index(b'\0', pos)
        mode, name = value[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), value[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(value)
    return out

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
a = load('ASSESSMENT-TREE-SCENE-PREPARATION-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1608-1612.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-SCENE-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, hash_ = header.split()
    tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']; raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1608,1609,1610,1611,1612]
assert [x['lines'] for x in h['historical_source_roles']] == [194,245,28,59,115]
ranges = {1608:[[1,194]],1609:[[1,203],[204,245]],1610:[[1,28]],1611:[[1,59]],1612:[[1,115]]}
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1608-1612.json'
        assert x['prior_role_status'] == 'unassessed' and x['new_personal_read_ranges'] == ranges[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is True and original == body and c['current_personal_ranges'] == ranges[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        current[i] = body
sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/'
core = 'crates/adventuresim-tactical-core/src/'
server = 'crates/adventuresim-tactical-server/src/'
residency, source, specimen, owner, preparation = (current[i] for i in range(1608,1613))
assert b'contact.scene_digest == environment.scene_digest' in residency
assert residency.index(b'TerrainContactHeightmap::from_terrain(terrain)?') < residency.index(b'cache: std::mem::take(cache)')
assert b'PREVIOUS_TREE_SCENES: usize = 2' in residency and b'cached.bark_material = cache.bark_for_terrain(species, binding)' in residency
assert b'TreePresentationSpecies' in source and b'oak_variant_for_site' in specimen
assert b'pub(crate) enum PresentationOwner' in owner and b'Scene,' in owner and b'RegionalMap,' in owner and b'RenderLayers::layer(REGIONAL_MAP_LAYER)' in owner
assert all(x in preparation for x in [b'Json(#[from] serde_json::Error)',b'DependenciesDecode(#[source] ciborium::de::Error<std::io::Error>)',b'SceneInput(#[from] Box<SceneInputError>)',b'GraphicsConfiguration { message: String }'])
assert b'JavaScriptError::new(&error.to_string())' in sources[base+'presentation/generation/wasm_error.rs']
assert b'exception.set_name(error.code())' in sources[base+'presentation/generation/wasm_error.rs']
assert b'PreparationError::GraphicsConfiguration { message }' in sources[base+'presentation/generation/city.rs']
assert b'retained.activate(' in sources[base+'presentation/obstacles/tree/presentation.rs']
assert b'TerrainContactHeightmap::from_terrain' in residency
assert b'terrain.required_elevation_range()?' in sources[base+'presentation/terrain/heightmap.rs']
assert b'terrain.required_height_at(point)?' in sources[base+'presentation/terrain/heightmap.rs']
input_ = sources[core+'scene_input.rs']; generation = sources[core+'scene_input/generation.rs']
assert b'pub fn digest(&self) -> SceneInputResult<String>' in input_ and b'self.validate()?' in input_ and b'let bytes = serde_json::to_vec(self)?' in input_ and b'Sha256::digest(bytes)' in input_
assert b'generate_unfurnished(GeneratedBuildingRecipes::default())?' in generation and b'digest: self.digest()?' in generation
assert b'building_recipes: GeneratedBuildingRecipes' in generation
assert b'buildings::prepare_buildings(&self.buildings, recipes)?' in sources[core+'scene_input/generation/geographic.rs']
assert b'self.prepare_geographic_terrain(building_recipes)?' in sources[core+'scene_input/generation/supported.rs']
env = sources[core+'scene_input/environment.rs']; terrain = sources[core+'scene.rs']; main = sources[server+'main.rs']; spawn = sources[server+'terrain_collision.rs']
assert b'#[component(immutable)]\n#[serde(deny_unknown_fields)]\npub struct SceneEnvironment' in env and b'pub scene_digest: String' in env
assert b'#[serde(try_from = "admission::TerrainWire")]\npub struct SceneTerrain' in terrain
assert b'let generated = input.generate()?' in main and b'input.environment_snapshot(generated.digest)' in main
assert b'terrain_collision::spawn_scene(' in main and b'scene.insert(environment)' in spawn and b'scene.insert(terrain)' in spawn

manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for x in a['recovered_historical_records']:
    original = lookup[x['original_path']]
    assert (x['sha256'], x['bytes'], x['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    body = contents(x['preserved_bytes'])
    assert sha(body) == x['published_sha256'] and len(body) == x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body) == x['sha256'] and oid('blob', body) == x['git_blob'])
    records[x['original_path']] = body
for n, indices in [(469,[1530,1531]),(476,[1546,1547,1548,1549]),(506,[1608,1609,1610]),(507,[1611,1612])]:
    path=f'segovb/READ-L12-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for reused in receipt.get('qualified_supporting_same_lens_reuse',[]):
        assert reused['registration'] in records and reused['registration_sha256'] == lookup[reused['registration']]['sha256']
        assert reused['receipt'] in records and reused['receipt_sha256'] == lookup[reused['receipt']]['sha256']
        assert reused['fresh_personal_body_reads_this_batch'] == []
seg = json.loads(records['segovb/READ-L12-507.json'])
assert seg['rows'][1]['fresh_personal_body_reads'] == [[1,115]]
assert [x['original_index'] for x in seg['qualified_supporting_same_lens_reuse']] == [1530,1547]
resolution = seg['connected_contract_resolutions'][0]
assert resolution['historical_original_index'] == 1537 and resolution['successor_original_index'] == 1612
assert resolution['historical_qualification'] in records and resolution['historical_qualification_sha256'] == lookup[resolution['historical_qualification']]['sha256']
assert resolution['current_source_reconciled'] is False and resolution['other_lens_coverage_transferred'] is False
for n, indices in [(218,list(range(1535,1550))),(230,[1608,1609,1610,1611,1612])]:
    path=f'adler/READ-L11-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources'] + receipt['connected_reinspection']:
        if row.get('registration_sha256'):
            reg = 'adler/' + row['registration'].replace('\\','/').split('/adler/')[-1]
            assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
ad218 = json.loads(records['adler/READ-L11-218.json']); ad230 = json.loads(records['adler/READ-L11-230.json'])
prior = next(x for x in ad218['connected_reinspection'] if x['original_index'] == 1612)
later = next(x for x in ad230['sources'] if x['original_index'] == 1612)
assert prior['fresh_reads'] == [[1,115]] and later['fresh_reads'] == []
assert prior['registration_sha256'] == later['registration_sha256'] == 'fa5850403997159941dc11c7c32b8ed3cd6321f7eca98ff35c35f4e6423a8661'
assert next(x for x in ad218['sources'] if x['original_index'] == 1537)['fresh_reads'] == []
deleted = load('ASSESSMENT-L01-DELETED-1537.json')
assert deleted['registry_state'] == 'deleted' and deleted['snapshot_sha256'] is None and deleted['historical_object_available'] is False
assert deleted['base_revision'] == '3a50b708381ad00c116445c2c999a49c3943088b' and deleted['base_sha256'] != w['roles'][1612]['historical_sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1340,1341))
baseline = load('BASELINE-95E-20261110.json'); ledger = baseline['newly_assessed_ledger']['entries']
assert len(ledger) >= 265 and a['conditional_findings'] == []
assert [x['id'] for x in a['existing_finding_connections']] == ['ADLER-L11-GRAPHICS-CONFIG-ERROR-OWNERSHIP']
assert sum(x['id']=='ADLER-L11-GRAPHICS-CONFIG-ERROR-OWNERSHIP' and 'ASSESSMENT-TREE-SCENE-PREPARATION-95E.json' in x['additional_evidence'] for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified five historical bodies, distinct successor/base attribution and bounded scene/preparation source; no product execution.')
