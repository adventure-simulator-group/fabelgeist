#!/usr/bin/env python3
"""Verify qualified post-e709 audit evidence without executing product code."""
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
P = R / 'delegated/poste709-followups'
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


catalog_bytes = (P / 'CATALOG.json').read_bytes()
assert H(catalog_bytes) == '5db3d852459561ea0e4fb56b754f4f38f3f6a8bf9ec48f3163c13ff2220638fe'
catalog = json.loads(catalog_bytes)
packets = {row['name']: row for row in catalog['packets']}
assert set(packets) == {'q2317', 'q2318', 'q2319', 'q2320',
                        'q2321', 'q2322', 'q2323', 'q2324'}
for name, row in packets.items():
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
            if row['archive_prefix']:
                assert path.parts[0] == row['archive_prefix']
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
packet(P / 'checkpoint-e709-recovery', P / 'checkpoint-e709-recovery/MANIFEST.json',
       catalog['recovery_manifest_sha256'])
for name, digest in catalog['provisional_catalog_sha256'].items():
    assert H((P / name).read_bytes()) == digest
top = {'CATALOG.json', 'AGENT-SCANS-MANIFEST.json', 'agent-scans', 'coordinator',
       'checkpoint-e709-recovery', 'checkpoint-e709-recovery-scan'}
top.update(catalog['provisional_catalog_sha256'])
top.update(packets)
top.update(name + '-archive.tar.gz' for name in packets)
assert {path.name for path in P.iterdir()} == top

for row in load(C / 'REPORT-CATALOG.json')['reports']:
    data = (P / row['logical_path']).read_bytes()
    assert (len(data), H(data)) == (row['bytes'], row['sha256'])
    registration = load(C / row['registration'])
    candidates = registration.get('reports', [registration])
    assert any(x['logical_path'] == row['logical_path'] and x['sha256'] == row['sha256']
               and x['bytes'] == row['bytes'] for x in candidates)
for row in load(C / 'REPLAY-CATALOG.json')['cases']:
    data = (C / row['log']).read_bytes()
    assert row['exit'] == 0 and (len(data), H(data)) == (row['bytes'], row['sha256'])
    record = load(C / row['record'])
    candidates = record.get('cases', [record])
    assert any(x['case'] == row['case'] and x['exit'] == row['exit']
               and x['sha256'] == row['sha256'] and x['bytes'] == row['bytes']
               for x in candidates)

for name in ['q2317', 'q2321', 'q2322']:
    receipt = load(P / 'agent-scans' / name / 'RECEIPT.json')
    assert receipt['manifest_sha256'] == packets[name]['manifest_sha256']
    assert receipt['verification']['exit_code'] == 0
    scan = receipt['digest_scan']
    assert scan['exit_code'] == 0 and not scan['forbidden_digest_total']
    assert not scan['other_candidate_counts'] and not scan['exceptions']
    scan = receipt['pattern_scan']
    assert scan['exit_code'] == 0 and not any(scan['counts'].values())
for name in ['q2318', 'q2319', 'q2323']:
    receipt = load(P / 'agent-scans' / name / 'RECEIPT.json')
    assert receipt['packet_manifest_sha256'] == packets[name]['manifest_sha256']
    assert all(value == 0 for value in receipt['results'].values())
for name in ['q2320', 'q2324']:
    directory = P / 'agent-scans' / name
    seal = load(directory / 'SEAL.json')
    assert seal['manifest_sha256'] == packets[name]['manifest_sha256']
    assert seal['archive_sha256'] == packets[name]['archive_sha256']
    for file, key in [('PACKET-SCAN.json', 'packet_scan_sha256'),
                      ('ARCHIVE-SCAN.json', 'archive_scan_sha256')]:
        raw = (directory / file).read_bytes()
        assert H(raw) == seal[key]
        scan = json.loads(raw)
        assert not scan['forbidden_digest_total']
        assert not scan['other_candidate_counts'] and not scan['exceptions']

preimport = R / 'delegated/poste709-preimport-scan'
raw = (preimport / 'RECORD.json').read_bytes()
assert H(raw) == '6c0eed4325ed98e99708671a60445cbf67ea9f3095b69fb22049caa91c3e9264'
rec = json.loads(raw)
assert rec['catalog_sha256'] == H(catalog_bytes) and rec['imported_bytes_match_prepared']
assert rec['digest_exit'] == rec['pattern_exit'] == 0
assert H((preimport / 'digest.json').read_bytes()) == rec['digest_sha256']
assert H((preimport / 'pattern.log').read_bytes()) == rec['pattern_sha256']
scan = load(preimport / 'digest.json')
assert not scan['forbidden_digest_total'] and not scan['other_candidate_counts'] and not scan['exceptions']
files = sorted(path.relative_to(P).as_posix() for path in P.rglob('*') if path.is_file())
aggregate = hashlib.sha256()
for name in files:
    aggregate.update(name.encode() + b'\0' + H((P / name).read_bytes()).encode() + b'\n')
