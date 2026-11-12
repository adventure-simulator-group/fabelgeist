#!/usr/bin/env python3
"""Verify the fixed post-31e1 evidence snapshot, not product behavior."""
import collections
import gzip
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
R = Path(__file__).resolve().parent
D = R / 'delegated'
P = D / 'post31e1-followups'
H = lambda b: hashlib.sha256(b).hexdigest()
load = lambda p: json.loads(p.read_bytes())
raw = (P / 'CATALOG.json').read_bytes()
assert H(raw) == 'baacfc6031c54a0d90689482b90ca2a90fbd79fa5013cb4f79c95b4c356c00a7'
catalog = json.loads(raw)
assert catalog['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert catalog['parent_checkpoint'] == '31e130bfa8f8914f759c7d5cddc3db92962704c7'
manifest = load(P / 'CONTENTS-MANIFEST.json')['files']
scan = D / 'post31e1-preimport-scan'
assert H((scan / 'RECORD.json').read_bytes()) == catalog['preimport_scan_record_sha256']
sr = load(scan / 'RECORD.json')
assert sr['digest_exit'] == sr['pattern_exit'] == 0
assert H((scan / 'digest.json').read_bytes()) == sr['digest_sha256']
assert H((scan / 'pattern.log').read_bytes()) == sr['pattern_sha256']
digest = load(scan / 'digest.json')
assert not digest['forbidden_digest_total'] and not digest['other_candidate_counts'] and not digest['exceptions']
assert b'issues 0' in (scan / 'pattern.log').read_bytes()
aggregate = hashlib.sha256()
for name in sorted(manifest, key=lambda n: PurePosixPath(n).parts):
    aggregate.update(name.encode() + b'\0' + manifest[name]['sha256'].encode() + b'\n')
assert len(manifest) == sr['files'] == 1136 and aggregate.hexdigest() == sr['aggregate_names_and_sha256']
assert {x.name for x in scan.iterdir()} == {'RECORD.json','digest.json','pattern.log'}
standalone = D / 'post31e1-standalone'
assert H((standalone / 'RESULT.json').read_bytes()) == 'd57953b5a7cc94824b369a7cd53a5fc97f8a5a7c72df29d53026ff0b793c5bbd'
st = load(standalone / 'RESULT.json')
assert st['exit'] == 0 and st['catalog_sha256'] == H(raw)
assert (standalone / 'VERIFIER.py.txt').read_bytes() == (P / 'VERIFY-EVIDENCE.py').read_bytes()
assert H((standalone / 'VERIFIER.py.txt').read_bytes()) == st['verifier_sha256']
for name,key in [('STDOUT.log','stdout'),('STDERR.log','stderr')]:
    data = (standalone / name).read_bytes()
    assert (len(data), H(data)) == (st[key + '_bytes'], st[key + '_sha256'])
assert st['stderr_bytes'] == 0 and {x.name for x in standalone.iterdir()} == {'RESULT.json','VERIFIER.py.txt','STDOUT.log','STDERR.log'}
imp = D / 'post31e1-import'
assert H((imp / 'RECORD.json').read_bytes()) == 'c090c216956260d63d04d2594010a28df06920711ebdb0378f681719f48d8bdc'
ir = load(imp / 'RECORD.json')
assert ir['catalog_sha256'] == H(raw) and H((imp / 'IMPORT.py.txt').read_bytes()) == ir['script_sha256']
assert [(x['target'],x['files']) for x in ir['copies']] == [('delegated/post31e1-followups',1138),('delegated/post31e1-preimport-scan',3),('delegated/post31e1-standalone',4)]
assert {x.name for x in imp.iterdir()} == {'RECORD.json','IMPORT.py.txt'}
application = D / 'post31e1-central-application'
assert H((application / 'RECORD.json').read_bytes()) == 'd7f2c4471e69b06c735987e1bb1136d64ba38076e69a4c9fdae245fe80013125'
ar = load(application / 'RECORD.json')
assert ar['exit'] == 0 and H((application / 'apply.py.txt').read_bytes()) == ar['script_sha256']
assert H((application / 'OUTPUT.log').read_bytes()) == ar['output_sha256']
assert {x.name for x in application.iterdir()} == {'RECORD.json','apply.py.txt','OUTPUT.log'}
treatment = D / 'post31e1-diff-treatment'
assert H((treatment / 'RECORD.json').read_bytes()) == 'c98eb588abeba3797defe63dc29959ac3521337fe171e431e52603df2d73106e'
tr = load(treatment / 'RECORD.json')
compressed = (treatment / 'INITIAL-DIFF-CHECK.log.gz').read_bytes()
assert H(compressed) == tr['compressed_stdout_sha256']
initial = gzip.decompress(compressed)
assert (len(initial), H(initial)) == (tr['initial_stdout_bytes'], tr['initial_stdout_sha256'])
headers = [m.groups() for line in initial.decode().splitlines()
           if (m := re.match(r'^(.*?):([0-9]+): (.*)$', line))]
assert len(headers) == tr['diagnostic_count'] == 636
assert dict(collections.Counter(row[2] for row in headers)) == tr['diagnostic_types']
names = sorted({row[0] for row in headers})
assert names == tr['affected_paths'] and len(names) == tr['affected_file_count'] == 114
append = ''.join(name + ' binary\n' for name in names).encode()
attributes = (R / '.gitattributes').read_bytes()
offset = attributes.find(append)
assert offset >= 0 and H(attributes[:offset]) == tr['gitattributes_before_sha256']
assert H(attributes[:offset + len(append)]) == tr['gitattributes_after_sha256']
assert {p.name for p in treatment.iterdir()} == {'RECORD.json','INITIAL-DIFF-CHECK.log.gz'}
snapshot_bytes = (R / 'FOLLOWUP-POST31E1-EVIDENCE.json').read_bytes()
assert H(snapshot_bytes) == '5a451b3765222e67e767e65d423ebead5f33df35d6242ed452b99f53837adfae'
snapshot = json.loads(snapshot_bytes)
counts = snapshot['observed_counts']
assert (counts['lenses'],counts['lenses_complete'],counts['queue_records'],counts['queue_semantically_reconciled'],counts['current_ledger_entries_and_relationships']) == (13,0,3713,0,1078)
report_bytes = (R / 'ASSESSMENT-CENTRAL-FOLLOWUPS-15A-95E.json').read_bytes()
assert H(report_bytes) == snapshot['central_report_sha256'] == catalog['central_report_sha256']
report = json.loads(report_bytes)
assert report['parent_published_checkpoint'] == catalog['parent_checkpoint'] and len(report['assessments']) == 5 and len(report['ledger_relationships']) == 6
assert snapshot['bundle_catalog_sha256'] == H(raw)
assert snapshot['checkpoint_31e1_recovery_record_sha256'] == catalog['recovery_record_sha256']
assert snapshot['encrypted_manifest_sha256'] == catalog['encrypted_manifest_sha256']
assert snapshot['privacy_correction_record_sha256'] == catalog['privacy_correction_record_sha256']
assert H((R / 'MAIN-RECHECK-20261112-1926.json').read_bytes()) == snapshot['main_observation_sha256']
ledger = load(R / 'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in snapshot['added_ledger_relationship_ids']:
    found = [x for x in ledger if x.get('id') == ident]
    assert len(found) == 1 and found[0]['record'] == 'ASSESSMENT-CENTRAL-FOLLOWUPS-15A-95E.json'
queue = load(R / 'RECONCILIATION-059.json')['entries']
for row in snapshot['queue_selections']:
    assert queue[row['index']]['contract_id'] == row['contract_id']
    assert row['current_reconciliation_at_snapshot'].startswith('partial')
env = os.environ.copy()
for name in ['PYTHONOPTIMIZE','PYTHONPATH','PYTHONHOME','PYTHONSTARTUP','PYTHONINSPECT','PYTHONUSERBASE']:
    env.pop(name, None)
env.update({'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','GIT_NO_REPLACE_OBJECTS':'1','GIT_NO_LAZY_FETCH':'1'})
subprocess.run([sys.executable,'-B',str(P / 'VERIFY-EVIDENCE.py')],cwd=R,env=env,check=True)
print('Bound five partial assessments, six planning relationships, eight pending delegated families and same-machine 31e recovery.')
print('Private-key and external-anchor checks are represented by same-machine receipts; optional checks are not repeated here.')
print('Ciphertext is opaque to scans; no independent custody, product behavior, checkpoint3821 recovery or audit completion is asserted.')
