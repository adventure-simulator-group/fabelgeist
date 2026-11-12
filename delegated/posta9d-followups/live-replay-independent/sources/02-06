"""Verify an observed main advance and its frozen predecessor offline.

This checks Git object identities and lockfile JSON only. It does not execute
product code, fetch the release, verify its archive, or credit a lens.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
record = json.loads((root / 'MAIN-ADVANCE-95E.json').read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
old = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
new = '95e63628f9d70b8dd6e6fede64334696c6995009'
assert record['previous_revision'] == old and record['current_revision'] == new
assert baseline['source_revision'] == progress['source_revision'] == new
assert baseline['previous_source_revision'] == progress['predecessor_revision'] == old
assert baseline['transition_evidence'] == 'MAIN-ADVANCE-95E.json'


def checked(item):
    path = (root / item['path']).resolve()
    if not path.is_relative_to(root):
        raise ValueError('evidence path escapes branch')
    raw = path.read_bytes()
    if len(raw) != item['bytes'] or hashlib.sha256(raw).hexdigest() != item['sha256']:
        raise ValueError('evidence file mismatch: ' + item['path'])
    return raw


def object_sha(kind, content):
    return hashlib.sha1(kind.encode() + b' ' + str(len(content)).encode() + b'\0' + content).hexdigest()


def tree_entries(raw, expected):
    nested = {}
    entries = {}
    for line in raw.decode().splitlines():
        header, name = line.split('\t', 1)
        mode, kind, oid = header.split()
        parts = name.split('/')
        if name.startswith('"') or kind not in ('blob', 'commit') or name in entries or any(p in ('', '.', '..') for p in parts):
            raise ValueError('invalid tree entry')
        entries[name] = [mode, kind, oid]
        node = nested
        for part in parts[:-1]:
            node = node.setdefault(part, {})
            if not isinstance(node, dict):
                raise ValueError('tree collision')
        if parts[-1] in node:
            raise ValueError('duplicate tree node')
        node[parts[-1]] = (mode, oid)
    def digest(node):
        content = bytearray()
        def order(item):
            name, value = item
            return name.encode() + (b'/' if isinstance(value, dict) else b'\0')
        for name, value in sorted(node.items(), key=order):
            mode, oid = ('40000', digest(value)) if isinstance(value, dict) else value
            content.extend(mode.encode() + b' ' + name.encode() + b'\0' + bytes.fromhex(oid))
        return object_sha('tree', content)
    if digest(nested) != expected:
        raise ValueError('tree object mismatch')
    return entries


trees = {}
locks = {}
for label, revision in [('previous', old), ('current', new)]:
    commit_item = record['commits'][label]
    tree_item = record['trees'][label]
    if commit_item['content_encoding'] != 'gzip':
        raise ValueError('unknown commit encoding')
    commit = gzip.decompress(checked(commit_item))
    if (len(commit) != commit_item['uncompressed_bytes'] or
            hashlib.sha256(commit).hexdigest() != commit_item['uncompressed_sha256']):
        raise ValueError('commit bytes mismatch')
    if commit_item['git_commit'] != revision or object_sha('commit', commit) != revision:
        raise ValueError('commit identity mismatch')
    headers = commit.split(b'\n\n', 1)[0].splitlines()
    if b'tree ' + tree_item['git_tree'].encode() not in headers:
        raise ValueError('commit tree binding mismatch')
    if label == 'current' and [h for h in headers if h.startswith(b'parent ')] != [b'parent ' + old.encode()]:
        raise ValueError('not a direct child')
    tree_raw = checked(tree_item)
    trees[label] = tree_entries(tree_raw, tree_item['git_tree'])
    lock_item = record['lockfiles'][label]
    lock_raw = checked(lock_item)
    if (object_sha('blob', lock_raw) != lock_item['git_blob'] or
            trees[label].get('world-runtime-release.lock.json') != ['100644', 'blob', lock_item['git_blob']]):
        raise ValueError('lockfile blob mismatch')
    locks[label] = json.loads(lock_raw)
if checked(record['trees']['previous']) != (root / 'CURRENT-TREE.tsv').read_bytes():
    raise ValueError('predecessor tree differs from published inventory')
changes = []
for path in sorted(set(trees['previous']) | set(trees['current'])):
    before, after = trees['previous'].get(path), trees['current'].get(path)
    if before != after:
        changes.append({'path': path, 'previous': before, 'current': after})
if changes != record['tree_changes'] or len(changes) != 1 or changes[0]['path'] != 'world-runtime-release.lock.json':
    raise ValueError('main-advance source delta mismatch')
unchanged = sum(trees['previous'].get(p) == trees['current'].get(p)
                for p in set(trees['previous']) | set(trees['current']))
if unchanged != record['unchanged_tree_entries']:
    raise ValueError('unchanged entry count mismatch')

before, after = locks['previous'], locks['current']
a = {x['path']: x for x in before['files']}
b = {x['path']: x for x in after['files']}
if len(a) != len(before['files']) or len(b) != len(after['files']):
    raise ValueError('duplicate lock member')
summary = {
    'old_schema': before['schema'], 'new_schema': after['schema'],
    'old_release': before['release'], 'new_release': after['release'],
    'old_archive_sha256': before['archive_sha256'], 'new_archive_sha256': after['archive_sha256'],
    'old_archive_size': before['archive_size'], 'new_archive_size': after['archive_size'],
    'added_paths': sorted(b.keys() - a.keys()), 'removed_paths': sorted(a.keys() - b.keys()),
    'changed_existing_paths': sorted(p for p in a.keys() & b.keys() if a[p] != b[p]),
    'unchanged_existing_paths': sorted(p for p in a.keys() & b.keys() if a[p] == b[p]),
}
if summary != record['lockfile_mechanical_summary']:
    raise ValueError('lock comparison mismatch')

snapshots = {}
for label, item in record['predecessor_snapshots'].items():
    raw = gzip.decompress(checked(item))
    if len(raw) != item['uncompressed_bytes'] or hashlib.sha256(raw).hexdigest() != item['uncompressed_sha256']:
        raise ValueError('frozen predecessor mismatch: ' + label)
    snapshots[label] = json.loads(raw)
predecessor = snapshots['predecessor-baseline']
old_progress = snapshots['predecessor-progress']
old_roles = snapshots['predecessor-role-worklist']
if any(doc['source_revision'] != old for doc in (predecessor, old_progress, old_roles)):
    raise ValueError('frozen revision mismatch')
for item in predecessor['source_delta']:
    path = item['path']
    if trees['previous'].get(path) != trees['current'].get(path):
        raise ValueError('predecessor delta also changed: ' + path)
if (baseline['predecessor_delta_paths'] != len(predecessor['source_delta']) or
        progress['predecessor_delta_paths'] != len(predecessor['source_delta']) or
        baseline['predecessor_ledger']['entries'] != len(predecessor['repair_ledger']['entries']) or
        progress['predecessor_ledger_entries'] != len(predecessor['repair_ledger']['entries'])):
    raise ValueError('predecessor obligation count mismatch')
if (len(baseline['lens_names']) != 13 or baseline['lens_status'] != ['incomplete'] * 13 or
        progress['lenses_complete'] != 0 or progress['new_transition_lens_cells_assessed'] != 0):
    raise ValueError('unsupported completion credit')
additional = baseline['additional_source_delta']
if (len(additional) != 1 or additional[0]['path'] != changes[0]['path'] or
        not additional[0]['lens_status'][0].startswith('partial bounded') or
        additional[0]['lens_status'][1:] != ['unassessed'] * 12 or
        progress['new_transition_lens_cells_partial'] != 1):
    raise ValueError('new transition coverage is not correctly qualified')
if (baseline['historical_queue_records_indexed'] != old_progress['historical_queue_records_indexed'] or
        baseline['historical_queue_records_semantically_reconciled'] != old_progress['current_queue_records_semantically_reconciled']):
    raise ValueError('historical queue count mismatch')
print('verified main commit ancestry, both trees and lock blobs, frozen predecessor, and open coverage')
