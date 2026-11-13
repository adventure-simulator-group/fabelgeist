#!/usr/bin/env python3
"""Verify separately captured guard files and bounded-scan receipt."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'TAIL-CAPTURE.json').read_bytes());s=json.loads((p/'TAIL-SCAN.json').read_bytes());assert c['schema']=='726-postf2ec-tail-staged-capture-v1' and c['unchanged_prior_paths']==265 and c['strict_diff_check_exit']==0 and c['prior_capture_sha256']==h((p/'LATE-CAPTURE.json').read_bytes()) and len(c['new_rows'])==22
assert s['capture_sha256']==h((p/'TAIL-CAPTURE.json').read_bytes()) and s['scanned_paths']==22 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
for row in c['new_rows']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;f=root.joinpath(*n.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(row['bytes'],row['sha256'],row['git_blob']) and bool(f.stat().st_mode&0o111)==(row['mode']=='100755')
r=json.loads((p/'RESTORATION.json').read_bytes());assert r['restore_exit']==r['q0_evidence_replay_exit']==0 and r['collections']==4 and r['matched_file_bytes_and_modes']==360 and r['matched_directory_modes']==15 and r['main_raw_sha256_matched']
print('Verified 22 separately captured guard paths, scan and authenticated restoration receipts; later receipt files remain separate.')
