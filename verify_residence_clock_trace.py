"""Verify preserved residence-clock evidence without executing product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-RESIDENCE-CLOCK-95E.json'
doc = json.loads((root / name).read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if doc['source_revision'] != revision or doc['lens'] != 1:
    raise ValueError('wrong source or lens')

def tree(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in result:
            raise ValueError('duplicate tree path')
        result[path] = oid
    return result

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe or incorrect source reference')
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
        hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + item['path'])
    if item['path'] in sources:
        raise ValueError('duplicate source')
    for first, last in item['ranges']:
        if not 1 <= first <= last <= item['lines']:
            raise ValueError('invalid read range')
    sources[item['path']] = raw.decode()
if len(sources) != 7:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/residence.rs': [
        'fn residence_now(', 'fn supported_occupant_counts_at(', 'fn settle_one_holding_period(',
        'let paid = *available >= amount', 'ctx.db.residence_charge().id().find(&id).is_none()',
        '*available = available.saturating_sub(amount)', '*total_spent = total_spent.saturating_add(amount)',
        'due_minute.saturating_add_minutes(RESIDENCE_BILLING_PERIOD_MINUTES)',
        'holding.status = ResidenceHoldingStatus::Dormant', 'pub fn settle_residence_billing(',
        'holding.status == ResidenceHoldingStatus::Active && holding.next_due_minute <= now',
        'crate::item::consume_personal_currency(ctx, character_id, total_spent)?'],
    'crates/adventuresim-stdb-module/src/residence/acquisition.rs': [
        'next_due_minute: now.saturating_add_minutes(RESIDENCE_BILLING_PERIOD_MINUTES)'],
    'crates/adventuresim-core/src/courtship.rs': [
        'pub const HOUSING_BILLING_PERIOD_MINUTES: u64 = 30 * MINUTES_PER_DAY',
        'Self::Cheap => HousingEconomy', 'owner_maintenance_per_30_days: 2',
        'property_tax_per_30_days: 2', 'self.next = due.checked_add_minutes(HOUSING_BILLING_PERIOD_MINUTES)',
        'pub fn plan_due_period_settlement('],
    'crates/adventuresim-world-schema/src/calendar.rs': [
        'pub const MINUTES_PER_DAY: u64 = 24 * 60', 'pub const MAX: Self = Self::new(u64::MAX)',
        'Self::new(self.minutes.saturating_add(elapsed_minutes))', 'pub const fn checked_add_minutes'],
    'crates/adventuresim-stdb-module/src/time/mod.rs': [
        'settle_lifecycle_after_character_time_write', 'settle_residence_billing(ctx, character_id)?'],
    'crates/adventuresim-stdb-module/src/time/model.rs': [
        'pub struct CharacterTime', 'pub minutes: StrategicMinute'],
    'crates/adventuresim-stdb-module/src/item.rs': [
        'pub fn personal_currency_total(', 'pub fn consume_personal_currency('],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
residence = sources['crates/adventuresim-stdb-module/src/residence.rs']
period = residence.split('fn settle_one_holding_period(', 1)[1].split('pub fn settle_residence_billing(', 1)[0]
if period.index('ctx.db.residence_charge().id().find(&id).is_none()') >= period.index('*available = available.saturating_sub(amount)'):
    raise ValueError('receipt/payment order changed')
ledger = json.loads((root / doc['historical_source']['ledger']).read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][258]['contract']
if (doc['historical_source']['checkpoint'] != '059' or
    old['proposed_id'] != doc['historical_source']['proposed_id'] or
    old['status'] != 'Conditional numeric lifetime/work policy, actual native trace pending'):
    raise ValueError('historical question qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
if roles[245]['historical_sha256'] == hashlib.sha256(residence.encode()).hexdigest():
    raise ValueError('distinct physical-p16 body claim changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or
    candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
    not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified seven Git-bound sources and separately attributed historical exhaustion question')
