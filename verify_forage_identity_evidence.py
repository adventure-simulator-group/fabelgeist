"""Verify source identities, bounded arithmetic and cited derive source only."""
import gzip
import hashlib
import json
import re
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
forage = json.loads((root / 'ASSESSMENT-L1-FORAGING-BOUNDS.json').read_text())
mission = json.loads((root / 'ASSESSMENT-L1-MISSION-ID-SERDE.json').read_text())
leader = json.loads((root / 'ASSESSMENT-L1-LEADER-BALLOTS.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-108-120.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, name = line.split('\t', 1)
    _, _, blob = header.split()
    if name in tree:
        raise ValueError('duplicate tree path')
    tree[name] = blob
sources = {}
for doc in (forage, mission, leader):
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
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid inspected range')
        sources[item['path']] = raw.decode()
source = sources['crates/adventuresim-core/src/foraging.rs']
section = source.split('pub const FORAGE_RESOURCES: &[ForageResource] = &[',1)[1].split('];',1)[0]
mins = [int(x) for x in re.findall(r'yield_min:\s*(\d+)', section)]
maxs = [int(x) for x in re.findall(r'yield_max:\s*(\d+)', section)]
if len(mins) != 15 or len(maxs) != 15 or any(lo < 1 or hi < lo or hi > 6 for lo,hi in zip(mins,maxs)):
    raise ValueError('authored forage ranges mismatch')
if not all(s in source for s in ('MAX_FORAGE_MINUTES: u64 = 24 * 60', 'MAX_SOURCES: usize = 5',
                                  'FOOD_DISCOVERY_RATE_PERMILLE: u64 = 1_750', 'Self::Common => 2_300',
                                  'terrain_check_millirank.min(5_000)', '.min(1_000)',
                                  'resources.is_empty()', 'source_ids.is_empty()')):
    raise ValueError('forage bound trace mismatch')
numerator = 2300 * 1440 * 1500 * 1000 * 1750
minimum_denominator = 8 * 60 * 1000**3
maximum_denominator = minimum_denominator * 5 * 15
if (numerator != 8694000000000000 or minimum_denominator != 480000000000 or
        maximum_denominator != 36000000000000 or not numerator < 2**64 or
        not 0 < minimum_denominator <= maximum_denominator < 2**64):
    raise ValueError('forage arithmetic bound mismatch')
identity = sources['crates/adventuresim-core/src/mission.rs']
if not all(s in identity for s in ('#[serde(transparent)]', 'pub struct $name(String)',
                                   'value.len() > 128', 'authority_id!(MissionId, "mission:")',
                                   'authority_id!(HostileGroupId, "hostile-group:")')):
    raise ValueError('mission wrapper trace mismatch')
native = mission['native_source']
archive = (root / native['archive']).resolve()
if not archive.is_relative_to(root):
    raise ValueError('archive escapes checkpoint')
with tarfile.open(archive, 'r:gz') as package:
    raw = package.extractfile(native['member']).read()
if hashlib.sha256(raw).hexdigest() != native['sha256'] or len(raw) != native['bytes']:
    raise ValueError('derive member mismatch')
derive = raw.decode()
if not all(s in derive for s in ('if cont.attrs.transparent()', 'deserialize_transparent(cont, params)',
                                  'quote_spanned!(span=> _serde::Deserialize::deserialize)',
                                  '|__transparent| #this_value { #(#assign),* }')):
    raise ValueError('transparent derive trace mismatch')
election = sources['crates/adventuresim-core/src/leadership.rs']
governance = sources['crates/adventuresim-stdb-module/src/strategic/governance.rs']
if not all(s in election for s in ('for &(voter, candidate) in votes', '*tallies.entry(candidate).or_default() += 1',
                                   '*count * 100 >= living.len() * 66')):
    raise ValueError('election trace mismatch')
if 'format!("{party_id}:{voter_id}")' not in governance or not (2 * 100 >= 3 * 66 and 100 < 3 * 66):
    raise ValueError('conditional ballot arithmetic mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as package:
    for item in history['historical_source_roles']:
        raw = package.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-108-120.json'):
            raise ValueError('historical attribution mismatch')
if [item['original_index'] for item in history['historical_source_roles']] != [108,111,114,115,117,118,120]:
    raise ValueError('historical population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
for identifier, record in [('NEW-MISSION-AUTHORITY-ID-DESERIALIZATION-001','ASSESSMENT-L1-MISSION-ID-SERDE.json'),
                           ('REVIEW-LEADER-BALLOT-UNIQUENESS-001','ASSESSMENT-L1-LEADER-BALLOTS.json')]:
    entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == identifier)
    if entry['record'] != record or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
        raise ValueError('ledger linkage mismatch')
print('verified forage, identifier and ballot source identities; product behavior remains unverified')
