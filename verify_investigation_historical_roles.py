"""Verify selected archived investigation source bodies, not product behavior."""
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
roles = load('COVERAGE-L01-ROLES.json')['roles']
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
entries = [
    ('ASSESSMENT-L01-HISTORICAL-1128-1129.json', [1128, 1129]),
    ('ASSESSMENT-L01-HISTORICAL-1130-1133.json', [1130, 1131, 1132, 1133]),
    ('ASSESSMENT-L01-HISTORICAL-1134-1135.json', [1134, 1135]),
]
progress = load('CONTINUATION-PROGRESS-95E.json')
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for path, indices in entries:
        doc = load(path)
        assert doc['lens'] == 1
        assert [r['original_index'] for r in doc['historical_source_roles']] == indices
        assert path in progress['post_transition_historical_body_reads']
        for row in doc['historical_source_roles']:
            role = roles[row['original_index']]
            assert role['historical_assessment'] == path
            assert role['status'] == 'full-historical-body-trace-open'
            assert role['source_id'] == row['source_id'] and role['historical_sha256'] == row['sha256']
            raw = archive.extractfile(row['sha256']).read()
            assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
            assert row['new_personal_read_ranges'] == [[1, row['lines']]]
            assert row['snapshot_head'] == '6533e444fa37552927070cc8d8190907fd3b413e'
            assert row['current_comparison']['source_revision'] == revision
            assert row['current_comparison']['byte_identical'] is True
            assert row['current_comparison']['sha256'] == sha(raw)
            assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
print('verified eight selected historical source bodies and current byte identities; no product ran')
