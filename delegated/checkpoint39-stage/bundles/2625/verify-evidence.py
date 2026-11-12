#!/usr/bin/env python3
"""Verify this packet's evidence only; read-only, no product or old verifier."""
import hashlib, json
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
def load(path):
    return json.loads((root / path).read_bytes())
def tree_entries(raw):
    rows = {}
    i = 0
    while i < len(raw):
        j = raw.index(b' ', i)
        k = raw.index(b'\0', j)
        name = raw[j+1:k].decode()
        assert name not in rows
        rows[name] = (raw[i:j].decode(), raw[k+1:k+21].hex())
        i = k + 21
    return rows

manifest = load('MANIFEST.json')
seen = {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()} - {'MANIFEST.json'}
assert seen == set(manifest['files'])
for path, expected in manifest['files'].items():
    body = (root / path).read_bytes()
    assert len(body) == expected['bytes'] and sha(body) == expected['sha256'], path

reg = load('registration.json')
receipt = load('read-receipt.json')
proof = load('git-proofs.json')
assert reg['role'] == 2625 and receipt['agent'] == reg['agent'] == '/root/hair_ops'
assert receipt['registration_sha256'] == sha((root / 'registration.json').read_bytes())
assert datetime.fromisoformat(reg['registered_utc']) < datetime.fromisoformat(receipt['recorded_utc'])
assert len(reg['rows']) == len(receipt['reads']) == len(proof['paths']) == 10
objects = {}
for p in (root / 'git-objects').iterdir():
    oid, typ = p.name.split('.')
    raw = p.read_bytes()
    assert hashlib.sha1(typ.encode() + b' ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid
    objects[oid] = (typ, raw)
proofs = {(x['revision'], x['path']): x for x in proof['paths']}
reads = {x['tag']: x for x in receipt['reads']}
assert len(proofs) == len(reads) == 10
for row in reg['rows']:
    r = reads[row['tag']]
    for k in ['revision', 'path', 'sha256', 'git_blob', 'retained_body']:
        assert r[k] == row[k]
    assert r['actual_display_ranges'] == row['planned_display_ranges']
    p = proofs[(row['revision'], row['path'])]
    typ, commit = objects[row['revision']]
    assert typ == 'commit' and commit.split(b'\n', 1)[0] == b'tree ' + p['root_tree'].encode()
    cur = p['root_tree']
    assert '/'.join(x['name'] for x in p['chain']) == row['path']
    for step in p['chain']:
        assert step['tree'] == cur
        typ, body = objects[cur]
        assert typ == 'tree' and tree_entries(body)[step['name']] == (step['mode'], step['object'])
        cur = step['object']
    typ, body = objects[cur]
    assert typ == 'blob' and cur == row['git_blob'] == p['blob']
    assert body == (root / row['retained_body']).read_bytes()
    assert sha(body) == row['sha256'] == p['sha256'] and len(body) == row['bytes'] == p['bytes']
    assert len(body.splitlines()) == row['actual_eof']
    covered = set()
    for lo, hi in r['actual_display_ranges']:
        assert 1 <= lo <= hi <= row['actual_eof']
        covered.update(range(lo, hi + 1))
    for lo, hi in r['unread_ranges']:
        assert 1 <= lo <= hi <= row['actual_eof']
        assert not covered.intersection(range(lo, hi + 1))
        covered.update(range(lo, hi + 1))
    assert covered == set(range(1, row['actual_eof'] + 1))

expected = '5be7e371146dbc5fa6df7699431643018b788550460a84cfc29faf6e32bea46e'
assert reg['original_registry_record']['sha256'] == expected
assert reg['original_registry_record']['material_ranges'] == [[1, 54]]
assert all(sha((root / ('source/' + x + '.rs')).read_bytes()) == expected for x in ['old-transport', 'old-6533', 'current'])
index = load('historical/INDEX.json')
for x in index['records']:
    body = (root / x['retained']).read_bytes()
    assert len(body) == x['bytes'] and sha(body) == x['sha256']
selection = load('historical/059-reconstructed-selection.json')
assert selection['queue']['index'] == 2943
assert [x['index'] for x in selection['ledger']['entries']] == [157, 208]
assert selection['queue']['entry']['contract_id'] == 'L12-CLOSURE-WRAPPED-TASSET-VALIDATOR-CLOSING-SCOPE'
assert load('REPORT.json')['personal_read']['receipt_sha256'] == sha((root / 'read-receipt.json').read_bytes())
print('Evidence verified: manifest bytes, ten Git commit/tree/blob paths, registration/receipt scopes, selected historical bytes and reconstructed-record labels.')
print('The hash checks do not establish actual display, external original custody, historical attribution beyond the retained records, or product behavior.')
