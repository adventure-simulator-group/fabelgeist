#!/usr/bin/env python3
"""Check recorded staged evidence; optional Git replay requires the checkpoint objects."""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P = Path(__file__).resolve().parent
H = lambda b: hashlib.sha256(b).hexdigest()
record = json.loads((P / 'RECORD.json').read_bytes())
capture = json.loads((P / 'STAGED-CAPTURE.json').read_bytes())
rows = capture['rows']
assert record['staged_files'] == capture['count'] == len(rows) == 1652
aggregate = hashlib.sha256()
for row in rows:
    path = PurePosixPath(row['path'])
    assert not path.is_absolute() and '..' not in path.parts and row['mode'] in ('100644', '100755')
    aggregate.update(row['path'].encode() + b'\0' + row['mode'].encode() + b'\0' + row['sha256'].encode() + b'\n')
assert aggregate.hexdigest() == capture['aggregate'] == record['aggregate_names_modes_and_sha256']
digest = (P.parent / 'checkpoint-31e1-staged-digest/OUTPUT.json').read_bytes()
pattern = (P.parent / 'checkpoint-31e1-staged-pattern/OUTPUT.log').read_bytes()
assert H(digest) == record['staged_digest_scan_sha256'] and H(pattern) == record['staged_pattern_scan_sha256']
d = json.loads(digest)
assert not d['forbidden_digest_total'] and not d['other_candidate_counts'] and not d['exceptions'] and d['stats']['top_files'] == len(rows)
assert b'issues 0' in pattern and record['driver_exit'] == record['diff_check_exit'] == 0
ap = argparse.ArgumentParser()
ap.add_argument('--repo', type=Path, help='repository containing published checkpoint 31e')
args = ap.parse_args()
if not args.repo:
    print('Verified recorded capture aggregate and bounded scan receipts; NOT REPEATED: Git object comparison requires --repo.')
    raise SystemExit(0)
env = os.environ.copy()
env.update({'GIT_NO_REPLACE_OBJECTS':'1', 'GIT_NO_LAZY_FETCH':'1'})
commit = '31e130bfa8f8914f759c7d5cddc3db92962704c7'
git = ['git', '-C', str(args.repo)]
tree = {}
for item in subprocess.check_output(git + ['ls-tree','-r','-z','--full-tree',commit], env=env).split(b'\0'):
    if item:
        meta, path = item.split(b'\t', 1)
        mode, kind, oid = meta.decode().split()
        tree[path.decode()] = (mode, kind, oid)
for row in rows:
    assert tree[row['path']] == (row['mode'], 'blob', row['git_blob'])
request = ''.join(row['git_blob'] + '\n' for row in rows).encode()
stream = subprocess.run(git + ['cat-file','--batch'], input=request, stdout=subprocess.PIPE, check=True, env=env).stdout
pos = 0
for row in rows:
    end = stream.index(b'\n', pos)
    oid, kind, size = stream[pos:end].decode().split()
    assert oid == row['git_blob'] and kind == 'blob' and int(size) == row['bytes']
    pos = end + 1
    data = stream[pos:pos + int(size)]
    assert len(data) == row['bytes'] and H(data) == row['sha256'] and stream[pos + len(data):pos + len(data) + 1] == b'\n'
    pos += len(data) + 1
assert pos == len(stream)
print('Compared all recorded staged Git blob bytes and executable modes with published checkpoint 31e.')
print('Recorded scans remain bounded; ciphertext opacity, independent custody and product behavior are not resolved.')
