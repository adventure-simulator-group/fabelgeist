#!/usr/bin/env python3
"""Verify qualified post-a9d audit evidence, not product behavior."""
import hashlib
import json
import os
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')

R = Path(__file__).resolve().parent
P = R / 'delegated/posta9d-followups'
H = lambda b: hashlib.sha256(b).hexdigest()
load = lambda p: json.loads(p.read_bytes())


def packet(directory, manifest, expected, self_manifest=True):
    raw = manifest.read_bytes()
    assert H(raw) == expected
    rows = json.loads(raw)['files']
    if isinstance(rows, dict):
        rows = [{'path': name, **details} for name, details in rows.items()]
    seen = set()
    for row in rows:
        path = PurePosixPath(row['path'])
        assert not path.is_absolute() and '..' not in path.parts
        name = path.as_posix()
        assert name not in seen
        seen.add(name)
        file = directory / path
        assert file.is_file() and not file.is_symlink()
        data = file.read_bytes()
        assert (len(data), H(data)) == (row['bytes'], row['sha256'])
    all_paths = list(directory.rglob('*'))
    assert all(not path.is_symlink() for path in all_paths)
    actual = {path.relative_to(directory).as_posix() for path in all_paths
              if path.is_file()}
    expected_paths = seen | ({'MANIFEST.json'} if self_manifest else set())
    assert actual == expected_paths
    return expected_paths


catalog_bytes = (P / 'BUNDLE-CATALOG.json').read_bytes()
assert H(catalog_bytes) == '444d8a5324afd390c9065f198bdd426c113ce3548f6762c5a7a865877be9487d'
catalog = json.loads(catalog_bytes)
assert len(catalog['packets']) == 13
for row in catalog['packets']:
    name = row['name']
    directory = P / name
    seen = packet(directory, directory / 'MANIFEST.json', row['manifest_sha256'])
    assert len(seen) == row['members']
    archive = P / (name + '-archive.tar.gz')
    assert H(archive.read_bytes()) == row['archive_sha256']
    with tarfile.open(archive, 'r:gz') as source:
        found = set()
        for member in source.getmembers():
            path = PurePosixPath(member.name)
            assert member.isfile() and not path.is_absolute() and '..' not in path.parts
            prefix = row['archive_prefix']
            if prefix:
                assert path.parts[0] == prefix
                path = PurePosixPath(*path.parts[1:])
            rel = path.as_posix()
            assert rel in seen and rel not in found
            found.add(rel)
            assert source.extractfile(member).read() == (directory / rel).read_bytes()
        assert found == seen

C = P / 'coordinator'
packet(C, C / 'MANIFEST.json', catalog['coordinator_manifest_sha256'])
packet(P / 'agent-scans', P / 'AGENT-SCANS-MANIFEST.json',
       catalog['agent_scans_manifest_sha256'], self_manifest=False)
for name, key in [('c7dc-tree-census', 'c7dc_tree_census_manifest_sha256'),
                  ('a9d-recovery', 'a9d_recovery_manifest_sha256')]:
    packet(P / name, P / name / 'MANIFEST.json', catalog[key])
for name, digest in catalog['a9d_recovery_scan_sha256'].items():
    assert H((P / 'a9d-recovery-scan' / name).read_bytes()) == digest

for projection in ['REPORT-REGISTRATION-PROJECTION.json',
                   'LIVE-INDEPENDENT-REGISTRATION-PROJECTION.json']:
    for row in load(C / projection)['reports']:
        data = (P / row['logical_path']).read_bytes()
        assert (len(data), H(data)) == (row['bytes'], row['sha256'])
row = load(C / 'LIVE-REPORT-REGISTRATION-PROJECTION.json')
data = (P / row['logical_path']).read_bytes()
assert (len(data), H(data)) == (row['bytes'], row['sha256'])

for row in load(C / 'replays-eight/RECORD.json')['cases']:
    data = (C / 'replays-eight' / (row['case'] + '.log')).read_bytes()
    assert row['exit'] == 0 and (len(data), H(data)) == (row['bytes'], row['sha256'])
