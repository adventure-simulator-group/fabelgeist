"""Verify a historical source body and its preserved current-trace lineage."""
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-HISTORICAL-129.json'
item = json.loads((root / name).read_text())['historical_source_roles'][0]
with tarfile.open(root / item['evidence_archive'], 'r:gz') as archive:
    raw = archive.extractfile(item['sha256']).read()
role = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][129]
if (item['original_index'] != 129 or hashlib.sha256(raw).hexdigest() != item['sha256'] or
        len(raw.splitlines()) != 1630 or item['new_personal_read_ranges'] != [[1,1630]] or
        role['source_id'] != item['source_id'] or role['status'] != 'full-historical-body-trace-open' or
        role['historical_assessment'] != name or not role.get('current_observations')):
    raise ValueError('historical identity or bounded-current lineage mismatch')
print('verified historical quest-catalog body and retained current traces; lens remains open')
