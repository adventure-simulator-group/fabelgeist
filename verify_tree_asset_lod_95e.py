#!/usr/bin/env python3
"""Verify tree asset/LOD evidence and source-only fixture observations."""
import difflib
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
a = load('ASSESSMENT-TREE-ASSET-LOD-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1603-1605.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-ASSET-HISTORICAL-GIT-PROOFS.json')
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
commit = shared['revisions'][proof['revision']]['commit']
raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1603,1604,1605]
assert [x['lines'] for x in h['historical_source_roles']] == [976,444,402]
hr = {1603:[[1,200],[201,400],[401,600],[601,800],[801,976]],1604:[[1,220],[221,444]],1605:[[1,210],[211,402]]}
cr = {1603:[[1,299],[685,932]],1604:[[1,205],[206,444]],1605:[[1,210],[211,402]]}
current, historic = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1603-1605.json'
        assert x['new_personal_read_ranges'] == hr[i]
        if i == 1605:
            assert x['prior_role_status'] == 'bounded-current-trace-only'
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i in (1604,1605)) and c['current_personal_ranges'] == cr[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        if i == 1603:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']
            assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        current[i], historic[i] = body, original
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1610]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]; body = archive.extractfile(x['sha256']).read(); path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] == tree[path] and x['new_personal_historical_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
fixture = a['current_static_fixture']; body = contents(fixture['preserved_bytes'])
assert fixture['revision'] == revision and sha(body) == fixture['sha256'] and len(body) == fixture['bytes']
assert oid('blob', body) == fixture['git_blob'] == tree[fixture['path']]
value = json.loads(body)
assert {key:value[key] for key in fixture['inspected_fields']} == fixture['inspected_fields']
assert fixture['inspected_fields']['seed'] == {'word':47105}
base = 'crates/adventuresim-tactical-client/src/'
asset, lod, raster, old_raster = current[1604], current[1605], current[1603], historic[1603]
imp = sources[base+'presentation/obstacles/tree/impostor.rs']
presentation = sources[base+'presentation/obstacles/tree/presentation.rs']
assert b'GzDecoder::new(compressed).read_to_end(&mut bytes)?' in asset and b'postcard::from_bytes(&bytes)' in asset
assert b'fn runtime(&self) -> TreeLodBake' in asset and b'assert_eq!(self.bake_version, TREE_IMPOSTOR_BAKE_VERSION)' in asset
assert b'bake.usage == usage' in asset and b'bake.species == species_tag(species)' in asset and b'bake.seed == seed' in asset and b'bake.lod == lod' in asset
assert b'art-demo/oak.tree-impostors' in asset and b'const TREE_IMPOSTOR_BAKE_VERSION: u32 = 19' in imp
assert b'resource_mut::<crate::presentation::PreparedTreeImpostorAssets>()' in sources[base+'art_demo/residency.rs']
assert b'pub(crate) struct TreeLod(pub(crate) u8)' in lod and b'pub(crate) lod: Option<u8>' in lod
assert b'tree_projected_lod_visibility(lod.0, focal_scale, radius)' in lod
assert b'_ => unreachable!("tree LOD is bounded")' in imp
assert b'(1 << (lod + 1)) | u8::from(lod < 4)' in presentation
assert b'for lod in 1..=3' in presentation and b'TreeLod(0)' in presentation and b'TreeLod(4)' in presentation
viewer = sources[base+'tactical_scene_viewer.rs']; specs = sources[base+'tactical_scene_viewer/view_specs.rs']
assert all(('forced_lod: Some(%d)'%i).encode() in viewer for i in range(5))
assert all(('.render_lod(%d)'%i).encode() in specs for i in range(5))
assert b'tree_lod_override.lod = view.render_lod_override' in viewer
assert b'const SCENE_SEED: u64 = 47_104' in old_raster and b'obstacle_seed.to_u64() & 3' in old_raster
assert b'assert_eq!(variant_index, 1)' in old_raster and b'FOCUSED_TREE_RECIPE_SEED' in old_raster
assert b'oak_variant_for_site(point)' in raster and b'TacticalSceneInput::load(' in raster
assert b'assert_eq!(variant_index, 1)' not in raster and b'FOCUSED_TREE_RECIPE_SEED' not in raster
assert b'.rng(obstacle_seed(point), &[])' in sources[base+'presentation/obstacles/tree/specimen.rs']

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
for n, indices in [(502,[1603]),(503,[1604,1605]),(506,[1608,1609,1610])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
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
for n, indices in [(228,[1603,1604,1605,1606]),(229,[1607]),(230,[1608,1609,1610,1611,1612])]:
    path=f'adler/READ-L11-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (81,82,85,1336,1337))
baseline = load('BASELINE-95E-20261110.json'); ledger = baseline['newly_assessed_ledger']['entries']
assert [x['id'] for x in a['existing_finding_connections']] == ['ADLER-L11-PREPARED-TREE-ASSET-ADMISSION','ADLER-L11-TREE-LOD-ADMISSION','ADLER-L11-FOCUSED-TREE-VARIANT-ORACLE-AUTHORITY']
assert a['conditional_findings'] == []
for item in a['existing_finding_connections'] + a['conditional_findings']:
    assert item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed'
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-TREE-ASSET-LOD-95E.json' for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified three historical bodies, scoped asset/LOD source, earlier attribution and static fixture metadata; no product execution.')
