#!/usr/bin/env python3
"""Verify source provenance and qualified grade/diagnostic evidence only."""
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
        result[name.decode()] = (mode.decode(), body[end+1:end+21].hex())
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
name = 'ASSESSMENT-GEOGRAPHY-GRADE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1823-1830.json'
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
lengths = {1823:248, 1824:146, 1825:115, 1826:98, 1827:46, 1828:261, 1829:242, 1830:67}
ranges = {1823:[[1,126]], 1824:[[1,78]], 1825:[[1,115]], 1826:[[1,38]], 1827:[], 1828:[], 1829:[[122,135]], 1830:[[1,67]]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['prior_current_observations'] == []
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
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
assert b'if lod.width < 2' in current[1823] and b'cell_rectangles_outside_inner_rectangle' in current[1823]
assert b'covered_area_square_metres += area' in current[1824]
assert b'measurement.covered_area_square_metres =' in current[1824]
assert b'if serializer.is_human_readable()' in current[1825]
assert b'if deserializer.is_human_readable()' in current[1825]
assert b'mesh.validate().map_err(D::Error::custom)?' in current[1825]
assert b'Self::Rejected(point) => Vec2::new' in current[1827]
assert b'actual: DiagnosticCount(actual as u64)' in current[1828]
assert b'SupportBound::Minimum => (limit - actual).max(0.0)' in current[1828]
assert b'actual: f64::INFINITY' in current[1828] and b'limit: 0.0' in current[1828]
assert b'let normal = (b - a).cross(c - a);' in current[1829]
assert b'normal.xz().length() / normal.y.abs()' in current[1829]
assert b'.fold(0.0, f32::max)' in current[1829]
assert b'actual: DiagnosticCount::new(actual)' in current[1830]
assert b'actual.and_then(|n| u64::try_from(n).ok())' in current[1830]
validation = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/surface/validation.rs']
assert b'!normal.is_finite() || normal.y == 0.0' in validation
assert validation.index(b'!normal.is_finite() || normal.y == 0.0') < validation.index(b'let measured_grade = self.mesh.maximum_grade();')
garden = sources['crates/adventuresim-tactical-server-dispatcher/examples/compare_city_terrain/gardens.rs']
assert b'"composed_grade_within_controller_bound"' in garden and b'"composed_coverage_complete"' in garden
native = a['native_source_evidence']
for item in native['installed_rust_src']:
    body = contents(item['preserved_bytes'])
    assert len(body) == item['bytes'] and sha(body) == item['sha256']
    assert all(1 <= lo <= hi <= len(body.splitlines()) for lo, hi in item['personally_inspected_ranges'])
    if item['name'] == 'core-f32.rs':
        assert b'Returns the maximum of the two numbers, ignoring NaN.' in body
        assert b'intrinsics::maximum_number_nsz_f32(self, other)' in body
assert b'channel = "nightly-2026-07-13"' in sources['rust-toolchain.toml']
with tarfile.open(root / 'native-evidence/glam-0.32.1/glam-0.32.1.crate', 'r:gz') as archive:
    v3 = archive.extractfile('glam-0.32.1/src/f32/vec3.rs').read()
    v2 = archive.extractfile('glam-0.32.1/src/f32/vec2.rs').read()
    assert b'x: self.y * rhs.z - rhs.y * self.z' in v3
    assert b'math::sqrt(self.dot(self))' in v2
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(598,[1823,1824,1825]), (599,[1826,1827,1828]), (600,[1829,1830])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
reuse = json.loads(records['segovb/READ-L12-600.json'])['qualified_supporting_same_lens_reuse']
assert [x['original_index'] for x in reuse] == [1813,1824,1828]
for row in reuse:
    assert row['fresh_personal_body_reads_this_batch'] == []
    assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
adler274 = json.loads(records['adler/READ-L11-274.json'])
assert [x['original_index'] for x in adler274['sources']] == [1827,1828,1829,1830]
adler279 = json.loads(records['adler/READ-L11-279.json'])
qualified = adler279['connected_reinspection'][0]
assert qualified['original_index'] == 2695 and qualified['fresh_reads'] == [[1,157]]
assert qualified['additional_original_frontier_credit'] == 0 and 'no158..159credit' in qualified['disposition']
assert sha(validation) == qualified['source_sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in [103, *range(1512,1521)]:
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
finding = a['conditional_findings'][0]
assert finding['id'] == 'ADLER-L11-SUPPORT-GRADE-NUMERIC-ADMISSION'
assert finding['accepted_historical_unique_count_change'] == 0
assert 'withdrawn' in finding['safeguards_and_limits'] and 'No input was decoded' in finding['trace']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified eight historical roles, attributed receipts and qualified grade/diagnostic source traces; no product execution.')
