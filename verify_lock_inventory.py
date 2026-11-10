"""Verify preserved lockfile inputs and their mechanical package differences."""
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
report = json.loads((root / 'LOCKFILE-DELTA.json').read_text())
archive = root / 'recovered/original-registry-source-archive/objects.tar.gz'
manifest = json.loads((archive.parent / 'MANIFEST.json').read_text())
originals = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(archive, 'r:gz') as tar:
    inputs = {
        'physical_p16': tar.extractfile(originals[39]).read(),
        'pr_working': tar.extractfile(originals[497]).read(),
    }
inputs['reported_selected'] = (root / 'lock-evidence/selected-Cargo.lock').read_bytes()
inputs['current'] = (root / 'lock-evidence/current-Cargo.lock').read_bytes()
def tree_blob(inventory, expected_sha256, expected_tree, wanted):
    raw = (root / inventory).read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected_sha256:
        raise ValueError('tree inventory mismatch')
    tree = {}
    target = None
    for line in raw.decode().splitlines():
        header, path = line.split('\t', 1)
        mode, kind, oid = header.split()
        if path.startswith('"') or kind not in ('blob', 'commit'):
            raise ValueError('unsupported tree entry')
        if path == wanted:
            target = oid
        parts = path.split('/')
        if any(part in ('', '.', '..') for part in parts):
            raise ValueError('unsafe tree path')
        node = tree
        for part in parts[:-1]:
            node = node.setdefault(part, {})
            if not isinstance(node, dict):
                raise ValueError('tree prefix collision')
        if parts[-1] in node:
            raise ValueError('duplicate tree entry')
        node[parts[-1]] = (mode, oid)
    def git_tree(node):
        content = bytearray()
        def order(item):
            name, value = item
            return name.encode() + (b'/' if isinstance(value, dict) else b'\0')
        for name, value in sorted(node.items(), key=order):
            mode, oid = ('40000', git_tree(value)) if isinstance(value, dict) else value
            content.extend(mode.encode() + b' ' + name.encode() + b'\0' + bytes.fromhex(oid))
        return hashlib.sha1(b'tree ' + str(len(content)).encode() + b'\0' + content).hexdigest()
    if git_tree(tree) != expected_tree or target is None:
        raise ValueError('Git tree proof mismatch')
    return target

for name in ('reported_selected', 'current'):
    identity = report['inputs'][name]
    expected_blob = tree_blob(identity['tree_inventory'], identity['tree_inventory_sha256'], identity['tree_git_sha1'], 'Cargo.lock')
    data = inputs[name]
    actual_blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    if actual_blob != expected_blob:
        raise ValueError('lockfile differs from recorded Git tree')

packages = {}
for name, raw in inputs.items():
    identity = report['inputs'][name]
    if hashlib.sha256(raw).hexdigest() != identity['sha256'] or len(raw) != identity['bytes']:
        raise ValueError('lock input mismatch: ' + name)
    parsed = tomllib.loads(raw.decode('utf-8'))
    rows = parsed['package']
    table = {(x['name'], x['version'], x.get('source', '')): x for x in rows}
    if len(table) != len(rows) or len(rows) != identity['package_records']:
        raise ValueError('lock package inventory mismatch: ' + name)
    packages[name] = table
for comparison in report['comparisons']:
    left = packages[comparison['from']]
    right = packages[comparison['to']]
    keys = sorted(left.keys() | right.keys())
    changes = []
    for key in keys:
        old, new = left.get(key), right.get(key)
        if old != new:
            changes.append({'key': list(key), 'before': old, 'after': new})
    added = sum(x['before'] is None for x in changes)
    removed = sum(x['after'] is None for x in changes)
    modified = len(changes) - added - removed
    if (added, removed, modified) != (comparison['added_records'], comparison['removed_records'], comparison['modified_records']):
        raise ValueError('package delta count mismatch')
    if changes != comparison['changes']:
        raise ValueError('package comparison mismatch')
    if len(left.keys() & right.keys()) - sum(x['before'] is not None and x['after'] is not None for x in changes) != comparison['unchanged_records']:
        raise ValueError('unchanged count mismatch')
print('verified four lockfile inputs and', len(report['comparisons']), 'mechanical comparisons')
