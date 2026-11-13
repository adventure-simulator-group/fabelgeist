#!/usr/bin/env python3
"""Verify captured evidence identities and its finite-scan receipt."""
from pathlib import Path, PurePosixPath
import hashlib
import json
import sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
root = p.parents[2]
sha = lambda data: hashlib.sha256(data).hexdigest()
c = json.loads((p / 'CAPTURE.json').read_bytes())
s = json.loads((p / 'SCAN.json').read_bytes())
assert c['schema'] == '726-post76fe-staged-capture-v1'
assert c['parent_checkpoint'] == '76fe53635e2bf1e3e08bbfaa364314f5e7888bf8'
assert c['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert c['root_candidates_matched'] == 7 and c['strict_diff_check_exit'] == 0
assert s['schema'] == '726-post76fe-staged-scan-v1'
assert s['capture_sha256'] == sha((p / 'CAPTURE.json').read_bytes())
assert s['scanned_paths'] == len(c['rows'])
assert s['forbidden_digest_total'] == s['exceptions'] == s['tar_pattern_issues'] == 0
assert not s['other_candidate_counts']
assert [item['exit'] for item in s['checks']] == [0, 0]
assert len({row['path'] for row in c['rows']}) == len(c['rows'])
for row in c['rows']:
    rel = PurePosixPath(row['path'])
    assert not rel.is_absolute() and '..' not in rel.parts
    file = root.joinpath(*rel.parts)
    assert file.is_file() and not file.is_symlink() and file.resolve().is_relative_to(root)
    data = file.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert (len(data), sha(data), blob) == (row['bytes'], row['sha256'], row['git_blob'])
    assert row['mode'] in ('100644', '100755')
    assert bool(file.stat().st_mode & 0o111) == (row['mode'] == '100755')
print('Verified captured identities and the finite-scan receipt. Closing '
      'metadata, opaque ciphertext and Git history retain their recorded limits.')
