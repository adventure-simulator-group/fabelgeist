#!/usr/bin/env python3
"""Check historical/source lineage and symbolic bounds; no product execution."""
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
a = load('ASSESSMENT-PLANT-WORK-AND-SPECIES-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1574-1581.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('PLANT-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    head, path = line.split('\t', 1)
    mode, kind, hash_ = head.split()
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

assert [x['original_index'] for x in h['historical_source_roles']] == list(range(1574, 1582))
assert [x['lines'] for x in h['historical_source_roles']] == [232, 140, 11, 301, 99, 67, 147, 96]
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1574-1581.json'
        ranges = [[1, 180], [181, 301]] if i == 1577 else [[1, x['lines']]]
        assert x['new_personal_read_ranges'] == ranges
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is True and original == body and c['current_personal_ranges'] == ranges
        check_chain(c['path'], proof['paths'][c['path']], original)
        current[i] = body
    extra = a['earlier_historical_source_identities']
    assert [x['original_index'] for x in extra] == [1562,1582,1593,1596,1600,1604,1605,1607,1699,1725]
    for x in extra:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256']
        assert len(body) == x['bytes'] and len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob']
        assert x['new_personal_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
client = 'crates/adventuresim-tactical-client/src/'
core = 'crates/adventuresim-tactical-core/src/'
plants, specimens, mask = current[1574], current[1578], current[1579]
terrain = sources[core + 'scene.rs']
admission = sources[core + 'scene/admission.rs']
refine = sources[core + 'scene_input/refinement.rs']
scene_input = sources[core + 'scene_input.rs']
assert b'const SITE_SPACING_METRES: f32 = 2.0' in plants and b'const MAX_SPECIMENS: usize = 512' in plants
assert b'for z in 0..count_z' in plants and b'for x in 0..count_x' in plants
assert plants.index(b'for z in 0..count_z') < plants.index(b'candidates.sort_by_key') < plants.index(b'candidates.truncate(MAX_SPECIMENS)')
assert b'self.grid_width().saturating_sub(1) as f32 * self.scale' in terrain
assert b'const MAX_PLAYABLE_SIDE: usize = 601' in scene_input and b'0.25..=2_000.0' in scene_input
assert b'(MAX_PLAYABLE_SIDE - 1) / largest_source_side' in scene_input and b'length > MAX_SCENE_INPUT_BYTES' in scene_input
assert b'32 * 1024 * 1024' in sources[core + 'scene_input/descriptor.rs']
assert b'AUTHORITATIVE_DETAIL_SPACING_METRES: f32 = 0.5' in refine
assert b'.refined(AUTHORITATIVE_DETAIL_SPACING_METRES' in refine
assert b'.checked_mul(6)' in admission and b'n > u32::MAX as usize' in admission
assert 600 // 600 == 1 and 600 * 20 == 12000 and 12000 // 2 == 6000
assert (24000 + 1) ** 2 <= 2**32 - 1 and 24000**2 * 6 <= 2**32 - 1
assert 6000**2 == 36_000_000 and (600 * 2000 * 2)**2 * 6 > 2**32 - 1
assert b'CoverageMask::new' in plants and b'grass_cover_mask_pixels(ground, seed)?' in sources[client + 'presentation/ground_scatter/cover_mask.rs']
assert b'SceneGround::from_samples(' in mask and b'expect("masking scatter preserves the validated ground grid")' in mask
understory = sources[client + 'presentation/ground_scatter/understory.rs']
views = sources[client + 'tactical_scene_viewer/view_specs.rs']
viewer = sources[client + 'tactical_scene_viewer.rs']
instanced = sources[client + 'presentation/ground_scatter/instanced_understory.rs']
assert b'enum UnderstorySpecies' in understory and all(s in understory for s in [b'CommonHazel', b'Blackthorn', b'CommonHawthorn'])
assert b"common_name: &'static str" in specimens and all(s in specimens for s in [b'"common hazel"',b'"blackthorn"',b'"common hawthorn"'])
assert b"understory_species: Option<&'static str>" in views and all(b'.understory("' + s + b'")' in views for s in [b'common hazel',b'blackthorn',b'common hawthorn'])
assert b'view.understory_species == Some(specimen.common_name)' in viewer and b'Visibility::Visible' in viewer
assert b'"Shared {common_name} "' in viewer and b'"Instanced {species:?} shrub {label} ({})"' in instanced

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
for n in [482,488,489,490,491,496,497,500,503,505,543,555]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for partial in receipt.get('partial_sequence', []):
        assert partial['path'] in records and partial['sha256'] == lookup[partial['path']]['sha256']
    for previous in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert previous['receipt'] in records and previous['receipt_sha256'] == lookup[previous['receipt']]['sha256']
        assert previous['registration'] in records and previous['registration_sha256'] == lookup[previous['registration']]['sha256']
assert [x['original_index'] for x in json.loads(records['segovb/READ-L12-488.json'])['rows']] == [1574,1575,1576]
assert [x['original_index'] for x in json.loads(records['segovb/READ-L12-489.json'])['rows']] == [1577,1578]
assert [x['original_index'] for x in json.loads(records['segovb/READ-L12-490.json'])['rows']] == [1579,1580,1581]
prior = json.loads(records['segovb/READ-L12-555.json'])
assert [x['original_index'] for x in prior['rows']] == [1725] and len(prior['partial_sequence']) == 2
assert any('Additive correction:' in x and 'not Shared' in x for x in prior['connected_dispositions'])
assert len(json.loads(records['segovb/READ-L12-543.json'])['partial_sequence']) == 8
adler = json.loads(records['adler/READ-L11-222.json'])
check = json.loads(records['adler/VERIFY-L11-222.json'])
assert [x['original_index'] for x in adler['sources']] == list(range(1574,1583))
assert adler['accepted_unique_count_change'] == 0 and adler['current_reconciled'] is False and adler['validated'] is False
assert check['receipt_sha256'] == lookup['adler/READ-L11-222.json']['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-222.json']['sha256']
for row in adler['sources']:
    path = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
    assert path in records and row['registration_sha256'] == lookup[path]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (110,258,1322,1323,1324))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
p = load('CONTINUATION-PROGRESS-95E.json')
assert len(a['conditional_findings']) == len(a['existing_finding_connections']) == 1
for item in a['conditional_findings'] + a['existing_finding_connections']:
    assert item['repair'] == 'not started' and item['validation'] == 'not performed' and item['accepted_historical_unique_count_change'] == 0
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-PLANT-WORK-AND-SPECIES-95E.json' for x in ledger) == 1
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
check = load(baseline['latest_main_recheck'])
assert check['observed_revision'] == revision and check['ref'] == 'refs/heads/main'
print('Verified eight historical bodies, earlier connected receipts, bounded current sources and symbolic work case; no product execution.')
