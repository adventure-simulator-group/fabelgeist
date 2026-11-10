"""Verify deleted historical roles and separately recovered base Git objects; no product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
registry = json.loads(gzip.decompress((root / work['registry']).read_bytes()))['files']
current = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    current[path] = header.split()[-1]

def read_gz(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root)
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw

for index in (669, 670):
    name = f'ASSESSMENT-L01-DELETED-{index}.json'
    doc = json.loads((root / name).read_text())
    role, original = work['roles'][index], registry[index]
    assert doc['original_index'] == index and doc['source_id'] == role['source_id'] == original['source_id']
    assert original['state'] == doc['registry_state'] == 'deleted' and original['sha256'] is None
    assert role['historical_sha256'] is None and not role['historical_object_available'] and doc['snapshot_sha256'] is None
    assert role['status'] == 'deleted-base-trace-open' and role['historical_assessment'] == name
    assert doc['base_sha256'] == original['base_sha256'] and name in progress['post_transition_deleted_base_reads']
    path = (root / doc['preserved_base_bytes']).resolve()
    assert path.is_relative_to(root)
    base = path.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(base)).encode() + b'\0' + base).hexdigest()
    assert len(base) == doc['base_bytes'] and len(base.splitlines()) == doc['base_lines']
    assert hashlib.sha256(base).hexdigest() == doc['base_sha256'] and blob == doc['base_git_blob']
    assert doc['new_personal_base_read_ranges'] == [[1, doc['base_lines']]]
    commit = read_gz(doc['commit_proof'])
    commit_oid = hashlib.sha1(b'commit ' + str(len(commit)).encode() + b'\0' + commit).hexdigest()
    assert commit_oid == doc['base_revision'] == doc['commit_proof']['git_commit'] == '3a50b708381ad00c116445c2c999a49c3943088b'
    expected_tree = commit.split(b'\n', 1)[0].decode().split(' ', 1)[1]
    parts = original['path'].split('/')
    assert len(parts) == len(doc['tree_proof'])
    for i, (component, proof) in enumerate(zip(parts, doc['tree_proof'])):
        raw = read_gz(proof)
        oid = hashlib.sha1(b'tree ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        assert oid == expected_tree == proof['tree'] and component == proof['component']
        offset = 0
        matches = []
        while offset < len(raw):
            space = raw.index(b' ', offset)
            nul = raw.index(b'\0', space)
            mode = raw[offset:space].decode()
            entry = raw[space+1:nul].decode()
            entry_oid = raw[nul+1:nul+21].hex()
            offset = nul+21
            if entry == component:
                matches.append((mode, entry_oid))
        assert len(matches) == 1 and matches[0][1] == proof['entry_oid']
        mode, expected_tree = matches[0]
        if i < len(parts)-1:
            assert mode in ('40000', '040000') and proof['entry_kind'] == 'tree'
        else:
            assert mode == '100644' and proof['entry_kind'] == 'blob' and expected_tree == blob
    assert original['path'] not in current and doc['current_95e_path_status'] == 'absent in current source tree'
print('verified two deleted pr-working roles and distinct 3a50 pre-deletion base proofs; no snapshot body inferred')
