#!/usr/bin/env python3
"""Bind the current evidence checkpoint to its replayable application."""
import hashlib
import io
import json
import subprocess
import sys
import tarfile
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError('Evidence verification requires assertions')
root = Path(__file__).resolve().parent
stage = root / 'delegated/postd7a5-followups'
app = stage / 'application-v2'
sha = lambda data: hashlib.sha256(data).hexdigest()
subprocess.run([sys.executable, '-B', str(stage / 'application/VERIFY.py')], check=True)
subprocess.run([sys.executable, '-B', str(app / 'VERIFY.py')], check=True)
previous = json.loads((stage / 'application/RECORD.json').read_bytes())
assert json.loads((app / 'RECORD.json').read_bytes())['predecessor_record_sha256'] == sha((stage / 'application/RECORD.json').read_bytes())
record = json.loads((app / 'RECORD.json').read_bytes())
with tarfile.open(stage / 'application/APPLICATION-EVIDENCE.tar.gz', 'r:gz') as first, \
        tarfile.open(app / record['archive'], 'r:gz') as successor:
    names1 = {x.name for x in first.getmembers()}
    names2 = {x.name for x in successor.getmembers()}
    assert names1 == names2
    for name in names1 - {'after/CONTINUATION.md'}:
        assert first.extractfile(name).read() == successor.extractfile(name).read()
    old = first.extractfile('after/CONTINUATION.md').read().decode()
    new = successor.extractfile('after/CONTINUATION.md').read().decode()
    paragraph = ('`delegated/postd7a5-followups/INDEX.md` locates the independent\n'
                 'reviews, the successor wording correction and later pending packets.\n\n')
    assert new.count(paragraph) == 1 and new.replace(paragraph, '', 1) == old
with tarfile.open(app / record['archive'], 'r:gz') as archive:
    for name in ['BASELINE-95E-20261110.json', 'CONTINUATION-PROGRESS-95E.json',
                 'RECONCILIATION-059.json', 'CONTINUATION.md']:
        assert archive.extractfile('after/' + name).read() == (root / name).read_bytes()
    for member, path in [('inputs/report.json', root / 'ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json'),
                         ('inputs/pending.json', stage / 'PENDING-DELEGATES.json'),
                         ('inputs/main-observation.json', root / 'MAIN-RECHECK-20261113-0033.json')]:
        assert archive.extractfile(member).read() == path.read_bytes()
report_data = (root / 'ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json').read_bytes()
report = json.loads(report_data)
with tarfile.open(stage / 'central/CENTRAL-EVIDENCE.tar.gz', 'r:gz') as archive:
    draft_data = archive.extractfile('drafts/CONSOLIDATED-DRAFT.json').read()
draft = json.loads(draft_data)
assert sha(draft_data) == report['consolidated_draft_sha256']
for name in ['partial_queue_assessments', 'context_only_observations', 'ledger_relationships']:
    assert draft[name] == report[name]
old_change = (stage / 'SUCCESSOR-CHANGE.json').read_bytes()
correction = json.loads((stage / 'SUCCESSOR-CHANGE-v2.json').read_bytes())
assert correction['predecessor_sha256'] == sha(old_change)
assert correction['report_sha256'] == sha(report_data)
pending = json.loads((stage / 'PENDING-DELEGATES.json').read_bytes())
assert len(pending['packets']) == 20
for row in pending['packets']:
    assert (root / row['public_transport']).exists(), row['queue_index']
for group in ['transport', 'late-transport', 'batch-transport', 'central',
              'checkpoint-d7a5-recovery', 'dispatch-transport',
              'encrypted-originals', 'review-encrypted-originals',
              'postfreeze-packets', 'postfreeze-encrypted-originals',
              'navigation-encrypted-originals']:
    subprocess.run([sys.executable, '-B', str(stage / group / 'VERIFY.py')], check=True)
subprocess.run([sys.executable, '-B', str(stage / 'package-review/VERIFY.py'),
                '--stage', str(stage)], check=True)
subprocess.run([sys.executable, '-B', str(stage / 'application-review/VERIFY.py')], check=True)
subprocess.run([sys.executable, '-B', str(stage / 'navigation-review/VERIFY.py'),
                '--published', str(root)], check=True)
post = json.loads((stage / 'POSTFREEZE-PENDING.json').read_bytes())
assert [row['queue_indices'] for row in post['packets']] == [[3171, 3172, 3174, 3175], [3220, 3221, 3222, 3223]]
assert all((stage / row[key]).is_file() for row in post['packets'] for key in ['public_transport', 'receipt'])
assert (stage / 'INDEX.md').is_file()
print('Bound current checkpoint artifacts, corrected successor wording, pending '
      'pointers and source-only package checks. Restricted originals, same-machine '
      'key custody and audit closure remain separately qualified.')
