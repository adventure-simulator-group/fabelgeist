"""Verify bounded source and recovery evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ALCOHOL-CALENDAR-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision
current = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert path not in current and kind in ('blob', 'commit')
    current[path] = oid

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw

def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == current[item['path']]
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 17
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
a = sources[core + 'alcohol.rs']
must(a, ['pub const MODEST_ETHANOL_ML: u32 = 15;',
 'pub const HEAVY_ETHANOL_ML: u32 = 45;', 'pub const MAX_ALCOHOL_INTERVAL_MINUTES: u64 = DAYS_PER_YEAR * MINUTES_PER_DAY;',
 'if elapsed > MAX_ALCOHOL_INTERVAL_MINUTES', 'checked_interval(start, end)?;',
 '.saturating_mul(properties.abv_basis_points as u32)',
 'properties.net_hydration_ml <= water_content_ml(properties)',
 'if properties.net_hydration_ml < water', '&& ethanol_ml(s.properties) > 0',
 'let mut remaining = *requested;', 'while supply.quantity > 0 && remaining > 0',
 'remaining = remaining.saturating_sub(each);',
 'assert!(!properties_valid(impossible));', 'assert_eq!(emergency_hydration_ml(impossible), 50);'])
must(sources['crates/adventuresim-world-schema/src/basis_points.rs'], ['pub const BASIS_POINTS_PER_WHOLE: u16 = 10_000;'])
v = sources[core + 'item_catalog_validation.rs']
must(v, ['validate_item(item, index, &mut ids, &mut errors);',
 'validate_capabilities(capabilities, &path, kind, errors);',
 '|| serving > 10_000', '|| abv > u64::from(BASIS_POINTS_PER_WHOLE)',
 '|| hydration > serving', 'get("disinfectant_focused")', 'get("potable")'])
assert 'properties_valid(' not in v
must(sources[core + 'item_catalog_schema.rs'], ['pub struct Alcohol {', 'pub serving_ml: u32,', 'pub abv_basis_points: u16,', 'pub net_hydration_ml: u32,'])
must(sources['crates/adventuresim-core/build.rs'], ['item_catalog_validation::validate_documents_with_sources(&documents, &source_files, &sources)'])
must(sources[core + 'item_catalog.rs'], ['build.rs` validates and combines them', 'serde_json::from_str(ITEM_CATALOG_JSON)'])
must(sources[module + 'item.rs'], ['item.alcohol_serving_ml = alcohol.serving_ml;', 'item.alcohol_net_hydration_ml = alcohol.net_hydration_ml;'])
must(sources[module + 'alcohol.rs'], ['serving_ml: item.alcohol_serving_ml,', 'let full_hydration = emergency_hydration_ml(p);'])
camp = sources['crates/strategic-web/src/routes/settlements/camp.rs']
must(camp, ['adventuresim_core::alcohol::rest_evenings(', 'adventuresim_core::alcohol::HEAVY_ETHANOL_ML',
 'adventuresim_core::alcohol::MODEST_ETHANOL_ML', 'expected_morale_demands.sort_by_key',
 'adventuresim_core::alcohol::hydration_after_expected_drinking('])
activity = sources[core + 'activity.rs']
must(activity, ['pub fn sundays_overlapping(start_minute: StrategicMinute, elapsed_minutes: u64) -> Vec<u64>',
 '(first_day..=last_day)', '.filter(|day| is_sunday_day_index(*day))', '.collect()'])
calendar = sources['crates/adventuresim-world-schema/src/calendar.rs']
must(calendar, ['pub const MINUTES_PER_DAY: u64 = 24 * 60;', 'pub const DAYS_PER_WEEK: u64 = 7;',
 'const SUNDAY_INDEX: u64 = 6;', 'pub const fn new(minutes: u64) -> Self',
 'self.minutes.saturating_add(elapsed_minutes)', 'self.minutes.saturating_sub(elapsed_minutes)'])
must(sources[module + 'condition.rs'], ['for sunday in sundays_overlapping(canonical_start, elapsed)',
 'format!("missed-canonical-sunday:{sunday}")'])
must(sources[module + 'strategic/travel_reducers.rs'], ['party.wilderness_canonical_anchor_minute',
 'crate::time::refresh_clock(ctx)', 'apply_canonical_wilderness_observance('])
must(sources[module + 'time/settlement_rest.rs'], ['party.wilderness_canonical_anchor_minute = Some(refresh_clock(ctx)?);'])
must(sources[module + 'time/clock.rs'], ['pub fn refresh_clock(ctx: &ReducerContext)', 'calculate_official_minute('])
must(sources[core + 'strategic_time.rs'], ['One real week advances exactly one strategic year.', 'pub fn official_minute(epoch_micros: i64, now_micros: i64)'])

# Arithmetic and source-data inspection only: product code is not executed.
serving, abv, hydration = 100, 5000, 90
assert 1 <= serving <= 10000 and abv <= 10000 and hydration <= serving
water = serving * (10000 - abv) // 10000
assert water == 50 and hydration > water and min(hydration, water) == 50
assert 10000 * 10000 < 2**32
assert min(1000000 * 10000, 2**32 - 1) // 10000 == 429496
catalog = json.loads(sources['content/items/catalog.yaml'])
rows = [(x['id'], x['capabilities']['alcohol']) for x in catalog['items']
        if 'alcohol' in x.get('capabilities', {})]
assert [(i, x['serving_ml'], x['abv_basis_points'], x['net_hydration_ml']) for i,x in rows] == [
 ('aqua_vitae', 100, 5000, 0), ('small_beer', 500, 300, 425), ('table_wine', 250, 1200, 150)]
assert all(x['net_hydration_ml'] <= x['serving_ml'] * (10000-x['abv_basis_points']) // 10000 for _,x in rows)
last = ((2**64 - 1) - 1) // 1440
assert last == 12810238940076077 and (last + 1) // 7 == 1830034134296582

manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
receipt = doc['recovered_historical_receipts'][0]
entry = next(x for x in manifest['entries'] if x['path'] == receipt['original_path'])
raw = preserved(receipt)
assert entry['sha256'] == receipt['sha256'] and entry['bytes'] == len(raw) == receipt['bytes']
earlier = json.loads(raw)
assert earlier['accepted_unique_count_change'] == 0 and earlier['current_source_reconciled'] is False
assert 'explicit normalization policies' in earlier['evidence']['numeric_role_inventory'][0]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][226]
assert old['receipt_sha256'] == receipt['sha256'] and old['contract']['source_originals'] == [853,855]
assert 'Alcoholhydrationexpecteddrinking' in ' '.join(ledger['segovb_continuation']['lens12']['pending_connected_contracts'][983])
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for ref in doc['historical_source_roles']:
    role = work['roles'][ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
for identifier in ('REVIEW-ALCOHOL-CATALOG-HYDRATION-CONTRACT-001', 'OPEN-CANONICAL-SUNDAY-WORK-BUDGET-001'):
    assert sum(x['id'] == identifier and x['record'] == name for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['new_nonfinding_assessments'] and progress['queue_records_semantically_reconciled'] == 0
assert 'ASSESSMENT-L01-HISTORICAL-853-855.json' in progress['post_transition_historical_body_reads']
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 17 Git-bound sources, one original receipt and bounded arithmetic; no product ran')
