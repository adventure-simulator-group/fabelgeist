"""Verify preserved household allocation evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-HOUSEHOLD-PACKING-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 4 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
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
prop = sources['crates/adventuresim-core/src/settlement_property.rs']
must(prop, ['if home.market_reserve != supply.reserve(home.tier)', 'if !home.market_reserve',
            'capacity += u64::from(home.resident_capacity)', 'capacity < u64::from(population)',
            'self.validate(&self.settlement_id, self.population)?',
            'remaining.iter_mut().find(|(_, free)| *free >= *count)',
            'return Err(PropertyError::InsufficientCapacity)', '*free -= count'])
houses = sources['crates/adventuresim-tactical-core/src/city_layout/houses.rs']
must(houses, ['Self::Cottage => HousingTier::Cheap', 'Self::CraftTownHouse | Self::HallHouse => HousingTier::Moderate',
              'Self::MerchantHouse => HousingTier::Fancy', 'Self::Cottage => 6',
              'Self::CraftTownHouse => 13', 'Self::HallHouse => 16', 'Self::MerchantHouse => 30'])
homes = sources['crates/adventuresim-tactical-core/src/city_layout/compiled/homes.rs']
must(homes, ['homes.sort_by_key(|(id, ..)| *id)', 'resident_capacity: class.resident_capacity()',
             'market_reserve: market.reserve(tier)', 'catalog.validate(settlement_id, population)?'])
reducer = sources['crates/adventuresim-stdb-module/src/residence/properties.rs']
must(reducer, ['require_strategic_gateway(ctx)?', 'families.sort_by(|left, right| left.0.cmp(&right.0))',
               '.allocate(&families)', 'filter(|member| residents.contains(&member.character_id))'])
planner = sources['crates/adventuresim-tactical-server-dispatcher/src/settlement_properties.rs']
must(planner, ['layout.unhoused_population > 0', '!layout.unplaced_services.is_empty()',
               '!layout.demand_shortfalls.is_empty()', '.generated_homes(settlement_id, population)'])
web = sources['crates/strategic-web/src/routes/settlement_properties.rs']
must(web, ['settlement_properties::generated_homes(', '"register_settlement_properties"'])
tiers = sources['crates/adventuresim-core/src/courtship.rs']
must(tiers, ['pub const ALL: [Self; 3] = [Self::Cheap, Self::Moderate, Self::Fancy]'])
# Symbolic capacities only; this does not construct or execute a product catalog.
assert 13 + 6 == 19 and 6 + 13 == 19
assert 13 - 6 == 7 and 7 < 13 and 6 < 13
assert 6 <= 6 and 13 <= 13
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 5
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('Greedyfirst-fit mayrejectfeasible' in x for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'], old['evidence']['registration']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][303]
assert prior['receipt'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['accepted_unique_count_change'] == 0
assert prior['contract']['current_reconciled'] is False
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_operation_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified household source, symbolic capacity reasoning and 5 historical records; no product ran')
