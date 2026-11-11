"""Verify selected source and earlier correction evidence without product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L12-CORRECTION-FRONTIER-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'CONT-REF-CORRECTION-SOURCE-NAMESPACE'
assert doc['source_revision'] == rev and doc['lens'] == 12 and doc['candidates'][0]['id'] == cid
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1); tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines'] and row['revision'] == rev
    assert all(1 <= a <= b <= row['lines'] for a, b in row['ranges'])
    contents[row['path'].split('/src/', 1)[-1]] = raw
local = contents['local_problem.rs']
writer = local.split(b'fn surface_public_threat(', 1)[1].split(b'\nfn surface_new_problem(', 1)[0]
assert b'"public-threat:{character_id}:{}"' in writer and b'learned_at: observer_minute' in writer
assert b'lead.exact_location_id != site.id.0' in writer and b'lead.corrected_by = disclosure_id.clone();' in writer
assert b'public_threat_disclosure().insert(disclosure)' in writer and b'.update(disclosure.clone());' in writer
assert b'disclose_exact_case_site(' in writer
sites = contents['investigation/sites.rs']
reader = sites.split(b'fn lead_is_live_at(', 1)[1].split(b'\npub(crate) fn exact_case_site_for_observer_at(', 1)[0]
assert b'.investigation_lead()' in reader and b'.find(&lead.corrected_by)' in reader
assert b'correction_recorded_at.is_some()' in reader
assert b'public_threat_disclosure()' not in reader
assert b'recorded_at <= minute && correction_recorded_at.is_none_or(|corrected_at| corrected_at > minute)' in sites
actor = contents['world_actor/presence.rs']
view = actor.split(b'fn exact_case_site_visible_to_observer_view(', 1)[1].split(b'\n}', 1)[0]
assert b'.investigation_lead()' in view and b'.find(&lead.corrected_by)' in view
assert b'correction.recorded_at > minute' in view and b'public_threat_disclosure()' not in view
assert b'exact_case_site_for_observer_at(' in actor and b'exact_case_site_visible_to_observer_view(' in actor
model = contents['investigation/model.rs']
assert b'pub corrected_by: String' in model and b'pub recorded_at: StrategicMinute' in model
assert b'source_id: format!("public-threat:{public_case_id}")' in model
claims = contents['investigation/claims.rs']
assert b'return "a later account".into();' in claims
assert b'.find(&lead.corrected_by)' in contents['investigation/projections.rs']
# The time example is symbolic ordering only, not an invocation of a product function.
assert 100 <= 150 < 200
transport = {x['path']: x for x in json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 26
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes()); original = transport[row['original_path']]
    assert row['sha256'] == original['sha256'] and row['git_blob'] == original['git_blob']
    assert row['bytes'] == original['bytes'] and sha(raw) == row['published_sha256'] and len(raw) == row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
assert not any(row['redacted_local_path_occurrences'] for row in records)
by_path = {x['original_path']: x for x in records}
base = 'evidence/continuation-segovb/audit-resume-20261027/'
def historical(path):
    return json.loads(gzip.decompress((root / by_path[path]['preserved_bytes']).read_bytes()))
batch = historical(base + 'lens11/batch-085-receipt.json')
adoption = historical(base + 'ADOPTION-054.json')
assert batch['lens'] == 11 and batch['accepted_unique_count_change'] == 0
assert batch['current_reconciled'] is False and batch['validated'] is False
assert any('correction-source-namespace' in f['source_key'] for f in batch['findings'])
assert adoption['accepted_unique_count_change'] == 0 and adoption['implementation_gate_complete'] is False
for num in (254, 260, 323):
    receipt = historical(f'segovb/READ-L12-{num}.json')
    assert receipt['lens'] == 12 and receipt['accepted_unique_count_change'] == 0
    assert receipt['current_source_reconciled'] is False and receipt['implementation_gate_complete'] is False
history = load('ASSESSMENT-L01-HISTORICAL-1139-1140.json')['historical_source_roles']
assert [r['original_index'] for r in history] == [1139, 1140]
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        assert roles[row['original_index']]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1139-1140.json'
        assert roles[row['original_index']]['status'] == 'full-historical-body-trace-open'
q = load('RECONCILIATION-059.json')['entries'][15]
assert q['contract_id'] == cid and q['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L12-CORRECTION-FRONTIER-95E.json']) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified correction source, historical attribution and archived roles; no product ran')
