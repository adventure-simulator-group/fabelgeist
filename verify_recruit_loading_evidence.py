"""Verify recruitment and loading source evidence, not product behavior.

Run the tree and role verifiers first. No reducer, database, UI, build or test
is run by this evidence check.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
recruit = json.loads((root / 'ASSESSMENT-L1-RECRUITMENT-ROLE.json').read_text())
loading = json.loads((root / 'ASSESSMENT-L01-LOADING-GATE.json').read_text())

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in out:
            raise ValueError('duplicate tree path')
        out[name] = blob
    return out

trees = {recruit['current_revision']: tree('CURRENT-TREE.tsv'),
         loading['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (recruit, loading):
    for key in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(key, []):
            path = (root / item['preserved_bytes']).resolve()
            if not path.is_relative_to(root):
                raise ValueError('source escapes checkpoint')
            raw = gzip.decompress(path.read_bytes())
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            if (hashlib.sha256(raw).hexdigest() != item['sha256'] or
                    len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
                    blob != item['git_blob'] or blob != trees[item['revision']].get(item['path'])):
                raise ValueError('source identity mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()

current = recruit['current_revision']
governance = sources[(current, 'crates/adventuresim-stdb-module/src/strategic/governance.rs')]
create = governance.split('pub fn create_recruitment_role(', 1)[1].split('#[reducer]', 1)[0]
validate = governance.split('fn validate_recruitment_requirements(', 1)[1].split('#[reducer]', 1)[0]
matching = governance.split('fn role_requirements(', 1)[1].split('#[derive', 1)[0]
for name in ('physiology', 'surgery', 'command', 'religion'):
    if ('requirements.' + name) not in create or ('requirements.' + name) in validate:
        raise ValueError('admission field distinction missing: ' + name)
    if 'requirements.' + name + ' = 0;' not in matching:
        raise ValueError('matching projection missing: ' + name)
form = sources[(current, 'crates/strategic-web/src/routes/parties.rs')]
if any(name + ': 0,' not in form for name in ('physiology', 'surgery', 'command', 'religion')):
    raise ValueError('web form zero projection missing')
progress_source = sources[(current, 'crates/adventuresim-tactical-client/src/loading_screen/progress.rs')]
view = sources[(current, 'crates/adventuresim-tactical-client/src/loading_screen/view.rs')]
if 'self.completed.min(self.total) as f32 / self.total as f32' not in progress_source or 'progress.fraction() * FRACTION_QUANTUM' not in view:
    raise ValueError('loading fraction source missing')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == recruit['contract_id'] and x['record'] == 'ASSESSMENT-L1-RECRUITMENT-ROLE.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('recruitment ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
for name in loading['paths']:
    source = next(x for x in loading['current_sources'] if x['path'] == name)
    if loading['personally_inspected_current_ranges'][name] != [[1, source['lines']]]:
        raise ValueError('loading body read range mismatch')
    entry = next(x for x in baseline['source_delta'] if x['path'] == name)
    if not entry['lens_status'][0].startswith('assessed:') or 'ASSESSMENT-L01-LOADING-GATE.json' not in entry['lens_status'][0]:
        raise ValueError('loading delta disposition mismatch')
recon = json.loads((root / 'RECONCILIATION-059.json').read_text())
entry = next(x for x in recon['entries'] if x['contract_id'] == recruit['historical_related_scope']['contract_id'])
if entry['source_pointer'] != recruit['historical_related_scope']['queue_pointer'] or not entry['current_reconciliation'].startswith('partial'):
    raise ValueError('historical scope linkage mismatch')
print('verified recruitment and loading source identities; policy and runtime remain open')
