"""Verify historical disease and browser fixture source identities only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
fixtures = json.loads((root / 'ASSESSMENT-L01-BROWSER-FIXTURES.json').read_text())
disease = json.loads((root / 'ASSESSMENT-L01-DISEASE-FULL.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {fixtures['current_revision']: tree('CURRENT-TREE.tsv'),
         fixtures['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for key in ('current_sources', 'reported_selected_sources'):
    for item in fixtures[key]:
        path = (root / item['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                blob != trees[item['revision']].get(item['path'])):
            raise ValueError('source identity mismatch')
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid source range')
        sources[(item['revision'], item['path'])] = raw.decode()
cur = fixtures['current_revision']
regional = sources[(cur, 'crates/strategic-web/tests/regional-map.browser.cjs')]
scene = sources[(cur, 'crates/strategic-web/tests/strategic-scene.browser.cjs')]
if ('<script src="/tactical/sfx.js"></script>' not in regional or
        'link.dataset.characterId = "100"' not in scene or
        '<script src="/tactical/sfx.js"></script>' not in scene):
    raise ValueError('expected source trace missing')
entry = disease['historical_source_roles'][0]
with tarfile.open(root / entry['evidence_archive'], 'r:gz') as tar:
    raw = tar.extractfile(entry['sha256']).read()
if (hashlib.sha256(raw).hexdigest() != entry['sha256'] or len(raw.splitlines()) != 3842 or
        entry['new_personal_read_ranges'] != [[1,3842]] or
        entry['incremental_personal_read_ranges'] != [[1,1280],[1281,3842]]):
    raise ValueError('historical body or read ranges mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
role = roles[93]
if (role['status'] != 'full-historical-body-trace-open' or
        role.get('historical_assessment') != 'ASSESSMENT-L01-DISEASE-FULL.json' or
        role.get('partial_historical_observations') != [
            {'assessment':'ASSESSMENT-L01-DISEASE-PARTIAL.json','ranges':[[1,1280]]}]):
    raise ValueError('historical read lineage mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
for path in fixtures['paths']:
    row = next(x for x in baseline['source_delta'] if x['path'] == path)
    if not row['lens_status'][0].startswith('partial'):
        raise ValueError('fixture delta disposition mismatch')
entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == 'NEW-TACTICAL-ENTRY-CHARACTER-ID-001')
if 'ASSESSMENT-L01-BROWSER-FIXTURES.json' not in entry.get('additional_assessments', []):
    raise ValueError('ledger linkage mismatch')
print('verified fixture and historical disease source identities; no browser or test was run')
