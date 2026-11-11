#!/usr/bin/env python3
"""Verify qualified fixture, historical receipt and source provenance only."""
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
name = 'ASSESSMENT-GROUNDING-FIXTURES-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1849-1851.json'
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
lengths = {1849:912, 1850:309, 1851:289}
ranges = {1849:[[860,912]], 1850:[], 1851:[]}
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
assert b'error.location_metres.attempted_metres().x.to_bits()' in current[1849]
assert b'east.to_bits()' in current[1849] and b'GeometryError::NonFinite' in current[1849]
assert b'if width == 0.0' in current[1850] and b'SupportCollision::ContactOnly' in current[1850]
assert b'foundation.volume_cubic_metres() - area * 0.2' in current[1851]
assert b'SupportConstraint::SurfaceOverlap' in current[1851] and b'SupportConstraint::CutFill' in current[1851]
for ident, count in [('1238', 4), ('965', 8)]:
    fixture = json.loads(sources[f'assets/tactical-grounding/goslar-{ident}.json'])
    assert len(fixture['geographic_triangles']) == count
    solution = fixture['doorway_solution']
    assert solution['maximum_grade'] == 0.65 and solution['minimum_going_m'] == 0.25
    assert solution['maximum_displacement_m'] == 6.0 and solution['maximum_riser_m'] == 0.19
    assert fixture['provenance']['baseline'] == '5745fdca9d002438dc44a312ed17189fd0e846f3'
collider = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/collider.rs']
assert b'Self::ContactOnly => None' in collider and b'Self::Solid(collider) => Some(collider)' in collider
policy = sources['crates/adventuresim-tactical-core/src/city_layout/compiled/support/policy.rs']
assert b'positive_metres(0.19)' in policy and b'positive_metres(0.25)' in policy
server = sources['crates/adventuresim-tactical-server/src/boundaries.rs']
assert b'if let Some(collider) = collider.into_solid()' in server
enc = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/enclosure.rs']
assert b'let Some(faces) = super::admission::solid_faces' in enc and b'SharedShape::convex_mesh' in enc
assert b'.into_solid()\n            .into_iter()' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/replacement.rs']
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
for n, indices in [(606,[1849]), (607,[1850,1851])]:
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
for path, prior in [('adler/READ-L11-279.json','ASSESSMENT-GEOGRAPHY-GRADE-95E.json'), ('adler/READ-L11-280.json','ASSESSMENT-FOUNDATION-VOLUME-95E.json')]:
    previous = load(prior)
    rec = next(x for x in previous['recovered_historical_records'] if x['original_path'] == path)
    body = contents(rec['preserved_bytes'])
    assert sha(body) == rec['published_sha256'] and rec['sha256'] == lookup[path]['sha256']
    receipt = json.loads(body)
    assert [r['original_index'] for r in receipt['sources']] == ([1849] if '279' in path else [1850,1851])
q = load('RECONCILIATION-059.json')['entries']
for i in [102,260,1506,*range(1537,1542)]:
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ('REVIEW-BOUNDARY-WIRE-CONTACT-ONLY-001','ADLER-L11-FOUNDATION-VOLUME-ARITHMETIC','SEGOVB-L12-APRON-FLOOR-INTERVAL-ELEVATION-ADMISSION'):
    assert sum(x['id'] == ident and name in x.get('related_current_evidence',[]) for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical roles, separately attributed receipts, fixture bytes and contact-only source trace; no product execution.')
