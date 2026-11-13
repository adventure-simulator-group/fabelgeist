#!/usr/bin/env python3
"""Verify public evidence identities and same-machine restoration receipts."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import stat
import subprocess
import sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
root = parser.parse_args().publication.resolve()
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda name: json.loads((p / name).read_bytes())
git = lambda *args: subprocess.check_output(['git', '-C', str(root), *args])
env = dict(os.environ)
env.pop('PYTHONOPTIMIZE', None)
env['PYTHONNOUSERSITE'] = '1'
env['PYTHONDONTWRITEBYTECODE'] = '1'
for name in ('transport', 'late-transport', 'review-transport',
             'navigation-review-transport', 'encrypted-originals'):
    subprocess.run([sys.executable, '-B', str(p / name / 'VERIFY.py')],
                   cwd=root, env=env, check=True)
for name in ('reconciliation', 'reconciliation-v5'):
    subprocess.run([sys.executable, '-B', str(p / name / 'VERIFY.py'),
                    '--publication', str(root)], cwd=root, env=env, check=True)
parent = 'b111f7444ab7e3b13b2099d5ce9d59f08faf9424'
assert git('rev-parse', parent + '^{tree}').decode().strip() == '84668f9ace2a06e346564f2185d5e57747403363'
assert git('rev-parse', '95e63628f9d70b8dd6e6fede64334696c6995009^{tree}').decode().strip() == '1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
queue = json.loads(git('show', parent + ':RECONCILIATION-059.json'))['entries']
pending = load('PENDING-SOURCE-REVIEWS.json')
assert pending['parent_checkpoint'] == parent
assert [r['queue_index'] for r in pending['rows']] == [3344, 3353, 3354, 3356, 3368, 3462]
for row in pending['rows']:
    q = queue[row['queue_index']]
    assert (q['contract_id'], q['historical_lens'], q['current_reconciliation']) == (
        row['historical_contract_id'], row['historical_lens'], row['central_state_at_parent'])
    assert row['central_state_at_parent'] == 'unassessed'
    git('cat-file', '-e', parent + ':' + row['original_public_transport'])
    prefix = 'delegated/postb111-followups/'
    assert row['peer_public_transport'].startswith(prefix)
    assert (p / row['peer_public_transport'][len(prefix):]).is_file()
assert (p / 'late-transport/q3368-projection.tar.gz').is_file()
correction = load('PENDING-CORRECTION.json')
assert correction['successor_sha256'] == sha((p / 'PENDING-SOURCE-REVIEWS.json').read_bytes())
for name, directory, count in [('PUBLIC-PRESCAN.json', 'transport', 11),
                               ('LATE-PRESCAN.json', 'late-transport', 6),
                               ('NAV-PRESCAN.json', 'navigation-review-transport', 4)]:
    receipt = load(name)
    assert receipt['transport_manifest_sha256'] == sha((p / directory / 'MANIFEST.json').read_bytes())
    assert receipt['scanned_paths'] == count
    assert receipt['forbidden_digest_total'] == receipt['exceptions'] == receipt['tar_pattern_issues'] == 0
    assert not receipt['other_candidate_counts'] and all(x['exit'] == 0 for x in receipt['checks'])
capture = load('REVIEW-PRESCAN-CAPTURE.json')
scan = load('REVIEW-PRESCAN.json')
assert len(capture['rows']) == scan['scanned_paths'] == 26
assert scan['capture_sha256'] == sha((p / 'REVIEW-PRESCAN-CAPTURE.json').read_bytes())
assert scan['forbidden_digest_total'] == scan['exceptions'] == scan['tar_pattern_issues'] == 0
assert not scan['other_candidate_counts'] and all(x['exit'] == 0 for x in scan['checks'])
assert len({r['path'] for r in capture['rows']}) == len(capture['rows'])
for row in capture['rows']:
    file = p / row['path']
    assert file.is_file() and not file.is_symlink() and file.resolve().is_relative_to(p)
    data = file.read_bytes()
    assert (len(data), sha(data)) == (row['bytes'], row['sha256'])
    assert bool(stat.S_IMODE(file.stat().st_mode) & 0o111) == bool(int(row['mode'], 8) & 0o111)
restore = load('PRIMARY-RESTORATION.json')
assert restore['ciphertext_manifest_sha256'] == sha((p / 'encrypted-originals/MANIFEST.json').read_bytes())
assert (restore['restore_exit'], restore['collections'], restore['matched_file_bytes_and_modes'],
        restore['matched_directory_modes']) == (0, 42, 4363, 459)
assert restore['source_root_modes_compared'] == restore['source_root_mode_adjustments_after_comparison'] == 42
assert restore['public_archives_extracted'] == 12 and restore['public_archive_files'] == 255
assert restore['selected_replay_exits'] == [0] * 14
assert all(x['exit'] == 0 and x['stderr_bytes'] == 0 for x in restore['selected_replay_checks'])
earlier = restore['separate_earlier_delegate_restoration']
assert (earlier['matched_file_bytes_and_modes'], earlier['matched_directory_modes']) == (495, 70)
assert earlier['counted_in_primary_42'] is False
recovery = load('REMOTE-RECOVERY.json')
assert recovery['commit'] == parent and recovery['captured_paths_matched'] == 141
assert len(recovery['checks']) == 8 and all(x['exit'] == 0 for x in recovery['checks'])
print('Verified public transport identities, historical pending snapshot, '
      'bounded-scan receipts and same-machine restoration records. Full '
      'decryption, independently held custody, source semantics, missing '
      'checkpoint3821 and product behavior remain separately qualified.')
