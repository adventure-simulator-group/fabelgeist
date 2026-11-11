"""Verify schedule aggregate source and attribution without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L05-SCHEDULE-AGGREGATE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 5 and doc['source_revision'] == revision

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
assert len(sources) == 6
allocation = sources['crates/adventuresim-core/src/strategic_schedule/allocation.rs']
must(allocation, ['pub enum OrganizationAllocation', 'Scheduled {', 'pub struct ActivityMinutes(u16)',
                  '.is_multiple_of(ACTIVITY_SEGMENT_MINUTES)', 'pub struct DailySchedulePlan {',
                  'pub activities: [ActivityMinutes; 10]', 'pub apprenticeship: OrganizationAllocation',
                  'pub practice: OrganizationAllocation'])
validator = allocation.split('pub fn validate_daily_allocation(', 1)[1].split('#[derive', 1)[0]
must(validator, ['minutes.into_iter().map(u64::from).sum::<u64>() > MINUTES_PER_DAY',
                 '.map(ActivityMinutes::try_from)',
                 'OrganizationAllocation::parse(apprenticeship.0, apprenticeship.1)?',
                 'OrganizationAllocation::parse(practice.0, practice.1)?'])
assert 'minutes[7]' not in validator and 'minutes[8]' not in validator
org_parse = allocation.split('impl OrganizationAllocation {', 1)[1].split('    pub fn minutes', 1)[0]
must(org_parse, ['(0, None) => Ok(Self::None)', '(0, Some(_)) => Err',
                 'if organization_id.is_empty()', 'ActivityMinutes::try_from(minutes)?'])
must(sources['crates/adventuresim-core/src/activity.rs'], ['pub const ACTIVITY_SEGMENT_MINUTES: u16 = 15;'])
must(sources['crates/adventuresim-world-schema/src/calendar.rs'], ['pub const MINUTES_PER_DAY: u64 = 24 * 60;'])
# Symbolic integers only; no product constructor or validator is invoked.
assert sum([0] * 10) <= 24 * 60 and 1440 % 15 == 0 and 1440 + 1440 > 24 * 60
assert 0 % 15 == 0
native = sources['crates/adventuresim-stdb-module/src/time/stationary.rs']
web = sources['crates/strategic-web/src/schedule.rs']
for body, prefix in [(native, 'downtime'), (web, 'schedule')]:
    for field in ('apprenticeship_minutes', 'profession_practice_minutes'):
        assert body.count(prefix + '.' + field) >= 2
must(native, ['apprenticeship_minutes: plan.apprenticeship.minutes()',
              'profession_practice_minutes: plan.practice.minutes()',
              'validate_organization_schedule(ctx, character_id, &schedule.downtime)?'])
must(web, ['schedule.apprenticeship_organization_id = plan.apprenticeship.organization_id()',
           'schedule.practice_organization_id = plan.practice.organization_id()',
           'ValidatedSchedule::try_from(core_allocation(schedule))?'])
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
paths = set()
for line in raw.decode().splitlines():
    assert line.startswith(revision + ':')
    paths.add(line[len(revision)+1:].split(':', 1)[0])
assert paths == {'crates/adventuresim-core/src/strategic_schedule.rs',
                 'crates/adventuresim-core/src/strategic_schedule/allocation.rs',
                 'crates/adventuresim-stdb-module/src/time/stationary.rs',
                 'crates/strategic-web/src/schedule.rs'}
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 7
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('Parallelorganizationminutes consistency/membershipnativeadaptercontractnotprovedhere' in x
           for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
assert old['evidence']['partial_read_union'][-1]['union'] == [[1, 1363]]
for suffix, expected in [('001', [[1, 460]]), ('002', [[1, 920]])]:
    part = support['parallel-l13-20261029/READ-PL13-210-PARTIAL-' + suffix + '.json'][1]
    assert part['rows'][0]['fresh_personal_assessed_ranges'] == expected
assert old['evidence']['partial_read_union'][0]['new_personal_ranges'] == [[461, 920]]
assert old['evidence']['partial_read_union'][1]['new_personal_ranges'] == [[921, 1363]]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['discovery_receipts'][207]
assert prior['path'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['sha256'] == doc['recovered_historical_receipts'][0]['sha256']
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
assert name in progress['post_transition_bounded_aggregate_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified schedule source, bounded caller search and 7 historical records; no product ran')
