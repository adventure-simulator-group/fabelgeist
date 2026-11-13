#!/usr/bin/env python3
"""Verify draft identity, published report bindings and selected 059 history."""
from pathlib import Path
import argparse
import hashlib
import io
import json
import subprocess
import sys
import tarfile

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
root = parser.parse_args().publication.resolve()
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda name: json.loads((p / name).read_bytes())
git = lambda *args: subprocess.check_output(['git', '-C', str(root), *args])
parent = 'b111f7444ab7e3b13b2099d5ce9d59f08faf9424'
assert git('rev-parse', parent + '^{tree}').decode().strip() == '84668f9ace2a06e346564f2185d5e57747403363'
record = load('RECORD.json')
assert record['schema'] == '726-postb111-reconciliation-evidence-v1'
assert record['parent_checkpoint'] == parent and len(record['files']) == 13
assert len({row['path'] for row in record['files']}) == 13
for row in record['files']:
    file = p / row['path']
    assert file.parent == p and file.is_file() and not file.is_symlink()
    data = file.read_bytes()
    assert (len(data), sha(data)) == (row['bytes'], row['sha256'])
assert {x.name for x in p.iterdir()} == {r['path'] for r in record['files']} | {
    'RECORD.json', 'README.md', 'VERIFY.py'}
drafts = [load('DRAFT-v' + str(i) + '.json') for i in range(1, 5)]
for i, draft in enumerate(drafts, 1):
    assert draft['status'] == 'draft for independent review; not adopted'
    assert draft['parent_checkpoint'] == parent
    assert draft['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
    assert draft['counts_if_adopted'] == drafts[0]['counts_if_adopted']
    if i > 1:
        correction = load('CORRECTION-v' + str(i) + '.json')
        assert correction['predecessor_sha256'] == sha((p / ('DRAFT-v' + str(i-1) + '.json')).read_bytes())
        assert correction['successor_sha256'] == sha((p / ('DRAFT-v' + str(i) + '.json')).read_bytes())
        assert correction['counts_changed'] is False
assert sha((p / 'DRAFT-v4.json').read_bytes()) == '08722589f73bfddd7df95e77e0183558b7fb2a9120b1a3bfa19d2e2d1c9854db'
indices = [0, 19, 2497, 2501, 2503, 2504, 2510, 2521, 2534]
assert [row['index'] for row in drafts[-1]['assessments']] == indices
assert len(drafts[-1]['proposed_ledger_relationships']) == 9
history = load('HISTORY-COMPARISON.json')
reg = load('HISTORY-REGISTRATION.json')
original = git('show', reg['historical_checkpoint_059_commit'] + ':' + reg['historical_queue_git_path'])
reconstructed = git('show', parent + ':baseline-059/queue.json')
assert sha(original) == history['original_queue_sha256'] == reg['historical_queue_sha256']
assert sha(reconstructed) == history['reconstructed_queue_sha256'] == reg['reconstructed_queue_sha256']
assert original != reconstructed and history['whole_stream_bytes_equal'] is False
a = json.loads(original)['entries']
b = json.loads(reconstructed)['entries']
assert len(a) == len(b) == 3713
assert [r['index'] for r in history['rows']] == indices
assert all(r['parsed_objects_equal'] and a[r['index']] == b[r['index']] for r in history['rows'])
capture = load('REPORT-CAPTURE.json')
assert len(capture['reports']) == 8 and capture['selected_queue_indices'] == indices
for row in capture['reports']:
    data = git('show', parent + ':' + row['archive'])
    assert sha(data) == row['archive_sha256']
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        matches = [member for member in archive if member.isfile() and member.name == row['member']]
        assert len(matches) == 1
        report = archive.extractfile(matches[0]).read()
    assert (len(report), sha(report)) == (row['report_bytes'], row['report_sha256'])
q0 = load('Q0-ORIGINAL-CAPTURE.json')
archive_bytes = git('show', parent + ':' + q0['archive'])
assert sha(archive_bytes) == q0['archive_sha256']
with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode='r:gz') as archive:
    report = archive.extractfile(q0['report_member']).read()
    manifest = json.load(archive.extractfile('q0-behavior-v2/MANIFEST.json'))
assert (len(report), sha(report)) == (q0['report_bytes'], q0['report_sha256'])
assert sha(report) == manifest['files']['REPORT.md']['sha256']
assert drafts[-1]['q0_separate_capture_sha256'] == sha((p / 'Q0-ORIGINAL-CAPTURE.json').read_bytes())
print('Verified exact draft chain, selected parsed 059 equality and separately '
      'captured report identities. The Git object store and restricted '
      'originals remain dependencies; source semantics, independent custody, '
      'checkpoint3821 recovery and product behavior are not established.')
