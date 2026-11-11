#!/usr/bin/env python3
"""Verify the seventh delegated evidence packet, never product behavior.

An archived source object or a prospective registration proves neither a
personal read nor later-checkpoint coverage. Path-redacted copies are identified
as reconstructions, and all older source and reviewer attribution is retained.
"""
import gzip
import hashlib
import io
import json
import re
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()

def oid(kind, body):
    header = kind.encode() + b' ' + str(len(body)).encode() + b'\0'
    return hashlib.sha1(header + body).hexdigest()

def entries(body):
    result = {}
    pos = 0
    while pos < len(body):
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode().lstrip('0'), body[end+1:end+21].hex())
        pos = end + 21
    assert pos == len(body)
    return result

local = re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-delegate-[^"\'\s]+', re.I)
secret = re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}', re.I)
cp = load('DELEGATED-EVIDENCE-CHECKPOINT-7.json')
assert cp['working_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert cp['historical_revision'] == '6533e444fa37552927070cc8d8190907fd3b413e'
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
lookup = {x['path']: x for x in transport['entries']}
work = load('COVERAGE-L01-ROLES.json')['roles']
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]

expected = {
    'web-residence': ('/root/hair_ops', [2378, 2379, 2380]),
    'web-medical': ('/root/rig_identity', [2376, 2377]),
    'web-settlement': ('/root/mhr_character', [2374, 2382]),
    'web-travel': ('/root/hair_ops', [2381, 2383]),
    'web-query': ('/root/rig_identity', [2385, 2386, 2387, 2388]),
    'web-projection': ('/root/hair_ops', [2384, 2389]),
    'strategic-map': ('/root/mhr_character', [2390, 2391, 2392, 2393]),
    'web-quest-templates': ('/root/rig_identity', [2394, 2395, 2396]),
    'web-character-details': ('/root/mhr_character', [2397, 2398]),
}
by, metadata = {}, {}
for bundle in cp['bundles']:
    name = bundle['bundle_id']
    assert name not in by and (bundle['agent'], bundle['scope_original_roles']) == expected[name]
    raw = (root / bundle['archive']).read_bytes()
    assert sha(raw) == bundle['archive_sha256'] and len(raw) == bundle['archive_bytes']
    members = {x['path']: x for x in bundle['members']}
    assert len(members) == len(bundle['members'])
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as tar:
        files = tar.getmembers()
        assert len(files) == len(members)
        assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files)
        data = {x.name: tar.extractfile(x).read() for x in files}
    assert set(data) == set(members)
    for path, body in data.items():
        member = members[path]
        assert sha(body) == member['published_sha256'] and len(body) == member['published_bytes']
        assert member['content_encoding'] in ('gzip', 'identity')
        plain = gzip.decompress(body) if member['content_encoding'] == 'gzip' else body
        assert sha(plain) == member['published_uncompressed_sha256']
        assert not local.search(plain) and not secret.search(plain), (name, path)
        if member['redacted_nonessential_local_path_occurrences']:
            assert plain.count(b'<LOCAL_PATH_') == member['redacted_nonessential_local_path_occurrences']
            assert 'Reconstructed public copy' in member['qualification']
        else:
            assert sha(body) == member['original_sha256'] and len(body) == member['original_bytes']
            assert sha(plain) == member['original_uncompressed_sha256']
    assert data[bundle['report_archive_member']] == (root / bundle['report_path']).read_bytes()
    by[name], metadata[name] = data, members
assert set(by) == set(expected)
assert sum(len(x['members']) for x in cp['bundles']) == 1189
assert sum(x['redacted_nonessential_local_path_occurrences'] for b in cp['bundles'] for x in b['members']) == 8274

# Match all exact 059 records to the original transport, keeping older
# attribution and original source revisions; these do not restore 3821.
history = [
    ('web-residence', 'historical-manifest.json', 'records', 'original_transport_path', 'copy_path', 37),
    ('web-medical', 'history-manifest.json', 'original_059_records', 'original_path', 'preserved_bytes', 22),
    ('web-settlement', 'historical-059-index.json', 'records', 'original_path', 'copy', 35),
    ('web-travel', 'historical-manifest.json', 'records', 'original_transport_path', 'copy_path', 19),
    ('web-query', 'history-manifest.json', 'original_059_records', 'original_path', 'preserved_bytes', 33),
    ('web-projection', 'historical-manifest.json', 'records', 'original_transport_path', 'copy_path', 22),
    ('strategic-map', 'historical-059-index.json', 'records', 'original_path', 'copy', 20),
    ('web-quest-templates', 'history-manifest.json', 'original_059_records', 'original_path', 'preserved_bytes', 26),
    ('web-character-details', 'historical-059-index.json', 'records', 'original_path', 'copy', 39),
]
assert sum(x[-1] for x in history) == 253
for name, index, key, pathkey, copykey, count in history:
    data = by[name]
    rows = json.loads(data[index])[key]
    assert len(rows) == count
    for item in rows:
        original, body = lookup[item[pathkey]], data[item[copykey]]
        assert not metadata[name][item[copykey]]['redacted_nonessential_local_path_occurrences']
        assert sha(body) == item['sha256'] == original['sha256']
        assert len(body) == item['bytes'] == original['bytes']
        assert oid('blob', body) == item['git_blob'] == original['git_blob']

