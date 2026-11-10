"""Verify a deleted role and pre-deletion Git tree proof without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-DELETED-272.json'
doc = json.loads((root / name).read_text())
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
registry = json.loads(gzip.decompress((root / work['registry']).read_bytes()))['files']
role = work['roles'][272]
original = registry[272]
if (doc['original_index'] != 272 or doc['source_id'] != role['source_id'] or
    doc['source_id'] != original['source_id'] or original['state'] != 'deleted' or
    original['sha256'] is not None or role['historical_sha256'] is not None or
    role['historical_object_available'] or role['status'] != 'deleted-base-trace-open' or
    role['historical_assessment'] != name or doc['base_sha256'] != original['base_sha256'] or
    doc['registry_state'] != 'deleted' or doc['snapshot_sha256'] is not None):
    raise ValueError('deleted snapshot qualification mismatch')
base_path = (root / doc['preserved_base_bytes']).resolve()
if not base_path.is_relative_to(root):
    raise ValueError('unsafe base path')
base = base_path.read_bytes()
blob = hashlib.sha1(b'blob ' + str(len(base)).encode() + b'\0' + base).hexdigest()
if (len(base) != doc['base_bytes'] or len(base.splitlines()) != doc['base_lines'] or
    hashlib.sha256(base).hexdigest() != doc['base_sha256'] or blob != doc['base_git_blob'] or
    doc['new_personal_base_read_ranges'] != [[1, doc['base_lines']]]):
    raise ValueError('base source mismatch')

def read_gz(item):
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root):
        raise ValueError('unsafe proof path')
    raw = gzip.decompress(path.read_bytes())
    if len(raw) != item['bytes'] or hashlib.sha256(raw).hexdigest() != item['sha256']:
        raise ValueError('proof byte mismatch')
    return raw

commit = read_gz(doc['commit_proof'])
commit_oid = hashlib.sha1(b'commit ' + str(len(commit)).encode() + b'\0' + commit).hexdigest()
if commit_oid != doc['base_revision'] or commit_oid != doc['commit_proof']['git_commit']:
    raise ValueError('base commit mismatch')
expected_tree = commit.split(b'\n', 1)[0].decode().split(' ', 1)[1]
parts = original['path'].split('/')
if len(parts) != len(doc['tree_proof']):
    raise ValueError('tree proof length mismatch')
for i, (component, proof) in enumerate(zip(parts, doc['tree_proof'])):
    raw = read_gz(proof)
    oid = hashlib.sha1(b'tree ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if oid != expected_tree or oid != proof['tree'] or component != proof['component']:
        raise ValueError('tree identity mismatch')
    offset = 0
    matches = []
    while offset < len(raw):
        space = raw.index(b' ', offset)
        nul = raw.index(b'\0', space)
        mode = raw[offset:space].decode()
        entry_name = raw[space + 1:nul].decode()
        entry_oid = raw[nul + 1:nul + 21].hex()
        offset = nul + 21
        if entry_name == component:
            matches.append((mode, entry_oid))
    if len(matches) != 1 or matches[0][1] != proof['entry_oid']:
        raise ValueError('tree entry mismatch')
    mode, expected_tree = matches[0]
    if i < len(parts) - 1:
        if mode not in ('40000', '040000') or proof['entry_kind'] != 'tree':
            raise ValueError('intermediate entry mismatch')
    elif mode != '100644' or proof['entry_kind'] != 'blob' or expected_tree != blob:
        raise ValueError('terminal entry mismatch')
current = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    current[path] = header.split()[-1]
if current.get(original['path']) != blob:
    raise ValueError('qualified current byte identity mismatch')
if name not in progress['post_transition_deleted_base_reads']:
    raise ValueError('missing deleted-base progress link')
print('verified deleted physical-p16 identity and full a120 commit/tree/base-byte proof; current semantic scope remains open')
