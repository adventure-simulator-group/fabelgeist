#!/usr/bin/env python3
"""Check delegated recovery evidence and Git objects, never product behavior."""
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


def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()


def entries(data):
    found = {}
    i = 0
    while i < len(data):
        end = data.index(b'\0', i)
        mode, name = data[i:end].split(b' ', 1)
        found[name.decode()] = (mode.decode().lstrip('0'), data[end + 1:end + 21].hex())
        i = end + 21
    assert i == len(data)
    return found


local = re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-delegate-[^"\'\s]+', re.I)
secret = re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}', re.I)
cp = load('DELEGATED-EVIDENCE-CHECKPOINT-3.json')
assert cp['working_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert cp['historical_revision'] == '6533e444fa37552927070cc8d8190907fd3b413e'
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
lookup = {x['path']: x for x in transport['entries']}
work = load('COVERAGE-L01-ROLES.json')['roles']
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    head, path = line.split('\t', 1)
    tree[path] = head.split()[2]

by, meta = {}, {}
for bundle in cp['bundles']:
    name = bundle['bundle_id']  # An agent may produce more than one bundle.
    assert name not in by
    raw = (root / bundle['archive']).read_bytes()
    assert sha(raw) == bundle['archive_sha256'] and len(raw) == bundle['archive_bytes']
    members = {x['path']: x for x in bundle['members']}
    assert len(members) == len(bundle['members'])
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
        files = archive.getmembers()
        assert len(files) == len(members)
        assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files)
        data = {x.name: archive.extractfile(x).read() for x in files}
    assert set(data) == set(members)
    for path, body in data.items():
        item = members[path]
        assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
        plain = gzip.decompress(body) if item['content_encoding'] == 'gzip' else body
        assert sha(plain) == item['published_uncompressed_sha256']
        assert not local.search(plain) and not secret.search(plain)
        if item['redacted_nonessential_local_path_occurrences']:
            assert plain.count(b'<LOCAL_PATH_') == item['redacted_nonessential_local_path_occurrences']
            assert 'Reconstructed public copy' in item['qualification']
        else:
            assert sha(body) == item['original_sha256'] and len(body) == item['original_bytes']
            assert sha(plain) == item['original_uncompressed_sha256']
    assert data[bundle['report_archive_member']] == (root / bundle['report_path']).read_bytes()
    by[name], meta[name] = data, members
assert set(by) == {'mhr-layout', 'math-units', 'physics-contact', 'shell-contact'}
assert sum(len(b['members']) for b in cp['bundles']) == 329

specs = [
    ('mhr-layout', 'historical-059-index.json', 'records', 'path', 'copy', 15),
    ('math-units', 'historical-manifest.json', 'records', 'original_transport_path', 'copy_path', 14),
    ('physics-contact', 'history-manifest.json', 'original_059_records', 'original_path', 'preserved_bytes', 33),
    ('shell-contact', 'historical-059-index.json', 'records', 'path', 'copy', 40),
]
for name, index, key, pathkey, copykey, count in specs:
    data = by[name]
    rows = json.loads(data[index])[key]
    assert len(rows) == count
    for row in rows:
        original, body = lookup[row[pathkey]], data[row[copykey]]
        assert sha(body) == row['sha256'] == original['sha256']
        assert len(body) == row['bytes'] == original['bytes']
        assert oid('blob', body) == row['git_blob'] == original['git_blob']

with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archived:
    for name in ('mhr-layout', 'shell-contact'):
        data = by[name]
        for item in json.loads(data['registration.json'])['sources']:
            body = data[item['source_file']]
            assert sha(body) == item['sha256'] and len(body) == item['bytes']
            assert oid('blob', body) == item['git_blob_sha1']
            if item['kind'] == 'historical':
                assert body == archived.extractfile(work[item['original_role_index']]['historical_sha256']).read()
            else:
                assert tree[item['path']] == item['git_blob_sha1']
    data = by['math-units']
    for item in json.loads(data['registration.json'])['ordered_roles']:
        body = data['source/' + Path(item['path']).name]
        assert sha(body) == item['sha256'] and len(body) == item['bytes']
        assert oid('blob', body) == item['git_blob'] == tree[item['path']]
        assert body == archived.extractfile(work[item['original_index']]['historical_sha256']).read()
    data = by['physics-contact']
    for item in json.loads(data['registration.json'])['entries']:
        body = data['sources/' + ('95e6/' if item['revision'].startswith('95e6') else '6533/') + item['path']]
        assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid('blob', body) == item['git_blob']
        if item['revision'].startswith('95e6'):
            assert tree[item['path']] == item['git_blob']
        else:
            assert body == archived.extractfile(work[item['original_role']]['historical_sha256']).read()

