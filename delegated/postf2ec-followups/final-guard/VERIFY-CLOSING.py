#!/usr/bin/env python3
"""Verify the separately captured closing evidence paths and scan receipt."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'CLOSING-CAPTURE.json').read_bytes());s=json.loads((p/'CLOSING-SCAN.json').read_bytes());assert c['schema']=='726-postf2ec-closing-staged-capture-v1' and c['unchanged_prior_paths']==299 and c['strict_diff_check_exit']==0 and c['prior_capture_sha256']==h((p/'RECEIPT-CAPTURE.json').read_bytes()) and len(c['new_rows'])==21
assert s['capture_sha256']==h((p/'CLOSING-CAPTURE.json').read_bytes()) and s['scanned_paths']==21 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
for row in c['new_rows']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;f=root.joinpath(*n.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(row['bytes'],row['sha256'],row['git_blob']) and bool(f.stat().st_mode&0o111)==(row['mode']=='100755')
print('Verified 21 closing evidence paths and bounded scan receipt; later closing receipt files remain separate.')
