#!/usr/bin/env python3
"""Verify captured public bytes and bounded scans without treating modes as Git data."""
from pathlib import Path,PurePosixPath
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;root=p.parent;h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'CAPTURE.json').read_bytes());s=json.loads((p/'SCAN.json').read_bytes());assert c['schema']=='726-posta422-frozen-public-capture-v1' and len(c['rows'])==135 and s['capture_sha256']==h((p/'CAPTURE.json').read_bytes()) and s['scanned_paths']==135 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
for r in c['rows']:
 q=PurePosixPath(r['path']);assert not q.is_absolute() and '..' not in q.parts;f=root.joinpath(*q.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(r['bytes'],r['sha256'],r['git_blob']) and bool(f.stat().st_mode&0o111)==bool(int(r['mode'],8)&0o111)
print('Verified 135 captured public file identities, executable bits and bounded scan. Git does not preserve other permission bits or directory modes; archived originals retain their own modes. Later metadata and opaque ciphertext remain separately qualified.')