assert len(files) == rec['files'] and aggregate.hexdigest() == rec['aggregate_names_and_sha256']

recovery_dir = P / 'checkpoint-e709-recovery'
recovery = load(recovery_dir / 'RECOVERY-RECORD.json')
log = (recovery_dir / 'evidence.log').read_bytes()
assert log == (C / 'checkpoint-e709-prepublication/evidence.log').read_bytes()
assert H(log) == recovery['transcript_sha256'] == 'eef2aece71ff86c017cd8689ac6dd2c9fb80e7d9bcb65830672599a139be975e'
assert recovery['recovery_driver_exit'] == 0 and recovery['post_replay_clean']
assert recovery['symbolic_ref_exit_after'] == 1 and recovery['prepublication_and_recovery_transcripts_byte_identical']
assert recovery['assertion_preflight'] == {'assertions_enabled': True, 'optimization_level': 0}
assert load(recovery_dir / 'ASSERT-PREFLIGHT.log') == recovery['assertion_preflight']
assert 'same machine' in recovery['qualification']
scan_dir = P / 'checkpoint-e709-recovery-scan'
raw = (scan_dir / 'RECORD.json').read_bytes()
assert H(raw) == catalog['recovery_scan_record_sha256']
rec = json.loads(raw)
assert rec['manifest_sha256'] == catalog['recovery_manifest_sha256']
assert rec['digest_exit'] == rec['pattern_exit'] == 0
assert H((scan_dir / 'digest.json').read_bytes()) == rec['digest_sha256']
assert H((scan_dir / 'pattern.log').read_bytes()) == rec['pattern_sha256']
scan = load(scan_dir / 'digest.json')
assert not scan['forbidden_digest_total'] and not scan['other_candidate_counts'] and not scan['exceptions']

correction = load(C / 'Q2323-LEDGER-ID-CORRECTION.json')
assert H((C / 'CENTRAL-14B-Q2323-ADDENDUM.json').read_bytes()) == correction['initial_draft_sha256']
assert H((C / 'CENTRAL-14B-Q2323-ADDENDUM-2.json').read_bytes()) == correction['corrected_draft_sha256']
application = R / 'delegated/poste709-central-application'
raw = (application / 'RECORD.json').read_bytes()
assert H(raw) == '599d9d0746978a2e7c807684003ad09f8ee6485ecc89e21fbdb0d1cf2931c351'
record = json.loads(raw)
assert record['exit'] == 0 and H((application / 'apply.py.txt').read_bytes()) == record['script_sha256']
assert H((application / 'OUTPUT.log').read_bytes()) == record['output_sha256']
assert {p.name for p in application.iterdir()} == {'RECORD.json', 'apply.py.txt', 'OUTPUT.log'}

incidents = R / 'delegated/poste709-collaboration-incidents'
assert H((incidents / 'IMPORT-FAILURE.json').read_bytes()) == 'c7dd2bee7a9af56539ac019e68ed9d37cfb42905f6076e26be3eba4c1c10be49'
assert H((incidents / 'STAGED-SCAN-SUPERSESSION.json').read_bytes()) == 'a8d75043b767e6a092612232de36b970e6cf20080db727566f4f877298cd50ae'
for name, expected, files in [
    ('q2330', '53b02fc742e9fc5f522b425da51990fa7b874d582d73776a45848eaa949f1e4f',
     [('INCIDENT.json', 'incident_sha256'), ('REGISTER-SOURCE.py.txt', 'helper_sha256')]),
    ('q2328', 'b011e431eb77d5bc40bd0d54a7f8e67063aefdf0aa8d1c2dd00bcc6806850ddf',
     [('INCIDENT-PROJECTION.json', 'projection_sha256'),
      ('POST-EVENT-OBSERVATION.json', 'observation_sha256'),
      ('SEPARATELY-DECODED-CONNECTOR.json', 'separate_connector_sha256')])]:
    directory = incidents / (name + '-scan')
    raw = (directory / 'RECORD.json').read_bytes()
    assert H(raw) == expected
    record = json.loads(raw)
    for file, key in files:
        assert H((incidents / name / file).read_bytes()) == record[key]
    assert record['digest_exit'] == record['pattern_exit'] == 0
    assert H((directory / 'digest.json').read_bytes()) == record['digest_sha256']
    assert H((directory / 'pattern.log').read_bytes()) == record['pattern_sha256']
    scan = load(directory / 'digest.json')
    assert not scan['forbidden_digest_total'] and not scan['other_candidate_counts'] and not scan['exceptions']
