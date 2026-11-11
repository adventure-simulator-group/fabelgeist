#!/usr/bin/env python3
"""Verify third central delegated reconciliation, with no product execution."""
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
name = 'ASSESSMENT-DELEGATED-RECONCILIATION-3-95E.json'
a = load(name)
h = load(a['source_roles'])
roles = h['historical_source_roles']
assert [x['original_index'] for x in roles] == [2309, 2310, 2311, 2323, 2324, 2325, 2327, 2328, 2329, 2340, 2341, 2342]
work = load('COVERAGE-L01-ROLES.json')['roles']
cp = load(a['evidence_checkpoint'])
archives = {}
for bundle in cp['bundles']:
    with tarfile.open(root / bundle['archive'], 'r:gz') as archive:
        archives[bundle['bundle_id']] = {x.name: archive.extractfile(x).read() for x in archive.getmembers()}
for role in roles:
    r = work[role['original_index']]
    assert r['status'] == 'full-historical-body-trace-open' and r['historical_assessment'] == a['source_roles']
    assert r['source_id'] == role['source_id'] and r['historical_sha256'] == role['sha256']
    assert sha((root / role['local_registration']['path']).read_bytes()) == role['local_registration']['sha256']
    assert role['new_personal_read_ranges'] == [[1, role['lines']]]
    current = role['current_comparison']
    assert current['source_revision'] == a['source_revision'] and current['current_personal_ranges'] == [[1, current['lines']]]
    i = role['original_index']
    if i in (2309, 2310, 2311):
        data = archives['math-units']
        report = json.loads(data['report.json'])
        evidence = next(x for x in report['role_records'] if x['index'] == i)
        assert role['new_personal_read_revision'] == a['source_revision']
        assert current['byte_identical'] and evidence['fresh_personal_current_reads'] == role['new_personal_read_ranges']
        assert 'not separately displayed' in role['new_personal_read_qualification']
    elif i in (2327, 2328, 2329):
        data = archives['physics-contact']
        report = json.loads(data['REPORT.json'])
        evidence = next(x for x in report['original_roles'] if x['original_index'] == i)
        assert role['new_personal_read_revision'] == a['source_revision']
        assert current['byte_identical'] and evidence['historical_personal_displayed_ranges'] == []
        assert evidence['current_displayed_ranges'] == role['new_personal_read_ranges']
    else:
        data = archives['mhr-layout' if i in (2323, 2324, 2325) else 'shell-contact']
        receipt = json.loads(data['read-receipt.json'])
        reads = receipt.get('personal_reads', receipt.get('actual_source_displays'))
        old = next(x for x in reads if x['path'] == current['path'] and x['revision'] == role['snapshot_head'])
        now = next(x for x in reads if x['path'] == current['path'] and x['revision'] == a['source_revision'])
        assert role['new_personal_read_revision'] == role['snapshot_head']
        assert old['ranges'] == role['new_personal_read_ranges'] and old['sha256'] == role['sha256']
        assert now['ranges'] == current['current_personal_ranges'] and now['sha256'] == current['sha256']
assert 'separately personally read tail386–413' in next(x for x in roles if x['original_index'] == 2329)['historical_attribution_qualification']
assert 'null read status' in next(x for x in roles if x['original_index'] == 2311)['historical_attribution_qualification']
assert len(a['delegated_observations']) == 23 and {x['agent'] for x in a['delegated_observations']} == {'/root/hair_ops', '/root/mhr_character', '/root/rig_identity'}
old = load('baseline-059/queue.json')['entries']
queue = load('RECONCILIATION-059.json')['entries']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
ids = {x['id']: x for x in ledger}
assert len(ids) == len(ledger)
for i in (53,157,168,169,174,431,432,433,434,435,437,438,439,440,441,442,443,444,445,446,477,479,480,575,1267,1853,1856,1858,2410,2414):
    x = queue[i]
    assert x['contract_id'] == old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ')
    assert name in x['current_reconciliation'] or name in x.get('further_current_evidence', [])
assert 'first rejects the empty mesh' in queue[437]['bounded_current_disposition']
assert 'stored zero capacity' in queue[480]['bounded_current_disposition']
assert 'resizes provided masses' in queue[444]['bounded_current_disposition']
for i in (174,435,438,439,440,441,442,443,444,445,446,477,479,480,1267,1853):
    x = ids[old[i]['contract_id']]
    assert x['record'] == name and x['repair'] == 'not started' and x['validation'] == 'not performed'
for ident in ('REVIEW-MHR-PARAMETER-COLUMN-MEMBERSHIP-95E','REVIEW-PHYSICS-SHORT-NONZERO-CAPSULE-95E','REVIEW-PHYSICS-SWEPT-MESH-FINAL-SURFACE-95E','REVIEW-PHYSICS-MESH-RESOURCE-COHERENCE-95E','REVIEW-SHELL-FIXED-SURFACE-SELF-FLAG-95E','REVIEW-SHELL-POST-ADMISSION-TOPOLOGY-95E','REVIEW-SHELL-CCD-NONZERO-SPEED-95E','REVIEW-SHELL-CCD-NEAR-PARALLEL-95E'):
    x = ids[ident]
    assert x['record'] == name and x['repair'] == 'not started' and x['validation'] == 'not performed'
assert any('/segovb_continuation/lens13/delegated_connected_contracts_pending_alias/193' in x for x in a['central_decisions'])
assert any('not separately displayed' in x['new_personal_read_qualification'] for x in roles)
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['delegated_batch3_central_reconciliation'] == name and p['new_current_ledger_candidates'] == len(ledger)
assert p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert a['repair'] == 'not started' and a['validation'] == 'not performed'
print('Verified third delegated source attribution, historical qualifications and connected ledger; no product execution.')
