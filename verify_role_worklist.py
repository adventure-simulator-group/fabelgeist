"""Verify source-role identities and bounded evidence links, not audit closure."""
import collections
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
reg_raw = gzip.decompress((root / work['registry']).read_bytes())
if hashlib.sha256(reg_raw).hexdigest() != work['registry_sha256']:
    raise ValueError('registry hash mismatch')
registry = json.loads(reg_raw)['files']
tree_bytes = (root / work['current_tree_inventory']).read_bytes()
if hashlib.sha256(tree_bytes).hexdigest() != work['current_tree_sha256']:
    raise ValueError('tree inventory hash mismatch')
paths = {}
tree = {}
for line in tree_bytes.decode().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, oid = header.split()
    if path.startswith('"') or path in paths or kind not in ('blob', 'commit'):
        raise ValueError('unsupported or duplicate tree path')
    paths[path] = oid
    parts = path.split('/')
    if any(part in ('', '.', '..') for part in parts):
        raise ValueError('unsafe tree path')
    node = tree
    for part in parts[:-1]:
        node = node.setdefault(part, {})
        if not isinstance(node, dict):
            raise ValueError('tree prefix collision')
    if parts[-1] in node:
        raise ValueError('tree entry collision')
    node[parts[-1]] = (mode, oid)

def git_tree(node):
    content = bytearray()
    def ordering(item):
        name, value = item
        return name.encode() + (b'/' if isinstance(value, dict) else b'\0')
    for name, value in sorted(node.items(), key=ordering):
        mode, oid = ('40000', git_tree(value)) if isinstance(value, dict) else value
        content.extend(mode.encode() + b' ' + name.encode() + b'\0' + bytes.fromhex(oid))
    return hashlib.sha1(b'tree ' + str(len(content)).encode() + b'\0' + content).hexdigest()

if git_tree(tree) != work['current_tree_git_sha1']:
    raise ValueError('Git tree reconstruction mismatch')
archive = root / 'recovered/original-registry-source-archive/objects.tar.gz'
manifest = json.loads((archive.parent / 'MANIFEST.json').read_text())
if hashlib.sha256(archive.read_bytes()).hexdigest() != manifest['objects_archive_sha256']:
    raise ValueError('original source archive mismatch')
object_shas = {item['original']: item['sha256'] for item in manifest['sources']}
roles = work['roles']
if len(roles) != len(registry) or len({x['source_id'] for x in roles}) != len(roles):
    raise ValueError('role population mismatch')
needed = set()
for i, (role, original) in enumerate(zip(roles, registry)):
    if role['original_index'] != i or role['registry_pointer'] != '/files/' + str(i):
        raise ValueError('role index mismatch')
    for role_key, original_key in [('source_id', 'source_id'), ('snapshot', 'snapshot'), ('scope_kind', 'scope_kind'), ('historical_sha256', 'sha256')]:
        if role[role_key] != original.get(original_key):
            raise ValueError('role identity mismatch: ' + str(i))
    if role['historical_object_available'] != (i in object_shas):
        raise ValueError('object-availability mismatch')
    if role['status'] not in ('unassessed', 'bounded-current-trace-only', 'lock-metadata-partial', 'assessed-nonapplicable', 'full-historical-body-trace-open', 'deleted-base-trace-open'):
        raise ValueError('invalid role status')
    if role['status'] not in ('unassessed', 'deleted-base-trace-open'):
        if i not in object_shas or object_shas[i] != role['historical_sha256']:
            raise ValueError('missing or incorrect historical object')
        needed.add(role['historical_sha256'])

objects = {}
with tarfile.open(archive, 'r:gz') as tar:
    for member in tar:
        if member.name in needed:
            raw = tar.extractfile(member).read()
            if hashlib.sha256(raw).hexdigest() != member.name:
                raise ValueError('source object mismatch')
            objects[member.name] = raw
if set(objects) != needed:
    raise ValueError('missing source objects')

def git_blob(raw):
    return hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()

