#!/usr/bin/env python3
"""Verify the receipt snapshot and immediate prepublication main guard."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'RECEIPT-CAPTURE.json').read_bytes());s=json.loads((p/'RECEIPT-SCAN.json').read_bytes());m=json.loads((p/'PREPUSH-MAIN.json').read_bytes());r=json.loads((p/'PREPUSH-RESTORATION.json').read_bytes());assert c['schema']=='726-postf2ec-receipt-staged-capture-v1' and c['unchanged_prior_paths']==287 and c['strict_diff_check_exit']==0 and c['prior_capture_sha256']==h((p/'TAIL-CAPTURE.json').read_bytes()) and len(c['new_rows'])==12
assert s['capture_sha256']==h((p/'RECEIPT-CAPTURE.json').read_bytes()) and s['scanned_paths']==12 and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
for row in c['new_rows']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;f=root.joinpath(*n.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(row['bytes'],row['sha256'],row['git_blob']) and bool(f.stat().st_mode&0o111)==(row['mode']=='100755')
assert m['source_commit']=='95e63628f9d70b8dd6e6fede64334696c6995009' and m['source_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f' and m['method']=='GitHub connector fetch branches/main API' and m['before_utc']<=m['after_utc'] and len(m['raw_serialized_response_sha256'])==64
assert r['encrypt_exit']==r['restore_exit']==0 and r['matched_file_bytes_and_modes']==15 and r['matched_directory_modes']==4 and r['raw_main_matched']
subprocess.run([sys.executable,'-B',str(p.parent/'prepush-encrypted/VERIFY.py')],check=True)
print('Verified 12 receipt paths, their bounded scan, the latest main-observation metadata and same-machine custody receipt; later closing metadata remains separate.')
