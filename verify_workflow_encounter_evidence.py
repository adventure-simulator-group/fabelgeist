"""Verify checkpoint source identities and conditional score arithmetic only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
workflow = json.loads((root / 'ASSESSMENT-L01-WORKFLOW-SOURCES.json').read_text())
encounter = json.loads((root / 'ASSESSMENT-L1-ENCOUNTER-SCORES.json').read_text())
historical = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-97-105.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, blob = header.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {workflow['current_revision']: tree('CURRENT-TREE.tsv'),
         workflow['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (workflow, encounter):
    for category in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(category, []):
            path = (root / item['preserved_bytes']).resolve()
            if not path.is_relative_to(root):
                raise ValueError('source escapes checkpoint')
            raw = gzip.decompress(path.read_bytes())
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                    len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                    blob != trees[item['revision']].get(item['path'])):
                raise ValueError('source identity mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()
for path in workflow['absent_at_reported_selected_revision']:
    if path in trees[workflow['reported_selected_revision']]:
        raise ValueError('selected-tree absence mismatch')
cur = workflow['current_revision']
script = sources[(cur, 'scripts/cargo_artifacts.py')]
if not all(piece in script for piece in ('DEFAULT_MAX_AGE_DAYS = 14', 'DEFAULT_MAX_SIZE_GIB = 64',
                                          'math.isfinite(number)', 'if args.apply:', 'subprocess.run(command')):
    raise ValueError('artifact-helper trace mismatch')
if '--features", "gameplay' not in sources[(cur, 'scripts/build_wasm.py')]:
    raise ValueError('WASM feature trace mismatch')
for path in ('scripts/dev_stack.py', 'scripts/just_tasks.py'):
    if '--build-options=--debug' not in sources[(cur, path)]:
        raise ValueError('binding trace mismatch')
core = sources[(cur, 'crates/adventuresim-core/src/encounter.rs')]
module = sources[(cur, 'crates/adventuresim-stdb-module/src/strategic/encounters.rs')]
skill = sources[(cur, 'crates/adventuresim-core/src/skill.rs')]
bestiary = sources[(cur, 'crates/adventuresim-core/src/bestiary.rs')]
if not all(piece in core for piece in ('party_stealth: u16', 'enemy_awareness: u16',
                                       '.index(1000) as u16)', '+ party_stealth', '+ enemy_awareness')):
    raise ValueError('encounter helper trace mismatch')
if not all(piece in module for piece in ('training.max(0.0) * 100.0', '200_u16.saturating_add(enemy_stealth)')):
    raise ValueError('strategic producer trace mismatch')
if not all(piece in skill for piece in ('const MAX_CHECK: f32 = 5.0', 'aptitude.clamp(0.0, MAX_CHECK)',
                                        'self.training_rank(hours).min(aptitude)')):
    raise ValueError('skill cap trace mismatch')
if 'pub stealth: u8' not in bestiary or not (500 + 999 < 65536 and 200 + 255 + 999 < 65536 and 65535 + 1 > 65535):
    raise ValueError('source bound or conditional arithmetic mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for entry in historical['historical_source_roles']:
        raw = archive.extractfile(entry['sha256']).read()
        role = roles[entry['original_index']]
        if (hashlib.sha256(raw).hexdigest() != entry['sha256'] or
                len(raw.splitlines()) != entry['lines'] or
                entry['new_personal_read_ranges'] != [[1, entry['lines']]] or
                role['source_id'] != entry['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-97-105.json'):
            raise ValueError('historical identity or attribution mismatch')
if [x['original_index'] for x in historical['historical_source_roles']] != [97,101,102,103,105]:
    raise ValueError('historical role population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
for path in workflow['paths']:
    row = next(item for item in baseline['source_delta'] if item['path'] == path)
    if not row['lens_status'][0].startswith('partial'):
        raise ValueError('delta disposition mismatch')
entry = next(item for item in baseline['repair_ledger']['entries'] if item['id'] == 'REVIEW-ENCOUNTER-SNEAK-SCORE-001')
if entry['record'] != 'ASSESSMENT-L1-ENCOUNTER-SCORES.json' or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
    raise ValueError('ledger linkage mismatch')
print('verified workflow, encounter and historical source identities; no workflow or product behavior was run')
