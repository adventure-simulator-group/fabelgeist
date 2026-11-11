#!/usr/bin/env python3
"""Verify client-source and recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-CLIENT-ENDPOINT-INPUT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1973-1976.json'
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
lengths = {1973:1514, 1974:178, 1975:475, 1976:242}
prior = {1973:'bounded-current-trace-only', 1974:'unassessed', 1975:'bounded-current-trace-only', 1976:'unassessed'}
ranges = {1973:[[233,261],[626,687]], 1974:[[40,178]], 1975:[], 1976:[[1,242]]}
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
        assert c['current_personal_ranges'] == ranges[i]
        if i == 1976:
            assert not c['byte_identical'] and c['current_content_read_via_identical_historical_ranges'] == []
            before_lines, current_lines = before.splitlines(), body.splitlines()
            differences = [(j+1,x,y) for j,(x,y) in enumerate(zip(before_lines,current_lines)) if x != y]
            assert len(before_lines) == len(current_lines) and differences == [(14,b'use crate::FIXED_TIMESTEP_HZ;',b'use crate::TACTICAL_TICK_HZ;'), (31,b'        app.insert_resource(Time::<Fixed>::from_hz(FIXED_TIMESTEP_HZ))',b'        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TACTICAL_TICK_HZ)))')]
        else:
            assert before == body and c['byte_identical'] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
client = current[1973]
assert b'if server_url.contains("://")' in client and b'.starts_with("wss://")' in client and b'ClientConfig::builder().with_no_encryption()' in client
assert b'install_runtime_snapshot' in client and b'prepare_attack_input' in client and b'checked_add' in current[1974]
assert b'scene_digest: String' in current[1975] and b'.replicate_once::<TerrainLandformRecipe>()' in current[1976]
cargo_record = next(x for x in load('ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json')['current_supporting_sources'] if x['path'] == 'Cargo.lock')
cargo = contents(cargo_record['preserved_bytes'])
assert oid('blob', cargo) == tree['Cargo.lock'] and sha(cargo) == cargo_record['sha256']
packages = tomllib.loads(cargo.decode())['package']; native = {}; package_rows = {}
for item in a['native_source_archives']:
    archive = (root / item['archive']).read_bytes()
    assert len(archive) == item['bytes'] and sha(archive) == item['sha256']
    with tarfile.open(root / item['archive'], 'r:gz') as pack:
        for member in item['members']:
            body = pack.extractfile(member['member']).read()
            assert sha(body) == member['sha256'] and len(body) == member['bytes'] and len(body.splitlines()) == member['lines']
            assert all(1 <= lo <= hi <= member['lines'] for lo,hi in member['personally_inspected_ranges'])
            native[member['member']] = body
for pkg, version in [('aeronet_websocket','0.21.0'), ('tokio-tungstenite','0.26.2'), ('tungstenite','0.26.2'), ('http','1.5.0')]:
    row = next(x for x in packages if x['name'] == pkg and x['version'] == version)
    record = next(x for x in a['native_source_archives'] if x['package'] == pkg+'-'+version)
    assert row['checksum'] == record['sha256']; package_rows[pkg] = row
assert 'tokio-tungstenite 0.26.2' in package_rows['aeronet_websocket']['dependencies']
assert 'tungstenite 0.26.2' in package_rows['tokio-tungstenite']['dependencies']
assert 'http' in package_rows['tungstenite']['dependencies']
assert b'target.into_client_request()' in native['aeronet_websocket-0.21.0/src/client/mod.rs']
assert b'self.with_connector(Connector::Plain)' in native['aeronet_websocket-0.21.0/src/client/config.rs']
assert b'tokio_tungstenite::connect_async_tls_with_config(' in native['aeronet_websocket-0.21.0/src/client/backend.rs']
assert b'self.parse::<Uri>()?.into_client_request()' in native['tungstenite-0.26.2/src/client.rs']
assert b'Some("wss") => Ok(Mode::Tls)' in native['tungstenite-0.26.2/src/client.rs']
assert b'Scheme2::Other(i)' in native['http-1.5.0/src/uri/scheme.rs'] and b'Other(ref v) => &v[..]' in native['http-1.5.0/src/uri/scheme.rs']
assert b'let mut scheme = s.split_to(n + 3)' in native['http-1.5.0/src/uri/mod.rs'] and b'Scheme2::Other(Box::new(val))' in native['http-1.5.0/src/uri/mod.rs']
connect = native['tokio-tungstenite-0.26.2/src/connect.rs']
assert b'Some("wss") => Some(443)' in connect and b'Some("ws") => Some(80)' in connect
assert connect.index(b'.ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?') < connect.index(b'TcpStream::connect(addr).await')
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
for n, indices in [(660,[1973]),(661,[1974,1975]),(662,[1976])]:
    path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']] == indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
for n, indices in [(303,[1973,1974]),(304,[1975,1976])]:
    path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [r['original_index'] for r in z['sources']] == indices and z['current_reconciled'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources'] + z.get('connected_reinspection',[]):
        p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
        assert p in records and row['registration_sha256'] == lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
        assert p in records and row['sha256'] == lookup[p]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in range(1621,1631):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings']) == 1 and a['conditional_findings'][0]['id'] == 'REVIEW-NATIVE-WEBSOCKET-SCHEME-CASE-001'
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == a['conditional_findings'][0]['id'] for x in ledger) == 1
assert next(x for x in ledger if x['id'] == a['conditional_findings'][0]['id'])['record'] == name
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified four historical client bodies, attributed receipts and pinned endpoint source; no parser, network or product execution.')
