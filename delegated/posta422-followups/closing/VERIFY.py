#!/usr/bin/env python3
"""Verify the captured postguard evidence and its bounded scan receipt."""
from pathlib import Path,PurePosixPath
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;repo=p.parents[2];h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'CAPTURE.json').read_bytes());s=json.loads((p/'SCAN-32.json').read_bytes());assert c['schema']=='726-posta422-postguard-capture-v1' and c['parent_checkpoint']=='a4223bf23487b5872bc5efe5b272d40f859456f2' and c['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and len(c['rows'])==175 and c['prior_paths_matched']==143 and len(c['extra_rows'])==32 and s['capture_sha256']==h((p/'CAPTURE.json').read_bytes()) and s['scanned_paths']==32 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
assert len({r['path'] for r in c['rows']})==175
for r in c['rows']:
 q=PurePosixPath(r['path']);assert not q.is_absolute() and '..' not in q.parts;f=repo.joinpath(*q.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(repo);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(r['bytes'],r['sha256'],r['git_blob']) and bool(f.stat().st_mode&0o111)==(r['mode']=='100755')
print('Verified 175 captured identities and the separately bounded 32-path scan. This closing metadata and opaque ciphertext retain their recorded limits.')