for name in ['replay-q2314', 'replay-q2315', 'replay-live-independent',
             'replay-live-same-author', 'replay-live-same-author-optional']:
    directory = C / name
    row = load(directory / 'RECORD.json')
    data = (directory / 'VERIFICATION.log').read_bytes()
    assert row['exit'] == 0 and (len(data), H(data)) == (row['bytes'], row['sha256'])

for file in [C / 'PREIMPORT-DIGEST-SCAN.json',
             P / 'a9d-recovery-scan/digest.json']:
    scan = load(file)
    assert scan['forbidden_digest_total'] == 0
    assert not scan['other_candidate_counts'] and not scan['exceptions']
classification = load(C / 'MHR-LIB-PRIVACY-CLASSIFICATION.json')
assert classification['drive_qualified_token_count'] == 1
assert not classification['contains_current_account_name']
assert not classification['contains_user_home_component']
assert not classification['contains_email_syntax']
assert 'unknown' in classification['classification']

recovery = load(P / 'a9d-recovery/RECOVERY-RECORD.json')
data = (P / 'a9d-recovery/evidence.log').read_bytes()
assert data == (C / 'a9d-prepublication-evidence.log').read_bytes()
assert H(data) == recovery['transcript_sha256'] == '3b18ff765be79713260afed4ea0b3dd21a931bd55d97f498b289f647ca82f4da'
assert recovery['recovery_driver_exit'] == 0 and recovery['post_replay_clean']
assert 'same machine' in recovery['qualification']

controlled = R / 'delegated/a9d-assert-enabled-replay'
manifest_hash = 'c667eb85089717a3dcf477ddddcfcdb37fbaa54aea4724d69ea61b39b21a334f'
packet(controlled, controlled / 'MANIFEST.json', manifest_hash)
result = load(controlled / 'RESULT.json')
controls = load(controlled / 'ASSERT-CONTROLS.json')
assert controls['direct_child_preflight'] == {'assertions_enabled': True, 'optimization_level': 0}
assert load(controlled / 'ASSERT-PREFLIGHT.log') == controls['direct_child_preflight']
assert result['before_tree_exit'] == result['driver_exit'] == result['after_tree_exit'] == 0
assert result['inventories_equal'] and result['discovery_equal'] and result['final_status_clean']
assert not result['external_tmp_residue']
assert (controlled / 'before/INVENTORY.json').read_bytes() == (controlled / 'after/INVENTORY.json').read_bytes()
assert H((controlled / 'before/INVENTORY.json').read_bytes()) == result['before_inventory_sha256'] == result['after_inventory_sha256']
assert (controlled / 'DISCOVERY-BEFORE.json').read_bytes() == (controlled / 'DISCOVERY-AFTER.json').read_bytes()
assert H((controlled / 'DISCOVERY-BEFORE.json').read_bytes()) == result['discovery_before_sha256'] == result['discovery_after_sha256']
assert (controlled / 'evidence.log').read_bytes() == data
assert result['transcript_sha256'] == H(data) and result['final_head'] == recovery['checkpoint']
assert H((controlled / 'proof.py.txt').read_bytes()) == load(controlled / 'PACKAGING.json')['original_proof_script_sha256']
scan_dir = R / 'delegated/a9d-assert-enabled-replay-scan'
scan_record_bytes = (scan_dir / 'RECORD.json').read_bytes()
assert H(scan_record_bytes) == '8d874297701591a985e5114a39cff75b38fea51f245f4722ecbbeed2f576de17'
scan_record = json.loads(scan_record_bytes)
assert scan_record['packet_manifest_sha256'] == manifest_hash
assert scan_record['digest_exit'] == scan_record['pattern_exit'] == 0
assert H((scan_dir / 'digest.json').read_bytes()) == scan_record['digest_scan_sha256']
assert H((scan_dir / 'pattern.log').read_bytes()) == scan_record['pattern_scan_sha256']
scan = load(scan_dir / 'digest.json')
assert not scan['forbidden_digest_total'] and not scan['other_candidate_counts'] and not scan['exceptions']

