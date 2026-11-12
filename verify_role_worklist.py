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
    if role['status'] not in ('unassessed', 'bounded-current-trace-only', 'lock-metadata-partial', 'source-metadata-partial', 'assessed-nonapplicable', 'full-historical-body-trace-open', 'deleted-base-trace-open'):
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

advance = json.loads((root / 'MAIN-ADVANCE-95E.json').read_text())
new_paths = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    head, path = line.split('\t', 1)
    new_paths[path] = head.split()[2]
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
                and (source.get('revision') in (None, work['source_revision'])
                     or (source.get('revision') == '95e63628f9d70b8dd6e6fede64334696c6995009'
                         and new_paths.get(path) == role['current_git_blob']))
                for source in observed_sources(cached[name])
            )
            if not matched:
                raise ValueError('unmatched bounded evidence link')
    if status == 'lock-metadata-partial':
        if role['assessment'] == 'ASSESSMENT-DELEGATED-MHR-LOCK-95E.json':
            doc = json.loads((root / role['assessment']).read_text())
            if (doc['original_index'] != role['original_index']
                    or doc['source_id'] != role['source_id']
                    or doc['historical_sha256'] != role['historical_sha256']
                    or doc['historical_lines'] != len(objects[role['historical_sha256']].splitlines())):
                raise ValueError('partial lock historical identity mismatch')
            cp = json.loads((root / doc['evidence_checkpoint']).read_text())
            bundle = next(x for x in cp['bundles'] if x['archive'] == doc['evidence_archive'])
            raw_archive = (root / bundle['archive']).read_bytes()
            if hashlib.sha256(raw_archive).hexdigest() != bundle['archive_sha256']:
                raise ValueError('partial lock bundle mismatch')
            with tarfile.open(root / bundle['archive'], 'r:gz') as lock_archive:
                current = lock_archive.extractfile('current-2313.source').read()
            current_inventory = json.loads((root / 'MAIN-ADVANCE-95E.json').read_text())['trees']['current']['path']
            current_paths = {}
            for line in (root / current_inventory).read_text().splitlines():
                head, path = line.split('\t', 1)
                current_paths[path] = head.split()[2]
            if (hashlib.sha256(current).hexdigest() != doc['current_sha256']
                    or git_blob(current) != doc['current_git_blob']
                    or doc['current_git_blob'] != current_paths[role['source_id'].split(':', 1)[1]]
                    or len(current.splitlines()) != doc['current_lines']):
                raise ValueError('partial lock current identity mismatch')
            for prefix in ('historical', 'current'):
                intervals = sorted(doc[prefix + '_personal_ranges'] + doc[prefix + '_unread_ranges'])
                cursor = 1
                for lo, hi in intervals:
                    if lo != cursor or hi < lo:
                        raise ValueError('partial lock range gap or overlap')
                    cursor = hi + 1
                if (cursor != doc[prefix + '_lines'] + 1
                        or sum(hi - lo + 1 for lo, hi in doc[prefix + '_unread_ranges']) != doc[prefix + '_unread_lines']
                        or doc[prefix + '_unread_lines'] == 0):
                    raise ValueError('partial lock unread accounting mismatch')
        elif role['assessment'] != 'LOCKFILE-DELTA.json':
            raise ValueError('unexpected lock assessment')
    elif status == 'source-metadata-partial':
        doc = json.loads((root / role['assessment']).read_text())
        rows = [x for x in doc['historical_source_roles']
                if x['original_index'] == role['original_index']]
        if len(rows) != 1:
            raise ValueError('partial source role count mismatch')
        item = rows[0]
        if (item['source_id'] != role['source_id']
                or item['sha256'] != role['historical_sha256']
                or item['lines'] != len(objects[role['historical_sha256']].splitlines())):
            raise ValueError('partial source historical identity mismatch')
        cp = json.loads((root / doc['evidence']).read_text())
        bundle = next(x for x in cp['bundles'] if x['archive'] == item['evidence_archive'])
        raw_archive = (root / bundle['archive']).read_bytes()
        if hashlib.sha256(raw_archive).hexdigest() != bundle['archive_sha256']:
            raise ValueError('partial source bundle mismatch')
        with tarfile.open(root / bundle['archive'], 'r:gz') as partial_archive:
            historical = partial_archive.extractfile(item['historical_member']).read()
            current = partial_archive.extractfile(item['current_member']).read()
            receipt = json.load(partial_archive.extractfile(item['receipt_member']))
        cc = item['current_comparison']
        current_inventory = json.loads((root / 'MAIN-ADVANCE-95E.json').read_text())['trees']['current']['path']
        current_paths = {}
        for line in (root / current_inventory).read_text().splitlines():
            head, path = line.split('\t', 1)
            current_paths[path] = head.split()[2]
        if (historical != objects[role['historical_sha256']]
                or hashlib.sha256(current).hexdigest() != cc['sha256']
                or git_blob(current) != cc['git_blob']
                or cc['git_blob'] != current_paths[role['source_id'].split(':', 1)[1]]
                or len(current.splitlines()) != cc['lines']):
            raise ValueError('partial source current identity mismatch')
        for partition_index, (lines, personal, unread, expected) in enumerate((
                (item['lines'], item['new_personal_read_ranges'], item['historical_unread_ranges'], item['historical_unread_lines']),
                (cc['lines'], cc['personal_ranges'], cc['unread_ranges'], cc['unread_lines']))):
            cursor = 1
            for lo, hi in sorted(personal + unread):
                if lo != cursor or hi < lo:
                    raise ValueError('partial source range gap or overlap')
                cursor = hi + 1
            if (cursor != lines + 1 or sum(hi - lo + 1 for lo, hi in unread) != expected
                    or (expected == 0 and not (role['original_index'] == 2512 and partition_index == 1))):
                raise ValueError('partial source unread accounting mismatch')
        if role['original_index'] == 2434:
            rr = next(x for x in receipt['original_roles'] if x['original_role'] == 2434)
            if (rr['historical_ranges'] != item['new_personal_read_ranges']
                    or rr['current_ranges'] != cc['personal_ranges']):
                raise ValueError('quality source actual receipt mismatch')
        elif role['original_index'] == 2452:
            rr = [x for x in receipt['reads'] if x['role'] == 2452]
            if (len(rr) != 2 or any(x['full_body_displayed'] for x in rr)
                    or next(x for x in rr if x['revision'] == item['snapshot_head'])['actual_display_union'] != item['new_personal_read_ranges']
                    or next(x for x in rr if x['revision'] == cc['source_revision'])['actual_display_union'] != cc['personal_ranges']
                    or item['incident'] != 'EVIDENCE-INCIDENT-20261111-P17-METRICS.json'):
                raise ValueError('P17 metrics source actual receipt mismatch')
        elif role['original_index'] == 2470:
            rr = next(x for x in receipt['roles'] if x['original_index'] == 2470)
            if (rr['rig_quality_actual_display_ranges'] != item['actual_personal_display_chunks']
                    or rr['current_actual_display_ranges'] != cc['actual_personal_display_chunks']
                    or rr['rig_quality_unread_ranges'] != item['historical_unread_ranges']
                    or rr['current_unread_ranges'] != cc['unread_ranges']
                    or rr['pr_working_actual_display_ranges']):
                raise ValueError('export source actual receipt mismatch')
        elif role['original_index'] == 2475:
            rr = [x for x in receipt['reads'] if x['role'] == 2475]
            if (len(rr) != 2 or any(x['full_body_displayed'] for x in rr)
                    or next(x for x in rr if x['revision'] == item['snapshot_head'])['actual_display_union'] != item['new_personal_read_ranges']
                    or next(x for x in rr if x['revision'] == cc['source_revision'])['actual_display_union'] != cc['personal_ranges']):
                raise ValueError('quality configuration actual receipt mismatch')
        elif role['original_index'] == 2490:
            if (receipt['original_index'] != 2490
                    or receipt['p16_actual_display_ranges'] != item['actual_personal_display_chunks']
                    or receipt['current_actual_display_ranges'] != cc['actual_personal_display_chunks']
                    or receipt['p16_unread_ranges'] != item['historical_unread_ranges']
                    or receipt['current_unread_ranges'] != cc['unread_ranges']
                    or receipt['base_actual_display_ranges']
                    or receipt['older_origin_actual_display_ranges']
                    or receipt['pr_working_actual_display_ranges']):
                raise ValueError('P16 IK actual and unread range receipt mismatch')
        elif role['original_index'] == 2503:
            rr = [x for x in receipt['assigned_personal_reads'] if x['role'] == 2503]
            old = next(x for x in rr if x['source_identity'] != cc['source_revision'])
            current = next(x for x in rr if x['source_identity'] == cc['source_revision'])
            if (len(rr) != 2
                    or old['contiguously_displayed_ranges'] != item['actual_personal_display_chunks']
                    or current['contiguously_displayed_ranges'] != cc['actual_personal_display_chunks']
                    or old['not_contiguously_displayed_ranges'] != item['historical_unread_ranges']
                    or current['not_contiguously_displayed_ranges'] != cc['unread_ranges']):
                raise ValueError('P16 LOD actual and unread range receipt mismatch')
        elif role['original_index'] == 2512:
            rr = next(x for x in receipt['roles'] if x['original_index'] == 2512)
            if (rr['p16_display_windows'] != item['actual_personal_display_chunks']
                    or rr['current_display_windows'] != cc['actual_personal_display_chunks']
                    or rr['p16_unread_complement'] != item['historical_unread_ranges']
                    or rr['current_unread_complement'] != cc['unread_ranges']
                    or rr['0393_a120_6533_personal_display_windows']):
                raise ValueError('P16 materials actual and unread range receipt mismatch')
        else:
            raise ValueError('unexpected partial source role')
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
        item = matches[0]
        if item['lines'] != source_lines:
            raise ValueError('historical body line-count mismatch')
        if 'personal_read_ranges_union' in item:
            partial_name = item['previous_partial_assessment']
            if Path(partial_name).name != partial_name:
                raise ValueError('unsafe partial assessment name')
            partial = json.loads((root / partial_name).read_text())
            if (partial['original_index'] != role['original_index']
                    or partial['source_id'] != role['source_id']
                    or partial['historical_sha256'] != role['historical_sha256']
                    or partial['new_personal_read_ranges'] != item['previous_personal_read_ranges']
                    or item['personal_read_ranges_union'] != [[1, source_lines]]):
                raise ValueError('historical partial lineage mismatch')
            cursor = 1
            for lo, hi in sorted(item['previous_personal_read_ranges'] + item['new_personal_read_ranges']):
                if lo != cursor or hi < lo:
                    raise ValueError('historical range union gap or overlap')
                cursor = hi + 1
            if cursor != source_lines + 1:
                raise ValueError('historical range union incomplete')
        else:
            cursor = 1
            for lo, hi in item['new_personal_read_ranges']:
                if lo != cursor or hi < lo:
                    raise ValueError('historical body range gap or overlap')
                cursor = hi + 1
            if cursor != source_lines + 1:
                raise ValueError('historical body range incomplete')
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
