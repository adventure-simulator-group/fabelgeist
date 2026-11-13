#!/usr/bin/env python3
"""Verify the staged evidence snapshot and additive custody receipts."""
from pathlib import Path,PurePosixPath
import hashlib,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;follow=p.parent;repo=follow.parent.parent;h=lambda b:hashlib.sha256(b).hexdigest()
c=json.loads((p/'CAPTURE.json').read_bytes());s=json.loads((p/'EXTRA-SCAN.json').read_bytes());assert c['schema']=='726-posta422-staged-capture-v1' and c['parent_checkpoint']=='a4223bf23487b5872bc5efe5b272d40f859456f2' and c['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and len(c['rows'])==143 and c['frozen_paths_matched']==135 and len(c['extra_rows'])==8 and s['capture_sha256']==h((p/'CAPTURE.json').read_bytes()) and s['scanned_paths']==8 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
assert len({r['path'] for r in c['rows']})==143
for r in c['rows']:
 q=PurePosixPath(r['path']);assert not q.is_absolute() and '..' not in q.parts;f=repo.joinpath(*q.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(repo);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(r['bytes'],r['sha256'],r['git_blob']) and bool(f.stat().st_mode&0o111)==(r['mode']=='100755')
status=json.loads((p/'STATUS-QUALIFICATION.json').read_bytes());assert status['schema']=='726-posta422-status-scope-addendum-v1' and status['q10_v2_structural_review_public_manifest_sha256']=='491475d39faed33f34351db149b9be800e1f00f51729512b5bc06a4a63b82d7c'
scope=json.loads((follow/'proposals/VERIFY-STATUS-SCOPE.json').read_bytes());assert scope['verifier_sha256']==h((follow/'proposals/VERIFY-STATUS.py').read_bytes())
main=json.loads((p/'MAIN.json').read_bytes());assert main['source_revision']==c['source_revision'] and main['source_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
for cmd in [[follow/'frozen/VERIFY.py'],[follow/'VERIFY-ADDENDUM.py','--publication',repo],[follow/'late-independent/transport/VERIFY.py'],[follow/'late-independent-receipt/transport/VERIFY.py'],[follow/'final-encrypted/VERIFY.py']]:
 r=subprocess.run([sys.executable,'-B',*[str(x) for x in cmd]],cwd=repo);assert r.returncode==0
receipt=json.loads((p/'RESTORATION.json').read_bytes());assert receipt['restore_exit']==0 and receipt['selected_replay_exits']==[0,0,0]
print('Verified 143 captured staged identities, separate bounded scans and additive public/ciphertext identities. The main observation, custody, chronology, later additions and audit closure retain their recorded limits.')
