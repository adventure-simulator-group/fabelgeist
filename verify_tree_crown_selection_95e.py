#!/usr/bin/env python3
"""Verify recovered crown evidence and current source identities, without product execution."""
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
a = load('ASSESSMENT-TREE-CROWN-SELECTION-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1600-1602.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-CROWN-HISTORICAL-GIT-PROOFS.json')
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

assert [x['original_index'] for x in h['historical_source_roles']] == [1600,1601,1602]
assert [x['lines'] for x in h['historical_source_roles']] == [1231,54,56]
hr = {1600:[[1,220],[221,440],[441,660],[661,880],[881,1060],[1061,1231]],1601:[[1,54]],1602:[[1,56]]}
cr = {1600:[[1,220],[221,440],[441,660],[661,880],[881,1060],[1061,1233]],1601:[[1,54]],1602:[[1,56]]}
current, historic = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1600-1602.json'
        assert x['new_personal_read_ranges'] == hr[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i in (1601,1602)) and c['current_personal_ranges'] == cr[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        if i == 1600:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']
            assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        current[i], historic[i] = body, original
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1603,1607,1609]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] and x['new_personal_historical_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/presentation/obstacles/tree/'
skeleton = sources[base+'geometry/skeleton.rs']
assert b'for primary_index in 0..20_u64' in skeleton and b'primary_index as f32 * 2.399_963_1' in skeleton
assert b'(primary_unit_draw(streams::PRIMARY_PHASE) - 0.5) * 0.42' in skeleton
assert b'for secondary_index in 0..5_u64' in skeleton and b'(primary_index * 8 + secondary_index) as u16' in skeleton
assert b'.min(TREE_PRIMARY_GROUP_COUNT - 1)' in skeleton and b'Some(TreeShootId::from_seed(twig_seed))' in skeleton
assert b'const TREE_PRIMARY_GROUP_COUNT: u8 = 7' in sources[base+'geometry.rs']
assert b'primary_group: shoot.primary_group' in sources[base+'geometry/leaves/arrangement.rs']
assert b'secondary_group: shoot.secondary_group' in sources[base+'geometry/leaves/arrangement.rs']
imp = current[1600]
assert b'(Some((first, last)), _) => (first..=last).contains(&leaf.secondary_group)' in imp
assert b'(first..=last).contains(&branch.secondary_group)' in imp
assert b'branch.depth == 2 && branch.primary_group == primary_group' in imp
assert b'(groups[first], groups[last])' in imp and b'const LOD1_MACRO_SUBCLUSTERS_PER_PRIMARY: usize = 2' in imp
assert b'1 << primary_group,\n                            Some(secondary_group_range)' in imp
assert b'filter(|branch| probe.includes_branch(branch))' in imp and b'filter(|leaf| probe.includes_leaf(leaf))' in imp
orientation = current[1602]
assert b'branch.primary_group == primary_group' in orientation and b'(first..=last).contains(&branch.secondary_group)' in orientation
raster = sources[base+'impostor/raster.rs']
assert b'filter(|branch| card.includes_branch(branch))' in raster and b'filter(|leaf| card.includes_leaf(leaf))' in raster
source = sources[base+'source.rs']
assert b'TreePresentationSpecies::CommonBeech => {' in source and b'COMMON_BEECH_PARAMETERS' in source and b'BEECH_TREE_BAKE_STYLE' in source
presentation = sources[base+'presentation.rs']
assert b'if let Some(prepared) = prepared {' in presentation and b'cached.species.bake_style()' in presentation
random = sources['crates/fabelgeist-determinism/src/lib.rs']
assert b'(value >> 40) as f32 / INCLUSIVE_UNIT_F32_SCALE' in random
assert b'((1_u32 << 24) - 1) as f32' in random
start = imp.index(b'pub(in crate::presentation) fn tree_source_geometry_hash(')
end = imp.index(b'pub(in crate::presentation) fn tree_impostor_material(',start)
hash_body = imp[start:end]
assert all(field not in hash_body for field in [b'branch.depth',b'branch.primary_group',b'branch.secondary_group',b'leaf.primary_group',b'leaf.secondary_group'])
assert b'const TREE_IMPOSTOR_BAKE_VERSION: u32 = 16' in historic[1600] and b'const TREE_IMPOSTOR_BAKE_VERSION: u32 = 19' in imp
assert b'lod1_runtime_width_scale: 1.0,' in historic[1600] and b'lod1_runtime_width_scale: 0.98,' in imp

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
for n, indices in [(500,[1600]),(501,[1601,1602]),(502,[1603]),(506,[1608,1609,1610])]:
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
    previous = None
    for j, x in enumerate(receipt.get('partial_sequence',[])):
        assert n == 500 and x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
        part = json.loads(records[x['path']]); assert part['original_index'] == 1600 and part['actual_EOF'] == 1231
        assert part['source_sha256'] == w['roles'][1600]['historical_sha256'] and part['original_credit'] == 0
        assert part['fresh_personal_body_reads'] == [[[1,600]],[[601,1200]],[[1201,1231]]][j]
        assert part['unread_tail'] == [[[601,1231]],[[1201,1231]],[]][j]
        if previous is not None:
            assert part['predecessor'] == previous
        previous = x
for n, indices in [(227,[1600,1601,1602]),(228,[1603,1604,1605,1606]),(229,[1607]),(230,[1608,1609,1610,1611,1612])]:
    path=f'adler/READ-L11-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (79,80,1334,1335,1336))
baseline = load('BASELINE-95E-20261110.json');ledger = baseline['newly_assessed_ledger']['entries']
item = a['existing_finding_connections'][0]
assert item['id'] == 'ADLER-L11-TREE-CROWN-SELECTION-OWNERSHIP' and item['accepted_historical_unique_count_change'] == 0
assert item['repair'] == 'not started' and item['validation'] == 'not performed'
assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-TREE-CROWN-SELECTION-95E.json' for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified three historical/current bodies, earlier attribution and scoped current crown source; no product execution.')
