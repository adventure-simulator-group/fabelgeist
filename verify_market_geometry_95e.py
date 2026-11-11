#!/usr/bin/env python3
"""Verify market geometry, historical receipt and source provenance only."""
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
name = 'ASSESSMENT-MARKET-GEOMETRY-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1857-1859.json'
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
lengths = {1857:164, 1858:290, 1859:23}
ranges = {1857:[[35,92]], 1858:[], 1859:[]}
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
market = current[1857]
assert b'Self::Market { corners_metres, .. } => corners_metres.into_iter().all(Vec2::is_finite)' in market
assert b'if cross == 0.0 {' in market and b'continue;' in market
assert b'convex_quad_contains(corners_metres, point)' in market
assert b'pub struct MeleeAttackKey(u64)' in current[1859]
assert b'including zero; absence is Option' in current[1859]
assert b'Self::NotNeeded | Self::CannotArrive => 0.0' in current[1858]
assert b'urban::validate(self)?' in sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'input.streets.iter().any(|street| !street.is_valid())' in sources['crates/adventuresim-tactical-core/src/scene_input/urban.rs']
assert b'SPATIAL_BUCKET_METRES: f32 = 32.0' in sources['crates/adventuresim-tactical-core/src/city_layout.rs']
urban = sources['crates/adventuresim-tactical-core/src/scene/ground/urban.rs']
assert b'.any(|&index| self.streets[index].contains(point.metres()))' in urban
assert b'CityStreetPatch::Market { corners_metres, .. } => corners_bounds(corners_metres)' in urban
assert b'CityStreetSurface::Fieldstone => GroundSubstrate::Road' in urban
assert b'cover: GroundCover::Bare' in urban
assert b'.surface_at(point)' in sources['crates/adventuresim-tactical-core/src/scene/ground.rs']
assert b'street.contains(self.access[0].start_metres())' in sources['crates/adventuresim-tactical-core/src/city_layout/gardens/geometry.rs']
assert b'BuildingOrientation::from_frontage_tangent(corners[1] - corners[0])?' in sources['crates/adventuresim-tactical-core/src/scene_input/furniture/candidates/market.rs']
assert b'corner| market.contains(corner)' in sources['crates/adventuresim-tactical-core/src/scene_input/furniture/placement.rs']
native = a['native_source_evidence']; archive = (root / native['archive']).read_bytes()
assert sha(archive) == native['archive_sha256']
with tarfile.open(root / native['archive'], 'r:gz') as tar:
    body = tar.extractfile(native['member']).read()
assert sha(body) == native['sha256'] and len(body) == native['bytes']
assert b'(self.x * rhs.y) - (self.y * rhs.x)' in body
assert b'if rcp.is_finite() && rcp > 0.0' in body
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
for n, indices in [(611,[1857,1858,1859]), (645,[1936,1937,1938]), (653,[1959,1960])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        if n == 611:
            assert row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]]
            assert row['same_lens_reuse'] == []
assert 'ZERO body coverage' in json.dumps(json.loads(records['segovb/READ-L12-653.json']))
for n, indices in [(294,[1924,1925,1926,1927,1928,1929]), (297,[1945,1946,1947,1948])]:
    path = f'adler/READ-L11-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources'] + receipt.get('connected_reinspection',[]):
        registration = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
        assert registration in records and row['registration_sha256'] == lookup[registration]['sha256']
for path, prior in [('adler/READ-L11-282.json','ASSESSMENT-MOVEMENT-REPORT-95E.json'), ('adler/READ-L11-290.json','ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json'), ('adler/READ-L11-299.json','ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json')]:
    rec = next(x for x in load(prior)['recovered_historical_records'] if x['original_path'] == path)
    body = contents(rec['preserved_bytes'])
    assert sha(body) == rec['published_sha256'] and rec['sha256'] == lookup[path]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in (105,1545,1546,1589,1606):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
finding = a['conditional_findings'][0]
assert finding['id'] == 'ADLER-L11-CITY-PATCH-GEOMETRY-ADMISSION'
assert finding['accepted_historical_unique_count_change'] == 0 and 'were not constructed or executed' in finding['trace']
assert 'local to bucket (0,0), not a whole-map result' in finding['safeguards_and_limits']
original = load('baseline-059/ledger.json')['discovery_findings_and_dependencies'][finding['id']]
assert 'four' in json.dumps(original).lower() and 'bucket' in json.dumps(original).lower()
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical roles, original qualified receipts and bucket-limited market trace; no product execution.')
