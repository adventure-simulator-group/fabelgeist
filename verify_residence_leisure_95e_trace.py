"""Verify pinned residence evidence and historical attribution, without product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = load('ASSESSMENT-L12-RESIDENCE-LEISURE-95E.json')
assert doc['source_revision'] == rev and doc['lens'] == 12
assert [r['id'] for r in doc['candidates']] == [
    'SEGOVB-L12-RESIDENCE-LEISURE-SOURCE-DISPATCH',
    'REVIEW-RESIDENCE-STALE-SPOUSE-SELECTION-001',
]
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == rev and all(1 <= lo <= hi <= row['lines'] for lo, hi in row['ranges'])
    contents[row['path']] = raw
residence = contents['crates/adventuresim-stdb-module/src/residence.rs']
leisure = residence.split(b'pub fn apply_residence_leisure_morale(', 1)[1].split(b'\nfn designate_holding_at', 1)[0]
assert b'.find(|event| {' in leisure and b'LeisureMoraleSourceId::parse(source).is_spouse()' in leisure
assert b'refresh_bounded_leisure_morale(' in leisure and b'MoraleEventKind::ResidenceLeisure' in leisure
parser = residence.split(b'impl<\'a> LeisureMoraleSourceId<\'a>', 1)[1].split(b'pub fn apply_residence_leisure_morale', 1)[0]
assert b'.strip_prefix("spouse-leisure:")' in parser and b'!coordinate.is_empty()' in parser
assert b'event.kind' not in parser and b'event.kind' not in leisure
reproduction = contents['crates/adventuresim-stdb-module/src/relationship/reproduction.rs']
spouse = reproduction.split(b'fn refresh_spouse_pair_morale(', 1)[1].split(b'fn settle_spouse_leisure_pair', 1)[0]
assert b'"spouse-leisure:{}:{}"' in spouse and b'first_id.min(second_id)' in spouse and b'first_id.max(second_id)' in spouse
assert b'MoraleEventKind::SpouseLeisure' in spouse and b'upsert_fixed_morale_event_without_refresh' in spouse
assert b'event.source_id.as_deref() == Some(&source)' in spouse
condition = contents['crates/adventuresim-stdb-module/src/condition.rs']
upsert = condition.split(b'pub(crate) fn upsert_fixed_morale_event_without_refresh(', 1)[1].split(b'\n}', 1)[0]
assert b'event.source_id.as_deref() == Some(source_id)' in upsert
assert b'morale_event().id().update(event)' in upsert and b'morale_event().insert(event)' in upsert
assert b'for event in ctx.db.morale_event().character_id().filter(character_id)' in condition
assert b'rank_morale_sources(&mut raw_sources, will)' in condition
core = contents['crates/adventuresim-core/src/courtship.rs']
assert b'RESIDENCE_MORALE_CAP_MILLI: u32 = 8_000' in core
assert b'SPOUSE_LEISURE_MORALE_CAP_MILLI: u32 = 12_000' in core
assert b'LEISURE_MORALE_STACK_CAP_MILLI: u32 = 16_000' in core
assert b'SPOUSE_LEISURE_MORALE_DURATION_MINUTES: u64 = 30 * MINUTES_PER_DAY' in core
assert b'WEDDING_NOTICE_MINUTES: u64 = MINUTES_PER_YEAR' in core
assert b'other.expires_at_minute.is_after(now_minute)' in core
assert b'LEISURE_MORALE_STACK_CAP_MILLI.saturating_sub(other_live)' in core
marriage = contents['crates/adventuresim-stdb-module/src/relationship/marriage.rs']
resolution = marriage.split(b'fn resolve_marriage(', 1)[1].split(b'#[reducer]\npub fn end_marriage', 1)[0]
assert b'marriage.resolved_minute = Some(minute)' in resolution
assert b'marriage_participant()' in resolution and b'.delete(subject_id)' in resolution
assert b'character_kinship().id().delete(&id)' in resolution and b'morale_event()' not in resolution
assert b'require_strategic_character_authority(ctx, actor_id)' in marriage
assert b'TemporalScope::ExclusiveShared' in marriage
commitments = contents['crates/adventuresim-stdb-module/src/relationship/commitments.rs']
assert b'scheduled_from_minute.saturating_add_minutes(WEDDING_NOTICE_MINUTES)' in commitments
assert b'.marriage_participant()' in commitments
family = contents['crates/adventuresim-stdb-module/src/relationship/family.rs']
assert b'row.resolved_minute.is_none_or(|resolved| resolved > minute)' in family

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 20
by_path = {}
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    original = transport[row['original_path']]
    assert row['sha256'] == original['sha256'] and row['git_blob'] == original['git_blob']
    assert row['bytes'] == original['bytes'] and sha(raw) == row['published_sha256']
    assert len(raw) == row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    by_path[row['original_path']] = raw
for number, indices in ((274,[1180]), (275,[1181,1182,1183]), (276,[1184,1185,1186,1187])):
    name = f'segovb/READ-L12-{number}.json'; receipt = json.loads(by_path[name])
    assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
    for row in receipt['rows']:
        assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
        assert row['fresh_personal_body_reads'] == row['assessed_ranges']
        assert row['other_lens_verdict_transfer'] is False
    check = json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256'] == sha(by_path[name])
    assert check['state_sha256'] == sha(by_path[f'segovb/STATE-L12-{number}.json'])

histname = 'ASSESSMENT-L01-HISTORICAL-1180-1187.json'
history = load(histname)['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1180, 1188))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == histname and role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1180:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
q = load('RECONCILIATION-059.json')['entries']
for index, contract in [(224,'SEGOVB-L12-RESIDENCE-LEISURE-SOURCE-DISPATCH'),
                        (965,'Q-L12-c90948909bba7a8305cf'),
                        (966,'Q-L12-d29054d59a8cd7a315ce'),
                        (967,'Q-L12-0f6ddb4c5227797c70d3')]:
    assert q[index]['contract_id'] == contract and q[index]['current_reconciliation'].startswith('partial ')
progress = load('CONTINUATION-PROGRESS-95E.json')
assert 'ASSESSMENT-L12-RESIDENCE-LEISURE-95E.json' in progress['post_transition_bounded_string_traces']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified residence evidence, conditional cap trace and earlier attribution; no product ran')
