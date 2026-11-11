#!/usr/bin/env python3
"""Verify source, provenance and pinned macro identity without executing product code."""
import gzip
import hashlib
import json
import tarfile
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
        end = data.index(b'\0', pos); mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex()); pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child']); current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-HOME-IDENTITY-95E.json'; hist = 'ASSESSMENT-L01-HISTORICAL-1790-1794.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('HOME-IDENTITY-HISTORICAL-GIT-PROOFS.json'); shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']: nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json'); tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t',1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1790:(1969,1969),1791:(217,227),1792:(160,160),1793:(216,216),1794:(57,57)}
ranges = {1790:[[847,887]],1791:[[1,227]],1792:[],1793:[],1794:[]}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n,m = lengths[i]; row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read(); c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        same = i != 1791
        assert (old == body) == c['byte_identical'] == same
        assert c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == ([[1,n]] if same else [])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == m
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if not same:
            diff = contents(c['exact_diff']['preserved_bytes'])
            assert sha(diff) == c['exact_diff']['sha256'] and len(diff) == c['exact_diff']['bytes']
            assert b'diff --git ' in diff and b'@@ ' in diff
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body) == item['git_blob'] == tree[item['path']]; sources[item['path']] = body
native = {}
for item in a['native_archive_members']:
    raw = (root / item['archive']).read_bytes(); assert sha(raw) == item['archive_sha256']
    with tarfile.open(root / item['archive'], 'r:gz') as archive: body = archive.extractfile(item['member']).read()
    assert sha(body) == item['member_sha256'] and len(body) == item['member_bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo,hi in item['personal_ranges'])
    assert item['archive_sha256'].encode() in sources['Cargo.lock']; native[item['member']] = body
macro = native['serde_derive-1.0.229/src/de.rs']
assert b'if cont.attrs.transparent()' in macro and b'deserialize_transparent(cont, params)' in macro
assert b'quote_spanned!(span=> _serde::Deserialize::deserialize)' in macro
assert b'|__transparent| #this_value { #(#assign),* }' in macro
prop = sources['crates/adventuresim-core/src/settlement_property.rs']
assert b'#[serde(transparent)]\npub struct PropertyId(String);' in prop
assert b'StrategicIdentityComponent::try_new(settlement)' in prop and b'if building_id == 0' in prop
assert b'"property:{}:{settlement}:{building_id}"' in prop
assert b'serde_json::from_str(json).map_err(|_| PropertyError::InvalidCatalog)' in prop
assert b'home.id != PropertyId::new(settlement, home.building_id)?' in prop
assert b'catalog.validate(settlement_id, population)?' in current[1794]
assert b'catalog\n        .validate(' in sources['crates/adventuresim-stdb-module/src/residence/properties.rs']
assert b'.validate(&catalog.settlement_id, catalog.population)' in sources['crates/adventuresim-tactical-core/src/scene_input/properties.rs']
assert b'pub id: String' in sources['crates/adventuresim-stdb-module/src/strategic/world_import.rs']
assert b'pub settlement_id: String' in prop
assert b'settlement_id: StrategicIdentityComponent' in sources['crates/adventuresim-core/src/strategic_place.rs']
assert b'pub fn settlement_building_seed(settlement_id: &str)' in sources['crates/adventuresim-core/src/settlement_population/demographics.rs']
assert b'held_guard_input_survives_zero_velocity_until_front_contact' in current[1790]
assert b'SpeedError::Fraction(error) => Self::from(error)' in current[1791]
assert b'BuildingFixture::HeatingReview' in current[1792] and b'SceneBuildingId(index as u64 + 1)' in current[1792]
review = json.loads(sources['assets/tactical-scenes/heating-review.review.json'])
views = {v['slug']:v['building'] for v in review['views']}
assert views['town-upper-hearth'] == 8 and views['merchant-upper-hearth'] == 9
assert b'assets/tactical-grounding/goslar-1238.json' in current[1793]
transport = json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(581,[1790]),(582,[1791,1792,1793]),(583,[1794])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for row in receipt.get('qualified_supporting_same_lens_reuse',[]):
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
q = load('RECONCILIATION-059.json')['entries']
for i in (63,1465,1466,1468,1469,1470,1471,1472,1473):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-ACTION-STATE-95E.json' in q[63]['current_reconciliation']
assert 'ASSESSMENT-TURN-INTENT-95E.json' in q[1465]['current_reconciliation']
assert 'ASSESSMENT-TURN-INTENT-95E.json' in q[1466]['current_reconciliation']
findings = a['conditional_findings']
assert {f['id'] for f in findings} == {'REVIEW-HOME-PROPERTY-ID-DECODE-ADMISSION-001','Q-L12-c212b0e211a52d755161'}
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in findings:
    assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
    assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, prior receipts, home identity source and pinned Serde member; no product execution.')
