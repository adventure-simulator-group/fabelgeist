"""Verify tactical-entry and partial historical evidence without running the product."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-TACTICAL-ENTRY-IDENTITY.json').read_text())
historical = json.loads((root / 'ASSESSMENT-L01-DISEASE-PARTIAL.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {doc['current_revision']: tree('CURRENT-TREE.tsv'),
         doc['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for key in ('current_sources', 'reported_selected_sources'):
    for item in doc[key]:
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

cur = doc['current_revision']
renderer = sources[(cur, 'crates/strategic-web/static/strategic-renderer.js')]
template = sources[(cur, 'crates/strategic-web/src/templates/mission.rs')]
starting = sources[(cur, 'crates/adventuresim-core/src/starting_character.rs')]
runtime = sources[(cur, 'crates/adventuresim-tactical-client/src/browser_runtime.rs')]
netcode = sources[(cur, 'crates/adventuresim-tactical-netcode/src/client.rs')]
server = sources[(cur, 'crates/adventuresim-stdb-module/src/tactical.rs')]
for body, needles in [(renderer, ('character_id: Number(link.dataset.characterId)',
                                'runtime.wasm_command(JSON.stringify(payload))')),
                      (template, ('data-character-id=(character_id)',)),
                      (starting, ('| 0x8000_0000_0000_0000', '| 0xd000_0000_0000_0000')),
                      (runtime, ('character_id: u64', 'let command = serde_json::from_str(json)',
                                 'player_id: character_id')),
                      (netcode, ('CharacterId(client.player_id)', 'commands.client_trigger(JoinRequest')),
                      (server, ('authorized_party_member_ids.contains(&character_id)',
                                'ctx.sender() != server'))]:
    if any(needle not in body for needle in needles):
        raise ValueError('expected identity source trace missing')

# Binary64 arithmetic only; no browser, command, network or game execution.
a = 1 << 53
if int(float(a)) != int(float(a + 1)):
    raise ValueError('numeric identity example mismatch')
entry = historical['historical_source_roles'][0]
registry = json.loads(gzip.decompress((root / 'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
if entry['original_index'] != 93 or entry['source_id'] != registry[93]['source_id'] or entry['sha256'] != registry[93]['sha256']:
    raise ValueError('historical role identity mismatch')
with tarfile.open(root / entry['evidence_archive'], 'r:gz') as tar:
    raw = tar.extractfile(entry['sha256']).read()
if (hashlib.sha256(raw).hexdigest() != entry['sha256'] or len(raw.splitlines()) != entry['lines'] or
        entry['new_personal_read_ranges'] != [[1, 1280]]):
    raise ValueError('partial historical body mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
role = roles[93]
if role['status'] != 'unassessed' or role.get('partial_historical_observations') != [
        {'assessment':'ASSESSMENT-L01-DISEASE-PARTIAL.json','ranges':[[1,1280]]}]:
    raise ValueError('partial historical read was promoted or lost')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == doc['contract_id'] and x['record'] == 'ASSESSMENT-L1-TACTICAL-ENTRY-IDENTITY.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('identity ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
row = next(x for x in baseline['source_delta'] if x['path'] == 'crates/strategic-web/static/strategic-renderer.js')
if not row['lens_status'][0].startswith('partial'):
    raise ValueError('renderer delta disposition mismatch')
print('verified tactical-entry sources and partial historical body; enrollment outcome remains unobserved')
