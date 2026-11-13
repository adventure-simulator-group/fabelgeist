#!/usr/bin/env python3
"""Check successor evidence identities; never execute product code."""
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

p = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
root = parser.parse_args().publication.resolve()
load = lambda path: json.loads(path.read_bytes())
sha256 = lambda data: hashlib.sha256(data).hexdigest()
env = dict(os.environ)
env.pop('PYTHONOPTIMIZE', None)
env['PYTHONNOUSERSITE'] = '1'
env['PYTHONDONTWRITEBYTECODE'] = '1'

for script, args in [
    (p / 'VERIFY.py', ['--publication', str(root)]),
    (p / 'application/VERIFY.py', ['--publication', str(root)]),
    (p / 'application/VERIFY-STATUS.py', ['--publication', str(root)]),
    (p / 'final-encrypted/VERIFY.py', []),
]:
    subprocess.run([sys.executable, '-B', str(script), *args],
                   cwd=root, env=env, check=True)

receipt = load(p / 'FINAL-RESTORATION.json')
assert receipt['schema'] == '726-post5f6a-final-restoration-v1'
assert receipt['restore_exit'] == 0 and receipt['collections'] == 15
assert receipt['ciphertext_manifest_sha256'] == sha256(
    (p / 'final-encrypted/MANIFEST.json').read_bytes())
assert receipt['matched_file_bytes_and_modes'] == 186
assert receipt['matched_directory_modes'] == 20
assert receipt['source_root_modes_compared'] == 15
assert receipt['source_root_mode_adjustments_after_comparison'] == 15
assert receipt['selected_replay_exits'] == [0] * 6

main = load(p / 'PREFLIGHT-MAIN.json')
assert main['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert main['source_tree'] == '1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
assert main['raw_serialized_response_sha256'] == (
    'e73ff5879ef21e897aa55799ca79afd7a614a75a5fd7c869d8479a1dd9a15178')

capture = load(p / 'ROOT-CANDIDATE-CAPTURE.json')
scan = load(p / 'ROOT-CANDIDATE-SCAN.json')
assert len(capture['rows']) == scan['scanned_paths'] == 7
assert {row['path'] for row in capture['rows']} == {
    'ASSESSMENT-CENTRAL-FOLLOWUPS-20-95E.json',
    'BASELINE-95E-20261110.json',
    'CONTINUATION-PROGRESS-95E.json',
    'CONTINUATION.md',
    'MAIN-RECHECK-20261113-0536.json',
    'RECONCILIATION-059.json',
    'verify_post5f6a_checkpoint.py',
}
assert scan['capture_sha256'] == sha256((p / 'ROOT-CANDIDATE-CAPTURE.json').read_bytes())
assert scan['forbidden_digest_total'] == scan['exceptions'] == 0
assert scan['tar_pattern_issues'] == 0 and not scan['other_candidate_counts']
assert all(item['exit'] == 0 for item in scan['checks'])
for row in capture['rows']:
    path = root / row['path']
    assert path.parent == root and path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert (len(data), sha256(data), blob) == (
        row['bytes'], row['sha256'], row['git_blob'])
    assert not (stat.S_IMODE(path.stat().st_mode) & 0o111)

print('Verified successor evidence identities, bounded-scan receipts and '
      'same-machine restoration receipt. The main observation is point-in-time; '
      'decryption, independent custody, source semantics, historical '
      'completeness and product behavior remain separately qualified.')
