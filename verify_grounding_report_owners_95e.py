#!/usr/bin/env python3
"""Verify grounding-report and recovered receipt provenance without executing product code."""
import gzip, hashlib, json, tarfile, tomllib
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
        assert oid('tree', raw) == current == part['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body) and sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-GROUNDING-REPORT-OWNERS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1982-1986.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1982:176, 1983:328, 1984:153, 1985:204, 1986:122}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body


assert all((root / p).is_file() for p in a['prior_current_evidence'])
export, inspect, footprint, vista, gate = (current[i] for i in lengths)
assert b'.expect("validated property corner")' in export and b'ImportedTerrainCapture::sample_points' in export and b'build_imported_scene(' in export
assert b'representation: &str' in inspect and b'"playable"' in inspect and b'"distant"' in inspect and b'"representation": representation' in inspect
assert b'&comparison["candidates"][1]["front_court_rear_elevations_m"]' in inspect and b'ProposedCompoundLevels::from_capture(' in inspect
terraces = sources['crates/adventuresim-tactical-server-dispatcher/examples/inspect_city_grounding/terraces.rs']
solutions = sources['crates/adventuresim-tactical-server-dispatcher/examples/inspect_city_grounding/solutions.rs']
assert b'candidate("level_courtyard_and_both_buildings",[front_floor;3])' in terraces
assert b'candidate("stepped_court_with_level_central_landing",[front_floor,court,rear])' in terraces
assert b'let values: &[Value; 3] = value.as_array()?.as_slice().try_into().ok()?' in solutions
assert b'SupportElevation::from_metres(value.as_f64()? as f32)' in solutions and b'pub(super) struct ProposedCompoundLevels' in solutions
assert b'"complete_playable_coverage":corners.iter().all' in footprint and b'let Some(elevation) = terrain.height_at(point)' in footprint
assert vista.index(b'result.area_m2 += area(&polygon)') < vista.index(b'SupportElevation::from_metres(height_in_triangle')
assert b'let corners = corners.map(|p| p - origin)' in vista and b'cell_rectangles_outside_inner_rectangle(a, b, half)' in vista and b'let point = quad[i] + origin' in vista
assert b'verification_scope":"Endpoint constraint only' in gate
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else:
        assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob']
    records[item['original_path']] = body


for n, indices in [(664,[1982,1983]),(665,[1984,1985,1986]),(667,[1991,1992,1993,1994])]:
    path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']] == indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        if row['original_index'] in lengths:
            assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
    for row in z.get('qualified_supporting_same_lens_reuse',[]) + z.get('qualified_connected_original_reuse',[]):
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
for n, indices in [(305,[1977,1978,1979,1980,1981,1982]),(306,[1983,1984,1985,1986,1987,1988])]:
    path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [r['original_index'] for r in z['sources']] == indices and z['current_reconciled'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources'] + z.get('connected_reinspection',[]):
        p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
        assert p in records and row['registration_sha256'] == lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
        assert p in records and row['sha256'] == lookup[p]['sha256']
original = load('baseline-059/queue.json')['entries']; q = load('RECONCILIATION-059.json')['entries']
ids = [x['id'] for x in a['conditional_findings']]
assert ids == [original[i]['contract_id'] for i in (262,263,264)]
for i in (112,262,263,264,1635,1636,1637,1638,1639,1640,1642,1643,1644,1648):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ids:
    assert sum(x['id'] == ident for x in ledger) == 1 and next(x for x in ledger if x['id'] == ident)['record'] == name
assert name in next(x for x in ledger if x['id'] == 'ADLER-L11-VISTA-PLAYABLE-SEAM-FRAME')['additional_current_evidence']
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical grounding-report bodies, recovered own-lens attribution and current semantic chains; no diagnostic or product execution.')