# The two MHR-authored bundles retain uncompressed commit/tree objects and
# exact path chains for role and support bodies.
for name in ('mhr-layout', 'shell-contact'):
    data = by[name]
    for proof in json.loads(data['git-proofs.json'])['proofs']:
        commit = data['git-objects/' + proof['revision'] + '.commit']
        assert oid('commit', commit) == proof['revision']
        assert commit.splitlines()[0] == b'tree ' + proof['commit_tree'].encode()
        current = proof['commit_tree']
        assert [x['entry'] for x in proof['membership']] == proof['path'].split('/')
        for step in proof['membership']:
            assert step['parent_tree'] == current
            body = data['git-objects/' + current + '.tree']
            assert oid('tree', body) == current
            assert entries(body)[step['entry']] == (step['mode'].lstrip('0'), step['object'])
            current = step['object']
        assert current == proof['blob_sha1'] == oid('blob', data[proof['source_file']])

# Physics includes the complete f82 transport commit/tree paths, including
# distinct adler-original and historical-family variants. Verify both, even
# where privacy redaction means the exact original bytes are unavailable here.
data, metadata = by['physics-contact'], meta['physics-contact']
proofs = json.loads(data['git-proofs.json'])
objects = {}
for object_id, item in proofs['objects'].items():
    body = gzip.decompress(data[item['preserved_bytes']])
    assert sha(body) == item['sha256'] and len(body) == item['bytes']
    assert oid(item['type'], body) == object_id
    objects[object_id] = body
for revision, item in proofs['commits'].items():
    assert revision == item['commit_object']
    assert objects[revision].splitlines()[0] == b'tree ' + item['root_tree'].encode()
for proof in proofs['paths']:
    current = proofs['commits'][proof['revision']]['root_tree']
    assert [x['name'] for x in proof['path_chain']] == proof['path'].split('/')
    for step in proof['path_chain']:
        assert current == step['tree']
        assert entries(objects[current])[step['name']] == (step['mode'].lstrip('0'), step['object'])
        current = step['object']
    assert current == proof['git_blob']
for row in json.loads(data['history-manifest.json'])['older_774_receipts']:
    proof = next(x for x in proofs['paths'] if x['revision'] == row['transport_revision'] and x['path'] == row['transport_path'])
    member = metadata[row['preserved_bytes']]
    assert row['sha256'] == proof['sha256'] == member['original_sha256']
    assert row['bytes'] == proof['bytes'] == member['original_bytes']
    assert row['git_blob'] == proof['git_blob']
    if not member['redacted_nonessential_local_path_occurrences']:
        assert oid('blob', data[row['preserved_bytes']]) == row['git_blob']

# Other copies bind to the earlier transfer manifest; an original digest in a
# redacted member is a prepublication attestation, not a hash of public bytes.
for name in ('mhr-layout', 'shell-contact'):
    data, metadata = by[name], meta[name]
    index = json.loads(data['older-774/INDEX.json'])
    manifest = data[index['export_manifest_copy']]
    assert sha(manifest) == index['export_manifest_sha256'] == '0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5'
    manifest_rows = {x['file']: x for x in json.loads(manifest)['files']}
    for row in index['records']:
        original = metadata[row['original_copy']]
        assert row['manifest_entry'] == manifest_rows[row['manifest_entry']['file']]
        assert original['original_sha256'] == row['original_sha256'] == row['manifest_entry']['original_record_sha256']
        if row['export_differs']:
            assert sha(data[row['normalized_copy']]) == row['normalized_sha256'] == row['manifest_entry']['published_export_sha256']

# The shell packet calls these local inputs "original" in the sense of the
# 774 export manifest. The f82 tree separately proves that those bytes were
# carried under historical-family-execution, while the path-normalized
# bytes also occur in the distinct adler-original tree. Preserve both facts.
qual = load('DELEGATED-CHECKPOINT-3-QUALIFICATION.json')
for pair in qual['shell_older_variant_pairs']:
    shell = meta['shell-contact'][pair['shell_local_original_copy']]
    normalized = by['shell-contact'][pair['shell_path_normalized_copy']]
    historical = next(x for x in proofs['paths'] if x['path'] == pair['historical_family_f82_path'])
    adler = next(x for x in proofs['paths'] if x['path'] == pair['adler_original_f82_path'])
    assert shell['original_sha256'] == historical['sha256'] == pair['historical_family_sha256']
    assert sha(normalized) == adler['sha256'] == pair['adler_original_sha256']
    assert historical['git_blob'] != adler['git_blob']

assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch3_pending_central_reconciliation'] == 'DELEGATED-EVIDENCE-CHECKPOINT-3.json'
print('Verified 329 delegated members, 102 original 059 records, role sources and Git path proofs; no product execution.')