# Verify byte identities for each version independently, without turning
# byte equality or a prospective scope into an additional personal read.
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for name in ('web-residence', 'web-travel', 'web-projection'):
        data = by[name]
        rows = json.loads(data['registration.json'])['ordered_roles']
        assert sorted(x['original_index'] for x in rows) == sorted(expected[name][1])
        for item in rows:
            assert len(item['versions']) == 2
            for version in item['versions']:
                current = version['revision'] == cp['working_revision']
                assert current or version['revision'] == cp['historical_revision']
                body = data['source/' + ('95e6-' if current else '6533-') + Path(item['path']).name]
                assert version['present'] and sha(body) == version['sha256'] and len(body) == version['bytes']
                assert oid('blob', body) == version['git_blob']
                if current:
                    assert tree[item['path']] == version['git_blob']
                else:
                    assert body == archive.extractfile(work[item['original_index']]['historical_sha256']).read()
    for name in ('web-medical', 'web-query', 'web-quest-templates'):
        data = by[name]
        rows = json.loads(data['registration.json'])['entries']
        assert sorted(set(x['original_role'] for x in rows)) == sorted(expected[name][1]) and len(rows) == 2 * len(expected[name][1])
        for item in rows:
            current = item['revision'] == cp['working_revision']
            assert current or item['revision'] == cp['historical_revision']
            body = data['sources/' + ('95e6/' if current else '6533/') + item['path']]
            assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid('blob', body) == item['git_blob']
            if current:
                assert tree[item['path']] == item['git_blob']
            else:
                assert body == archive.extractfile(work[item['original_role']]['historical_sha256']).read()
    for name in ('web-settlement', 'strategic-map', 'web-character-details'):
        data = by[name]
        rows = json.loads(data['registration.json'])['sources']
        assert sorted(set(x['original_role'] for x in rows)) == sorted(expected[name][1]) and len(rows) == 2 * len(expected[name][1])
        for item in rows:
            body = data[item['archive']]
            assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid('blob', body) == item['git_blob_sha1']
            if item['kind'] == 'current':
                assert item['revision'] == cp['working_revision'] and tree[item['path']] == item['git_blob_sha1']
            else:
                assert item['kind'] == 'historical' and item['revision'] == cp['historical_revision']
                assert body == archive.extractfile(work[item['original_role']]['historical_sha256']).read()

# Three packets include raw commit/tree proofs for each exact path and older
# 774 receipts. Previous current-agent material keeps its original author.
for name in ('web-medical', 'web-query', 'web-quest-templates'):
    data, members = by[name], metadata[name]
    proof = json.loads(data['git-proofs.json'])
    objects = {}
    for object_id, item in proof['objects'].items():
        body = gzip.decompress(data[item['preserved_bytes']])
        assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid(item['type'], body) == object_id
        objects[object_id] = body
    for rev, item in proof['commits'].items():
        assert rev == item['commit_object'] and objects[rev].splitlines()[0] == b'tree ' + item['root_tree'].encode()
    for item in proof['paths']:
        cur = proof['commits'][item['revision']]['root_tree']
        assert [step['name'] for step in item['path_chain']] == item['path'].split('/')
        for step in item['path_chain']:
            assert step['tree'] == cur and entries(objects[cur])[step['name']] == (step['mode'].lstrip('0'), step['object'])
            cur = step['object']
        assert cur == item['git_blob']
    index = json.loads(data['history-manifest.json'])
    for item in index['older_774_receipts']:
        pathproof = next(p for p in proof['paths'] if p['revision'] == item['transport_revision'] and p['path'] == item['transport_path'])
        member = members[item['preserved_bytes']]
        assert item['sha256'] == pathproof['sha256'] == member['original_sha256']
        assert item['bytes'] == pathproof['bytes'] == member['original_bytes'] and item['git_blob'] == pathproof['git_blob']
        if not member['redacted_nonessential_local_path_occurrences']:
            assert oid('blob', data[item['preserved_bytes']]) == item['git_blob']
    for item in index['prior_current_agent_records']:
        member = members[item['preserved_bytes']]
        assert item['previous_agent'] and item['sha256'] == member['original_sha256'] and item['bytes'] == member['original_bytes']
        if not member['redacted_nonessential_local_path_occurrences']:
            assert sha(data[item['preserved_bytes']]) == item['sha256']

# Three other packets use a second raw Git-proof schema and distinguish the
# original 774 local record from its separately normalized exported variant.
for name in ('web-settlement', 'strategic-map', 'web-character-details'):
    data, members = by[name], metadata[name]
    proof = json.loads(data['git-proofs.json'])
    objects = {}
    for object_id, item in proof['objects'].items():
        body = gzip.decompress(data[item['archive']])
        assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid(item['type'], body) == object_id
        objects[object_id] = body
    for item in proof['proofs']:
        assert objects[item['revision']].splitlines()[0] == b'tree ' + item['commit_tree'].encode()
        cur = item['commit_tree']
        assert [step['name'] for step in item['chain']] == item['path'].split('/')
        for step in item['chain']:
            assert step['tree'] == cur and entries(objects[cur])[step['name']] == (step['mode'].lstrip('0'), step['object'])
            cur = step['object']
        assert cur == item['blob'] == oid('blob', data[item['archive']])
    manifest = data['older-774-export-manifest.json']
    assert sha(manifest) == '0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5'
    rows = {x['file']: x for x in json.loads(manifest)['files']}
    for item in json.loads(data['older-774-index.json'])['records']:
        record = rows['records/' + item['path']]
        assert record['original_record_sha256'] == item['original_sha256'] == members[item['original_copy']]['original_sha256']
        assert record['published_export_sha256'] == item['normalized_export_sha256'] == sha(data[item['normalized_copy']])
        assert item['distinct'] == (item['original_sha256'] != item['normalized_export_sha256'])

assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch7_pending_central_reconciliation'] == 'DELEGATED-EVIDENCE-CHECKPOINT-7.json'
print('Verified 1189 members, 253 exact 059 records, original source bodies and Git proofs; no product execution.')