evolution = R / 'delegated/posta9d-central-application'
raw = (evolution / 'EVOLUTION.json').read_bytes()
assert H(raw) == 'cb72451272fdf887198c5ab0fdada3b6cbdbcb3393ef13f5e064bb6561ec3239'
record = json.loads(raw)
for name, key in [('apply-initial.py.txt', 'initial_exact_script_sha256'),
                  ('OUTPUT-initial-redacted.log', 'initial_public_redacted_log_sha256'),
                  ('apply-successor.py.txt', 'successor_script_sha256'),
                  ('OUTPUT-successor.log', 'successor_log_sha256')]:
    assert H((evolution / name).read_bytes()) == record[key]
assert record['successor_exit'] == 0 and 'TypeError' in record['initial_failure']
assert {p.name for p in evolution.iterdir()} == {
    'EVOLUTION.json', 'apply-initial.py.txt', 'OUTPUT-initial-redacted.log',
    'apply-successor.py.txt', 'OUTPUT-successor.log'}

raw = (R / 'delegated/posta9d-diff-treatment/RECORD.json').read_bytes()
assert H(raw) == 'a97b9f6c82b196da53e819d1e91b49848ab1a0f9a893ba70b3ea3fb017ab0a1a'
diff_record = json.loads(raw)
assert diff_record['initial_diff_check_exit'] == 2 and len(diff_record['exact_paths']) == 102
attributes = (R / '.gitattributes').read_text().splitlines()
assert all(attributes.count(path + ' binary') == 1 for path in diff_record['exact_paths'])

snapshot_bytes = (R / 'FOLLOWUP-POSTA9D-EVIDENCE.json').read_bytes()
assert H(snapshot_bytes) == '830a2f2d6448c2b8e2ab357135041641e7060f378ba22dce1eb64cea7bfad2c9'
snapshot = json.loads(snapshot_bytes)
counts = snapshot['observed_counts']
assert (counts['lenses'], counts['lenses_complete'], counts['queue_records'],
        counts['queue_semantically_reconciled'],
        counts['current_ledger_entries_and_relationships']) == (13, 0, 3713, 0, 1041)
assert H((C / 'LEDGER-RELATIONSHIPS-SNAPSHOT.json').read_bytes()) == snapshot['ledger_relationships_snapshot_sha256']
for root_name, coordinator_name, hash_key in [
    ('ASSESSMENT-CENTRAL-FOLLOWUPS-14A-95E.json', 'CENTRAL-REPORT-SNAPSHOT.json', 'central_report_sha256'),
    ('LIVE-STATE-REPLAY-QUALIFICATION-14A.json', 'LIVE-DESIGN-SNAPSHOT.json', 'live_design_qualification_sha256')]:
    data = (R / root_name).read_bytes()
    assert data == (C / coordinator_name).read_bytes() and H(data) == snapshot[hash_key]

baseline = load(R / 'BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
progress = load(R / 'CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger)
for identifier in snapshot['added_ledger_relationship_ids']:
    found = [row for row in ledger if row.get('id') == identifier]
    assert len(found) == 1 and found[0]['record'] == 'ASSESSMENT-CENTRAL-FOLLOWUPS-14A-95E.json'
queue = load(R / 'RECONCILIATION-059.json')['entries']
for row in snapshot['queue_selections']:
    assert queue[row['index']]['contract_id'] == row['contract_id']
    assert row['current_reconciliation_at_snapshot'].startswith('partial')

env = os.environ.copy()
env.pop('PYTHONOPTIMIZE', None)
for name in ['q2304', 'q2305', 'q2309', 'q2310', 'q2311', 'q2314',
             'live-replay-same-author', 'live-replay-independent']:
    subprocess.run([sys.executable, '-B', str(P / name / 'VERIFY-EVIDENCE.py')],
                   cwd=R, env=env, check=True)
print('Bound packet bytes, registered reports, recorded replays and time-bound relationships.')
print('Dependency-rich replays remain same-machine receipts; separately held inputs were not replayed here.')
print('The drive-qualified source example and bounded scans retain their explicit privacy limits.')
print('No product behavior, independent custody, checkpoint3821 recovery or audit completion is asserted.')
