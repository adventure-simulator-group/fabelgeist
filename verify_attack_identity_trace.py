"""Verify the source-bound identity trace and older receipt provenance only."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ATTACK-IDENTITY-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision


def path_bytes(path):
    path = (root / path).resolve()
    assert path.is_relative_to(root)
    return gzip.decompress(path.read_bytes())


def preserved(item):
    assert item['content_encoding'] == 'gzip'
    raw = path_bytes(item['preserved_bytes'])
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    assert len(raw) == item['bytes']
    return raw


def git_hash(kind, raw):
    return hashlib.sha1(kind.encode() + b' ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()


def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)


tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    head, path = line.split('\t', 1)
    _, kind, oid = head.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and git_hash('blob', raw) == item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 11
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
must(sources[core + 'autoresolve/melee_defense.rs'], [
    'pub(super) fn attack_id(self, attacker_id: u64) -> u64',
    'attacker_id.wrapping_shl(32) ^ MeleeTimelineEvent::tick_at(self.started_at_seconds)',
])
must(sources[core + 'autoresolve/joint_melee.rs'], [
    'candidate.attack_timing.contact_at_seconds == contact_seconds',
    '.position(|member| *member == attack_id)',
    'attacker_id(*left_side, left, allies, enemies).cmp(&attacker_id(',
    'if !scheduled_attack_is_current(attacker, attack.attack_timing)',
])
must(sources[core + 'autoresolve/melee_round/scheduling.rs'], [
    'attacker.melee_attack_started_at_seconds == Some(timing.started_at_seconds)',
    'attacker.melee_attack_contact_at_seconds == Some(timing.contact_at_seconds)',
])
must(sources[core + 'autoresolve/timeline.rs'], [
    'pub attack_id: Option<u64>', 'pub affected_attack_id: Option<u64>',
    'pub simultaneous_members: Vec<u64>', 'pub simultaneous_order: Option<u32>',
    'const STEPS_PER_SECOND: f32 = 64.0',
    '(time.seconds().max(0.0) * Self::STEPS_PER_SECOND).round() as u64',
])
must(sources[core + 'autoresolve/melee_round/mod.rs'], [
    'event.attack_id = Some(timing.attack_id(attacker.id));',
    'event.simultaneous_members = batch.members;',
    'event.simultaneous_order = Some(batch.order);',
])
must(sources[core + 'autoresolve/melee_iteration/evidence/timeline.rs'], [
    '.filter_map(|event| event.affected_attack_id)', '.filter(|id| contacted.contains(id))',
])
must(sources[core + 'autoresolve.rs'], [
    'pub struct Combatant {', 'pub id: u64,', 'pub fn new(id: u64) -> Self',
    'pub struct CombatantStrategicState {',
])
must(sources[core + 'starting_character.rs'], [
    'pub fn default_character(identity_seed: &str)', '0xd000_0000_0000_0000',
    'pub fn default_character_with_id(id: u64)', 'character.id = id;',
])
must(sources[module + 'character.rs'], ['pub struct Character {', 'pub id: u64,'])
must(sources[module + 'world_actor.rs'], [
    'fn field_character_id(context_id: &str, ordinal: u16) -> u64',
    ')) | (1u64 << 63)',
])
must(sources[module + 'capability/combatant.rs'], [
    'character_id: u64,', 'Combatant::from_strategic_state(CombatantStrategicState {',
    'id: character_id,',
])
# Symbolic integer/collection reasoning, not execution of product functions.
mask = (1 << 64) - 1
packed_a = ((1 << 32) & mask) ^ 0
packed_b = ((((1 << 32) + 1) << 32) & mask) ^ 0
assert packed_a == packed_b == 1 << 32
members = [packed_a, packed_b]
assert members.index(packed_a) == members.index(packed_b) == 0

manifest = json.loads(path_bytes('recovered/original-transport-manifest.json.gz'))
entries = {x['path']: x for x in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw)
    assert git_hash('blob', raw) == item['git_blob']
    assert item['original_path'] not in receipts
    receipts[item['original_path']] = (item, json.loads(raw))
assert len(receipts) == 6
prefix = 'evidence/continuation-segovb/audit-resume-20261027/'
for p in ('lens11/batch-011-receipt.json', 'lens11/batch-012-receipt.json'):
    old = receipts[prefix + p][1]
    assert old['lens'] == 11 and old['accepted_unique_count_change'] == 0
    assert old['clear'] is False and old['current_reconciled'] is False
old = receipts[prefix + 'lens13/batch-023-033-receipt.json'][1]
assert old['lens'] == 13 and old['accepted_new_unique_count'] == 0
assert old['frozen_commit'] == 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
cross = ledger['discovery_findings_and_dependencies']['CROSS-LENS-PACKED-ATTACK-ID-CAPACITY']
assert cross['sources'] == [x.removeprefix('evidence/continuation-segovb/') for x in list(receipts)[:3]]
assert 'no new accepted global unique count' in cross['status']
delegated = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
for i, n in ((230, 132), (232, 134), (233, 135)):
    item, prior = receipts[f'segovb/READ-L13-PARALLEL-{n}.json']
    assert delegated[i]['receipt'] == item['original_path']
    assert delegated[i]['receipt_sha256'] == item['sha256']
    assert prior['lens'] == 13 and prior['accepted_unique_count_change'] == 0
queue = json.loads((root / 'baseline-059/queue.json').read_text())['entries']
assert queue[9]['contract_id'] == 'CROSS-LENS-PACKED-ATTACK-ID-CAPACITY'

# Independently establish the older public handoff's exact Git tree and files.
older = doc['older_handoff_crosscheck']
commit = path_bytes(older['commit_raw'])
assert git_hash('commit', commit) == older['commit']
assert commit.splitlines()[0] == b'tree ' + older['root_tree'].encode()


def parse_tree(raw):
    result = {}
    pos = 0
    while pos < len(raw):
        space = raw.index(b' ', pos)
        zero = raw.index(b'\0', space + 1)
        mode = raw[pos:space].decode()
        part = raw[space + 1:zero].decode()
        oid = raw[zero + 1:zero + 21].hex()
        assert len(oid) == 40 and part not in result
        result[part] = (mode, oid)
        pos = zero + 21
    assert pos == len(raw)
    return result


trees = {}
for oid, path in older['tree_objects'].items():
    raw = path_bytes(path)
    assert git_hash('tree', raw) == oid
    trees[oid] = parse_tree(raw)
older_files = {}
for item in older['files']:
    raw = preserved(item)
    oid = older['root_tree']
    parts = ['handoff', '726-audit-20261027-adler'] + item['path'].split('/')
    for part in parts[:-1]:
        mode, oid = trees[oid][part]
        assert mode == '40000'
    mode, file_oid = trees[oid][parts[-1]]
    assert mode in ('100644', '100755')
    assert file_oid == item['git_blob'] == git_hash('blob', raw)
    assert item['path'] not in older_files
    older_files[item['path']] = (item, raw)
assert len(older_files) == 7
old_manifest_item, raw = older_files['MANIFEST.json']
assert old_manifest_item['sha256'] == 'a9f4d10e730d8c6ba3b3c753cbbdc12f6e2d61c1db41e5679416f66538b1601a'
old_manifest = json.loads(raw)
assert old_manifest['entry_count'] == len(old_manifest['entries']) == 18143
old_entries = {x['path']: x for x in old_manifest['entries']}
for path in list(receipts)[:3]:
    item, raw = older_files[path]
    assert old_entries[path]['sha256'] == item['sha256'] and old_entries[path]['bytes'] == len(raw)
    assert raw == preserved(receipts[path][0])

roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id']
    assert role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
for i in (863, 864, 865):
    assert roles[i]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-863-865.json'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name
               for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
reconciliation = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][9]
assert name in reconciliation['current_reconciliation'] and 'partial' in reconciliation['current_reconciliation']
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 11 Git-bound sources, six 059 receipts and older Git handoff provenance; no product ran')
