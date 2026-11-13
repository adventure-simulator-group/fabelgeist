#!/usr/bin/env python3
"""Replay the evidence-only application and optional predecessor checks."""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import io
import json
import os
import subprocess
import sys
import tarfile

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
from TRANSFORM import NAMES, REPORT, MAIN, PARENT, DRAFT_SHA256, transform

p = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
parser.add_argument('--candidate', type=Path)
parser.add_argument('--predecessor', type=Path)
args = parser.parse_args()
root = args.publication.resolve()
candidate = args.candidate.resolve() if args.candidate else None
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda file: json.loads(file.read_bytes())
git = lambda *cmd: subprocess.check_output(['git', '-C', str(root), *cmd])
env = dict(os.environ)
env.pop('PYTHONOPTIMIZE', None)
env['PYTHONNOUSERSITE'] = '1'
env['PYTHONDONTWRITEBYTECODE'] = '1'
record = load(p / 'RECORD.json')
assert record['schema'] == '726-postb111-application-archive-v1'
assert record['parent_checkpoint'] == PARENT
assert record['parent_tree'] == '84668f9ace2a06e346564f2185d5e57747403363'
assert git('rev-parse', PARENT + '^{tree}').decode().strip() == record['parent_tree']
assert sha((p / 'TRANSFORM.py').read_bytes()) == record['transform_sha256']
raw = (p / record['archive']).read_bytes()
assert (len(raw), sha(raw)) == (record['archive_bytes'], record['archive_sha256'])
wanted = {row['path']: row for row in record['members']}
assert len(wanted) == len(record['members']) == 17
data = {}
with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
    for member in archive:
        name = PurePosixPath(member.name)
        assert not name.is_absolute() and '..' not in name.parts
        assert member.isfile() and member.name in wanted and member.name not in data
        body = archive.extractfile(member).read()
        row = wanted[member.name]
        assert (len(body), sha(body), oct(member.mode)) == (row['bytes'], row['sha256'], row['mode'])
        data[member.name] = body
assert set(data) == set(wanted)
inputs = json.loads(data['inputs/input-identities.json'])
assert inputs['parent_checkpoint'] == PARENT and inputs['parent_tree'] == record['parent_tree']
assert len(inputs['before']) == 4 and inputs['draft_sha256'] == DRAFT_SHA256
before = {name: data['before/' + name] for name in NAMES}
for row in inputs['before']:
    body = git('show', PARENT + ':' + row['path'])
    assert body == before[row['path']]
    assert (len(body), sha(body), git('rev-parse', PARENT + ':' + row['path']).decode().strip()) == (
        row['bytes'], row['sha256'], row['git_blob'])
draft = data['inputs/draft-v5.json']
report = json.loads(data['inputs/report.json'])
observation = json.loads(data['inputs/main-observation.json'])
assert sha(draft) == DRAFT_SHA256 == report['adopted_from_draft_sha256']
assert data['inputs/draft-v5.json'] == (p.parent / 'reconciliation-v5/DRAFT-v5.json').read_bytes()
assert data['inputs/draft-correction-v5.json'] == (p.parent / 'reconciliation-v5/CORRECTION-v5.json').read_bytes()
assert data['inputs/q0-capture.json'] == (p.parent / 'reconciliation/Q0-ORIGINAL-CAPTURE.json').read_bytes()
assert inputs['main_observation_sha256'] == sha(data['inputs/main-observation.json'])
for number in (2, 3, 4):
    correction = json.loads(data['inputs/transform-correction-v' + str(number) + '.json'])
    assert correction['schema'] == '726-postb111-transform-correction-v' + str(number)
