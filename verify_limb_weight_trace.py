"""Verify the source and prior receipt for a still-open weight policy question."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-LIMB-WEIGHTS-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def preserved(item):
    p = (root / item['preserved_bytes']).resolve()
    assert p.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(p.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert oid == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 7
core = 'crates/adventuresim-core/src/'
body = sources[core + 'body.rs']
must(body, ['#[derive(Debug, Default, Clone, Copy, PartialEq)]\npub struct LimbWeights {',
 'pub left_arm: f32,', 'pub right_arm: f32,', 'pub left_leg: f32,', 'pub right_leg: f32,',
 'pub const fn both_arms() -> Self', 'pub const fn both_legs() -> Self',
 'pub const fn all_equal() -> Self', 'pub const fn normalize(self) -> Self'])
operation = body.split('pub const fn normalize(self) -> Self {', 1)[1].split('pub const fn arm(', 1)[0]
must(operation, ['let total = self.left_arm + self.right_arm + self.left_leg + self.right_leg;',
 'left_arm: self.left_arm / total,', 'right_arm: self.right_arm / total,',
 'left_leg: self.left_leg / total,', 'right_leg: self.right_leg / total,'])
assert 'if ' not in operation and 'is_finite' not in operation
must(sources[core + 'attribute.rs'], ['weights: LimbWeights,',
 'let weight = weights.by_part(part).clamp(0.0, 1.0);', 'sum + raw * weight * health'])
must(sources[core + 'skill.rs'], ['weights: LimbWeights,',
 'sum + weights.by_part(part).clamp(0.0, 1.0)', 'check *= usable_limbs;'])
must(sources[core + 'composite.rs'], ['pub fn limb_attr_by_weight(&self, attr: LimbAttribute, weights: LimbWeights)',
 'pub fn skill_check(&self, skill: Skill, weights: LimbWeights)'])
must(sources[core + 'autoresolve/melee_round/mobility.rs'], ['LimbWeights::both_legs()'])
must(sources[core + 'autoresolve/power.rs'], ['LimbWeights::both_arms()'])
must(sources[core + 'combat/targeting.rs'], ['LimbWeights::all_equal()', 'LimbWeights::both_legs()'])
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
item = doc['recovered_historical_receipts'][0]
entry = next(x for x in manifest['entries'] if x['path'] == item['original_path'])
raw = preserved(item)
assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw)
assert entry['git_blob'] == item['git_blob']
old = json.loads(raw)
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0
assert old['current_source_reconciled'] is False and old['implementation_gate_complete'] is False
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][241]
assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
assert 'actualLimbWeightsarbitrary/default/normalizecallers' in prior['contract']['needed']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
ref = doc['historical_source_roles'][0]
role = roles[ref['original_index']]
assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
assert role['historical_assessment'] == ref['historical_assessment']
for i in range(879, 887):
    assert roles[i]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-879-886.json'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name
               for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified seven Git-bound source records and one prior receipt; no product ran')
