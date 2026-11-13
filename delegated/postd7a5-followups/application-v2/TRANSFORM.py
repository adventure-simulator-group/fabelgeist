#!/usr/bin/env python3
"""Deterministically apply six bounded audit assessments; no product work."""
import copy
import json

NAMES = ['BASELINE-95E-20261110.json', 'CONTINUATION-PROGRESS-95E.json',
         'RECONCILIATION-059.json', 'CONTINUATION.md']
REPORT = 'ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json'
RECHECK = 'MAIN-RECHECK-20261113-0033.json'
PENDING = 'delegated/postd7a5-followups/PENDING-DELEGATES.json'


def render(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def transform(before, report, pending, observation):
    assert set(before) == set(NAMES)
    baseline = json.loads(before[NAMES[0]])
    progress = json.loads(before[NAMES[1]])
    queue = json.loads(before[NAMES[2]])
    document = before[NAMES[3]].decode()
    rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
    parent = 'd7a5e77ebdbd44a05cde30881493d6fc22ef025d'
    assert baseline['source_revision'] == report['source_revision'] == rev
    assert pending['source_revision'] == observation['observed_revision'] == rev
    assert report['parent_published_checkpoint'] == pending['parent_checkpoint'] == parent
    assert observation['ref'] == 'refs/heads/main'
    assert observation['observed_tree'] == '1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
    assessments = [x['record'] for x in report['partial_queue_assessments']]
    context = [x['record'] for x in report['context_only_observations']]
    relations = [x['record'] for x in report['ledger_relationships']]
    assert [a['queue_index'] for a in assessments] == [2945, 2957, 3040, 3011, 3059, 3166]
    assert [a['queue_index'] for a in context] == [2968, 3016]
    assert len(relations) == 6 and len(pending['packets']) == 20
    entries = baseline['newly_assessed_ledger']['entries']
    ids = [x['id'] for x in relations]
    assert len(entries) == 1093 and len({x['id'] for x in entries}) == 1093
    assert len(set(ids)) == 6 and not set(ids) & {x['id'] for x in entries}
    for relation in relations:
        row = copy.deepcopy(relation)
        assert row['repair'] == 'not started' and row['validation'] == 'not performed'
        row['record'] = REPORT
        entries.append(row)
    assert len(entries) == 1099
    baseline['latest_main_recheck'] = RECHECK
    assert len(queue['entries']) == 3713
    for row in assessments:
        q = queue['entries'][row['queue_index']]
        assert q['contract_id'] == row['historical_contract_id']
        assert q['historical_lens'] == row['historical_lens']
        assert q['current_reconciliation'] == 'unassessed'
        q['current_reconciliation'] = ('partial current 95e source assessment and '
            'historical relationship; see ' + REPORT + '; no semantic queue or lens '
            'closure and no checkpoint3821 attribution')
        q['further_current_evidence'] = ['delegated/postd7a5-followups/central/CENTRAL-EVIDENCE.tar.gz']
    for row in context:
        q = queue['entries'][row['queue_index']]
        assert q['contract_id'] == row['historical_contract_id']
        assert q['historical_lens'] == 1 and q['current_reconciliation'] == 'unassessed'
    assert sum(x['current_reconciliation'] == 'unassessed' for x in queue['entries']) == 2359
    assert sum(x['current_reconciliation'].startswith('partial') for x in queue['entries']) == 1354
    assert progress['new_current_ledger_candidates'] == 1093
    assert progress['lens_count'] == 13 and progress['lenses_complete'] == 0
    assert progress['queue_records_indexed'] == 3713
    assert progress['queue_records_semantically_reconciled'] == 0
    progress['new_current_ledger_candidates'] = 1099
    progress['central_postd7a5_bounded_reconciliations'] = (REPORT + '; six partial '
        'assessments, two context-only observations and six planning relationships; '
        'no queue or lens closure or accepted unique count increment.')
    progress['checkpoint_d7a5_same_machine_recovery'] = ('delegated/postd7a5-followups/'
        'checkpoint-d7a5-recovery/README.md; 417 captured files and identical '
        'evidence transcript in a separate same-machine checkout.')
    progress['postd7a5_pending_delegated_packets'] = (PENDING + '; 20 distinct '
        'queue obligations include nine inherited packets, nine later packets '
        'and two context-only producer-propagation obligations.')
    progress['postdac7_pending_resolution'] = ('One of ten prior pending packets '
        'now has partial central assessment; nine remain pending. Earlier index '
        'remains a historical snapshot and no queue record is closed.')
    progress['postd7a5_private_recovery'] = ('delegated/postd7a5-followups/'
        'encrypted-originals/README.md and RESTORATION-RECEIPT.json; same-machine '
        'primary key, earlier q2812 anchors and named nested keys are dependencies.')
    progress['main_recheck_20261113_0033'] = RECHECK
    progress['current_main_qualification'] = ('GitHub connector observed main at '
        '95e at 2026-11-13 00:33:20–00:33:21 UTC; this is a point-in-time '
        'observation and main may advance. See ' + RECHECK + '.')
    marker = 'product validation is authorized here.\n\n'
    assert document.count(marker) == 1
    insert = ('`' + REPORT + '` connects six bounded partial\n'
        'assessments and six planning relationships. Two other observations are\n'
        'context only: q2968 and q3016 remain unassessed for historical Lens 1.\n'
        'No semantic queue record or lens is closed, and no unique-defect count\n'
        'is added. `delegated/postd7a5-followups/PENDING-DELEGATES.json` indexes\n'
        '20 separately qualified pending obligations and context records.\n\n'
        '`delegated/postd7a5-followups/checkpoint-d7a5-recovery/README.md`\n'
        'records the same-machine recovery of d7a5. Its captured files and\n'
        'evidence transcript matched; independent custody and product behavior\n'
        'remain unproved. The later encrypted originals and recovery receipt\n'
        'preserve separately held key and earlier-anchor dependencies.\n\n'
        '`delegated/postd7a5-followups/INDEX.md` locates the independent\n'
        'reviews, the successor wording correction and later pending packets.\n\n'
        '`' + RECHECK + '` records a point-in-time\n'
        'connector observation of main at 95e. Main may have moved since.\n\n'
        'The following earlier checkpoint paragraphs retain their historical\n'
        'snapshot counts; q2945 now has the partial assessment above, and the\n'
        'q3016 context-only packet is preserved in this successor.\n\n')
    document = document.replace(marker, marker + insert, 1)
    assert baseline['historical_queue_records_semantically_reconciled'] == 0
    assert progress['repair'] == 'not started' and progress['product_validation'] == 'not performed'
    return {NAMES[0]: render(baseline), NAMES[1]: render(progress),
            NAMES[2]: render(queue), NAMES[3]: document.encode()}
