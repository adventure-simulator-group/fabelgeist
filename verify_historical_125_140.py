"""Verify historical body identities and preserved bounded-current lineage."""
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
name = 'ASSESSMENT-L01-HISTORICAL-125-140.json'
doc = json.loads((root / name).read_text())
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
if [r['original_index'] for r in doc['historical_source_roles']] != [125,126,127,132,137,138,140]:
    raise ValueError('historical population mismatch')
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in doc['historical_source_roles']:
        raw = archive.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or role['historical_assessment'] != name):
            raise ValueError('historical identity mismatch')
if not roles[127].get('current_observations'):
    raise ValueError('prior bounded-current observations must remain')
print('verified seven historical source bodies and retained current-trace lineage; lenses remain open')
