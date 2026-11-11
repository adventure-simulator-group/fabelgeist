#!/usr/bin/env python3
"""Verify scene-materialization and recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-SCENE-MATERIALIZATION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2000.json'
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
lengths = {2000:978}
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
scene = current[2000]
assert b'if let Ok(existing) = fs::read(&path)' in scene and b'if existing == bytes' in scene
assert b'directory.join(format!("scene-{key}.tmp"))' in scene and b'fs::write(&temporary, bytes)' in scene and b'fs::rename(&temporary, &path)' in scene
assert scene.index(b'if let Ok(existing) = fs::read(&path)') < scene.index(b'fs::write(&temporary, bytes)') < scene.index(b'fs::rename(&temporary, &path)')
assert b'const RANDOM_DETAIL_SCALE: u64 = 10_000' in scene and b'const RANDOM_DETAIL_BUCKETS: u64 = RANDOM_DETAIL_SCALE * 2 + 1' in scene
assert b'NonZeroU64::new(RANDOM_DETAIL_BUCKETS).expect("detail range is positive")' in scene
assert b'origin_east_metres: 0.0' in scene and b'origin_north_metres: 0.0' in scene
assert b'Never use `unwrap` or `expect` outside tests' in sources['crates/AGENTS.md']
assert b'channel = "nightly-2026-07-13"' in sources['rust-toolchain.toml']
native = a['native_rust_source']; body = contents(native['preserved_bytes'])
assert sha(body) == native['sha256'] and len(body) == native['bytes'] and len(body.splitlines()) == native['lines']
assert oid('blob', body) == native['github_contents_blob'] == '77be07b097b03a3d671cbf47c923f687c68dbd06'
assert native['source_commit'] == '77cf889bc178ddb44d6a1c78e5a820b5abb31d8d'
assert all(1 <= lo <= hi <= native['lines'] for lo,hi in native['personally_inspected_ranges'])
assert b'Other errors may also be returned according to' in body and b'will entirely replace its contents if it does' in body
assert b'replacing the original file if\n/// `to` already exists' in body
manifest = contents(native['channel_manifest']); assert sha(manifest) == native['channel_manifest_sha256']
channel = tomllib.loads(manifest.decode()); assert str(channel['date']) == '2026-07-13' and '77cf889bc' in channel['pkg']['rust-src']['version']
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



n = 670; path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
assert [r['original_index'] for r in z['rows']] == [2000] and z['current_source_reconciled'] is False
assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
for row in z['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert row['fresh_personal_body_reads'] == [[1,978]]
for row in z.get('qualified_supporting_same_lens_reuse',[]) + z.get('qualified_connected_original_reuse',[]):
    assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert row['fresh_personal_body_reads_this_batch'] == []
n = 309; path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
assert [r['original_index'] for r in z['sources']] == [2000,2001,2002] and z['current_reconciled'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
for row in z['sources'] + z.get('connected_reinspection',[]):
    p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert p in records and row['registration_sha256'] == lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
    assert p in records and row['sha256'] == lookup[p]['sha256']
q = load('RECONCILIATION-059.json')['entries']; original = load('baseline-059/queue.json')['entries']
ids = [x['id'] for x in a['conditional_findings']]
assert ids == [original[114]['contract_id'],original[266]['contract_id'],'REVIEW-MATERIALIZED-SCENE-SHARED-TEMP-RACE-001']
for i in (114,266,1639,1652,1656,1658,1659):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ids:
    assert sum(x['id'] == ident for x in ledger) == 1 and next(x for x in ledger if x['id'] == ident)['record'] == name
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified historical scene body, original findings and pinned Rust filesystem source; no scene materialization or product execution.')