after = transform(before, draft, report, observation)
assert all(after[name] == data['after/' + name] for name in NAMES)
construction = load(p / 'ADOPTION-CANDIDATE.json')
assert construction['status'] == 'candidate only; not installed or published'
assert construction['parent_checkpoint'] == PARENT
assert construction['reviewed_draft_sha256'] == sha(draft)
assert construction['transform_sha256'] == sha((p / 'TRANSFORM.py').read_bytes())
assert construction['report_sha256'] == sha(data['inputs/report.json'])
assert construction['main_observation_sha256'] == sha(data['inputs/main-observation.json'])
for name in NAMES:
    file = candidate / 'after' / name if candidate else root / name
    body = file.read_bytes()
    assert body == after[name]
    assert (len(body), sha(body)) == (construction['after'][name]['bytes'],
                                      construction['after'][name]['sha256'])
for name, member in [(REPORT, 'inputs/report.json'), (MAIN, 'inputs/main-observation.json')]:
    file = candidate / 'root' / name if candidate else root / name
    assert file.is_file() and not file.is_symlink() and file.read_bytes() == data[member]
assert (p / 'README.md').is_file()
for assessment in report['assessments']:
    for path in assessment['evidence']:
        git('cat-file', '-e', PARENT + ':' + path)
for group in ('review-transport', 'navigation-review-transport'):
    subprocess.run([sys.executable, '-B', str(p.parent / group / 'VERIFY.py')],
                   cwd=root, env=env, check=True)
review_groups = {
    'reconciliation': ('review-transport', 'reconciliation-review', 'reconciliation-receipt'),
    'organization': ('review-transport', 'organization-review', 'organization-receipt'),
    'navigation': ('navigation-review-transport', 'navigation-review', 'navigation-receipt'),
}
for key, (directory, public_label, receipt_label) in review_groups.items():
    manifest = load(p.parent / directory / 'MANIFEST.json')
    rows = {row['label']: row for row in manifest['bundles']}
    ref = report['independent_reviews'][key]
    assert rows[public_label]['source_manifest_sha256'] == ref['public_manifest_sha256']
    assert rows[receipt_label]['source_manifest_sha256'] == ref['receipt_manifest_sha256']
    for label, expected in [(public_label, ref['public_manifest_sha256']),
                            (receipt_label, ref['receipt_manifest_sha256'])]:
        with tarfile.open(p.parent / directory / rows[label]['file'], mode='r:gz') as archive:
            manifest_bytes = archive.extractfile(rows[label]['manifest_member']).read()
        assert sha(manifest_bytes) == expected
history = report['historical_provenance']
assert sha((root / 'baseline-059/queue.json').read_bytes()) == history['reconstructed_queue_sha256']
assert sha((root / 'baseline-059/ledger.json').read_bytes()) == history['reconstructed_ledger_sha256']
original = load(root / 'BASELINE-20261110.json')['historical_checkpoint_059']
assert (original['queue_sha256'], original['ledger_sha256']) == (
    history['original_queue_sha256'], history['original_ledger_sha256'])
if args.predecessor:
    prior = args.predecessor.resolve()
    assert prior != root
    assert subprocess.check_output(['git', '-C', str(prior), 'rev-parse', 'HEAD']).decode().strip() == PARENT
    assert subprocess.check_output(['git', '-C', str(prior), 'rev-parse', 'HEAD^{tree}']).decode().strip() == record['parent_tree']
    assert not subprocess.check_output(['git', '-C', str(prior), 'status', '--porcelain=v1', '-z'])
    for cmd in [
        ['verify_post5f6a_checkpoint.py'],
        ['delegated/post5f6a-followups/final-guard/VERIFY.py', '--publication', str(prior)],
        ['delegated/post5f6a-followups/closing/VERIFY.py'],
    ]:
        subprocess.run([sys.executable, '-B', *cmd], cwd=prior, env=env, check=True)
    print('Replayed three predecessor evidence checks against the exact clean b111 checkout.')
else:
    print('Predecessor replay not requested; the same-machine receipt remains separately available.')
print('Verified exact application images and all three review identities. '
      'This structural replay does not independently establish factual '
      'prose, independent custody, semantic closure or product behavior.')