def observed_sources(doc):
    yield from doc.get('source_identities', [])
    yield from doc.get('current_sources', [])
    for observation in doc.get('current_source_observations', []):
        if 'sources' in observation:
            yield from observation['sources']
        else:
            yield observation

cached = {}
for role in roles:
    status = role['status']
    if status == 'bounded-current-trace-only' or (
            status == 'full-historical-body-trace-open' and role.get('current_observations')):
        path = role['current_path']
        if role['source_id'].split(':', 1)[-1] != path or paths.get(path) != role['current_git_blob']:
            raise ValueError('current role path mismatch')
        if git_blob(objects[role['historical_sha256']]) != role['current_git_blob']:
            raise ValueError('historical/current byte identity mismatch')
        if not role['current_observations']:
            raise ValueError('missing bounded evidence link')
        for obs in role['current_observations']:
            name = obs['assessment']
            if Path(name).name != name:
                raise ValueError('unsafe assessment name')
            if name not in cached:
                cached[name] = json.loads((root / name).read_text())
            matched = any(
                source.get('path') == path
                and source.get('git_blob') == role['current_git_blob']
                and source.get('ranges', source.get('range')) == obs['ranges']
                and source.get('revision') in (None, work['source_revision'])
                for source in observed_sources(cached[name])
            )
            if not matched:
                raise ValueError('unmatched bounded evidence link')
    if status == 'lock-metadata-partial':
        if role['assessment'] != 'LOCKFILE-DELTA.json':
            raise ValueError('unexpected lock assessment')
    elif status == 'full-historical-body-trace-open':
        name = role['historical_assessment']
        if Path(name).name != name:
            raise ValueError('unsafe historical assessment name')
        if name not in cached:
            cached[name] = json.loads((root / name).read_text())
        matches = [x for x in cached[name]['historical_source_roles']
                   if x['original_index'] == role['original_index']]
        if (len(matches) != 1 or matches[0]['source_id'] != role['source_id']
                or matches[0]['sha256'] != role['historical_sha256']):
            raise ValueError('historical body assessment identity mismatch')
        source_lines = len(objects[role['historical_sha256']].splitlines())
        if (matches[0]['lines'] != source_lines
                or matches[0]['new_personal_read_ranges'] != [[1, source_lines]]):
            raise ValueError('historical body range mismatch')
    elif status == 'deleted-base-trace-open':
        original = registry[role['original_index']]
        doc = json.loads((root / role['historical_assessment']).read_text())
        if (original.get('state') != 'deleted' or role['historical_sha256'] is not None
                or role['historical_object_available'] or doc['original_index'] != role['original_index']
                or doc['source_id'] != role['source_id'] or doc['base_sha256'] != original.get('base_sha256')
                or doc['registry_state'] != 'deleted' or doc['snapshot_sha256'] is not None):
            raise ValueError('deleted role/base identity mismatch')
        base_path = (root / doc['preserved_base_bytes']).resolve()
        if not base_path.is_relative_to(root) or hashlib.sha256(base_path.read_bytes()).hexdigest() != doc['base_sha256']:
            raise ValueError('deleted role base bytes mismatch')
    elif status == 'assessed-nonapplicable':
        doc = json.loads((root / role['assessment']).read_text())
        if not any(x['original_index'] == role['original_index'] and x['sha256'] == role['historical_sha256'] for x in doc['historical_source_roles']):
            raise ValueError('missing manifest role evidence')
        current = doc['current_source']
        raw = (root / current['preserved_bytes']).read_bytes()
        if hashlib.sha256(raw).hexdigest() != current['sha256'] or git_blob(raw) != paths.get(current['path']):
            raise ValueError('current manifest source mismatch')

counts = dict(collections.Counter(x['status'] for x in roles))
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if counts != work['status_counts'] or counts != progress['original_lens1_role_status_counts']:
    raise ValueError('role status count mismatch')
print('verified Git tree,', len(roles), 'distinct role identities and', len(needed), 'referenced source objects; coverage remains qualified')
