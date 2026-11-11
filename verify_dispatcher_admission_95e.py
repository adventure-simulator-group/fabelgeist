#!/usr/bin/env python3
"""Verify dispatcher and recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-DISPATCHER-ADMISSION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1995-1999.json'
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
lengths = {1995:140,1996:105,1997:102,1998:346,1999:57}
prior = {1995:'unassessed',1996:'unassessed',1997:'unassessed',1998:'bounded-current-trace-only',1999:'unassessed'}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == prior[i] and item['prior_current_observations'] == row.get('current_observations', [])
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
validator, composition, materializer, dispatcher, regional = (current[i] for i in lengths)
assert b'base_port: u16' in dispatcher and b'Mutex<u16>' in dispatcher and b'*p += 1;' in dispatcher
assert dispatcher.index(b'*p += 1;') < dispatcher.index(b'let Some(launch) = launch_arguments(request)')
assert dispatcher.index(b'.on_insert(move |_ctx, request|') < dispatcher.index(b'conn.frame_tick()')
assert b'capture.geographic_surface(&input, &stages.ungraded_vista)?' in validator
assert b'projection::check(&grounded, geographic, projection_output)?' in composition
projection = sources['crates/adventuresim-tactical-server-dispatcher/examples/validate_city_support/projection.rs']
assert b'let decoded: CityGroundingProjection = serde_json::from_slice(&encoded)?' in projection and b'&reconstructed != grounded.terrain()' in projection
assert b'if triangles.is_empty()' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/source.rs']
assert b'Self::from_triangles(triangles)' in sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/source/presented.rs']
bindings = sources['crates/adventuresim-stdb-client/src/mod.rs']
assert b'"tactical_server_request" => db_update\n                    .tactical_server_request\n                    .append(__sdk::parse_row_list_as_inserts(table_rows.rows)?)' in bindings
assert b'callbacks.invoke_table_row_callbacks::<TacticalServerRequest>' in bindings and b'self.imp.frame_tick()' in bindings
cargo_record = next(x for x in load('ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json')['current_supporting_sources'] if x['path'] == 'Cargo.lock')
cargo = contents(cargo_record['preserved_bytes'])
assert oid('blob', cargo) == tree['Cargo.lock'] and sha(cargo) == cargo_record['sha256']
packages = tomllib.loads(cargo.decode())['package']; sdk = next(x for x in packages if x['name'] == 'spacetimedb-sdk' and x['version'] == '2.6.1')
native = {}
for item in a['native_source_members']:
    manifest = load(item['manifest']); archive = (root / item['archive']).read_bytes()
    assert len(archive) == manifest['bytes'] and sha(archive) == manifest['sha256'] == item['archive_sha256'] == sdk['checksum']
    with tarfile.open(root / item['archive'], 'r:gz') as pack:
        body = pack.extractfile(item['member']).read()
    assert sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['personally_inspected_ranges'])
    native[item['member']] = body
conn = native['spacetimedb-sdk-2.6.1/src/db_connection.rs']
start = conn.index(b'pub fn advance_one_message(&self)'); end = conn.index(b'async fn get_message',start); portion = conn[start:end]
assert portion.index(b'self.apply_pending_mutations()?') < portion.index(b'self.process_message(msg)')
assert b'while self.advance_one_message()? {}' in conn and b'ParsedMessage::SubscribeApplied {' in conn
start = conn.index(b'fn apply_update('); end = conn.index(b'fn end_connection',start); portion = conn[start:end]
assert portion.index(b'let event = get_event(&mut inner)') < portion.index(b'applied_diff.invoke_row_callbacks')
cache = native['spacetimedb-sdk-2.6.1/src/client_cache.rs']
assert b'entries: Default::default()' in cache and b'inserts.insert(&insert.bsatn, &insert.row)' in cache and b'PendingMutation::AddInsertCallback' in cache
assert b'for row in applied_diff.inserts()' in native['spacetimedb-sdk-2.6.1/src/callbacks.rs']
assert b'if let Some(callback) = sub.on_applied()' in native['spacetimedb-sdk-2.6.1/src/subscription.rs']
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




for n, indices in [(668,[1995,1996,1997]),(669,[1998,1999])]:
    path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']] == indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
    for row in z.get('qualified_supporting_same_lens_reuse',[]) + z.get('qualified_connected_original_reuse',[]):
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
n = 308; path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
assert [r['original_index'] for r in z['sources']] == list(lengths) and z['current_reconciled'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
for row in z['sources'] + z.get('connected_reinspection',[]):
    p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert p in records and row['registration_sha256'] == lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
    assert p in records and row['sha256'] == lookup[p]['sha256']
q = load('RECONCILIATION-059.json')['entries']; original = load('baseline-059/queue.json')['entries']
assert original[265]['contract_id'] == 'SEGOVB-L12-DISPATCHER-PORT-ALLOCATION-EXHAUSTION'
for i in (265,1649,1650,1651,1653,1654,1655,1656):
    assert q[i]['current_reconciliation'].startswith('partial ') and (name in q[i]['current_reconciliation'] or hist in q[i]['current_reconciliation'])
assert len(a['conditional_findings']) == 1 and a['conditional_findings'][0]['id'] == original[265]['contract_id']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == original[265]['contract_id'] for x in ledger) == 1
assert next(x for x in ledger if x['id'] == original[265]['contract_id'])['record'] == name
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical dispatcher bodies, original port finding and pinned initial-subscription source; no SDK, network or product execution.')
