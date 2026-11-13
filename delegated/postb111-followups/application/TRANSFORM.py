#!/usr/bin/env python3
"""Apply only the reviewed source-evidence reconciliation, never product code."""
import copy
import hashlib
import json
import sys
import textwrap

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')

NAMES = ['BASELINE-95E-20261110.json', 'CONTINUATION-PROGRESS-95E.json',
         'RECONCILIATION-059.json', 'CONTINUATION.md']
REPORT = 'ASSESSMENT-CENTRAL-FOLLOWUPS-21-95E.json'
MAIN = 'MAIN-RECHECK-20261113-0655.json'
PARENT = 'b111f7444ab7e3b13b2099d5ce9d59f08faf9424'
REV = '95e63628f9d70b8dd6e6fede64334696c6995009'
TREE = '1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
DRAFT_SHA256 = '1212f4f29a5dbf2ed6d4faff7788d9df13ed744a69b7c7c1262548d3899d1fef'
IDS = [0, 19, 2497, 2501, 2503, 2504, 2510, 2521, 2534]
LENSES = [8, 11, 3, 7, 9, 10, 4, 3, 4]
CONTRACTS = [
    'ADLER-CONT08-CANDIDATE-001', 'CONT-WEDDING-CAPACITY-POSTWRITE-ERROR',
    'EXTENSION-L3-ROOT-L2-EXT-0055', 'EXTENSION-L7-ROOT-L2-EXT-0055',
    'EXTENSION-L9-ROOT-L2-EXT-0055', 'EXTENSION-L10-ROOT-L2-EXT-0055',
    'EXTENSION-L4-ROOT-L2-EXT-0056', 'EXTENSION-L3-ROOT-L2-EXT-0057',
    'EXTENSION-L4-ROOT-L2-EXT-0058',
]
RELIDS = [
    'relationship-retained-fit-item-identity-95e',
    'relationship-wedding-capacity-prefix-95e',
    'relationship-organization-training-weight-narrowing-95e',
    'relationship-organization-profession-list-authority-95e',
    'relationship-organization-skill-requirement-shape-95e',
    'relationship-social-diagnosis-axis-code-95e',
    'relationship-social-belief-confidence-insert-95e',
    'relationship-dialogue-fresh-session-npc-spelling-95e',
    'relationship-notification-query-error-defaults-95e',
]

