#!/usr/bin/env python3
"""Verify recovered identity, provenance sources and recorded read scopes."""
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
a = load('ASSESSMENT-TREE-SHOOT-PROVENANCE-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1592-1597.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-SHOOT-HISTORICAL-GIT-PROOFS.json')
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

assert [x['original_index'] for x in h['historical_source_roles']] == list(range(1592,1598))
assert [x['lines'] for x in h['historical_source_roles']] == [366,505,279,20,61,32]
expected_history = {1592:[[1,190],[191,366]],1593:[[1,125],[126,250],[251,505]],1594:[[1,160],[161,279]],1595:[[1,20]],1596:[[1,61]],1597:[[1,32]]}
expected_current = {1592:[[1,90],[245,295],[320,374]],1593:[[1,125],[130,220],[240,275]],1594:[[1,150],[151,277]],1595:[[1,19]],1596:[[1,61]],1597:[[1,31]]}
current, historic = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1592-1597.json'
        assert x['new_personal_read_ranges'] == expected_history[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is False and original != body and c['current_personal_ranges'] == expected_current[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
        d = c['exact_diff']
        assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        current[i] = body
        historic[i] = original
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == list(range(1600,1608))
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] and x['new_personal_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)
        if x['original_index'] == 1600:
            assert x['new_personal_bounded_ranges'] == [[707,745]] and b'leaf.shoot_id as f32' in body

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/presentation/obstacles/tree/'
identity, old_identity = current[1595], historic[1595]
leaves, geometry, arrangement, selection = current[1593], current[1592], current[1594], current[1596]
assert b'struct TreeShootId(u64)' in identity and b'SHOOT_IDENTITY.seed(source, &[])' in identity
assert b'shoot.start.x.to_bits()' in old_identity and b'shoot.end.z.to_bits()' in old_identity
assert b'foliage: Option<TreeShootId>' in geometry and b'shoot_id: TreeShootId' in leaves
assert b'branch.foliage.map(|identity|' in arrangement and b'shoot_key.seed()' in selection
skeleton = sources[base+'geometry/skeleton.rs']
assert b'TreeShootId::from_seed(shoot_seed)' in skeleton and b'TreeShootId::from_seed(twig_seed)' in skeleton and b'TreeShootId::from_seed(tip_seed)' in skeleton
assert b'foliage: foliage.filter(|_| index + 1 == segment_count)' in skeleton
assert b'TreeShootId::from_seed(shoot_seed)' in sources[base+'geometry/skeleton/growth_axes.rs']
seed = sources['crates/fabelgeist-determinism/src/seed.rs']
assert b'pub const fn from_u64(value: u64) -> Self' in seed and b'Self { word: value }' in seed and b'pub const fn to_u64(self) -> u64' in seed
imp = sources[base+'impostor.rs']
start = imp.index(b'pub(in crate::presentation) fn tree_source_geometry_hash(')
end = imp.index(b'pub(in crate::presentation) fn tree_impostor_material(',start)
hash_body = imp[start:end]
assert b'leaf.shoot_id.seed().to_u64()' in hash_body
assert all(field not in hash_body for field in [b'leaf.right',b'leaf.up',b'leaf.torsion'])
assert b'leaf.torsion * 1.15' in leaves and b'twist * leaf.right' in leaves and b'leaf.up * (v - 0.5)' in leaves
raster = sources[base+'impostor/raster.rs']
assert b'procedural_woody_cambered_leaf_mesh(&source_leaves)' in raster and b'project_to_tile(card, Vec3::from_array(positions[index])' in raster
assets = sources[base+'impostor_assets.rs']
assert b'bake.usage == usage' in assets and b'bake.species == species_tag(species)' in assets and b'bake.seed == seed' in assets and b'bake.lod == lod' in assets
assert b'#[cfg(test)]\n    pub(crate) fn matches_art_demo' in assets

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
for n, indices in [(496,[1592,1593]),(497,[1594,1595,1596,1597]),(500,[1600]),(503,[1604,1605])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for partial in receipt.get('partial_sequence',[]):
        assert partial['path'] in records and partial['sha256'] == lookup[partial['path']]['sha256']
    for reused in receipt.get('qualified_supporting_same_lens_reuse',[]):
        assert reused['registration'] in records and reused['registration_sha256'] == lookup[reused['registration']]['sha256']
        assert reused['receipt'] in records and reused['receipt_sha256'] == lookup[reused['receipt']]['sha256']
for n, indices in [(224,list(range(1592,1598))),(227,[1600,1601,1602]),(228,[1603,1604,1605,1606]),(229,[1607])]:
    path=f'adler/READ-L11-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (77,80,1330,1331))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
p = load('CONTINUATION-PROGRESS-95E.json')
assert len(a['existing_finding_connections']) == 2
for item in a['existing_finding_connections']:
    assert item['repair'] == 'not started' and item['validation'] == 'not performed' and item['accepted_historical_unique_count_change'] == 0
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-TREE-SHOOT-PROVENANCE-95E.json' for x in ledger) == 1
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
check = load(baseline['latest_main_recheck'])
assert check['observed_revision'] == revision and check['ref'] == 'refs/heads/main'
print('Verified six historical bodies, earlier tree receipts and bounded current shoot/provenance source; no product execution.')
