"""Verify nested creator-lock metadata comparison without builds or feature resolution."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'CREATOR-LOCK-DELTA.json').read_text())
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
role = work['roles'][680]
old_id = doc['historical']
new_id = doc['current']
assert role['status'] == 'lock-metadata-partial' and role['assessment'] == 'LOCKFILE-DELTA.json'
assert role['metadata_assessment'] == 'CREATOR-LOCK-DELTA.json' and old_id['original_index'] == 680
assert old_id['source_id'] == role['source_id'] and old_id['sha256'] == role['historical_sha256']
manifest = json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text())
assert {x['original']:x['sha256'] for x in manifest['sources']}[680] == old_id['sha256']
with tarfile.open(root / old_id['object_archive'], 'r:gz') as tar:
    old = tar.extractfile(old_id['sha256']).read()
path = (root / new_id['preserved_bytes']).resolve()
assert path.is_relative_to(root) and new_id['content_encoding'] == 'gzip'
new = gzip.decompress(path.read_bytes())
for raw, identity in ((old,old_id),(new,new_id)):
    assert len(raw) == identity['bytes'] and len(raw.splitlines()) == identity['lines'] and hashlib.sha256(raw).hexdigest() == identity['sha256']
blob = hashlib.sha1(b'blob '+str(len(new)).encode()+b'\0'+new).hexdigest()
assert blob == new_id['git_blob'] and new_id['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
for inventory in ('main-advance-95e/current-tree.tsv','main-advance-95e/previous-tree.tsv'):
    entries = {line.split('\t',1)[1]:line.split('\t',1)[0].split()[-1] for line in (root/inventory).read_text().splitlines()}
    assert entries[new_id['path']] == blob

def table(raw, expected):
    rows = tomllib.loads(raw.decode())['package']
    result = {(x['name'],x['version'],x.get('source','')):x for x in rows}
    assert len(rows) == len(result) == expected
    return result
left = table(old, old_id['package_records'])
right = table(new, new_id['package_records'])
changes = []
for key in sorted(left.keys() | right.keys()):
    before, after = left.get(key), right.get(key)
    if before != after:
        changes.append({'key':list(key),'before':before,'after':after})
comp = doc['comparison']
assert changes == comp['changes']
assert sum(x['before'] is None for x in changes) == comp['added_records']
assert sum(x['after'] is None for x in changes) == comp['removed_records']
assert sum(x['before'] is not None and x['after'] is not None for x in changes) == comp['modified_records']
assert len(left.keys() & right.keys()) - comp['modified_records'] == comp['unchanged_records']
assert json.loads((root/'LOCKFILE-DELTA.json').read_text())['creator_lock_metadata'] == 'CREATOR-LOCK-DELTA.json'
assert 'CREATOR-LOCK-DELTA.json' in progress['post_transition_historical_metadata_reads']
assert work['roles'][681]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-681.json'
print('verified distinct nested creator lockfiles and mechanical package comparison; coverage remains incomplete')
