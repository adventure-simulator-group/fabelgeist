"""Verify the historical carry source and differing pinned current bytes."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-HISTORICAL-1081-1085.json'
doc = json.loads((root / name).read_text())
item = next(x for x in doc['historical_source_roles'] if x['original_index'] == 1085)
role = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][1085]
with tarfile.open(root / item['evidence_archive'], 'r:gz') as archive:
    historical = archive.extractfile(item['sha256']).read()
assert hashlib.sha256(historical).hexdigest() == item['sha256'] == role['historical_sha256']
assert len(historical.splitlines()) == item['lines'] and item['new_personal_read_ranges'] == [[1, item['lines']]]
assert role['historical_assessment'] == name and role['status'] == 'full-historical-body-trace-open'
current = gzip.decompress((root / item['current_comparison']['preserved_bytes']).read_bytes())
comparison = item['current_comparison']
assert len(current) == comparison['bytes'] and len(current.splitlines()) == comparison['lines']
assert hashlib.sha256(current).hexdigest() == comparison['sha256'] and current != historical
blob = hashlib.sha1(b'blob ' + str(len(current)).encode() + b'\0' + current).hexdigest()
tree = {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
        for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines()}
assert comparison['byte_identical'] is False and blob == comparison['git_blob'] == tree[role['source_id'].split(':', 1)[1]]
assert name in json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())['post_transition_historical_body_reads']
print('verified historical carry source and distinct current bytes; no product ran')
