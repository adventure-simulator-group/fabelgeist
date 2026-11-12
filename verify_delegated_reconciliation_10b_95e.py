#!/usr/bin/env python3
"""Verify ten qualified historical/current roles and connected ledger evidence."""
import hashlib
import json
import tarfile
from pathlib import Path

R = Path(__file__).resolve().parent
load = lambda name: json.loads((R / name).read_text())
sha = lambda body: hashlib.sha256(body).hexdigest()
def normalize(ranges):
    out = []
    for lo, hi in sorted(ranges):
        assert 1 <= lo <= hi
        if out and lo <= out[-1][1] + 1:
            out[-1][1] = max(out[-1][1], hi)
        else:
            out.append([lo, hi])
    return out

checkpoint = load('DELEGATED-EVIDENCE-CHECKPOINT-10.json')
bundles = {b['bundle_id']: b for b in checkpoint['bundles']}
selected = {'rig-quality-clothing': [2468, 2469],
            'rig-quality-export-hair': [2470, 2471, 2472],
            'rig-quality-model': [2473, 2474],
            'mixed-owner': [2475, 2479, 2480]}
doc = load('ASSESSMENT-DELEGATED-RIG-2468-2480.json')
rows = {r['original_index']: r for r in doc['historical_source_roles']}
assert sorted(rows) == sorted(i for v in selected.values() for i in v)
work = load('COVERAGE-L01-ROLES.json')['roles']
central = load('ASSESSMENT-DELEGATED-RECONCILIATION-10B-95E.json')
assert len(central['delegated_observations']) == 16
for name, indexes in selected.items():
    b = bundles[name]
    raw = (R / b['archive']).read_bytes()
    assert sha(raw) == b['archive_sha256'] and len(raw) == b['archive_bytes']
    with tarfile.open(R / b['archive'], 'r:gz') as archive:
        receipt_name = 'personal-read-receipt.json' if name == 'rig-quality-model' else 'read-receipt.json'
        receipt = json.load(archive.extractfile(receipt_name))
        report = json.load(archive.extractfile(b['report_archive_member']))
        assert report == load(b['report_path'])
        key = ('findings_and_qualified_questions' if name == 'rig-quality-model'
               else 'findings' if name == 'rig-quality-export-hair'
               else 'findings_and_reconciliations')
        observed = [o for o in central['delegated_observations'] if o['bundle_id'] == name]
        assert [o['original_finding'] for o in observed] == report[key]
        assert all(o['author'] == b['agent'] and o['report'] == b['report_path'] for o in observed)
        if name == 'rig-quality-clothing':
            correction = json.load(archive.extractfile('registration-correction.json'))
            assert 'mistakenly says P17' in correction['correction']
        if name == 'rig-quality-model':
            snapshot = json.load(archive.extractfile('snapshot-verification.json'))
            assert snapshot['exact_patch_matches_recorded_sha256']
            assert snapshot['reconstructed_patch_sha256'] == '6dcc1f6784b41bc1e76fd55bdd2f0c0b5083a5eb8ec18bcebd9519bdf4e869cf'
        for i in indexes:
            row = rows[i]
            cc = row['current_comparison']
            assert row['new_personal_read_attribution'] == b['agent']
            assert row['evidence_archive'] == b['archive'] and row['receipt_member'] == receipt_name
            assert work[i]['historical_sha256'] == row['sha256']
            old = archive.extractfile(row['historical_member']).read()
            assert sha(old) == row['sha256'] and len(old.splitlines()) == row['lines']
            if cc['status'] == 'present':
                current = archive.extractfile(row['current_member']).read()
                assert sha(current) == cc['sha256'] and len(current.splitlines()) == cc['lines']
            else:
                assert i == 2474 and cc['status'] == 'absent' and row['current_member'] is None
            if name in ('rig-quality-clothing', 'mixed-owner'):
                old_ranges = next(x for x in receipt['reads'] if x['role'] == i and x['revision'] == row['snapshot_head'])['actual_display_ranges']
                now_ranges = next(x for x in receipt['reads'] if x['role'] == i and x['revision'] == cc['source_revision'])['actual_display_ranges']
            elif name == 'rig-quality-export-hair':
                rr = next(x for x in receipt['roles'] if x['original_index'] == i)
                old_ranges = rr['rig_quality_actual_display_ranges']
                now_ranges = rr['current_actual_display_ranges']
                assert rr['pr_working_actual_display_ranges'] == []
            else:
                old_ranges = next(x for x in receipt['assigned_personal_reads'] if x.get('role') == i and x['revision'] == row['snapshot_head'])['ranges']
                now_ranges = next(x for x in receipt['assigned_personal_reads'] if x.get('role') == i and x['revision'] == cc['source_revision'])['ranges']
            assert old_ranges == row['actual_personal_display_chunks']
            assert now_ranges == cc['actual_personal_display_chunks']
            assert normalize(old_ranges) == row['new_personal_read_ranges']
            assert normalize(now_ranges) == cc['personal_ranges']
            if i in (2470, 2475):
                assert work[i]['status'] == 'source-metadata-partial'
                assert row['historical_unread_ranges'] and cc['unread_ranges']
                assert work[i]['assessment'] == 'ASSESSMENT-DELEGATED-RIG-2468-2480.json'
            else:
                assert work[i]['status'] == 'full-historical-body-trace-open'
                assert row['new_personal_read_ranges'] == [[1, row['lines']]]
                assert work[i]['historical_assessment'] == 'ASSESSMENT-DELEGATED-RIG-2468-2480.json'
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
ids = {x['id']: x for x in ledger}
assert len(ids) == len(ledger) and len(central['new_qualified_ledger_rows']) == 4
for ident in central['new_qualified_ledger_rows']:
    assert ids[ident]['record'] == 'ASSESSMENT-DELEGATED-RECONCILIATION-10B-95E.json'
    assert ids[ident]['repair'] == 'not started' and ids[ident]['validation'] == 'not performed'
assert 'before their joint extents are admitted' in ids['SEGOVB-04-EXPORTED-SOCKET-INPUT-CANDIDATE-001']['classification']
assert 'nonfinite value' in ids['SEGOVB-04-GLTF-COLOR-LINEARIZATION-CANDIDATE-001']['classification']
assert 'current native serializer' in ids['SEGOVB-04-GLTF-COLOR-LINEARIZATION-CANDIDATE-001']['relationship_qualification']
for ident in ('REVIEW-CLOTHING-REFIT-INVALID-FACE-001', 'REVIEW-GLB-BODY-NORMAL-NONFINITE-001',
              'ADLER-L11-MHR-MODEL-SHAPE-LIFETIME', 'SEGOVB-L12-CONNECTED-MHR-FK-CYCLE-ADMISSION',
              'SEGOVB-L12-CONNECTED-MHR-FK-PARENT-MEMBERSHIP-ADMISSION',
              'ADLER-L11-MHR-LOAD-ERROR-OWNERSHIP', 'REVIEW-MHR-TINY-TRIANGLE-NORMAL-95E'):
    assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10B-95E.json' in ids[ident]['further_current_evidence']
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['lenses_complete'] == progress['queue_records_semantically_reconciled'] == 0
assert progress['new_current_ledger_candidates'] == len(ledger)
assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10B-95E.json' in progress['delegated_batch10_partial_reconciliation']
print('Verified eight full and two partial roles, 16 attributed observations and four qualified ledger rows; other packets remain pending.')
