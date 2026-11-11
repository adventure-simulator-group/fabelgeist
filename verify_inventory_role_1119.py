"""Verify historical role 1119 and distinct current bytes; no product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
role = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][1119]
doc = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-1119.json').read_text())
record = doc['historical_source_roles'][0]
assert role['source_id'] == record['source_id'] and role['historical_sha256'] == record['sha256']
assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1119.json'
assert record['new_personal_read_ranges'] == [[1, 1451]] and record['lines'] == 1451
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    old = archive.extractfile(record['sha256']).read()
assert hashlib.sha256(old).hexdigest() == record['sha256']
current = gzip.decompress((root / 'main-advance-95e/inventory-container-1119-current.source.gz').read_bytes())
assert hashlib.sha256(current).hexdigest() == record['current_comparison']['sha256']
assert record['current_comparison']['source_revision'] == revision
assert old != current and record['current_comparison']['byte_identical'] is False
blob = hashlib.sha1(b'blob ' + str(len(current)).encode() + b'\0' + current).hexdigest()
path = role['source_id'].split(':', 1)[1]
line = next(x for x in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines() if x.endswith('\t' + path))
assert line.split('\t', 1)[0].split()[2] == blob
assert b'pub(crate) fn object_for_row(' in old
assert b'pub(crate) use object_lookup::object_for_row;' in current
assert b'pub(crate) fn object_for_row(' not in current
print('verified separate historical and 95e inventory container bytes; no product ran')
