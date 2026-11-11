"""Verify recovered source and receipt predicates without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
doc = load('ASSESSMENT-L09-INVESTIGATION-RECEIPT-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'SEGOVB-L12-INVESTIGATION-PRIVATE-RECEIPT-OWNER'
assert doc['source_revision'] == rev and doc['lens'] == 9 and doc['candidates'][0]['id'] == cid
assert load('baseline-059/queue.json')['entries'][211]['contract_id'] == cid
entries = [e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'] if e['id'] == cid]
assert len(entries) == 1 and entries[0]['record'] == 'ASSESSMENT-L09-INVESTIGATION-RECEIPT-95E.json'
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    metadata, path = line.split('\t', 1)
    tree[path] = metadata.split()[2]
contents = {}
for source in doc['current_sources']:
    raw = gzip.decompress((root / source['preserved_bytes']).read_bytes())
    assert sha(raw) == source['sha256'] and blob(raw) == source['git_blob'] == tree[source['path']]
    assert len(raw) == source['bytes'] and len(raw.splitlines()) == source['lines']
    assert source['revision'] == rev and all(1 <= a <= b <= source['lines'] for a,b in source['ranges'])
    contents[source['path'].split('/src/', 1)[-1]] = raw
reader = contents['investigation/actions.rs'].split(b'fn failed_attempt_history_kind(',1)[1].split(b'fn contiguous_failed_attempts(',1)[0]
assert b'object.len() == 5' in reader and b'.get("resolution")' in reader
assert b'serde_json::from_value::<action::Resolution>' in reader
assert b'serde_json::from_value::<action::BoundedProgressResolution>' not in reader
for field in (b'attempt_number', b'persistent_progress_bps', b'success_threshold_bps', b'guaranteed_by_attempt'):
    assert b'"'+field+b'"' in reader
    assert b'.get("'+field+b'")' not in reader
core = contents['investigation_action.rs']
progress = core.split(b'pub struct BoundedProgressResolution {',1)[1].split(b'}',1)[0]
for declaration in (b'pub attempt_number: u32',b'pub persistent_progress_bps: u16',b'pub success_threshold_bps: u16',b'pub guaranteed_by_attempt: u32'):
    assert declaration in progress
assert b'GENERATED_ACTION_PROGRESS_BPS_PER_FAILURE: u16 = 1_900' in core
assert b'"source": "exact_referred_witness_dialogue"' in contents['investigation/capabilities.rs']
assert b'attempt_success: true' in contents['quest_generation/model.rs']
assert b'pub private_resolution_json: String' in contents['investigation/model.rs']
assert b'"{}".into()' in contents['investigation/tests/projection_and_referrals.rs']
# Compare the source predicate's key set with a symbolic data illustration;
# this performs no JSON deserialization or product function call.
payload = doc['symbolic_invalid_bounded_payload']
keys = {'resolution','attempt_number','persistent_progress_bps','success_threshold_bps','guaranteed_by_attempt'}
assert set(payload) == keys and payload['resolution']['success'] is False
assert all(payload[k] is None for k in keys - {'resolution'})
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert sha(raw) == search['sha256'] and len(raw) == search['bytes']
assert len(raw.splitlines()) == search['lines'] == 17
manifest = {e['path']: e for e in json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_receipts'] + doc['recovered_historical_support']
assert len(records) == 12
for record in records:
    raw = gzip.decompress((root / record['preserved_bytes']).read_bytes())
    m = manifest[record['original_path']]
    assert sha(raw) == record['sha256'] == m['sha256']
    assert blob(raw) == record['git_blob'] == m['git_blob'] and len(raw) == record['bytes'] == m['bytes']
old = [json.loads(gzip.decompress((root / r['preserved_bytes']).read_bytes())) for r in doc['recovered_historical_receipts']]
assert [row['original_index'] for row in old[0]['rows']] == [1120,1121,1122]
assert [row['original_index'] for row in old[1]['rows']] == [1123]
assert all(x['current_source_reconciled'] is False for x in old)
historical = load('ASSESSMENT-L01-HISTORICAL-1120-1122.json')['historical_source_roles']
assert [r['original_index'] for r in historical] == [1120,1121,1122]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for role in historical:
        raw = archive.extractfile(role['sha256']).read()
        assert sha(raw) == role['sha256'] and len(raw.splitlines()) == role['lines']
        assert role['new_personal_read_ranges'] == [[1, role['lines']]]
        assert role['current_comparison']['byte_identical'] is True
        assert role['current_comparison']['sha256'] == role['sha256']
        assert role['current_comparison']['source_revision'] == rev
        assert blob(raw) == tree[role['source_id'].split(':',1)[1]]
queue = load('RECONCILIATION-059.json')['entries']
for i in (211,921,925): assert queue[i]['current_reconciliation'].startswith('partial ')
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['queue_records_semantically_reconciled'] == 0
assert 'ASSESSMENT-L09-INVESTIGATION-RECEIPT-95E.json' in p['post_transition_bounded_enum_shape_traces']
print('verified selected receipt evidence and symbolic shape; no product ran')
