#!/usr/bin/env python3
"""Verify the provisional central source-reconciliation evidence packet."""
import argparse
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P = Path(__file__).resolve().parent
E = P / 'evidence'
H = lambda b: hashlib.sha256(b).hexdigest()
load = lambda p: json.loads(p.read_bytes())
ap = argparse.ArgumentParser()
ap.add_argument('--git-root', type=Path, help='repository containing the pinned source commit')
ap.add_argument('--parent-checkout', type=Path, help='checkout containing the predecessor evidence tree')
args = ap.parse_args()
manifest = load(P / 'MANIFEST.json')
assert manifest['schema'] == '726-next-15a-central-packet-v1'
rev = manifest['source_revision']
assert rev == '95e63628f9d70b8dd6e6fede64334696c6995009'
files = manifest['files']
actual = {f.relative_to(E).as_posix() for f in E.rglob('*') if f.is_file()}
assert all(not f.is_symlink() for f in E.rglob('*')) and actual == set(files)
for name, row in files.items():
    path = PurePosixPath(name)
    assert not path.is_absolute() and '..' not in path.parts
    f = E / path
    data = f.read_bytes()
    assert (len(data), H(data)) == (row['bytes'], row['sha256'])
    assert bool(f.stat().st_mode & 0o111) == bool(row['mode'] & 0o111)
reports = load(E / 'reports/REGISTRATION.json')
assert reports['source_revision'] == rev and reports['registration_before_central_display']
report_receipt = load(E / 'reports/READ-RECEIPT.json')
assert report_receipt['central_reader'] == '/root'
for row in reports['reports']:
    data = (E / 'reports' / row['capture']).read_bytes()
    assert (len(data), H(data)) == (row['bytes'], row['sha256'])
    assert any(x['logical'] == row['logical'] and x['sha256'] == row['sha256'] for x in report_receipt['reports'])
for group in ['source', 'source2']:
    reg = load(E / group / 'REGISTRATION.json')
    receipt = load(E / group / 'READ-RECEIPT.json')
    assert reg['source_revision'] == rev and reg['registered_before_display']
    for row in reg['rows']:
        data = (E / group / row['display']).read_bytes()
        assert (len(data), H(data)) == (row['display_bytes'], row['display_sha256'])
        assert any(x['label'] == row['label'] and x['display_sha256'] == row['display_sha256'] for x in receipt['rows'])
draft = load(E / 'reports/CENTRAL-DRAFT.json')
assert draft['source_revision'] == rev and draft['parent_published_checkpoint'] == 'PENDING_14D_PUBLICATION'
assert [x['queue_index'] for x in draft['assessments']] == [2341, 2342, 2352, 2353, 2354]
assert len(draft['ledger_relationships']) == 6 and draft['repair'] == 'not started' and draft['product_validation'] == 'not performed'
if args.parent_checkout:
    parent = args.parent_checkout
    packet = parent / 'delegated/post9e9e-followups'
    raw = {
        'q2341-42/REPORT.md': (packet / 'q2341-42/REPORT.md').read_bytes(),
        'q2354/evidence/data/ASSESSMENT.md': (packet / 'q2354/evidence/data/ASSESSMENT.md').read_bytes(),
    }
    with tarfile.open(packet / 'q2352-53/evidence.tar.gz', 'r:gz') as t:
        for member in ['q2352-q2353/REPORT.json', 'q2352-q2353/REPORT.md']:
            m = t.getmember(member)
            assert m.isfile()
            raw['q2352-53/' + member] = t.extractfile(m).read()
    for row in reports['reports']:
        assert raw[row['logical']] == (E / 'reports' / row['capture']).read_bytes()
    lookup = load(E / 'reports/LEDGER-SEARCH.json')
    baseline = (parent / 'BASELINE-95E-20261110.json').read_bytes()
    assert H(baseline) == lookup['ledger_snapshot_whole_file_sha256']
    ledger = json.loads(baseline)['newly_assessed_ledger']['entries']
    assert len(ledger) == lookup['ledger_count'] == 1072
    for name, pattern in lookup['patterns'].items():
        found = [{'index': i, 'id': x['id']} for i, x in enumerate(ledger)
                 if re.search(pattern, json.dumps(x, ensure_ascii=False), re.I)]
        assert found == lookup['matches'][name]
    for item in lookup['selected_full_rows']:
        assert item['row'] == ledger[item['index']]
    print('Compared registered reports and bounded ledger navigation with the supplied predecessor evidence tree.')
else:
    print('NOT REPEATED: predecessor report and ledger comparison requires --parent-checkout.')
if args.git_root:
    env = os.environ.copy()
    env.update({'GIT_NO_REPLACE_OBJECTS': '1', 'GIT_NO_LAZY_FETCH': '1'})
    for group in ['source', 'source2']:
        reg = load(E / group / 'REGISTRATION.json')
        for row in reg['rows']:
            ref = rev + ':' + row['path']
            raw = subprocess.check_output(['git', '-C', str(args.git_root), 'show', ref], env=env)
            oid = subprocess.check_output(['git', '-C', str(args.git_root), 'rev-parse', ref], env=env, text=True).strip()
            assert H(raw) == row['full_file_sha256'] and oid == row['git_blob']
            lines = raw.decode().splitlines(keepends=True)
            parts = []
            for lo, hi in row['ranges']:
                assert 1 <= lo <= hi <= len(lines)
                parts.append(f"--- {row['path']}:{lo}-{hi} ---\n")
                parts.extend(f'{i:5d} {lines[i-1]}' for i in range(lo, hi + 1))
            assert ''.join(parts).encode() == (E / group / row['display']).read_bytes()
    print('Reconstructed registered source displays from pinned Git blobs in the supplied repository.')
else:
    print('NOT REPEATED: source-to-Git comparison requires --git-root.')
print('Verified packet bytes, Git executable modes and qualified read receipts; original full filesystem modes are recorded but not repeated by Git.')
print('The draft is not applied, audit completeness and checkpoint3821 recovery are not established.')
