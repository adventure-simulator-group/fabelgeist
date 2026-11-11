#!/usr/bin/env python3
"""Verify plant source and prior records, without executing product code."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-PLANT-LOD-ROOT-ASSOCIATION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1720-1724.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('PLANT-LOD-ROOT-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1720:113, 1721:166, 1722:350, 1723:122, 1724:78}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert old == body and c['byte_identical'] is True and c['current_personal_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
plants = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/plants.rs']
lod = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/plants/lod.rs']
determinism = sources['crates/fabelgeist-determinism/src/lib.rs']
stream = sources['crates/fabelgeist-determinism/src/stream.rs']
assert b'const SITE_SPACING_METRES: f32 = 2.0;' in plants
assert plants.count(b'+ 0.15') == 2 and plants.count(b'.inclusive_unit_f32() * 0.7') == 2
assert plants.count(b'candidates.push(PlantSite {') == 1
assert b'cache.spawn(&mut commands, site.species, transform)' in plants and b'anchors.push(PlantCaptureAnchor {' in plants
assert b'for level in PlantLod::ALL {' in lod and b'commands.spawn((' in lod
assert b'((1_u32 << 24) - 1) as f32' in determinism and b'(value >> 40) as f32 / INCLUSIVE_UNIT_F32_SCALE' in determinism
assert b'crate::inclusive_unit_f32(self.next_u64())' in stream
assert (1 + Fraction(15,100) - Fraction(85,100)) * 2 == Fraction(3,5)
assert Fraction(1,10000) ** 2 == Fraction(1,100000000)
observer = current[1720]
assert b'distance_squared(root) < 1e-8' in observer and b'VisibleEntities' in observer and b'PlantLodInstance' in observer and b'.indices()' in observer
obstacles = current[1721]
assert b'.collect::<Result<Vec<_>>>()?' in obstacles and b'TREE_TRUNK_HEIGHT_METRES * 0.5' in obstacles
assert b'TREE_TRUNK_HEIGHT_METRES * 0.5 - 0.22' in current[1722]
placement = current[1724]
assert b'.cross(up.vector())' in placement and b'SceneCameraOrientation::Fixed(rotation)' in placement
lifecycle = sources['crates/adventuresim-tactical-client/src/presentation/demo_lifecycle.rs']
assert b'pub(crate) fn clear_scene_entities' in lifecycle and b'With<GroundScatterLayer>' in lifecycle and b'ownership::PresentationOwner::RegionalMap' in lifecycle
# Earlier citations remain explicitly distinct from the later full-role reads.
prior = load('ASSESSMENT-LITTER-CAPTURE-CLEARANCE-95E.json')
assert 'ASSESSMENT-LITTER-CAPTURE-CLEARANCE-95E.json' in [x['file'] for x in a['historical_records']]
assert 'ASSESSMENT-CATALOG-ROW-IDENTITY-95E.json' in [x['file'] for x in a['historical_records']]

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(553,[1720,1721,1722]), (554,[1723,1724])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('partial_sequence', []):
        assert x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
    for x in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
        assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
assert a['new_unique_findings'] == []
q = load('RECONCILIATION-059.json')['entries']
for i in (1322,1431):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified plant roles, prior receipts and qualified source reasoning; no product execution.')