projection = load(incidents / 'q2328/INCIDENT-PROJECTION.json')
assert not projection['copied_from_accidental_path_before_removal']
assert projection['separately_decoded_sha256'] == H((incidents / 'q2328/SEPARATELY-DECODED-CONNECTOR.json').read_bytes())
assert 'cannot be established' in projection['recovery_limit']
supplement = incidents / 'q2328-supplement'
supplement_scan = incidents / 'q2328-supplement-scan'
raw = (supplement_scan / 'RECORD.json').read_bytes()
assert H(raw) == 'a9cb8e51936a20a6bb422163314f6d4e2b3c3ce96deea742be6451527cff4e03'
record = json.loads(raw)
assert {p.name for p in supplement.iterdir()} == set(record['files'])
assert all(H((supplement / name).read_bytes()) == digest for name, digest in record['files'].items())
assert record['digest_exit'] == record['pattern_exit'] == 0
assert H((supplement_scan / 'digest.json').read_bytes()) == record['digest_sha256']
assert H((supplement_scan / 'pattern.log').read_bytes()) == record['pattern_sha256']
scan = load(supplement_scan / 'digest.json')
assert not scan['forbidden_digest_total'] and not scan['other_candidate_counts'] and not scan['exceptions']
boundary = load(supplement / 'PRIVACY-BOUNDARY.json')
assert H((supplement / 'ORIGINAL-MANIFEST.json').read_bytes()) == boundary['original_manifest_sha256']
assert H((supplement / 'REGISTRATION-PROJECTION.json').read_bytes()) == boundary['public_registration_sha256']

raw = (R / 'delegated/poste709-diff-treatment/RECORD.json').read_bytes()
assert H(raw) == '9b4437c68830c85a16f968bec85652d729e4e6e6d5b151535700e2f46814bf10'
diff_record = json.loads(raw)
assert diff_record['initial_diff_check_exit'] == 2 and len(diff_record['exact_paths']) == 202
attributes = (R / '.gitattributes').read_text().splitlines()
assert all(attributes.count(path + ' binary') == 1 for path in diff_record['exact_paths'])
assert diff_record['qualified_historical_original'] in diff_record['exact_paths']

snapshot_bytes = (R / 'FOLLOWUP-POSTE709-EVIDENCE.json').read_bytes()
assert H(snapshot_bytes) == '5adad4408e0b0681f616bc292f2ca89464a3c1101d10f7168bf9983abe8e4251'
snapshot = json.loads(snapshot_bytes)
counts = snapshot['observed_counts']
assert (counts['lenses'], counts['lenses_complete'], counts['queue_records'],
        counts['queue_semantically_reconciled'],
        counts['current_ledger_entries_and_relationships']) == (13, 0, 3713, 0, 1051)
report_bytes = (R / 'ASSESSMENT-CENTRAL-FOLLOWUPS-14B-95E.json').read_bytes()
assert report_bytes == (C / 'CENTRAL-14B-SYNTHESIS-V2.json').read_bytes()
assert H(report_bytes) == snapshot['central_report_sha256'] == catalog['central_report_sha256']
assert snapshot['bundle_catalog_sha256'] == H(catalog_bytes)
assert snapshot['e709_recovery_manifest_sha256'] == catalog['recovery_manifest_sha256']
ledger = load(R / 'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert load(R / 'CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
for identifier in snapshot['added_ledger_relationship_ids']:
    found = [row for row in ledger if row.get('id') == identifier]
    assert len(found) == 1 and found[0]['record'] == 'ASSESSMENT-CENTRAL-FOLLOWUPS-14B-95E.json'
queue = load(R / 'RECONCILIATION-059.json')['entries']
for row in snapshot['queue_selections']:
    assert queue[row['index']]['contract_id'] == row['contract_id']
    assert row['current_reconciliation_at_snapshot'].startswith('partial')

env = os.environ.copy()
env.pop('PYTHONOPTIMIZE', None)
for name in ['q2318', 'q2319', 'q2320', 'q2323', 'q2324']:
    subprocess.run([sys.executable, '-B', str(P / name / 'VERIFY-EVIDENCE.py')],
                   cwd=R, env=env, check=True)
print('Bound eight qualified packets, recorded replays, recovery and time-bound relationships.')
print('Three queue-dependent verifiers and other optional original inputs are represented by same-machine receipts, not rerun here.')
print('No product behavior, independent custody, checkpoint3821 recovery or audit completion is asserted.')
