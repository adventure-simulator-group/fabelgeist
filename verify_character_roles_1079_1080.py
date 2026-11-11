"""Verify separately preserved historical and current character source identities."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-HISTORICAL-1079-1080.json'
doc = json.loads((root / name).read_text())
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
tree = {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
        for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines()}
assert [x['original_index'] for x in doc['historical_source_roles']] == [1079, 1080]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in doc['historical_source_roles']:
        role = roles[item['original_index']]
        historical = archive.extractfile(item['sha256']).read()
        assert hashlib.sha256(historical).hexdigest() == item['sha256'] == role['historical_sha256']
        assert item['source_id'] == role['source_id'] and len(historical.splitlines()) == item['lines']
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == name
        comparison = item['current_comparison']
        current = gzip.decompress((root / comparison['preserved_bytes']).read_bytes())
        assert len(current) == comparison['bytes'] and len(current.splitlines()) == comparison['lines']
        assert hashlib.sha256(current).hexdigest() == comparison['sha256']
        blob = hashlib.sha1(b'blob ' + str(len(current)).encode() + b'\0' + current).hexdigest()
        assert blob == comparison['git_blob'] == tree[item['source_id'].split(':', 1)[1]]
        assert historical != current and comparison['byte_identical'] is False
        if item['original_index'] == 1079:
            assert item['previous_personal_read_ranges'] == [[1, 420]]
            assert item['new_personal_read_ranges'] == [[421, 3983]]
            assert item['personal_read_ranges_union'] == [[1, 3983]]
        else:
            assert item['new_personal_read_ranges'] == [[1, item['lines']]]
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['post_transition_historical_body_reads']
print('verified two historical bodies and distinct current bytes; no product ran')
