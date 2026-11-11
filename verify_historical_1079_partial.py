"""Verify historical prefix and distinct pinned current bytes; no product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-HISTORICAL-1079-PARTIAL-001.json'
doc = json.loads((root / name).read_text())
with tarfile.open(root / doc['evidence_archive'], 'r:gz') as archive:
    historical = archive.extractfile(doc['historical_sha256']).read()
assert hashlib.sha256(historical).hexdigest() == doc['historical_sha256']
assert len(historical.splitlines()) == doc['lines'] == 3983
assert doc['new_personal_read_ranges'] == [[1, 420]] and doc['next_unread_line'] == 421
current = gzip.decompress((root / doc['current_comparison']['preserved_bytes']).read_bytes())
assert len(current) == doc['current_comparison']['bytes']
assert len(current.splitlines()) == doc['current_comparison']['lines']
assert hashlib.sha256(current).hexdigest() == doc['current_comparison']['sha256']
blob = hashlib.sha1(b'blob ' + str(len(current)).encode() + b'\0' + current).hexdigest()
assert blob == doc['current_comparison']['git_blob'] and current != historical
assert doc['current_comparison']['byte_identical'] is False
path = doc['source_id'].split(':', 1)[1]
tree = {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
        for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines()}
assert tree[path] == blob
role = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][1079]
assert role['source_id'] == doc['source_id'] and role['historical_sha256'] == doc['historical_sha256']
assert role['status'] in ('unassessed', 'full-historical-body-trace-open')
assert {'assessment': name, 'ranges': [[1, 420]]} in role['historical_partial_observations']
if role['status'] == 'full-historical-body-trace-open':
    full = json.loads((root / role['historical_assessment']).read_text())
    row = next(x for x in full['historical_source_roles'] if x['original_index'] == 1079)
    assert row['previous_partial_assessment'] == name
    assert row['previous_personal_read_ranges'] == [[1, 420]]
    assert row['new_personal_read_ranges'] == [[421, 3983]]
    assert row['personal_read_ranges_union'] == [[1, 3983]]
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['post_transition_historical_partial_body_reads']
print('verified historical character prefix and distinct current bytes; no product ran')
