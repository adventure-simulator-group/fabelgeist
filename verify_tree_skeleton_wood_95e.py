#!/usr/bin/env python3
"""Verify historical source identity, partial scopes and source-only comparisons."""
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
a = load('ASSESSMENT-TREE-SKELETON-WOOD-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1598-1599.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-SKELETON-HISTORICAL-GIT-PROOFS.json')
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

assert [x['original_index'] for x in h['historical_source_roles']] == [1598, 1599]
assert [x['lines'] for x in h['historical_source_roles']] == [1358, 1370]
old_bodies, current_bodies = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i, n = x['original_index'], x['lines']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1598-1599.json'
        assert x['new_personal_read_ranges'] == [[1,250],[251,500],[501,750],[751,1000],[1001,1200],[1201,n]]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == n
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is False and original != body
        assert c['current_personal_ranges'] == ({1598:[[330,402],[580,627],[710,773]],1599:[]}[i])
        check_chain(c['path'], proof['paths'][c['path']], original)
        diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
        d = c['exact_diff']
        assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        old_bodies[i], current_bodies[i] = original, body

support = {}
assert [x['population_index'] for x in a['earlier_historical_supporting_sources']] == [6,7,11,127,128,129]
for x in a['earlier_historical_supporting_sources']:
    body = contents(x['preserved_bytes'])
    path = x['source_id'].split(':', 1)[1]
    assert x['revision'] == proof['revision'] and sha(body) == x['sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
    assert oid('blob', body) == x['git_blob'] and x['additional_original_frontier_credit'] == 0
    assert x['earlier_reviewer'] == 'Adler Lens11 READ-L11-225'
    check_chain(path, proof['paths'][path], body)
    if x['population_index'] == 129:
        assert x['earlier_registered_scope'] == x['earlier_fresh_reads'] == [[1,53]]
        assert x['lines'] == 117 and x['new_personal_historical_supporting_ranges'] == [[1,117]]
    else:
        assert x['earlier_registered_scope'] == x['earlier_fresh_reads'] == [[1,x['lines']]]
        assert x['new_personal_historical_supporting_ranges'] == []
    support[x['population_index']] = x
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert x['ranges'] == [[1,223]]
    root_plan = body
old_skeleton, new_skeleton = old_bodies[1598], current_bodies[1598]
assert b'use libm::{cosf, powf, sinf};' in new_skeleton and b'.sin().powi(2)' in old_skeleton and b'powf(sinf(' in new_skeleton
assert b'let stem_tip = *stem_points.last().unwrap();' in old_skeleton and b'let stem_tip = stem_points[6];' in new_skeleton
assert b'append_oak_roots(&mut branches, seed, crown_phase, trunk_points[0], gnarling);' in new_skeleton
assert b'oak_root_points(trunk_points[0] - Vec3::Y * 0.07, root, gnarling)' in old_skeleton
assert b'oak_root_points(trunk_base - Vec3::Y * 0.07, root, gnarling)' in root_plan
assert b'foliage: foliage.filter(|_| index + 1 == segment_count)' in new_skeleton
old_wood, new_wood = old_bodies[1599], current_bodies[1599]
marker = b'#[cfg(test)]\nmod tests {'
assert marker in old_wood and marker in new_wood and old_wood.split(marker)[0] == new_wood.split(marker)[0]
for old, new in [(b'9_540',b'9_556'),(b'15_523',b'15_551'),(b'4_810',b'4_630'),(b'7_099',b'6_811'),(b'1_119',b'1_135'),(b'1_897',b'1_925'),(b'771',b'857'),(b'1_344',b'1_488')]:
    assert old in old_wood.split(marker)[1] and new in new_wood.split(marker)[1]

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
for n, index, eof in [(498,1598,1358),(499,1599,1370)]:
    path=f'segovb/READ-L12-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == [index]
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    row = receipt['rows'][0]
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert len(receipt['partial_sequence']) == 3
    previous = None
    for j, x in enumerate(receipt['partial_sequence']):
        assert x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
        partial = json.loads(records[x['path']])
        assert partial['source_sha256'] == w['roles'][index]['historical_sha256'] and partial['original_index'] == index and partial['actual_EOF'] == eof
        expected_fresh = [[[1,600]],[[601,1200]],[[1201,eof]]][j]
        assert partial['fresh_personal_body_reads'] == expected_fresh and partial['original_credit'] == 0
        assert partial['unread_tail'] == ([[expected_fresh[0][1]+1,eof]] if j < 2 else [])
        if previous is not None:
            assert partial['predecessor'] == previous
        previous = x
for n, index in [(225,1598),(226,1599)]:
    path=f'adler/READ-L11-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == [index]
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources'] + receipt['connected_reinspection']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
    if n == 225:
        for row in receipt['connected_reinspection']:
            s = support[row['population_index']]
            assert s['sha256'] == row['source_sha256'] and s['earlier_registered_scope'] == row['scope'] and s['earlier_fresh_reads'] == row['fresh_reads']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1332,1333))
baseline = load('BASELINE-95E-20261110.json')
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries']) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified two historical bodies, six scoped earlier support identities, exact diffs and partial chains; no product execution.')
