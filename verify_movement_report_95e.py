#!/usr/bin/env python3
"""Verify movement report, historical receipt and source provenance only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-MOVEMENT-REPORT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1852-1856.json'
a, h, worklist = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1852:689, 1853:227, 1854:77, 1855:123, 1856:199}
ranges = {1852:[], 1853:[], 1854:[], 1855:[], 1856:[[8,8],[89,89],[94,94],[152,152]]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed'
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == [[1, n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert b'TacticalCombatConfig::default()' in current[1852]
assert b'garden.access[2].end_metres()' in current[1853]
assert b'property.access[4].end_metres()' in current[1855]
single = current[1856]
assert b'FABELGEIST_CENTRELINE_ONLY' in single and b'vec![0.0]' in single
assert b'FABELGEIST_SINGLE_TRACK' in single and b'vec![tracks[index.parse::<usize>().unwrap()]]' in single
assert b'"lateral_metres":lateral' in single
assert b'centre and both usable-width edges, entry and return.' in single
assert b'assert!(!observations.is_empty())' in single
assert b'self.access.is_empty()' in sources['crates/adventuresim-tactical-core/src/city_layout/gardens/geometry.rs']
assert b'access: vec![' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/gardens.rs']
assert b'compound.access.is_empty()' in sources['crates/adventuresim-tactical-core/src/scene_input/compounds.rs']
assert b'for door in doors {' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/property.rs']
assert b'for (start, end) in [(junction, turn), (turn, door)]' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/property.rs']
assert b'entries.sort_by_key(|entry| entry.id)' in sources['crates/adventuresim-building-generator/src/entrances.rs']
assert b'constructed.sort_by_key(|entry| entry.entrance)' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/single.rs']
assert b'surface.regions.push(region)' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/single/apron.rs']
main = sources['crates/adventuresim-tactical-server/src/main.rs']
assert main.index(b'app.insert_resource(combat_config)') < main.index(b'.set(AdventureSimulatorPhysicsPlugin')
assert b'load_combat_config(&combat_path)' in sources['crates/adventuresim-tactical-server/src/startup.rs']
assert b'init_resource::<crate::combat_config::TacticalCombatConfig>()' in sources['crates/adventuresim-tactical-core/src/physics/plugin.rs']
assert b'combat_config: Res<crate::combat_config::TacticalCombatConfig>' in sources['crates/adventuresim-tactical-core/src/physics.rs']
assert b'controller_config::apply_motor_config(&mut controller, motor)' in sources['crates/adventuresim-tactical-core/src/physics.rs']
for item in a['native_source_evidence']:
    archive = (root / item['archive']).read_bytes()
    assert sha(archive) == item['archive_sha256']
    with tarfile.open(root / item['archive'], 'r:gz') as tar:
        body = tar.extractfile(item['member']).read()
    assert sha(body) == item['sha256'] and len(body) == item['bytes']
    assert all(1 <= lo <= hi <= len(body.splitlines()) for lo, hi in item['ranges'])
    if item['member'].endswith('src/app.rs'):
        assert b'if there is no existing instance of `R`' in body
    if item['member'].endswith('src/world/mod.rs'):
        assert b'If the resource already exists, nothing happens.' in body
lock = sources['Cargo.lock']
assert b'5fd2aa472d43b4d081ef41d26300d17bd22cba6d68df2a223eb358782fa40bc6' in lock
assert b'a350b2e4c2e47f7903d688affb8a723b31bca63af9789f1da88703f1ba04e354' in lock
search = a['tracked_search']; result = (root / search['preserved_bytes']).read_bytes()
assert sha(result) == search['sha256'] and len(result) == search['bytes']
assert all(term.encode() in result for term in search['terms'])
assert all(b'grounding/tests/movement/single.rs:' in line for line in result.splitlines())
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(608,[1852]), (609,[1853,1854]), (610,[1855,1856])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]]
        assert row['same_lens_reuse'] == []
for n, indices in [(281,[1852,1853,1854]), (282,[1855,1856,1857,1858,1859,1860,1861])]:
    path = f'adler/READ-L11-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    assert check['head'] == 'b97bcedc262f91de07635c69cd1a4fc4f9a7d13f'
    for row in receipt['sources']:
        registration = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
        assert registration in records and row['registration_sha256'] == lookup[registration]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in range(1541,1545):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-SINGLE-REPORT-TRACK-SCOPE-001'
assert finding['accepted_historical_unique_count_change'] == 0 and 'No scene, test, visit, report' in finding['trace']
assert 'external consumers' in finding['safeguards_and_limits']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, independently attributed receipts and conditional report-scope trace; no product execution.')
