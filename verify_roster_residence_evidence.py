"""Verify source identity, historical attribution and arithmetic only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
roster = json.loads((root / 'ASSESSMENT-L1-MISSION-ROSTERS.json').read_text())
residence = json.loads((root / 'ASSESSMENT-L1-NPC-HOUSING-RESERVE.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-121-124.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t',1)
    _, _, blob = header.split()
    if path in tree:
        raise ValueError('duplicate tree path')
    tree[path] = blob
sources = {}
for doc in (roster,residence):
    for item in doc['current_sources']:
        path = (root / item['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                blob != tree.get(item['path']) or item['revision'] != doc['current_revision']):
            raise ValueError('current source identity mismatch')
        if any(not 1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']):
            raise ValueError('invalid source range')
        sources[item['path']] = raw.decode()
party = sources['crates/adventuresim-core/src/mission/roster.rs']
enemy = sources['crates/adventuresim-core/src/mission/enemy_roster.rs']
launch = sources['crates/adventuresim-tactical-server-dispatcher/src/main.rs']
if not all(s in party for s in ('members: Vec<u64>', 'members.len() > MAX_TACTICAL_RECEIPT_PARTICIPANTS',
                                'members.iter().collect::<BTreeSet<_>>()', 'pub fn expected_members(&self) -> NonZeroU32')):
    raise ValueError('party roster trace mismatch')
if not all(s in enemy for s in ('enemy_ids: Vec<u64>', 'u32::try_from(length)',
                                'enemy_ids.iter().collect::<BTreeSet<_>>()')):
    raise ValueError('enemy roster trace mismatch')
if 'expected_party_members: roster.expected_members().to_string()' not in launch:
    raise ValueError('launch count trace mismatch')
npc = sources['crates/adventuresim-core/src/npc_policy.rs']
producer = sources['crates/adventuresim-stdb-module/src/residence.rs']
if not all(s in npc for s in ('NPC_HOUSING_RESERVE_PERIODS: u64 = 1', 'offer.initial_cost.saturating_add(',
                              '.saturating_mul(NPC_HOUSING_RESERVE_PERIODS)', ') <= available')):
    raise ValueError('affordability trace mismatch')
if not all(s in producer for s in ('pub rent_per_period: u32', 'initial_cost: u64::from(row.rent_per_period)',
                                   'recurring_cost: u64::from(row.rent_per_period)')):
    raise ValueError('rent producer trace mismatch')
maximum = 2**64 - 1
if not (maximum + 1 > maximum and 2 * (2**32 - 1) < maximum):
    raise ValueError('conditional affordability arithmetic mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in history['historical_source_roles']:
        raw = archive.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-121-124.json'):
            raise ValueError('historical attribution mismatch')
if [item['original_index'] for item in history['historical_source_roles']] != [121,122,123,124]:
    raise ValueError('historical role population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == 'REVIEW-NPC-RESIDENCE-RESERVE-001')
if entry['record'] != 'ASSESSMENT-L1-NPC-HOUSING-RESERVE.json' or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
    raise ValueError('residence ledger mismatch')
entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == 'NEW-TACTICAL-ENTRY-CHARACTER-ID-001')
if 'ASSESSMENT-L1-MISSION-ROSTERS.json' not in entry.get('additional_assessments',[]):
    raise ValueError('roster ledger linkage mismatch')
print('verified roster and residence source identities; runtime and extreme-input reachability remain open')