def render(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()

def transform(before, reviewed_draft, report, observation):
    assert set(before) == set(NAMES)
    assert hashlib.sha256(reviewed_draft).hexdigest() == DRAFT_SHA256
    draft = json.loads(reviewed_draft)
    assert draft['schema'] == '726-postb111-central-followups-draft-v5'
    assert draft['status'] == 'draft for independent review; not adopted'
    assert report['schema'] == '726-postb111-central-assessment-v1'
    assert report['status'] == ('adopted as nine bounded partial assessments and nine '
                                'planning relationships; no semantic closure')
    assert set(report) - set(draft) == {
        'adopted_from_draft_sha256', 'independent_reviews',
        'adoption_qualification', 'historical_provenance',
    }
    assert all(report[key] == draft[key] for key in draft if key not in ('schema', 'status'))
    assert report['adopted_from_draft_sha256'] == DRAFT_SHA256
    reviews = report['independent_reviews']
    assert set(reviews) == {'reconciliation', 'organization', 'navigation'}
    assert reviews['reconciliation']['public_manifest_sha256'] == (
        '914f79f90fbc65dd5dc8beff9a3e7bc5a39d7d16bc41455d2ce4f5adf8f70680')
    assert reviews['reconciliation']['receipt_manifest_sha256'] == (
        '80f0d460925f5ddf14e41ce70bfaeac4c2b74a86f48ddba3a30ca8878cd854d0')
    assert reviews['reconciliation']['public_transport'] == (
        'delegated/postb111-followups/review-transport/reconciliation-review.tar.gz')
    assert reviews['organization']['public_manifest_sha256'] == (
        '4fb4e0830ea7360b6a318d117079a4b60be056c0c288e1fa83d83f4a208e9e67')
    assert reviews['organization']['receipt_manifest_sha256'] == (
        '53f8f319da62a6b279d9a21eb4489a6097f59990ec18089aef7da9f2101ad0ff')
    assert reviews['organization']['public_transport'] == (
        'delegated/postb111-followups/review-transport/organization-review.tar.gz')
    assert reviews['navigation']['public_manifest_sha256'] == (
        'e5b34729bc62896fd6c076f9ed0f83936ecbadc9cc914f5d7f29fbece6ec6baa')
    assert reviews['navigation']['receipt_manifest_sha256'] == (
        '1d72119bb515eb6f1b342ac200575f9dd502717b467c892ab8399471b576e9df')
    assert reviews['navigation']['public_transport'] == (
        'delegated/postb111-followups/navigation-review-transport/navigation-review.tar.gz')
    assert all(isinstance(x['qualification'], str) and x['qualification'] for x in reviews.values())
    assert isinstance(report['adoption_qualification'], str) and report['adoption_qualification']
    assert draft['parent_checkpoint'] == PARENT
    assert draft['source_revision'] == observation['observed_revision'] == REV
    assert draft['source_tree'] == observation['observed_tree'] == TREE
    assert observation['ref'] == 'refs/heads/main'
    assert observation['method'] == 'GitHub connector fetch branches/main API'
    assert observation['before_utc'] == '2026-11-13 06:55:18 UTC'
    assert observation['after_utc'] == '2026-11-13 06:55:19 UTC'
    assert observation['serialized_joint_response_sha256'] == (
        '72657b188628c2709864241f154e980109ab5f9ccd342f3836de290a8b29c751')
    assessments = report['assessments']
    relationships = report['proposed_ledger_relationships']
    assert [x['index'] for x in assessments] == IDS
    assert [x['lens'] for x in assessments] == LENSES
    assert [x['contract'] for x in assessments] == CONTRACTS
    assert [x['id'] for x in relationships] == RELIDS
    assert report['counts_if_adopted'] == {
        'queue_records': 3713, 'partial_before': 1370, 'unassessed_before': 2343,
        'partial_after': 1379, 'unassessed_after': 2334, 'ledger_before': 1106,
        'proposed_relationships': 9, 'ledger_after': 1115, 'queue_closures': 0,
        'lens_completions': 0, 'accepted_unique_defect_increment': 0,
    }
    history = report['historical_provenance']
    assert history['original_queue_sha256'] == '2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08'
    assert history['original_ledger_sha256'] == '3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'
    assert history['reconstructed_queue_sha256'] == '3695264e6d4531a1137f0c3232e0d5163d941c49849cb4bddef31cd8403fd173'
    assert history['reconstructed_ledger_sha256'] == 'f9501199085284117ef88969fae443654af353429c657fd6bd1efb916582b180'
    baseline, progress, queue = (json.loads(before[name]) for name in NAMES[:3])
    doc = before[NAMES[3]].decode()
    old_baseline = copy.deepcopy(baseline)
    old_progress = copy.deepcopy(progress)
    old_queue = copy.deepcopy(queue)
    assert baseline['source_revision'] == progress['source_revision'] == REV
    entries = queue['entries']
    ledger = baseline['newly_assessed_ledger']['entries']
    assert len(entries) == 3713 and len(ledger) == 1106
    assert sum(x['current_reconciliation'].startswith('partial') for x in entries) == 1370
    assert sum(x['current_reconciliation'] == 'unassessed' for x in entries) == 2343
    old_ids = {x['id'] for x in ledger}
    assert len(old_ids) == 1106 and len(set(RELIDS)) == 9 and not old_ids.intersection(RELIDS)
    lenses_by_index = {a['index']: a['lens'] for a in assessments}
    for relation in relationships:
        assert set(relation) == {'id', 'queue', 'ledger', 'direction',
                                 'qualification', 'classification'}
        assert all(0 <= i < len(entries) for i in relation['queue'])
        assert all(0 <= i < 1106 for i in relation['ledger'])
        row = {
            'id': relation['id'],
            'assessed_lenses': sorted({lenses_by_index[i] for i in relation['queue']
                                       if i in lenses_by_index}),
            'classification': relation['classification'],
            'historical_related_records': ['baseline-059/queue.json#/entries/' + str(i)
                                           for i in relation['queue']],
            'current_related_records': ['BASELINE-95E-20261110.json#/newly_assessed_ledger/entries/' + str(i)
                                        for i in relation['ledger']],
            'relationship_qualification': relation['qualification'],
            'connected_repair_direction': relation['direction'],
            'discovery': 'partial; supported reach, policy, source complements and exact aliases remain open',
            'repair': 'not started', 'validation': 'not performed',
            'record': REPORT, 'historical_original_and_reconstruction': copy.deepcopy(history),
        }
        ledger.append(row)
    for assessment in assessments:
        i = assessment['index']
        row = entries[i]
        assert (row['contract_id'], row['historical_lens'], row['current_reconciliation']) == (
            assessment['contract'], assessment['lens'], 'unassessed')
        assert 'further_current_evidence' not in row
        row['current_reconciliation'] = ('partial current 95e source assessment and historical '
            'relationship; see ' + REPORT + '; no semantic queue or lens closure and no checkpoint3821 attribution')
        row['further_current_evidence'] = [REPORT, *assessment['evidence']]
    assert sum(x['current_reconciliation'].startswith('partial') for x in entries) == 1379
    assert sum(x['current_reconciliation'] == 'unassessed' for x in entries) == 2334
    assert len(ledger) == 1115
    assert progress['new_current_ledger_candidates'] == 1106
    assert progress['lenses_complete'] == progress['queue_records_semantically_reconciled'] == 0
    assert baseline['historical_queue_records_semantically_reconciled'] == 0
    assert progress['repair'] == 'not started' and progress['product_validation'] == 'not performed'
    progress['new_current_ledger_candidates'] = 1115
    progress['central_postb111_bounded_reconciliations'] = (
        REPORT + '; nine partial assessments, nine planning relationships, '
        'no semantic closure or accepted unique-defect increment.')
    progress['postb111_queue_snapshot'] = (
        '3,713 indexed: 1,379 partial and 2,334 unassessed; all remain semantically unreconciled.')
    progress['postb111_prior_pending_qualification'] = (
        'Earlier pending indexes are time-bound: q0, q19, q2497, q2501, q2503, '
        'q2504, q2510, q2521 and q2534 now have partial assessments; historical index bytes remain unchanged.')
    progress['checkpoint_b111_same_machine_recovery'] = (
        'delegated/postb111-followups/REMOTE-RECOVERY.json; 141 captured path identities '
        'matched after predecessor checks and remote fetch into the same-machine recovery checkout.')
    progress['main_recheck_20261113_0655'] = MAIN
    progress['current_main_qualification'] = (
        'GitHub connector observed main at 95e between ' + observation['before_utc'] + ' and '
        + observation['after_utc'] + '; this is point-in-time and main may advance. See ' + MAIN + '.')
    baseline['latest_main_recheck'] = MAIN
    marker = 'product validation is authorized here.\n\n'
    assert doc.count(marker) == 1
    paragraphs = [
        '`' + REPORT + '` connects nine bounded partial assessments and nine planning relationships. '
        'The working queue has 1,379 partial and 2,334 unassessed records, all semantically '
        'unreconciled; the ledger has 1,115 entries and relationships, not unique defects. '
        'No queue record or lens is closed and no unique-defect increment is accepted.',
        'The selected original and reconstructed 059 queue objects match as parsed while their '
        'complete streams differ. Historical records, normalized 774, missing pre-export originals, '
        'SOURCE004, the 73 nontext/no-positive-EOF obligations and missing checkpoint 3821 retain '
        'their separate qualifications. Older pending indexes retain their historical snapshot counts.',
        '`delegated/postb111-followups/REMOTE-RECOVERY.json` records same-machine recovery of '
        'the preceding checkpoint, and `' + MAIN + '` records a later point-in-time main observation. '
        'Main may advance. The predecessor checks require an exact b111 checkout; '
        '`delegated/postb111-followups/application/README.md` documents evidence-only replay.',
    ]
    insert = '\n\n'.join(textwrap.fill(par, width=80, break_long_words=False,
                                       break_on_hyphens=False) for par in paragraphs) + '\n\n'
    doc = doc.replace(marker, marker + insert, 1)
    assert queue.keys() == old_queue.keys()
    assert all(queue[k] == old_queue[k] for k in queue if k != 'entries')
    for i, (old, new) in enumerate(zip(old_queue['entries'], entries)):
        if i not in IDS:
            assert old == new
        else:
            assert {k: v for k, v in new.items() if k not in ('current_reconciliation', 'further_current_evidence')} == {
                k: v for k, v in old.items() if k != 'current_reconciliation'}
    assert ledger[:1106] == old_baseline['newly_assessed_ledger']['entries']
    for key in baseline:
        if key not in ('newly_assessed_ledger', 'latest_main_recheck'):
            assert baseline[key] == old_baseline[key]
    assert baseline['newly_assessed_ledger']['qualification'] == old_baseline['newly_assessed_ledger']['qualification']
    assert all(progress[k] == old_progress[k] for k in old_progress
               if k not in ('new_current_ledger_candidates', 'current_main_qualification'))
    return {NAMES[0]: render(baseline), NAMES[1]: render(progress),
            NAMES[2]: render(queue), NAMES[3]: doc.encode()}
