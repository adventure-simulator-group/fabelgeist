#!/usr/bin/env python3
"""Verify qualified post-b147 audit evidence without executing product code."""
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
D = R / 'delegated'
P = D / 'poste-b147-followups'
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
    paths = list(directory.rglob('*'))
    assert all(not p.is_symlink() for p in paths)
    actual = {p.relative_to(directory).as_posix() for p in paths if p.is_file()}
    assert actual == seen | ({'MANIFEST.json'} if self_manifest else set())
    return actual


raw = (P / 'CATALOG.json').read_bytes()
assert H(raw) == 'c18a2aa5ce8c14405208bf27e9a8e5d668e6a1efeafe4efa48a8792c43cc7d4d'
catalog = json.loads(raw)
assert catalog['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert catalog['parent_checkpoint'] == 'b147e0caaf982e0adc765f188353152db994b184'
packets = {row['name']: row for row in catalog['packets']}
assert set(packets) == {'q2325', 'q2326', 'q2328', 'q2329', 'q2330'}
for name, row in packets.items():
    directory = P / name
    files = packet(directory, directory / 'MANIFEST.json', row['manifest_sha256'])
    assert len(files) == row['members']
    archive = P / row['archive']
    assert H(archive.read_bytes()) == row['archive_sha256']
    found = set()
    with tarfile.open(archive, 'r:gz') as source:
        for member in source.getmembers():
            path = PurePosixPath(member.name)
            assert member.isfile() and not path.is_absolute() and '..' not in path.parts
            assert path.parts[0] == name
            rel = PurePosixPath(*path.parts[1:]).as_posix()
            assert rel in files and rel not in found
            found.add(rel)
            assert source.extractfile(member).read() == (directory / rel).read_bytes()
    assert found == files
C = P / 'coordinator'
packet(C, C / 'MANIFEST.json', catalog['coordinator_manifest_sha256'])
packet(P / 'agent-scans', P / 'AGENT-SCANS-MANIFEST.json',
       catalog['agent_scans_manifest_sha256'], self_manifest=False)
for name, key in [('checkpoint-b147-recovery', 'recovery_manifest_sha256'),
                  ('checkpoint-b147-prepublication', 'prepublication_manifest_sha256')]:
    packet(P / name, P / name / 'MANIFEST.json', catalog[key])
top = {'CATALOG.json', 'AGENT-SCANS-MANIFEST.json', 'agent-scans', 'coordinator',
       'checkpoint-b147-recovery', 'checkpoint-b147-prepublication',
       'q2333-temp-incident', 'q2333-temp-incident-scan'}
top.update(packets)
top.update(row['archive'] for row in packets.values())
assert {p.name for p in P.iterdir()} == top

for name in ['q2325', 'q2329']:
    d = P / 'agent-scans' / name
    receipt = load(d / 'RECEIPT.json')
    assert receipt['packet_manifest_sha256'] == packets[name]['manifest_sha256']
    assert all(value == 0 for value in receipt['results'].values())
    for file, digest in {**receipt['scanner_sha256'], **receipt['results_sha256']}.items():
        assert H((d / file).read_bytes()) == digest
q26 = load(P / 'agent-scans/q2326/SEAL.json')
assert q26['packet_manifest_sha256'] == packets['q2326']['manifest_sha256']
assert q26['archive_sha256'] == packets['q2326']['archive_sha256']
q30 = load(P / 'agent-scans/q2330/SEAL.json')
assert q30['packet_manifest_sha256'] == load(P / 'q2330/PRIVACY-BOUNDARY.json')['original_manifest_sha256']
assert q30['packet_manifest_sha256'] != packets['q2330']['manifest_sha256']
for name, path in [('original', 'original.log'), ('public-successor', 'public-successor.log')]:
    rec = next(x for x in load(C / 'q2328-replay/RECORD.json')['cases'] if x['case'] == name)
    data = (C / 'q2328-replay' / path).read_bytes()
    assert rec['exit'] == 0 and (len(data), H(data)) == (rec['bytes'], rec['sha256'])
rec = load(C / 'q2330-replay/RECORD.json')
data = (C / 'q2330-replay/VERIFICATION.log').read_bytes()
assert rec['exit'] == 0 and (len(data), H(data)) == (rec['bytes'], rec['sha256'])

recovery = P / 'checkpoint-b147-recovery'
pre = P / 'checkpoint-b147-prepublication'
r = load(recovery / 'RECOVERY-RECORD.json')
pre_result = load(pre / 'RESULT.json')
log = (recovery / 'evidence.log').read_bytes()
assert log == (pre / 'evidence.log').read_bytes()
assert H(log) == r['transcript_sha256'] == pre_result['transcript_sha256'] == 'b368727ca93736ea44ce61cbd81f9be2a01134b8d8a242a10702b9ffaec63c97'
assert r['recovery_driver_exit'] == pre_result['exit'] == 0 and r['post_replay_clean']
assert r['symbolic_ref_exit_after'] == 1 and r['prepublication_and_recovery_transcripts_byte_identical']
assert r['assertion_preflight'] == load(recovery / 'ASSERT-PREFLIGHT.log') == {'assertions_enabled': True, 'optimization_level': 0}
assert 'same machine' in r['qualification']

incident = P / 'q2333-temp-incident'
incscan = P / 'q2333-temp-incident-scan'
assert H((incident / 'INCIDENT-PROJECTION.json').read_bytes()) == catalog['temporary_incident_projection_sha256']
assert H((incscan / 'RECORD.json').read_bytes()) == catalog['temporary_incident_scan_record_sha256']
ir = load(incscan / 'RECORD.json')
assert ir['projection_sha256'] == catalog['temporary_incident_projection_sha256']
assert ir['digest_exit'] == ir['pattern_exit'] == 0
assert H((incscan / 'digest.json').read_bytes()) == ir['digest_sha256']
assert H((incscan / 'pattern.log').read_bytes()) == ir['pattern_sha256']
assert not load(incscan / 'digest.json')['forbidden_digest_total']
ip = load(incident / 'INCIDENT-PROJECTION.json')
assert ip['pre_write_existence_check'] == 'none' and not ip['original_now_exists']
assert 'cannot establish or recover' in ip['prior_content_qualification']

scan = D / 'poste-b147-preimport-scan'
assert H((scan / 'RECORD.json').read_bytes()) == catalog['preimport_scan_record_sha256']
sr = load(scan / 'RECORD.json')
assert sr['digest_exit'] == sr['pattern_exit'] == 0
assert H((scan / 'digest.json').read_bytes()) == sr['digest_sha256']
assert H((scan / 'pattern.log').read_bytes()) == sr['pattern_sha256']
digest = load(scan / 'digest.json')
assert not digest['forbidden_digest_total'] and not digest['other_candidate_counts'] and not digest['exceptions']
files = sorted(p for p in P.rglob('*') if p.is_file() and p != P / 'CATALOG.json')
aggregate = hashlib.sha256()
for p in files:
    aggregate.update(p.relative_to(P).as_posix().encode() + b'\0' + H(p.read_bytes()).encode() + b'\n')
assert len(files) == sr['files'] and aggregate.hexdigest() == sr['aggregate_names_and_sha256']

application = D / 'postb147-central-application'
assert H((application / 'RECORD.json').read_bytes()) == '1bb2a00aa940d6de67b984fadd617de498722094c9e1e154ee309e207b361a9c'
ar = load(application / 'RECORD.json')
assert ar['exit'] == 0 and H((application / 'apply.py.txt').read_bytes()) == ar['script_sha256']
assert H((application / 'OUTPUT.log').read_bytes()) == ar['output_sha256']
assert {p.name for p in application.iterdir()} == {'RECORD.json', 'apply.py.txt', 'OUTPUT.log'}

snapraw = (R / 'FOLLOWUP-POSTB147-EVIDENCE.json').read_bytes()
assert H(snapraw) == 'afb3a6333d6054d903106b52cd9cead06e6498dfc1245eb80bc0a55dd1125d06'
snapshot = json.loads(snapraw)
counts = snapshot['observed_counts']
assert (counts['lenses'], counts['lenses_complete'], counts['queue_records'],
        counts['queue_semantically_reconciled'], counts['current_ledger_entries_and_relationships']) == (13, 0, 3713, 0, 1060)
report = (R / 'ASSESSMENT-CENTRAL-FOLLOWUPS-14C-95E.json').read_bytes()
assert report == (C / 'CENTRAL-14C-SYNTHESIS.json').read_bytes()
assert H(report) == snapshot['central_report_sha256']
assert snapshot['bundle_catalog_sha256'] == H(raw)
assert snapshot['b147_recovery_manifest_sha256'] == catalog['recovery_manifest_sha256']
assert H((R / 'MAIN-RECHECK-20261112-1720.json').read_bytes()) == snapshot['main_observation_sha256']
ledger = load(R / 'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert load(R / 'CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
for identifier in snapshot['added_ledger_relationship_ids']:
    found = [row for row in ledger if row.get('id') == identifier]
    assert len(found) == 1 and found[0]['record'] == 'ASSESSMENT-CENTRAL-FOLLOWUPS-14C-95E.json'
queue = load(R / 'RECONCILIATION-059.json')['entries']
for row in snapshot['queue_selections']:
    assert queue[row['index']]['contract_id'] == row['contract_id']
    assert row['current_reconciliation_at_snapshot'].startswith('partial')

diff_path = D / 'postb147-diff-treatment/RECORD.json'
assert H(diff_path.read_bytes()) == '5cf5b3cdf56bce4b27913740fe23fbc63df0d6e3c066e9bc4e268a9df1e280bd'
diff = load(diff_path)
assert diff['initial_diff_check_exit'] == 2 and len(diff['exact_paths']) == 101
attributes = (R / '.gitattributes').read_text().splitlines()
assert all(attributes.count(path + ' binary') == 1 for path in diff['exact_paths'])

value_audit = D / 'postb147-value-digest-audit'
assert H((value_audit / 'RECORD.json').read_bytes()) == '4fa573dd68eb00122106cd123e32415b36abceac81f9365b441f10fb97d26448'
vr = load(value_audit / 'RECORD.json')
assert H((value_audit / 'RESULT.json').read_bytes()) == vr['result_sha256']
assert H((value_audit / 'OUTPUT.log').read_bytes()) == vr['output_sha256']
assert not load(value_audit / 'RESULT.json')['match_locations']
assert not load(value_audit / 'RESULT.json')['exceptions']
redaction = D / 'postb147-redaction-review/REPORT.json'
assert H(redaction.read_bytes()) == '21195b9e05d7a459828e1ab11448e90c3bb8ae8694e3dc18da9b8e3b5ea46203'
assert load(redaction)['predecessor_publication'] == 'not published; separately held privately'

failure = D / 'postb147-evidence-check-failure'
assert H((failure / 'RECORD.json').read_bytes()) == '564c8a5f3dd0194d823e4b5111d9016b4b233f70340baf1de65cb6faa478459f'
fr = load(failure / 'RECORD.json')
assert fr['exit'] == 1 and fr['publication_checkout_path_substitutions'] > 0
assert H((failure / 'EVIDENCE-PATHSAFE.log').read_bytes()) == fr['public_transcript_sha256']
assert H((failure / 'RESULT.json').read_bytes()) == fr['result_sha256']
assert H((failure / 'ASSERT-PREFLIGHT.log').read_bytes()) == fr['preflight_sha256']
assert H((failure / 'STARTED.json').read_bytes()) == fr['started_sha256']
assert b"KeyError: 'observed_revision'" in (failure / 'EVIDENCE-PATHSAFE.log').read_bytes()
observation = load(R / 'MAIN-RECHECK-20261112-1730.json')
assert observation['observed_revision'] == catalog['source_revision']
assert observation['ref'] == 'refs/heads/main'
assert observation['earlier_observation_sha256'] == H((R / 'MAIN-RECHECK-20261112-1720.json').read_bytes())

env = os.environ.copy()
env.pop('PYTHONOPTIMIZE', None)
for name, script in [('q2325', 'VERIFY-EVIDENCE.py'), ('q2326', 'VERIFY-EVIDENCE.py'),
                     ('q2328', 'VERIFY-PUBLIC-SUCCESSOR.py'), ('q2329', 'VERIFY-EVIDENCE.py'),
                     ('q2330', 'VERIFY-PUBLIC-SUCCESSOR.py')]:
    subprocess.run([sys.executable, '-B', str(P / name / script)], cwd=R, env=env, check=True)
print('Bound five qualified packets, recorded replays, b147 recovery and time-bound relationships.')
print('Optional dependency-rich verifications and restricted originals are represented by same-machine receipts, not rerun here.')
print('No product behavior, independent custody, checkpoint3821 recovery or audit completion is asserted.')
